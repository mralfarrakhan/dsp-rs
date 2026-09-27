use num_traits::Float;

use crate::processor::Processor;

#[derive(Debug, Clone, Copy)]
pub struct Gain<F: Float> {
    gain: F,
}

impl<F: Float> Gain<F> {
    pub fn new(gain: F) -> Self {
        Self { gain }
    }
}

impl<F: Float> Processor<F> for Gain<F> {
    #[inline]
    fn process_sample(&mut self, sample: F) -> F {
        sample * self.gain
    }
}
