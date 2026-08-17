//! A persistent, per-cell candidate grid ("pencil marks"): unlike
//! [`sudoku_core::bitmask::UnitMasks`], which only ever derives candidates
//! from currently-placed digits, this grid supports individual
//! (cell, digit) eliminations that aren't implied by any placement — e.g.
//! "digit 4 removed from cell 12 by a Naked Pair". That's the primitive
//! every technique in [`super::detect`] needs.

use sudoku_core::bitmask::{digit_to_bit, UnitMasks, FULL_MASK};
use sudoku_core::board::{box_of, col_of, index, row_of, Board, CELLS};

/// A 9x9 grid tracking, for every still-empty cell, which digits remain
/// candidates. Placed cells' candidate entries are always `0`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CandidateGrid {
    values: [u8; CELLS],
    candidates: [u16; CELLS],
}

impl CandidateGrid {
    /// Builds a grid from a board, with candidates already computed via
    /// [`CandidateGrid::rebuild_candidates`]. Returns `None` if the board is
    /// contradictory (duplicate digit in some row/col/box).
    pub fn from_board(board: &Board) -> Option<Self> {
        UnitMasks::from_board(board)?;

        let mut values = [0u8; CELLS];
        for (idx, value) in values.iter_mut().enumerate() {
            *value = board.get_by_index(idx);
        }

        let mut grid = CandidateGrid {
            values,
            candidates: [FULL_MASK; CELLS],
        };
        grid.rebuild_candidates();
        Some(grid)
    }

    #[inline]
    pub fn value_at(&self, idx: usize) -> u8 {
        self.values[idx]
    }

    #[inline]
    pub fn candidates_at(&self, idx: usize) -> u16 {
        self.candidates[idx]
    }

    #[inline]
    pub fn candidate_count(&self, idx: usize) -> u32 {
        self.candidates[idx].count_ones()
    }

    #[inline]
    pub fn has_candidate(&self, idx: usize, digit: u8) -> bool {
        self.candidates[idx] & digit_to_bit(digit) != 0
    }

    /// Sets `idx`'s own value and collapses its own candidates to just
    /// `digit`. Low-level primitive: does **not** touch peer cells — see
    /// [`CandidateGrid::place`] for the operation the solve loop actually
    /// uses.
    pub fn set_value(&mut self, idx: usize, digit: u8) {
        debug_assert!((1..=9).contains(&digit), "set_value expects a digit 1..=9");
        self.values[idx] = digit;
        self.candidates[idx] = digit_to_bit(digit);
    }

    /// Places `digit` at `idx` and immediately, *incrementally* removes it
    /// from every still-empty peer (same row/col/box) — this is the only
    /// placement path the technique solver uses.
    ///
    /// This deliberately does **not** go through a full
    /// [`CandidateGrid::rebuild_candidates`] pass. The Java reference this
    /// engine is ported from calls its equivalent full reset-and-reapply
    /// after *every* hint, including elimination-only ones — which silently
    /// discards any candidate an indirect technique (Naked Pair, X-Wing,
    /// ...) just removed, since a full rebuild only re-derives eliminations
    /// implied by currently placed digits. That bug makes any puzzle that
    /// needs an indirect technique to progress loop forever (the same
    /// eliminated candidate reappears, the same hint fires again,
    /// indefinitely). Propagating only the newly placed digit — never
    /// resetting anything — keeps earlier eliminations intact.
    pub fn place(&mut self, idx: usize, digit: u8) {
        self.set_value(idx, digit);
        let bit = digit_to_bit(digit);
        let (r, c, b) = (row_of(idx), col_of(idx), box_of(idx));
        for peer in row_cells(r)
            .into_iter()
            .chain(col_cells(c))
            .chain(box_cells(b))
        {
            if self.values[peer] == 0 {
                self.candidates[peer] &= !bit;
            }
        }
    }

    /// Removes `digit` from `idx`'s candidates, if present.
    pub fn remove_candidate(&mut self, idx: usize, digit: u8) {
        self.candidates[idx] &= !digit_to_bit(digit);
    }

