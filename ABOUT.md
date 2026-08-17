# About this project

## The problem

[Sudoku Arena](https://github.com/Fluume/sudoku-arena-generator) needs a steady, large-scale supply of Sudoku puzzles: graded by difficulty, exportable in different formats, and eventually tunable toward training a specific solving technique. Relying on external grid databases doesn't scale for this — they're hard to manipulate, hard to regenerate on demand, and give no control over difficulty grading or technique distribution. This project exists to remove that dependency: an independent, in-house generator we fully control.

Beyond Sudoku Arena itself, we think there's value in publishing this as a serious, reusable, open-source generator for the wider Sudoku community — not just an internal tool.

## What we want

- A **reconfigurable hierarchy** of solving techniques and difficulty levels — not a fixed, opaque set of presets.
- **Bulk generation**, fast enough to produce large batches for analysis, not just one puzzle at a time.
- **Training puzzles** for a specific technique: grids that require many occurrences of one target technique, and never anything harder.
- **Multiple export formats**, for interop with other tools and for large-scale data analysis.
- Bonus: **symmetric** puzzles.

## What we looked at first

**Sudoku Explainer** (Java) is fast and has a strong technique-based solver — genuinely one of the better reference implementations for solving-technique logic. But it's built for *playing*: a mutable board you query step-by-step for the next hint, with undo/redo and a fixed set of difficulty presets. An early attempt to fork it for bulk, parametrized generation ran into exactly that mismatch — the architecture resists being industrialized, because it was never designed to classify puzzles in bulk, only to solve one puzzle interactively. That's the mistake this project is careful not to repeat: the core engine here is a stateless `grid → result` function, not an interactive solving session.

**Denis Berthier's work** is a valuable reference on generator bias: top-down and bottom-up construction (the two classic approaches) both introduce non-trivial statistical bias in the distribution of generated puzzles, and his "controlled-bias" method corrects for it with a tightly constrained sampling protocol. The cost is speed — it's far too slow for on-demand bulk generation (his own reference dataset of ~6 million grids required a large, dedicated computation). For our purposes — producing gradeable, playable puzzles at volume, not publishing unbiased statistics about the space of all minimal Sudoku grids — classic top-down is the right trade-off: puzzle difficulty here is driven by which techniques are required to solve it, not by the raw distribution of clue counts, so top-down's bias doesn't undermine the difficulty grading built on top of it.

**[Corniel/sudoku](https://github.com/Corniel/sudoku)** was reviewed for generator design ideas alongside the above.

## Why Rust

- **No GC pauses.** Bitboard manipulation, constraint propagation, and backtracking are exactly the kind of workload where garbage collection pauses hurt — especially when the goal is generating *many* grids back to back, not solving one grid interactively.
- **Trivial, safe parallelism.** The [`rayon`](https://docs.rs/rayon) crate turns a sequential generation loop into one that saturates every CPU core with a small, mechanical change — and Rust's ownership model rules out data races at compile time. That maps directly onto "generate many grids in parallel."
- **A real open-source ecosystem.** [crates.io](https://crates.io) for distribution, `cargo doc` for generated documentation, [`criterion`](https://docs.rs/criterion) for benchmarking individual solving techniques, [`proptest`](https://docs.rs/proptest) for fuzzing combinatorial correctness. That's the toolkit a project intended for public, technical scrutiny should have.

Rust performance on this kind of workload is competitive with — often better than — well-written Java, without the interactive-first architectural trap that made the Sudoku Explainer fork difficult.

## Architecture

The engine is split into crates with a single responsibility each (see the [README](./README.md#project-layout) for the current list). The split that matters most: **the exact solver and the technique-based solver are independent.** The exact solver's only job is fast solution-uniqueness checking — it's the hot loop used while removing clues from a solved grid, and it must stay narrow and fast. The technique-based solver grades difficulty by simulating human solving techniques, and runs once per finished puzzle, never inside that hot loop. Puzzle construction itself (top-down today; bottom-up and Berthier-style controlled-bias are architecturally possible later) is a pluggable strategy, not a hardcoded algorithm — so the difficulty/technique hierarchy can be extended or re-tuned without touching the rest of the engine.

## Python bindings, via PyO3

Generation needs to be fast and should run in pure Rust; analysis needs to be ergonomic and benefits from Python's data ecosystem (pandas for manipulating results, matplotlib for visualizing difficulty distributions, technique frequency, generation time, etc.). [PyO3](https://pyo3.rs) compiles the Rust engine into a native Python module — the same way NumPy or Pandas embed native code — so the code that actually runs stays the real, compiled Rust engine, just callable from a script or notebook. That keeps "generate fast" and "explore ergonomically" as two separate concerns instead of forcing one language to do both. For very large batches (millions of grids), the plan is to export straight to a file from the Rust CLI (Parquet, for compactness and fast loading) rather than returning everything into Python memory at once.

This is a later milestone — not built yet (see [`ROADMAP.md`](./ROADMAP.md)).

## Why not WASM (for now)

A browser-based (WASM) build was considered and set aside. The goal right now is generation speed and ease of bulk analysis, not an interactive in-browser demo — and WASM adds real cost for no benefit here: HTTP header constraints for multi-threading, and a measurable (roughly 5-30%) performance hit versus native code. The engine's design doesn't close this door — a WASM build remains possible later if a public demo or client-side solver becomes a real need — but it's not a goal for this phase.
