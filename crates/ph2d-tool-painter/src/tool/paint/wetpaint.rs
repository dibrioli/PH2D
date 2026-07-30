//! **Wet Paint** ([`PaintMode::WetPaint`]) — the `ph2d-wet-paint` fluid engine
//! as a paint mode (ADR-0134). The MODE is the master switch: no `BrushSpec`
//! flag exists on purpose (two doors to one question diverge — the Knife
//! precedent), so "off" means "any other mode", and the OFF contract is that
//! no other mode ever constructs a session (gated below).
//!
//! ## The session model (the watercolor wet-session, generalized)
//!
//! A session freezes the canvas (`base`, a shared `Arc` — O(1)) and owns an
//! engine grid sized to it; every composite re-renders `pigment OVER base`
//! into `canvas_rgba` for exactly the engine's dirty rect. The session spans
//! STROKES — the water stays live between pen-ups, which is the module's
//! whole point — and it is **display-state, not document-state**: the pixels
//! the artist sees are always IN `canvas_rgba`, so ending a session is the
//! bake (nothing to write), and the per-stroke undo capture at pen-down
//! already holds the look. The engine grid therefore stays OUT of
//! `ModelSnapshot` — a `GridSnapshot` per undo step would be ~14 f32 planes
//! per canvas (~235 MB at 2048², the ADR-0117 disease).
//!
//! What guards that stance is the **canvas-identity guard** (the watercolor's
//! `wet_session_canvas`, made eager): any foreign mutation — undo, layer
//! switch, fill, resize, another tool — swaps the `canvas_rgba` Arc, and the
//! next wet-paint touch (dab OR tick) sees `Arc::ptr_eq` fail and ends the
//! session. The tick half is load-bearing: the sim composites WITHOUT a
//! pen-down, so a lazy at-pen-down check would let a live session repaint
//! over a canvas the undo just restored.
//!
//! ## Coordinates and dab mapping
//!
//! Engine cells are 1-based with a pad ring: canvas pixel `(0..W-1)` maps to
//! cell `(1..W)` (the reference app's `view.toCell`: `x = px + 1`). Dabs go
//! through the engine's OWN §9 parameter mapping (`dispatch_pressure_dab`)
//! with exactly two host substitutions: pressure = `coverage / strength`
//! (the dab's real pressure response, rescaled to §8's ~0..10 range) and
//! radius = the dab's real `radius_px`. Colour is taken at stroke start
//! (per-dab Randomize colour is a W2 seam).
//!
//! ## Authoring vs deposit (doc 21 — deposit-at-commit)
//!
//! The re-stamping methods (DragDot / Anchored / the shape editors) AUTHOR
//! through the normal flat pipeline (their batches are un-owned — see
//! [`PainterTool::wet_owns_the_dabs`]) and the fluid receives the FINAL dab
//! list exactly once, at commit (pen-up / Enter / Apply), through the
//! commit door (`wetpaint_commit`). The impasto height pass never runs in
//! this mode (relief pinned to a footprint would outlive paint that flows
//! away — gated in `stamp_dabs_inner`). A live deposit still only happens
//! inside a real incremental gesture: re-stamp previews piling paint into a
//! non-idempotent fluid while the artist just LOOKS (the I2 disease) is the
//! exact failure the ownership split + the route's belt refuse.

use super::*;
use offthread::{EngineSlot, SimWorker};
use ph2d_wet_paint::painter::{Dirty, Engine};
use std::sync::Weak;

