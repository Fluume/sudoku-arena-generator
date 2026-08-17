//! Clue-pattern symmetry: which cells must be removed or kept *together*
//! while digging, so the finished puzzle's clue positions form a symmetric
//! pattern. Unrelated to [`sudoku_core::symmetry`], which transforms a
//! *solved* grid's contents (digits), not a puzzle's clue positions.

use sudoku_core::board::{col_of, index, row_of};

/// A clue-pattern symmetry to preserve while digging a puzzle out of a
/// solved grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Symmetry {
    /// No constraint — cells are removed independently.
    #[default]
    None,
    /// 180° rotational symmetry, the most common convention in published
    /// Sudoku.
    Central,
    /// Mirror across the main diagonal (top-left to bottom-right).
    Diagonal,
    /// Mirror across the anti-diagonal (top-right to bottom-left).
    AntiDiagonal,
    /// Both diagonals at once.
    BiDiagonal,
    /// Mirror across the horizontal axis.
    Horizontal,
    /// Mirror across the vertical axis.
    Vertical,
    /// All 8 symmetries of the square (the full dihedral group).
    Full,
}

type Transform = fn(usize, usize) -> (usize, usize);

fn central(r: usize, c: usize) -> (usize, usize) {
    (8 - r, 8 - c)
}

fn diagonal(r: usize, c: usize) -> (usize, usize) {
    (c, r)
}

fn anti_diagonal(r: usize, c: usize) -> (usize, usize) {
    (8 - c, 8 - r)
}

fn horizontal(r: usize, c: usize) -> (usize, usize) {
    (8 - r, c)
}

fn vertical(r: usize, c: usize) -> (usize, usize) {
    (r, 8 - c)
}

impl Symmetry {
    /// The generating transforms whose closure defines this symmetry's
    /// orbits. `BiDiagonal` and `Full` aren't listed with all their
    /// members explicitly — they're the closure of a couple of generators
    /// (e.g. `Full` = closure of {diagonal, horizontal} already yields all
    /// 8 dihedral symmetries), which is shorter and correct by
    /// construction rather than by manual case enumeration.
    fn generators(self) -> &'static [Transform] {
        match self {
            Symmetry::None => &[],
            Symmetry::Central => &[central],
            Symmetry::Diagonal => &[diagonal],
            Symmetry::AntiDiagonal => &[anti_diagonal],
            Symmetry::BiDiagonal => &[diagonal, anti_diagonal],
            Symmetry::Horizontal => &[horizontal],
            Symmetry::Vertical => &[vertical],
            Symmetry::Full => &[diagonal, horizontal],
        }
    }

    /// All cell indices that must be removed or kept together under this
    /// symmetry, including `idx` itself — the closure of `idx` under this
    /// symmetry's generating transforms. Orbits are smaller than the full
    /// generator count when `idx` lies on a symmetry axis (e.g. the center
    /// cell's orbit is always just itself).
    pub fn orbit(self, idx: usize) -> Vec<usize> {
        let generators = self.generators();
        let mut seen = vec![idx];
        let mut frontier = vec![idx];

        while let Some(cur) = frontier.pop() {
            let (r, c) = (row_of(cur), col_of(cur));
            for g in generators {
                let (nr, nc) = g(r, c);
                let next = index(nr, nc);
                if !seen.contains(&next) {
                    seen.push(next);
                    frontier.push(next);
                }
            }
        }

        seen.sort_unstable();
        seen
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn none_orbit_is_always_a_singleton() {
        for idx in [0, 1, 40, 80] {
            assert_eq!(Symmetry::None.orbit(idx), vec![idx]);
        }
    }

    #[test]
    fn central_pairs_opposite_corners_and_fixes_the_center() {
        assert_eq!(
            Symmetry::Central.orbit(index(0, 0)),
            vec![index(0, 0), index(8, 8)]
        );
        assert_eq!(Symmetry::Central.orbit(index(4, 4)), vec![index(4, 4)]);
    }

    #[test]
    fn diagonal_fixes_the_diagonal_and_pairs_off_diagonal_cells() {
        assert_eq!(Symmetry::Diagonal.orbit(index(0, 0)), vec![index(0, 0)]);
        assert_eq!(
            Symmetry::Diagonal.orbit(index(0, 1)),
            vec![index(0, 1), index(1, 0)]
        );
    }

    #[test]
    fn anti_diagonal_fixes_the_anti_diagonal() {
        assert_eq!(Symmetry::AntiDiagonal.orbit(index(0, 8)), vec![index(0, 8)]);
        assert_eq!(
            Symmetry::AntiDiagonal.orbit(index(0, 0)),
            vec![index(0, 0), index(8, 8)]
        );
    }

    #[test]
    fn bi_diagonal_orbit_of_a_generic_cell_has_all_four_quadrant_reflections() {
        let mut orbit = Symmetry::BiDiagonal.orbit(index(0, 1));
        orbit.sort_unstable();
        let mut expected = vec![index(0, 1), index(1, 0), index(7, 8), index(8, 7)];
        expected.sort_unstable();
        assert_eq!(orbit, expected);
    }

    #[test]
    fn horizontal_pairs_top_and_bottom_and_fixes_the_middle_row() {
        assert_eq!(
            Symmetry::Horizontal.orbit(index(0, 0)),
            vec![index(0, 0), index(8, 0)]
        );
        assert_eq!(Symmetry::Horizontal.orbit(index(4, 0)), vec![index(4, 0)]);
    }

    #[test]
    fn vertical_pairs_left_and_right_and_fixes_the_middle_column() {
        assert_eq!(
            Symmetry::Vertical.orbit(index(0, 0)),
            vec![index(0, 0), index(0, 8)]
        );
        assert_eq!(Symmetry::Vertical.orbit(index(0, 4)), vec![index(0, 4)]);
    }

    #[test]
    fn full_orbit_sizes_match_the_dihedral_group_action() {
        // Center: fixed by every symmetry.
        assert_eq!(Symmetry::Full.orbit(index(4, 4)).len(), 1);
        // Corners: 4-way orbit (stabilizer = the corner's own diagonal reflection).
        let corner = Symmetry::Full.orbit(index(0, 0));
        assert_eq!(corner.len(), 4);
        assert!(corner.contains(&index(0, 0)));
        assert!(corner.contains(&index(0, 8)));
        assert!(corner.contains(&index(8, 0)));
        assert!(corner.contains(&index(8, 8)));
        // Edge midpoints: 4-way orbit too.
        assert_eq!(Symmetry::Full.orbit(index(0, 4)).len(), 4);
        // A cell on no symmetry axis: the full 8-way orbit.
        assert_eq!(Symmetry::Full.orbit(index(0, 1)).len(), 8);
    }

    #[test]
    fn every_orbit_partitions_the_grid_without_overlap() {
        for symmetry in [
            Symmetry::None,
            Symmetry::Central,
            Symmetry::Diagonal,
            Symmetry::AntiDiagonal,
            Symmetry::BiDiagonal,
            Symmetry::Horizontal,
            Symmetry::Vertical,
            Symmetry::Full,
        ] {
            for idx in 0..81 {
                let orbit = symmetry.orbit(idx);
                assert!(orbit.contains(&idx));
                // Orbit membership must be symmetric: every member's own
                // orbit must be exactly the same set.
                for &member in &orbit {
                    assert_eq!(symmetry.orbit(member), orbit);
                }
            }
        }
    }
}
