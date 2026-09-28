use num_traits::Float;

pub struct Buffer<'a, F: Float> {
    data: &'a mut [F],
    channels: usize,
    samples: usize,
}

impl<'a, F: Float> Buffer<'a, F> {
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

    pub fn channels(&self) -> usize {
        self.channels
    }

    pub fn samples(&self) -> usize {
        self.samples
    }

    pub fn channel(&self, channel: usize) -> Option<&[F]> {
        let start = channel.checked_mul(self.samples)?;
        let end = start.checked_add(self.samples)?;

        self.data.get(start..end)
    }

    pub fn channel_mut(&mut self, channel: usize) -> Option<&mut [F]> {
        let start = channel.checked_mul(self.samples)?;
        let end = start.checked_add(self.samples)?;

        self.data.get_mut(start..end)
    }

    pub fn channels_iter(&self) -> impl Iterator<Item = &[F]> {
        self.data.chunks_exact(self.samples)
    }

    pub fn channels_iter_mut(&mut self) -> impl Iterator<Item = &mut [F]> {
        self.data.chunks_exact_mut(self.samples)
    }
}
