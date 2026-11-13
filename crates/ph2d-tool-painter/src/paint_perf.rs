//! `PH2D_PAINT_PERF` aggregation — ONE summary line per window, not one per frame.
//!
//! `painter_bridge::dispatch` records the sub-phase split via [`record_dispatch`], the frame timer
//! the whole frame via [`end_frame`]; every [`WINDOW`] painter frames it prints the p50 of each
//! sub-phase and the flags of the WORST-dispatch frame.
//! (história: docs/Painter/historia_comentarios_app_painter_2026-11-10.md)

use std::cell::RefCell;

/// ~1.5 s at 60 fps: stable medians, few enough lines to paste.
const WINDOW: usize = 90;

/// Steps of the PANEL phase, in call order.
pub const PANEL_SUB: usize = 5;
pub const PANEL_LABELS: [&str; PANEL_SUB] =
    ["commit", "layerclone", "brushsnap", "pub", "shapebake"];

/// The twelve calls `draw_overlays` makes, in call order.
pub const CHROME_SUB: usize = 13;
pub const CHROME_LABELS: [&str; CHROME_SUB] = [
    "wet", "ring", "curve", "ellipse", "line", "poly", "badges", "selgiz", "deform", "stencil",
    "symm", "grid", "fill",
];

#[derive(Clone, Copy, Default)]
pub struct FrameInfo {
    pub gpu: bool,
    pub dispatch_ms: f32,
    /// Sub-phases of dispatch (they sum to ~`dispatch_ms`): the preview drain (try_drive + the CPU
    /// drain), the layers-panel snapshot publish + shape re-bake, the on-canvas overlays, the CPU
    /// preview upload. Whichever dominates is the cost.
    pub preview_ms: f32,
    pub panel_ms: f32,
    pub overlay_ms: f32,
    /// The OVERLAY phase, split by call.
    pub ov_tol_ms: f32,
    pub ov_selection_ms: f32,
    pub ov_chrome_ms: f32,
    /// The PANEL phase split by step (labels in [`PANEL_LABELS`]).
    pub panel_sub: [f32; PANEL_SUB],
    /// The CHROME call (`draw_overlays`) split by the twelve overlays it aggregates
    /// (labels in [`CHROME_LABELS`]).
    pub chrome_sub: [f32; CHROME_SUB],
    pub upload_ms: f32,
    /// The CPU fold of the relief planes inside the GPU route (ms) — the one cost there that
    /// depends on the window being a rect or the whole canvas.
    pub fold_ms: f32,
    /// `true` when the fold walked the WHOLE canvas.
    pub fold_full: bool,
    pub w: u32,
    pub h: u32,
    pub gray: bool,
    pub active_is_mask: bool,
    pub lane_partial: bool,
    pub trivial: bool,
    /// Which arm the drain took — the decisive signal (`FULL-composite` is the O(W×H) cost).
    pub branch: crate::DrainBranch,
    /// WHY a trivial stack fell to the full arm: relief is lit / a mask brush scratch is live.
    pub impasto: bool,
    pub mask_scratch: bool,
}

#[derive(Default)]
struct Agg {
    cur: Option<FrameInfo>,
    samples: Vec<FrameInfo>,
    gpu: u32,
    cpu: u32,
    frame_ms: Vec<f32>,
    /// L0: the OLDEST pointer event not yet shown (`None` = none pending); see [`stamp_pointer`].
    pending: Option<std::time::Instant>,
    /// `evento → fim do frame que o consumiu`, em ms.
    latency_ms: Vec<f32>,
    /// Events that arrived in the window (ALL of them): ~1/frame = pipeline delay, many = they
    /// queue — opposite cures.
    events: u32,
    /// Painting work OUTSIDE the frame (`on_canvas_pointer` runs in the input handler), in ms,
    /// accumulated between two `end_frame`s, per phase `[down, move, up]`.
    input_ms: [f32; 3],
    fold_ms: f32,
    fold_full: bool,
    /// Por-frame, o acumulado acima.
    input_hist: [Vec<f32>; 3],
    /// Window start, for the real frame PERIOD (`span / frames`); `frame p50` measures work only.
    window_start: Option<std::time::Instant>,
    /// B10.1: o soltar sob vigia ([`PenUp`]) e o fim do quadro anterior (o intervalo de parede).
    pen_up: Option<PenUp>,
    last_end: Option<std::time::Instant>,
}

