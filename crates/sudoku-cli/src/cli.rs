use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};

#[derive(Debug, Parser)]
#[command(name = "sudoku-gen", version, about = "Bulk Sudoku puzzle generator")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Generate a batch of puzzles.
    Generate(GenerateArgs),
    /// Rate a single puzzle using the technique-based solver.
    Rate(RateArgs),
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum FillModeArg {
    /// Fresh randomized backtracking fill every time.
    Random,
    /// Canonical seed grid + random symmetry transforms, periodically
    /// reseeded (see `--reseed-every`).
    Canonical,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum FormatArg {
    /// One 81-character line per puzzle (`0` = blank).
    Text,
    /// A JSON array of puzzle records.
    Json,
}

/// Clue-pattern symmetry to enforce while digging (see
/// `sudoku_generator::dig::Symmetry`).
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum SymmetryArg {
    None,
    Central,
    Diagonal,
    AntiDiagonal,
    BiDiagonal,
    Horizontal,
    Vertical,
    Full,
}

#[derive(Debug, Args)]
pub struct GenerateArgs {
    /// Number of puzzles to generate.
    #[arg(long, default_value_t = 1)]
    pub count: u32,

    /// RNG seed for reproducible output. Omit for OS entropy.
    #[arg(long)]
    pub seed: Option<u64>,

    /// How each solved grid is produced.
    #[arg(long, value_enum, default_value_t = FillModeArg::Canonical)]
    pub fill_mode: FillModeArg,

    /// In canonical mode, how many puzzles to generate before rebuilding the
    /// seed grid via randomized backtracking. Ignored in random mode.
    #[arg(long, default_value_t = 50)]
    pub reseed_every: u32,

    /// Clue-pattern symmetry to enforce while digging.
    #[arg(long, value_enum, default_value_t = SymmetryArg::None)]
    pub symmetry: SymmetryArg,

    /// Never remove a clue that would drop the puzzle's clue count below
    /// this. A floor, not an exact target — digging may stop above it if
    /// the random removal order runs out of safely-removable cells first.
    #[arg(long, default_value_t = 0)]
    pub min_clues: u8,

    /// Keep only puzzles whose difficulty (max technique weight) is at
    /// least this. Conflicts with --train-technique.
    #[arg(long, conflicts_with = "train_technique")]
    pub min_difficulty: Option<u32>,

    /// Keep only puzzles whose difficulty (max technique weight) is at
    /// most this. Conflicts with --train-technique.
    #[arg(long, conflicts_with = "train_technique")]
    pub max_difficulty: Option<u32>,

    /// Training mode: generate puzzles that maximize this technique's
    /// occurrence count, while never requiring anything harder than its
    /// own weight. See the technique hierarchy config (or `sudoku-gen
    /// rate`'s output) for valid ids. Conflicts with
    /// --min-difficulty/--max-difficulty.
    #[arg(long)]
    pub train_technique: Option<String>,

    /// In training mode, only accept puzzles where the target technique
    /// fires at least this many times. Ignored without --train-technique.
    #[arg(long, default_value_t = 1)]
    pub min_technique_count: u32,

    /// Max attempts per puzzle when --min-difficulty, --max-difficulty, or
    /// --train-technique is set. Ignored otherwise — plain generation is a
    /// single fast attempt, with no technique solving at all.
    #[arg(long, default_value_t = 200)]
    pub attempts: u32,

    /// Path to a custom technique hierarchy TOML file, used when grading
    /// difficulty or training. Defaults to the bundled hierarchy.
    #[arg(long)]
    pub technique_config: Option<PathBuf>,

    /// Output format.
    #[arg(long, value_enum, default_value_t = FormatArg::Text)]
    pub format: FormatArg,

    /// Output file path. Defaults to stdout.
    #[arg(long)]
    pub output: Option<PathBuf>,

    /// Pretty-print JSON output (ignored for text format).
    #[arg(long)]
    pub pretty: bool,
}

#[derive(Debug, Args)]
pub struct RateArgs {
    /// The puzzle to rate, as an 81-character line (`0` or `.` = blank).
    pub puzzle: String,

    /// Path to a custom technique hierarchy TOML file. Defaults to the
    /// bundled hierarchy.
    #[arg(long)]
    pub technique_config: Option<PathBuf>,

    /// Print the full step-by-step technique application trace (which
    /// technique fired, in what order, and exactly what it did) — for
    /// debugging a difficulty rating that looks wrong, by checking each
    /// claimed deduction individually.
    #[arg(long)]
    pub steps: bool,
}
