use rand::distributions::{Distribution, WeightedIndex};
use rand::prelude::*;
use std::collections::HashMap;

pub enum Sampler {
    Zipf(Zipf),
    Uniform(Uniform),
}

impl Sampler {
    pub fn new(pattern: &str, delay_interval: u64, zipf_alpha: f64) -> Self {
        match pattern {
            "zipf" => Sampler::Zipf(Zipf::new(delay_interval, zipf_alpha)),
            "uniform" => Sampler::Uniform(Uniform::new(delay_interval)),
            _ => {
                panic!("unsupported delay_pattern")
            }
        }
    }

    pub fn sample(&self, rng: &mut SmallRng) -> u64 {
        match self {
            Sampler::Zipf(z) => z.sample(rng),
            Sampler::Uniform(u) => u.sample(rng),
        }
    }
}

pub struct Zipf {
    dist: WeightedIndex<f64>,
}

impl Zipf {
    pub fn new(n: u64, alpha: f64) -> Self {
        let mut freq = HashMap::new();

        let mut sum = 0.0;
        for i in 1..=n {
            let f = 1.0 / (i as f64).powf(alpha);
            freq.insert(i, f);
            sum += f;
        }
        for f in freq.values_mut() {
            *f /= sum;
        }

        let dist = WeightedIndex::new(freq.values()).unwrap();
        Zipf { dist }
    }
    pub fn sample(&self, rng: &mut SmallRng) -> u64 {
        self.dist.sample(rng) as u64
    }
}

pub struct Uniform {
    delay_interval: u64,
}

impl Uniform {
    pub fn new(delay_interval: u64) -> Self {
        Self { delay_interval }
    }
    pub fn sample(&self, rng: &mut SmallRng) -> u64 {
        rng.gen_range(0..=self.delay_interval)
    }
}
