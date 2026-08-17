//! Generating puzzles rated by the technique-based solver: one shared
//! retry engine ([`generate_matching`]) powers both difficulty-targeted
//! generation and training mode (maximizing one technique's occurrence
//! count) — see `sudoku-cli`'s `generate` command for how each is built on
//! top of it.

use rand_core::RngCore;

use sudoku_core::board::Board;
use sudoku_solver::technique::TechniqueSolver;

use crate::dig::PuzzleConstructionStrategy;
use crate::fill::FullGridGenerator;

/// A generated puzzle together with its technique-based difficulty rating.
pub struct GradedPuzzle {
    pub puzzle: Board,
    pub solution: Board,
    pub clue_count: u8,
    /// Whether the solver's configured techniques fully solved the puzzle.
    /// When `false`, the solver got stuck partway through (some cell needed
    /// a technique the hierarchy doesn't cover) — `max_weight` and
    /// `technique_counts` then only describe the moves made *before*
    /// getting stuck, not the puzzle's true difficulty, since it was never
    /// fully explained. [`generate_matching`] rejects these outright.
    pub solved: bool,
    /// Weight of the hardest technique required. `None` only if the puzzle
    /// was already solved with no technique needed — can't happen from a
    /// real dig, since digging always removes at least one clue.
    pub max_weight: Option<u32>,
    /// How many times each technique fired, in first-applied order.
    pub technique_counts: Vec<(String, u32)>,
}

/// One generate-dig-rate cycle: a fresh solved grid, a puzzle constructed
/// from it via `strategy`, then graded by `solver`.
///
/// Returns `None` only if `solver` reports the constructed puzzle as
/// contradictory — which should never happen from a valid dig, but is kept
/// as an `Option` rather than unwrapped internally, consistent with the
/// rest of this engine's policy of never panicking on a solver result.
pub fn generate_graded_puzzle(
    strategy: &mut dyn PuzzleConstructionStrategy,
    fill_gen: &mut FullGridGenerator,
    solver: &TechniqueSolver,
    rng: &mut dyn RngCore,
) -> Option<GradedPuzzle> {
    let solution = fill_gen.next_grid(rng);
    let outcome = strategy.construct(&solution, rng);
    let trace = solver.solve(&outcome.puzzle)?;

    Some(GradedPuzzle {
        puzzle: outcome.puzzle,
        solution,
        clue_count: outcome.clue_count,
        solved: trace.solved,
        max_weight: trace.max_weight,
        technique_counts: trace.technique_counts,
    })
}

/// Retries [`generate_graded_puzzle`] up to `attempts` times, keeping the
/// highest-scoring accepted candidate (ties keep whichever was found
/// first). `accept` returns `None` to reject a candidate, `Some(score)` to
/// accept it. Returns `None` if no attempt was accepted.
///
/// Candidates the solver didn't fully solve (`GradedPuzzle::solved ==
/// false`) are rejected unconditionally, before `accept` is even called —
/// their reported difficulty only reflects the moves made before the
/// solver got stuck, not the puzzle's true difficulty, so they can't
/// honestly satisfy a difficulty-range or training request. Rejecting them
/// here (rather than relying on every caller's `accept` closure to
/// remember) protects every mode built on this function.
///
/// This one retry engine powers both difficulty-range filtering (accept
/// everything in range with a constant score, so the first match wins) and
/// training mode (score = the target technique's occurrence count, so
/// repeated attempts converge toward puzzles that use it more).
pub fn generate_matching(
    strategy: &mut dyn PuzzleConstructionStrategy,
    fill_gen: &mut FullGridGenerator,
    solver: &TechniqueSolver,
    rng: &mut dyn RngCore,
    attempts: u32,
    mut accept: impl FnMut(&GradedPuzzle) -> Option<i64>,
) -> Option<GradedPuzzle> {
    let mut best: Option<(i64, GradedPuzzle)> = None;

    for _ in 0..attempts {
        let Some(candidate) = generate_graded_puzzle(strategy, fill_gen, solver, rng) else {
            continue;
        };
        if !candidate.solved {
            continue;
        }
        let Some(score) = accept(&candidate) else {
            continue;
        };
        if best
            .as_ref()
            .is_none_or(|(best_score, _)| score > *best_score)
        {
            best = Some((score, candidate));
        }
    }

    best.map(|(_, puzzle)| puzzle)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dig::TopDown;
    use crate::fill::FillMode;
    use rand::{rngs::StdRng, SeedableRng};
    use sudoku_solver::technique::TechniqueHierarchy;

    fn default_solver() -> TechniqueSolver {
        TechniqueSolver::from_hierarchy(&TechniqueHierarchy::default_hierarchy())
    }

    #[test]
    fn generate_graded_puzzle_produces_a_rated_puzzle() {
        let mut rng = StdRng::seed_from_u64(1);
        let mut fill_gen = FullGridGenerator::new(FillMode::RandomBacktracking);
        let mut strategy = TopDown::default();
        let solver = default_solver();

        let graded =
            generate_graded_puzzle(&mut strategy, &mut fill_gen, &solver, &mut rng).unwrap();
        assert!(graded.clue_count < 81);
        assert!(
            graded.max_weight.is_some(),
            "digging always removes at least one clue, so some technique must be needed"
        );
    }

    #[test]
    fn generate_matching_returns_none_when_everything_is_rejected() {
        let mut rng = StdRng::seed_from_u64(2);
        let mut fill_gen = FullGridGenerator::new(FillMode::RandomBacktracking);
        let mut strategy = TopDown::default();
        let solver = default_solver();

        let result =
            generate_matching(&mut strategy, &mut fill_gen, &solver, &mut rng, 5, |_| None);
        assert!(result.is_none());
    }

    #[test]
    fn generate_matching_rejects_unsolved_candidates_regardless_of_accept() {
        use sudoku_solver::technique::TechniqueHierarchy;

        // A hierarchy with only Naked Single can't fully solve a real,
        // locally-minimal dug puzzle, so every candidate should come back
        // `solved: false` and get rejected before `accept` even matters
        // (it unconditionally accepts everything here, with score 0).
        let toml = r#"
            [[technique]]
            id = "naked_single"
            name = "Naked Single"
            category = "Easy"
            weight = 100
        "#;
        let hierarchy = TechniqueHierarchy::from_toml_str(toml).unwrap();
        let solver = TechniqueSolver::from_hierarchy(&hierarchy);

        let mut rng = StdRng::seed_from_u64(5);
        let mut fill_gen = FullGridGenerator::new(FillMode::RandomBacktracking);
        let mut strategy = TopDown::default();

        let result = generate_matching(&mut strategy, &mut fill_gen, &solver, &mut rng, 10, |_| {
            Some(0)
        });
        assert!(
            result.is_none(),
            "no unsolved candidate should ever be accepted, no matter what `accept` says"
        );
    }

    #[test]
    fn generate_matching_accepts_first_candidate_on_score_ties() {
        let mut rng = StdRng::seed_from_u64(4);
        let mut fill_gen = FullGridGenerator::new(FillMode::RandomBacktracking);
        let mut strategy = TopDown::default();
        let solver = default_solver();

        let mut first_seen: Option<Board> = None;
        let result = generate_matching(&mut strategy, &mut fill_gen, &solver, &mut rng, 3, |g| {
            if first_seen.is_none() {
                first_seen = Some(g.puzzle);
            }
            Some(0)
        });

        assert_eq!(result.unwrap().puzzle, first_seen.unwrap());
    }
}
