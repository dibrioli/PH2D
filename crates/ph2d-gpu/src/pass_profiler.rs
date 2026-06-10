//! GPU-timestamp pass profiler (`PH2D_FLUID_PROFILE=1`) — measures where the
//! GPU EXECUTION time goes, per pass, per frame.
//!
//! The CPU-side `[fluid]`/`[frame]` profilers proved the watercolor FPS
//! collapse is GPU-execution-bound (CPU encode ≈ 2.8 ms while `acquire_frame`
//! stalls ~131 ms on GPU backpressure) — but CPU timers around `queue.submit`
//! only see the submit, not the execution. This module owns one
//! [`wgpu::QuerySet`] of TIMESTAMP queries; instrumented passes across the
//! workspace attach begin/end writes via [`compute_writes`]/[`render_writes`],
//! copies and foreign submits (Vello) are bracketed with empty marker passes
//! ([`copy_span_begin`]/[`span_begin`]), and once per frame the shell calls
//! [`end_frame`] to resolve + read back (pipelined, no blocking poll) and
//! print per-label averages every `WINDOW` frames.
//!
//! Always compiled, inert unless BOTH hold:
//! 1. the shell called [`init`] (it gates on the `PH2D_FLUID_PROFILE` env var),
//! 2. the device has [`wgpu::Features::TIMESTAMP_QUERY`] (requested in
//!    `context.rs` when the adapter advertises it; on Apple Silicon timestamps
//!    are stage-boundary only, so per-PASS writes are exactly what Metal
//!    supports — encoder-level `write_timestamp` is NOT available there).
//!
//! Every public fn is a no-op returning `None`/`()` when uninitialized, so
//! instrumentation sites cost one `OnceLock::get` + branch in production.

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

/// Timestamp queries per frame (begin+end pairs ⇒ ≤ 256 scopes). The fluid
/// step alone is ~40 passes/frame; 512 leaves headroom for the layer-compositor
/// chain + markers without approaching Metal's counter-sample-buffer limits.
const CAPACITY: u32 = 512;
/// Pipelined readback ring depth (same idea as the fluid stats readback).
const RING: usize = 3;
/// Frames per printed report.
const WINDOW: u32 = 120;

struct Prof {
    query_set: wgpu::QuerySet,
    resolve_buf: wgpu::Buffer,
    /// ns per timestamp tick (`Queue::get_timestamp_period`).
    period: f32,
    inner: Mutex<Inner>,
}

