//! Sudoku solving: a fast [`exact`] uniqueness solver used in the puzzle
//! construction hot loop, and a [`technique`] module implementing
//! human-style solving techniques for difficulty grading. These two are
//! kept deliberately independent: `exact` never depends on `technique` or
//! vice versa.

pub mod exact;
pub mod technique;
