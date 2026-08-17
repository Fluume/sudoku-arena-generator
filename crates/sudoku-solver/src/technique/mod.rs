//! Technique-based difficulty grading: the reconfigurable technique
//! [`hierarchy`] config, the per-cell [`grid::CandidateGrid`] technique
//! detectors operate on, the [`detect`] modules implementing each
//! technique, and the [`solve::TechniqueSolver`] that drives them.

pub mod detect;
pub mod grid;
mod hierarchy;
pub mod solve;

pub use hierarchy::{ConfigError, TechniqueDef, TechniqueHierarchy};
pub use solve::{
    AppliedStep, SolveTrace, Technique, TechniqueEffect, TechniqueHint, TechniqueSolver,
};
