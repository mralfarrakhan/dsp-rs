//! Core digital signal processing abstractions and multi-channel execution.

use core::marker::PhantomData;

use num_traits::Float;

use crate::buffer::Buffer;

/// Fundamental abstraction for a digital signal processor.
///
/// A `Processor` operates on audio streams either sample-by-sample via [`process_sample`](Processor::process_sample)
/// or block-by-block via [`process_buffer`](Processor::process_buffer).
pub trait Processor<F: Float> {
    /// Associated copyable parameter type representing configuration/coefficients.
    type Parameters: Copy;

    /// Returns the current configuration parameters of this processor.
    fn parameters(&self) -> Self::Parameters;

    /// Updates the configuration parameters of this processor.
    fn set_parameters(&mut self, parameters: Self::Parameters);

    /// Processes a single audio sample in-place and returns the resulting sample.
    fn process_sample(&mut self, sample: F) -> F;

    /// Processes an audio buffer in-place.
    ///
    /// The default implementation iterates through the buffer calling [`process_sample`](Processor::process_sample).
    /// Processors can override this method to provide SIMD-vectorized or optimized batch operations.
    fn process_buffer(&mut self, buffer: &mut [F]) {
        for sample in buffer {
            *sample = self.process_sample(*sample)
        }
    }

    /// Resets the internal state (delay lines, filter histories) to zero.
    ///
    /// Does not alter configured parameters. The default implementation is a no-op for stateless processors.
    fn reset(&mut self) {}
}

/// A multi-channel wrapper that executes an array of identical processors across audio channels.
///
/// Holds `[P; CHANNELS]` without dynamic heap allocation.
pub struct MultiChannelProcessor<F: Float, P: Processor<F>, const CHANNELS: usize> {
    processors: [P; CHANNELS],
    _marker: PhantomData<F>,
}

impl<F: Float, P: Processor<F>, const CHANNELS: usize> MultiChannelProcessor<F, P, CHANNELS> {
    /// Creates a multi-channel processor from an explicit array of per-channel processors.
    pub fn new(processors: [P; CHANNELS]) -> Self {
        Self {
            processors,
            _marker: PhantomData,
        }
    }

    /// Creates a multi-channel processor by cloning a prototype processor across all channels.
    pub fn new_uniform(processor: P) -> Self
    where
        P: Clone,
    {
        Self {
            processors: core::array::from_fn(|_| processor.clone()),
            _marker: PhantomData,
        }
    }

    /// Resets the internal state of all channel processors to zero.
    pub fn reset(&mut self) {
        for p in &mut self.processors {
            p.reset();
        }
    }

    /// Broadcasts a parameter update to all channel processors simultaneously.
    pub fn set_parameters(&mut self, parameters: P::Parameters) {
        for p in &mut self.processors {
            p.set_parameters(parameters);
        }
    }

    /// Returns the current configuration parameters for all channels.
    pub fn parameters(&self) -> [P::Parameters; CHANNELS] {
        core::array::from_fn(|i| self.processors[i].parameters())
    }

    /// Processes a multi-channel planar [`Buffer`] in-place.
    ///
    /// Matches channel slices to respective processors using `.zip()`.
    #[inline]
    pub fn process(&mut self, buffer: &mut Buffer<'_, F>) {
        for (p, b) in self.processors.iter_mut().zip(buffer.channels_iter_mut()) {
            p.process_buffer(b);
        }
    }
}
