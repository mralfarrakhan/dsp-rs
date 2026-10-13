mod common;

use common::dsp_test;
use dsp_rs::{
    BiquadFilter, Coefficients, FilterType, Zero, chain, gain::Gain, parallel,
    processor::Processor,
};

const SAMPLE_RATE: f32 = 48_000.0;
const SAMPLE_SIZE: usize = 65_536;

#[test]
fn test_parallel_empty() {
    let mut p = parallel!();
    assert_eq!(p, Zero);
    assert_eq!(p.process_sample(2.5), 0.0);

    let mut buf = [1.0, 2.0, 3.0];
    p.process_buffer(&mut buf);
    assert_eq!(buf, [0.0, 0.0, 0.0]);
}

#[test]
fn test_parallel_single() {
    let mut p = parallel!(Gain::new(2.0));
    assert_eq!(p.head, Gain::new(2.0));
    assert_eq!(p.tail, Zero);
    assert_eq!(p.process_sample(3.0), 6.0);

    let mut buf = [1.0, 2.0, 3.0];
    p.process_buffer(&mut buf);
    assert_eq!(buf, [2.0, 4.0, 6.0]);
}

#[test]
fn test_parallel_two() {
    let mut p = parallel!(Gain::new(2.0), Gain::new(3.0));
    // 1.0 * 2.0 + 1.0 * 3.0 = 5.0
    assert_eq!(p.process_sample(1.0), 5.0);

    let mut buf = [1.0, 2.0, 3.0];
    p.process_buffer(&mut buf);
    assert_eq!(buf, [5.0, 10.0, 15.0]);
}

#[test]
fn test_parallel_nested_three() {
    let mut p = parallel!(Gain::new(1.0), Gain::new(2.0), Gain::new(3.0),);
    // 2.0 * (1 + 2 + 3) = 12.0
    assert_eq!(p.process_sample(2.0), 12.0);

    let mut buf = [1.0, 2.0, -1.0];
    p.process_buffer(&mut buf);
    assert_eq!(buf, [6.0, 12.0, -6.0]);
}

#[test]
fn test_parallel_hlist_structure() {
    let p = parallel!(Gain::new(2.0), Gain::new(3.0));
    // Canonical HList: Parallel<Gain, Parallel<Gain, Zero>>
    assert_eq!(p.head, Gain::new(2.0));
    assert_eq!(p.tail.head, Gain::new(3.0));
    assert_eq!(p.tail.tail, Zero);
}

#[test]
fn test_parallel_buffer_equivalence() {
    let lpf1 = BiquadFilter::new(
        Coefficients::butterworth(FilterType::LowPass, SAMPLE_RATE, 1000.0).unwrap(),
    );
    let lpf2 = BiquadFilter::new(
        Coefficients::butterworth(FilterType::LowPass, SAMPLE_RATE, 5000.0).unwrap(),
    );

    let mut p1 = parallel!(lpf1, lpf2);
    let mut p2 = parallel!(lpf1, lpf2);

    let input = [0.5f32, -0.25, 0.8, -0.1, 0.0, 0.3];
    let mut buf = input;
    p1.process_buffer(&mut buf);

    for (i, &x) in input.iter().enumerate() {
        let expected = p2.process_sample(x);
        assert!((buf[i] - expected).abs() < 1e-6);
    }
}

#[test]
fn plot_parallel() {
    let lower_crossover = 1_000.0;
    let upper_crossover = 10_000.0;

    let lpf_low = BiquadFilter::new(
        Coefficients::butterworth(FilterType::LowPass, SAMPLE_RATE, lower_crossover).unwrap(),
    );
    let hpf_low = BiquadFilter::new(
        Coefficients::butterworth(FilterType::HighPass, SAMPLE_RATE, lower_crossover).unwrap(),
    );
    let lpf_high = BiquadFilter::new(
        Coefficients::butterworth(FilterType::LowPass, SAMPLE_RATE, upper_crossover).unwrap(),
    );
    let hpf_high = BiquadFilter::new(
        Coefficients::butterworth(FilterType::HighPass, SAMPLE_RATE, upper_crossover).unwrap(),
    );
    let apf_high = BiquadFilter::new(
        Coefficients::butterworth(FilterType::AllPass, SAMPLE_RATE, upper_crossover).unwrap(),
    );

    let first_band = chain!(lpf_low, lpf_low, apf_high);

    let second_band = chain!(hpf_low, hpf_low, lpf_high, lpf_high);

    let third_band = chain!(hpf_low, hpf_low, hpf_high, hpf_high);

    let parallel_filter = parallel!(first_band, second_band, third_band);

    dsp_test::response(parallel_filter, SAMPLE_RATE)
        .title("3-Band LR4 Crossover (fcl = 1 kHz, fcu = 10kHz)")
        .benchmark()
        .show_phase()
        .db()
        .samples(SAMPLE_SIZE)
        .magnitude_range(-10.0, 10.0)
        .log()
        .frequency_range(20.0, 20_000.0)
        .save("test_output/parallel.png");
}

#[test]
fn bench_perf() {
    use dsp_rs::chain;
    use std::time::Instant;

    let iters = 100;
    let mut buf = vec![0.5f32; SAMPLE_SIZE];

    let lpf = BiquadFilter::new(
        Coefficients::butterworth(FilterType::LowPass, SAMPLE_RATE, 10_000.0).unwrap(),
    );
    let mut single = lpf;
    let mut ch = chain!(lpf, lpf);
    let mut par = parallel!(lpf, lpf);

    // Warmup
    single.process_buffer(&mut buf);
    ch.process_buffer(&mut buf);
    par.process_buffer(&mut buf);

    let start = Instant::now();
    for _ in 0..iters {
        single.process_buffer(&mut buf);
    }
    let dur_single = start.elapsed() / (iters as u32);

    let start = Instant::now();
    for _ in 0..iters {
        ch.process_buffer(&mut buf);
    }
    let dur_chain = start.elapsed() / (iters as u32);

    let start = Instant::now();
    for _ in 0..iters {
        par.process_buffer(&mut buf);
    }
    let dur_par = start.elapsed() / (iters as u32);

    let total_samples = SAMPLE_SIZE as f64;
    println!(
        "\n=== PERFORMANCE REPORT (Buffer size: {}) ===",
        SAMPLE_SIZE
    );
    println!(
        "Single Biquad:     {:>8.2?} | {:>6.2} MSa/s ({:.2} ns/sample)",
        dur_single,
        (total_samples / dur_single.as_secs_f64()) / 1e6,
        (dur_single.as_secs_f64() * 1e9) / total_samples
    );
    println!(
        "Chain (2x series): {:>8.2?} | {:>6.2} MSa/s ({:.2} ns/sample)",
        dur_chain,
        (total_samples / dur_chain.as_secs_f64()) / 1e6,
        (dur_chain.as_secs_f64() * 1e9) / total_samples
    );
    println!(
        "Parallel (2x sum): {:>8.2?} | {:>6.2} MSa/s ({:.2} ns/sample)",
        dur_par,
        (total_samples / dur_par.as_secs_f64()) / 1e6,
        (dur_par.as_secs_f64() * 1e9) / total_samples
    );
}
