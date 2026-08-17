//! Generic parallel batch-work splitting: divides `count` work items across
//! chunks (sized to the available parallelism), gives each chunk an
//! independently-derived seed, and runs a caller-supplied worker over each
//! chunk in parallel. Knows nothing about puzzles specifically — just "split
//! N work items into seeded chunks and run them in parallel."

use rand::{rngs::StdRng, RngCore, SeedableRng};
use rayon::prelude::*;

/// Splits `count` across up to `rayon::current_num_threads()` chunks,
/// deriving each chunk's seed sequentially up front (from `base_seed`, or
/// OS entropy if `None`) so chunk contents never depend on how rayon
/// schedules the work, then runs `worker` once per chunk in parallel and
/// concatenates results in chunk order.
///
/// Reproducible given the same seed *and* the same degree of parallelism
/// (thread count) — chunk boundaries depend on `rayon::current_num_threads()`,
/// so this doesn't reproduce a purely sequential run's exact output, nor is
/// it portable across machines with a different core count.
pub fn parallel_batches<T, F>(count: u32, base_seed: Option<u64>, worker: F) -> Vec<T>
where
    T: Send,
    F: Fn(u32, u64) -> Vec<T> + Sync,
{
    if count == 0 {
        return Vec::new();
    }

    let num_chunks = rayon::current_num_threads().min(count as usize).max(1);

    let mut seed_rng = match base_seed {
        Some(seed) => StdRng::seed_from_u64(seed),
        None => StdRng::from_entropy(),
    };
    let chunk_seeds: Vec<u64> = (0..num_chunks).map(|_| seed_rng.next_u64()).collect();

    let base = count as usize / num_chunks;
    let extra = count as usize % num_chunks;
    let chunk_counts: Vec<u32> = (0..num_chunks)
        .map(|i| (base + if i < extra { 1 } else { 0 }) as u32)
        .collect();

    chunk_counts
        .into_par_iter()
        .zip(chunk_seeds.into_par_iter())
        .flat_map_iter(|(chunk_count, chunk_seed)| worker(chunk_count, chunk_seed))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn total_output_length_matches_count() {
        for count in [0u32, 1, 2, 5, 17, 100] {
            let result = parallel_batches(count, Some(1), |chunk_count, _seed| {
                (0..chunk_count).collect::<Vec<u32>>()
            });
            assert_eq!(result.len(), count as usize, "count={count}");
        }
    }

    #[test]
    fn same_seed_reproduces_identical_output() {
        let a = parallel_batches(50, Some(42), |chunk_count, chunk_seed| {
            vec![(chunk_count, chunk_seed)]
        });
        let b = parallel_batches(50, Some(42), |chunk_count, chunk_seed| {
            vec![(chunk_count, chunk_seed)]
        });
        assert_eq!(a, b);
    }

    #[test]
    fn different_seeds_produce_different_chunk_seeds() {
        let a = parallel_batches(50, Some(1), |_chunk_count, chunk_seed| vec![chunk_seed]);
        let b = parallel_batches(50, Some(2), |_chunk_count, chunk_seed| vec![chunk_seed]);
        assert_ne!(a, b);
    }

    #[test]
    fn zero_count_returns_empty_without_panicking() {
        let result: Vec<u32> =
            parallel_batches(0, Some(1), |chunk_count, _| (0..chunk_count).collect());
        assert!(result.is_empty());
    }
}
