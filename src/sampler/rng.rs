use rand::{RngCore, SeedableRng};
use rand_pcg::Pcg32;

use crate::core::ONE_MINUS_EPSILON;

#[derive(Debug, Clone)]
pub struct RNG {
    rng: Pcg32,
    // seed: [u8; 8]
}

impl RNG {
    pub fn init(sequence_idx: u64) -> Self {
        let mut rng = Self {
            rng: Pcg32::seed_from_u64(0u64)
        };

        rng.set_sequence(sequence_idx);

        rng
    }

    pub fn set_sequence(&mut self, sequence_idx: u64) {
        let seed = (sequence_idx as u32) as u64;
        let stream = (sequence_idx >> 32) as u64;

        self.rng = Pcg32::new(seed, stream);
    }

    pub fn uniform_u32(&mut self) -> u32 {
        self.rng.next_u32()
    }

    pub fn uniform_u32_bounded(&mut self, b: u32) -> u32 {
        let threshold = b.wrapping_neg() % b;
        loop {
            let r = self.uniform_u32();
            if r >= threshold {
                return r % b;
            }
        }
    }

    pub fn uniform_float(&mut self) -> f32 {
        let value = self.uniform_u32() as f32 * (1.0 / (1u64 << 32) as f32);

        value.min(ONE_MINUS_EPSILON)
    }
}