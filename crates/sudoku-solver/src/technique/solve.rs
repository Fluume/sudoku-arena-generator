//! Drives the technique detectors: given a puzzle, repeatedly apply the
//! easiest applicable technique (per the configured [`TechniqueHierarchy`])
//! until the grid is solved or no known technique applies.

use std::collections::HashMap;

use sudoku_core::board::Board;

use super::detect;
use super::grid::CandidateGrid;
use super::hierarchy::{TechniqueDef, TechniqueHierarchy};

/// What applying a [`TechniqueHint`] does to the grid.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TechniqueEffect {
    /// Places a single digit.
    Place { idx: usize, digit: u8 },
    /// Removes one or more (cell, digit) candidates.
    Eliminate(Vec<(usize, u8)>),
}

/// A single applicable move found by a [`Technique`].
pub struct TechniqueHint {
    pub technique_id: &'static str,
    pub effect: TechniqueEffect,
}

/// A solving technique: scans the grid and reports the first move it finds,
/// if any. Implementations live under [`super::detect`], one (or two, for
/// Hidden Single) per technique.
///
/// `Send + Sync` so a [`TechniqueSolver`] can be shared read-only across
/// threads (parallel batch generation) — every implementation is a
/// stateless unit struct, so this is satisfied automatically.
pub trait Technique: Send + Sync {
    /// Stable id, matching a [`TechniqueDef::id`] in the hierarchy config.
    fn id(&self) -> &'static str;
    fn find_hint(&self, grid: &CandidateGrid) -> Option<TechniqueHint>;
}

/// Applies a hint's effect to the grid. Placements propagate incrementally
/// via [`CandidateGrid::place`]; eliminations never trigger a full rebuild
/// (see that method's doc comment for why that matters).
pub fn apply_hint(grid: &mut CandidateGrid, hint: &TechniqueHint) {
    match &hint.effect {
        TechniqueEffect::Place { idx, digit } => grid.place(*idx, *digit),
        TechniqueEffect::Eliminate(removals) => {
            for &(idx, digit) in removals {
                grid.remove_candidate(idx, digit);
            }
        }
    }
}

/// A single step in a [`SolveTrace`]: which technique fired, its weight,
/// and exactly what it did. Lets a difficulty rating that looks wrong be
/// checked step by step — e.g. confirming that whichever technique is
/// reported as the hardest actually fired, and on what cell/digit, rather
/// than just trusting the aggregate weight.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppliedStep {
    pub technique_id: String,
    pub weight: u32,
    pub effect: TechniqueEffect,
}

/// The outcome of running a [`TechniqueSolver`] over a puzzle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SolveTrace {
    /// Whether the grid was fully solved using only the configured
    /// techniques (a `false` here means the puzzle needs a technique this
    /// hierarchy doesn't cover — not that it's unsolvable).
    pub solved: bool,
    /// Difficulty rating: the weight of the hardest technique applied.
    /// `None` if the puzzle was already solved (no technique needed).
    pub max_weight: Option<u32>,
    /// How many times each technique fired, in first-applied order —
    /// groundwork for the training-mode milestone (grids that maximize one
    /// technique's occurrence count).
    pub technique_counts: Vec<(String, u32)>,
    /// The full ordered sequence of applied steps, one per technique
    /// application — see [`AppliedStep`].
    pub steps: Vec<AppliedStep>,
}

/// Runs the technique hierarchy's techniques, easiest first, applying one
/// hint at a time.
pub struct TechniqueSolver {
    ordered: Vec<(TechniqueDef, Box<dyn Technique>)>,
}

impl TechniqueSolver {
    /// Builds a solver from a hierarchy: techniques are tried in ascending
    /// weight order. Hierarchy entries with no matching implementation in
    /// [`detect::all_known_techniques`] are silently skipped, so the config
    /// can describe techniques that aren't implemented yet.
    pub fn from_hierarchy(hierarchy: &TechniqueHierarchy) -> Self {
        let mut known: HashMap<&'static str, Box<dyn Technique>> = detect::all_known_techniques()
            .into_iter()
            .map(|t| (t.id(), t))
            .collect();

        let mut defs: Vec<&TechniqueDef> = hierarchy.techniques.iter().collect();
        defs.sort_by_key(|def| def.weight);

        let ordered = defs
            .into_iter()
            .filter_map(|def| {
                known
                    .remove(def.id.as_str())
                    .map(|technique| (def.clone(), technique))
            })
            .collect();

        TechniqueSolver { ordered }
    }

