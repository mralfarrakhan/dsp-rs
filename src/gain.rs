//! Scalar gain and decibel amplification stages.

use num_traits::Float;

use crate::processor::Processor;

/// Configuration parameters for a gain processor.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GainParameters<F: Float> {
    /// Linear amplitude multiplier.
    pub gain: F,
}

impl<F: Float> GainParameters<F> {
    /// Creates gain parameters with a linear amplitude multiplier.
    pub fn new(gain: F) -> Self {
        Self { gain }
    }

    /// Creates gain parameters from a decibel (dB) value.
    ///
    /// Uses the standard audio formula: $\text{linear} = 10^{\frac{\text{dB}}{20}}$.
    pub fn from_db(db: F) -> Option<Self> {
        let ten = F::from(10.0)?;
        let twenty = F::from(20.0)?;
        let gain = ten.powf(db / twenty);
        Some(Self { gain })
    }
}

/// A simple scalar gain stage supporting linear and decibel controls.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Gain<F: Float> {
    parameters: GainParameters<F>,
}

impl<F: Float> Gain<F> {
    /// Creates a new gain stage with a linear amplitude multiplier.
    pub fn new(gain: F) -> Self {
        Self {
            parameters: GainParameters { gain },
        }
    }

    /// Creates a new gain stage from a decibel (dB) value.
    pub fn from_db(db: F) -> Option<Self> {
        Some(Self {
            parameters: GainParameters::from_db(db)?,
        })
    }

    /// Sets the gain from a decibel (dB) value.
    pub fn set_db(&mut self, db: F) -> Option<()> {
        self.parameters = GainParameters::from_db(db)?;
        Some(())
    }
}

impl<F: Float> Processor<F> for Gain<F> {
    type Parameters = GainParameters<F>;

    fn parameters(&self) -> Self::Parameters {
        self.parameters
    }

    fn set_parameters(&mut self, parameters: Self::Parameters) {
        self.parameters = parameters;
    }

    #[inline]
    fn process_sample(&mut self, sample: F) -> F {
        sample * self.parameters.gain
    }

    /// Processes an audio buffer in-place using an inlined SIMD-friendly scalar loop.
    #[inline]
    fn process_buffer(&mut self, buffer: &mut [F]) {
        let g = self.parameters.gain;
        for sample in buffer {
            *sample = *sample * g;
        }
    }

    fn reset(&mut self) {}
}
