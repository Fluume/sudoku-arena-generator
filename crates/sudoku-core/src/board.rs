use crate::error::ParseError;

/// Side length of a Sudoku grid.
pub const SIZE: usize = 9;
/// Total number of cells in a Sudoku grid.
pub const CELLS: usize = 81;

/// Character used for a blank cell in the canonical single-line exchange format.
pub const BLANK_CHAR: char = '0';

#[inline]
pub const fn index(row: usize, col: usize) -> usize {
    row * SIZE + col
}

#[inline]
pub const fn row_of(idx: usize) -> usize {
    idx / SIZE
}

#[inline]
pub const fn col_of(idx: usize) -> usize {
    idx % SIZE
}

#[inline]
pub const fn box_of(idx: usize) -> usize {
    let r = row_of(idx);
    let c = col_of(idx);
    (r / 3) * 3 + c / 3
}

/// A 9x9 Sudoku board. `0` represents an empty cell, `1..=9` a placed digit.
///
/// This type makes no claim about validity (duplicate digits are representable);
/// callers that need a validity guarantee should go through
/// [`crate::bitmask::UnitMasks::from_board`], which detects contradictions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Board {
    cells: [u8; CELLS],
}

impl Board {
    /// A board with all 81 cells empty.
    pub fn empty() -> Self {
        Board { cells: [0; CELLS] }
    }

    /// Parses the canonical single-line exchange format: exactly 81 characters,
    /// each either a digit `1`-`9` or a blank marker (`0` or `.`).
    pub fn from_line(s: &str) -> Result<Self, ParseError> {
        let chars: Vec<char> = s.chars().collect();
        if chars.len() != CELLS {
            return Err(ParseError::WrongLength {
                expected: CELLS,
                actual: chars.len(),
            });
        }

        let mut cells = [0u8; CELLS];
        for (i, ch) in chars.into_iter().enumerate() {
            cells[i] = match ch {
                '1'..='9' => ch as u8 - b'0',
                '0' | '.' => 0,
                other => {
                    return Err(ParseError::InvalidChar {
                        index: i,
                        found: other,
                    })
                }
            };
        }
        Ok(Board { cells })
    }

    /// Renders the board as the canonical single-line exchange format
    /// (blanks as `0`), suitable for interop with other Sudoku tools.
    pub fn to_line(&self) -> String {
        self.cells
            .iter()
            .map(|&c| {
                if c == 0 {
                    BLANK_CHAR
                } else {
                    (b'0' + c) as char
                }
            })
            .collect()
    }

    /// Builds a board from a `[row][col]` array of digits (`0` = empty).
    pub fn from_array(grid: [[u8; SIZE]; SIZE]) -> Self {
        let mut cells = [0u8; CELLS];
        for (row, row_values) in grid.iter().enumerate() {
            for (col, &value) in row_values.iter().enumerate() {
                cells[index(row, col)] = value;
            }
        }
        Board { cells }
    }

    /// Exports the board as a `[row][col]` array of digits (`0` = empty).
    pub fn to_array(&self) -> [[u8; SIZE]; SIZE] {
        let mut grid = [[0u8; SIZE]; SIZE];
        for (row, row_values) in grid.iter_mut().enumerate() {
            for (col, value) in row_values.iter_mut().enumerate() {
                *value = self.cells[index(row, col)];
            }
        }
        grid
    }

    #[inline]
    pub fn get(&self, row: usize, col: usize) -> u8 {
        self.cells[index(row, col)]
    }

    #[inline]
    pub fn set(&mut self, row: usize, col: usize, value: u8) {
        self.cells[index(row, col)] = value;
    }

    #[inline]
    pub fn get_by_index(&self, idx: usize) -> u8 {
        self.cells[idx]
    }

    #[inline]
    pub fn set_by_index(&mut self, idx: usize, value: u8) {
        self.cells[idx] = value;
    }

    /// Number of non-empty cells.
    pub fn clue_count(&self) -> u8 {
        self.cells.iter().filter(|&&c| c != 0).count() as u8
    }

    /// Whether every cell is filled (does not imply validity).
    pub fn is_complete(&self) -> bool {
        self.cells.iter().all(|&c| c != 0)
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Board {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_line())
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Board {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Board::from_line(&s).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_LINE: &str =
        "534678912672195348198342567859761423426853791713924856961537284287419635345286179";

    #[test]
    fn round_trips_line_format() {
        let board = Board::from_line(VALID_LINE).unwrap();
        assert_eq!(board.to_line(), VALID_LINE);
        assert!(board.is_complete());
        assert_eq!(board.clue_count(), 81);
    }

    #[test]
    fn round_trips_array_format() {
        let board = Board::from_line(VALID_LINE).unwrap();
        let array = board.to_array();
        let rebuilt = Board::from_array(array);
        assert_eq!(board, rebuilt);
    }

    #[test]
    fn rejects_wrong_length() {
        assert_eq!(
            Board::from_line("123"),
            Err(ParseError::WrongLength {
                expected: 81,
                actual: 3
            })
        );
    }

    #[test]
    fn rejects_invalid_char() {
        let mut s = ".".repeat(81);
        s.replace_range(5..6, "x");
        assert_eq!(
            Board::from_line(&s),
            Err(ParseError::InvalidChar {
                index: 5,
                found: 'x'
            })
        );
    }

    #[test]
    fn to_line_renders_blanks_as_zero() {
        let board = Board::empty();
        assert_eq!(board.to_line(), "0".repeat(81));
    }

    #[test]
    fn blank_dot_and_zero_are_equivalent() {
        let dots = ".".repeat(81);
        let zeros = "0".repeat(81);
        assert_eq!(
            Board::from_line(&dots).unwrap(),
            Board::from_line(&zeros).unwrap()
        );
    }

    #[test]
    fn box_of_matches_expected_layout() {
        // Top-left 3x3 box is box 0, top-middle is box 1, middle-left is box 3, center is box 4.
        assert_eq!(box_of(index(0, 0)), 0);
        assert_eq!(box_of(index(0, 4)), 1);
        assert_eq!(box_of(index(3, 0)), 3);
        assert_eq!(box_of(index(4, 4)), 4);
        assert_eq!(box_of(index(8, 8)), 8);
    }
}
