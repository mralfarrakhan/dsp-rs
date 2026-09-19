use std::{collections::VecDeque, path::Path, time::Instant};

use dsp_rs::processor::Processor;
use plotters::{
    backend::BitMapBackend,
    chart::{ChartBuilder, SeriesLabelPosition},
    drawing::IntoDrawingArea,
    element::PathElement,
    series::{DashedLineSeries, LineSeries},
    style::{BLACK, BLUE, Color, IntoFont, IntoTextStyle, RGBColor, WHITE},
};

use crate::common::dsp_test::common::format_with_commas;

pub struct TemporalResponse<P: Processor<f32>> {
    processor: P,
    sample_rate: f32,
    duration_ms: f32,
    frequency: f32,
    low_level: f32,
    high_level: f32,
    min_amplitude: f32,
    max_amplitude: f32,
    title: String,
    show_info: bool,
    benchmark_iterations: Option<usize>,
}

pub fn temporal_response<P>(processor: P, sample_rate: f32) -> TemporalResponse<P>
where
    P: Processor<f32>,
{
    TemporalResponse {
        processor,
        sample_rate,
        duration_ms: 1000.0,
        frequency: 1000.0,
        low_level: 0.1,
        high_level: 1.0,
        min_amplitude: -2.0,
        max_amplitude: 2.0,
        title: "Temporal Response (Dynamics)".to_string(),
        show_info: false,
        benchmark_iterations: None,
    }
}

