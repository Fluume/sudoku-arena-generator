//! Core Sudoku board representation shared across the whole engine:
//! the [`Board`] type, parsing/export, candidate bitmask bookkeeping
//! ([`bitmask`]), and validity-preserving symmetry transforms
//! ([`symmetry`]).
//!
//! This crate deliberately contains no solving or generation logic.

pub mod bitmask;
pub mod board;
pub mod error;
pub mod symmetry;

pub use board::{Board, CELLS, SIZE};
pub use error::ParseError;