    /// Recomputes every empty cell's candidates from scratch: full 1-9,
    /// then eliminate each placed cell's digit from its still-empty peers.
    ///
    /// A full reset-and-reapply pass, safe only as a one-time
    /// initialization (see [`CandidateGrid::from_board`]) — calling this
    /// again mid-solve would discard any indirect technique's eliminations,
    /// see [`CandidateGrid::place`]'s doc comment for why that matters.
    pub fn rebuild_candidates(&mut self) {
        for idx in 0..CELLS {
            self.candidates[idx] = if self.values[idx] == 0 { FULL_MASK } else { 0 };
        }

        for idx in 0..CELLS {
            let digit = self.values[idx];
            if digit == 0 {
                continue;
            }
            let bit = digit_to_bit(digit);
            let (r, c, b) = (row_of(idx), col_of(idx), box_of(idx));
            for peer in row_cells(r)
                .into_iter()
                .chain(col_cells(c))
                .chain(box_cells(b))
            {
                if self.values[peer] == 0 {
                    self.candidates[peer] &= !bit;
                }
            }
        }
    }

    pub fn is_solved(&self) -> bool {
        self.values.iter().all(|&v| v != 0)
    }

    pub fn to_board(&self) -> Board {
        let mut board = Board::empty();
        for (idx, &value) in self.values.iter().enumerate() {
            board.set_by_index(idx, value);
        }
        board
    }
}

/// Whether `a` and `b` are different cells sharing a row, column, or box —
/// the standard Sudoku "sees" relation used by elimination techniques.
#[inline]
pub fn sees(a: usize, b: usize) -> bool {
    a != b && (row_of(a) == row_of(b) || col_of(a) == col_of(b) || box_of(a) == box_of(b))
}

pub fn row_cells(r: usize) -> [usize; 9] {
    let mut cells = [0usize; 9];
    for (c, cell) in cells.iter_mut().enumerate() {
        *cell = index(r, c);
    }
    cells
}

pub fn col_cells(c: usize) -> [usize; 9] {
    let mut cells = [0usize; 9];
    for (r, cell) in cells.iter_mut().enumerate() {
        *cell = index(r, c);
    }
    cells
}

pub fn box_cells(b: usize) -> [usize; 9] {
    let base_row = (b / 3) * 3;
    let base_col = (b % 3) * 3;
    let mut cells = [0usize; 9];
    let mut i = 0;
    for dr in 0..3 {
        for dc in 0..3 {
            cells[i] = index(base_row + dr, base_col + dc);
            i += 1;
        }
    }
    cells
}

/// All 27 units (9 rows, then 9 columns, then 9 boxes) — for techniques
/// that scan uniformly across region kinds instead of hand-writing three
/// separate loops.
pub fn all_units() -> [[usize; 9]; 27] {
    let mut units = [[0usize; 9]; 27];
    for (r, unit) in units.iter_mut().take(9).enumerate() {
        *unit = row_cells(r);
    }
    for c in 0..9 {
        units[9 + c] = col_cells(c);
    }
    for b in 0..9 {
        units[18 + b] = box_cells(b);
    }
    units
}

#[cfg(test)]
mod tests {
    use super::*;

    const PUZZLE: &str =
        "53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..79";

    #[test]
    fn empty_board_has_full_candidates_everywhere() {
        let grid = CandidateGrid::from_board(&Board::empty()).unwrap();
        for idx in 0..CELLS {
            assert_eq!(grid.candidates_at(idx), FULL_MASK);
            assert_eq!(grid.value_at(idx), 0);
        }
        assert!(!grid.is_solved());
    }

    #[test]
    fn contradictory_board_returns_none() {
        let mut board = Board::empty();
        board.set(0, 0, 5);
        board.set(0, 1, 5);
        assert!(CandidateGrid::from_board(&board).is_none());
    }

    #[test]
    fn from_board_eliminates_peer_candidates_of_givens() {
        let board = Board::from_line(PUZZLE).unwrap();
        let grid = CandidateGrid::from_board(&board).unwrap();
        // Cell (0,2) is empty in the well-known example; row 0 has givens
        // 5 (col0) and 3 (col1), and box 0 has 5,3,6 — none of those should
        // remain candidates there.
        let idx = index(0, 2);
        assert_eq!(grid.value_at(idx), 0);
        assert!(!grid.has_candidate(idx, 5));
        assert!(!grid.has_candidate(idx, 3));
        assert!(!grid.has_candidate(idx, 6));
    }