#[allow(dead_code)]
impl<P> TemporalResponse<P>
where
    P: Processor<f32>,
{
    pub fn duration_ms(mut self, ms: f32) -> Self {
        assert!(ms > 0.0, "duration_ms must be positive");
        self.duration_ms = ms;
        self
    }

    pub fn frequency(mut self, hz: f32) -> Self {
        assert!(hz > 0.0, "frequency must be positive");
        self.frequency = hz;
        self
    }

    pub fn frequency_hz(self, hz: f32) -> Self {
        self.frequency(hz)
    }

    pub fn levels(mut self, low: f32, high: f32) -> Self {
        assert!(low >= 0.0, "low level must be non-negative");
        assert!(low < high, "low level must be less than high level");
        self.low_level = low;
        self.high_level = high;
        self
    }

    pub fn levels_db(mut self, low_db: f32, high_db: f32) -> Self {
        assert!(low_db < high_db, "low_db must be less than high_db");
        self.low_level = 10.0f32.powf(low_db / 20.0);
        self.high_level = 10.0f32.powf(high_db / 20.0);
        self
    }

    pub fn amplitude_range(mut self, min: f32, max: f32) -> Self {
        assert!(min < max, "min must be less than max");
        self.min_amplitude = min;
        self.max_amplitude = max;
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
        assert!(self.sample_rate > 0.0, "sample_rate must be positive");
        assert!(self.duration_ms > 0.0, "duration_ms must be positive");
        assert!(self.frequency > 0.0, "frequency must be positive");
        assert!(
            self.frequency <= self.sample_rate / 2.0,
            "frequency cannot exceed Nyquist frequency (sample_rate / 2)"
        );
        assert!(
            self.min_amplitude < self.max_amplitude,
            "min_amplitude must be less than max_amplitude"
        );

        let total_samples = (self.duration_ms * 0.001 * self.sample_rate).round() as usize;
        assert!(
            total_samples >= 100,
            "duration too short for reliable analysis"
        );

        // 1. Generate 3-stage pulse naive square wave (0..25% low, 25..75% high, 75..100% low)
        let t1 = total_samples / 4;
        let t2 = (3 * total_samples) / 4;

        let mut input_signal = Vec::with_capacity(total_samples);
        let mut input_amp_series = Vec::with_capacity(total_samples);

        for i in 0..total_samples {
            let amp = if i >= t1 && i < t2 {
                self.high_level
            } else {
                self.low_level
            };
            let phase = ((i as f64 * self.frequency as f64) / self.sample_rate as f64).fract();
            let square = if phase < 0.5 { 1.0f32 } else { -1.0f32 };
            input_signal.push(square * amp);
            input_amp_series.push(amp);
        }

        // 2. Process through the processor
        let mut output_signal = Vec::with_capacity(total_samples);
        for &sample in &input_signal {
            output_signal.push(self.processor.process_sample(sample));
        }

        // 3. Optional benchmark on the signal
        let bench_result = self.benchmark_iterations.map(|iters| {
            let iters = iters.max(1);
            let mut bench_buf = vec![0.0f32; total_samples];
            bench_buf[0] = 1.0;

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

        // 4. Sliding window peak envelope detection (1 period causal window)
        let window_samples = (self.sample_rate / self.frequency).round().max(1.0) as usize;
        let mut deque: VecDeque<usize> = VecDeque::new();
        let mut output_envelope = Vec::with_capacity(total_samples);

        for i in 0..total_samples {
            let abs_val = output_signal[i].abs();

            while let Some(&front_idx) = deque.front() {
                if front_idx + window_samples <= i {
                    deque.pop_front();
                } else {
                    break;
                }
            }

            while let Some(&back_idx) = deque.back() {
                if output_signal[back_idx].abs() <= abs_val {
                    deque.pop_back();
                } else {
                    break;
                }
            }

            deque.push_back(i);
            let peak = output_signal[*deque.front().unwrap()].abs();
            output_envelope.push(peak);
        }

        let step = (total_samples / 3000).max(1);

        let mut raw_plot_data = Vec::with_capacity((total_samples / step + 1) * 2);
        for (chunk_idx, chunk) in output_signal.chunks(step).enumerate() {
            let t_ms = ((chunk_idx * step) as f32 / self.sample_rate) * 1000.0;
            let mut min_val = f32::INFINITY;
            let mut max_val = f32::NEG_INFINITY;
            for &s in chunk {
                if s < min_val {
                    min_val = s;
                }
                if s > max_val {
                    max_val = s;
                }
            }
            raw_plot_data.push((t_ms, min_val));
            raw_plot_data.push((t_ms, max_val));
        }

        let input_upper_data: Vec<(f32, f32)> = (0..total_samples)
            .step_by(step)
            .map(|i| {
                let t_ms = (i as f32 / self.sample_rate) * 1000.0;
                (t_ms, input_amp_series[i])
            })
            .collect();

        let input_lower_data: Vec<(f32, f32)> = (0..total_samples)
            .step_by(step)
            .map(|i| {
                let t_ms = (i as f32 / self.sample_rate) * 1000.0;
                (t_ms, -input_amp_series[i])
            })
            .collect();

        let output_upper_data: Vec<(f32, f32)> = (0..total_samples)
            .step_by(step)
            .map(|i| {
                let t_ms = (i as f32 / self.sample_rate) * 1000.0;
                (t_ms, output_envelope[i])
            })
            .collect();

        let output_lower_data: Vec<(f32, f32)> = (0..total_samples)
            .step_by(step)
            .map(|i| {
                let t_ms = (i as f32 / self.sample_rate) * 1000.0;
                (t_ms, -output_envelope[i])
            })
            .collect();

        let path = path.as_ref();
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent).expect("failed to create plot output directory");
        }

        // 6. Header metadata
        let signal_duration_secs = self.duration_ms * 0.001;
        let signal_info = if self.show_info {
            let sample_rate_str = if self.sample_rate.fract() == 0.0 {
                format!("{} Hz", format_with_commas(self.sample_rate as usize))
            } else {
                format!("{:.1} Hz", self.sample_rate)
            };

            let duration_str = if self.duration_ms.fract() == 0.0 {
                format!("{} ms", format_with_commas(self.duration_ms as usize))
            } else {
                format!("{:.1} ms", self.duration_ms)
            };

            let freq_str = if self.frequency.fract() == 0.0 {
                format!("{} Hz", format_with_commas(self.frequency as usize))
            } else {
                format!("{:.1} Hz", self.frequency)
            };

            let levels_str = format!("{:.2} to {:.2}", self.low_level, self.high_level);

            Some(format!(
                "Sample Rate: {}   |   Duration: {}   |   Freq: {}   |   Pulse: {}",
                sample_rate_str, duration_str, freq_str, levels_str
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

            let throughput = (total_samples as f64) / avg_secs;
            let ns_per_sample = (avg_secs * 1e9) / (total_samples as f64);
            let throughput_str = if throughput >= 1e9 {
                format!("{:.2} GSa/s ({:.2} ns/sa)", throughput / 1e9, ns_per_sample)
            } else if throughput >= 1e6 {
                format!("{:.1} MSa/s ({:.2} ns/sa)", throughput / 1e6, ns_per_sample)
            } else if throughput >= 1e3 {
                format!("{:.1} kSa/s ({:.2} ns/sa)", throughput / 1e3, ns_per_sample)
            } else {
                format!("{:.0} Sa/s ({:.2} ns/sa)", throughput, ns_per_sample)
            };

            let realtime_factor = (signal_duration_secs as f64) / avg_secs;
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

        let mut builder = ChartBuilder::on(&chart_area);
        builder
            .margin(20)
            .x_label_area_size(60)
            .y_label_area_size(70);
        if let Some(cap) = caption_title {
            builder.caption(cap, ("sans-serif", 30));
        }

        let mut chart = builder
            .build_cartesian_2d(
                0.0f32..self.duration_ms,
                self.min_amplitude..self.max_amplitude,
            )
            .expect("failed to create chart");

        chart
            .configure_mesh()
            .x_desc("Time (ms)")
            .y_desc("Amplitude")
            .x_label_formatter(&|t| format!("{t:.0}"))
            .y_label_formatter(&|a| format!("{a:.1}"))
            .draw()
            .expect("failed to draw chart axes");

        // 1. Draw raw signal in faint light blue
        chart
            .draw_series(LineSeries::new(raw_plot_data, RGBColor(190, 215, 245)))
            .expect("failed to draw raw signal")
            .label("Raw Signal")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], RGBColor(190, 215, 245)));

        // 2. Draw input reference (upper and lower step lines)
        chart
            .draw_series(DashedLineSeries::new(
                input_upper_data,
                6,
                4,
                RGBColor(140, 140, 140).stroke_width(2),
            ))
            .expect("failed to draw input upper reference")
            .label("Input Reference (±)")
            .legend(|(x, y)| {
                PathElement::new(
                    vec![(x, y), (x + 20, y)],
                    RGBColor(140, 140, 140).stroke_width(2),
                )
            });

        chart
            .draw_series(DashedLineSeries::new(
                input_lower_data,
                6,
                4,
                RGBColor(140, 140, 140).stroke_width(2),
            ))
            .expect("failed to draw input lower reference");

        // 3. Draw output envelope (upper and lower solid blue lines)
        chart
            .draw_series(LineSeries::new(output_upper_data, BLUE.stroke_width(2)))
            .expect("failed to draw output upper envelope")
            .label("Output Envelope (±)")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], BLUE.stroke_width(2)));

        chart
            .draw_series(LineSeries::new(output_lower_data, BLUE.stroke_width(2)))
            .expect("failed to draw output lower envelope");

        chart
            .configure_series_labels()
            .background_style(WHITE.mix(0.8))
            .border_style(BLACK)
            .position(SeriesLabelPosition::UpperRight)
            .draw()
            .expect("failed to draw legend");

        root.present().expect("failed to save plot");
    }
}
