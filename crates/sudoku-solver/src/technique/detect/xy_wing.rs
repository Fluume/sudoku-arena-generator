//! XY-Wing: a pivot cell with 2 candidates {X,Y}, and two "wing" cells that
//! each see the pivot: one with candidates {X,Z}, the other {Y,Z} (some
//! shared third digit Z). Any cell that sees both wings can't be Z, since
//! whichever of X/Y the pivot turns out to be, one of the wings forces Z
//! into the cell that sees it.
//!
//! Detected generically: among the pivot's and both wings' 3 candidate
//! sets, the union must be exactly {X,Y,Z} (3 digits) and no digit is
//! common to all three — the one digit shared by just the two wings is Z.

use sudoku_core::bitmask::bit_to_digit;
use sudoku_core::board::CELLS;

use crate::technique::grid::{sees, CandidateGrid};
use crate::technique::solve::{Technique, TechniqueEffect, TechniqueHint};

pub struct XyWing;

impl Technique for XyWing {
    fn id(&self) -> &'static str {
        "xy_wing"
    }

    fn find_hint(&self, grid: &CandidateGrid) -> Option<TechniqueHint> {
        let bivalue_cells: Vec<usize> = (0..CELLS)
            .filter(|&idx| grid.value_at(idx) == 0 && grid.candidate_count(idx) == 2)
            .collect();

        for &pivot in &bivalue_cells {
            let pivot_mask = grid.candidates_at(pivot);
            let wings: Vec<usize> = bivalue_cells
                .iter()
                .copied()
                .filter(|&w| sees(w, pivot))
                .collect();

            for i in 0..wings.len() {
                for j in (i + 1)..wings.len() {
                    let (w1, w2) = (wings[i], wings[j]);
                    let m1 = grid.candidates_at(w1);
                    let m2 = grid.candidates_at(w2);

                    let union = pivot_mask | m1 | m2;
                    let triple = pivot_mask & m1 & m2;
                    if union.count_ones() != 3 || triple != 0 {
                        continue;
                    }
                    let z_mask = m1 & m2;
                    if z_mask.count_ones() != 1 {
                        continue;
                    }
                    let z = bit_to_digit(z_mask);

                    let removals: Vec<(usize, u8)> = (0..CELLS)
                        .filter(|&idx| {
                            idx != pivot
                                && idx != w1
                                && idx != w2
                                && grid.value_at(idx) == 0
                                && grid.has_candidate(idx, z)
                                && sees(idx, w1)
                                && sees(idx, w2)
                        })
                        .map(|idx| (idx, z))
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
    use sudoku_core::board::{index, Board};

    #[test]
    fn empty_board_has_no_xy_wing() {
        let grid = CandidateGrid::from_board(&Board::empty()).unwrap();
        assert!(XyWing.find_hint(&grid).is_none());
    }

    #[test]
    fn finds_xy_wing_and_eliminates_z_from_a_cell_seeing_both_wings() {
        let mut grid = CandidateGrid::from_board(&Board::empty()).unwrap();
        let pivot = index(0, 0);
        let wing1 = index(0, 4); // sees pivot via row 0, candidates {1,3}
        let wing2 = index(4, 0); // sees pivot via col 0, candidates {2,3}

        for d in 1..=9u8 {
            if d != 1 && d != 2 {
                grid.remove_candidate(pivot, d);
            }
            if d != 1 && d != 3 {
                grid.remove_candidate(wing1, d);
            }
            if d != 2 && d != 3 {
                grid.remove_candidate(wing2, d);
            }
        }

        let hint = XyWing.find_hint(&grid).unwrap();
        match hint.effect {
            TechniqueEffect::Eliminate(removals) => {
                assert!(!removals.is_empty());
                // (4,4) sees wing1 via column 4 and wing2 via row 4.
                assert!(removals.contains(&(index(4, 4), 3)));
                for (idx, digit) in &removals {
                    assert_eq!(*digit, 3);
                    assert!(![pivot, wing1, wing2].contains(idx));
                }
            }
            _ => panic!("expected eliminations"),
        }
    }
}
