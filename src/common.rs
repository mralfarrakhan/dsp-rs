use num_traits::Float;

use crate::processor::Processor;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub struct Nil;

impl<F: Float> Processor<F> for Nil {
    #[inline]
    fn process_sample(&mut self, sample: F) -> F {
        sample
    }

    #[inline]
    fn process_buffer(&mut self, _buffer: &mut [F])
    where
        F: Copy,
    {
    }
}
