use num_traits::Float;

pub trait Denormal: Float {
    #[inline]
    fn zapgremlins(self) -> Self {
        if self.abs() < Self::min_positive_value() {
            Self::zero()
        } else {
            self
        }
    }

    #[inline]
    fn assign_zapped(&mut self, new_value: Self) {
        *self = new_value.zapgremlins();
    }
}

impl<F: Float> Denormal for F {}