    /// Convenience constructor using the bundled default hierarchy.
    pub fn with_default_hierarchy() -> Self {
        Self::from_hierarchy(&TechniqueHierarchy::default_hierarchy())
    }

    /// Solves `board` using this solver's techniques. Returns `None` if
    /// `board` is contradictory (see [`CandidateGrid::from_board`]).
    pub fn solve(&self, board: &Board) -> Option<SolveTrace> {
        let mut grid = CandidateGrid::from_board(board)?;
        let mut max_weight: Option<u32> = None;
        let mut technique_counts: Vec<(String, u32)> = Vec::new();
        let mut steps: Vec<AppliedStep> = Vec::new();

        loop {
            if grid.is_solved() {
                return Some(SolveTrace {
                    solved: true,
                    max_weight,
                    technique_counts,
                    steps,
                });
            }

            let found = self
                .ordered
                .iter()
                .find_map(|(def, technique)| technique.find_hint(&grid).map(|hint| (def, hint)));

            let Some((def, hint)) = found else {
                return Some(SolveTrace {
                    solved: false,
                    max_weight,
                    technique_counts,
                    steps,
                });
            };

            steps.push(AppliedStep {
                technique_id: def.id.clone(),
                weight: def.weight,
                effect: hint.effect.clone(),
            });
            apply_hint(&mut grid, &hint);
            max_weight = Some(max_weight.map_or(def.weight, |w| w.max(def.weight)));
            record_count(&mut technique_counts, def.id.as_str());
        }
    }
}

fn record_count(counts: &mut Vec<(String, u32)>, id: &str) {
    match counts.iter_mut().find(|(existing_id, _)| existing_id == id) {
        Some(entry) => entry.1 += 1,
        None => counts.push((id.to_string(), 1)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EASY_PUZZLE: &str =
        "53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..79";
    const SOLUTION: &str =
        "534678912672195348198342567859761423426853791713924856961537284287419635345286179";

    #[test]
    fn already_solved_board_needs_no_technique() {
        let board = Board::from_line(SOLUTION).unwrap();
        let solver = TechniqueSolver::with_default_hierarchy();
        let trace = solver.solve(&board).unwrap();
        assert!(trace.solved);
        assert_eq!(trace.max_weight, None);
        assert!(trace.technique_counts.is_empty());
    }

    #[test]
    fn well_known_easy_puzzle_solves_with_singles_only() {
        let board = Board::from_line(EASY_PUZZLE).unwrap();
        let solver = TechniqueSolver::with_default_hierarchy();
        let trace = solver.solve(&board).unwrap();
        assert!(
            trace.solved,
            "expected the well-known example to be fully solved"
        );

        // Every technique that fired should be a single (last digit/
        // block/line/naked), never anything harder, for this famously easy
        // example.
        for (id, _) in &trace.technique_counts {
            assert!(
                id == "last_digit"
                    || id == "hidden_single_block"
                    || id == "hidden_single_line"
                    || id == "naked_single",
                "unexpected technique fired: {id}"
            );
        }
    }

    #[test]
    fn steps_record_the_full_applied_sequence_in_order() {
        let board = Board::from_line(EASY_PUZZLE).unwrap();
        let solver = TechniqueSolver::with_default_hierarchy();
        let trace = solver.solve(&board).unwrap();

        // The step log's technique_id counts must match technique_counts
        // exactly (same data, two views), and every step's weight must be
        // <= the reported max_weight, with at least one step hitting it.
        let mut counts_from_steps: Vec<(String, u32)> = Vec::new();
        for step in &trace.steps {
            record_count(&mut counts_from_steps, &step.technique_id);
            assert!(step.weight <= trace.max_weight.unwrap());
        }
        assert!(trace
            .steps
            .iter()
            .any(|s| s.weight == trace.max_weight.unwrap()));

        let mut expected = trace.technique_counts.clone();
        let mut actual = counts_from_steps;
        expected.sort();
        actual.sort();
        assert_eq!(expected, actual);
    }

    #[test]
    fn contradictory_board_returns_none() {
        let mut board = Board::empty();
        board.set(0, 0, 5);
        board.set(0, 1, 5);
        let solver = TechniqueSolver::with_default_hierarchy();
        assert!(solver.solve(&board).is_none());
    }
}
