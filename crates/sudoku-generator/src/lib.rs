//! Puzzle generation: producing solved grids ([`fill`]), turning them into
//! puzzles via a pluggable strategy ([`dig`]), orchestrating both
//! ([`generate_puzzle`]), rating puzzles against the technique solver for
//! difficulty-targeted and training-mode generation ([`grade`]), and
//! splitting a batch across threads ([`parallel_batches`]).

pub mod dig;
pub mod fill;
mod generate;
pub mod grade;
pub mod parallel;

pub use generate::{generate_puzzle, GeneratedPuzzle};
pub use grade::{generate_graded_puzzle, generate_matching, GradedPuzzle};
pub use parallel::parallel_batches;
