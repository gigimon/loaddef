use std::sync::Mutex;

use rand::{Rng, SeedableRng, rngs::StdRng};

pub struct RandomSource {
    inner: RandomSourceInner,
}

enum RandomSourceInner {
    ThreadLocal,
    Seeded(Mutex<StdRng>),
}

impl RandomSource {
    pub fn new(seed: Option<u64>) -> Self {
        let inner = match seed {
            Some(value) => RandomSourceInner::Seeded(Mutex::new(StdRng::seed_from_u64(value))),
            None => RandomSourceInner::ThreadLocal,
        };
        Self { inner }
    }

    pub fn gen_u64_inclusive(&self, min: u64, max: u64) -> u64 {
        match &self.inner {
            RandomSourceInner::ThreadLocal => rand::rng().random_range(min..=max),
            RandomSourceInner::Seeded(rng) => rng
                .lock()
                .expect("seeded rng poisoned")
                .random_range(min..=max),
        }
    }

    pub fn gen_usize_inclusive(&self, min: usize, max: usize) -> usize {
        match &self.inner {
            RandomSourceInner::ThreadLocal => rand::rng().random_range(min..=max),
            RandomSourceInner::Seeded(rng) => rng
                .lock()
                .expect("seeded rng poisoned")
                .random_range(min..=max),
        }
    }

    pub fn fill_bytes(&self, bytes: &mut [u8]) {
        match &self.inner {
            RandomSourceInner::ThreadLocal => rand::rng().fill(bytes),
            RandomSourceInner::Seeded(rng) => rng.lock().expect("seeded rng poisoned").fill(bytes),
        }
    }
}
