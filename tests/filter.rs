mod common;

use common::dsp_test;
use dsp_rs::filter::FirstOrderLowPass;

const SAMPLE_RATE: f32 = 48_000.0;

#[test]
fn plot_filter() {
    let filter = FirstOrderLowPass::new(SAMPLE_RATE, 10_000.0);

    dsp_test::response(filter, SAMPLE_RATE)
        .db()
        .magnitude_range(-10.0, 10.0)
        .log()
        .frequency_range(20.0, 20_000.0)
        .save("test_output/lowpass.png");
}