/// O estado autorado do Wet Paint — o arm, a sessão viva, os knobs, o tool e o
/// stash do commit.
///
/// ⚠️ **Nem o passo nem o orçamento de tempo moram mais aqui.** A sim roda numa
/// thread própria ([`offthread`]) e o relógio dela é o do WORKER, então este
/// tool não tem mais `acc`, não tem cap de contagem (`WET_MAX_STEPS`) e não tem
/// orçamento de milissegundos (`SimBudget`): o frame não paga passo nenhum, ele
/// MOSTRA o que o worker já fez. Os seis defeitos que a era do agendamento
/// fechou — realimentação em `dt`, orçamento fixo, atribuição, catraca, régua
/// pregada e o passo atômico — ficam registrados no doc 28 §5.31-§5.37, e o
/// passo interrompível que a última delas construiu é **o que torna esta wave
/// possível**: o worker devolve o motor em fronteira de ESTÁGIO (3-10 ms), não
/// de passo (33-38).
pub(crate) struct WetPaintState {
    /// The Wet Paint **checkbox** — the authored, persistent ARM (Enio
    /// 2026-07-21, the Watercolor/Impasto pattern): while `true`, the
    /// `"brush"` wire resolves to [`PaintMode::WetPaint`], so leaving to the
    /// eraser / selection / any tool and coming back returns to the FLUID
    /// instead of the plain digital brush. One fact, mode-independent, not a
    /// `BrushSpec` field (per-slot copies of one truth would disagree).
    /// Entering the mode by ANY door arms it — a checkbox that reads OFF
    /// while the paint is wet would be the lying radio this file refuses.
    pub(super) armed: bool,
    /// The live session; `None` until the first Wet Paint dab lands.
    pub(super) session: Option<WetSession>,
    /// A live freehand paint GESTURE is open (`paint_begin` .. pen-up). This —
    /// not `paint.stroke` — is the deposit gate: the lifecycle `mem::take`s
    /// the stroke while stamping, and the shape editors' per-frame re-stamps
    /// never run `paint_begin` at all, so the flag is exactly "dabs are the
    /// artist's hand, once".
    pub(super) live_gesture: bool,
    /// The authored knob values (W3) — survive the session, the mode and the
    /// tool round-trips; the section reset restores [`WetKnobs::default`].
    pub(super) knobs: WetKnobs,
    /// Doc 21 (deposit-at-commit): the commit-deposit STASH — the exact batch
    /// the last flat authoring preview painted (`stamp_drag_preview`'s own
    /// parameter, untiled/mirrored; the dispatcher re-tiles at deposit
    /// exactly as it did at preview). The deposit IS the preview by
    /// construction — nothing is re-derived at commit. Transient, never
    /// serialized; cleared by pen-down, cancel, mode-leave and teardown.
    pub(super) pending_deposit: Vec<Dab>,
    /// Doc 21: TRUE only across the single `stamp_dabs` replay inside
    /// `wetpaint_commit_deposit` — the third key on the wet arm's gate. A
    /// leak of this flag while a preview re-stamp runs is I2 resurrected
    /// (every refill becomes a fluid deposit); it is written in exactly two
    /// adjacent statements around that one call, nowhere else.
    pub(super) deposit_pass: bool,
    /// The wet TOOL (doc 22): which of the model's tools the brush wire
    /// drives while armed. Paint by default; Erase is NOT here — it is the
    /// rail eraser's other view.
    pub(super) tool: WetTool,
    /// The tilt DIAL (doc 22): on/off + ring (0..8, magnitude `ring/4`) +
    /// spoke (0..11, 30° steps). Boot = the model's boot = the engine's
    /// boot: ON, straight down, half radius (gate-pinned no-op reconcile).
    pub(super) tilt_on: bool,
    pub(super) tilt_ring: u8,
    pub(super) tilt_spoke: u8,
    /// Show-wet overlay (display-only; NEVER baked — the end-session door
    /// recomposites clean first).
    pub(super) show_wet: bool,
    /// Paper checkbox (doc 22 §2.8): the tooth becomes visually part of the
    /// painting (granulation + emboss into the pigment colours). Baked on
    /// purpose — it is part of the painting, unlike the show-wet veil.
    pub(super) paper_visual: bool,
    /// Experimental: K–M pigment mixing (`sim.km_mixing`).
    pub(super) km_mixing: bool,
    /// Experimental: K–M glaze stacking in the composite.
    pub(super) km_glaze: bool,
    /// The Tuning side panel's visibility (the basic section's checkbox).
    pub(super) tuning_open: bool,
    /// **Quantos pixels de canvas medem uma célula de fluido** (1..=30, o
    /// slider no topo da seção; ver [`grid_map`]). `1` é a grade de sempre.
    ///
    /// ⚠️ **Autorado, e trocá-lo ENCERRA a sessão** — a grade tem dimensão, e
    /// uma sessão viva já a tem congelada em `WetSession::ratio`. Encerrar é o
    /// BAKE (a tinta que se vê já está no `canvas_rgba`), então nada é perdido;
    /// a alternativa — reamostrar catorze planos de `f32` para a resolução
    /// nova — inventaria água que o solver não produziu, e faria da razão um
    /// parâmetro que altera o passado em vez do futuro.
    pub(super) grid_ratio: u8,
    /// Quantas células de FLUIDO medem uma célula de FLUXO (plano 30). `1` = a
    /// grade de fluxo É a fina, que é o motor que sempre shipou.
    pub(super) flow_ratio: u8,
}

impl Default for WetPaintState {
    fn default() -> Self {
        Self {
            armed: false,
            session: None,
            live_gesture: false,
            knobs: WetKnobs::default(),
            pending_deposit: Vec::new(),
            deposit_pass: false,
            tool: WetTool::default(),
            tilt_on: true,
            tilt_ring: 4,
            tilt_spoke: 3,
            show_wet: false,
            paper_visual: false,
            km_mixing: false,
            km_glaze: false,
            tuning_open: false,
            grid_ratio: grid_map::DEFAULT_RATIO,
            flow_ratio: 1,
        }
    }
}

