//! One module per solving technique, each implementing
//! [`super::solve::Technique`]. See `ROADMAP.md` for which techniques are
//! implemented and which are still stubs.

use sudoku_core::bitmask::bit_to_digit;
use sudoku_core::board::{col_of, row_of};

use super::grid::{all_units, col_cells, row_cells, CandidateGrid};
use super::solve::{Technique, TechniqueEffect, TechniqueHint};

mod hidden_pair;
mod hidden_quad;
mod hidden_single;
mod hidden_triple;
mod jellyfish;
mod last_digit;
mod locked_candidates;
mod naked_pair;
mod naked_quad;
mod naked_single;
mod naked_triple;
mod skyscraper;
mod swordfish;
mod two_string_kite;
mod x_wing;
mod xy_wing;
mod xyz_wing;

/// All techniques with a real implementation, as fresh (stateless)
/// instances. [`super::solve::TechniqueSolver::from_hierarchy`] matches
/// these against the configured hierarchy by id.
pub fn all_known_techniques() -> Vec<Box<dyn Technique>> {
    vec![
        Box::new(last_digit::LastDigit),
        Box::new(hidden_single::HiddenSingleBlock),
        Box::new(hidden_single::HiddenSingleLine),
        Box::new(naked_single::NakedSingle),
        Box::new(locked_candidates::LockedCandidates),
        Box::new(naked_pair::NakedPair),
        Box::new(naked_triple::NakedTriple),
        Box::new(naked_quad::NakedQuad),
        Box::new(hidden_pair::HiddenPair),
        Box::new(hidden_triple::HiddenTriple),
        Box::new(hidden_quad::HiddenQuad),
        Box::new(x_wing::XWing),
        Box::new(swordfish::Swordfish),
        Box::new(jellyfish::Jellyfish),
        Box::new(skyscraper::Skyscraper),
        Box::new(two_string_kite::TwoStringKite),
        Box::new(xy_wing::XyWing),
        Box::new(xyz_wing::XyzWing),
    ]
}

/// Shared "Hidden N" search: `degree` digits whose combined candidate
/// positions within some unit are confined to exactly `degree` cells —
/// eliminate every other candidate from those cells.
///
/// A digit only counts toward a combination if it actually still has a
/// candidate cell in the unit; without that guard, a digit with zero
/// remaining candidates (already placed elsewhere in the unit) could pair
/// up with a single genuinely-confined digit and produce a bogus "hidden
/// pair" that wrongly strips unrelated real candidates — a soundness trap
/// the Java reference's `CommonTuples` helper doesn't guard against either.
pub(super) fn find_hidden_n(
    grid: &CandidateGrid,
    degree: usize,
    id: &'static str,
) -> Option<TechniqueHint> {
    for unit in all_units() {
        'combo: for combo in combinations(9, degree) {
            let values: Vec<u8> = combo.iter().map(|&i| i as u8 + 1).collect();

            for &v in &values {
                let occurs = unit
                    .iter()
                    .any(|&idx| grid.value_at(idx) == 0 && grid.has_candidate(idx, v));
                if !occurs {
                    continue 'combo;
                }
            }

            let positions: Vec<usize> = unit
                .iter()
                .copied()
                .filter(|&idx| {
                    grid.value_at(idx) == 0 && values.iter().any(|&v| grid.has_candidate(idx, v))
                })
                .collect();
            if positions.len() != degree {
                continue;
            }

            let removals: Vec<(usize, u8)> = positions
                .iter()
                .copied()
                .flat_map(|idx| {
                    let values = &values;
                    (1..=9u8)
                        .filter(move |d| !values.contains(d) && grid.has_candidate(idx, *d))
                        .map(move |d| (idx, d))
                })
                .collect();
            if !removals.is_empty() {
                return Some(TechniqueHint {
                    technique_id: id,
                    effect: TechniqueEffect::Eliminate(removals),
                });
            }
        }
    }
    None
}

/// Shared "fish" search (X-Wing degree 2, Swordfish 3, Jellyfish 4): a
/// digit confined, across `degree` rows, to `degree` columns overall (each
/// row contributing 2..=`degree` of those columns) — eliminate the digit
/// from those columns in every other row. Tries rows-based first, then the
/// column/row-swapped mirror.
///
/// The Java reference this is ported from loops candidate values `0..8`
/// instead of `1..=9` for this family of techniques, which never checks
/// digit 9 and always spuriously checks a nonexistent "digit 0" — fixed
/// here rather than reproduced.
pub(super) fn find_fish(
    grid: &CandidateGrid,
    degree: usize,
    id: &'static str,
) -> Option<TechniqueHint> {
    find_fish_direction(grid, degree, id, true)
        .or_else(|| find_fish_direction(grid, degree, id, false))
}

