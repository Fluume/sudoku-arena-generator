use rand::seq::SliceRandom;
use rand_core::RngCore;

use sudoku_core::board::{Board, CELLS};
use sudoku_solver::exact::has_unique_solution;

use super::symmetry::Symmetry;
use super::{DigOutcome, PuzzleConstructionStrategy};

/// Classic top-down construction: remove clues from a solved grid, one
/// symmetry orbit at a time in random order, keeping each removal only if
/// the puzzle still has a unique solution.
///
/// With the default `Symmetry::None`, every orbit is a single cell — this
/// is exactly the classic single-cell algorithm. With any other
/// `Symmetry`, cells are removed and restored in their full orbit
/// together, so the finished puzzle's clue pattern always respects that
/// symmetry.
///
/// The result is *locally* minimal (no single remaining orbit can be
/// removed without breaking uniqueness), not necessarily a globally
/// minimum-clue puzzle — that is a much harder, unrelated problem (the
/// 17-clue-minimum question).
pub struct TopDown {
    symmetry: Symmetry,
    min_clues: u8,
}

impl TopDown {
    pub fn new(symmetry: Symmetry) -> Self {
        TopDown {
            symmetry,
            min_clues: 0,
        }
    }

    /// Never removes an orbit that would drop the clue count below
    /// `min_clues`. A floor, not an exact target — the puzzle may end up
    /// with more clues than this if the random removal order runs out of
    /// safely-removable orbits before reaching it.
    pub fn with_min_clues(mut self, min_clues: u8) -> Self {
        self.min_clues = min_clues;
        self
    }
}

impl Default for TopDown {
    fn default() -> Self {
        TopDown::new(Symmetry::None)
    }
}

impl PuzzleConstructionStrategy for TopDown {
    fn construct(&mut self, solved: &Board, rng: &mut dyn RngCore) -> DigOutcome {
        let mut board = *solved;
        let mut orbits = build_orbits(self.symmetry);
        orbits.shuffle(rng);

        for orbit in orbits {
            // Orbits are disjoint and each processed exactly once, so this
            // is exactly the clue count that would remain if removed.
            if board.clue_count() as usize - orbit.len() < self.min_clues as usize {
                continue;
            }

            let saved: Vec<(usize, u8)> = orbit
                .iter()
                .map(|&idx| (idx, board.get_by_index(idx)))
                .collect();
            for &idx in &orbit {
                board.set_by_index(idx, 0);
            }
            if !has_unique_solution(&board) {
                for (idx, digit) in saved {
                    board.set_by_index(idx, digit);
                }
            }
        }

        DigOutcome {
            clue_count: board.clue_count(),
            puzzle: board,
        }
    }
}

