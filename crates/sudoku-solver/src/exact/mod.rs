//! The exact solver: fast solution-uniqueness checking, and nothing else.
//! This is the hot loop used while digging clues out of a solved grid, so
//! its public surface stays deliberately narrow — it never grades
//! difficulty or reports *which* techniques would solve a puzzle.

mod backtrack;
mod bitgrid;

pub use backtrack::{count_solutions_capped, has_unique_solution, SolutionCount};
