mod common;

use common::dsp_test;
use dsp_rs::{chain, filter::FirstOrderLowPass};

const SAMPLE_RATE: f32 = 48_000.0;
const SAMPLE_SIZE: usize = 65_536;

#[test]
fn plot_filter() {
    let filter = FirstOrderLowPass::new(SAMPLE_RATE, 10_000.0);

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
    let cutoff = 10_000.0 / (2.0f32.sqrt() - 1.0).sqrt();

    let chain_filter = chain!(
        FirstOrderLowPass::new(SAMPLE_RATE, cutoff),
        FirstOrderLowPass::new(SAMPLE_RATE, cutoff),
    );

    dsp_test::response(chain_filter, SAMPLE_RATE)
        .title("Chained Lowpass Filter (2x fc = 10 kHz)")
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
    let filter = FirstOrderLowPass::new(SAMPLE_RATE, 2_000.0);

    dsp_test::temporal_response(filter, SAMPLE_RATE)
        .title("Lowpass Filter Temporal Response (fc = 2 kHz, 1 kHz Square)")
        .levels(0.1, 1.0)
        .benchmark()
        .save("test_output/temporal_lowpass.png");
}
