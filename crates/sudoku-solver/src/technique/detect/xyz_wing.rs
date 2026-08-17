//! XYZ-Wing: like XY-Wing, but the pivot has 3 candidates {X,Y,Z} and each
//! wing is a 2-candidate *subset* of the pivot's candidates (one {X,Z}, the
//! other {Y,Z}). Because Z is also a pivot candidate, a cell must see the
//! pivot *and* both wings to be provably not-Z.

use sudoku_core::bitmask::bit_to_digit;
use sudoku_core::board::CELLS;

use crate::technique::grid::{sees, CandidateGrid};
use crate::technique::solve::{Technique, TechniqueEffect, TechniqueHint};

pub struct XyzWing;

impl Technique for XyzWing {
    fn id(&self) -> &'static str {
        "xyz_wing"
    }

    fn find_hint(&self, grid: &CandidateGrid) -> Option<TechniqueHint> {
        let bivalue_cells: Vec<usize> = (0..CELLS)
            .filter(|&idx| grid.value_at(idx) == 0 && grid.candidate_count(idx) == 2)
            .collect();
        let trivalue_cells: Vec<usize> = (0..CELLS)
            .filter(|&idx| grid.value_at(idx) == 0 && grid.candidate_count(idx) == 3)
            .collect();

        for &pivot in &trivalue_cells {
            let pivot_mask = grid.candidates_at(pivot);
            // A wing must be a candidate-subset of the pivot (2 of its 3
            // digits) and see it.
            let wings: Vec<usize> = bivalue_cells
                .iter()
                .copied()
                .filter(|&w| sees(w, pivot) && (grid.candidates_at(w) | pivot_mask) == pivot_mask)
                .collect();

            for i in 0..wings.len() {
                for j in (i + 1)..wings.len() {
                    let (w1, w2) = (wings[i], wings[j]);
                    let m1 = grid.candidates_at(w1);
                    let m2 = grid.candidates_at(w2);

                    let union = pivot_mask | m1 | m2;
                    let triple = pivot_mask & m1 & m2;
                    if union.count_ones() != 3 || triple.count_ones() != 1 {
                        continue;
                    }
                    let z = bit_to_digit(triple);

                    let removals: Vec<(usize, u8)> = (0..CELLS)
                        .filter(|&idx| {
                            idx != pivot
                                && idx != w1
                                && idx != w2
                                && grid.value_at(idx) == 0
                                && grid.has_candidate(idx, z)
                                && sees(idx, pivot)
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
    fn empty_board_has_no_xyz_wing() {
        let grid = CandidateGrid::from_board(&Board::empty()).unwrap();
        assert!(XyzWing.find_hint(&grid).is_none());
    }

    #[test]
    fn finds_xyz_wing_and_eliminates_z_from_a_cell_seeing_all_three() {
        let mut grid = CandidateGrid::from_board(&Board::empty()).unwrap();
        // All three pattern cells share box 4, so any other box-4 cell
        // sees all three at once.
        let pivot = index(4, 4);
        let wing1 = index(3, 3); // candidates {1,3}
        let wing2 = index(5, 5); // candidates {2,3}

        for d in 1..=9u8 {
            if d != 1 && d != 2 && d != 3 {
                grid.remove_candidate(pivot, d);
            }
            if d != 1 && d != 3 {
                grid.remove_candidate(wing1, d);
            }
            if d != 2 && d != 3 {
                grid.remove_candidate(wing2, d);
            }
        }

        let hint = XyzWing.find_hint(&grid).unwrap();
        match hint.effect {
            TechniqueEffect::Eliminate(removals) => {
                assert!(!removals.is_empty());
                assert!(removals.contains(&(index(3, 5), 3)));
                for (idx, digit) in &removals {
                    assert_eq!(*digit, 3);
                    assert!(![pivot, wing1, wing2].contains(idx));
                }
            }
            _ => panic!("expected eliminations"),
        }
    }
}