/// Quadros que o `[pen-up]` vigia depois de cada soltar (~1 s a 60 q/s): o quadro lento do pen-up 4K cai neles e a
/// mediana da janela de 90 escondia-o (Impasto: 15/15 quadros de 64–74 ms com p50 18,5).
const QUADROS_DEPOIS_DO_UP: u32 = 60;

/// B10.1: o pior quadro e o pior intervalo nos [`QUADROS_DEPOIS_DO_UP`] quadros depois de um soltar.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PenUp {
    /// O relógio de parede do soltar (ns desde a época, o do `[quadro-lento]` e do `date +%s%N`).
    pub parede_ns: u128,
    /// O custo do `Up` no tratamento da entrada (fora do quadro).
    pub up_ms: f32,
    pub quadros: u32,
    pub max_quadro_ms: f32,
    pub max_intervalo_ms: f32,
}

impl PenUp {
    /// Conta um quadro; `true` quando a vigia acabou.
    pub fn quadro(&mut self, quadro_ms: f32, intervalo_ms: f32) -> bool {
        self.quadros += 1;
        self.max_quadro_ms = self.max_quadro_ms.max(quadro_ms);
        self.max_intervalo_ms = self.max_intervalo_ms.max(intervalo_ms);
        self.quadros >= QUADROS_DEPOIS_DO_UP
    }

    pub fn linha(&self) -> String {
        format!(
            "[pen-up] parede={} up={:.1} max_quadro={:.1} max_intervalo={:.1} em {} quadros",
            self.parede_ns, self.up_ms, self.max_quadro_ms, self.max_intervalo_ms, self.quadros
        )
    }
}

fn parede_ns() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos())
}

thread_local! {
    static AGG: RefCell<Agg> = RefCell::new(Agg::default());
    static ON: std::cell::Cell<i8> = const { std::cell::Cell::new(-1) };
}

/// L0: stamps the arrival of a canvas pointer event (event → end of the consuming frame).
/// Stamps the OLDEST unserved event, never the latest — that is the one the artist waits on.
/// The shell's clock, not the tool's (`CanvasPaintTool` is frozen, §6).
pub fn stamp_pointer() {
    if !on() {
        return;
    }
    AGG.with(|cell| {
        let a = &mut *cell.borrow_mut();
        a.events = a.events.saturating_add(1);
        if a.pending.is_none() {
            a.pending = Some(std::time::Instant::now());
        }
    });
}

/// Records the CPU fold cost of the relief planes (inside `try_drive`) and whether it walked the
/// whole canvas; `record_dispatch` drains it into the frame's `FrameInfo`. No-op without the env.
pub fn note_gpu_fold(ms: f32, full: bool) {
    if !on() {
        return;
    }
    AGG.with(|cell| {
        let mut a = cell.borrow_mut();
        a.fold_ms += ms;
        a.fold_full |= full;
    });
}

pub fn record_input(ms: f32, phase: InputPhase) {
    if !on() {
        return;
    }
    AGG.with(|cell| {
        let a = &mut *cell.borrow_mut();
        a.input_ms[phase as usize] += ms;
        if matches!(phase, InputPhase::Up) {
            // Um soltar antes de a vigia do anterior acabar fecha-a com o que viu.
            if let Some(p) = a.pen_up.take() {
                eprintln!("{}", p.linha());
            }
            a.pen_up = Some(PenUp {
                parede_ns: parede_ns(),
                up_ms: ms,
                ..PenUp::default()
            });
        }
    });
}

/// A fase de um evento de ponteiro, para o balde de INPUT. Um `Down` abre um traço (e paga o clone de
/// canvas), um `Move` carimba dabs, um `Up` fecha e COMMITA — três custos de natureza diferente.
#[derive(Clone, Copy, Debug)]
pub enum InputPhase {
    Down = 0,
    Move = 1,
    Up = 2,
}

impl InputPhase {
    /// The phase of a canvas event (lives next to the enum it produces).
    pub const fn of(phase: ph2d_editor_core::tool::PointerPhase) -> Self {
        match phase {
            ph2d_editor_core::tool::PointerPhase::Down => Self::Down,
            ph2d_editor_core::tool::PointerPhase::Up => Self::Up,
            _ => Self::Move,
        }
    }
}

/// Whether `PH2D_PAINT_PERF` is set (cached — no per-frame syscall).
pub fn on() -> bool {
    ON.with(|c| {
        if c.get() < 0 {
            c.set(i8::from(std::env::var_os("PH2D_PAINT_PERF").is_some()));
        }
        c.get() > 0
    })
}

