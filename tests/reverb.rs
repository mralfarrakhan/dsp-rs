mod common;

use crate::common::dsp_test;
use dsp_rs::{Processor, REVERB_MIN_BUFFER_SAMPLES, Reverb, ReverbParameters};

const SAMPLE_RATE: f32 = 48_000.0;

#[test]
fn test_reverb_buffer_partitioning() {
    let mut small_mem = [0.0f32; REVERB_MIN_BUFFER_SAMPLES - 1];
    assert!(Reverb::new(&mut small_mem, ReverbParameters::new(0.5, 0.5, 0.5)).is_none());

    let mut valid_mem = [0.0f32; REVERB_MIN_BUFFER_SAMPLES];
    assert!(Reverb::new(&mut valid_mem, ReverbParameters::new(0.5, 0.5, 0.5)).is_some());
}

#[test]
fn test_reverb_impulse_response() {
    let mut mem = [0.0f32; REVERB_MIN_BUFFER_SAMPLES];
    let params = ReverbParameters::new(0.6, 0.3, 1.0); // 100% wet
    let mut reverb = Reverb::new(&mut mem, params).unwrap();

    let mut output = [0.0f32; 10_000];
    output[0] = reverb.process_sample(1.0);

    for s in output[1..].iter_mut() {
        *s = reverb.process_sample(0.0);
    }

    // Verify non-empty tail without explosions or NaNs
    let mut has_energy = false;
    for &sample in &output[Reverb::<f32>::COMB1_SAMPLES..] {
        assert!(!sample.is_nan(), "Sample should not be NaN");
        assert!(sample.abs() < 1.0, "Reverb output exploded: {}", sample);
        if sample.abs() > 1e-4 {
            has_energy = true;
        }
    }
    assert!(has_energy, "Reverb tail should have diffused energy");
}

#[test]
fn test_reverb_wet_dry_mix() {
    let mut mem = [0.0f32; REVERB_MIN_BUFFER_SAMPLES];
    let params = ReverbParameters::new(0.5, 0.5, 0.0); // 0% wet = pure dry
    let mut reverb = Reverb::new(&mut mem, params).unwrap();

    assert_eq!(reverb.parameters(), params);
    assert_eq!(reverb.process_sample(4.2), 4.2);
}

#[test]
fn test_reverb_reset() {
    let mut mem = [0.0f32; REVERB_MIN_BUFFER_SAMPLES];
    let params = ReverbParameters::new(0.8, 0.2, 1.0);
    let mut reverb = Reverb::new(&mut mem, params).unwrap();

    // Seed impulse into reverb
    reverb.process_sample(1.0);
    for _ in 0..2000 {
        reverb.process_sample(0.0);
    }

    // Reverb has active ringing tail; reset it
    reverb.reset();

    // After reset, reading next 100 samples with zero input should be completely zero
    for _ in 0..100 {
        assert_eq!(reverb.process_sample(0.0), 0.0);
    }
}

#[test]
fn test_reverb_plot() {
    let mut mem = [0.0f32; REVERB_MIN_BUFFER_SAMPLES];
    let params = ReverbParameters::new(0.7, 0.3, 0.4);
    let reverb = Reverb::new(&mut mem, params).unwrap();

    dsp_test::temporal_response(reverb, SAMPLE_RATE)
        .title("Schroeder Reverb temporal response")
        .benchmark()
        .save("test_output/temporal_reverb.png");
}