/// One scope = one (label, begin-query, end-query) triple. `end` is `None`
/// for an opened-but-never-closed span (dropped at frame end; its begin
/// query was still written, which keeps the resolve range fully written).
type Scope = (&'static str, u32, Option<u32>);

struct ReadSlot {
    staging: wgpu::Buffer,
    ready: Arc<AtomicBool>,
    in_flight: bool,
    /// The scope table snapshotted at submit time.
    scopes: Vec<Scope>,
}

#[derive(Default)]
struct Accum {
    /// label → (sum_ns, sample_count). BTreeMap: tiny key set + deterministic
    /// iteration (ADR-0022 bans std HashMap workspace-wide).
    per_label: BTreeMap<&'static str, (u64, u32)>,
    /// whole-frame GPU span (max end − min begin) + pass count.
    span_ns: u64,
    passes: u64,
    frames: u32,
    dropped: u32,
}

struct Inner {
    next_idx: u32,
    scopes: Vec<Scope>,
    ring: Vec<ReadSlot>,
    accum: Accum,
}

static PROF: OnceLock<Prof> = OnceLock::new();

/// Build the profiler if `PH2D_FLUID_PROFILE` is on AND the device was created
/// with `TIMESTAMP_QUERY`. Call once from the shell right after device
/// creation. Safe to call when either gate fails (stays inert; prints why
/// when the env var is set but the feature is missing).
pub fn init(device: &wgpu::Device, queue: &wgpu::Queue) {
    if !std::env::var("PH2D_FLUID_PROFILE").is_ok_and(|v| v != "0") {
        return;
    }
    init_forced(device, queue);
}

/// [`init`] without the env-var gate — the GPU round-trip gate test needs to
/// turn the profiler on without `set_var` (unsafe in edition 2024, and this
/// crate is `forbid(unsafe_code)`). Not for production callers.
#[doc(hidden)]
pub fn init_forced(device: &wgpu::Device, queue: &wgpu::Queue) {
    if !device.features().contains(wgpu::Features::TIMESTAMP_QUERY) {
        eprintln!(
            "[gpu] PH2D_FLUID_PROFILE set but the device lacks TIMESTAMP_QUERY — \
             GPU pass timings unavailable on this adapter"
        );
        return;
    }
    let query_set = device.create_query_set(&wgpu::QuerySetDescriptor {
        label: Some("ph2d gpu pass profiler timestamps"),
        ty: wgpu::QueryType::Timestamp,
        count: CAPACITY,
    });
    let resolve_buf = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("ph2d gpu pass profiler resolve"),
        size: u64::from(CAPACITY) * 8,
        usage: wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let ring = (0..RING)
        .map(|_| ReadSlot {
            staging: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("ph2d gpu pass profiler staging"),
                size: u64::from(CAPACITY) * 8,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                mapped_at_creation: false,
            }),
            ready: Arc::new(AtomicBool::new(false)),
            in_flight: false,
            scopes: Vec::new(),
        })
        .collect();
    let period = queue.get_timestamp_period();
    let _ = PROF.set(Prof {
        query_set,
        resolve_buf,
        period,
        inner: Mutex::new(Inner {
            next_idx: 0,
            scopes: Vec::new(),
            ring,
            accum: Accum::default(),
        }),
    });
    eprintln!("[gpu] pass profiler ON (timestamp period {period:.3} ns/tick)");
}

fn alloc_pair(label: &'static str) -> Option<(&'static Prof, u32, u32)> {
    let prof = PROF.get()?;
    let mut inner = prof.inner.lock().ok()?;
    if inner.next_idx + 2 > CAPACITY {
        inner.accum.dropped += 1;
        return None;
    }
    let b = inner.next_idx;
    inner.next_idx += 2;
    inner.scopes.push((label, b, Some(b + 1)));
    Some((prof, b, b + 1))
}

/// Timestamp writes for a compute pass. Attach to the pass descriptor:
/// `timestamp_writes: ph2d_gpu::pass_profiler::compute_writes("fluid.diffuse")`.
/// `None` (free) when profiling is off.
#[must_use]
pub fn compute_writes(label: &'static str) -> Option<wgpu::ComputePassTimestampWrites<'static>> {
    let (prof, b, e) = alloc_pair(label)?;
    Some(wgpu::ComputePassTimestampWrites {
        query_set: &prof.query_set,
        beginning_of_pass_write_index: Some(b),
        end_of_pass_write_index: Some(e),
    })
}

/// Timestamp writes for a render pass (sprite/tonemap/compositor).
#[must_use]
pub fn render_writes(label: &'static str) -> Option<wgpu::RenderPassTimestampWrites<'static>> {
    let (prof, b, e) = alloc_pair(label)?;
    Some(wgpu::RenderPassTimestampWrites {
        query_set: &prof.query_set,
        beginning_of_pass_write_index: Some(b),
        end_of_pass_write_index: Some(e),
    })
}

/// Token pairing a span begin with its end (the end query index + the scope
/// slot to complete).
pub struct SpanToken {
    scope_idx: usize,
    end_idx: u32,
}

fn alloc_span(label: &'static str) -> Option<(&'static Prof, u32, SpanToken)> {
    let prof = PROF.get()?;
    let mut inner = prof.inner.lock().ok()?;
    if inner.next_idx + 2 > CAPACITY {
        inner.accum.dropped += 1;
        return None;
    }
    let b = inner.next_idx;
    inner.next_idx += 2;
    let scope_idx = inner.scopes.len();
    // `end` stays None until `*_span_end` proves the end marker was recorded.
    inner.scopes.push((label, b, None));
    Some((
        prof,
        b,
        SpanToken {
            scope_idx,
            end_idx: b + 1,
        },
    ))
}

fn empty_marker_pass(
    enc: &mut wgpu::CommandEncoder,
    prof: &'static Prof,
    begin: Option<u32>,
    end: Option<u32>,
) {
    // Apple Silicon samples timestamps at stage (encoder) boundaries only, so
    // an EMPTY compute pass is the portable "write a timestamp here" idiom —
    // wgpu's encoder-level `write_timestamp` needs a feature Metal lacks.
    let _pass = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
        label: Some("ph2d gpu profiler marker"),
        timestamp_writes: Some(wgpu::ComputePassTimestampWrites {
            query_set: &prof.query_set,
            beginning_of_pass_write_index: begin,
            end_of_pass_write_index: end,
        }),
    });
}

