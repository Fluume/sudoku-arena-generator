use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde::Serialize;

use sudoku_core::Board;
use sudoku_generator::GradedPuzzle;

/// JSON-exportable view of a generated puzzle. `Board`'s `serde` impl
/// (feature-gated in `sudoku-core`) renders it as the 81-character line
/// format, so this is compact and interoperable with other Sudoku tools.
///
/// `difficulty`/`technique_counts` are only populated when the puzzle was
/// actually graded (difficulty range or training mode); plain generation
/// leaves them out of the JSON entirely (`skip_serializing_if`), so its
/// output shape is unchanged from before those modes existed.
#[derive(Debug, Serialize)]
pub struct PuzzleRecord {
    pub puzzle: Board,
    pub solution: Board,
    pub clue_count: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub difficulty: Option<u32>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub technique_counts: Vec<(String, u32)>,
}

impl From<&GradedPuzzle> for PuzzleRecord {
    fn from(graded: &GradedPuzzle) -> Self {
        PuzzleRecord {
            puzzle: graded.puzzle,
            solution: graded.solution,
            clue_count: graded.clue_count,
            difficulty: graded.max_weight,
            technique_counts: graded.technique_counts.clone(),
        }
    }
}

/// One 81-character line per puzzle, `0` for blanks.
pub fn render_text(puzzles: &[GradedPuzzle]) -> String {
    puzzles
        .iter()
        .map(|p| p.puzzle.to_line())
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn render_json(puzzles: &[GradedPuzzle], pretty: bool) -> Result<String> {
    let records: Vec<PuzzleRecord> = puzzles.iter().map(PuzzleRecord::from).collect();
    let json = if pretty {
        serde_json::to_string_pretty(&records)?
    } else {
        serde_json::to_string(&records)?
    };
    Ok(json)
}

pub fn write_output(content: &str, output: Option<&Path>) -> Result<()> {
    match output {
        Some(path) => fs::write(path, content)
            .with_context(|| format!("failed to write output file {}", path.display())),
        None => {
            println!("{content}");
            Ok(())
        }
    }
}
