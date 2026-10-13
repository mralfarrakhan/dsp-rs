//! Algorithmic reverberation based on the classic Schroeder topology.
//!
//! Provides a minimal, stack-safe Schroeder [`Reverb`] featuring 4 parallel low-pass damped
//! [`CombFilter`]s and 2 series [`AllPassDelay`] diffusers, partitioned from a single caller-provided slice.

use num_traits::Float;

use crate::{delay::DelayLine, processor::Processor};

/// Feedback comb filter with a 1-pole low-pass filter in the feedback loop.
///
/// Simulates frequency-dependent high-frequency room absorption (damping).
pub struct CombFilter<'a, F: Float> {
    delay_line: DelayLine<'a, F>,
    delay_samples: usize,
    filter_state: F,
    feedback: F,
    damping: F,
}

impl<'a, F: Float> CombFilter<'a, F> {
    /// Creates a comb filter backed by the provided buffer slice with fixed delay length.
    pub fn new(buffer: &'a mut [F], delay_samples: usize) -> Self {
        Self {
            delay_line: DelayLine::new(buffer),
            delay_samples,
            filter_state: F::zero(),
            feedback: F::zero(),
            damping: F::zero(),
        }
    }

    /// Sets the feedback reflection coefficient.
    pub fn set_feedback(&mut self, feedback: F) {
        self.feedback = feedback;
    }

    /// Sets the low-pass damping coefficient (0.0 = no damping, 1.0 = maximum damping).
    pub fn set_damping(&mut self, damping: F) {
        self.damping = damping;
    }

    /// Processes a sample through the damped feedback comb filter.
    #[inline]
    pub fn process_sample(&mut self, sample: F) -> F {
        let output = self.delay_line.read_sample(self.delay_samples);
        let one = F::one();
        self.filter_state = output * (one - self.damping) + self.filter_state * self.damping;
        self.delay_line
            .write_sample(sample + self.filter_state * self.feedback);
        output
    }

    /// Clears internal delay memory and filter state.
    pub fn reset(&mut self) {
        self.delay_line.reset();
        self.filter_state = F::zero();
    }
}

/// An all-pass delay diffuser with flat frequency magnitude response.
///
/// Scrambles signal phase and smears transient impulses into dense reflections.
pub struct AllPassDelay<'a, F: Float> {
    delay_line: DelayLine<'a, F>,
    delay_samples: usize,
    feedback: F,
}

impl<'a, F: Float> AllPassDelay<'a, F> {
    /// Creates an all-pass delay diffuser with default Schroeder feedback gain ($g = 0.5$).
    pub fn new(buffer: &'a mut [F], delay_samples: usize) -> Self {
        let half = F::from(0.5).unwrap_or_else(F::zero);
        Self {
            delay_line: DelayLine::new(buffer),
            delay_samples,
            feedback: half,
        }
    }

    /// Processes a sample through the all-pass phase diffuser.
    #[inline]
    pub fn process_sample(&mut self, sample: F) -> F {
        let delayed = self.delay_line.read_sample(self.delay_samples);
        let output = -sample + delayed;
        self.delay_line.write_sample(sample + delayed * self.feedback);
        output
    }

    /// Clears internal delay memory to zero.
    pub fn reset(&mut self) {
        self.delay_line.reset();
    }
}

/// 3-knob configuration parameters for [`Reverb`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ReverbParameters<F: Float> {
    /// Room size / decay time (0.0 = small, 1.0 = large).
    pub room_size: F,
    /// High-frequency absorption damping (0.0 = bright, 1.0 = dark).
    pub damping: F,
    /// Wet signal blend (0.0 = 100% dry, 1.0 = 100% wet). Dry mix is automatically `1.0 - wet`.
    pub wet: F,
}

impl<F: Float> ReverbParameters<F> {
    /// Creates reverb parameters from room size, damping factor, and wet level.
    pub fn new(room_size: F, damping: F, wet: F) -> Self {
        Self {
            room_size,
            damping,
            wet,
        }
    }
}

/// Minimum buffer capacity required to initialize [`Reverb`] (5,934 samples $\approx 23.7\text{ KB}$ for `f32`).
pub const REVERB_MIN_BUFFER_SAMPLES: usize = Reverb::<f32>::MIN_BUFFER_SAMPLES;

/// A minimal algorithmic reverb based on the classic Schroeder topology.
///
/// Features 4 parallel damped comb filters and 2 series all-pass diffusers.
/// The entire reverb operates over a single, partitioned caller-provided slice,
/// ensuring zero heap allocations and compatibility with embedded systems (e.g. ESP32).
pub struct Reverb<'a, F: Float> {
    comb1: CombFilter<'a, F>,
    comb2: CombFilter<'a, F>,
    comb3: CombFilter<'a, F>,
    comb4: CombFilter<'a, F>,
    allpass1: AllPassDelay<'a, F>,
    allpass2: AllPassDelay<'a, F>,
    parameters: ReverbParameters<F>,
}

