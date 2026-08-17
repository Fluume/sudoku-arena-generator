//! Hidden Quad: four digits confined, within a unit, to the same 4 cells —
//! eliminate every other candidate from those 4 cells. See
//! [`super::find_hidden_n`] for the shared search.

use super::find_hidden_n;
use crate::technique::grid::CandidateGrid;
use crate::technique::solve::{Technique, TechniqueHint};

pub struct HiddenQuad;

impl Technique for HiddenQuad {
    fn id(&self) -> &'static str {
        "hidden_quad"
    }

    fn find_hint(&self, grid: &CandidateGrid) -> Option<TechniqueHint> {
        find_hidden_n(grid, 4, self.id())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::technique::grid::row_cells;
    use crate::technique::solve::TechniqueEffect;
    use sudoku_core::board::Board;

    #[test]
    fn empty_board_has_no_hidden_quad() {
        let grid = CandidateGrid::from_board(&Board::empty()).unwrap();
        assert!(HiddenQuad.find_hint(&grid).is_none());
    }

    #[test]
    fn finds_hidden_quad_and_strips_other_candidates() {
        let mut grid = CandidateGrid::from_board(&Board::empty()).unwrap();
        let unit = row_cells(0);
        let confined = [unit[0], unit[2], unit[4], unit[6]];
        for &idx in &unit {
            if !confined.contains(&idx) {
                grid.remove_candidate(idx, 1);
                grid.remove_candidate(idx, 2);
                grid.remove_candidate(idx, 3);
                grid.remove_candidate(idx, 4);
            }
        }

        let hint = HiddenQuad.find_hint(&grid).unwrap();
        match hint.effect {
            TechniqueEffect::Eliminate(removals) => {
                assert!(!removals.is_empty());
                for (idx, digit) in &removals {
                    assert!(confined.contains(idx));
                    assert!(![1u8, 2, 3, 4].contains(digit));
                }
            }
            _ => panic!("expected eliminations"),
        }
    }
}
