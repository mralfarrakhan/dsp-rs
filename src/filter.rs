use num_traits::{Float, FloatConst};

use crate::{denormals::Denormal, processor::Processor};

pub struct FirstOrderLowPass<F: Float> {
    alpha: F,
    y: F,
}

impl<F> FirstOrderLowPass<F>
where
    F: Float + FloatConst,
{
    pub fn new(sample_rate: F, cutoff: F) -> Self {
        let two = F::from(2.0).unwrap();

        let dt = F::one() / sample_rate;
        let rc = F::one() / (two * F::PI() * cutoff);
        let alpha = dt / (rc + dt);

        Self {
            alpha,
            y: F::zero(),
        }
    }
}

impl<F: Float> Processor<F> for FirstOrderLowPass<F> {
    #[inline]
    fn process_sample(&mut self, sample: F) -> F {
        self.y.assign_flush(self.y + self.alpha * (sample - self.y));
        self.y
    }
}
