# sudoku-arena-generator

A fast, standalone Sudoku puzzle generator, written in Rust. Built for [Sudoku Arena](https://github.com/Fluume/sudoku-arena-generator), and open-sourced for the wider Sudoku community.

See [`ABOUT.md`](./ABOUT.md) for why this project exists and why it's built the way it is, and [`ROADMAP.md`](./ROADMAP.md) for what's built so far versus what's planned.

This README is a step-by-step guide, written for people with little or no Rust experience.

## 1. Install Rust

This project needs the Rust toolchain (the `cargo` build tool and the `rustc` compiler). If you don't have it yet:

1. Go to <https://rustup.rs> and follow the instructions for your OS (on Windows, this downloads `rustup-init.exe`; on macOS/Linux, it's a one-line shell command).
2. Once installed, open a **new** terminal and check it worked:

   ```sh
   rustc --version
   cargo --version
   ```

   Both should print a version number. If they don't, close and reopen your terminal (the installer updates your `PATH`, which existing terminal windows won't pick up).

You don't need to know Rust to use this project — the commands below are all you need.

## 2. Get the code

```sh
git clone https://github.com/Fluume/sudoku-arena-generator.git
cd sudoku-arena-generator
```

## 3. Build it

```sh
cargo build --release
```

The first build will download and compile several dependencies, so it can take a minute or two. Later builds are much faster (Cargo caches everything).

This project is a **workspace**: one repository containing several small, focused packages ("crates") that depend on each other. You don't need to build them individually — `cargo build` at the repo root builds all of them.

## 4. Generate some puzzles

The CLI binary is called `sudoku-gen`. You can run it in two ways:

**Without a separate build step** (Cargo builds it for you if needed):

```sh
cargo run -p sudoku-cli -- generate --count 5
```

