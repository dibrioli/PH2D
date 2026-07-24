//! `PH2D_PAINT_PERF` aggregation — ONE summary line per window, not one per frame.
//!
//! The per-frame log drowned the terminal. Instead, `painter_bridge::dispatch` records this frame's
//! dispatch cost + producer + flags via [`record_dispatch`], and `run_render_frame`'s frame timer
//! reports the whole-frame cost via [`end_frame`]; the two are paired here and a compact summary
//! (median + max of both, producer mix, the flags) is printed once every [`WINDOW`] painter frames.
//! So painting a scenario for ~2 s yields one or two lines to paste — enough to locate the cost.

use std::cell::RefCell;

/// ~1.5 s at 60 fps: stable medians, few enough lines to paste.
const WINDOW: usize = 90;

#[derive(Clone, Copy, Default)]
pub(super) struct FrameInfo {
    pub gpu: bool,
    pub dispatch_ms: f32,
    pub w: u32,
    pub h: u32,
    pub gray: bool,
    pub active_is_mask: bool,
    pub lane_partial: bool,
    pub trivial: bool,
}

#[derive(Default)]
struct Agg {
    /// This frame's dispatch info, set by `record_dispatch`, consumed by `end_frame`. `None` on a
    /// frame the painter was not active — that frame is not counted.
    cur: Option<FrameInfo>,
    samples: Vec<(f32, f32)>, // (dispatch_ms, total_frame_ms)
    gpu: u32,
    cpu: u32,
    last: FrameInfo,
}

thread_local! {
    static AGG: RefCell<Agg> = RefCell::new(Agg::default());
    /// Cached env probe (-1 = unknown, 0 = off, 1 = on) — avoid a syscall per frame.
    static ON: std::cell::Cell<i8> = const { std::cell::Cell::new(-1) };
}

/// Whether `PH2D_PAINT_PERF` is set (cached).
pub(super) fn on() -> bool {
    ON.with(|c| {
        if c.get() < 0 {
            c.set(i8::from(std::env::var_os("PH2D_PAINT_PERF").is_some()));
        }
        c.get() > 0
    })
}

/// Record this frame's painter dispatch. Call only when the painter is active.
pub(super) fn record_dispatch(info: FrameInfo) {
    AGG.with(|a| a.borrow_mut().cur = Some(info));
}

/// Close the frame with its whole-frame wall clock; aggregate + emit a summary every `WINDOW`
/// painter frames. A frame with no recorded dispatch (painter inactive) is skipped.
pub(super) fn end_frame(total_ms: f32) {
    AGG.with(|cell| {
        let a = &mut *cell.borrow_mut();
        let Some(cur) = a.cur.take() else { return };
        a.samples.push((cur.dispatch_ms, total_ms));
        if cur.gpu {
            a.gpu += 1;
        } else {
            a.cpu += 1;
        }
        a.last = cur;
        if a.samples.len() >= WINDOW {
            emit(a);
            a.samples.clear();
            a.gpu = 0;
            a.cpu = 0;
        }
    });
}

fn pct(sorted: &[f32], p: f64) -> f32 {
    if sorted.is_empty() {
        return 0.0;
    }
    sorted[((sorted.len() as f64 * p) as usize).min(sorted.len() - 1)]
}

fn emit(a: &Agg) {
    let n = a.samples.len();
    let mut disp: Vec<f32> = a.samples.iter().map(|s| s.0).collect();
    let mut total: Vec<f32> = a.samples.iter().map(|s| s.1).collect();
    disp.sort_by(|x, y| x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal));
    total.sort_by(|x, y| x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal));
    let f = a.last;
    eprintln!(
        "[paint-perf] {n} frames: GPU {} / CPU {} | dispatch p50={:.2} max={:.2} ms | \
         frame p50={:.2} max={:.2} ms | canvas {}x{} gray={} mask={} lane={} trivial={}",
        a.gpu,
        a.cpu,
        pct(&disp, 0.5),
        disp.last().copied().unwrap_or(0.0),
        pct(&total, 0.5),
        total.last().copied().unwrap_or(0.0),
        f.w,
        f.h,
        f.gray,
        f.active_is_mask,
        if f.lane_partial {
            "partial"
        } else {
            "full/idle"
        },
        f.trivial,
    );
}
