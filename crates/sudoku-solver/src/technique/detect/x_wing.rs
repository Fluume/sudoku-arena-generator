//! X-Wing: a digit confined, in each of 2 rows, to the same 2 columns (or
//! the column-based mirror) — eliminate it from those columns in every
//! other row. See [`super::find_fish`] for the shared search (degree 2).

use super::find_fish;
use crate::technique::grid::CandidateGrid;
use crate::technique::solve::{Technique, TechniqueHint};

pub struct XWing;

impl Technique for XWing {
    fn id(&self) -> &'static str {
        "x_wing"
    }

    fn find_hint(&self, grid: &CandidateGrid) -> Option<TechniqueHint> {
        find_fish(grid, 2, self.id())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::technique::grid::row_cells;
    use crate::technique::solve::TechniqueEffect;
    use sudoku_core::board::{col_of, row_of, Board};

    #[test]
    fn empty_board_has_no_x_wing() {
        let grid = CandidateGrid::from_board(&Board::empty()).unwrap();
        assert!(XWing.find_hint(&grid).is_none());
    }

    #[test]
    fn finds_x_wing_and_eliminates_from_other_rows() {
        let mut grid = CandidateGrid::from_board(&Board::empty()).unwrap();
        for &row in &[0usize, 3] {
            for idx in row_cells(row) {
                if col_of(idx) != 2 && col_of(idx) != 6 {
                    grid.remove_candidate(idx, 5);
                }
            }
        }

        let hint = XWing.find_hint(&grid).unwrap();
        match hint.effect {
            TechniqueEffect::Eliminate(removals) => {
                assert!(!removals.is_empty());
                for (idx, digit) in &removals {
                    assert_eq!(*digit, 5);
                    assert!(col_of(*idx) == 2 || col_of(*idx) == 6);
                    assert!(row_of(*idx) != 0 && row_of(*idx) != 3);
                }
            }
            _ => panic!("expected eliminations"),
        }
    }
}
