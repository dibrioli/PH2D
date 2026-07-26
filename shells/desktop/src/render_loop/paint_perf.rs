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

/// Steps of the PANEL phase, in call order.
pub(super) const PANEL_SUB: usize = 5;
pub(super) const PANEL_LABELS: [&str; PANEL_SUB] =
    ["commit", "layerclone", "brushsnap", "pub", "shapebake"];

/// The twelve calls `draw_overlays` makes, in call order.
pub(super) const CHROME_SUB: usize = 12;
pub(super) const CHROME_LABELS: [&str; CHROME_SUB] = [
    "wet", "ring", "curve", "ellipse", "line", "poly", "badges", "selgiz", "deform", "stencil",
    "symm", "fill",
];

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
    /// The OVERLAY phase, split by call — it aggregated three of them, and a 3,9 ms number that could
    /// belong to any of the three is a number nobody can act on (Enio, 2026-07-25, 4096²).
    pub ov_tol_ms: f32,
    pub ov_selection_ms: f32,
    pub ov_chrome_ms: f32,
    /// The PANEL phase split by step (labels in [`PANEL_LABELS`]) — a 3,8 ms number shared by a
    /// revision-gated clone, a `brush_settings` snapshot and two shape re-bakes names none of them.
    pub panel_sub: [f32; PANEL_SUB],
    /// The CHROME call (`draw_overlays`) split by the twelve overlays it aggregates
    /// (labels in [`CHROME_LABELS`]).
    pub chrome_sub: [f32; CHROME_SUB],
    pub upload_ms: f32,
    pub w: u32,
    pub h: u32,
    pub gray: bool,
    pub active_is_mask: bool,
    pub lane_partial: bool,
    pub trivial: bool,
    /// Which arm the drain took — the decisive signal (`FULL-composite` is the O(W×H) cost).
    pub branch: ph2d_tool_painter::DrainBranch,
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
    /// **L0 — o relógio que o artista SENTE.** O instante do evento de ponteiro mais ANTIGO que ainda
    /// não foi mostrado; `None` quando não há nada pendente. Ver [`stamp_pointer`].
    pending: Option<std::time::Instant>,
    /// `evento → fim do frame que o consumiu`, em ms.
    latency_ms: Vec<f32>,
    /// **Quantos eventos chegaram na janela** — TODOS, não só os que abriram um lote.
    ///
    /// ⚠️ Sem ele o relatório não distingue as duas causas de uma latência alta, e elas têm curas
    /// OPOSTAS: se chegam ~1 evento por frame, o atraso é do **pipeline** (o frame demora a sair); se
    /// chegam muitos, eles **ENFILEIRAM** e a cura é consumir o lote inteiro por frame. A 1ª leitura
    /// real (Enio, 4096², 2026-07-25) deu `p50 34,3 ms` com `frame p50 4,7 ms` — aritmética que só
    /// fecha se uma das duas for verdade, e o relatório não dizia qual.
    events: u32,
    /// **O trabalho de pintar, que acontece FORA do frame.** Acumulado dentro da janela entre dois
    /// `end_frame`, em ms.
    ///
    /// ⚠️ **O `PaintFrameTimer` cobre o `run_render_frame` inteiro — e o `on_canvas_pointer` NÃO roda
    /// lá dentro.** Ele roda no handler de input do winit, então carimbar dabs (a 4096², com impasto)
    /// nunca apareceu em `frame`, nem em `dispatch`, nem em nenhum dos 17 sub-slots. A 1ª leitura com
    /// período real mostrou o buraco: `frame p50 16,7 ms` e **período 99,9 ms/frame**, com 8,8 eventos
    /// por frame — 83 ms por frame fora de tudo que o relatório media.
    input_ms: f32,
    /// Por-frame, o acumulado acima.
    input_hist: Vec<f32>,
    /// Início da janela, para o **PERÍODO** real do frame (`span / frames`).
    ///
    /// ⚠️ O `frame p50` mede o **TRABALHO** dentro do frame, não o intervalo entre frames — e a
    /// diferença é o que o app passa esperando o vsync, a fila de eventos e o compositor. Uma latência
    /// de 34 ms sobre um trabalho de 4,7 é impossível de ler sem o período ao lado.
    window_start: Option<std::time::Instant>,
}

