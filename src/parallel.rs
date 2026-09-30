use num_traits::Float;

use crate::processor::Processor;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Parallel<H, T> {
    pub head: H,
    pub tail: T,
}

impl<H, T> Parallel<H, T> {
    pub const fn new(head: H, tail: T) -> Self {
        Self { head, tail }
    }
}

impl<F, H, T> Processor<F> for Parallel<H, T>
where
    F: Float,
    H: Processor<F>,
    T: Processor<F>,
{
    type Parameters = ();

    fn set_parameters(&mut self, _: Self::Parameters) {}

    #[inline]
    fn process_sample(&mut self, sample: F) -> F {
        self.head.process_sample(sample) + self.tail.process_sample(sample)
    }

    #[inline]
    fn process_buffer(&mut self, buffer: &mut [F])
    where
        F: Copy,
    {
        for sample in buffer {
            *sample = self.process_sample(*sample);
        }
    }
}

#[macro_export]
macro_rules! parallel {
    () => {
        $crate::Zero
    };

    ($single:expr $(,)?) => {
        $crate::parallel::Parallel::new($single, $crate::Zero)
    };

    ($head:expr, $($tail:expr),+ $(,)?) => {
        $crate::parallel::Parallel::new(
            $head,
            $crate::parallel!($($tail),+),
        )
    };
}
