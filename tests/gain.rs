mod common;

use dsp_rs::{Gain, GainParameters, Processor};

use crate::common::dsp_test;

const SAMPLE_RATE: f32 = 48_000.0;

#[test]
fn test_gain_parameters_and_linear() {
    let mut gain = Gain::new(2.5f32);
    assert_eq!(gain.parameters(), GainParameters { gain: 2.5 });

    assert_eq!(gain.process_sample(2.0), 5.0);

    gain.set_parameters(GainParameters { gain: 0.5 });
    assert_eq!(gain.parameters(), GainParameters { gain: 0.5 });
    assert_eq!(gain.process_sample(4.0), 2.0);
}

#[test]
fn test_gain_from_db() {
    // 0 dB = 1.0
    let g0 = Gain::from_db(0.0f32).unwrap();
    assert!((g0.parameters().gain - 1.0).abs() < 1e-6);

    // +6.0206 dB ≈ 2.0
    let g_plus6 = Gain::from_db(6.0206f32).unwrap();
    assert!((g_plus6.parameters().gain - 2.0).abs() < 1e-4);

    // -6.0206 dB ≈ 0.5
    let g_minus6 = Gain::from_db(-6.0206f32).unwrap();
    assert!((g_minus6.parameters().gain - 0.5).abs() < 1e-4);

    // -20 dB = 0.1
    let g_minus20 = Gain::from_db(-20.0f32).unwrap();
    assert!((g_minus20.parameters().gain - 0.1).abs() < 1e-5);
}

#[test]
fn test_gain_set_db() {
    let mut gain = Gain::new(1.0f32);
    gain.set_db(-20.0f32).unwrap();
    assert!((gain.parameters().gain - 0.1).abs() < 1e-5);
    assert!((gain.process_sample(10.0) - 1.0).abs() < 1e-4);
}

#[test]
fn test_gain_process_buffer() {
    let mut gain = Gain::new(3.0f32);
    let mut buf = [1.0f32, -2.0, 0.5, 0.0];
    gain.process_buffer(&mut buf);
    assert_eq!(buf, [3.0, -6.0, 1.5, 0.0]);
}

#[test]
fn test_gain_plot() {
    let gain = Gain::new(0.6);
    dsp_test::temporal_response(gain, SAMPLE_RATE)
        .title("Gain 0.6x temporal response")
        .benchmark()
        .save("test_output/temporal_gain.png");
}
