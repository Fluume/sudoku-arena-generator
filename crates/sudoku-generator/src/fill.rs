//! Full-grid (solved) generation. Deliberately separate from
//! [`sudoku_solver::exact`]: that solver's job is uniqueness *checking*,
//! not producing fresh solved grids, so the randomized-fill traversal below
//! lives here instead, reusing `sudoku_core`'s bitmask bookkeeping.

use rand::seq::SliceRandom;
use rand_core::RngCore;

use sudoku_core::bitmask::{bit_to_digit, UnitMasks};
use sudoku_core::board::{col_of, row_of, Board, CELLS};
use sudoku_core::symmetry::RandomSymmetry;

/// How a [`FullGridGenerator`] produces each new solved grid.
#[derive(Debug, Clone, Copy)]
pub enum FillMode {
    /// Build a fresh grid via randomized backtracking every time.
    RandomBacktracking,
    /// Reuse one seed grid, applying a fresh random symmetry transform on
    /// each call, and only rebuild the seed (via randomized backtracking)
    /// every `reseed_every` calls. Much cheaper than backtracking from
    /// scratch every time, while periodic reseeding keeps generation from
    /// staying stuck sampling a single isomorphism class.
    CanonicalWithTransforms { reseed_every: u32 },
}

/// Produces solved (fully filled, valid) Sudoku grids according to a
/// [`FillMode`].
pub struct FullGridGenerator {
    mode: FillMode,
    seed: Option<Board>,
    since_reseed: u32,
}

impl FullGridGenerator {
    pub fn new(mode: FillMode) -> Self {
        FullGridGenerator {
            mode,
            seed: None,
            since_reseed: 0,
        }
    }

    /// Produces the next solved grid.
    ///
    /// Must be reused across a whole generation batch rather than
    /// reconstructed per puzzle: a fresh instance would never accumulate
    /// `since_reseed`, silently turning "reseed every N puzzles" into
    /// "reseed every puzzle".
    pub fn next_grid(&mut self, rng: &mut dyn RngCore) -> Board {
        match self.mode {
            FillMode::RandomBacktracking => random_fill(rng),
            FillMode::CanonicalWithTransforms { reseed_every } => {
                let needs_new_seed = self.seed.is_none() || self.since_reseed >= reseed_every;
                if needs_new_seed {
                    self.seed = Some(random_fill(rng));
                    self.since_reseed = 0;
                }
                self.since_reseed += 1;

                let seed = self.seed.as_ref().expect("seed was just populated above");
                RandomSymmetry::random(rng).apply(seed)
            }
        }
    }
}

/// Fills an empty grid via randomized backtracking: at each cell, candidate
/// digits are tried in random order, backtracking on dead ends. Always
/// succeeds for an initially-empty grid (a solution always exists).
fn random_fill(rng: &mut dyn RngCore) -> Board {
    let mut board = Board::empty();
    let mut masks = UnitMasks::empty();
    let filled = fill_from(&mut board, &mut masks, 0, rng);
    debug_assert!(
        filled,
        "randomized backtracking must be able to fill an empty grid"
    );
    board
}

fn fill_from(board: &mut Board, masks: &mut UnitMasks, idx: usize, rng: &mut dyn RngCore) -> bool {
    if idx == CELLS {
        return true;
    }

    let (r, c) = (row_of(idx), col_of(idx));
    let mut candidates = digits_of(masks.candidates_at(r, c));
    candidates.shuffle(rng);

    for digit in candidates {
        board.set(r, c, digit);
        masks.place(r, c, digit);

        if fill_from(board, masks, idx + 1, rng) {
            return true;
        }

        masks.unplace(r, c, digit);
        board.set(r, c, 0);
    }

    false
}

fn digits_of(mask: u16) -> Vec<u8> {
    let mut remaining = mask;
    let mut out = Vec::with_capacity(mask.count_ones() as usize);
    while remaining != 0 {
        let bit = remaining & remaining.wrapping_neg();
        remaining &= remaining - 1;
        out.push(bit_to_digit(bit));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{rngs::StdRng, SeedableRng};
    use sudoku_solver::exact::has_unique_solution;

    #[test]
    fn random_backtracking_produces_complete_valid_grid() {
        let mut rng = StdRng::seed_from_u64(1);
        let mut gen = FullGridGenerator::new(FillMode::RandomBacktracking);
        for _ in 0..5 {
            let board = gen.next_grid(&mut rng);
            assert!(board.is_complete());
            assert!(UnitMasks::from_board(&board).is_some());
            assert!(has_unique_solution(&board));
        }
    }

    #[test]
    fn canonical_with_transforms_produces_complete_valid_grids() {
        let mut rng = StdRng::seed_from_u64(2);
        let mut gen = FullGridGenerator::new(FillMode::CanonicalWithTransforms { reseed_every: 3 });
        for _ in 0..10 {
            let board = gen.next_grid(&mut rng);
            assert!(board.is_complete());
            assert!(UnitMasks::from_board(&board).is_some());
        }
    }

    #[test]
    fn same_seed_is_reproducible() {
        let mut rng_a = StdRng::seed_from_u64(7);
        let mut rng_b = StdRng::seed_from_u64(7);
        let mut gen_a = FullGridGenerator::new(FillMode::RandomBacktracking);
        let mut gen_b = FullGridGenerator::new(FillMode::RandomBacktracking);
        assert_eq!(gen_a.next_grid(&mut rng_a), gen_b.next_grid(&mut rng_b));
    }
}
