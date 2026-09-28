use num_traits::Float;

use crate::processor::Processor;

#[derive(Debug, Clone, Copy)]
pub struct GainParameters<F: Float> {
    pub gain: F,
}

#[derive(Debug, Clone, Copy)]
pub struct Gain<F: Float> {
    parameters: GainParameters<F>,
}

impl<F: Float> Gain<F> {
    pub fn new(gain: F) -> Self {
        Self {
            parameters: GainParameters { gain },
        }
    }
}

impl<F: Float> Processor<F> for Gain<F> {
    type Parameters = GainParameters<F>;

    fn set_parameter(&mut self, parameters: Self::Parameters) {
        self.parameters = parameters
    }

    #[inline]
    fn process_sample(&mut self, sample: F) -> F {
        sample * self.parameters.gain
    }
}
