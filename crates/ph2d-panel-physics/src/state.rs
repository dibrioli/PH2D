//! Panel state: the snapshot the shell publishes each frame, and the intents
//! the panel emits back.
//!
//! ## Why intents and not `ToolPanelEvent`
//!
//! Every other docked panel forwards to a **tool** (`EditorAction::ToolPanelEvent`
//! → `tool.handle_panel_event`). Physics is the world/scene-settings category
//! (ADR-0131 D8) — there is no tool to forward to, and inventing one so the
//! existing pipe fits would be a tool that is not a tool.
//!
//! So this follows the other panel that edits a document rather than a tool:
//! the panel pushes intents onto a queue and the shell's bridge drains them
//! (`ph2d-panel-motion-graph`'s `drain_intents`, and the timeline's
//! `TimelineIntent`). The shell stays the only thing that touches the
//! `PhysicsBridge`.

use ph2d_physics_ecs::{InteractionSettings, PhysicsSettings};
use std::cell::{Cell, RefCell};

thread_local! {
    /// Live snapshot published by the host before each `paint`. `None` until
    /// the first push — the panel then paints the engine defaults, which is
    /// what a world with no physics in it would report anyway.
    static CURRENT: RefCell<Option<PhysicsSnapshot>> = const { RefCell::new(None) };
    /// Edits waiting for the shell to drain them.
    static INTENTS: RefCell<Vec<PhysicsIntent>> = const { RefCell::new(Vec::new()) };
    static LAST_CONTENT_H: Cell<f32> = const { Cell::new(0.0) };
    static LAST_VISIBLE_H: Cell<f32> = const { Cell::new(0.0) };
    /// ⭐⭐ **A LENTE DO PINTOR** — todo cabeçalho de secção que o último `paint` desenhou.
    ///
    /// Não é conveniência de teste: é a metade do censo que nenhuma lista escrita à mão pode
    /// dar. O dreno de dobra tem a sua lente (`rows::is_section_header`); esta é a do pintor, e
    /// o gate `every_painted_section_header_folds` (tests/seam.rs) morre quando as duas
    /// divergirem — que é exactamente como o `PHYSICS_SEC_LAYERS` viveu morto.
    ///
    /// ⚠️ Escrita pelo `paint::body::header`, o **único** sítio deste painel que desenha um
    /// `SectionHeader`, e limpa no início de cada `paint`: uma lente que acumula quadros
    /// responderia por um painel que já não está na tela.
    static PAINTED_SECTION_HEADERS: RefCell<Vec<ph2d_a11y::NodeId>> =
        const { RefCell::new(Vec::new()) };
}

/// What the panel needs to know about the world this frame.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct PhysicsSnapshot {
    /// The world's authored settings — what every row reads.
    pub settings: PhysicsSettings,
    /// `ProjectSettings::pixels_per_meter`, shown as a readout.
    ///
    /// ⚠️ **Displayed, never owned** (ADR-0131 D4): the world scale is a
    /// PROJECT setting, and a second door onto it would diverge. It is here so
    /// the metre-valued rows can be read in pixels, not so they can be edited.
    pub pixels_per_meter: f32,
    /// The shell's `App.show_colliders` — the same flag the `B` key toggles.
    /// The panel reflects it; it does not keep its own.
    pub show_colliders: bool,
    /// How many physics bodies are live, for the readout. Zero is worth seeing:
    /// it is the difference between "gravity is wrong" and "nothing in this
    /// scene has a body yet".
    pub body_count: usize,
    /// **The interaction tool** (W-Hand) — what the pointer does to a running
    /// scene.
    ///
    /// ⚠️ It rides the same snapshot as the world settings but it is NOT one of
    /// them: it lives in the shell's `App` and is never persisted (see
    /// `ph2d_physics_ecs::interaction`). Two different lifetimes, one pipe — which
    /// is why the panel emits a SEPARATE intent for it rather than folding it into
    /// `SetSettings`.
    pub interaction: InteractionSettings,
    /// **Quantos segundos de CORRIDA GRAVADA o documento carrega** (W25).
    ///
    /// ⚠️ **A fita é um fato do DOCUMENTO e este painel é o painel do
    /// documento** — a §14 do Inspector mostra o mesmo número, mas ela só existe
    /// enquanto houver um corpo Dynamic selecionado, e uma corrida sobrevive ao
    /// player que a gravou (ela viaja no arquivo). Sem esta vista, apagar o
    /// personagem deixava a corrida **inalcançável**.
    ///
    /// Zero é a resposta *"ninguém correu"*, e é ela que apaga o botão.
    pub recorded_run_seconds: f32,
    /// Quantos segundos de corrida DESCARTADA esperam por um desfazer (W25).
    ///
    /// Sessão apenas, nunca no arquivo — a mesma regra da §14, e a mesma razão:
    /// uma corrida descartada foi descartada.
    pub discarded_run_seconds: f32,
}

