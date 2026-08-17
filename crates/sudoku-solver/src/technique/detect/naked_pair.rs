//! Naked Pair: two cells in a unit whose candidates are both exactly the
//! same 2-digit set → those two digits can't appear anywhere else in the
//! unit, so eliminate them from every other cell.
//!
//! The Java reference's `apply()` for this technique is a no-op (it only
//! ever records which region/values matched, never which cells, and never
//! performs the elimination) — this is a genuine reimplementation, not a
//! port, using the corrected/completed algorithm.

use super::digits_of;
use crate::technique::grid::{all_units, CandidateGrid};
use crate::technique::solve::{Technique, TechniqueEffect, TechniqueHint};

pub struct NakedPair;

impl Technique for NakedPair {
    fn id(&self) -> &'static str {
        "naked_pair"
    }

    fn find_hint(&self, grid: &CandidateGrid) -> Option<TechniqueHint> {
        for unit in all_units() {
            let cells: Vec<usize> = unit
                .iter()
                .copied()
                .filter(|&idx| grid.value_at(idx) == 0 && grid.candidate_count(idx) == 2)
                .collect();

            for i in 0..cells.len() {
                for j in (i + 1)..cells.len() {
                    let (a, b) = (cells[i], cells[j]);
                    let mask = grid.candidates_at(a);
                    if mask != grid.candidates_at(b) {
                        continue;
                    }

                    let digits = digits_of(mask);
                    let removals: Vec<(usize, u8)> = unit
                        .iter()
                        .copied()
                        .filter(|&idx| idx != a && idx != b && grid.value_at(idx) == 0)
                        .flat_map(|idx| {
                            digits
                                .iter()
                                .copied()
                                .filter(move |&d| grid.has_candidate(idx, d))
                                .map(move |d| (idx, d))
                        })
                        .collect();

                    if !removals.is_empty() {
                        return Some(TechniqueHint {
                            technique_id: self.id(),
                            effect: TechniqueEffect::Eliminate(removals),
                        });
                    }
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::technique::grid::row_cells;
    use sudoku_core::board::Board;

    #[test]
    fn empty_board_has_no_naked_pair() {
        let grid = CandidateGrid::from_board(&Board::empty()).unwrap();
        assert!(NakedPair.find_hint(&grid).is_none());
    }

    #[test]
    fn eliminates_pair_digits_from_rest_of_unit() {
        let mut grid = CandidateGrid::from_board(&Board::empty()).unwrap();
        let unit = row_cells(0);
        for &idx in &[unit[0], unit[1]] {
            for d in 1..=9u8 {
                if d != 2 && d != 7 {
                    grid.remove_candidate(idx, d);
                }
            }
        }

        let hint = NakedPair.find_hint(&grid).unwrap();
        match hint.effect {
            TechniqueEffect::Eliminate(removals) => {
                assert!(!removals.is_empty());
                for (idx, digit) in &removals {
                    assert!(*digit == 2 || *digit == 7);
                    assert!(*idx != unit[0] && *idx != unit[1]);
                }
            }
            _ => panic!("expected eliminations"),
        }
    }
}
