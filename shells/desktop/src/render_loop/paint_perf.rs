//! `PH2D_PAINT_PERF` aggregation — ONE summary line per window, not one per frame.
//!
//! The per-frame log drowned the terminal. Instead, `painter_bridge::dispatch` records this frame's
//! dispatch cost — split into sub-phases — plus the producer + flags via [`record_dispatch`], and
//! `run_render_frame`'s frame timer reports the whole-frame cost via [`end_frame`]; the two are paired
//! and a compact summary is printed once every [`WINDOW`] painter frames. It reports the p50 of each
//! sub-phase (so the dominant one is obvious) and the flags of the WORST-dispatch frame (so they
//! describe the expensive case, not an idle tail frame). Painting a scenario for ~2 s yields one or
//! two lines to paste — enough to locate the cost inside dispatch.

use std::cell::RefCell;

/// ~1.5 s at 60 fps: stable medians, few enough lines to paste.
const WINDOW: usize = 90;

#[derive(Clone, Copy, Default)]
pub(super) struct FrameInfo {
    pub gpu: bool,
    pub dispatch_ms: f32,
    /// Sub-phases of dispatch (they sum to ~`dispatch_ms`): the preview drain (try_drive + the CPU
    /// drain), the layers-panel snapshot publish + shape re-bake, the on-canvas overlays, the CPU
    /// preview upload. Whichever dominates is the cost.
    pub preview_ms: f32,
    pub panel_ms: f32,
    pub overlay_ms: f32,
    pub upload_ms: f32,
    pub w: u32,
    pub h: u32,
    pub gray: bool,
    pub active_is_mask: bool,
    pub lane_partial: bool,
    pub trivial: bool,
}

#[derive(Default)]
struct Agg {
    cur: Option<FrameInfo>,
    samples: Vec<FrameInfo>,
    gpu: u32,
    cpu: u32,
    frame_ms: Vec<f32>,
}

thread_local! {
    static AGG: RefCell<Agg> = RefCell::new(Agg::default());
    static ON: std::cell::Cell<i8> = const { std::cell::Cell::new(-1) };
}

/// Whether `PH2D_PAINT_PERF` is set (cached — no per-frame syscall).
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
        a.samples.push(cur);
        a.frame_ms.push(total_ms);
        if cur.gpu {
            a.gpu += 1;
        } else {
            a.cpu += 1;
        }
        if a.samples.len() >= WINDOW {
            emit(a);
            a.samples.clear();
            a.frame_ms.clear();
            a.gpu = 0;
            a.cpu = 0;
        }
    });
}

fn p50(v: &[f32]) -> f32 {
    if v.is_empty() {
        return 0.0;
    }
    let mut s = v.to_vec();
    s.sort_by(|x, y| x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal));
    s[s.len() / 2]
}

fn emit(a: &Agg) {
    let n = a.samples.len();
    let phase = |f: fn(&FrameInfo) -> f32| p50(&a.samples.iter().map(f).collect::<Vec<_>>());
    // The flags of the SLOWEST-dispatch frame, so they describe the expensive case (not an idle tail).
    let worst = a
        .samples
        .iter()
        .copied()
        .max_by(|x, y| {
            x.dispatch_ms
                .partial_cmp(&y.dispatch_ms)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .unwrap_or_default();
    eprintln!(
        "[paint-perf] {n}f GPU {}/CPU {} | frame p50={:.1} | dispatch p50={:.1} max={:.1} \
         [preview {:.1} panel {:.1} overlay {:.1} upload {:.1}] | \
         WORST: {} {}x{} gray={} mask={} lane={} trivial={}",
        a.gpu,
        a.cpu,
        p50(&a.frame_ms),
        phase(|f| f.dispatch_ms),
        worst.dispatch_ms,
        phase(|f| f.preview_ms),
        phase(|f| f.panel_ms),
        phase(|f| f.overlay_ms),
        phase(|f| f.upload_ms),
        if worst.gpu { "GPU" } else { "CPU" },
        worst.w,
        worst.h,
        worst.gray,
        worst.active_is_mask,
        if worst.lane_partial {
            "partial"
        } else {
            "full/idle"
        },
        worst.trivial,
    );
}
