//! Internal search state for the exact solver: cell values, incremental
//! row/col/box candidate bookkeeping, and a bitset of still-unset cells.
//! No search *policy* lives here — see `backtrack.rs` for that.

use sudoku_core::bitmask::UnitMasks;
use sudoku_core::board::{col_of, row_of, Board, CELLS};

pub(crate) struct BitGrid {
    unset: u128,
    masks: UnitMasks,
}

pub(crate) enum MrvResult {
    /// No unset cells remain: a full assignment was reached.
    Complete,
    /// Some unset cell has zero candidates: this branch is dead.
    Contradiction,
    /// The unset cell with the fewest candidates (ties broken by scan
    /// order), and its candidate bitmask.
    Cell { idx: usize, candidates: u16 },
}

impl BitGrid {
    /// Builds search state from a board. Returns `None` if the board is
    /// already contradictory (duplicate digit in some row/col/box).
    pub(crate) fn from_board(board: &Board) -> Option<Self> {
        let masks = UnitMasks::from_board(board)?;
        let mut unset: u128 = 0;
        for idx in 0..CELLS {
            if board.get_by_index(idx) == 0 {
                unset |= 1 << idx;
            }
        }
        Some(BitGrid { unset, masks })
    }

    pub(crate) fn place(&mut self, idx: usize, digit: u8) {
        self.unset &= !(1u128 << idx);
        self.masks.place(row_of(idx), col_of(idx), digit);
    }

    pub(crate) fn unplace(&mut self, idx: usize, digit: u8) {
        self.unset |= 1u128 << idx;
        self.masks.unplace(row_of(idx), col_of(idx), digit);
    }

    /// Picks the unset cell with the fewest remaining candidates (minimum
    /// remaining values), short-circuiting as soon as a contradiction
    /// (0 candidates) or a naked single (1 candidate) is found.
    pub(crate) fn select_mrv_cell(&self) -> MrvResult {
        if self.unset == 0 {
            return MrvResult::Complete;
        }

        let mut best: Option<(usize, u16, u32)> = None;
        let mut remaining = self.unset;
        while remaining != 0 {
            let idx = remaining.trailing_zeros() as usize;
            remaining &= remaining - 1;

            let candidates = self.masks.candidates_at(row_of(idx), col_of(idx));
            let count = candidates.count_ones();

            if count == 0 {
                return MrvResult::Contradiction;
            }
            if count == 1 {
                return MrvResult::Cell { idx, candidates };
            }
            if best.is_none_or(|(_, _, best_count)| count < best_count) {
                best = Some((idx, candidates, count));
            }
        }

        let (idx, candidates, _) = best.expect("unset != 0 implies at least one scanned cell");
        MrvResult::Cell { idx, candidates }
    }
}
