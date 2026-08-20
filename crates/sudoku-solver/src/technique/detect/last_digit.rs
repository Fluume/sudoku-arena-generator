//! Last Digit: the simplest solving step there is — a row, column, or block
//! with exactly one empty cell left, whose digit is whatever value 1-9 the
//! other 8 filled cells in that unit don't already contain. Unlike every
//! other technique here, this needs no candidate/pencil-mark reasoning at
//! all — it's pure counting over placed digits within a single unit. That's
//! why it's rated below even Hidden Single (block): it's the first
//! deduction absolute-beginner players learn, well before they ever track
//! candidates.

use sudoku_core::bitmask::{bit_to_digit, digit_to_bit, FULL_MASK};

use crate::technique::grid::{all_units, CandidateGrid};
use crate::technique::solve::{Technique, TechniqueEffect, TechniqueHint};

/// Scans one 9-cell unit for a single remaining empty cell, returning it
/// paired with the one digit 1-9 not already present elsewhere in the unit.
/// Returns `None` if the unit has zero or more than one empty cell.
fn find_last_digit_in_unit(grid: &CandidateGrid, unit: &[usize; 9]) -> Option<(usize, u8)> {
    let mut empty_at = None;
    let mut used_mask = 0u16;
    for &idx in unit {
        let value = grid.value_at(idx);
        if value == 0 {
            if empty_at.is_some() {
                return None; // more than one empty cell: not a last digit yet
            }
            empty_at = Some(idx);
        } else {
            used_mask |= digit_to_bit(value);
        }
    }
    let idx = empty_at?;
    Some((idx, bit_to_digit(FULL_MASK & !used_mask)))
}

pub struct LastDigit;

impl Technique for LastDigit {
    fn id(&self) -> &'static str {
        "last_digit"
    }

    fn find_hint(&self, grid: &CandidateGrid) -> Option<TechniqueHint> {
        for unit in all_units() {
            if let Some((idx, digit)) = find_last_digit_in_unit(grid, &unit) {
                return Some(TechniqueHint {
                    technique_id: self.id(),
                    effect: TechniqueEffect::Place { idx, digit },
                });
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::technique::grid::{box_cells, row_cells};
    use sudoku_core::board::Board;

    #[test]
    fn empty_board_has_no_last_digit() {
        let grid = CandidateGrid::from_board(&Board::empty()).unwrap();
        assert!(LastDigit.find_hint(&grid).is_none());
    }

    #[test]
    fn finds_the_missing_digit_in_a_block_with_one_empty_cell() {
        let mut board = Board::empty();
        // Fill 8 of box 0's 9 cells with digits 1..=8, leaving the 9th
        // (row 2, col 2) as the only gap — the missing digit is 9.
        let cells = box_cells(0);
        for (i, &d) in [1u8, 2, 3, 4, 5, 6, 7, 8].iter().enumerate() {
            board.set_by_index(cells[i], d);
        }
        let grid = CandidateGrid::from_board(&board).unwrap();
        let hint = LastDigit.find_hint(&grid).unwrap();
        assert_eq!(hint.technique_id, "last_digit");
        match hint.effect {
            TechniqueEffect::Place { idx, digit } => {
                assert_eq!(idx, cells[8]);
                assert_eq!(digit, 9);
            }
            _ => panic!("expected a placement"),
        }
    }

    #[test]
    fn ignores_a_unit_with_more_than_one_empty_cell() {
        let mut board = Board::empty();
        let cells = row_cells(0);
        for (i, &d) in [1u8, 2, 3, 4, 5, 6, 7].iter().enumerate() {
            board.set_by_index(cells[i], d);
        }
        let grid = CandidateGrid::from_board(&board).unwrap();
        assert!(LastDigit.find_hint(&grid).is_none());
    }

    #[test]
    fn ignores_a_unit_that_is_already_full() {
        let mut board = Board::empty();
        let cells = box_cells(0);
        for (i, &d) in [1u8, 2, 3, 4, 5, 6, 7, 8, 9].iter().enumerate() {
            board.set_by_index(cells[i], d);
        }
        let grid = CandidateGrid::from_board(&board).unwrap();
        assert!(LastDigit.find_hint(&grid).is_none());
    }
}
