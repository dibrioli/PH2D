//! **Is the safe single-allocation path real?** ADR-0117's D2 rests on this, so it is measured,
//! not assumed.
//!
//! ```text
//! cargo test -p ph2d-audio-edit --release --test measure_arc_build -- --nocapture
//! ```
//!
//! `Arc::from(Vec<T>)` cannot reuse the Vec's buffer — an `Arc` stores its refcount inline, right
//! before the data, and a Vec's allocation has no room for one. So it allocates a second buffer
//! and memcpy's: **2 blocks**.
//!
//! `Arc<[T]>: FromIterator<T>` specializes on `TrustedLen`: given an iterator whose length is
//! known exactly and trusted, it allocates the `ArcInner` once and writes the samples straight
//! into it. `Map<Range<usize>, F>` is `TrustedLen`. If that specialization is real, this is
//! **1 block** — and both crates keep `#![forbid(unsafe_code)]`.
//!
//! The specialization is an implementation detail of the standard library, which is exactly why
//! it gets a measurement instead of a comment.

use std::sync::Arc;

#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

const N: usize = 4_000_000; // 16 MB of f32

#[test]
fn the_trusted_len_collect_allocates_once() {
    // Route A — the one every op in `ops.rs` takes today.
    let profiler = dhat::Profiler::builder().testing().build();
    let v: Vec<f32> = (0..N).map(|i| i as f32).collect();
    let a: Arc<[f32]> = v.into();
    let via_vec = dhat::HeapStats::get();
    drop(profiler);
    std::hint::black_box(&a);
    drop(a);

    // Route B — build the Arc directly from a TrustedLen iterator.
    let profiler = dhat::Profiler::builder().testing().build();
    let b: Arc<[f32]> = (0..N).map(|i| i as f32).collect();
    let direct = dhat::HeapStats::get();
    drop(profiler);
    std::hint::black_box(&b);

    println!("\n=== ADR-0117 D2: building an Arc<[f32]> of {N} samples ===");
    println!(
        "Vec -> Arc::from :  {} blocks, {:.1} MB allocated, peak {:.1} MB",
        via_vec.total_blocks,
        via_vec.total_bytes as f64 / 1_048_576.0,
        via_vec.max_bytes as f64 / 1_048_576.0
    );
    println!(
        "collect::<Arc<_>> : {} blocks, {:.1} MB allocated, peak {:.1} MB",
        direct.total_blocks,
        direct.total_bytes as f64 / 1_048_576.0,
        direct.max_bytes as f64 / 1_048_576.0
    );
    println!();

    assert_eq!(
        via_vec.total_blocks, 2,
        "the Vec route should allocate the Vec AND the Arc"
    );
    assert_eq!(
        direct.total_blocks, 1,
        "TrustedLen collect must allocate the Arc ONCE — if this is 2, the std specialization is \
         not what ADR-0117 D2 assumes and the whole decision needs rethinking"
    );
    assert!(
        direct.max_bytes < via_vec.max_bytes,
        "the direct route must peak lower — that is the entire point"
    );
}