/// Record this frame's painter dispatch. Call only when the painter is active.
pub fn record_dispatch(info: FrameInfo) {
    AGG.with(|a| {
        let mut a = a.borrow_mut();
        // Drain the fold noted in this frame's `preview` phase; zeroing stops inheritance.
        let mut info = info;
        info.fold_ms = std::mem::take(&mut a.fold_ms);
        info.fold_full = std::mem::take(&mut a.fold_full);
        a.cur = Some(info);
    });
}

thread_local! {
    /// O fim do quadro anterior e o limiar de [`quadro_lento`] (`None` = desligado; lido uma vez).
    static LENTO: std::cell::Cell<(Option<std::time::Instant>, Option<Option<f32>>)> =
        const { std::cell::Cell::new((None, None)) };
}

/// ⚠️ **Medição, nunca produto** (dono 10/11: «trava no mouse-up numa imagem 4K»): `PH2D_QUADRO_LENTO_MS=<n>` imprime
/// cada quadro cujo INTERVALO desde o fim do anterior passa de `n` ms, com o relógio de parede (ns desde a época, o
/// mesmo do `date +%s%N` da sonda) — o agregado por janela esconde um quadro só, e um quadro parado À ESPERA (placa,
/// trinco) não aparece no perfil de CPU. `quadro` = o relógio do `PaintFrameTimer`; `intervalo` = fim a fim.
fn quadro_lento(total_ms: f32, info: Option<FrameInfo>) {
    LENTO.with(|c| {
        let (antes, limiar) = c.get();
        let limiar = limiar.unwrap_or_else(|| {
            std::env::var("PH2D_QUADRO_LENTO_MS")
                .ok()
                .and_then(|v| v.parse().ok())
        });
        let agora = std::time::Instant::now();
        if let (Some(n), Some(a)) = (limiar, antes) {
            let intervalo = agora.duration_since(a).as_secs_f32() * 1e3;
            if intervalo > n {
                let parede = parede_ns();
                // n.º 8: as partes do Painter NESTE quadro (o resto do `quadro` é a shell e a apresentação).
                let partes = info.map_or_else(String::new, |f| {
                    format!(
                        " painter: dispatch={:.1} preview={:.1} painel={:.1} overlay={:.1} upload={:.1} dobra={:.1} ramo={:?}",
                        f.dispatch_ms, f.preview_ms, f.panel_ms, f.overlay_ms, f.upload_ms, f.fold_ms, f.branch
                    )
                });
                eprintln!(
                    "[quadro-lento] parede={parede} intervalo={intervalo:.1} quadro={total_ms:.1}{partes}"
                );
            }
        }
        c.set((Some(agora), Some(limiar)));
    });
}

