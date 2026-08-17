use rand_core::RngCore;

use sudoku_core::board::Board;

use crate::dig::PuzzleConstructionStrategy;
use crate::fill::FullGridGenerator;

/// A generated puzzle together with its known solution and clue count.
pub struct GeneratedPuzzle {
    pub puzzle: Board,
    pub solution: Board,
    pub clue_count: u8,
}

/// Generates a single puzzle: a fresh solved grid from `fill_gen`, then a
/// puzzle constructed from it via `strategy`.
///
/// Callers generating a batch must reuse the same `fill_gen` (and, for
/// reproducibility, the same `rng`) across every puzzle in the batch — see
/// [`FullGridGenerator::next_grid`].
pub fn generate_puzzle(
    strategy: &mut dyn PuzzleConstructionStrategy,
    fill_gen: &mut FullGridGenerator,
    rng: &mut dyn RngCore,
) -> GeneratedPuzzle {
    let solution = fill_gen.next_grid(rng);
    let outcome = strategy.construct(&solution, rng);
    GeneratedPuzzle {
        puzzle: outcome.puzzle,
        solution,
        clue_count: outcome.clue_count,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dig::TopDown;
    use crate::fill::FillMode;
    use rand::{rngs::StdRng, SeedableRng};
    use sudoku_solver::exact::has_unique_solution;

    #[test]
    fn generate_puzzle_produces_a_valid_uniquely_solvable_puzzle() {
        let mut rng = StdRng::seed_from_u64(11);
        let mut fill_gen = FullGridGenerator::new(FillMode::RandomBacktracking);
        let mut strategy = TopDown::default();

        let generated = generate_puzzle(&mut strategy, &mut fill_gen, &mut rng);

        assert!(generated.solution.is_complete());
        assert!(has_unique_solution(&generated.puzzle));
        assert_eq!(generated.clue_count, generated.puzzle.clue_count());
        assert!(generated.clue_count < 81);
    }
}
