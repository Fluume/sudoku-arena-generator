//! Swordfish: a digit confined, across 3 rows, to 3 columns overall (each
//! row contributing 2 or 3 of them) — eliminate it from those columns in
//! every other row. See [`super::find_fish`] for the shared search
//! (degree 3).

use super::find_fish;
use crate::technique::grid::CandidateGrid;
use crate::technique::solve::{Technique, TechniqueHint};

pub struct Swordfish;

impl Technique for Swordfish {
    fn id(&self) -> &'static str {
        "swordfish"
    }

    fn find_hint(&self, grid: &CandidateGrid) -> Option<TechniqueHint> {
        find_fish(grid, 3, self.id())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::technique::grid::row_cells;
    use crate::technique::solve::TechniqueEffect;
    use sudoku_core::board::{col_of, row_of, Board};

    #[test]
    fn empty_board_has_no_swordfish() {
        let grid = CandidateGrid::from_board(&Board::empty()).unwrap();
        assert!(Swordfish.find_hint(&grid).is_none());
    }

    #[test]
    fn finds_swordfish_and_eliminates_from_other_rows() {
        let mut grid = CandidateGrid::from_board(&Board::empty()).unwrap();
        let pattern: [(usize, &[usize]); 3] = [(0, &[1, 4]), (3, &[4, 7]), (6, &[1, 7])];
        for (row, cols) in pattern {
            for idx in row_cells(row) {
                if !cols.contains(&col_of(idx)) {
                    grid.remove_candidate(idx, 6);
                }
            }
        }

        let hint = Swordfish.find_hint(&grid).unwrap();
        match hint.effect {
            TechniqueEffect::Eliminate(removals) => {
                assert!(!removals.is_empty());
                for (idx, digit) in &removals {
                    assert_eq!(*digit, 6);
                    assert!([1usize, 4, 7].contains(&col_of(*idx)));
                    assert!(![0usize, 3, 6].contains(&row_of(*idx)));
                }
            }
            _ => panic!("expected eliminations"),
        }
    }
}
