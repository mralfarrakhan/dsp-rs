# dsp-rs

A lightweight, zero-allocation, `#![no_std]` digital signal processing (DSP) library in Rust (2024 edition).

Designed for real-time audio and embedded microcontrollers (ESP32, STM32, Daisy Seed) as well as desktop audio engines and plugins.

---

## Features

- **`#![no_std]` & Zero-Allocation:** All structures operate on the stack or over caller-provided borrowed slices (`&'a mut [F]`). Zero heap allocation required.
- **Heterogeneous Pipelines (HList):**
  - [`chain!`]: Serial composition connecting stages in series with zero runtime dispatch overhead.
  - [`parallel!`]: Parallel branching with additive mixing.
- **2nd-Order Biquad Filters:** Direct Form II Transposed (DF2T) implementation with denormal protection, and analytical coefficient generators for Low-Pass, High-Pass, and All-Pass responses.
- **Delay & Reverb:**
  - Non-owning [`DelayLine`] (16 bytes on the stack, safe for embedded FreeRTOS tasks).
  - Standalone feedback [`Delay`] effect with wet/dry mix.
  - Vintage algorithmic Schroeder [`Reverb`] (4 parallel damped comb filters + 2 series all-pass diffusers) partitioned from a single slice.
- **Multi-Channel & Buffers:** Planar [`Buffer`] slicing and monomorphized [`MultiChannelProcessor`] with parameter broadcasting.
- **Gain:** Scalar gain and decibel (`dB`) conversions with inlined SIMD buffer operations.

---

## Architecture

| Component | Type | Description |
| :--- | :--- | :--- |
| [`Processor`](src/processor.rs) | Trait | Base abstraction with `process_sample`, `process_buffer`, and block-rate parameter handling. |
| [`MultiChannelProcessor`](src/processor.rs) | Struct | Runs `[P; CHANNELS]` across planar channels with parameter broadcasting. |
| [`BiquadFilter`](src/filter.rs) | Struct | DF2T 2nd-order IIR filter with `Coefficients::from_params` and `butterworth`. |
| [`DelayLine`](src/delay.rs) | Struct | Non-owning circular ring buffer over `&'a mut [F]` with integer sample reads. |
| [`Delay`](src/delay.rs) | Struct | Feedback delay with `DelayParameters { delay_samples, feedback, wet }`. |
| [`Reverb`](src/reverb.rs) | Struct | Schroeder reverb partitioned from a single slice (`REVERB_MIN_BUFFER_SAMPLES`). |
| [`Gain`](src/gain.rs) | Struct | Linear & dB volume stage with vectorized buffer execution. |
| [`Buffer`](src/buffer.rs) | Struct | Non-interleaved planar multi-channel audio buffer view. |
| [`Chain`](src/chain.rs) | Combinator | Series evaluation terminating on `Nil`. |
| [`Parallel`](src/parallel.rs) | Combinator | Parallel summing terminating on `Zero`. |

---

## Quick Start

### 1. Filtering & Chaining

```rust
use dsp_rs::{BiquadFilter, Coefficients, FilterType, Gain, Processor, chain};

let sample_rate = 48_000.0f32;

// 1. Calculate Butterworth filter coefficients
let coeffs = Coefficients::butterworth(FilterType::LowPass, sample_rate, 1_000.0).unwrap();
let filter = BiquadFilter::new(coeffs);
let gain = Gain::from_db(-6.0f32).unwrap();

// 2. Compose into a serial pipeline
let mut pipeline = chain!(filter, gain);

// 3. Process an audio buffer in-place
let mut buffer = [0.5f32; 128];
pipeline.process_buffer(&mut buffer);
```

### 2. Feedback Delay

```rust
use dsp_rs::{Delay, DelayParameters, Processor};

let sample_rate = 48_000.0f32;

// Backing buffer can live in static memory, PSRAM, or stack
let mut mem = [0.0f32; 24_000]; // 500 ms at 48 kHz
let params = DelayParameters::from_time(sample_rate, 0.25, 0.4, 0.5).unwrap();
let mut delay = Delay::new(&mut mem, params);

let output = delay.process_sample(1.0);
```

### 3. Schroeder Reverb

```rust
use dsp_rs::{Processor, REVERB_MIN_BUFFER_SAMPLES, Reverb, ReverbParameters};

// Single partitioned buffer (~5,934 samples ≈ 23.7 KB in f32)
let mut reverb_mem = [0.0f32; REVERB_MIN_BUFFER_SAMPLES];
let params = ReverbParameters::new(0.7, 0.3, 0.4); // 70% room, 30% damping, 40% wet
let mut reverb = Reverb::new(&mut reverb_mem, params).unwrap();

let wet_sample = reverb.process_sample(0.5);
```

---

## Embedded & Microcontroller Safety (ESP32)

Delay and reverb effects require internal memory buffers. On microcontrollers running FreeRTOS (such as the ESP32), task stacks are limited (typically 4 KB – 8 KB), so allocating large arrays on the stack can cause instant crashes.

`dsp-rs` uses **non-owning borrowed slices (`&'a mut [F]`)**:
- `DelayLine` and `Reverb` structs occupy only **16 to 80 bytes on the stack**.
- Backing memory can safely be stored in **static BSS**, **external PSRAM**, or a custom heap allocator:

```rust
// Static BSS: zero stack usage!
static mut REVERB_MEM: [f32; REVERB_MIN_BUFFER_SAMPLES] = [0.0; REVERB_MIN_BUFFER_SAMPLES];

let reverb = unsafe {
    Reverb::new(&mut REVERB_MEM, ReverbParameters::new(0.6, 0.2, 0.5)).unwrap()
};
```

---

## Testing & Verification

Run the test suite, benchmarks, and doc tests:

```bash
cargo test
cargo clippy --all-targets
cargo doc --no-deps
cargo check --no-default-features --features libm
```

---

## License

Licensed under either of [Apache License, Version 2.0](LICENSE) or [MIT License](LICENSE) at your option.
