//! Naked Quad: four cells in a unit whose candidates, unioned together, are
//! exactly 4 digits → eliminate those 4 digits from every other cell in the
//! unit. Direct port of the Java reference (this one, unlike Naked
//! Pair/Triple, has a real, complete `apply()`).

use super::{combinations, digits_of};
use crate::technique::grid::{all_units, CandidateGrid};
use crate::technique::solve::{Technique, TechniqueEffect, TechniqueHint};

pub struct NakedQuad;

impl Technique for NakedQuad {
    fn id(&self) -> &'static str {
        "naked_quad"
    }

    fn find_hint(&self, grid: &CandidateGrid) -> Option<TechniqueHint> {
        for unit in all_units() {
            let cells: Vec<usize> = unit
                .iter()
                .copied()
                .filter(|&idx| grid.value_at(idx) == 0)
                .collect();
            // Need the quad plus at least one more cell to eliminate from.
            if cells.len() < 5 {
                continue;
            }

            for combo in combinations(cells.len(), 4) {
                let chosen: Vec<usize> = combo.iter().map(|&i| cells[i]).collect();
                let mask = chosen
                    .iter()
                    .fold(0u16, |acc, &idx| acc | grid.candidates_at(idx));
                if mask.count_ones() != 4 {
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
    fn empty_board_has_no_naked_quad() {
        let grid = CandidateGrid::from_board(&Board::empty()).unwrap();
        assert!(NakedQuad.find_hint(&grid).is_none());
    }

    #[test]
    fn eliminates_quad_digits_from_rest_of_unit() {
        let mut grid = CandidateGrid::from_board(&Board::empty()).unwrap();
        let unit = row_cells(0);
        let sets: [&[u8]; 4] = [&[1, 2], &[2, 3], &[3, 4], &[1, 4]];
        for (i, set) in sets.iter().enumerate() {
            let idx = unit[i];
            for d in 1..=9u8 {
                if !set.contains(&d) {
                    grid.remove_candidate(idx, d);
                }
            }
        }

        let hint = NakedQuad.find_hint(&grid).unwrap();
        match hint.effect {
            TechniqueEffect::Eliminate(removals) => {
                assert!(!removals.is_empty());
                for (idx, digit) in &removals {
                    assert!([1u8, 2, 3, 4].contains(digit));
                    assert!(!unit[0..4].contains(idx));
                }
            }
            _ => panic!("expected eliminations"),
        }
    }

    #[test]
    fn requires_at_least_five_empty_cells_in_the_unit() {
        let mut grid = CandidateGrid::from_board(&Board::empty()).unwrap();
        let unit = row_cells(0);
        let sets: [&[u8]; 4] = [&[1, 2], &[2, 3], &[3, 4], &[1, 4]];
        for (i, set) in sets.iter().enumerate() {
            let idx = unit[i];
            for d in 1..=9u8 {
                if !set.contains(&d) {
                    grid.remove_candidate(idx, d);
                }
            }
        }
        // Fill the rest of the row, leaving only the 4 quad cells empty.
        for &idx in &unit[4..9] {
            grid.place(idx, 9 - (idx as u8 % 5));
        }

        assert!(NakedQuad.find_hint(&grid).is_none());
    }
}
