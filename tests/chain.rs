mod common;

use dsp_rs::{
    Nil, chain,
    filter::FirstOrderFilter,
    gain::Gain,
    processor::Processor,
};

const SAMPLE_RATE: f32 = 48_000.0;

#[test]
fn test_chain_empty() {
    let mut c = chain!();
    assert_eq!(c, Nil);
    assert_eq!(c.process_sample(2.5), 2.5);

    let mut buf = [1.0, 2.0, 3.0];
    c.process_buffer(&mut buf);
    assert_eq!(buf, [1.0, 2.0, 3.0]);
}

#[test]
fn test_chain_single() {
    let mut c = chain!(Gain::new(2.0));
    assert_eq!(c.head, Gain::new(2.0));
    assert_eq!(c.tail, Nil);
    assert_eq!(c.process_sample(3.0), 6.0);

    let mut buf = [1.0, 2.0, 3.0];
    c.process_buffer(&mut buf);
    assert_eq!(buf, [2.0, 4.0, 6.0]);
}

#[test]
fn test_chain_two() {
    let mut c = chain!(Gain::new(2.0), Gain::new(3.0));
    // 1.0 * 2.0 * 3.0 = 6.0
    assert_eq!(c.process_sample(1.0), 6.0);

    let mut buf = [1.0, 2.0, 3.0];
    c.process_buffer(&mut buf);
    assert_eq!(buf, [6.0, 12.0, 18.0]);
}

#[test]
fn test_chain_nested_three() {
    let mut c = chain!(Gain::new(1.0), Gain::new(2.0), Gain::new(3.0));
    // 2.0 * (1 * 2 * 3) = 12.0
    assert_eq!(c.process_sample(2.0), 12.0);

    let mut buf = [1.0, 2.0, -1.0];
    c.process_buffer(&mut buf);
    assert_eq!(buf, [6.0, 12.0, -6.0]);
}

#[test]
fn test_chain_hlist_structure() {
    let c = chain!(Gain::new(2.0), Gain::new(3.0));
    // Canonical HList: Chain<Gain, Chain<Gain, Nil>>
    assert_eq!(c.head, Gain::new(2.0));
    assert_eq!(c.tail.head, Gain::new(3.0));
    assert_eq!(c.tail.tail, Nil);
}

#[test]
fn test_chain_buffer_equivalence() {
    let mut c1 = chain!(
        FirstOrderFilter::new_lpf(SAMPLE_RATE, 1000.0),
        FirstOrderFilter::new_lpf(SAMPLE_RATE, 5000.0),
    );
    let mut c2 = chain!(
        FirstOrderFilter::new_lpf(SAMPLE_RATE, 1000.0),
        FirstOrderFilter::new_lpf(SAMPLE_RATE, 5000.0),
    );

    let input = [0.5f32, -0.25, 0.8, -0.1, 0.0, 0.3];
    let mut buf = input;
    c1.process_buffer(&mut buf);

    for (i, &x) in input.iter().enumerate() {
        let expected = c2.process_sample(x);
        assert!((buf[i] - expected).abs() < 1e-6);
    }
}
