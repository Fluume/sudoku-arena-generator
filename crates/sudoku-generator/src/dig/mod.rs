//! Puzzle construction ("digging"): turning a solved grid into a puzzle.
//! Exposed as a pluggable [`PuzzleConstructionStrategy`] trait so
//! alternative strategies (bottom-up, Berthier-style controlled-bias) can
//! be added later without changing callers — see ROADMAP.md.

use rand_core::RngCore;
use sudoku_core::board::Board;

/// Outcome of constructing a puzzle from a solved grid.
#[non_exhaustive]
pub struct DigOutcome {
    pub puzzle: Board,
    pub clue_count: u8,
}

/// A strategy for turning a solved grid into a puzzle. The trait says
/// nothing about which solver(s) an implementation consults internally
/// (uniqueness checking only, technique-based grading, or both), which is
/// what lets future strategies slot in as pure additions.
pub trait PuzzleConstructionStrategy {
    fn construct(&mut self, solved: &Board, rng: &mut dyn RngCore) -> DigOutcome;
}

mod symmetry;
mod top_down;
pub use symmetry::Symmetry;
pub use top_down::TopDown;
