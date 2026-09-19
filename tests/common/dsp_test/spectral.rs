use std::path::Path;
use std::time::Instant;

use plotters::prelude::*;
use rustfft::{FftPlanner, num_complex::Complex};

use dsp_rs::processor::Processor;

use crate::common::dsp_test::common::format_with_commas;

fn unwrap_phase_degrees(phases: &mut [f32]) {
    let mut offset = 0.0f32;
    for i in 1..phases.len() {
        let diff = (phases[i] + offset) - phases[i - 1];
        if diff > 180.0 {
            let cycles = ((diff + 180.0) / 360.0).floor();
            offset -= cycles * 360.0;
        } else if diff < -180.0 {
            let cycles = ((-diff + 180.0) / 360.0).floor();
            offset += cycles * 360.0;
        }
        phases[i] += offset;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MagnitudeScale {
    Linear,
    Decibel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrequencyScale {
    Linear,
    Log,
}

pub fn response<P>(processor: P, sample_rate: f32) -> SpectralResponse<P>
where
    P: Processor<f32>,
{
    spectral_response(processor, sample_rate)
}

pub struct SpectralResponse<P: Processor<f32>> {
    processor: P,

    sample_rate: f32,
    samples: usize,

    min_freq: f32,
    max_freq: f32,

    min_magnitude: Option<f32>,
    max_magnitude: Option<f32>,

    magnitude_scale: MagnitudeScale,
    frequency_scale: FrequencyScale,

    title: String,
    show_info: bool,
    benchmark_iterations: Option<usize>,

    show_phase: bool,
    min_phase: Option<f32>,
    max_phase: Option<f32>,
}

pub fn spectral_response<P>(processor: P, sample_rate: f32) -> SpectralResponse<P>
where
    P: Processor<f32>,
{
    SpectralResponse {
        processor,

        sample_rate,
        samples: 16_384,

        min_freq: 20.0,
        max_freq: 20_000.0,

        min_magnitude: None,
        max_magnitude: None,

        magnitude_scale: MagnitudeScale::Linear,
        frequency_scale: FrequencyScale::Linear,

        title: "Spectral Response".to_string(),
        show_info: false,
        benchmark_iterations: None,

        show_phase: false,
        min_phase: None,
        max_phase: None,
    }
}

#[allow(dead_code)]
impl<P> SpectralResponse<P>
where
    P: Processor<f32>,
{
    pub fn samples(mut self, samples: usize) -> Self {
        self.samples = samples;
        self
    }

    pub fn db(mut self) -> Self {
        self.magnitude_scale = MagnitudeScale::Decibel;
        self
    }

    pub fn linear_magnitude(mut self) -> Self {
        self.magnitude_scale = MagnitudeScale::Linear;
        self
    }

    pub fn log(mut self) -> Self {
        self.frequency_scale = FrequencyScale::Log;
        self
    }

    pub fn linear_frequency(mut self) -> Self {
        self.frequency_scale = FrequencyScale::Linear;
        self
    }

    pub fn frequency_range(mut self, min: f32, max: f32) -> Self {
        self.min_freq = min;
        self.max_freq = max;
        self
    }

    pub fn magnitude_range(mut self, min: f32, max: f32) -> Self {
        self.min_magnitude = Some(min);
        self.max_magnitude = Some(max);
        self
    }

    pub fn show_phase(mut self) -> Self {
        self.show_phase = true;
        self
    }

    pub fn phase_range(mut self, min: f32, max: f32) -> Self {
        assert!(min < max, "min_phase must be less than max_phase");
        self.show_phase = true;
        self.min_phase = Some(min);
        self.max_phase = Some(max);
        self
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn with_info(mut self) -> Self {
        self.show_info = true;
        self
    }

    pub fn benchmark(mut self) -> Self {
        self.show_info = true;
        self.benchmark_iterations = Some(10);
        self
    }

    pub fn benchmark_iterations(mut self, iters: usize) -> Self {
        self.show_info = true;
        self.benchmark_iterations = Some(iters.max(1));
        self
    }

    pub fn save(mut self, path: impl AsRef<Path>) {
        assert!(self.samples > 0, "samples must be greater than 0");
        assert!(self.sample_rate > 0.0, "sample_rate must be positive");
        assert!(self.min_freq >= 0.0, "min_freq must be non-negative");
        assert!(
            self.max_freq > self.min_freq,
            "max_freq must be greater than min_freq"
        );
        assert!(
            self.max_freq <= self.sample_rate / 2.0,
            "max_freq cannot exceed Nyquist frequency (sample_rate / 2)"
        );

        if matches!(self.frequency_scale, FrequencyScale::Log) {
            assert!(
                self.min_freq > 0.0,
                "log-frequency plots require min_freq > 0"
            );
        }

        if let (Some(min), Some(max)) = (self.min_magnitude, self.max_magnitude) {
            assert!(min < max, "min_magnitude must be less than max_magnitude");
        }

        // 1. Record impulse response from the initial processor state
        let mut signal: Vec<Complex<f32>> = (0..self.samples)
            .map(|i| {
                let input = if i == 0 { 1.0 } else { 0.0 };
                let output = self.processor.process_sample(input);

                Complex::new(output, 0.0)
            })
            .collect();

        // 2. Run performance benchmark if requested (post-impulse response)
        let bench_result = self.benchmark_iterations.map(|iters| {
            let iters = iters.max(1);
            let mut bench_buf = vec![0.0f32; self.samples];
            bench_buf[0] = 1.0;

            // Warmup iterations
            for _ in 0..2 {
                self.processor.process_buffer(&mut bench_buf);
            }

            let bench_start = Instant::now();
            for _ in 0..iters {
                self.processor.process_buffer(&mut bench_buf);
            }
            let bench_elapsed = bench_start.elapsed();
            let avg_duration = bench_elapsed / (iters as u32);
            (iters, avg_duration)
        });

        // 3. FFT computation
        let mut planner = FftPlanner::<f32>::new();
        let fft = planner.plan_fft_forward(self.samples);

        fft.process(&mut signal);

        let response: Vec<(f32, f32, f32)> = signal
            .iter()
            .take(self.samples / 2 + 1)
            .enumerate()
            .filter_map(|(k, value)| {
                let frequency = k as f32 * self.sample_rate / self.samples as f32;

                if frequency < self.min_freq || frequency > self.max_freq {
                    return None;
                }

                let magnitude = value.norm();
                let phase_deg = value.im.atan2(value.re).to_degrees();
                Some((frequency, magnitude, phase_deg))
            })
            .collect();

        let mut unwrapped_phases: Vec<f32> = response.iter().map(|&(_, _, p)| p).collect();
        unwrap_phase_degrees(&mut unwrapped_phases);

        let plot_data: Vec<(f32, f32)> = response
            .iter()
            .map(|&(frequency, magnitude, _)| {
                let y = match self.magnitude_scale {
                    MagnitudeScale::Linear => magnitude,
                    MagnitudeScale::Decibel => {
                        if magnitude.is_nan() {
                            f32::NAN
                        } else {
                            20.0 * magnitude.max(1e-10).log10()
                        }
                    }
                };

                (frequency, y)
            })
            .collect();

        let phase_plot_data: Vec<(f32, f32)> = response
            .iter()
            .zip(unwrapped_phases.iter())
            .map(|(&(frequency, _, _), &phase)| (frequency, phase))
            .collect();

        let y_range = match (self.min_magnitude, self.max_magnitude) {
            (Some(min), Some(max)) => min..max,
            _ => match self.magnitude_scale {
                MagnitudeScale::Linear => 0.0..1.1,
                MagnitudeScale::Decibel => -80.0..10.0,
            },
        };

        let phase_range = match (self.min_phase, self.max_phase) {
            (Some(min), Some(max)) => min..max,
            _ => -180.0..180.0,
        };

        let path = path.as_ref();
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent).expect("failed to create plot output directory");
        }

        // Format metadata lines if enabled
        let signal_duration_secs = self.samples as f64 / self.sample_rate as f64;
        let signal_info = if self.show_info {
            let sample_rate_str = if self.sample_rate.fract() == 0.0 {
                format!("{} Hz", format_with_commas(self.sample_rate as usize))
            } else {
                format!("{:.1} Hz", self.sample_rate)
            };

            let samples_str = format_with_commas(self.samples);

            let duration_str = if signal_duration_secs >= 1.0 {
                format!("{:.2} s", signal_duration_secs)
            } else if signal_duration_secs >= 0.001 {
                format!("{:.1} ms", signal_duration_secs * 1000.0)
            } else {
                format!("{:.1} µs", signal_duration_secs * 1_000_000.0)
            };

            Some(format!(
                "Sample Rate: {}   |   Samples: {}   |   Duration: {}",
                sample_rate_str, samples_str, duration_str
            ))
        } else {
            None
        };

        let perf_info = bench_result.map(|(iters, avg_duration)| {
            let avg_secs = avg_duration.as_secs_f64().max(1e-12);

            let avg_time_str = if avg_secs < 1e-6 {
                format!("{:.1} ns", avg_secs * 1e9)
            } else if avg_secs < 1e-3 {
                format!("{:.1} µs", avg_secs * 1e6)
            } else if avg_secs < 1.0 {
                format!("{:.2} ms", avg_secs * 1e3)
            } else {
                format!("{:.2} s", avg_secs)
            };

            let throughput = (self.samples as f64) / avg_secs;
            let ns_per_sample = (avg_secs * 1e9) / (self.samples as f64);
            let throughput_str = if throughput >= 1e9 {
                format!("{:.2} GSa/s ({:.2} ns/sa)", throughput / 1e9, ns_per_sample)
            } else if throughput >= 1e6 {
                format!("{:.1} MSa/s ({:.2} ns/sa)", throughput / 1e6, ns_per_sample)
            } else if throughput >= 1e3 {
                format!("{:.1} kSa/s ({:.2} ns/sa)", throughput / 1e3, ns_per_sample)
            } else {
                format!("{:.0} Sa/s ({:.2} ns/sa)", throughput, ns_per_sample)
            };

            let realtime_factor = signal_duration_secs / avg_secs;
            let rt_str = if realtime_factor >= 1000.0 {
                format!(
                    "{}× real-time",
                    format_with_commas(realtime_factor.round() as usize)
                )
            } else if realtime_factor >= 10.0 {
                format!("{:.1}× real-time", realtime_factor)
            } else {
                format!("{:.2}× real-time", realtime_factor)
            };

            format!(
                "Avg Process Time ({} iters): {}   |   Throughput: {}   |   {}",
                iters, avg_time_str, throughput_str, rt_str
            )
        });

        let canvas_height = if self.show_info {
            if perf_info.is_some() { 650 } else { 620 }
        } else {
            600
        };

        let root = BitMapBackend::new(path, (1000, canvas_height)).into_drawing_area();
        root.fill(&WHITE)
            .expect("failed to initialize drawing area");

        let y_desc = match self.magnitude_scale {
            MagnitudeScale::Linear => "Magnitude",
            MagnitudeScale::Decibel => "Magnitude (dB)",
        };

        // Render header and prepare chart area
        let (chart_area, caption_title) = if self.show_info {
            let header_height = if perf_info.is_some() { 80 } else { 55 };
            let (header_area, chart_area) = root.split_vertically(header_height);

            let center_pos = plotters::style::text_anchor::Pos::new(
                plotters::style::text_anchor::HPos::Center,
                plotters::style::text_anchor::VPos::Top,
            );

            let title_style = ("sans-serif", 24)
                .into_font()
                .into_text_style(&header_area)
                .pos(center_pos);

            let subtitle_style = ("sans-serif", 13)
                .into_font()
                .color(&RGBColor(90, 90, 90))
                .into_text_style(&header_area)
                .pos(center_pos);

            header_area
                .draw_text(&self.title, &title_style, (500, 10))
                .expect("failed to draw title");

            if let Some(info) = &signal_info {
                header_area
                    .draw_text(info, &subtitle_style, (500, 40))
                    .expect("failed to draw signal info");
            }

            if let Some(perf) = &perf_info {
                header_area
                    .draw_text(perf, &subtitle_style, (500, 58))
                    .expect("failed to draw perf info");
            }

            (chart_area, None)
        } else {
            (root.clone(), Some(self.title.as_str()))
        };

        match (self.frequency_scale, self.show_phase) {
            (FrequencyScale::Linear, false) => {
                let mut builder = ChartBuilder::on(&chart_area);
                builder
                    .margin(20)
                    .x_label_area_size(60)
                    .y_label_area_size(70);
                if let Some(cap) = caption_title {
                    builder.caption(cap, ("sans-serif", 30));
                }

                let mut chart = builder
                    .build_cartesian_2d(self.min_freq..self.max_freq, y_range)
                    .expect("failed to create chart");

                chart
                    .configure_mesh()
                    .x_desc("Frequency (Hz)")
                    .y_desc(y_desc)
                    .x_label_formatter(&|x| format!("{x:.0}"))
                    .draw()
                    .expect("failed to draw chart axes");

                chart
                    .draw_series(LineSeries::new(plot_data, &BLUE))
                    .expect("failed to draw frequency response");
            }
            (FrequencyScale::Linear, true) => {
                let mut builder = ChartBuilder::on(&chart_area);
                builder
                    .margin(20)
                    .x_label_area_size(60)
                    .y_label_area_size(70)
                    .right_y_label_area_size(70);
                if let Some(cap) = caption_title {
                    builder.caption(cap, ("sans-serif", 30));
                }

                let mut chart = builder
                    .build_cartesian_2d(self.min_freq..self.max_freq, y_range)
                    .expect("failed to create chart")
                    .set_secondary_coord(self.min_freq..self.max_freq, phase_range);

                chart
                    .configure_mesh()
                    .x_desc("Frequency (Hz)")
                    .y_desc(y_desc)
                    .x_label_formatter(&|x| format!("{x:.0}"))
                    .draw()
                    .expect("failed to draw chart axes");

                chart
                    .configure_secondary_axes()
                    .y_desc("Phase (deg)")
                    .y_label_formatter(&|y| format!("{y:.0}°"))
                    .draw()
                    .expect("failed to draw secondary axes");

                chart
                    .draw_series(LineSeries::new(plot_data, &BLUE))
                    .expect("failed to draw frequency response")
                    .label("Magnitude")
                    .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], BLUE));

                chart
                    .draw_secondary_series(DashedLineSeries::new(
                        phase_plot_data,
                        5,
                        5,
                        RED.stroke_width(2),
                    ))
                    .expect("failed to draw phase response")
                    .label("Phase")
                    .legend(|(x, y)| {
                        PathElement::new(vec![(x, y), (x + 20, y)], RED.stroke_width(2))
                    });

                chart
                    .configure_series_labels()
                    .background_style(WHITE.mix(0.8))
                    .border_style(BLACK)
                    .position(SeriesLabelPosition::UpperRight)
                    .draw()
                    .expect("failed to draw legend");
            }
            (FrequencyScale::Log, false) => {
                let mut builder = ChartBuilder::on(&chart_area);
                builder
                    .margin(20)
                    .x_label_area_size(60)
                    .y_label_area_size(70);
                if let Some(cap) = caption_title {
                    builder.caption(cap, ("sans-serif", 30));
                }

                let mut chart = builder
                    .build_cartesian_2d((self.min_freq..self.max_freq).log_scale(), y_range)
                    .expect("failed to create chart");

                chart
                    .configure_mesh()
                    .x_desc("Frequency (Hz)")
                    .y_desc(y_desc)
                    .x_label_formatter(&|f| {
                        if *f >= 1000.0 {
                            format!("{:.0}k", f / 1000.0)
                        } else {
                            format!("{f:.0}")
                        }
                    })
                    .draw()
                    .expect("failed to draw chart axes");

                chart
                    .draw_series(LineSeries::new(plot_data, &BLUE))
                    .expect("failed to draw frequency response");
            }
            (FrequencyScale::Log, true) => {
                let mut builder = ChartBuilder::on(&chart_area);
                builder
                    .margin(20)
                    .x_label_area_size(60)
                    .y_label_area_size(70)
                    .right_y_label_area_size(70);
                if let Some(cap) = caption_title {
                    builder.caption(cap, ("sans-serif", 30));
                }

                let mut chart = builder
                    .build_cartesian_2d((self.min_freq..self.max_freq).log_scale(), y_range)
                    .expect("failed to create chart")
                    .set_secondary_coord((self.min_freq..self.max_freq).log_scale(), phase_range);

                chart
                    .configure_mesh()
                    .x_desc("Frequency (Hz)")
                    .y_desc(y_desc)
                    .x_label_formatter(&|f| {
                        if *f >= 1000.0 {
                            format!("{:.0}k", f / 1000.0)
                        } else {
                            format!("{f:.0}")
                        }
                    })
                    .draw()
                    .expect("failed to draw chart axes");

                chart
                    .configure_secondary_axes()
                    .y_desc("Phase (deg)")
                    .y_label_formatter(&|y| format!("{y:.0}°"))
                    .draw()
                    .expect("failed to draw secondary axes");

                chart
                    .draw_series(LineSeries::new(plot_data, &BLUE))
                    .expect("failed to draw frequency response")
                    .label("Magnitude")
                    .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], BLUE));

                chart
                    .draw_secondary_series(DashedLineSeries::new(
                        phase_plot_data,
                        5,
                        5,
                        RED.stroke_width(2),
                    ))
                    .expect("failed to draw phase response")
                    .label("Phase")
                    .legend(|(x, y)| {
                        PathElement::new(vec![(x, y), (x + 20, y)], RED.stroke_width(2))
                    });

                chart
                    .configure_series_labels()
                    .background_style(WHITE.mix(0.8))
                    .border_style(BLACK)
                    .position(SeriesLabelPosition::UpperRight)
                    .draw()
                    .expect("failed to draw legend");
            }
        }

        root.present().expect("failed to save plot");
    }
}
