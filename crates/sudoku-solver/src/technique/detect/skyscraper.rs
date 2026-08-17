//! Skyscraper: a digit confined to exactly 2 cells in each of 2 rows (or
//! the column-based mirror), where the two rows share exactly one of those
//! columns (the "base"). The other two cells are the "roof" tips —
//! eliminate the digit from any cell that sees both roof tips.
//!
//! The Java reference this is ported from loops candidate values `0..8`
//! instead of `1..=9`, which never checks digit 9 — fixed here.

use sudoku_core::board::{col_of, index, row_of, CELLS};

use crate::technique::grid::{col_cells, row_cells, sees, CandidateGrid};
use crate::technique::solve::{Technique, TechniqueEffect, TechniqueHint};

pub struct Skyscraper;

impl Technique for Skyscraper {
    fn id(&self) -> &'static str {
        "skyscraper"
    }

    fn find_hint(&self, grid: &CandidateGrid) -> Option<TechniqueHint> {
        find_direction(grid, true).or_else(|| find_direction(grid, false))
    }
}

fn find_direction(grid: &CandidateGrid, by_rows: bool) -> Option<TechniqueHint> {
    for digit in 1..=9u8 {
        let mut valid_lines: Vec<(usize, [usize; 2])> = Vec::new();
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
                valid_lines.push((line, [crosses[0], crosses[1]]));
            }
        }

        for i in 0..valid_lines.len() {
            for j in (i + 1)..valid_lines.len() {
                let (line1, c1) = valid_lines[i];
                let (line2, c2) = valid_lines[j];
                let Some((roof1, roof2)) = shared_and_roofs(c1, c2) else {
                    continue;
                };

                let end1 = if by_rows {
                    index(line1, roof1)
                } else {
                    index(roof1, line1)
                };
                let end2 = if by_rows {
                    index(line2, roof2)
                } else {
                    index(roof2, line2)
                };

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
                        technique_id: "skyscraper",
                        effect: TechniqueEffect::Eliminate(removals),
                    });
                }
            }
        }
    }
    None
}

/// If `c1` and `c2` share exactly one coordinate, returns the two "roof"
/// (non-shared) coordinates, `(from c1, from c2)`.
fn shared_and_roofs(c1: [usize; 2], c2: [usize; 2]) -> Option<(usize, usize)> {
    if c1[0] == c2[0] && c1[1] != c2[1] {
        Some((c1[1], c2[1]))
    } else if c1[1] == c2[1] && c1[0] != c2[0] {
        Some((c1[0], c2[0]))
    } else if c1[0] == c2[1] && c1[1] != c2[0] {
        Some((c1[1], c2[0]))
    } else if c1[1] == c2[0] && c1[0] != c2[1] {
        Some((c1[0], c2[1]))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sudoku_core::board::Board;

    #[test]
    fn empty_board_has_no_skyscraper() {
        let grid = CandidateGrid::from_board(&Board::empty()).unwrap();
        assert!(Skyscraper.find_hint(&grid).is_none());
    }

    #[test]
    fn finds_skyscraper_and_eliminates_a_cell_seeing_both_roofs() {
        let mut grid = CandidateGrid::from_board(&Board::empty()).unwrap();
        // Row 0 confined to cols {1, 8}; row 5 confined to cols {2, 8}.
        // Shared base column 8, roofs at col 1 (row 0) and col 2 (row 5).
        for &idx in &row_cells(0) {
            if col_of(idx) != 1 && col_of(idx) != 8 {
                grid.remove_candidate(idx, 4);
            }
        }
        for &idx in &row_cells(5) {
            if col_of(idx) != 2 && col_of(idx) != 8 {
                grid.remove_candidate(idx, 4);
            }
        }

        let hint = Skyscraper.find_hint(&grid).unwrap();
        match hint.effect {
            TechniqueEffect::Eliminate(removals) => {
                assert!(!removals.is_empty());
                // (row 3, col 1) sees the col-1 roof via its column and the
                // col-2 roof via sharing its box.
                assert!(removals.contains(&(index(3, 1), 4)));
                for (idx, digit) in &removals {
                    assert_eq!(*digit, 4);
                    assert_ne!(*idx, index(0, 1));
                    assert_ne!(*idx, index(5, 2));
                }
            }
            _ => panic!("expected eliminations"),
        }
    }
}
