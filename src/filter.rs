use num_traits::{Float, FloatConst};

use crate::{denormals::Denormal, processor::Processor};

pub struct FirstOrderLowPass<F: Float> {
    b_0: F,
    b_1: F,
    a_1: F,
    x: F,
    y: F,
}

impl<F> FirstOrderLowPass<F>
where
    F: Float + FloatConst,
{
    pub fn new(sample_rate: F, cutoff: F) -> Self {
        let k = F::tan(F::PI() * cutoff / sample_rate);
        let b_0 = k.div(F::one() + k);
        let b_1 = b_0;
        let a_1 = (k - F::one()).div(k + F::one());

        Self {
            b_0,
            b_1,
            a_1,
            x: F::zero(),
            y: F::zero(),
        }
    }
}

impl<F: Float> Processor<F> for FirstOrderLowPass<F> {
    #[inline]
    fn process_sample(&mut self, sample: F) -> F {
        self.y
            .assign_flush(self.b_0 * sample + self.b_1 * self.x - self.a_1 * self.y);
        self.x.assign_flush(sample);
        self.y
    }
}
