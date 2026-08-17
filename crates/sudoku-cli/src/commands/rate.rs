use anyhow::{bail, Context, Result};

use sudoku_core::board::{col_of, row_of};
use sudoku_core::Board;
use sudoku_solver::technique::{TechniqueEffect, TechniqueHierarchy, TechniqueSolver};

use crate::cli::RateArgs;

pub fn run(args: &RateArgs) -> Result<()> {
    let board = Board::from_line(&args.puzzle).context("invalid puzzle line")?;

    let hierarchy = match &args.technique_config {
        Some(path) => TechniqueHierarchy::from_file(path)
            .with_context(|| format!("failed to load technique config {}", path.display()))?,
        None => TechniqueHierarchy::default_hierarchy(),
    };
    let solver = TechniqueSolver::from_hierarchy(&hierarchy);

    let Some(trace) = solver.solve(&board) else {
        bail!("puzzle is contradictory (duplicate digit in a row, column, or box)");
    };

    if trace.solved {
        println!("solved: yes");
    } else {
        println!("solved: no (needs a technique not in this hierarchy)");
    }

    match trace.max_weight {
        Some(weight) => println!("difficulty (max technique weight): {weight}"),
        None => println!("difficulty: none (already solved)"),
    }

    if trace.technique_counts.is_empty() {
        println!("techniques used: none");
    } else {
        println!("techniques used:");
        for (id, count) in &trace.technique_counts {
            let name = hierarchy
                .get(id)
                .map(|def| def.name.as_str())
                .unwrap_or(id.as_str());
            println!("  {name} ({id}): {count}");
        }
    }

    if args.steps {
        println!("steps:");
        for (i, step) in trace.steps.iter().enumerate() {
            let name = hierarchy
                .get(&step.technique_id)
                .map(|def| def.name.as_str())
                .unwrap_or(step.technique_id.as_str());
            let description = describe_effect(&step.effect);
            println!(
                "  {:>3}. [{}] {name} ({}): {description}",
                i + 1,
                step.weight,
                step.technique_id
            );
        }
    }

    Ok(())
}

/// A human-readable cell reference, `r{row}c{col}`, 1-indexed.
fn cell(idx: usize) -> String {
    format!("r{}c{}", row_of(idx) + 1, col_of(idx) + 1)
}

fn describe_effect(effect: &TechniqueEffect) -> String {
    match effect {
        TechniqueEffect::Place { idx, digit } => format!("place {digit} at {}", cell(*idx)),
        TechniqueEffect::Eliminate(removals) => {
            let parts: Vec<String> = removals
                .iter()
                .map(|(idx, digit)| format!("{digit} from {}", cell(*idx)))
                .collect();
            format!("eliminate {}", parts.join(", "))
        }
    }
}
