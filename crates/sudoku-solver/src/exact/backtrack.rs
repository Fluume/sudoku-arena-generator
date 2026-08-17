use sudoku_core::bitmask::bit_to_digit;
use sudoku_core::board::Board;

use super::bitgrid::{BitGrid, MrvResult};

/// Result of counting solutions up to a cap.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SolutionCount {
    Zero,
    One,
    Multiple,
}

/// Counts solutions to `board`, stopping as soon as `cap` is reached.
/// `cap` must be at least 1. For uniqueness checking, prefer
/// [`has_unique_solution`], which uses the cheapest cap (2).
pub fn count_solutions_capped(board: &Board, cap: u8) -> SolutionCount {
    assert!(cap >= 1, "cap must be at least 1");

    let Some(mut grid) = BitGrid::from_board(board) else {
        return SolutionCount::Zero;
    };

    let mut count: u32 = 0;
    recurse(&mut grid, &mut count, cap as u32);

    match count {
        0 => SolutionCount::Zero,
        1 => SolutionCount::One,
        _ => SolutionCount::Multiple,
    }
}

/// Whether `board` has exactly one solution. This is the hot-loop primitive
/// used while digging clues out of a solved grid: it stops searching the
/// instant a second solution is found, rather than enumerating every
/// solution.
pub fn has_unique_solution(board: &Board) -> bool {
    count_solutions_capped(board, 2) == SolutionCount::One
}

/// Returns `true` if the search reached `cap` and should stop immediately
/// (unwinding every recursive frame above it).
fn recurse(grid: &mut BitGrid, count: &mut u32, cap: u32) -> bool {
    match grid.select_mrv_cell() {
        MrvResult::Complete => {
            *count += 1;
            *count >= cap
        }
        MrvResult::Contradiction => false,
        MrvResult::Cell { idx, candidates } => {
            let mut remaining = candidates;
            while remaining != 0 {
                let bit = remaining & remaining.wrapping_neg();
                remaining &= remaining - 1;
                let digit = bit_to_digit(bit);

                grid.place(idx, digit);
                let stop = recurse(grid, count, cap);
                grid.unplace(idx, digit);

                if stop {
                    return true;
                }
            }
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Well-known example puzzle (Wikipedia "Sudoku" article) with a unique solution.
    const PUZZLE: &str =
        "53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..79";
    const SOLUTION: &str =
        "534678912672195348198342567859761423426853791713924856961537284287419635345286179";

    #[test]
    fn well_known_puzzle_has_unique_solution() {
        let board = Board::from_line(PUZZLE).unwrap();
        assert!(has_unique_solution(&board));
        assert_eq!(count_solutions_capped(&board, 2), SolutionCount::One);
    }

    #[test]
    fn completed_solution_counts_as_one() {
        let board = Board::from_line(SOLUTION).unwrap();
        assert_eq!(count_solutions_capped(&board, 2), SolutionCount::One);
    }

    #[test]
    fn contradictory_board_has_zero_solutions() {
        let mut board = Board::empty();
        board.set(0, 0, 5);
        board.set(0, 1, 5); // duplicate in the same row
        assert_eq!(count_solutions_capped(&board, 2), SolutionCount::Zero);
        assert!(!has_unique_solution(&board));
    }

    #[test]
    fn empty_board_has_multiple_solutions() {
        let board = Board::empty();
        assert_eq!(count_solutions_capped(&board, 2), SolutionCount::Multiple);
        assert!(!has_unique_solution(&board));
    }

    #[test]
    fn single_filled_row_is_far_underconstrained_and_has_multiple_solutions() {
        // Only 9 clues (one full row), well below the 17-clue minimum a
        // unique puzzle needs: exercises the early-exit-on-2nd-solution path
        // on a grid that isn't fully empty.
        let mut board = Board::empty();
        for (col, digit) in (1..=9u8).enumerate() {
            board.set(0, col, digit);
        }
        assert_eq!(count_solutions_capped(&board, 2), SolutionCount::Multiple);
    }
}
