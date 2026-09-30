use num_traits::Float;

use crate::processor::Processor;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub struct Nil;

impl<F: Float> Processor<F> for Nil {
    type Parameters = ();

    fn set_parameters(&mut self, _: Self::Parameters) {}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub struct Zero;

impl<F: Float> Processor<F> for Zero {
    type Parameters = ();

    #[inline]
    fn set_parameters(&mut self, _: Self::Parameters) {}

    #[inline]
    fn process_sample(&mut self, _sample: F) -> F {
        F::zero()
    }

    #[inline]
    fn process_buffer(&mut self, buffer: &mut [F])
    where
        F: Copy,
    {
        for sample in buffer {
            *sample = F::zero();
        }
    }
}

pub trait Denormal: Float {
    #[inline]
    fn zapgremlins(self) -> Self {
        let offset = Self::from(1.0e-25).unwrap_or_else(Self::zero);
        (self + offset) - offset
    }

    #[inline]
    fn assign_zapped(&mut self, new_value: Self) {
        *self = new_value.zapgremlins();
    }
}

impl<F: Float> Denormal for F {}