    #[test]
    fn set_value_collapses_own_candidates_without_touching_peers() {
        let mut grid = CandidateGrid::from_board(&Board::empty()).unwrap();
        let idx = index(4, 4);
        grid.set_value(idx, 7);
        assert_eq!(grid.value_at(idx), 7);
        assert_eq!(grid.candidates_at(idx), digit_to_bit(7));

        // A peer's candidates are untouched until rebuild_candidates runs.
        let peer = index(4, 0);
        assert!(grid.has_candidate(peer, 7));
    }

    #[test]
    fn rebuild_candidates_propagates_placements_to_peers() {
        let mut grid = CandidateGrid::from_board(&Board::empty()).unwrap();
        let idx = index(4, 4);
        grid.set_value(idx, 7);
        grid.rebuild_candidates();

        assert!(!grid.has_candidate(index(4, 0), 7)); // same row
        assert!(!grid.has_candidate(index(0, 4), 7)); // same column
        assert!(!grid.has_candidate(index(3, 3), 7)); // same box
        assert!(grid.has_candidate(index(8, 8), 7)); // unrelated cell keeps it
    }

    #[test]
    fn place_propagates_incrementally_without_discarding_earlier_eliminations() {
        let mut grid = CandidateGrid::from_board(&Board::empty()).unwrap();

        // Simulate an indirect technique eliminating candidate 9 from a
        // cell unrelated to the upcoming placement.
        let untouched = index(8, 8);
        grid.remove_candidate(untouched, 9);
        assert!(!grid.has_candidate(untouched, 9));

        // Placing an unrelated digit elsewhere must not resurrect that
        // earlier elimination (this is exactly the bug found in the Java
        // reference's full-rebuild-after-every-hint loop).
        grid.place(index(0, 0), 1);
        assert!(!grid.has_candidate(untouched, 9));

        // But peers of the placement do lose the placed digit.
        assert!(!grid.has_candidate(index(0, 5), 1));
    }

    #[test]
    fn remove_candidate_only_affects_the_targeted_cell() {
        let mut grid = CandidateGrid::from_board(&Board::empty()).unwrap();
        let idx = index(2, 2);
        grid.remove_candidate(idx, 3);
        assert!(!grid.has_candidate(idx, 3));
        assert_eq!(grid.candidate_count(idx), 8);
        assert!(grid.has_candidate(index(2, 3), 3));
    }

    #[test]
    fn to_board_round_trips_values() {
        let board = Board::from_line(PUZZLE).unwrap();
        let grid = CandidateGrid::from_board(&board).unwrap();
        assert_eq!(grid.to_board(), board);
    }

    #[test]
    fn sees_relation() {
        assert!(sees(index(0, 0), index(0, 5))); // same row
        assert!(sees(index(0, 0), index(5, 0))); // same col
        assert!(sees(index(0, 0), index(1, 1))); // same box
        assert!(!sees(index(0, 0), index(0, 0))); // self
        assert!(!sees(index(0, 0), index(4, 5))); // unrelated
    }

    #[test]
    fn row_col_box_cells_have_expected_membership() {
        assert_eq!(row_cells(3), [27, 28, 29, 30, 31, 32, 33, 34, 35]);
        assert_eq!(col_cells(0), [0, 9, 18, 27, 36, 45, 54, 63, 72]);
        assert_eq!(box_cells(4), [30, 31, 32, 39, 40, 41, 48, 49, 50]);
    }

    #[test]
    fn all_units_has_27_units_of_9_cells() {
        let units = all_units();
        assert_eq!(units.len(), 27);
        for unit in units {
            let mut sorted: Vec<usize> = unit.to_vec();
            sorted.sort_unstable();
            sorted.dedup();
            assert_eq!(sorted.len(), 9, "unit cells must be distinct");
        }
    }
}
