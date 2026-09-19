use std::path::Path;

use plotters::{
    backend::BitMapBackend,
    chart::ChartBuilder,
    coord::combinators::IntoLogRange,
    drawing::IntoDrawingArea,
    series::LineSeries,
    style::{BLUE, WHITE},
};
use rustfft::{FftPlanner, num_complex::Complex};

use dsp_rs::processor::Processor;

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

#[allow(dead_code)]
pub struct Response<P: Processor<f32>> {
    processor: P,

    sample_rate: f32,
    samples: usize,

    min_freq: f32,
    max_freq: f32,

    min_magnitude: Option<f32>,
    max_magnitude: Option<f32>,

    magnitude_scale: MagnitudeScale,
    frequency_scale: FrequencyScale,
}

pub fn response<P>(processor: P, sample_rate: f32) -> Response<P>
where
    P: Processor<f32>,
{
    Response {
        processor,

        sample_rate,
        samples: 16_384,

        min_freq: 20.0,
        max_freq: 20_000.0,

        min_magnitude: None,
        max_magnitude: None,

        magnitude_scale: MagnitudeScale::Linear,
        frequency_scale: FrequencyScale::Linear,
    }
}

#[allow(dead_code)]
impl<P> Response<P>
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

        let mut signal: Vec<Complex<f32>> = (0..self.samples)
            .map(|i| {
                let input = if i == 0 { 1.0 } else { 0.0 };
                let output = self.processor.process_sample(input);

                Complex::new(output, 0.0)
            })
            .collect();

        let mut planner = FftPlanner::<f32>::new();
        let fft = planner.plan_fft_forward(self.samples);

        fft.process(&mut signal);

        let response: Vec<(f32, f32)> = signal
            .iter()
            .take(self.samples / 2 + 1)
            .enumerate()
            .filter_map(|(k, value)| {
                let frequency = k as f32 * self.sample_rate / self.samples as f32;

                if frequency < self.min_freq || frequency > self.max_freq {
                    return None;
                }

                Some((frequency, value.norm()))
            })
            .collect();

        let plot_data: Vec<(f32, f32)> = response
            .iter()
            .map(|&(frequency, magnitude)| {
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

        let y_range = match (self.min_magnitude, self.max_magnitude) {
            (Some(min), Some(max)) => min..max,
            _ => match self.magnitude_scale {
                MagnitudeScale::Linear => 0.0..1.1,
                MagnitudeScale::Decibel => -80.0..10.0,
            },
        };

        let path = path.as_ref();
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent).expect("failed to create plot output directory");
        }

        let root = BitMapBackend::new(path, (1000, 600)).into_drawing_area();
        root.fill(&WHITE)
            .expect("failed to initialize drawing area");

        let y_desc = match self.magnitude_scale {
            MagnitudeScale::Linear => "Magnitude",
            MagnitudeScale::Decibel => "Magnitude (dB)",
        };

        match self.frequency_scale {
            FrequencyScale::Linear => {
                let mut chart = ChartBuilder::on(&root)
                    .margin(20)
                    .caption("Frequency Response", ("sans-serif", 30))
                    .x_label_area_size(60)
                    .y_label_area_size(70)
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
            FrequencyScale::Log => {
                let mut chart = ChartBuilder::on(&root)
                    .margin(20)
                    .caption("Frequency Response", ("sans-serif", 30))
                    .x_label_area_size(60)
                    .y_label_area_size(70)
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
        }

        root.present().expect("failed to save plot");
    }
}
