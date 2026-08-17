//! Jellyfish: a digit confined, across 4 rows, to 4 columns overall (each
//! row contributing 2 to 4 of them) — eliminate it from those columns in
//! every other row. See [`super::find_fish`] for the shared search
//! (degree 4).

use super::find_fish;
use crate::technique::grid::CandidateGrid;
use crate::technique::solve::{Technique, TechniqueHint};

pub struct Jellyfish;

impl Technique for Jellyfish {
    fn id(&self) -> &'static str {
        "jellyfish"
    }

    fn find_hint(&self, grid: &CandidateGrid) -> Option<TechniqueHint> {
        find_fish(grid, 4, self.id())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::technique::grid::row_cells;
    use crate::technique::solve::TechniqueEffect;
    use sudoku_core::board::{col_of, row_of, Board};

    #[test]
    fn empty_board_has_no_jellyfish() {
        let grid = CandidateGrid::from_board(&Board::empty()).unwrap();
        assert!(Jellyfish.find_hint(&grid).is_none());
    }

    #[test]
    fn finds_jellyfish_and_eliminates_from_other_rows() {
        let mut grid = CandidateGrid::from_board(&Board::empty()).unwrap();
        let pattern: [(usize, &[usize]); 4] =
            [(0, &[0, 1]), (1, &[1, 2]), (2, &[2, 3]), (3, &[3, 0])];
        for (row, cols) in pattern {
            for idx in row_cells(row) {
                if !cols.contains(&col_of(idx)) {
                    grid.remove_candidate(idx, 7);
                }
            }
        }

        let hint = Jellyfish.find_hint(&grid).unwrap();
        match hint.effect {
            TechniqueEffect::Eliminate(removals) => {
                assert!(!removals.is_empty());
                for (idx, digit) in &removals {
                    assert_eq!(*digit, 7);
                    assert!([0usize, 1, 2, 3].contains(&col_of(*idx)));
                    assert!(![0usize, 1, 2, 3].contains(&row_of(*idx)));
                }
            }
            _ => panic!("expected eliminations"),
        }
    }
}
