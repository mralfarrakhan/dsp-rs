use num_traits::Float;

use crate::processor::Processor;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub struct Nil;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Chain<H, T> {
    pub head: H,
    pub tail: T,
}

impl<H, T> Chain<H, T> {
    pub const fn new(head: H, tail: T) -> Self {
        Self { head, tail }
    }

    pub fn into_parts(self) -> (H, T) {
        (self.head, self.tail)
    }
}

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

impl<F, H, T> Processor<F> for Chain<H, T>
where
    F: Float,
    H: Processor<F>,
    T: Processor<F>,
{
    #[inline]
    fn process_sample(&mut self, sample: F) -> F {
        let sample = self.head.process_sample(sample);
        self.tail.process_sample(sample)
    }

    #[inline]
    fn process_buffer(&mut self, buffer: &mut [F])
    where
        F: Copy,
    {
        self.head.process_buffer(buffer);
        self.tail.process_buffer(buffer);
    }
}

#[macro_export]
macro_rules! chain {
    () => {
        $crate::chain::Nil
    };

    ($single:expr $(,)?) => {
        $single
    };

    ($head:expr, $($tail:expr),+ $(,)?) => {
        $crate::chain::Chain::new(
            $head,
            $crate::chain!($($tail),+),
        )
    };
}