mod dab_route; // a ROTA DO DAB (pincel -> fluido) — filho por LOC
pub(super) mod grid_map; // a grade do fluido != a grade de PIXELS — filho por LOC
mod session; // what a wet SESSION is (data model) — child file (LOC cap)
use session::{Lane, PaperKey};
pub(super) use session::{WetEngineFacts, WetSession};

impl PainterTool {
    /// Whether the WET module owns this batch of dabs — the ONE routing
    /// question, asked by BOTH `stamp_dabs` (to bypass the snapshot/restore
    /// wrapper, which would kill the session — see W2.5) and
    /// `stamp_dabs_inner` (to enter the wet arm). Two copies of this
    /// condition would diverge, and the diverged half is a canvas gate that
    /// either leaks or kills the water.
    ///
    /// Wet Paint owns (doc 21): the INCREMENTAL methods' live batches and the
    /// commit door's `deposit_pass` replay — nothing else. A non-incremental
    /// AUTHORING batch (DragDot / Anchored / the shape editors' re-stamps) is
    /// deliberately UN-owned: it falls through the completely normal flat
    /// pipeline (snapshot/restore wrapper + colour routes), which is what
    /// makes the flat preview — and hands it Selection/protection/alpha-lock
    /// byte-identically to Paint mode. The eraser with no live session (W2.6)
    /// also falls through (it erases the BAKED canvas, which is what is
    /// visibly there). Non-wet modes: the mode conjunct short-circuits first —
    /// not one instruction changes (gate G0a).
    pub(super) fn wet_owns_the_dabs(&self) -> bool {
        matches!(self.paint.paint_mode, PaintMode::WetPaint)
            && (self.paint.brush.stroke_method.is_incremental() || self.paint.wetpaint.deposit_pass)
            && (!self.paint.eraser || self.paint.wetpaint.session.is_some())
    }

    /// Pen-up: close the engine's direct stroke (the sim resumes). Called from
    /// `paint_end`; the session itself stays alive — the water is still wet.
    pub(super) fn wetpaint_stroke_end(&mut self) {
        self.paint.wetpaint.live_gesture = false;
        if let Some(sess) = self.paint.wetpaint.session.as_mut()
            && sess.stroke_open
        {
            sess.bring_home();
            sess.engine.end_direct_stroke();
            sess.stroke_open = false;
            sess.lanes.clear();
        }
    }

