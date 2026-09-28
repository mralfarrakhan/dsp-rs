use num_traits::{Float, FloatConst};

use crate::{common::Denormal, processor::Processor};

#[derive(Debug, Clone, Copy)]
pub struct FirstOrderFilterParameters<F: Float> {
    b_0: F,
    b_1: F,
    a_1: F,
}

#[derive(Debug, Clone, Copy)]
pub struct FirstOrderFilter<F: Float> {
    parameters: FirstOrderFilterParameters<F>,
    x: F,
    y: F,
}

impl<F> FirstOrderFilter<F>
where
    F: Float + FloatConst,
{
    pub fn new_lpf(sample_rate: F, cutoff: F) -> Self {
        let k = F::tan(F::PI() * cutoff / sample_rate);
        let b_0 = k.div(F::one() + k);
        let b_1 = b_0;
        let a_1 = (k - F::one()).div(k + F::one());

        Self {
            parameters: FirstOrderFilterParameters { b_0, b_1, a_1 },
            x: F::zero(),
            y: F::zero(),
        }
    }
}

impl<F: Float> Processor<F> for FirstOrderFilter<F> {
    type Parameters = FirstOrderFilterParameters<F>;

    fn set_parameter(&mut self, parameters: Self::Parameters) {
        self.parameters = parameters;
    }

    #[inline]
    fn process_sample(&mut self, sample: F) -> F {
        self.y.assign_zapped(
            self.parameters.b_0 * sample + self.parameters.b_1 * self.x
                - self.parameters.a_1 * self.y,
        );
        self.x.assign_zapped(sample);
        self.y
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SecondOrderFilterParameters<F: Float> {
    b_0: F,
    b_1: F,
    b_2: F,
    a_1: F,
    a_2: F,
}

#[derive(Debug, Clone, Copy)]
pub struct SecondOrderFilter<F: Float> {
    parameters: SecondOrderFilterParameters<F>,
    s_1: F,
    s_2: F,
}

impl<F> SecondOrderFilter<F>
where
    F: Float + FloatConst,
{
    pub fn new_lpf(sample_rate: F, cutoff: F) -> Self {
        let k = F::tan(F::PI() * cutoff / sample_rate);
        let a_0 = F::one() + F::SQRT_2() * k + k.powi(2);
        let b_0 = k.powi(2).div(a_0);
        let a_1 = F::from(2)
            .unwrap()
            .mul(k.powi(2).sub(F::from(1).unwrap()))
            .div(a_0);
        let a_2 = (F::one() - F::SQRT_2() * k + k.powi(2)).div(a_0);

        Self {
            parameters: SecondOrderFilterParameters {
                b_0,
                b_1: F::from(2).unwrap().mul(b_0),
                b_2: b_0,
                a_1,
                a_2,
            },
            s_1: F::zero(),
            s_2: F::zero(),
        }
    }

    pub fn new_hpf(sample_rate: F, cutoff: F) -> Self {
        let k = F::tan(F::PI() * cutoff / sample_rate);
        let a_0 = F::one() + F::SQRT_2() * k + k.powi(2);
        let b_0 = a_0.recip();
        let a_1 = F::from(2)
            .unwrap()
            .mul(k.powi(2).sub(F::from(1).unwrap()))
            .div(a_0);
        let a_2 = (F::one() - F::SQRT_2() * k + k.powi(2)).div(a_0);

        Self {
            parameters: SecondOrderFilterParameters {
                b_0,
                b_1: F::from(2).unwrap().neg().mul(b_0),
                b_2: b_0,
                a_1,
                a_2,
            },
            s_1: F::zero(),
            s_2: F::zero(),
        }
    }

    pub fn new_apf(sample_rate: F, cutoff: F) -> Self {
        let k = F::tan(F::PI() * cutoff / sample_rate);
        let a_0 = F::one() + F::SQRT_2() * k + k.powi(2);
        let a_1 = F::from(2)
            .unwrap()
            .mul(k.powi(2).sub(F::from(1).unwrap()))
            .div(a_0);
        let a_2 = (F::one() - F::SQRT_2() * k + k.powi(2)).div(a_0);

        Self {
            parameters: SecondOrderFilterParameters {
                b_0: a_2,
                b_1: a_1,
                b_2: F::one(),
                a_1,
                a_2,
            },
            s_1: F::zero(),
            s_2: F::zero(),
        }
    }
}

impl<F: Float> Processor<F> for SecondOrderFilter<F> {
    type Parameters = SecondOrderFilterParameters<F>;

    fn set_parameter(&mut self, parameters: Self::Parameters) {
        self.parameters = parameters;
    }

    #[inline]
    fn process_sample(&mut self, sample: F) -> F {
        let y = self.parameters.b_0 * sample + self.s_1;
        self.s_1
            .assign_zapped(self.parameters.b_1 * sample - self.parameters.a_1 * y + self.s_2);
        self.s_2
            .assign_zapped(self.parameters.b_2 * sample - self.parameters.a_2 * y);

        y
    }

    #[inline]
    fn process_buffer(&mut self, buffer: &mut [F])
    where
        F: Copy,
    {
        let b_0 = self.parameters.b_0;
        let b_1 = self.parameters.b_1;
        let b_2 = self.parameters.b_2;
        let a_1 = self.parameters.a_1;
        let a_2 = self.parameters.a_2;
        let mut s_1 = self.s_1;
        let mut s_2 = self.s_2;

        for sample in buffer.iter_mut() {
            let x = *sample;
            let y = b_0 * x + s_1;
            s_1.assign_zapped(b_1 * x - a_1 * y + s_2);
            s_2.assign_zapped(b_2 * x - a_2 * y);
            *sample = y;
        }

        self.s_1 = s_1;
        self.s_2 = s_2;
    }
}
