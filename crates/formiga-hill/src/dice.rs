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

    /// An index below `len`, each as likely as the others, from one roll; `None`, with nothing
    /// rolled, when there is nothing to choose from.
    pub fn index(&mut self, len: usize) -> Option<usize> {
        if len == 0 {
            return None;
        }
        // A roll a hair under 1 can still come to `len` once multiplied, so it wraps round.
        Some((self.unit() * len as f32) as usize % len)
    }

    /// One of `items`, each as likely as the others, from one roll; `None`, with nothing rolled,
    /// when there are none.
    pub fn choose<'a, T>(&mut self, items: &'a [T]) -> Option<&'a T> {
        self.index(items.len()).map(|index| &items[index])
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
    fn an_index_rolls_as_the_sum_written_out_did() {
        let (mut dice, mut by_hand) = (Dice::new(11), Dice::new(11));
        for len in (1..200).chain([1000, 4096, 1 << 20, 16_777_217]) {
            for _ in 0..20 {
                let expected = (by_hand.unit() * len as f32) as usize % len;
                assert_eq!(dice.index(len), Some(expected));
            }
        }
        let items = ["moss", "fern", "pip"];
        for _ in 0..50 {
            let expected = items[(by_hand.unit() * 3.0) as usize % 3];
            assert_eq!(dice.choose(&items), Some(&expected));
        }
    }

    #[test]
    fn nothing_to_choose_from_rolls_nothing() {
        let (mut dice, mut untouched) = (Dice::new(5), Dice::new(5));
        assert_eq!(dice.index(0), None);
        assert_eq!(dice.choose::<u8>(&[]), None);
        assert_eq!(dice.next_u64(), untouched.next_u64());
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