impl<'a, F: Float> Reverb<'a, F> {
    /// Prime-tuned delay length for Comb 1 (1,116 samples).
    pub const COMB1_SAMPLES: usize = 1116;
    /// Prime-tuned delay length for Comb 2 (1,188 samples).
    pub const COMB2_SAMPLES: usize = 1188;
    /// Prime-tuned delay length for Comb 3 (1,277 samples).
    pub const COMB3_SAMPLES: usize = 1277;
    /// Prime-tuned delay length for Comb 4 (1,356 samples).
    pub const COMB4_SAMPLES: usize = 1356;
    /// Prime-tuned delay length for All-Pass 1 (556 samples).
    pub const ALLPASS1_SAMPLES: usize = 556;
    /// Prime-tuned delay length for All-Pass 2 (441 samples).
    pub const ALLPASS2_SAMPLES: usize = 441;

    /// Total number of samples required in the backing buffer slice (5,934 samples).
    pub const MIN_BUFFER_SAMPLES: usize = Self::COMB1_SAMPLES
        + Self::COMB2_SAMPLES
        + Self::COMB3_SAMPLES
        + Self::COMB4_SAMPLES
        + Self::ALLPASS1_SAMPLES
        + Self::ALLPASS2_SAMPLES;

    /// Creates a reverb processor by partitioning `buffer` into 6 sub-slices.
    ///
    /// Returns `None` if `buffer.len() < MIN_BUFFER_SAMPLES`.
    pub fn new(buffer: &'a mut [F], parameters: ReverbParameters<F>) -> Option<Self> {
        if buffer.len() < Self::MIN_BUFFER_SAMPLES {
            return None;
        }

        let (c1_buf, rest) = buffer.split_at_mut(Self::COMB1_SAMPLES);
        let (c2_buf, rest) = rest.split_at_mut(Self::COMB2_SAMPLES);
        let (c3_buf, rest) = rest.split_at_mut(Self::COMB3_SAMPLES);
        let (c4_buf, rest) = rest.split_at_mut(Self::COMB4_SAMPLES);
        let (ap1_buf, rest) = rest.split_at_mut(Self::ALLPASS1_SAMPLES);
        let (ap2_buf, _) = rest.split_at_mut(Self::ALLPASS2_SAMPLES);

        let mut reverb = Self {
            comb1: CombFilter::new(c1_buf, Self::COMB1_SAMPLES),
            comb2: CombFilter::new(c2_buf, Self::COMB2_SAMPLES),
            comb3: CombFilter::new(c3_buf, Self::COMB3_SAMPLES),
            comb4: CombFilter::new(c4_buf, Self::COMB4_SAMPLES),
            allpass1: AllPassDelay::new(ap1_buf, Self::ALLPASS1_SAMPLES),
            allpass2: AllPassDelay::new(ap2_buf, Self::ALLPASS2_SAMPLES),
            parameters,
        };

        reverb.apply_parameters();
        Some(reverb)
    }

    fn apply_parameters(&mut self) {
        // Map room_size (0..1) to stable feedback range (0.7 .. 0.98)
        let min_fb = F::from(0.70).unwrap_or_else(F::zero);
        let max_fb = F::from(0.98).unwrap_or_else(F::one);
        let feedback = min_fb + (max_fb - min_fb) * self.parameters.room_size.clamp(F::zero(), F::one());

        // Damping scaling (0 .. 0.4)
        let max_damp = F::from(0.40).unwrap_or_else(F::zero);
        let damping = max_damp * self.parameters.damping.clamp(F::zero(), F::one());

        self.comb1.set_feedback(feedback);
        self.comb2.set_feedback(feedback);
        self.comb3.set_feedback(feedback);
        self.comb4.set_feedback(feedback);

        self.comb1.set_damping(damping);
        self.comb2.set_damping(damping);
        self.comb3.set_damping(damping);
        self.comb4.set_damping(damping);
    }
}

impl<'a, F: Float> Processor<F> for Reverb<'a, F> {
    type Parameters = ReverbParameters<F>;

    fn parameters(&self) -> Self::Parameters {
        self.parameters
    }

    fn set_parameters(&mut self, parameters: Self::Parameters) {
        self.parameters = parameters;
        self.apply_parameters();
    }

    #[inline]
    fn process_sample(&mut self, sample: F) -> F {
        // Run parallel comb filters
        let c1 = self.comb1.process_sample(sample);
        let c2 = self.comb2.process_sample(sample);
        let c3 = self.comb3.process_sample(sample);
        let c4 = self.comb4.process_sample(sample);

        // Sum and scale by 0.25
        let quarter = F::from(0.25).unwrap_or_else(F::one);
        let comb_sum = (c1 + c2 + c3 + c4) * quarter;

        // Run series all-pass diffusers
        let ap1 = self.allpass1.process_sample(comb_sum);
        let wet_out = self.allpass2.process_sample(ap1);

        // Dry / wet blend
        let one = F::one();
        let wet = self.parameters.wet.clamp(F::zero(), one);
        sample * (one - wet) + wet_out * wet
    }

    fn reset(&mut self) {
        self.comb1.reset();
        self.comb2.reset();
        self.comb3.reset();
        self.comb4.reset();
        self.allpass1.reset();
        self.allpass2.reset();
    }
}
