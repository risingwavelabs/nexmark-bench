use rand::Rng;

use crate::generator::NexmarkGenerator;

impl<R: Rng> NexmarkGenerator<R> {
    pub fn next_price(&mut self) -> usize {
        (10.0_f32.powf(self.rng.gen_range(0.0..1.0) * 6.0) * 100.0).ceil() as usize
    }
}
