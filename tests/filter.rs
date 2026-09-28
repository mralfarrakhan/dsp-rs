mod common;

use std::ops::Div;

use common::dsp_test;
use dsp_rs::{
    chain,
    filter::{FirstOrderFilter, SecondOrderFilter},
    processor::Processor,
};

const SAMPLE_RATE: f32 = 48_000.0;
const SAMPLE_SIZE: usize = 65_536;

#[test]
fn plot_filter() {
    let filter = FirstOrderFilter::new_lpf(SAMPLE_RATE, 10_000.0);

    dsp_test::response(filter, SAMPLE_RATE)
        .title("1st-Order Lowpass Filter (fc = 10 kHz)")
        .benchmark()
        .show_phase()
        .db()
        .samples(SAMPLE_SIZE)
        .magnitude_range(-10.0, 10.0)
        .log()
        .frequency_range(20.0, 20_000.0)
        .save("test_output/lowpass.png");
}

#[test]
fn plot_chain() {
    let cutoff = SAMPLE_RATE.div(std::f32::consts::PI)
        * f32::atan(
            f32::tan(std::f32::consts::PI * 10_000f32 / SAMPLE_RATE)
                .div((2f32.sqrt() - 1f32).sqrt()),
        );

    let chain_filter = chain!(
        FirstOrderFilter::new_lpf(SAMPLE_RATE, cutoff),
        FirstOrderFilter::new_lpf(SAMPLE_RATE, cutoff),
    );

    dsp_test::response(chain_filter, SAMPLE_RATE)
        .title("Chained Lowpass Filter (fc = 10 kHz)")
        .benchmark()
        .show_phase()
        .db()
        .samples(SAMPLE_SIZE)
        .magnitude_range(-10.0, 10.0)
        .log()
        .frequency_range(20.0, 20_000.0)
        .save("test_output/chain_lowpass.png");
}

#[test]
fn plot_temporal_filter() {
    let filter = FirstOrderFilter::new_lpf(SAMPLE_RATE, 2_000.0);

    dsp_test::temporal_response(filter, SAMPLE_RATE)
        .title("Lowpass Filter Temporal Response (fc = 2 kHz, 1 kHz Square)")
        .levels(0.1, 1.0)
        .benchmark()
        .save("test_output/temporal_lowpass.png");
}

#[test]
fn plot_2nd_order_lpf() {
    let filter = SecondOrderFilter::new_lpf(SAMPLE_RATE, 10_000f32);

    dsp_test::response(filter, SAMPLE_RATE)
        .title("2nd-Order Lowpass Filter (fc = 10 kHz)")
        .benchmark()
        .show_phase()
        .db()
        .samples(SAMPLE_SIZE)
        .magnitude_range(-10.0, 10.0)
        .log()
        .frequency_range(20.0, 20_000.0)
        .save("test_output/2ndlowpass.png");
}

#[test]
fn plot_2nd_order_hpf() {
    let filter = SecondOrderFilter::new_hpf(SAMPLE_RATE, 10_000f32);

    dsp_test::response(filter, SAMPLE_RATE)
        .title("2nd-Order Highpass Filter (fc = 10 kHz)")
        .benchmark()
        .show_phase()
        .db()
        .samples(SAMPLE_SIZE)
        .magnitude_range(-80.0, 10.0)
        .log()
        .frequency_range(20.0, 20_000.0)
        .save("test_output/2ndhighpass.png");
}

#[test]
fn verify_2nd_order_hpf_10k() {
    use rustfft::num_complex::Complex;

    let sample_rate = 48_000.0f32;
    let cutoff = 10_000.0f32;

    let k = (std::f32::consts::PI * cutoff / sample_rate).tan();
    let a_0 = 1.0 + std::f32::consts::SQRT_2 * k + k * k;
    let b_0 = 1.0 / a_0;
    let b_1 = -2.0 * b_0;
    let b_2 = b_0;
    let a_1 = 2.0 * (k * k - 1.0) / a_0;
    let a_2 = (1.0 - std::f32::consts::SQRT_2 * k + k * k) / a_0;

    let eval_h = |f: f32| -> (f32, f32) {
        let omega = 2.0 * std::f32::consts::PI * f / sample_rate;
        let z = Complex::from_polar(1.0, omega);
        let z_inv = z.inv();
        let z_inv2 = z_inv * z_inv;

        let num = Complex::new(b_0, 0.0)
            + Complex::new(b_1, 0.0) * z_inv
            + Complex::new(b_2, 0.0) * z_inv2;
        let den = Complex::new(1.0, 0.0)
            + Complex::new(a_1, 0.0) * z_inv
            + Complex::new(a_2, 0.0) * z_inv2;
        let h = num / den;

        let mag_db = 20.0 * h.norm().log10();
        let phase_deg = h.im.atan2(h.re).to_degrees();
        (mag_db, phase_deg)
    };

    println!("\n=== 2nd-Order HPF (fc = 10 kHz, fs = 48 kHz) ===");
    for &f in &[
        100.0, 1_000.0, 5_000.0, 10_000.0, 15_000.0, 20_000.0, 24_000.0,
    ] {
        let (db, phase) = eval_h(f);
        println!(
            "f = {:>5.0} Hz | Mag = {:>7.2} dB | Phase = {:>6.1}°",
            f, db, phase
        );
    }

    let (db_cutoff, phase_cutoff) = eval_h(10_000.0);
    assert!(
        (db_cutoff - (-3.0103)).abs() < 1e-3,
        "Expected -3.01 dB at cutoff, got {}",
        db_cutoff
    );
    assert!(
        (phase_cutoff - 90.0).abs() < 1e-2,
        "Expected +90 deg phase at cutoff, got {}",
        phase_cutoff
    );

    let (db_nyquist, phase_nyquist) = eval_h(24_000.0);
    assert!(
        (db_nyquist - 0.0).abs() < 1e-3,
        "Expected 0 dB at Nyquist, got {}",
        db_nyquist
    );
    assert!(
        (phase_nyquist - 0.0).abs() < 1e-2,
        "Expected 0 deg phase at Nyquist, got {}",
        phase_nyquist
    );

    // Verify time-domain simulation of SecondOrderHighPass matches analytical transfer function
    let mut filter = SecondOrderFilter::new_hpf(sample_rate, cutoff);
    let n_fft = 16_384;
    let mut impulse_resp: Vec<Complex<f32>> = (0..n_fft)
        .map(|i| {
            let x = if i == 0 { 1.0 } else { 0.0 };
            Complex::new(filter.process_sample(x), 0.0)
        })
        .collect();

    let mut planner = rustfft::FftPlanner::new();
    let fft = planner.plan_fft_forward(n_fft);
    fft.process(&mut impulse_resp);

    for &test_f in &[500.0, 1000.0, 5000.0, 10_000.0, 15_000.0, 20_000.0] {
        let bin = ((test_f / sample_rate) * n_fft as f32).round() as usize;
        let actual_f = bin as f32 * sample_rate / n_fft as f32;
        let sim_mag_db = 20.0 * impulse_resp[bin].norm().log10();
        let (expected_mag_db, _) = eval_h(actual_f);
        assert!(
            (sim_mag_db - expected_mag_db).abs() < 0.05,
            "Mismatch at {} Hz: sim = {:.2} dB, expected = {:.2} dB",
            actual_f,
            sim_mag_db,
            expected_mag_db
        );
    }
}