thread_local! {
    static AGG: RefCell<Agg> = RefCell::new(Agg::default());
    static ON: std::cell::Cell<i8> = const { std::cell::Cell::new(-1) };
}

/// **L0: carimba a chegada de um evento de ponteiro de canvas** (o plano 26, frente L).
///
/// O módulo media `ms/frame` e `ms/dispatch` e **nunca mediu do evento até o pixel** — que é o único
/// número que a pergunta *"por que o Procreate parece mais rápido?"* de fato cobra, e cujo alvo é
/// público: **9 ms** (o Apple Pencil saiu de 20 para 9, e não foi compute — foi pipeline).
///
/// ⚠️ **Carimba o MAIS ANTIGO não-servido, nunca o mais recente.** Entre dois frames chegam vários
/// eventos; o que o artista percebe como atraso é o do PRIMEIRO deles, porque é ele que está na tela
/// esperando há mais tempo. Carimbar o último mediria a latência do evento mais sortudo do lote e
/// reportaria um número que ninguém sente.
///
/// ⚠️ **É o relógio da SHELL, não do tool** — o contrato `CanvasPaintTool` está congelado (§6) e um
/// método novo nele exigiria ADR. A shell já é dona do evento; ela o carimba na entrega.
pub(crate) fn stamp_pointer() {
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

/// **Quanto este evento custou DENTRO do `on_canvas_pointer`** — o trabalho de carimbar dabs.
///
/// Irmão do [`stamp_pointer`]: aquele diz quando o evento CHEGOU, este diz quanto ele CUSTOU. Os dois
/// juntos são a diferença entre *"o frame demora a sair"* e *"pintar é caro e ninguém estava olhando"*.
pub(crate) fn record_input(ms: f32) {
    if !on() {
        return;
    }
    AGG.with(|cell| cell.borrow_mut().input_ms += ms);
}

/// Whether `PH2D_PAINT_PERF` is set (cached — no per-frame syscall).
pub(crate) fn on() -> bool {
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
        if a.window_start.is_none() {
            a.window_start = Some(std::time::Instant::now());
        }
        // L0: este frame consumiu o que estava pendente — fecha o relógio do evento mais antigo.
        // ⚠️ Ele mede *evento → fim do frame*, e o present acontece a uma fração de ms daqui; a
        // diferença é honesta e menor que a resolução com que o artista percebe atraso. Prometer
        // "→ pixel" quando o instrumento para no fim do frame seria vender o que ele não mede.
        if let Some(t0) = a.pending.take() {
            a.latency_ms.push(t0.elapsed().as_secs_f32() * 1e3);
        }
        a.samples.push(cur);
        a.frame_ms.push(total_ms);
        a.input_hist.push(std::mem::take(&mut a.input_ms));
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
            a.input_hist.clear();
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

/// O quantil `q` de `v` (0..1). ⚠️ A latência se julga pela **CAUDA**: uma mediana de 8 ms com um
/// p95 de 40 é exatamente o que o artista descreve como *"às vezes trava"*, e a mediana sozinha diz
/// que está tudo bem.
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
        "[paint-perf] {n}f GPU {}/CPU {} | frame p50={:.1} | dispatch p50={:.1} max={:.1} \
         [preview {:.1} panel {:.1} overlay {:.1} (tol {:.1} sel {:.1} chrome {:.1}) upload {:.1}] | \
         WORST: {} {}x{} branch={} impasto={} mask_scratch={} gray={} mask={} lane={} trivial={}",
        a.gpu,
        a.cpu,
        p50(&a.frame_ms),
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
    // ⚠️ **E o SPLIT do frame PIOR, não só a magnitude dele.** O `max=` diz QUANTO o pior frame custou
    // e as flags dizem em que estado ele estava — mas as fases eram todas p50, ou seja descreviam o
    // frame TÍPICO. Um custo de UMA VEZ (o primeiro traço alocando as texturas do preview no tamanho
    // do canvas) é invisível numa mediana por construção: ele é exatamente o outlier que a mediana
    // existe para descartar. Sem esta linha, "muito delay no primeiro traço" não tem onde ser lido.
    eprintln!(
        "[paint-perf] WORST split: dispatch={:.1} [preview {:.1} panel {:.1} overlay {:.1} upload {:.1}]",
        worst.dispatch_ms, worst.preview_ms, worst.panel_ms, worst.overlay_ms, worst.upload_ms
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
    // L0: o número que a pergunta do Enio cobra, com o alvo público ao lado dele — e com as DUAS
    // grandezas que dizem de onde ele vem (o período real do frame, e quantos eventos por frame).
    if !a.latency_ms.is_empty() {
        #[allow(clippy::cast_precision_loss)]
        let frames = a.samples.len().max(1) as f32;
        let period = a
            .window_start
            .map_or(0.0, |t| t.elapsed().as_secs_f32() * 1e3 / frames);
        eprintln!(
            "[paint-perf]   EVENTO->FRAME p50={:.1} p95={:.1} max={:.1} ms (n={}) · alvo 9 \
             | periodo real {period:.1} ms/frame · {:.1} eventos/frame \
             | INPUT (fora do frame) p50={:.1} max={:.1} ms",
            pq(&a.latency_ms, 0.5),
            pq(&a.latency_ms, 0.95),
            a.latency_ms.iter().fold(0.0f32, |m, v| m.max(*v)),
            a.latency_ms.len(),
            f32::from(u16::try_from(a.events).unwrap_or(u16::MAX)) / frames,
            p50(&a.input_hist),
            a.input_hist.iter().fold(0.0f32, |m, v| m.max(*v)),
        );
    }
}

/// What the preview-diag trap reports: who owns the slot, and the CPU partial-upload bbox.
pub(super) type PreviewDiagState = (bool, Option<(u32, u32, u32, u32)>);

/// Has the preview-diag state CHANGED since the last frame? (`PH2D_PREVIEW_DIAG`.)
///
/// The trap answers *"which producer owns the slot, and what bbox did the CPU upload"* — both of which
/// are interesting exactly when they MOVE. Logged per frame they emitted 60 lines a second, and the
/// transition everyone was hunting scrolled past between two identical neighbours.
///
/// State lives here, next to the other diagnostic's accumulators, rather than in the bridge: a `static`
/// declared inside a `dispatch` that already carries a dozen `dbg_` locals is a thirteenth thing to read
/// past, and this one has to persist across frames.
pub(super) fn preview_diag_changed(now: PreviewDiagState) -> bool {
    use std::sync::Mutex;
    static LAST: Mutex<Option<PreviewDiagState>> = Mutex::new(None);
    let Ok(mut last) = LAST.lock() else {
        // A poisoned lock must not silence a diagnostic — logging one extra line is the harmless
        // direction to be wrong in.
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
/// ⚠️ `std::env::set_var` é `unsafe` na edição 2024 e a shell tem `forbid(unsafe_code)`; e mesmo que
/// não tivesse, mutar o ambiente de um processo de teste multi-thread é uma corrida. O `ON` já é
/// `thread_local`, então armá-lo direto é a porta certa.
#[cfg(test)]
pub(super) fn force_on() {
    ON.with(|c| c.set(1));
}

/// Só em teste: o custo do input por frame, na janela corrente.
#[cfg(test)]
pub(super) fn input_hist() -> Vec<f32> {
    AGG.with(|a| a.borrow().input_hist.clone())
}

/// Só em teste: quantos eventos a janela corrente viu.
#[cfg(test)]
pub(super) fn events_seen() -> u32 {
    AGG.with(|a| a.borrow().events)
}

/// Só em teste: as amostras de latência acumuladas na janela corrente.
#[cfg(test)]
pub(super) fn latency_samples() -> Vec<f32> {
    AGG.with(|a| a.borrow().latency_ms.clone())
}
