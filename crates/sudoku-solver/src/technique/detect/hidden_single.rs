//! Hidden Single: a digit that has only one possible cell left within a
//! unit (block, row, or column), even though that cell may have other
//! candidates too. Two techniques share this detection logic, differing
//! only in which unit kind they scan — matching the project's difficulty
//! hierarchy, which rates block-scoped hidden singles as easier than
//! row/column ones.

use crate::technique::grid::{box_cells, col_cells, row_cells, CandidateGrid};
use crate::technique::solve::{Technique, TechniqueEffect, TechniqueHint};

/// Scans one 9-cell unit for a hidden single: a digit with exactly one
/// remaining empty candidate cell in the unit, and not already placed
/// anywhere else in it.
fn find_hidden_single_in_unit(grid: &CandidateGrid, unit: &[usize; 9]) -> Option<(usize, u8)> {
    'digit: for digit in 1..=9u8 {
        let mut count = 0u8;
        let mut at = 0usize;
        for &idx in unit {
            if grid.value_at(idx) == digit {
                continue 'digit; // already placed in this unit: not hidden anywhere
            }
            if grid.value_at(idx) == 0 && grid.has_candidate(idx, digit) {
                count += 1;
                at = idx;
                if count > 1 {
                    continue 'digit;
                }
            }
        }
        if count == 1 {
            return Some((at, digit));
        }
    }
    None
}

pub struct HiddenSingleBlock;

impl Technique for HiddenSingleBlock {
    fn id(&self) -> &'static str {
        "hidden_single_block"
    }

    fn find_hint(&self, grid: &CandidateGrid) -> Option<TechniqueHint> {
        for b in 0..9 {
            if let Some((idx, digit)) = find_hidden_single_in_unit(grid, &box_cells(b)) {
                return Some(TechniqueHint {
                    technique_id: self.id(),
                    effect: TechniqueEffect::Place { idx, digit },
                });
            }
        }
        None
    }
}

pub struct HiddenSingleLine;

impl Technique for HiddenSingleLine {
    fn id(&self) -> &'static str {
        "hidden_single_line"
    }

    fn find_hint(&self, grid: &CandidateGrid) -> Option<TechniqueHint> {
        for r in 0..9 {
            if let Some((idx, digit)) = find_hidden_single_in_unit(grid, &row_cells(r)) {
                return Some(TechniqueHint {
                    technique_id: self.id(),
                    effect: TechniqueEffect::Place { idx, digit },
                });
            }
        }
        for c in 0..9 {
            if let Some((idx, digit)) = find_hidden_single_in_unit(grid, &col_cells(c)) {
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
    fn empty_board_has_no_hidden_single() {
        let grid = CandidateGrid::from_board(&Board::empty()).unwrap();
        assert!(HiddenSingleBlock.find_hint(&grid).is_none());
        assert!(HiddenSingleLine.find_hint(&grid).is_none());
    }

    #[test]
    fn finds_hidden_single_in_block() {
        let mut grid = CandidateGrid::from_board(&Board::empty()).unwrap();
        for &idx in &box_cells(0) {
            if idx != 0 {
                grid.remove_candidate(idx, 5);
            }
        }
        let hint = HiddenSingleBlock.find_hint(&grid).unwrap();
        assert_eq!(hint.technique_id, "hidden_single_block");
        match hint.effect {
            TechniqueEffect::Place { idx, digit } => {
                assert_eq!(idx, 0);
                assert_eq!(digit, 5);
            }
            _ => panic!("expected a placement"),
        }
    }

    #[test]
    fn finds_hidden_single_in_row() {
        let mut grid = CandidateGrid::from_board(&Board::empty()).unwrap();
        for &idx in &row_cells(3) {
            if idx != row_cells(3)[7] {
                grid.remove_candidate(idx, 2);
            }
        }
        let hint = HiddenSingleLine.find_hint(&grid).unwrap();
        match hint.effect {
            TechniqueEffect::Place { idx, digit } => {
                assert_eq!(idx, row_cells(3)[7]);
                assert_eq!(digit, 2);
            }
            _ => panic!("expected a placement"),
        }
    }

    #[test]
    fn ignores_digit_already_placed_in_the_unit() {
        let mut grid = CandidateGrid::from_board(&Board::empty()).unwrap();
        // Low-level primitive: marks idx 0 as placed without propagating to
        // peers, so other box-0 cells still (inconsistently) list 5 as a
        // candidate — exactly the case the "already placed" guard exists for.
        grid.set_value(0, 5);
        assert!(HiddenSingleBlock.find_hint(&grid).is_none());
    }
}
