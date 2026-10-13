use core::marker::PhantomData;

use num_traits::Float;

use crate::buffer::Buffer;

pub trait Processor<F: Float>: Clone + Copy {
    type Parameters: Copy;

    fn parameters(&self) -> Self::Parameters;

    fn set_parameters(&mut self, parameters: Self::Parameters);

    fn process_sample(&mut self, sample: F) -> F;

    fn process_buffer(&mut self, buffer: &mut [F]) {
        for sample in buffer {
            *sample = self.process_sample(*sample)
        }
    }

    fn reset(&mut self) {}
}

pub struct MultiChannelProcessor<F: Float, P: Processor<F>, const CHANNELS: usize> {
    processors: [P; CHANNELS],
    _marker: PhantomData<F>,
}

impl<F: Float, P: Processor<F>, const CHANNELS: usize> MultiChannelProcessor<F, P, CHANNELS> {
    pub fn new(processor: P) -> Self {
        Self {
            processors: [processor; CHANNELS],
            _marker: PhantomData,
        }
    }

    pub fn reset(&mut self) {
        for p in &mut self.processors {
            p.reset();
        }
    }

    pub fn set_parameters(&mut self, parameters: P::Parameters) {
        for p in &mut self.processors {
            p.set_parameters(parameters);
        }
    }

    pub fn parameters(&self) -> [P::Parameters; CHANNELS] {
        core::array::from_fn(|i| self.processors[i].parameters())
    }

    #[inline]
    pub fn process(&mut self, buffer: &mut Buffer<'_, F>) {
        for (p, b) in self.processors.iter_mut().zip(buffer.channels_iter_mut()) {
            p.process_buffer(b);
        }
    }
}
