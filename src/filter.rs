use num_traits::{Float, FloatConst};

use crate::{common::Denormal, processor::Processor};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FilterType {
    LowPass,
    HighPass,
    AllPass,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Coefficients<F: Float> {
    pub b0: F,
    pub b1: F,
    pub b2: F,
    pub a1: F,
    pub a2: F,
}

impl<F: Float> Coefficients<F> {
    pub const fn new(b0: F, b1: F, b2: F, a1: F, a2: F) -> Self {
        Self { b0, b1, b2, a1, a2 }
    }
}

impl<F> Coefficients<F>
where
    F: Float + FloatConst,
{
    pub fn from_params(
        filter_type: FilterType,
        sample_rate: F,
        cutoff: F,
        q: F,
    ) -> Option<Self> {
        let zero = F::zero();
        let one = F::one();
        let two = F::from(2)?;
        let half = one / two;
        let nyquist = sample_rate * half;

        if sample_rate <= zero || cutoff <= zero || cutoff >= nyquist || q <= zero {
            return None;
        }

        let k = (F::PI() * cutoff / sample_rate).tan();
        let k2 = k * k;
        let norm = one + (k / q) + k2;

        let (b0, b1, b2, a1, a2) = match filter_type {
            FilterType::LowPass => {
                let b0 = k2 / norm;
                let b1 = two * b0;
                let b2 = b0;
                let a1 = two * (k2 - one) / norm;
                let a2 = (one - (k / q) + k2) / norm;
                (b0, b1, b2, a1, a2)
            }
            FilterType::HighPass => {
                let b0 = one / norm;
                let b1 = -two * b0;
                let b2 = b0;
                let a1 = two * (k2 - one) / norm;
                let a2 = (one - (k / q) + k2) / norm;
                (b0, b1, b2, a1, a2)
            }
            FilterType::AllPass => {
                let a1 = two * (k2 - one) / norm;
                let a2 = (one - (k / q) + k2) / norm;
                let b0 = a2;
                let b1 = a1;
                let b2 = one;
                (b0, b1, b2, a1, a2)
            }
        };

        Some(Self { b0, b1, b2, a1, a2 })
    }

    pub fn butterworth(
        filter_type: FilterType,
        sample_rate: F,
        cutoff: F,
    ) -> Option<Self> {
        let q = F::one() / F::SQRT_2();
        Self::from_params(filter_type, sample_rate, cutoff, q)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BiquadFilter<F: Float> {
    coefficients: Coefficients<F>,
    s_1: F,
    s_2: F,
}

impl<F: Float> BiquadFilter<F> {
    pub fn new(coefficients: Coefficients<F>) -> Self {
        Self {
            coefficients,
            s_1: F::zero(),
            s_2: F::zero(),
        }
    }

    pub fn reset(&mut self) {
        self.s_1 = F::zero();
        self.s_2 = F::zero();
    }
}

impl<F: Float> Processor<F> for BiquadFilter<F> {
    type Parameters = Coefficients<F>;

    fn parameters(&self) -> Self::Parameters {
        self.coefficients
    }

    fn set_parameters(&mut self, parameters: Self::Parameters) {
        self.coefficients = parameters;
    }

    #[inline]
    fn process_sample(&mut self, sample: F) -> F {
        let y = self.coefficients.b0 * sample + self.s_1;
        self.s_1
            .assign_zapped(self.coefficients.b1 * sample - self.coefficients.a1 * y + self.s_2);
        self.s_2
            .assign_zapped(self.coefficients.b2 * sample - self.coefficients.a2 * y);

        y
    }

    #[inline]
    fn process_buffer(&mut self, buffer: &mut [F]) {
        let b0 = self.coefficients.b0;
        let b1 = self.coefficients.b1;
        let b2 = self.coefficients.b2;
        let a1 = self.coefficients.a1;
        let a2 = self.coefficients.a2;
        let mut s_1 = self.s_1;
        let mut s_2 = self.s_2;

        for sample in buffer.iter_mut() {
            let x = *sample;
            let y = b0 * x + s_1;
            s_1.assign_zapped(b1 * x - a1 * y + s_2);
            s_2.assign_zapped(b2 * x - a2 * y);
            *sample = y;
        }

        self.s_1 = s_1;
        self.s_2 = s_2;
    }

    fn reset(&mut self) {
        self.s_1 = F::zero();
        self.s_2 = F::zero();
    }
}
