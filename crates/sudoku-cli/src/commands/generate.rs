use anyhow::{Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use rand::{rngs::StdRng, SeedableRng};

use sudoku_generator::dig::{Symmetry, TopDown};
use sudoku_generator::fill::{FillMode, FullGridGenerator};
use sudoku_generator::{generate_matching, generate_puzzle, parallel_batches, GradedPuzzle};
use sudoku_solver::technique::{TechniqueHierarchy, TechniqueSolver};

use crate::cli::{FillModeArg, FormatArg, GenerateArgs, SymmetryArg};
use crate::output::{render_json, render_text, write_output};

/// A difficulty/training constraint to satisfy via [`generate_matching`].
/// Plain generation (neither set) skips grading entirely — see `run` below.
enum Mode {
    DifficultyRange {
        min: Option<u32>,
        max: Option<u32>,
    },
    Training {
        id: String,
        weight: u32,
        min_count: u32,
    },
}

pub fn run(args: &GenerateArgs) -> Result<()> {
    let fill_mode = match args.fill_mode {
        FillModeArg::Random => FillMode::RandomBacktracking,
        FillModeArg::Canonical => FillMode::CanonicalWithTransforms {
            reseed_every: args.reseed_every,
        },
    };
    let symmetry = match args.symmetry {
        SymmetryArg::None => Symmetry::None,
        SymmetryArg::Central => Symmetry::Central,
        SymmetryArg::Diagonal => Symmetry::Diagonal,
        SymmetryArg::AntiDiagonal => Symmetry::AntiDiagonal,
        SymmetryArg::BiDiagonal => Symmetry::BiDiagonal,
        SymmetryArg::Horizontal => Symmetry::Horizontal,
        SymmetryArg::Vertical => Symmetry::Vertical,
        SymmetryArg::Full => Symmetry::Full,
    };
    let min_clues = args.min_clues;
    let attempts = args.attempts;

    // Built once, shared read-only across every parallel chunk — `None`
    // when no grading was requested, so plain generation never pays for
    // technique solving at all.
    let (solver, mode) = if args.train_technique.is_some()
        || args.min_difficulty.is_some()
        || args.max_difficulty.is_some()
    {
        let hierarchy = match &args.technique_config {
            Some(path) => TechniqueHierarchy::from_file(path)
                .with_context(|| format!("failed to load technique config {}", path.display()))?,
            None => TechniqueHierarchy::default_hierarchy(),
        };
        let mode = match &args.train_technique {
            Some(id) => {
                let weight = hierarchy.get(id).map(|def| def.weight).with_context(|| {
                    format!("unknown technique id '{id}' in the technique hierarchy")
                })?;
                Mode::Training {
                    id: id.clone(),
                    weight,
                    min_count: args.min_technique_count,
                }
            }
            None => Mode::DifficultyRange {
                min: args.min_difficulty,
                max: args.max_difficulty,
            },
        };
        (
            Some(TechniqueSolver::from_hierarchy(&hierarchy)),
            Some(mode),
        )
    } else {
        (None, None)
    };

    let progress = ProgressBar::new(args.count as u64);
    if let Ok(style) =
        ProgressStyle::with_template("{bar:40.cyan/blue} {pos}/{len} puzzles ({eta})")
    {
        progress.set_style(style);
    }

    // Each chunk (one per available core, roughly) gets its own RNG, full-
    // grid generator, and digging strategy — mutable state that can't be
    // shared across threads — while `solver`/`mode` are read-only and
    // shared by reference. See `parallel_batches`'s doc comment for the
    // reproducibility guarantees this gives (same seed + same thread
    // count/machine).
    let puzzles = parallel_batches(args.count, args.seed, |chunk_count, chunk_seed| {
        let mut rng = StdRng::seed_from_u64(chunk_seed);
        let mut fill_gen = FullGridGenerator::new(fill_mode);
        let mut strategy = TopDown::new(symmetry).with_min_clues(min_clues);

        let mut chunk_puzzles = Vec::with_capacity(chunk_count as usize);
        for _ in 0..chunk_count {
            let graded = match (&solver, &mode) {
                (
                    Some(solver),
                    Some(Mode::Training {
                        id,
                        weight,
                        min_count,
                    }),
                ) => generate_matching(
                    &mut strategy,
                    &mut fill_gen,
                    solver,
                    &mut rng,
                    attempts,
                    |g| {
                        if g.max_weight.unwrap_or(0) > *weight {
                            return None;
                        }
                        let count = g
                            .technique_counts
                            .iter()
                            .find(|(tid, _)| tid == id)
                            .map(|(_, c)| *c)
                            .unwrap_or(0);
                        (count >= *min_count).then_some(count as i64)
                    },
                ),
                (Some(solver), Some(Mode::DifficultyRange { min, max })) => {
                    let (min, max) = (*min, *max);
                    generate_matching(
                        &mut strategy,
                        &mut fill_gen,
                        solver,
                        &mut rng,
                        attempts,
                        |g| {
                            let difficulty = g.max_weight.unwrap_or(0);
                            let ok = min.is_none_or(|m| difficulty >= m)
                                && max.is_none_or(|m| difficulty <= m);
                            ok.then_some(0)
                        },
                    )
                }
                _ => {
                    let generated = generate_puzzle(&mut strategy, &mut fill_gen, &mut rng);
                    // Plain mode never runs the technique solver at all, so
                    // `solved` doesn't really apply — `true` since nothing
                    // failed, and this record never passes through
                    // `generate_matching`'s solved check anyway.
                    Some(GradedPuzzle {
                        puzzle: generated.puzzle,
                        solution: generated.solution,
                        clue_count: generated.clue_count,
                        solved: true,
                        max_weight: None,
                        technique_counts: Vec::new(),
                    })
                }
            };

            match graded {
                Some(p) => chunk_puzzles.push(p),
                None => {
                    let message = format!(
                        "warning: no puzzle found matching the requested constraint within {attempts} attempts, skipping"
                    );
                    // `ProgressBar::println` silently no-ops when the draw
                    // target is hidden (stderr isn't an interactive
                    // terminal — piped, redirected, CI) — fall back to a
                    // plain `eprintln!` there so the warning is never lost.
                    if progress.is_hidden() {
                        eprintln!("{message}");
                    } else {
                        progress.println(message);
                    }
                }
            }
            progress.inc(1);
        }

        chunk_puzzles
    });

    progress.finish_and_clear();

    let rendered = match args.format {
        FormatArg::Text => render_text(&puzzles),
        FormatArg::Json => render_json(&puzzles, args.pretty)?,
    };

    write_output(&rendered, args.output.as_deref())
}
