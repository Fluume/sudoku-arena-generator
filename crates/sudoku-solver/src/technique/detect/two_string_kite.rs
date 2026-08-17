//! 2-String Kite: a digit confined to exactly 2 cells in some row, and
//! also confined to exactly 2 cells in some column, where one cell from
//! the row and one cell from the column share a box (the "knot"). The
//! remaining row cell and remaining column cell are the kite's two free
//! ends — eliminate the digit from any cell that sees both.
//!
//! The Java reference this is ported from loops candidate values `0..8`
//! instead of `1..=9`, which never checks digit 9 — fixed here.
//!
//! Also fixed here: the pattern only holds ("free end 1 has the digit, or
//! free end 2 does — never neither") when the row's 2 candidate cells, the
//! column's 2 candidate cells, and the connecting box are all *genuinely*
//! distinct positions. If the row's chosen "knot" cell happens to coincide
//! with the column's, or with either free end (a real, reachable geometric
//! coincidence — e.g. when the row and column happen to intersect at one
//! of the pattern's own candidate cells), the alternating strong-link chain
//! this technique relies on breaks down and the derived elimination is no
//! longer valid — it can eliminate a digit that's actually forced. A
//! reference implementation (SukakuExplainer's generalized Turbot Fish,
//! which Skyscraper/2-String Kite are special cases of) guards against
//! exactly this by rejecting any match whose connecting region coincides
//! with the base or cover region; the equivalent guard here is requiring
//! all 4 pattern cells to be pairwise distinct.

use sudoku_core::board::{box_of, col_of, index, row_of, CELLS};

use crate::technique::grid::{col_cells, row_cells, sees, CandidateGrid};
use crate::technique::solve::{Technique, TechniqueEffect, TechniqueHint};

pub struct TwoStringKite;

impl Technique for TwoStringKite {
    fn id(&self) -> &'static str {
        "two_string_kite"
    }

    fn find_hint(&self, grid: &CandidateGrid) -> Option<TechniqueHint> {
        for digit in 1..=9u8 {
            let valid_rows = valid_lines(grid, digit, true);
            let valid_cols = valid_lines(grid, digit, false);

            for &(row, row_cols) in &valid_rows {
                for &(col, col_rows) in &valid_cols {
                    for rc in 0..2 {
                        for cr in 0..2 {
                            let knot_a = index(row, row_cols[rc]);
                            let knot_b = index(col_rows[cr], col);
                            let end1 = index(row, row_cols[1 - rc]);
                            let end2 = index(col_rows[1 - cr], col);

                            // All 4 pattern cells must be genuinely distinct
                            // — see the module doc comment for why a
                            // coincidental overlap invalidates the pattern.
                            let cells = [knot_a, end1, knot_b, end2];
                            let all_distinct = (0..cells.len())
                                .all(|i| (i + 1..cells.len()).all(|j| cells[i] != cells[j]));
                            if !all_distinct {
                                continue;
                            }

                            if box_of(knot_a) != box_of(knot_b) {
                                continue;
                            }

                            let removals: Vec<(usize, u8)> = (0..CELLS)
                                .filter(|&idx| {
                                    idx != end1
                                        && idx != end2
                                        && grid.value_at(idx) == 0
                                        && grid.has_candidate(idx, digit)
                                        && sees(idx, end1)
                                        && sees(idx, end2)
                                })
                                .map(|idx| (idx, digit))
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
            }
        }
        None
    }
}

/// Rows (or, if `!by_rows`, columns) where `digit` has exactly 2 remaining
/// candidate cells, with the cross-axis positions of those 2 cells.
fn valid_lines(grid: &CandidateGrid, digit: u8, by_rows: bool) -> Vec<(usize, [usize; 2])> {
    let mut lines = Vec::new();
    for line in 0..9 {
        let cells = if by_rows {
            row_cells(line)
        } else {
            col_cells(line)
        };
        let crosses: Vec<usize> = cells
            .iter()
            .copied()
            .filter(|&idx| grid.value_at(idx) == 0 && grid.has_candidate(idx, digit))
            .map(|idx| if by_rows { col_of(idx) } else { row_of(idx) })
            .collect();
        if crosses.len() == 2 {
            lines.push((line, [crosses[0], crosses[1]]));
        }
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use sudoku_core::board::Board;

    #[test]
    fn empty_board_has_no_two_string_kite() {
        let grid = CandidateGrid::from_board(&Board::empty()).unwrap();
        assert!(TwoStringKite.find_hint(&grid).is_none());
    }

    #[test]
    fn does_not_eliminate_when_the_connecting_cells_coincide_with_a_free_end() {
        let mut grid = CandidateGrid::from_board(&Board::empty()).unwrap();
        // Row 0 confined to cols {3, 6}; column 3 confined to rows {0, 5}.
        // The row's and column's candidate at (0,3) is the SAME cell for
        // both patterns — a degenerate case where the kite's usual "at
        // least one free end has the digit" reasoning breaks down (see the
        // module doc comment: here it's actually "both free ends have the
        // digit, or neither does"). Must not fire an elimination.
        for &idx in &row_cells(0) {
            if col_of(idx) != 3 && col_of(idx) != 6 {
                grid.remove_candidate(idx, 5);
            }
        }
        for &idx in &col_cells(3) {
            if row_of(idx) != 0 && row_of(idx) != 5 {
                grid.remove_candidate(idx, 5);
            }
        }

        assert!(
            TwoStringKite.find_hint(&grid).is_none(),
            "the degenerate connecting-cell coincidence must not produce a (wrong) elimination"
        );
    }

    #[test]
    fn finds_kite_and_eliminates_a_cell_seeing_both_ends() {
        let mut grid = CandidateGrid::from_board(&Board::empty()).unwrap();
        // Row 2 confined to cols {1, 7}; column 8 confined to rows {0, 6}.
        // (row2, col7) and (row0, col8) share box 2 — that's the knot.
        // Free ends: (row2, col1) and (row6, col8).
        for &idx in &row_cells(2) {
            if col_of(idx) != 1 && col_of(idx) != 7 {
                grid.remove_candidate(idx, 6);
            }
        }
        for &idx in &col_cells(8) {
            if row_of(idx) != 0 && row_of(idx) != 6 {
                grid.remove_candidate(idx, 6);
            }
        }

        let hint = TwoStringKite.find_hint(&grid).unwrap();
        match hint.effect {
            TechniqueEffect::Eliminate(removals) => {
                assert!(!removals.is_empty());
                // (row 6, col 1) sees the (row2,col1) end via its column
                // and the (row6,col8) end via its row.
                assert!(removals.contains(&(index(6, 1), 6)));
                for (idx, digit) in &removals {
                    assert_eq!(*digit, 6);
                    assert_ne!(*idx, index(2, 1));
                    assert_ne!(*idx, index(6, 8));
                }
            }
            _ => panic!("expected eliminations"),
        }
    }
}
