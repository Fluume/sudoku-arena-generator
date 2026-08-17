//! Hidden Pair: two digits confined, within a unit, to the same 2 cells —
//! eliminate every other candidate from those 2 cells. See
//! [`super::find_hidden_n`] for the shared search.

use super::find_hidden_n;
use crate::technique::grid::CandidateGrid;
use crate::technique::solve::{Technique, TechniqueHint};

pub struct HiddenPair;

impl Technique for HiddenPair {
    fn id(&self) -> &'static str {
        "hidden_pair"
    }

    fn find_hint(&self, grid: &CandidateGrid) -> Option<TechniqueHint> {
        find_hidden_n(grid, 2, self.id())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::technique::grid::row_cells;
    use crate::technique::solve::TechniqueEffect;
    use sudoku_core::board::Board;

    #[test]
    fn empty_board_has_no_hidden_pair() {
        let grid = CandidateGrid::from_board(&Board::empty()).unwrap();
        assert!(HiddenPair.find_hint(&grid).is_none());
    }

    #[test]
    fn finds_hidden_pair_and_strips_other_candidates() {
        let mut grid = CandidateGrid::from_board(&Board::empty()).unwrap();
        let unit = row_cells(0);
        // Confine digits 4 and 8 to exactly cells unit[2] and unit[5],
        // which also keep other (unrelated) candidates.
        for &idx in &unit {
            if idx != unit[2] && idx != unit[5] {
                grid.remove_candidate(idx, 4);
                grid.remove_candidate(idx, 8);
            }
        }

        let hint = HiddenPair.find_hint(&grid).unwrap();
        match hint.effect {
            TechniqueEffect::Eliminate(removals) => {
                assert!(!removals.is_empty());
                for (idx, digit) in &removals {
                    assert!(*idx == unit[2] || *idx == unit[5]);
                    assert!(*digit != 4 && *digit != 8);
                }
            }
            _ => panic!("expected eliminations"),
        }
    }

    #[test]
    fn does_not_pair_a_confined_digit_with_an_already_placed_one() {
        let mut grid = CandidateGrid::from_board(&Board::empty()).unwrap();
        let unit = row_cells(0);
        // Digit 4 is placed, so it has zero remaining candidates in this
        // row. Digit 6 is confined to exactly two cells that still carry
        // other real candidates too.
        grid.place(unit[0], 4);
        for &idx in &unit[1..] {
            if idx != unit[3] && idx != unit[4] {
                grid.remove_candidate(idx, 6);
            }
        }
        // Without the "each value must actually occur" guard, {4,6} would
        // look like a hidden pair confined to unit[3]/unit[4] and wrongly
        // strip their other real candidates down to just {4,6}.
        assert!(HiddenPair.find_hint(&grid).is_none());
    }
}
