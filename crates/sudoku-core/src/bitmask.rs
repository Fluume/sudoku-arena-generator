//! Shared row/column/box candidate bookkeeping, used by both the exact solver
//! and full-grid generation. This module intentionally contains no search
//! policy (no backtracking, no cell-selection heuristics) — just the bitwise
//! primitives for tracking which digits are already used in each unit.

use crate::board::{box_of, col_of, index, row_of, Board, CELLS, SIZE};

/// Bitmask with all nine digit bits set (digits `1..=9` map to bits `0..=8`).
pub const FULL_MASK: u16 = 0b1_1111_1111;

#[inline]
pub const fn digit_to_bit(digit: u8) -> u16 {
    1 << (digit - 1)
}

#[inline]
pub fn bit_to_digit(bit: u16) -> u8 {
    debug_assert!(
        bit != 0 && bit.is_power_of_two(),
        "expected a single set bit"
    );
    bit.trailing_zeros() as u8 + 1
}

/// Tracks, per row/column/box, which digits are already placed.
///
/// A mask bit set means "this digit is used in this unit" (i.e. NOT a valid
/// candidate). Available candidates for a cell are the digits not used in its
/// row, column, or box.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnitMasks {
    pub row: [u16; SIZE],
    pub col: [u16; SIZE],
    pub box_: [u16; SIZE],
}

impl UnitMasks {
    /// An empty board's masks: no digit used anywhere.
    pub fn empty() -> Self {
        UnitMasks {
            row: [0; SIZE],
            col: [0; SIZE],
            box_: [0; SIZE],
        }
    }

    /// Builds masks from a board's filled cells. Returns `None` if the board
    /// is contradictory (the same digit appears twice in a row, column, or
    /// box), rather than panicking, since boards may come from untrusted
    /// input.
    pub fn from_board(board: &Board) -> Option<Self> {
        let mut masks = UnitMasks::empty();
        for idx in 0..CELLS {
            let digit = board.get_by_index(idx);
            if digit == 0 {
                continue;
            }
            let bit = digit_to_bit(digit);
            let (r, c, b) = (row_of(idx), col_of(idx), box_of(idx));
            if masks.row[r] & bit != 0 || masks.col[c] & bit != 0 || masks.box_[b] & bit != 0 {
                return None;
            }
            masks.row[r] |= bit;
            masks.col[c] |= bit;
            masks.box_[b] |= bit;
        }
        Some(masks)
    }

    /// Candidate digits (as a bitmask) still available at `(row, col)`.
    #[inline]
    pub fn candidates_at(&self, row: usize, col: usize) -> u16 {
        let b = box_of(index(row, col));
        FULL_MASK & !(self.row[row] | self.col[col] | self.box_[b])
    }

    /// Marks `digit` as used at `(row, col)`.
    #[inline]
    pub fn place(&mut self, row: usize, col: usize, digit: u8) {
        let bit = digit_to_bit(digit);
        let b = box_of(index(row, col));
        self.row[row] |= bit;
        self.col[col] |= bit;
        self.box_[b] |= bit;
    }

    /// Reverses a previous [`UnitMasks::place`] call for the same cell/digit.
    #[inline]
    pub fn unplace(&mut self, row: usize, col: usize, digit: u8) {
        let bit = digit_to_bit(digit);
        let b = box_of(index(row, col));
        self.row[row] &= !bit;
        self.col[col] &= !bit;
        self.box_[b] &= !bit;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn digit_bit_round_trip() {
        for d in 1..=9u8 {
            assert_eq!(bit_to_digit(digit_to_bit(d)), d);
        }
    }

    #[test]
    fn empty_board_has_full_candidates_everywhere() {
        let masks = UnitMasks::from_board(&Board::empty()).unwrap();
        assert_eq!(masks.candidates_at(0, 0), FULL_MASK);
        assert_eq!(masks.candidates_at(8, 8), FULL_MASK);
    }

    #[test]
    fn detects_row_contradiction() {
        let mut board = Board::empty();
        board.set(0, 0, 5);
        board.set(0, 1, 5);
        assert!(UnitMasks::from_board(&board).is_none());
    }

    #[test]
    fn detects_box_contradiction_without_row_or_col_overlap() {
        let mut board = Board::empty();
        board.set(0, 0, 7);
        board.set(1, 1, 7); // same box, different row/col
        assert!(UnitMasks::from_board(&board).is_none());
    }

    #[test]
    fn place_and_unplace_are_inverses() {
        let mut masks = UnitMasks::empty();
        let before = masks;
        masks.place(2, 3, 4);
        assert_ne!(masks.candidates_at(2, 3) & digit_to_bit(4), digit_to_bit(4));
        masks.unplace(2, 3, 4);
        assert_eq!(masks, before);
    }

    #[test]
    fn placed_digit_removed_from_row_col_box_candidates() {
        let mut masks = UnitMasks::empty();
        masks.place(4, 4, 9); // center cell, box 4
        let bit = digit_to_bit(9);
        assert_eq!(masks.candidates_at(4, 0) & bit, 0); // same row
        assert_eq!(masks.candidates_at(0, 4) & bit, 0); // same col
        assert_eq!(masks.candidates_at(3, 3) & bit, 0); // same box
        assert_eq!(masks.candidates_at(8, 8) & bit, bit); // unrelated cell keeps candidate
    }
}
