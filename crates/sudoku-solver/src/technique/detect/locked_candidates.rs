//! Locked Candidates: Pointing (a digit confined, within one box, to a
//! single row or column → eliminate it from the rest of that row/column)
//! and Claiming (a digit confined, within one row or column, to a single
//! box → eliminate it from the rest of that box). Both are registered
//! under one hierarchy entry (`pointing_claiming`) at the same weight.
//!
//! The Java reference this is ported from only implements Pointing —
//! Claiming was written fresh here, as the mirror-image algorithm.

use sudoku_core::board::{box_of, col_of, row_of};

use crate::technique::grid::{box_cells, col_cells, row_cells, CandidateGrid};
use crate::technique::solve::{Technique, TechniqueEffect, TechniqueHint};

const ID: &str = "pointing_claiming";

fn pointing_hint(grid: &CandidateGrid) -> Option<TechniqueHint> {
    for b in 0..9 {
        let cells = box_cells(b);
        for digit in 1..=9u8 {
            let occurrences: Vec<usize> = cells
                .iter()
                .copied()
                .filter(|&idx| grid.value_at(idx) == 0 && grid.has_candidate(idx, digit))
                .collect();
            let Some(&first) = occurrences.first() else {
                continue;
            };

            if occurrences.iter().all(|&idx| row_of(idx) == row_of(first)) {
                let removals = eliminate_outside(grid, &row_cells(row_of(first)), b, digit);
                if !removals.is_empty() {
                    return Some(TechniqueHint {
                        technique_id: ID,
                        effect: TechniqueEffect::Eliminate(removals),
                    });
                }
            }
            if occurrences.iter().all(|&idx| col_of(idx) == col_of(first)) {
                let removals = eliminate_outside(grid, &col_cells(col_of(first)), b, digit);
                if !removals.is_empty() {
                    return Some(TechniqueHint {
                        technique_id: ID,
                        effect: TechniqueEffect::Eliminate(removals),
                    });
                }
            }
        }
    }
    None
}

/// Eliminates `digit` from every empty candidate cell in `line` that does
/// NOT belong to box `exclude_box`.
fn eliminate_outside(
    grid: &CandidateGrid,
    line: &[usize; 9],
    exclude_box: usize,
    digit: u8,
) -> Vec<(usize, u8)> {
    line.iter()
        .copied()
        .filter(|&idx| {
            box_of(idx) != exclude_box && grid.value_at(idx) == 0 && grid.has_candidate(idx, digit)
        })
        .map(|idx| (idx, digit))
        .collect()
}

fn claiming_hint(grid: &CandidateGrid) -> Option<TechniqueHint> {
    for r in 0..9 {
        if let Some(hint) = claiming_in_unit(grid, &row_cells(r)) {
            return Some(hint);
        }
    }
    for c in 0..9 {
        if let Some(hint) = claiming_in_unit(grid, &col_cells(c)) {
            return Some(hint);
        }
    }
    None
}

fn claiming_in_unit(grid: &CandidateGrid, unit: &[usize; 9]) -> Option<TechniqueHint> {
    for digit in 1..=9u8 {
        let occurrences: Vec<usize> = unit
            .iter()
            .copied()
            .filter(|&idx| grid.value_at(idx) == 0 && grid.has_candidate(idx, digit))
            .collect();
        let Some(&first) = occurrences.first() else {
            continue;
        };
        let target_box = box_of(first);
        if !occurrences.iter().all(|&idx| box_of(idx) == target_box) {
            continue;
        }

        let removals: Vec<(usize, u8)> = box_cells(target_box)
            .iter()
            .copied()
            .filter(|idx| {
                !unit.contains(idx) && grid.value_at(*idx) == 0 && grid.has_candidate(*idx, digit)
            })
            .map(|idx| (idx, digit))
            .collect();
        if !removals.is_empty() {
            return Some(TechniqueHint {
                technique_id: ID,
                effect: TechniqueEffect::Eliminate(removals),
            });
        }
    }
    None
}

pub struct LockedCandidates;

impl Technique for LockedCandidates {
    fn id(&self) -> &'static str {
        ID
    }

    fn find_hint(&self, grid: &CandidateGrid) -> Option<TechniqueHint> {
        pointing_hint(grid).or_else(|| claiming_hint(grid))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sudoku_core::board::Board;

    #[test]
    fn empty_board_has_no_locked_candidates() {
        let grid = CandidateGrid::from_board(&Board::empty()).unwrap();
        assert!(LockedCandidates.find_hint(&grid).is_none());
    }

    #[test]
    fn pointing_eliminates_from_rest_of_row() {
        let mut grid = CandidateGrid::from_board(&Board::empty()).unwrap();
        // Confine digit 4 within box 0 to row 0 only.
        for &idx in &box_cells(0) {
            if row_of(idx) != 0 {
                grid.remove_candidate(idx, 4);
            }
        }
        let hint = LockedCandidates.find_hint(&grid).unwrap();
        assert_eq!(hint.technique_id, "pointing_claiming");
        match hint.effect {
            TechniqueEffect::Eliminate(removals) => {
                assert!(!removals.is_empty());
                for (idx, digit) in &removals {
                    assert_eq!(*digit, 4);
                    assert_eq!(row_of(*idx), 0);
                    assert_ne!(box_of(*idx), 0);
                }
            }
            _ => panic!("expected eliminations"),
        }
    }

    #[test]
    fn claiming_eliminates_from_rest_of_box() {
        let mut grid = CandidateGrid::from_board(&Board::empty()).unwrap();
        // Confine digit 6 within row 0 to box 0 only.
        for &idx in &row_cells(0) {
            if box_of(idx) != 0 {
                grid.remove_candidate(idx, 6);
            }
        }
        let hint = LockedCandidates.find_hint(&grid).unwrap();
        match hint.effect {
            TechniqueEffect::Eliminate(removals) => {
                assert!(!removals.is_empty());
                for (idx, digit) in &removals {
                    assert_eq!(*digit, 6);
                    assert_eq!(box_of(*idx), 0);
                    assert_ne!(row_of(*idx), 0);
                }
            }
            _ => panic!("expected eliminations"),
        }
    }
}
