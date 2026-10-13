//! Circular delay lines and delay effects.
//!
//! Provides [`DelayLine`] and [`Delay`] operating over non-owning mutable slices (`&'a mut [F]`),
//! ensuring memory safety and zero allocation across embedded microcontrollers and desktop hosts.

use num_traits::Float;

use crate::processor::Processor;

/// A non-owning circular ring buffer for sample delays.
///
/// Wraps a caller-provided mutable slice. Takes only 16 bytes on the stack, allowing
/// large buffers to safely live in static BSS memory, external PSRAM (e.g. ESP32), or heap.
pub struct DelayLine<'a, F: Float> {
    buffer: &'a mut [F],
    write_pos: usize,
}

impl<'a, F: Float> DelayLine<'a, F> {
    /// Creates a new delay line over the provided slice, initializing all samples to zero.
    pub fn new(buffer: &'a mut [F]) -> Self {
        for s in buffer.iter_mut() {
            *s = F::zero();
        }
        Self {
            buffer,
            write_pos: 0,
        }
    }

    /// Returns the capacity of the delay line in samples.
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    /// Returns `true` if the delay buffer has zero capacity.
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Writes a sample to the current write position and advances the circular pointer.
    #[inline]
    pub fn write_sample(&mut self, sample: F) {
        if self.buffer.is_empty() {
            return;
        }
        self.buffer[self.write_pos] = sample;
        self.write_pos += 1;
        if self.write_pos >= self.buffer.len() {
            self.write_pos = 0;
        }
    }

    /// Reads a sample from `delay_samples` in the past.
    ///
    /// Clamps the delay to `self.len() - 1` to guarantee memory safety.
    #[inline]
    pub fn read_sample(&self, delay_samples: usize) -> F {
        if self.buffer.is_empty() {
            return F::zero();
        }
        let delay = delay_samples.min(self.buffer.len().saturating_sub(1));
        let offset = delay % self.buffer.len();
        let read_idx = if self.write_pos >= offset {
            self.write_pos - offset
        } else {
            self.write_pos + self.buffer.len() - offset
        };
        self.buffer[read_idx]
    }

    /// Clears all samples in the circular buffer back to zero and resets the write pointer.
    pub fn reset(&mut self) {
        for s in self.buffer.iter_mut() {
            *s = F::zero();
        }
        self.write_pos = 0;
    }
}

/// Configuration parameters for a feedback delay effect.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DelayParameters<F: Float> {
    /// Delay time in integer samples.
    pub delay_samples: usize,
    /// Feedback multiplier (typically 0.0 to 0.99).
    pub feedback: F,
    /// Wet signal blend (0.0 = 100% dry, 1.0 = 100% wet).
    pub wet: F,
}

impl<F: Float> DelayParameters<F> {
    /// Creates delay parameters from integer samples, feedback, and wet level.
    pub fn new(delay_samples: usize, feedback: F, wet: F) -> Self {
        Self {
            delay_samples,
            feedback,
            wet,
        }
    }

    /// Helper to calculate `delay_samples` from a sample rate and time duration in seconds.
    ///
    /// Returns `None` if `sample_rate <= 0` or `time_seconds < 0`.
    pub fn from_time(sample_rate: F, time_seconds: F, feedback: F, wet: F) -> Option<Self> {
        let zero = F::zero();
        if sample_rate <= zero || time_seconds < zero {
            return None;
        }
        let delay_samples = (sample_rate * time_seconds).round().to_usize()?;
        Some(Self {
            delay_samples,
            feedback,
            wet,
        })
    }
}

/// A standalone feedback delay effect with wet/dry mix.
pub struct Delay<'a, F: Float> {
    delay_line: DelayLine<'a, F>,
    parameters: DelayParameters<F>,
}

impl<'a, F: Float> Delay<'a, F> {
    /// Creates a delay effect backed by the provided buffer slice and configured with initial parameters.
    pub fn new(buffer: &'a mut [F], parameters: DelayParameters<F>) -> Self {
        Self {
            delay_line: DelayLine::new(buffer),
            parameters,
        }
    }
}

impl<'a, F: Float> Processor<F> for Delay<'a, F> {
    type Parameters = DelayParameters<F>;

    fn parameters(&self) -> Self::Parameters {
        self.parameters
    }

    fn set_parameters(&mut self, parameters: Self::Parameters) {
        self.parameters = parameters;
    }

    #[inline]
    fn process_sample(&mut self, sample: F) -> F {
        let delayed = self.delay_line.read_sample(self.parameters.delay_samples);
        let feedback_input = sample + delayed * self.parameters.feedback;
        self.delay_line.write_sample(feedback_input);

        let one = F::one();
        sample * (one - self.parameters.wet) + delayed * self.parameters.wet
    }

    fn reset(&mut self) {
        self.delay_line.reset();
    }
}
