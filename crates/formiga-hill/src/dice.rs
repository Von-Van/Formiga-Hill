//! Small deterministic randomness for free play: seeded from who is playing, so a visit unfolds
//! the same way from the same start, which keeps behaviour reproducible in tests.

/// SplitMix64: tiny, fast, and well mixed enough for choosing where to wander.
#[derive(Clone, Debug)]
pub struct Dice(u64);

impl Dice {
    pub fn new(seed: u64) -> Self {
        Self(seed ^ 0x9e37_79b9_7f4a_7c15)
    }

    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    /// A number in `[0, 1)`.
    pub fn unit(&mut self) -> f32 {
        (self.next_u64() >> 40) as f32 / (1u64 << 24) as f32
    }

    pub fn range(&mut self, low: f32, high: f32) -> f32 {
        low + (high - low) * self.unit()
    }

    pub fn chance(&mut self, probability: f32) -> bool {
        self.unit() < probability
    }

    /// An index chosen in proportion to `weights`; `None` if none of them is above zero.
    pub fn weighted(&mut self, weights: &[f32]) -> Option<usize> {
        let total: f32 = weights.iter().map(|weight| weight.max(0.0)).sum();
        if total <= 0.0 {
            return None;
        }
        let mut roll = self.unit() * total;
        for (index, weight) in weights.iter().enumerate() {
            let weight = weight.max(0.0);
            if roll < weight {
                return Some(index);
            }
            roll -= weight;
        }
        weights.iter().rposition(|weight| *weight > 0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_same_seed_rolls_the_same() {
        let (mut a, mut b) = (Dice::new(7), Dice::new(7));
        for _ in 0..10 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
        assert_ne!(Dice::new(7).next_u64(), Dice::new(8).next_u64());
    }

    #[test]
    fn units_stay_in_range_and_spread() {
        let mut dice = Dice::new(1);
        let rolls: Vec<f32> = (0..2000).map(|_| dice.unit()).collect();
        assert!(rolls.iter().all(|roll| (0.0..1.0).contains(roll)));
        let low = rolls.iter().filter(|roll| **roll < 0.5).count();
        assert!((900..1100).contains(&low));
    }

    #[test]
    fn weighted_choices_follow_their_weights() {
        let mut dice = Dice::new(3);
        let mut counts = [0; 3];
        for _ in 0..3000 {
            counts[dice.weighted(&[1.0, 0.0, 3.0]).unwrap()] += 1;
        }
        assert_eq!(counts[1], 0);
        assert!(counts[2] > counts[0] * 2);
        assert_eq!(dice.weighted(&[0.0, -1.0]), None);
    }
}
