use arcstr::ArcStr;
use rand::{distributions::Alphanumeric, distributions::DistString, Rng};

use crate::generator::NexmarkGenerator;

const MIN_STRING_LENGTH: usize = 3;

pub(super) fn next_string<R: Rng>(rng: &mut R, max_length: usize) -> ArcStr {
    let len = rng.gen_range(MIN_STRING_LENGTH..=max_length);
    ArcStr::from(Alphanumeric.sample_string(rng, len))
}

fn next_extra<R: Rng>(rng: &mut R, current_size: usize, desired_average_size: usize) -> ArcStr {
    if current_size > desired_average_size {
        return String::new().into();
    }

    let avg_extra_size = desired_average_size - current_size;
    let delta = (avg_extra_size as f32 * 0.2).round() as usize;
    if delta == 0 {
        return String::new().into();
    }

    let desired_size =
        rng.gen_range((avg_extra_size.saturating_sub(delta))..=(avg_extra_size + delta));
    ArcStr::from(Alphanumeric.sample_string(rng, desired_size))
}

impl<R: Rng> NexmarkGenerator<R> {
    pub fn next_string(&mut self, max_length: usize) -> ArcStr {
        next_string(&mut self.rng, max_length)
    }

    pub fn next_extra(&mut self, current_size: usize, desired_average_size: usize) -> ArcStr {
        next_extra(&mut self.rng, current_size, desired_average_size)
    }
}
