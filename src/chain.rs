//! Serial processor composition (Heterogeneous List).

use num_traits::Float;

use crate::processor::Processor;

/// A heterogeneous list node composing two processors in series.
///
/// Output of `head` is routed directly into the input of `tail`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Chain<H, T> {
    /// The first processor in the chain.
    pub head: H,
    /// The remaining processor(s) in the chain.
    pub tail: T,
}

impl<H, T> Chain<H, T> {
    /// Creates a new serial chain of two processors.
    pub const fn new(head: H, tail: T) -> Self {
        Self { head, tail }
    }
}

impl<F, H, T> Processor<F> for Chain<H, T>
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
        let sample = self.head.process_sample(sample);
        self.tail.process_sample(sample)
    }

    #[inline]
    fn process_buffer(&mut self, buffer: &mut [F]) {
        self.head.process_buffer(buffer);
        self.tail.process_buffer(buffer);
    }

    fn reset(&mut self) {
        self.head.reset();
        self.tail.reset();
    }
}

/// Constructs a static serial pipeline from multiple processors.
///
/// Terminates on [`Nil`](crate::Nil) (identity pass-through).
///
/// # Example
///
/// ```rust
/// use dsp_rs::{Gain, chain, Processor};
///
/// let mut pipeline = chain!(Gain::new(2.0f32), Gain::new(3.0f32));
/// assert_eq!(pipeline.process_sample(1.0), 6.0);
/// ```
#[macro_export]
macro_rules! chain {
    () => {
        $crate::Nil
    };

    ($single:expr $(,)?) => {
        $crate::chain::Chain::new($single, $crate::Nil)
    };

    ($head:expr, $($tail:expr),+ $(,)?) => {
        $crate::chain::Chain::new(
            $head,
            $crate::chain!($($tail),+),
        )
    };
}