    /// Per-frame heartbeat: **MOSTRA o que o worker já simulou** e devolve o
    /// motor a ele. No session = a true no-op (the OFF contract: not one byte is
    /// looked at).
    ///
    /// ⚠️ **O frame não dá passo nenhum** — a sim vive em [`offthread`], no
    /// relógio DELA. O tick faz três coisas: pergunta ao contador se há passo
    /// novo (atômico, sem trazer o motor), composita se houver, e entrega o
    /// motor de volta. O `dt` do shell deixou de ser entrada da água: era ele
    /// que fechava o laço de realimentação da era do agendamento (doc 28 §5.31).
    pub(super) fn wetpaint_tick(&mut self, _dt_s: f32) {
        // Doc 21 §F layer 2: the deposit flag must never survive into a tick.
        debug_assert!(!self.paint.wetpaint.deposit_pass);
        if self.paint.wetpaint.session.is_none() {
            return;
        }
        self.wetpaint_guard();
        // Doc 21 law D: the water FREEZES while the artist authors a flat
        // preview (flow AND drying) — zero steps, zero composites, zero knob
        // reconciles; a composite would copy `sess.base` verbatim where the
        // pigment is empty and ERASE the preview inside its dirty rect (a
        // torn state). The guard above still ran: foreign swaps (undo, layer
        // switch) kill the session even mid-authoring. The hold is DERIVED
        // (`drag_preview` is the flat re-stamp record in this mode) — no
        // state, no release door; commit/cancel drop the record and the
        // next tick simply resumes.
        if self.wet_authoring_hold() {
            return;
        }
        let facts = self.wet_facts();
        let Some(sess) = self.paint.wetpaint.session.as_mut() else {
            return;
        };
        // ── Há algo novo a mostrar? ─────────────────────────────────────────
        //
        // Perguntado ao CONTADOR de passos do worker (um atômico), NUNCA
        // trazendo o motor para casa: sem isto o tick buscaria o engine a cada
        // frame só para descobrir que nada mudou, e o worker perderia ~30% do
        // núcleo esperando na fronteira de estágio.
        let done = sess.sim_steps();
        let fresh = done != sess.seen_steps;
        // Facts land while the water SITS too (Dry Speed / Gravity / the
        // tilt are sim forces): the tick reconciles with the stamp's door.
        let facts_moved = sess.applied != facts;
        if fresh || facts_moved {
            // ⚠️ **PEDE e não espera.** O tick é a única porta que pode voltar
            // de mãos vazias (a água corre a ~33 Hz, o display a 60 ⇒ mostrar
            // um passo no frame seguinte é invisível), e bloquear aqui custava
            // o estágio inteiro DENTRO do frame — medido, 60,6 ms de pior tick
            // na poça de três traços. O `seen_steps` só anda quando de fato
            // compositamos, então nada é perdido: o frame seguinte tenta.
            //
            // ⚠️ **Sem `return` no ramo de falha, e a razão é o `hand_off_sim`
            // abaixo:** quando o pedido falha o motor está **no canal** — nem
            // aqui, nem com o worker —, e cair no hand-off (que é no-op ali)
            // mantém a estrutura honesta. ⚠️ **Isto NÃO é o que custou 60× de
            // taxa**, embora eu tenha escrito isso antes de reler a medição: o
            // colapso 33,4 → 0,5 Hz foi o `want` sobrevivendo à entrega
            // (`hand_off_sim`), e consertar o `return` não mudou um Hz. As duas
            // mutações estão documentadas em `offthread_tests.rs`.
            let t_wait = std::time::Instant::now();
            let got = sess.try_bring_home();
            crate::wet_diag::note_wait(t_wait.elapsed().as_secs_f32() * 1e3);
            if got {
                sess.seen_steps = done;
                sess.reconcile_facts(facts);
                if fresh {
                    let t_comp = std::time::Instant::now();
                    self.wetpaint_composite();
                    // Diagnóstico (`PH2D_FLUID_PROFILE`): a metade COMPOSITE do
                    // tick. Com a sim fora da thread, ela é a ÚNICA metade que o
                    // frame ainda paga.
                    crate::wet_diag::note_composite(t_comp.elapsed().as_secs_f32() * 1e3);
                }
            }
        }
        // ⚠️ **O hand-off é INCONDICIONAL** (no-op se o motor já está lá), e não
        // um `else` do bloco acima: gateá-lo em "houve trabalho" deixaria uma
        // sessão cujos facts nascem iguais ao boot com o motor em casa para
        // sempre — água que nunca simula, com todos os gates verdes.
        if let Some(sess) = self.paint.wetpaint.session.as_mut() {
            sess.hand_off_sim();
        }
    }

    /// The canvas-identity guard (module doc): a foreign `canvas_rgba` swap
    /// ends the session before anything composites over restored pixels.
    ///
    /// The comparison is `Weak::as_ptr` against `Arc::as_ptr` — see
    /// [`WetSession::canvas`] for why the token is weak, and why comparing its
    /// address is sound *because* it is weak (the handle pins the allocation,
    /// so the address it names can never be re-issued to a different canvas).
    fn wetpaint_guard(&mut self) {
        if let Some(sess) = &self.paint.wetpaint.session
            && !std::ptr::eq(sess.canvas.as_ptr(), Arc::as_ptr(&self.canvas_rgba))
        {
            self.paint.wetpaint.session = None;
        }
    }
}

mod authored_actions; // canvas actions + session birth + facts — child file (LOC cap)
mod composite;
mod offthread; // a sim FORA da thread do frame (o slot + o worker) — filho por LOC // the composite half (visual terms + veil) — child file (LOC cap)

#[cfg(test)]
#[path = "wetpaint/grid_ratio_tests.rs"]
mod grid_ratio_tests; // os gates de PRODUTO da razão da grade

#[cfg(test)]
#[path = "wetpaint/flow_ratio_tests.rs"]
mod flow_ratio_tests; // os gates de PRODUTO da razão da grade de FLUXO
#[cfg(test)]
#[path = "wetpaint/offthread_tests.rs"]
mod offthread_tests; // os gates da sim FORA da thread do frame
#[cfg(test)]
mod tests; // the W1/W2 gates — child file (workspace file-LOC cap)
#[cfg(test)]
mod tests_doc22; // the doc-22 gates (tuning/tilt/tools/actions/flags)
#[cfg(test)]
#[path = "wetpaint/undo_drip_tests.rs"]
mod undo_drip_tests; // o escorrido que sobrou do Undo (smoke do Enio, 2026-07-26)

/// Porta de MEDIÇÃO para o composite (`super` é privado ao módulo `paint`, e
/// as sondas moram em `paint::tests`): a metade não-sim do tick, cronometrável
/// sem re-implementar nada.
#[cfg(test)]
pub(in crate::tool::paint) fn composite_for_measure(t: &mut PainterTool) {
    t.wetpaint_composite();
}