impl Default for PhysicsSnapshot {
    fn default() -> Self {
        Self {
            settings: PhysicsSettings::default(),
            // Read from the setting's OWNER, not transcribed: this is the
            // scale a project has before anyone changes it, and a literal here
            // would be a second copy of it free to drift.
            pixels_per_meter: ph2d_editor_core::project::ProjectSettings::default()
                .pixels_per_meter,
            show_colliders: true,
            body_count: 0,
            interaction: InteractionSettings::default(),
            recorded_run_seconds: 0.0,
            discarded_run_seconds: 0.0,
        }
    }
}

/// An edit the artist made, for the shell to apply.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum PhysicsIntent {
    /// Replace the world settings wholesale.
    ///
    /// One intent for all ten knobs, not ten intents: the panel already holds a
    /// complete `PhysicsSettings` (the snapshot) and edits one field of it, so
    /// per-knob variants would be ten ways to say the same thing and ten places
    /// for the shell to forget one.
    SetSettings(PhysicsSettings),
    /// Flip the collider overlay — routed to the shell's `App.show_colliders`,
    /// the flag the `B` key owns.
    ToggleColliders,
    /// Replace the interaction-tool state wholesale (W-Hand).
    ///
    /// A SECOND intent rather than a field of `SetSettings`, because the two have
    /// different lifetimes: the world settings travel in the project file and this
    /// does not. Folding them would make every knob drag of a runtime tool look
    /// like a document edit to whatever the shell does with `SetSettings` next.
    SetInteraction(InteractionSettings),
    /// **Descartar a corrida gravada** (W25) — a vista de mundo do
    /// `PlayerFieldEdit::ClearRun`.
    ///
    /// ⚠️ **Um intent, e não uma escrita:** a fita mora na shell (`App`), fora
    /// do `PhysicsBridge` e fora de `PhysicsSettings`. Enfiá-la no
    /// `SetSettings` a faria viajar no arquivo de settings de mundo — e ela já
    /// viaja no seu próprio campo do `ProjectFile`, o que seriam duas cópias da
    /// mesma corrida.
    ClearRun,
    /// **Devolver a corrida descartada** (W25) — o irmão exato do acima.
    RestoreRun,
}

/// Retained per-instance state. Empty on purpose: the authority is the shell's
/// `PhysicsBridge`, and the panel renders the per-frame snapshot.
#[derive(Clone, Debug, Default)]
pub struct PhysicsPanelState;

/// Host → panel, once per frame before `paint`.
pub fn set_current_physics(snapshot: Option<PhysicsSnapshot>) {
    CURRENT.with(|c| *c.borrow_mut() = snapshot);
}

/// What `paint` and `event` read.
pub(crate) fn current() -> PhysicsSnapshot {
    CURRENT.with(|c| c.borrow().unwrap_or_default())
}

/// Panel → host. Queued by `event`, drained by the shell's bridge.
pub(crate) fn push_intent(intent: PhysicsIntent) {
    INTENTS.with(|c| c.borrow_mut().push(intent));
}

/// Take everything the artist did since the last frame.
pub fn drain_intents() -> Vec<PhysicsIntent> {
    INTENTS.with(|c| std::mem::take(&mut *c.borrow_mut()))
}

/// Scroll bookkeeping (the shell's dock needs the measured heights).
pub fn last_content_h() -> f32 {
    LAST_CONTENT_H.with(|c| c.get())
}

/// See [`last_content_h`].
pub fn last_visible_h() -> f32 {
    LAST_VISIBLE_H.with(|c| c.get())
}

/// Every section header the last [`crate::paint`] drew, in paint order — see
/// [`PAINTED_SECTION_HEADERS`].
pub fn last_painted_section_headers() -> Vec<ph2d_a11y::NodeId> {
    PAINTED_SECTION_HEADERS.with(|c| c.borrow().clone())
}

/// Start a fresh paint's ledger. Called once, at the top of `paint`.
pub(crate) fn begin_painted_section_headers() {
    PAINTED_SECTION_HEADERS.with(|c| c.borrow_mut().clear());
}

/// Record one header the painter just drew.
pub(crate) fn note_painted_section_header(id: ph2d_a11y::NodeId) {
    PAINTED_SECTION_HEADERS.with(|c| c.borrow_mut().push(id));
}

pub(crate) fn set_last_content_h(v: f32) {
    LAST_CONTENT_H.with(|c| c.set(v));
}

pub(crate) fn set_last_visible_h(v: f32) {
    LAST_VISIBLE_H.with(|c| c.set(v));
}
