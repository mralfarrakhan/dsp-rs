use num_traits::Float;

use crate::processor::Processor;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GainParameters<F: Float> {
    pub gain: F,
}

impl<F: Float> GainParameters<F> {
    pub fn new(gain: F) -> Self {
        Self { gain }
    }

    pub fn from_db(db: F) -> Option<Self> {
        let ten = F::from(10.0)?;
        let twenty = F::from(20.0)?;
        let gain = ten.powf(db / twenty);
        Some(Self { gain })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Gain<F: Float> {
    parameters: GainParameters<F>,
}

impl<F: Float> Gain<F> {
    pub fn new(gain: F) -> Self {
        Self {
            parameters: GainParameters { gain },
        }
    }

    pub fn from_db(db: F) -> Option<Self> {
        Some(Self {
            parameters: GainParameters::from_db(db)?,
        })
    }

    pub fn set_db(&mut self, db: F) -> Option<()> {
        self.parameters = GainParameters::from_db(db)?;
        Some(())
    }
}

impl<F: Float> Processor<F> for Gain<F> {
    type Parameters = GainParameters<F>;

    fn parameters(&self) -> Self::Parameters {
        self.parameters
    }

    fn set_parameters(&mut self, parameters: Self::Parameters) {
        self.parameters = parameters;
    }

    #[inline]
    fn process_sample(&mut self, sample: F) -> F {
        sample * self.parameters.gain
    }

    #[inline]
    fn process_buffer(&mut self, buffer: &mut [F]) {
        let g = self.parameters.gain;
        for sample in buffer {
            *sample = *sample * g;
        }
    }

    fn reset(&mut self) {}
}
