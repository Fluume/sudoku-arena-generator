//! Validity-preserving transforms on a *solved* board: band/stack
//! permutation, row/column permutation within a band/stack, digit
//! relabeling, and transposition. Composing any of these on a valid solved
//! grid yields another valid solved grid, which makes them a cheap way to
//! sample many grids from one canonical seed.

use rand_core::RngCore;

use crate::board::{Board, SIZE};

/// Fisher-Yates shuffle using only [`RngCore::next_u32`], so callers only
/// need `rand_core` rather than the full `rand` crate. The small modulo bias
/// this introduces is irrelevant for sampling Sudoku symmetries.
fn shuffle<T>(rng: &mut dyn RngCore, slice: &mut [T]) {
    for i in (1..slice.len()).rev() {
        let j = (rng.next_u32() as usize) % (i + 1);
        slice.swap(i, j);
    }
}

/// Builds a box-respecting permutation of `0..9`: the three groups of three
/// (bands, for rows; stacks, for columns) are shuffled among themselves, and
/// the three elements within each group are shuffled among themselves. This
/// is what keeps the permutation validity-preserving (an arbitrary row
/// permutation would break box constraints; this restricted one cannot).
fn random_unit_permutation(rng: &mut dyn RngCore) -> [usize; SIZE] {
    let mut groups = [0usize, 1, 2];
    shuffle(rng, &mut groups);

    let mut perm = [0usize; SIZE];
    let mut pos = 0;
    for &g in &groups {
        let mut items = [g * 3, g * 3 + 1, g * 3 + 2];
        shuffle(rng, &mut items);
        for item in items {
            perm[pos] = item;
            pos += 1;
        }
    }
    perm
}

fn random_digit_map(rng: &mut dyn RngCore) -> [u8; 9] {
    let mut digits = [1u8, 2, 3, 4, 5, 6, 7, 8, 9];
    shuffle(rng, &mut digits);
    digits
}

/// A randomly sampled composition of validity-preserving transforms.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RandomSymmetry {
    row_perm: [usize; SIZE],
    col_perm: [usize; SIZE],
    digit_map: [u8; 9],
    transpose: bool,
}

impl RandomSymmetry {
    /// Samples a random combination of band/stack permutation, in-band/
    /// in-stack permutation, digit relabeling, and transposition.
    pub fn random(rng: &mut dyn RngCore) -> Self {
        RandomSymmetry {
            row_perm: random_unit_permutation(rng),
            col_perm: random_unit_permutation(rng),
            digit_map: random_digit_map(rng),
            transpose: rng.next_u32().is_multiple_of(2),
        }
    }

    /// Applies the transform to `board`, producing a new board. If `board`
    /// is a valid solved grid, the result is too.
    pub fn apply(&self, board: &Board) -> Board {
        let mut result = Board::empty();
        for r in 0..SIZE {
            for c in 0..SIZE {
                let digit = board.get(self.row_perm[r], self.col_perm[c]);
                let mapped = if digit == 0 {
                    0
                } else {
                    self.digit_map[(digit - 1) as usize]
                };
                result.set(r, c, mapped);
            }
        }

        if self.transpose {
            let mut transposed = Board::empty();
            for r in 0..SIZE {
                for c in 0..SIZE {
                    transposed.set(r, c, result.get(c, r));
                }
            }
            result = transposed;
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bitmask::UnitMasks;

    // A fixed, known-valid solved grid used as a transform seed in tests.
    const SOLVED: &str =
        "534678912672195348198342567859761423426853791713924856961537284287419635345286179";

    /// A tiny deterministic RNG (xorshift) so symmetry tests don't depend on
    /// an external `rand` crate or system entropy.
    struct XorShift(u32);
    impl RngCore for XorShift {
        fn next_u32(&mut self) -> u32 {
            self.0 ^= self.0 << 13;
            self.0 ^= self.0 >> 17;
            self.0 ^= self.0 << 5;
            self.0
        }
        fn next_u64(&mut self) -> u64 {
            (self.next_u32() as u64) << 32 | self.next_u32() as u64
        }
        fn fill_bytes(&mut self, dest: &mut [u8]) {
            for chunk in dest.chunks_mut(4) {
                chunk.copy_from_slice(&self.next_u32().to_le_bytes()[..chunk.len()]);
            }
        }
        fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand_core::Error> {
            self.fill_bytes(dest);
            Ok(())
        }
    }

    #[test]
    fn transformed_solved_grid_stays_valid_and_complete() {
        let board = Board::from_line(SOLVED).unwrap();
        let mut rng = XorShift(0xC0FFEE);
        for _ in 0..50 {
            let symmetry = RandomSymmetry::random(&mut rng);
            let transformed = symmetry.apply(&board);
            assert!(transformed.is_complete());
            assert!(
                UnitMasks::from_board(&transformed).is_some(),
                "transform produced a contradictory grid"
            );
        }
    }

    #[test]
    fn identity_like_permutation_is_deterministic_for_fixed_seed() {
        let board = Board::from_line(SOLVED).unwrap();
        let mut rng_a = XorShift(42);
        let mut rng_b = XorShift(42);
        let a = RandomSymmetry::random(&mut rng_a).apply(&board);
        let b = RandomSymmetry::random(&mut rng_b).apply(&board);
        assert_eq!(a, b);
    }
}
