#![no_std]

//! # dsp-rs
//!
//! A lightweight, zero-allocation, `#![no_std]` digital signal processing (DSP) library in Rust.
//! Designed for real-time audio applications on desktop and embedded microcontrollers (such as ESP32, STM32).
//!
//! ## Features
//!
//! - **[`Processor`]:** The fundamental trait for DSP units, with sample-by-sample and block-rate processing.
//! - **Heterogeneous Pipelines:** Serial pipelines with [`Chain`] / [`chain!`], and parallel mixing with [`Parallel`] / [`parallel!`].
//! - **Filters:** Direct Form II Transposed [`BiquadFilter`] with [`Coefficients`] generators for Low-Pass, High-Pass, and All-Pass responses.
//! - **Delay & Reverb:** Non-owning, stack-safe [`DelayLine`], feedback [`Delay`], and vintage Schroeder [`Reverb`].
//! - **Multi-Channel & Buffers:** Planar [`Buffer`] slicing and monomorphized [`MultiChannelProcessor`].
//! - **Zero Allocation:** 100% `#![no_std]` compliant with no heap memory requirements.
//!
//! ## Quick Start
//!
//! ```rust
//! use dsp_rs::{BiquadFilter, Coefficients, FilterType, Gain, Processor, chain};
//!
//! let sample_rate = 48_000.0f32;
//! let coeffs = Coefficients::butterworth(FilterType::LowPass, sample_rate, 1_000.0).unwrap();
//! let filter = BiquadFilter::new(coeffs);
//! let gain = Gain::from_db(-6.0f32).unwrap();
//!
//! let mut pipeline = chain!(filter, gain);
//!
//! let mut buffer = [0.5f32; 64];
//! pipeline.process_buffer(&mut buffer);
//! ```

pub mod buffer;
pub mod chain;
pub mod common;
pub mod delay;
pub mod filter;
pub mod gain;
pub mod parallel;
pub mod processor;
pub mod reverb;

pub use buffer::Buffer;
pub use chain::Chain;
pub use common::{Nil, Zero};
pub use delay::{Delay, DelayLine, DelayParameters};
pub use filter::{BiquadFilter, Coefficients, FilterType};
pub use gain::{Gain, GainParameters};
pub use parallel::Parallel;
pub use processor::{MultiChannelProcessor, Processor};
pub use reverb::{
    AllPassDelay, CombFilter, REVERB_MIN_BUFFER_SAMPLES, Reverb, ReverbParameters,
};