fn find_fish_direction(
    grid: &CandidateGrid,
    degree: usize,
    id: &'static str,
    by_rows: bool,
) -> Option<TechniqueHint> {
    for digit in 1..=9u8 {
        for combo in combinations(9, degree) {
            let mut union: Vec<usize> = Vec::new();
            let mut valid = true;

            for &line_idx in &combo {
                let cells = if by_rows {
                    row_cells(line_idx)
                } else {
                    col_cells(line_idx)
                };
                let mut positions: Vec<usize> = Vec::new();
                for idx in cells {
                    if grid.value_at(idx) == 0 && grid.has_candidate(idx, digit) {
                        let cross = if by_rows { col_of(idx) } else { row_of(idx) };
                        if !positions.contains(&cross) {
                            positions.push(cross);
                        }
                    }
                }
                if positions.is_empty() || positions.len() > degree {
                    valid = false;
                    break;
                }
                for p in positions {
                    if !union.contains(&p) {
                        union.push(p);
                    }
                }
                if union.len() > degree {
                    valid = false;
                    break;
                }
            }

            if !valid || union.len() != degree {
                continue;
            }

            let mut removals: Vec<(usize, u8)> = Vec::new();
            for other_line in 0..9 {
                if combo.contains(&other_line) {
                    continue;
                }
                let cells = if by_rows {
                    row_cells(other_line)
                } else {
                    col_cells(other_line)
                };
                for idx in cells {
                    let cross = if by_rows { col_of(idx) } else { row_of(idx) };
                    if union.contains(&cross)
                        && grid.value_at(idx) == 0
                        && grid.has_candidate(idx, digit)
                    {
                        removals.push((idx, digit));
                    }
                }
            }

            if !removals.is_empty() {
                return Some(TechniqueHint {
                    technique_id: id,
                    effect: TechniqueEffect::Eliminate(removals),
                });
            }
        }
    }
    None
}

/// Decodes a candidate bitmask into its digits, ascending. Shared by every
/// technique that needs to turn a small (pair/triple/quad-sized) mask back
/// into concrete digits to eliminate.
pub(super) fn digits_of(mask: u16) -> Vec<u8> {
    let mut remaining = mask;
    let mut out = Vec::with_capacity(mask.count_ones() as usize);
    while remaining != 0 {
        let bit = remaining & remaining.wrapping_neg();
        remaining &= remaining - 1;
        out.push(bit_to_digit(bit));
    }
    out
}

/// All k-combinations of `0..n`, in ascending order. `n` is always small
/// here (at most 9), so a plain recursive generator is simpler and safer
/// than porting the Java reference's stateful bit-trick iterator.
pub fn combinations(n: usize, k: usize) -> Vec<Vec<usize>> {
    let mut results = Vec::new();
    let mut current = Vec::with_capacity(k);
    combinations_from(0, n, k, &mut current, &mut results);
    results
}

fn combinations_from(
    start: usize,
    n: usize,
    k: usize,
    current: &mut Vec<usize>,
    results: &mut Vec<Vec<usize>>,
) {
    if current.len() == k {
        results.push(current.clone());
        return;
    }
    for i in start..n {
        current.push(i);
        combinations_from(i + 1, n, k, current, results);
        current.pop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn combinations_of_4_choose_2() {
        let combos = combinations(4, 2);
        assert_eq!(
            combos,
            vec![
                vec![0, 1],
                vec![0, 2],
                vec![0, 3],
                vec![1, 2],
                vec![1, 3],
                vec![2, 3]
            ]
        );
    }

    #[test]
    fn combinations_count_matches_binomial_coefficient() {
        // C(9,2)=36, C(9,3)=84, C(9,4)=126
        assert_eq!(combinations(9, 2).len(), 36);
        assert_eq!(combinations(9, 3).len(), 84);
        assert_eq!(combinations(9, 4).len(), 126);
    }

    #[test]
    fn combinations_of_k_equal_n_is_one_full_set() {
        assert_eq!(combinations(3, 3), vec![vec![0, 1, 2]]);
    }

    #[test]
    fn digits_of_decodes_a_mask() {
        use sudoku_core::bitmask::digit_to_bit;
        let mask = digit_to_bit(2) | digit_to_bit(7) | digit_to_bit(9);
        assert_eq!(digits_of(mask), vec![2, 7, 9]);
    }
}