/// Open a span INSIDE an existing encoder (brackets copies — e.g. the
/// preview-slot `copy_texture_to_texture`). Pair with [`copy_span_end`] on the
/// same encoder, after the copies.
#[must_use]
pub fn copy_span_begin(enc: &mut wgpu::CommandEncoder, label: &'static str) -> Option<SpanToken> {
    let (prof, b, token) = alloc_span(label)?;
    empty_marker_pass(enc, prof, Some(b), None);
    Some(token)
}

/// Close a [`copy_span_begin`] span.
pub fn copy_span_end(enc: &mut wgpu::CommandEncoder, token: SpanToken) {
    let Some(prof) = PROF.get() else { return };
    empty_marker_pass(enc, prof, None, Some(token.end_idx));
    if let Ok(mut inner) = prof.inner.lock()
        && let Some(s) = inner.scopes.get_mut(token.scope_idx)
    {
        s.2 = Some(token.end_idx);
    }
}

/// Open a span as its OWN submit — brackets work that submits internally on
/// the same queue (Vello). GPU work executes in queue-submission order, so
/// `end − begin` covers everything submitted in between.
#[must_use]
pub fn span_begin(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    label: &'static str,
) -> Option<SpanToken> {
    let (prof, b, token) = alloc_span(label)?;
    let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("ph2d gpu profiler span begin"),
    });
    empty_marker_pass(&mut enc, prof, Some(b), None);
    queue.submit([enc.finish()]);
    Some(token)
}

/// Close a [`span_begin`] span (its own submit).
pub fn span_end(device: &wgpu::Device, queue: &wgpu::Queue, token: SpanToken) {
    let Some(prof) = PROF.get() else { return };
    let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("ph2d gpu profiler span end"),
    });
    empty_marker_pass(&mut enc, prof, None, Some(token.end_idx));
    queue.submit([enc.finish()]);
    if let Ok(mut inner) = prof.inner.lock()
        && let Some(s) = inner.scopes.get_mut(token.scope_idx)
    {
        s.2 = Some(token.end_idx);
    }
}