**Using the already-built binary** (faster if you've already run `cargo build --release`):

```sh
# Windows
.\target\release\sudoku-gen.exe generate --count 5

# macOS/Linux
./target/release/sudoku-gen generate --count 5
```

Everything after `--` (or after `sudoku-gen` if you're using the built binary directly) is passed to the CLI itself — see [CLI usage](#cli-usage) below for all available options.

## CLI usage

### `sudoku-gen generate`

Generates a batch of puzzles and prints them (or writes them to a file). The batch is generated in parallel (a live progress bar shows on stderr) — `--count` is split into chunks, one per available CPU core, each with its own RNG stream derived from `--seed`. This means output is reproducible given the same seed **and** the same machine (same number of cores) — it won't match a single-threaded run bit-for-bit, and isn't portable across machines with a different core count.

| Flag | Default | Description |
|---|---|---|
| `--count <N>` | `1` | Number of puzzles to generate. |
| `--seed <N>` | random | RNG seed. Set this to get the exact same puzzles on every run — useful for reproducible tests or debugging. |
| `--fill-mode <random\|canonical>` | `canonical` | How each solved grid is produced. `canonical` reuses one seed grid and applies random symmetry transforms (much faster); `random` runs a fresh randomized backtracking search every time. |
| `--reseed-every <N>` | `50` | In `canonical` mode, how many puzzles to generate before rebuilding the seed grid from scratch. Ignored in `random` mode. |
| `--symmetry <see below>` | `none` | Clue-pattern symmetry to enforce while digging (which cells are removed together). One of `none`, `central`, `diagonal`, `anti-diagonal`, `bi-diagonal`, `horizontal`, `vertical`, `full`. `central` (180° rotation) is the most common convention in published Sudoku; `full` enforces all 8 symmetries of the square and is the most restrictive (fewest clues removable). |
| `--min-clues <N>` | `0` | Never remove a clue that would drop the puzzle below this many clues. A floor, not an exact target — digging may stop above it if it runs out of safely-removable cells first. |
| `--min-difficulty <N>` / `--max-difficulty <N>` | unset | Keep only puzzles whose difficulty (the weight of the hardest technique required — see `sudoku-gen rate`) falls in this range. Conflicts with `--train-technique`. |
| `--train-technique <id>` | unset | Training mode: generate puzzles that maximize this technique's occurrence count, while never requiring anything harder than its own weight. Conflicts with `--min-difficulty`/`--max-difficulty`. |
| `--min-technique-count <N>` | `1` | In training mode, only accept puzzles where `--train-technique`'s technique fires at least this many times. Ignored without `--train-technique`. |
| `--attempts <N>` | `200` | Max retries per puzzle when `--min-difficulty`, `--max-difficulty`, or `--train-technique` is set. If no candidate satisfies the constraint within this budget, that puzzle is skipped with a warning on stderr — the batch continues, so the final count may be less than `--count`. Ignored otherwise (plain generation is always a single fast attempt). |
| `--technique-config <path>` | bundled default | Same as `rate`'s flag — which hierarchy to grade against, when grading is active. |
| `--format <text\|json>` | `text` | Output format (see below). |
| `--output <path>` | stdout | Write to a file instead of printing to the terminal. |
| `--pretty` | off | Pretty-print JSON output. |

**Text format** — one puzzle per line, 81 characters, `0` for blank cells:

```sh
cargo run -p sudoku-cli -- generate --count 3 --seed 1
```

```
018009000200000006090002000500060000000000410030400082940205007000000900060300024
400000003001000086900005100602001000050084000013200000086030051000000400000070000
201000000007085300000000095106200000000003008000047063520000000000000032003400070
```

This is a widely used Sudoku exchange format, so these lines can be pasted straight into most other Sudoku tools.

**JSON format** — an array with each puzzle's clue grid, full solution, and clue count, handy for feeding into analysis scripts:

```sh
cargo run -p sudoku-cli -- generate --count 2 --seed 42 --format json --pretty
```

```json
[
  {
    "puzzle": "090700105000000080040000000009000300200000460700080009000030002980100600130850900",
    "solution": "896742135521693784347518296419265378258379461763481529675934812984127653132856947",
    "clue_count": 25
  }
]
```

**Writing to a file:**

```sh
cargo run -p sudoku-cli -- generate --count 100000 --format json --output puzzles.json
```

**Symmetric puzzles** — clue positions respect the chosen symmetry:

```sh
cargo run -p sudoku-cli -- generate --count 1 --symmetry central --seed 1
```

**Training mode** — puzzles that lean heavily on one technique. JSON output gains `difficulty` and `technique_counts` fields whenever grading is active (plain generation omits them, so its JSON shape is unchanged):

```sh
cargo run -p sudoku-cli -- generate --count 1 --train-technique two_string_kite --format json --pretty
```

```json
[
  {
    "puzzle": "500007100000040037000050640070305000002180000860000000420000010001206000008000000",
    "solution": "543627198286941537917853642174365829352189764869472351425738916731296485698514273",
    "clue_count": 24,
    "difficulty": 720,
    "technique_counts": [["hidden_single_block", 47], ["hidden_single_line", 6], ["pointing_claiming", 6], ["naked_triple", 1], ["two_string_kite", 5]]
  }
]
```

**Minimum occurrence count for the trained technique** — reject any puzzle where `two_string_kite` fires fewer than 8 times:

```sh
cargo run -p sudoku-cli -- generate --count 1 --train-technique two_string_kite --min-technique-count 8 --attempts 500
```

**Difficulty-targeted generation:**

```sh
cargo run -p sudoku-cli -- generate --count 5 --min-difficulty 400 --max-difficulty 700
```

**Minimum clue count** — never dig below 30 clues:

```sh
cargo run -p sudoku-cli -- generate --count 5 --min-clues 30
```

### `sudoku-gen rate`

Rates a single puzzle using the technique-based solver: solves it with the easiest applicable technique at each step, and reports whether it fully solved, the difficulty (the weight of the hardest technique needed), and how many times each technique fired.

```sh
cargo run -p sudoku-cli -- rate "530070000600195000098000060800060003400803001700020006060000280000419005000080079"
```

```
solved: yes
difficulty (max technique weight): 100
techniques used:
  Hidden Single (block) (hidden_single_block): 51
```

| Flag | Default | Description |
|---|---|---|
| `--technique-config <path>` | bundled default | Load a custom technique hierarchy TOML file instead of the built-in one (see `crates/sudoku-solver/src/technique/techniques.default.toml` for the format — every technique's category and weight can be freely redefined, without recompiling). |
| `--steps` | off | Print the full step-by-step trace: every technique application, in order, with exactly what it did (which cell/digit) — for debugging a difficulty rating that looks wrong, by checking each claimed deduction individually. |

```sh
cargo run -p sudoku-cli -- rate --steps "530070000600195000098000060800060003400803001700020006060000280000419005000080079"
```

```
...
steps:
    1. [100] Hidden Single (block) (hidden_single_block): place 5 at r1c1
    2. [100] Hidden Single (block) (hidden_single_block): place 7 at r2c8
   ...
```

## Project layout

```
crates/
├── sudoku-core/       board representation, bitmask bookkeeping, symmetry transforms
├── sudoku-solver/     exact (uniqueness) solver + technique-based solver (17 techniques) + hierarchy config
├── sudoku-generator/  full-grid generation + pluggable puzzle construction strategies
└── sudoku-cli/        the `sudoku-gen` command-line tool
```

Each crate has a single, narrow responsibility — see `ABOUT.md` for why.

## Running the tests

```sh
cargo test --workspace
```

## Python bindings

Not built yet — planned as a later milestone (see `ROADMAP.md`) via [PyO3](https://pyo3.rs), so the generator can be driven from Python for large-scale analysis (pandas, matplotlib) without losing Rust's performance.

## License

Dual-licensed under [MIT](./LICENSE-MIT) or [Apache-2.0](./LICENSE-APACHE), at your option.
