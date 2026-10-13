//! Planar multi-channel audio buffer views.

use num_traits::Float;

/// A non-owning view over a planar (non-interleaved) multi-channel audio buffer.
///
/// Data is assumed to be laid out sequentially by channel:
/// `[channel_0_samples..., channel_1_samples..., ...]`.
pub struct Buffer<'a, F: Float> {
    data: &'a mut [F],
    channels: usize,
    samples: usize,
}

impl<'a, F: Float> Buffer<'a, F> {
    /// Creates a new buffer view over the provided slice.
    ///
    /// Returns `None` if:
    /// - `channels == 0` or `samples == 0`
    /// - `channels * samples` overflows
    /// - `data.len() < channels * samples`
    pub fn new(data: &'a mut [F], channels: usize, samples: usize) -> Option<Self> {
        if channels == 0 || samples == 0 {
            return None;
        }

        let len = channels.checked_mul(samples)?;

        if data.len() < len {
            return None;
        }

        Some(Self {
            data: &mut data[..len],
            channels,
            samples,
        })
    }

    /// Returns the number of channels.
    pub fn channels(&self) -> usize {
        self.channels
    }

    /// Returns the number of samples per channel.
    pub fn samples(&self) -> usize {
        self.samples
    }

    /// Returns an immutable slice for the specified channel, or `None` if out of bounds.
    pub fn channel(&self, channel: usize) -> Option<&[F]> {
        let start = channel.checked_mul(self.samples)?;
        let end = start.checked_add(self.samples)?;

        self.data.get(start..end)
    }

    /// Returns a mutable slice for the specified channel, or `None` if out of bounds.
    pub fn channel_mut(&mut self, channel: usize) -> Option<&mut [F]> {
        let start = channel.checked_mul(self.samples)?;
        let end = start.checked_add(self.samples)?;

        self.data.get_mut(start..end)
    }

    /// Returns an iterator over immutable channel slices.
    pub fn channels_iter(&self) -> impl Iterator<Item = &[F]> {
        self.data.chunks_exact(self.samples)
    }

    /// Returns an iterator over mutable channel slices.
    pub fn channels_iter_mut(&mut self) -> impl Iterator<Item = &mut [F]> {
        self.data.chunks_exact_mut(self.samples)
    }
}