/// Per-frame tail: resolve this frame's queries into a free ring slot
/// (async map, no blocking poll), drain any COMPLETED prior slot into the
/// accumulator, and print the per-label table every [`WINDOW`] sampled
/// frames. Call once per frame from the shell, after the last instrumented
/// submit. No-op when off.
pub fn end_frame(device: &wgpu::Device, queue: &wgpu::Queue) {
    let Some(prof) = PROF.get() else { return };
    let Ok(mut inner) = prof.inner.lock() else {
        return;
    };
    let inner = &mut *inner;

    // 1. Drain completed readbacks (the map callback fired on some earlier
    //    poll — `acquire_frame`/present polls the device every frame).
    let _ = device.poll(wgpu::PollType::Poll);
    for slot in &mut inner.ring {
        if !slot.in_flight || !slot.ready.load(Ordering::Acquire) {
            continue;
        }
        slot.in_flight = false;
        let data = slot.staging.slice(..).get_mapped_range();
        let tick = |i: u32| -> u64 {
            let o = i as usize * 8;
            u64::from_le_bytes(data[o..o + 8].try_into().unwrap_or([0; 8]))
        };
        let mut frame_min = u64::MAX;
        let mut frame_max = 0u64;
        let mut passes = 0u64;
        for &(label, b, e) in &slot.scopes {
            let Some(e) = e else { continue };
            let (tb, te) = (tick(b), tick(e));
            // Unwritten / out-of-order samples resolve as 0 or inverted —
            // clamp instead of poisoning the average.
            let d = te.saturating_sub(tb);
            let ns = (d as f64 * f64::from(prof.period)) as u64;
            let ent = inner.accum.per_label.entry(label).or_insert((0, 0));
            ent.0 += ns;
            ent.1 += 1;
            if tb != 0 {
                frame_min = frame_min.min(tb);
            }
            frame_max = frame_max.max(te);
            passes += 1;
        }
        drop(data);
        slot.staging.unmap();
        slot.scopes.clear();
        if passes > 0 {
            let span = frame_max.saturating_sub(if frame_min == u64::MAX {
                frame_max
            } else {
                frame_min
            });
            inner.accum.span_ns += (span as f64 * f64::from(prof.period)) as u64;
            inner.accum.passes += passes;
            inner.accum.frames += 1;
        }
    }

    // 2. Print the report window.
    if inner.accum.frames >= WINDOW {
        let f = f64::from(inner.accum.frames);
        let mut rows: Vec<(&'static str, f64, f64)> = inner
            .accum
            .per_label
            .iter()
            .map(|(label, &(sum_ns, n))| {
                // Per-FRAME average (a label dispatched k× per frame shows k× its
                // per-pass cost — that's the number that explains the frame time).
                (*label, sum_ns as f64 / f / 1.0e6, f64::from(n) / f)
            })
            .collect();
        rows.sort_by(|a, b| b.1.total_cmp(&a.1));
        let mut line = format!(
            "[gpu] avg over {} frames: gpu-busy(span)={:.2}ms passes/frame={:.0}",
            inner.accum.frames,
            inner.accum.span_ns as f64 / f / 1.0e6,
            inner.accum.passes as f64 / f,
        );
        if inner.accum.dropped > 0 {
            line.push_str(&format!(" dropped-scopes={}", inner.accum.dropped));
        }
        eprintln!("{line}");
        let mut detail = String::from("[gpu]   ");
        for (label, ms, per_frame) in &rows {
            if *ms < 0.005 {
                continue; // noise floor — keep the line readable
            }
            detail.push_str(&format!("{label}={ms:.2}ms(×{per_frame:.1}) "));
        }
        eprintln!("{detail}");
        inner.accum = Accum::default();
    }

    // 3. Resolve + kick the async readback for THIS frame's scopes.
    let used = inner.next_idx;
    if used == 0 {
        return;
    }
    let scopes = std::mem::take(&mut inner.scopes);
    inner.next_idx = 0;
    let Some(slot) = inner.ring.iter_mut().find(|s| !s.in_flight) else {
        return; // ring saturated — skip this frame's sample
    };
    let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("ph2d gpu profiler resolve"),
    });
    enc.resolve_query_set(&prof.query_set, 0..used, &prof.resolve_buf, 0);
    enc.copy_buffer_to_buffer(&prof.resolve_buf, 0, &slot.staging, 0, u64::from(used) * 8);
    queue.submit([enc.finish()]);
    slot.scopes = scopes;
    slot.in_flight = true;
    slot.ready.store(false, Ordering::Release);
    let ready = Arc::clone(&slot.ready);
    slot.staging
        .slice(..)
        .map_async(wgpu::MapMode::Read, move |r| {
            if r.is_ok() {
                ready.store(true, Ordering::Release);
            }
        });
}
