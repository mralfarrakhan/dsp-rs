mod common;

use common::dsp_test;
use dsp_rs::{chain, filter::FirstOrderLowPass};

const SAMPLE_RATE: f32 = 48_000.0;

#[test]
fn plot_filter() {
    let filter = FirstOrderLowPass::new(SAMPLE_RATE, 10_000.0);

    dsp_test::response(filter, SAMPLE_RATE)
        .title("1st-Order Lowpass Filter (fc = 10 kHz)")
        .benchmark()
        .db()
        .magnitude_range(-10.0, 10.0)
        .log()
        .frequency_range(20.0, 20_000.0)
        .save("test_output/lowpass.png");
}

#[test]
fn plot_chain() {
    let chain_filter = chain!(
        FirstOrderLowPass::new(SAMPLE_RATE, 10_000.0),
        FirstOrderLowPass::new(SAMPLE_RATE, 10_000.0),
    );

    dsp_test::response(chain_filter, SAMPLE_RATE)
        .title("Chained Lowpass Filter (2x fc = 10 kHz)")
        .benchmark()
        .db()
        .magnitude_range(-10.0, 10.0)
        .log()
        .frequency_range(20.0, 20_000.0)
        .save("test_output/chain_lowpass.png");
}
