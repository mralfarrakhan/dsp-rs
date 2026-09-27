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
            .assign_zapped(self.b_0 * sample + self.b_1 * self.x - self.a_1 * self.y);
        self.x.assign_zapped(sample);
        self.y
    }
}

pub struct SecondOrderLowPass<F: Float> {
    b_0: F,
    b_1: F,
    b_2: F,
    a_1: F,
    a_2: F,
    x_1: F,
    x_2: F,
    y_1: F,
    y_2: F,
}

impl<F> SecondOrderLowPass<F>
where
    F: Float + FloatConst,
{
    pub fn new(sample_rate: F, cutoff: F) -> Self {
        let k = F::tan(F::PI() * cutoff / sample_rate);
        let a_0 = F::one() + F::SQRT_2() * k + k.powi(2);
        let b_0 = k.powi(2).div(a_0);
        let a_1 = F::from(2)
            .unwrap()
            .mul(k.powi(2).sub(F::from(1).unwrap()))
            .div(a_0);
        let a_2 = (F::one() - F::SQRT_2() * k + k.powi(2)).div(a_0);

        Self {
            b_0,
            b_1: F::from(2).unwrap().mul(b_0),
            b_2: b_0,
            a_1,
            a_2,
            x_1: F::zero(),
            x_2: F::zero(),
            y_1: F::zero(),
            y_2: F::zero(),
        }
    }
}

impl<F: Float> Processor<F> for SecondOrderLowPass<F> {
    fn process_sample(&mut self, sample: F) -> F {
        let y = self.b_0 * sample + self.b_1 * self.x_1 + self.b_2 * self.x_2
            - self.a_1 * self.y_1
            - self.a_2 * self.y_2;

        self.x_2.assign_zapped(self.x_1);
        self.x_1.assign_zapped(sample);
        self.y_2.assign_zapped(self.y_1);
        self.y_1.assign_zapped(y);

        y
    }
}
