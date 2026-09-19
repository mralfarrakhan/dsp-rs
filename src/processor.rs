use num_traits::Float;

pub trait Processor<F: Float> {
    fn process_sample(&mut self, sample: F) -> F;

    fn process_buffer(&mut self, buffer: &mut [F])
    where
        F: Copy,
    {
        for sample in buffer {
            *sample = self.process_sample(*sample)
        }
    }
}
