//! Common identity sentinels and denormal floating-point protection.

use num_traits::Float;

use crate::processor::Processor;

/// Identity processor for serial chains ([`Chain`](crate::Chain)).
///
/// Passes input audio through unchanged without overhead.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub struct Nil;

impl<F: Float> Processor<F> for Nil {
    type Parameters = ();

    fn parameters(&self) -> Self::Parameters {}

    fn set_parameters(&mut self, _: Self::Parameters) {}

    #[inline]
    fn process_sample(&mut self, sample: F) -> F {
        sample
    }

    #[inline]
    fn process_buffer(&mut self, _buffer: &mut [F]) {}

    fn reset(&mut self) {}
}

/// Additive zero processor for parallel pipelines ([`Parallel`](crate::Parallel)).
///
/// Output is always zero ($0.0$).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub struct Zero;

impl<F: Float> Processor<F> for Zero {
    type Parameters = ();

    fn parameters(&self) -> Self::Parameters {}

    #[inline]
    fn set_parameters(&mut self, _: Self::Parameters) {}

    #[inline]
    fn process_sample(&mut self, _sample: F) -> F {
        F::zero()
    }

    #[inline]
    fn process_buffer(&mut self, buffer: &mut [F]) {
        for sample in buffer {
            *sample = F::zero();
        }
    }

    fn reset(&mut self) {}
}

/// Helper trait for flushing subnormal/denormal floating-point values to zero.
///
/// Prevents extreme CPU performance degradation caused by microcode exceptions
/// when decaying filter tails approach zero.
pub trait Denormal: Float {
    /// Adds and subtracts a minuscule offset ($\approx 10^{-25}$) to flush subnormals to zero.
    #[inline]
    fn zapgremlins(self) -> Self {
        let offset = Self::from(1.0e-25).unwrap_or_else(Self::zero);
        (self + offset) - offset
    }

    /// Assigns `new_value` to `*self` after applying denormal flushing.
    #[inline]
    fn assign_zapped(&mut self, new_value: Self) {
        *self = new_value.zapgremlins();
    }
}

impl<F: Float> Denormal for F {}