/// Close the frame with its whole-frame wall clock; aggregate + emit a summary every `WINDOW`
/// painter frames. A frame with no recorded dispatch (painter inactive) is skipped.
pub fn end_frame(total_ms: f32) {
    quadro_lento(total_ms, AGG.with(|c| c.borrow().cur));
    AGG.with(|cell| {
        let a = &mut *cell.borrow_mut();
        let agora = std::time::Instant::now();
        let intervalo = a
            .last_end
            .replace(agora)
            .map_or(total_ms, |t| agora.duration_since(t).as_secs_f32() * 1e3);
        if let Some(p) = a.pen_up.as_mut()
            && p.quadro(total_ms, intervalo)
        {
            eprintln!("{}", p.linha());
            a.pen_up = None;
        }
        let Some(cur) = a.cur.take() else { return };
        if a.window_start.is_none() {
            a.window_start = Some(std::time::Instant::now());
        }
        // L0: este frame consumiu o que estava pendente — fecha o relógio do evento mais antigo.
        // Measures event → end of frame (the present follows a fraction of a ms later).
        if let Some(t0) = a.pending.take() {
            a.latency_ms.push(t0.elapsed().as_secs_f32() * 1e3);
        }
        a.samples.push(cur);
        a.frame_ms.push(total_ms);
        for i in 0..3 {
            a.input_hist[i].push(std::mem::take(&mut a.input_ms[i]));
        }
        if cur.gpu {
            a.gpu += 1;
        } else {
            a.cpu += 1;
        }
        if a.samples.len() >= WINDOW {
            emit(a);
            a.samples.clear();
            a.frame_ms.clear();
            a.latency_ms.clear();
            for h in &mut a.input_hist {
                h.clear();
            }
            a.events = 0;
            a.window_start = None;
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

/// The `q` quantile of `v` (0..1) — latency is judged by its TAIL.
fn pq(v: &[f32], q: f32) -> f32 {
    if v.is_empty() {
        return 0.0;
    }
    let mut s = v.to_vec();
    s.sort_by(|x, y| x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal));
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let i = ((s.len() - 1) as f32 * q).round() as usize;
    s[i.min(s.len() - 1)]
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
        "[paint-perf] {n}f GPU {}/CPU {} | frame p50={:.1} max={:.1} | dispatch p50={:.1} max={:.1} \
         [preview {:.1} panel {:.1} overlay {:.1} (tol {:.1} sel {:.1} chrome {:.1}) upload {:.1}] | \
         WORST: {} {}x{} branch={} impasto={} mask_scratch={} gray={} mask={} lane={} trivial={}",
        a.gpu,
        a.cpu,
        p50(&a.frame_ms),
        a.frame_ms.iter().fold(0.0f32, |m, v| m.max(*v)),
        phase(|f| f.dispatch_ms),
        worst.dispatch_ms,
        phase(|f| f.preview_ms),
        phase(|f| f.panel_ms),
        phase(|f| f.overlay_ms),
        phase(|f| f.ov_tol_ms),
        phase(|f| f.ov_selection_ms),
        phase(|f| f.ov_chrome_ms),
        phase(|f| f.upload_ms),
        if worst.gpu { "GPU" } else { "CPU" },
        worst.w,
        worst.h,
        worst.branch.name(),
        worst.impasto,
        worst.mask_scratch,
        worst.gray,
        worst.active_is_mask,
        if worst.lane_partial {
            "partial"
        } else {
            "full/idle"
        },
        worst.trivial,
    );
    // The WORST frame's split too: a one-off cost is invisible in a median by construction.
    eprintln!(
        "[paint-perf] WORST split: dispatch={:.1} [preview {:.1} (fold {:.1} janela={}) panel {:.1} \
         overlay {:.1} upload {:.1}]",
        worst.dispatch_ms,
        worst.preview_ms,
        worst.fold_ms,
        if worst.fold_full { "TELA" } else { "rect" },
        worst.panel_ms,
        worst.overlay_ms,
        worst.upload_ms
    );
    // The two aggregate phases, split by step. Printed on their own lines (and only for the steps that
    // register anything) so the dominant one is named rather than shared.
    let mut panel = String::new();
    for (i, name) in PANEL_LABELS.iter().enumerate() {
        let v = p50(&a.samples.iter().map(|f| f.panel_sub[i]).collect::<Vec<_>>());
        let mx = a.samples.iter().fold(0.0f32, |m, f| m.max(f.panel_sub[i]));
        panel.push_str(&format!(" {name} {v:.2}/{mx:.2}"));
    }
    eprintln!("[paint-perf]   PANEL p50/max:{panel}");
    let mut chrome = String::new();
    for (i, name) in CHROME_LABELS.iter().enumerate() {
        let v = p50(&a
            .samples
            .iter()
            .map(|f| f.chrome_sub[i])
            .collect::<Vec<_>>());
        let mx = a.samples.iter().fold(0.0f32, |m, f| m.max(f.chrome_sub[i]));
        chrome.push_str(&format!(" {name} {v:.2}/{mx:.2}"));
    }
    eprintln!("[paint-perf]   CHROME p50/max:{chrome}");
    // The tool's watercolour half, when it painted this window (in the report the artist runs).
    // `ns/texel` constant = the work grew · rising = contention.
    let wash = crate::wash_diag::take();
    if wash.composite.n > 0 || wash.stamp.n > 0 {
        eprintln!(
            "[paint-perf]   AQUARELA p50/max: composite {:.2}/{:.2} x{} | carimbo {:.2}/{:.2} x{} \
             | pour {:.2} x{} | secagem {:.2} x{} | pen-down {:.2} x{} \
             | janela {:.2} M texels/composite | {:.1} ns/texel",
            wash.composite.avg_ms,
            wash.composite.max_ms,
            wash.composite.n,
            wash.stamp.avg_ms,
            wash.stamp.max_ms,
            wash.stamp.n,
            wash.pour.avg_ms,
            wash.pour.n,
            wash.dry.avg_ms,
            wash.dry.n,
            wash.pendown.avg_ms,
            wash.pendown.n,
            wash.window_px_per_composite / 1e6,
            wash.ns_per_texel,
        );
    }
    // L0: event→frame latency with the public target, the real period and events per frame.
    if !a.latency_ms.is_empty() {
        #[allow(clippy::cast_precision_loss)]
        let frames = a.samples.len().max(1) as f32;
        let period = a
            .window_start
            .map_or(0.0, |t| t.elapsed().as_secs_f32() * 1e3 / frames);
        eprintln!(
            "[paint-perf]   EVENTO->FRAME p50={:.1} p95={:.1} max={:.1} ms (n={}) · alvo 9 \
             | periodo real {period:.1} ms/frame · {:.1} eventos/frame \
             | INPUT (fora do frame) down {:.1}/{:.1} move {:.1}/{:.1} up {:.1}/{:.1} ms (p50/max)",
            pq(&a.latency_ms, 0.5),
            pq(&a.latency_ms, 0.95),
            a.latency_ms.iter().fold(0.0f32, |m, v| m.max(*v)),
            a.latency_ms.len(),
            f32::from(u16::try_from(a.events).unwrap_or(u16::MAX)) / frames,
            p50(&a.input_hist[0]),
            a.input_hist[0].iter().fold(0.0f32, |m, v| m.max(*v)),
            p50(&a.input_hist[1]),
            a.input_hist[1].iter().fold(0.0f32, |m, v| m.max(*v)),
            p50(&a.input_hist[2]),
            a.input_hist[2].iter().fold(0.0f32, |m, v| m.max(*v)),
        );
    }
}

/// What the preview-diag trap reports: who owns the slot, and the CPU partial-upload bbox.
pub type PreviewDiagState = (bool, Option<(u32, u32, u32, u32)>);

/// Has the preview-diag state CHANGED since the last frame? (`PH2D_PREVIEW_DIAG`.)
///
/// Logs on CHANGE only (per frame it hid the transitions it exists to show); the state
/// persists across frames here.
pub fn preview_diag_changed(now: PreviewDiagState) -> bool {
    use std::sync::Mutex;
    static LAST: Mutex<Option<PreviewDiagState>> = Mutex::new(None);
    let Ok(mut last) = LAST.lock() else {
        // A poisoned lock must not silence a diagnostic.
        return true;
    };
    if last.as_ref() == Some(&now) {
        return false;
    }
    *last = Some(now);
    true
}

#[cfg(test)]
#[path = "paint_perf_tests.rs"]
mod tests;

/// Só em teste: liga o agregador nesta thread SEM tocar no ambiente.
///
/// `ON` is `thread_local`, so arming it directly avoids `set_var` (unsafe, racy in tests).
#[cfg(test)]
pub(crate) fn force_on() {
    ON.with(|c| c.set(1));
}

/// Só em teste: o custo do input por frame, na janela corrente.
#[cfg(test)]
pub(crate) fn input_hist_by_phase() -> (Vec<f32>, Vec<f32>, Vec<f32>) {
    AGG.with(|a| {
        let h = &a.borrow().input_hist;
        (h[0].clone(), h[1].clone(), h[2].clone())
    })
}

/// A soma das três fases por frame — o leitor agregado.
///
/// `cfg(test)`: the report prints the three phases apart; only the gate sums.
#[cfg(test)]
pub(crate) fn fold_hist() -> Vec<(f32, bool)> {
    AGG.with(|a| {
        a.borrow()
            .samples
            .iter()
            .map(|f| (f.fold_ms, f.fold_full))
            .collect()
    })
}

#[cfg(test)]
pub(crate) fn input_hist() -> Vec<f32> {
    // A soma das três fases — o que este leitor sempre quis dizer (*quanto o input custou neste frame*).
    AGG.with(|a| {
        let h = &a.borrow().input_hist;
        (0..h[0].len())
            .map(|i| h[0][i] + h[1][i] + h[2][i])
            .collect()
    })
}

/// Só em teste: quantos eventos a janela corrente viu.
#[cfg(test)]
pub(crate) fn events_seen() -> u32 {
    AGG.with(|a| a.borrow().events)
}

/// Só em teste: as amostras de latência acumuladas na janela corrente.
#[cfg(test)]
pub(crate) fn latency_samples() -> Vec<f32> {
    AGG.with(|a| a.borrow().latency_ms.clone())
}
