use num_traits::Float;

pub trait Denormal: Float {
    #[inline]
    fn flush_denormal(self) -> Self {
        if self != Self::zero() && self.abs() < Self::min_positive_value() {
            Self::zero()
        } else {
            self
        }
    }

    #[inline]
    fn assign_flush(&mut self, new_value: Self) {
        *self = new_value.flush_denormal();
    }
}

impl<F: Float> Denormal for F {}
