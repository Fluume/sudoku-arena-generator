//! Naked Triple: three cells in a unit whose candidates, unioned together,
//! are exactly 3 digits (each cell individually may have only 2 of the 3)
//! → eliminate those 3 digits from every other cell in the unit.
//!
//! Like Naked Pair, the Java reference's `apply()` for this technique is a
//! no-op — this is a genuine reimplementation.

use super::{combinations, digits_of};
use crate::technique::grid::{all_units, CandidateGrid};
use crate::technique::solve::{Technique, TechniqueEffect, TechniqueHint};

pub struct NakedTriple;

impl Technique for NakedTriple {
    fn id(&self) -> &'static str {
        "naked_triple"
    }

    fn find_hint(&self, grid: &CandidateGrid) -> Option<TechniqueHint> {
        for unit in all_units() {
            let cells: Vec<usize> = unit
                .iter()
                .copied()
                .filter(|&idx| {
                    grid.value_at(idx) == 0 && (2..=3).contains(&grid.candidate_count(idx))
                })
                .collect();
            if cells.len() < 3 {
                continue;
            }

            for combo in combinations(cells.len(), 3) {
                let chosen: Vec<usize> = combo.iter().map(|&i| cells[i]).collect();
                let mask = chosen
                    .iter()
                    .fold(0u16, |acc, &idx| acc | grid.candidates_at(idx));
                if mask.count_ones() != 3 {
                    continue;
                }

                let digits = digits_of(mask);
                let removals: Vec<(usize, u8)> = unit
                    .iter()
                    .copied()
                    .filter(|idx| !chosen.contains(idx) && grid.value_at(*idx) == 0)
                    .flat_map(|idx| {
                        digits
                            .iter()
                            .copied()
                            .filter(move |&d| grid.has_candidate(idx, d))
                            .map(move |d| (idx, d))
                    })
                    .collect();

                if !removals.is_empty() {
                    return Some(TechniqueHint {
                        technique_id: self.id(),
                        effect: TechniqueEffect::Eliminate(removals),
                    });
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::technique::grid::row_cells;
    use sudoku_core::board::Board;

    #[test]
    fn empty_board_has_no_naked_triple() {
        let grid = CandidateGrid::from_board(&Board::empty()).unwrap();
        assert!(NakedTriple.find_hint(&grid).is_none());
    }

    #[test]
    fn eliminates_triple_digits_from_rest_of_unit() {
        let mut grid = CandidateGrid::from_board(&Board::empty()).unwrap();
        let unit = row_cells(0);
        let sets: [&[u8]; 3] = [&[1, 2], &[2, 3], &[1, 3]];
        for (i, set) in sets.iter().enumerate() {
            let idx = unit[i];
            for d in 1..=9u8 {
                if !set.contains(&d) {
                    grid.remove_candidate(idx, d);
                }
            }
        }

        let hint = NakedTriple.find_hint(&grid).unwrap();
        match hint.effect {
            TechniqueEffect::Eliminate(removals) => {
                assert!(!removals.is_empty());
                for (idx, digit) in &removals {
                    assert!([1u8, 2, 3].contains(digit));
                    assert!(!unit[0..3].contains(idx));
                }
            }
            _ => panic!("expected eliminations"),
        }
    }
}
