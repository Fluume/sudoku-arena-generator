//! Hidden Triple: three digits confined, within a unit, to the same 3
//! cells — eliminate every other candidate from those 3 cells. See
//! [`super::find_hidden_n`] for the shared search.

use super::find_hidden_n;
use crate::technique::grid::CandidateGrid;
use crate::technique::solve::{Technique, TechniqueHint};

pub struct HiddenTriple;

impl Technique for HiddenTriple {
    fn id(&self) -> &'static str {
        "hidden_triple"
    }

    fn find_hint(&self, grid: &CandidateGrid) -> Option<TechniqueHint> {
        find_hidden_n(grid, 3, self.id())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::technique::grid::row_cells;
    use crate::technique::solve::TechniqueEffect;
    use sudoku_core::board::Board;

    #[test]
    fn empty_board_has_no_hidden_triple() {
        let grid = CandidateGrid::from_board(&Board::empty()).unwrap();
        assert!(HiddenTriple.find_hint(&grid).is_none());
    }

    #[test]
    fn finds_hidden_triple_and_strips_other_candidates() {
        let mut grid = CandidateGrid::from_board(&Board::empty()).unwrap();
        let unit = row_cells(0);
        let confined = [unit[1], unit[4], unit[7]];
        for &idx in &unit {
            if !confined.contains(&idx) {
                grid.remove_candidate(idx, 3);
                grid.remove_candidate(idx, 5);
                grid.remove_candidate(idx, 9);
            }
        }

        let hint = HiddenTriple.find_hint(&grid).unwrap();
        match hint.effect {
            TechniqueEffect::Eliminate(removals) => {
                assert!(!removals.is_empty());
                for (idx, digit) in &removals {
                    assert!(confined.contains(idx));
                    assert!(![3u8, 5, 9].contains(digit));
                }
            }
            _ => panic!("expected eliminations"),
        }
    }
}
