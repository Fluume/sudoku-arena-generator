//! Naked Single: an empty cell with exactly one remaining candidate.

use sudoku_core::bitmask::bit_to_digit;
use sudoku_core::board::CELLS;

use crate::technique::grid::CandidateGrid;
use crate::technique::solve::{Technique, TechniqueEffect, TechniqueHint};

pub struct NakedSingle;

impl Technique for NakedSingle {
    fn id(&self) -> &'static str {
        "naked_single"
    }

    fn find_hint(&self, grid: &CandidateGrid) -> Option<TechniqueHint> {
        for idx in 0..CELLS {
            if grid.value_at(idx) == 0 && grid.candidate_count(idx) == 1 {
                let digit = bit_to_digit(grid.candidates_at(idx));
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
    use sudoku_core::board::Board;

    #[test]
    fn empty_board_has_no_naked_single() {
        let grid = CandidateGrid::from_board(&Board::empty()).unwrap();
        assert!(NakedSingle.find_hint(&grid).is_none());
    }

    #[test]
    fn finds_naked_single() {
        let mut grid = CandidateGrid::from_board(&Board::empty()).unwrap();
        for digit in 1..=9u8 {
            if digit != 6 {
                grid.remove_candidate(40, digit);
            }
        }
        let hint = NakedSingle.find_hint(&grid).unwrap();
        match hint.effect {
            TechniqueEffect::Place { idx, digit } => {
                assert_eq!(idx, 40);
                assert_eq!(digit, 6);
            }
            _ => panic!("expected a placement"),
        }
    }

    #[test]
    fn scans_in_row_major_index_order() {
        let mut grid = CandidateGrid::from_board(&Board::empty()).unwrap();
        for digit in 1..=9u8 {
            if digit != 3 {
                grid.remove_candidate(50, digit);
            }
            if digit != 7 {
                grid.remove_candidate(10, digit);
            }
        }
        // idx 10 comes before idx 50 in row-major order, so it must win.
        let hint = NakedSingle.find_hint(&grid).unwrap();
        match hint.effect {
            TechniqueEffect::Place { idx, digit } => {
                assert_eq!(idx, 10);
                assert_eq!(digit, 7);
            }
            _ => panic!("expected a placement"),
        }
    }
}
