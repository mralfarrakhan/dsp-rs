//! Parallel processor composition and summing (Heterogeneous List).

use num_traits::Float;

use crate::processor::Processor;

/// A heterogeneous list node executing two processors in parallel and summing their outputs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Parallel<H, T> {
    /// The first branch processor.
    pub head: H,
    /// The remaining branch processor(s).
    pub tail: T,
}

impl<H, T> Parallel<H, T> {
    /// Creates a new parallel summing node of two processors.
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

    fn parameters(&self) -> Self::Parameters {}

    fn set_parameters(&mut self, _: Self::Parameters) {}

    #[inline]
    fn process_sample(&mut self, sample: F) -> F {
        self.head.process_sample(sample) + self.tail.process_sample(sample)
    }

    #[inline]
    fn process_buffer(&mut self, buffer: &mut [F]) {
        for sample in buffer {
            *sample = self.process_sample(*sample);
        }
    }

    fn reset(&mut self) {
        self.head.reset();
        self.tail.reset();
    }
}

/// Constructs a static parallel summing pipeline from multiple processors.
///
/// Terminates on [`Zero`](crate::Zero) (additive identity).
///
/// # Example
///
/// ```rust
/// use dsp_rs::{Gain, parallel, Processor};
///
/// let mut mixer = parallel!(Gain::new(0.5f32), Gain::new(0.2f32));
/// assert_eq!(mixer.process_sample(2.0), 1.4);
/// ```
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
