# Roadmap

## Done

The first vertical slice: a genuinely runnable pipeline from "nothing" to "verified, uniquely-solvable puzzles out of the CLI."

- [x] Workspace scaffolding (`sudoku-core`, `sudoku-solver`, `sudoku-generator`, `sudoku-cli`).
- [x] `sudoku-core`: board representation, parsing/export (single-line and array formats), row/col/box bitmask bookkeeping, band/stack/digit/transposition symmetry transforms.
- [x] `sudoku_solver::exact`: bitwise backtracking uniqueness solver (MRV cell selection, contradiction short-circuit, naked-single fast path, early exit on a 2nd solution).
- [x] `sudoku_solver::technique::hierarchy`: reconfigurable technique hierarchy loaded from TOML.
- [x] `sudoku-generator`: full-grid generation (randomized backtracking, and a canonical-seed + random-symmetry-transform hybrid with periodic reseeding), and a pluggable `PuzzleConstructionStrategy` trait with a `TopDown` implementation.
- [x] `sudoku-cli`: `sudoku-gen generate` — bulk generation with a seedable RNG, text and JSON export.
- [x] Unit test coverage for all of the above (exact-solver fixtures, generation/digging invariants, hierarchy config validation).

**Technique detection** — `sudoku_solver::technique::grid::CandidateGrid` (a persistent per-cell pencil-mark grid, since the exact solver's `UnitMasks` can't represent technique-driven eliminations), a `Technique` trait, and `TechniqueSolver` that solves a puzzle by applying the easiest applicable technique repeatedly (driven by the hierarchy's weight order), producing a difficulty rating and a per-technique occurrence trace. Ported from an earlier Java reference implementation (`fr.fluume.sudoku`), with several corrections found and fixed along the way rather than translated as-is:
- The Java solver's `rebuildPotentialValues()` does a full reset-and-reapply-from-placements pass after *every* hint, including elimination-only ones — silently discarding indirect techniques' eliminations and risking infinite loops on any puzzle that needs one to progress. Fixed by only ever propagating placements incrementally (`CandidateGrid::place`), never resetting.
- Naked Pair/Triple's `apply()` in Java are no-ops (detection only, no elimination, no cell coordinates recorded) — reimplemented with real elimination logic.
- Swordfish/Jellyfish/Skyscraper/2-String Kite loop candidate values `0..8` in Java instead of `1..=9` (never checks digit 9) — fixed.
- Hidden Pair/Triple/Quad's shared union-of-positions check doesn't guard against a value with zero remaining candidates (e.g. already placed elsewhere in the unit) pairing up with a genuinely-confined value and producing a bogus, over-eliminating "hidden pair" — guarded against here.
- "Locked Candidates" in Java only implements Pointing (box→line); Claiming (line→box) didn't exist and was written fresh as the mirror algorithm. Both share one hierarchy entry (`pointing_claiming`), per the agreed hierarchy.
- [x] 17 techniques implemented and unit-tested: Last Digit, Hidden Single (block/row/col), Naked Single, Pointing, Claiming, Naked Pair/Triple/Quad, Hidden Pair/Triple/Quad, X-Wing, Swordfish, Jellyfish, Skyscraper, 2-String Kite, XY-Wing, XYZ-Wing.
  - Last Digit isn't from the Java reference — added on top of it as the weakest technique in the hierarchy (below Hidden Single), for absolute-beginner puzzles: a row/column/block with exactly one empty cell, solved by counting alone (no candidates needed).
- [x] `sudoku-gen rate <puzzle>` CLI command: solves a puzzle, reports difficulty and technique usage.

**Training mode, clue-pattern symmetry, and difficulty-targeted generation** — all built on the technique solver and exposed as `sudoku-gen generate` flags:
- [x] `sudoku_generator::dig::Symmetry` (`None`/`Central`/`Diagonal`/`AntiDiagonal`/`BiDiagonal`/`Horizontal`/`Vertical`/`Full`): which cells must be removed together while digging, so the finished puzzle's clue pattern respects the chosen symmetry. Orbits are computed generically as the closure of a cell under a small set of generating coordinate transforms (e.g. `Full` = closure of {diagonal, horizontal} already yields the complete 8-element dihedral group), rather than hand-enumerating each of the 8 cases. `TopDown` now removes/restores a whole symmetry orbit at a time instead of one cell — `Symmetry::None` (the default) makes every orbit a single cell, identical to the previous behavior.
- [x] `sudoku_generator::grade`: `generate_graded_puzzle` (one generate-dig-rate cycle) and `generate_matching` (retries up to N attempts, keeps the highest-scoring accepted candidate) — one shared retry engine powering both difficulty-range filtering and training mode.
- [x] `sudoku-gen generate --symmetry <...>`, `--min-difficulty`/`--max-difficulty <N>`, `--train-technique <id>` (mutually exclusive with the difficulty range), `--attempts <N>`, `--technique-config <path>`. Plain generation (none of these set) is unaffected — still a single fast attempt with no technique solving. JSON output gains `difficulty`/`technique_counts` fields only when grading is active.

**Minimum clue count, parallel batch generation, and CLI progress:**
- [x] `TopDown::with_min_clues`: never removes an orbit that would drop the clue count below a floor — checked exactly (orbits are disjoint and each processed once, so `clue_count() - orbit.len()` is always precise), skipping the orbit entirely rather than remove-then-restore. `sudoku-gen generate --min-clues <N>`.
- [x] `sudoku_generator::parallel_batches`: splits a batch across `rayon::current_num_threads()` chunks, each with its own independently-seeded, reused `FullGridGenerator`/`TopDown` — so `--reseed-every`'s efficiency benefit survives *within* a chunk, unlike naively parallelizing one puzzle per task. Chunk seeds are derived sequentially up front from the base seed, so output doesn't depend on scheduling order. Reproducible given the same seed **and** the same machine (thread count) — not a bit-for-bit match for a purely sequential run, nor portable across machines with a different core count. Required adding `Technique: Send + Sync` (trivially satisfied — every technique is a stateless unit struct) so `TechniqueSolver` can be shared read-only across chunk threads.
- [x] Live progress bar (`indicatif`) on `sudoku-gen generate`, one tick per completed puzzle across all chunks. Found and fixed a real bug along the way: `ProgressBar::println` silently no-ops when stderr isn't an interactive terminal (piped, redirected, CI) — the "no puzzle found" warnings were being dropped entirely in that case. Fixed by falling back to plain `eprintln!` when `progress.is_hidden()`.

## Next

Roughly in dependency order — each of these builds on what's already in place.

1. **The 5 deferred techniques**: Finned X-Wing, Multi-Coloring, Simple Coloring, Unique Rectangle, W-Wing. These were empty stubs (zero detection logic) in the Java reference, so unlike the 16 above they need to be designed from general Sudoku technique knowledge rather than ported.
2. **Berthier-style controlled-bias / bottom-up strategies.** Additional `PuzzleConstructionStrategy` implementations, now that the technique solver exists to drive bias-aware construction.
3. **Parquet export**, for large batches (millions of grids) without loading everything into memory at once.
4. **PyO3 Python bindings** (`sudoku-py`), exposing the generator/solver to Python for pandas/matplotlib-based analysis, built via `maturin`.
5. **Benchmarks and fuzzing**: `criterion` benchmarks per technique and for the exact solver's hot loop; `proptest` fuzzing for combinatorial correctness (every generated puzzle has a unique solution, every solved grid is valid, every symmetric puzzle's clue pattern actually satisfies its symmetry, etc).
6. **Persistent exact-solver state**: the exact solver currently rebuilds its search state from a `Board` on every call; profiling under real generation load will show whether an incrementally-updated, persistent solver instance is worth the added complexity during digging.
7. **`--jobs` flag**: `parallel_batches` currently always uses rayon's default global thread pool (all logical CPUs) — explicit thread-count control wasn't requested yet, but would be a cheap follow-up (build a custom `rayon::ThreadPool`).
8. **crates.io publishing**: package metadata, versioning policy, and public API stability review for the crates intended for community reuse.

## Explicitly not planned right now

- A WASM build — see [`ABOUT.md`](./ABOUT.md#why-not-wasm-for-now).
