mod common;

use crate::common::dsp_test;
use dsp_rs::{Delay, DelayLine, DelayParameters, Processor};

const SAMPLE_RATE: f32 = 48_000.0;

#[test]
fn test_delay_line_circular_wrap() {
    let mut mem = [0.0f32; 4];
    let mut dl = DelayLine::new(&mut mem);

    // Initial read should be zero
    assert_eq!(dl.read_sample(1), 0.0);

    // Write 10.0, 20.0, 30.0
    dl.write_sample(10.0);
    assert_eq!(dl.read_sample(1), 10.0);

    dl.write_sample(20.0);
    assert_eq!(dl.read_sample(1), 20.0);
    assert_eq!(dl.read_sample(2), 10.0);

    dl.write_sample(30.0);
    assert_eq!(dl.read_sample(1), 30.0);
    assert_eq!(dl.read_sample(2), 20.0);
    assert_eq!(dl.read_sample(3), 10.0);

    // Write 40.0 (wraps around write_pos to 0)
    dl.write_sample(40.0);
    assert_eq!(dl.read_sample(1), 40.0);
    assert_eq!(dl.read_sample(2), 30.0);
    assert_eq!(dl.read_sample(3), 20.0);

    // Write 50.0 (overwrites 10.0)
    dl.write_sample(50.0);
    assert_eq!(dl.read_sample(1), 50.0);
    assert_eq!(dl.read_sample(2), 40.0);
    assert_eq!(dl.read_sample(3), 30.0);
}

#[test]
fn test_delay_impulse_response() {
    let mut mem = [0.0f32; 16];
    let params = DelayParameters::new(3, 0.5, 1.0); // 3 samples delay, 50% feedback, 100% wet
    let mut delay = Delay::new(&mut mem, params);

    let impulse = [1.0f32, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
    let mut output = [0.0f32; 10];

    for (i, &x) in impulse.iter().enumerate() {
        output[i] = delay.process_sample(x);
    }

    // Delay = 3 samples:
    // samples 0, 1, 2 = 0.0
    assert_eq!(output[0], 0.0);
    assert_eq!(output[1], 0.0);
    assert_eq!(output[2], 0.0);
    // sample 3 = 1.0
    assert_eq!(output[3], 1.0);
    // samples 4, 5 = 0.0
    assert_eq!(output[4], 0.0);
    assert_eq!(output[5], 0.0);
    // sample 6 = 0.5 (feedback 1.0 * 0.5)
    assert_eq!(output[6], 0.5);
    // sample 9 = 0.25 (feedback 0.5 * 0.5)
    assert_eq!(output[9], 0.25);
}

#[test]
fn test_delay_wet_dry_mix() {
    let mut mem = [0.0f32; 8];
    let params = DelayParameters::new(2, 0.0, 0.0); // 0% wet = pure dry
    let mut delay = Delay::new(&mut mem, params);

    assert_eq!(delay.process_sample(3.5), 3.5);
    assert_eq!(delay.process_sample(7.0), 7.0);

    // Switch to 50% wet
    delay.set_parameters(DelayParameters::new(2, 0.0, 0.5));
    // Sample 0: input 2.0, delayed 3.5 -> 2.0 * 0.5 + 3.5 * 0.5 = 2.75
    let out = delay.process_sample(2.0);
    assert_eq!(out, 2.75);
}

#[test]
fn test_delay_from_time() {
    let params = DelayParameters::from_time(SAMPLE_RATE, 0.01, 0.4, 0.8).unwrap();
    assert_eq!(params.delay_samples, 480);
    assert_eq!(params.feedback, 0.4);
    assert_eq!(params.wet, 0.8);

    // Invalid time
    assert!(DelayParameters::from_time(SAMPLE_RATE, -0.1, 0.0, 0.0).is_none());
    assert!(DelayParameters::from_time(0.0f32, 0.1, 0.0, 0.0).is_none());
}

#[test]
fn test_delay_reset() {
    let mut mem = [0.0f32; 8];
    let mut delay = Delay::new(&mut mem, DelayParameters::new(2, 0.5, 1.0));

    delay.process_sample(1.0);
    delay.process_sample(0.0);

    delay.reset();

    // After reset, reading delayed samples should produce 0.0
    assert_eq!(delay.process_sample(0.0), 0.0);
    assert_eq!(delay.process_sample(0.0), 0.0);
}

#[test]
fn test_delay_plot() {
    let mut mem = [0.0f32; 48_000];
    let params = DelayParameters::from_time(SAMPLE_RATE, 0.05, 0.4, 0.5).unwrap();
    let delay = Delay::new(&mut mem, params);

    dsp_test::temporal_response(delay, SAMPLE_RATE)
        .title("Delay (50ms, 40% fb, 50% wet) temporal response")
        .benchmark()
        .save("test_output/temporal_delay.png");
}