/// Partitions all 81 cells into their symmetry orbits, each orbit appearing
/// exactly once.
fn build_orbits(symmetry: Symmetry) -> Vec<Vec<usize>> {
    let mut visited = [false; CELLS];
    let mut orbits = Vec::new();
    for idx in 0..CELLS {
        if visited[idx] {
            continue;
        }
        let orbit = symmetry.orbit(idx);
        for &member in &orbit {
            visited[member] = true;
        }
        orbits.push(orbit);
    }
    orbits
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{rngs::StdRng, SeedableRng};

    const SOLUTION: &str =
        "534678912672195348198342567859761423426853791713924856961537284287419635345286179";

    #[test]
    fn top_down_output_is_a_subset_of_the_solution_with_unique_solution() {
        let solved = Board::from_line(SOLUTION).unwrap();
        let mut rng = StdRng::seed_from_u64(3);
        let outcome = TopDown::default().construct(&solved, &mut rng);

        assert!(
            outcome.clue_count < 81,
            "digging should remove at least one clue"
        );
        assert!(has_unique_solution(&outcome.puzzle));

        for idx in 0..CELLS {
            let digit = outcome.puzzle.get_by_index(idx);
            if digit != 0 {
                assert_eq!(
                    digit,
                    solved.get_by_index(idx),
                    "clue must match the solved grid"
                );
            }
        }
    }

    #[test]
    fn top_down_result_is_locally_minimal() {
        let solved = Board::from_line(SOLUTION).unwrap();
        let mut rng = StdRng::seed_from_u64(4);
        let outcome = TopDown::default().construct(&solved, &mut rng);

        for idx in 0..CELLS {
            let digit = outcome.puzzle.get_by_index(idx);
            if digit == 0 {
                continue;
            }
            let mut probe = outcome.puzzle;
            probe.set_by_index(idx, 0);
            assert!(
                !has_unique_solution(&probe),
                "removing clue at index {idx} should not preserve uniqueness"
            );
        }
    }

    #[test]
    fn min_clues_is_never_violated() {
        let solved = Board::from_line(SOLUTION).unwrap();
        for seed in 0..10 {
            let mut rng = StdRng::seed_from_u64(seed);
            let outcome = TopDown::default()
                .with_min_clues(30)
                .construct(&solved, &mut rng);
            assert!(
                outcome.clue_count >= 30,
                "seed {seed}: clue count {} fell below the requested floor of 30",
                outcome.clue_count
            );
        }
    }

    #[test]
    fn min_clues_above_the_natural_minimum_yields_more_clues_than_unconstrained_digging() {
        let solved = Board::from_line(SOLUTION).unwrap();
        let mut rng_unconstrained = StdRng::seed_from_u64(4);
        let unconstrained = TopDown::default().construct(&solved, &mut rng_unconstrained);

        let mut rng_floored = StdRng::seed_from_u64(4);
        let floored = TopDown::default()
            .with_min_clues(40)
            .construct(&solved, &mut rng_floored);

        assert!(
            unconstrained.clue_count < 40,
            "test assumes unconstrained digging drops below 40 for this seed"
        );
        assert!(floored.clue_count >= 40);
    }

    #[test]
    fn zero_min_clues_behaves_like_unconstrained_digging() {
        let solved = Board::from_line(SOLUTION).unwrap();
        let mut rng_a = StdRng::seed_from_u64(7);
        let a = TopDown::default().construct(&solved, &mut rng_a);
        let mut rng_b = StdRng::seed_from_u64(7);
        let b = TopDown::default()
            .with_min_clues(0)
            .construct(&solved, &mut rng_b);
        assert_eq!(a.puzzle, b.puzzle);
    }

    #[test]
    fn central_symmetry_produces_a_centrally_symmetric_clue_pattern() {
        let solved = Board::from_line(SOLUTION).unwrap();
        for seed in 0..10 {
            let mut rng = StdRng::seed_from_u64(seed);
            let outcome = TopDown::new(Symmetry::Central).construct(&solved, &mut rng);
            for idx in 0..CELLS {
                let is_clue = outcome.puzzle.get_by_index(idx) != 0;
                for member in Symmetry::Central.orbit(idx) {
                    let member_is_clue = outcome.puzzle.get_by_index(member) != 0;
                    assert_eq!(is_clue, member_is_clue, "seed {seed}: cell {idx} and orbit member {member} disagree on clue presence");
                }
            }
        }
    }

    #[test]
    fn full_symmetry_produces_a_fully_symmetric_clue_pattern() {
        let solved = Board::from_line(SOLUTION).unwrap();
        for seed in 0..10 {
            let mut rng = StdRng::seed_from_u64(seed);
            let outcome = TopDown::new(Symmetry::Full).construct(&solved, &mut rng);
            for idx in 0..CELLS {
                let is_clue = outcome.puzzle.get_by_index(idx) != 0;
                for member in Symmetry::Full.orbit(idx) {
                    let member_is_clue = outcome.puzzle.get_by_index(member) != 0;
                    assert_eq!(is_clue, member_is_clue, "seed {seed}: cell {idx} and orbit member {member} disagree on clue presence");
                }
            }
        }
    }
}
