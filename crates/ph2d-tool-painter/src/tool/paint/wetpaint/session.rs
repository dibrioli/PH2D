//! **What a wet SESSION is** — the data model behind [`PaintMode::WetPaint`],
//! split from `wetpaint.rs` for the workspace file-LOC cap. The parent keeps
//! what the tool DOES to a session (the dab route, the stroke lifecycle, the
//! identity guard); this file is what a session IS and how it reconciles with
//! the authored facts.

use super::*;

/// Everything the session pushes into the ENGINE (doc 22) — compared as one
/// value per batch/tick; the boot constant equals the engine's own boot (gate-pinned).
#[derive(Clone, Copy, PartialEq)]
pub(in crate::tool::paint) struct WetEngineFacts {
    pub(in crate::tool::paint) knobs: WetKnobs,
    pub(in crate::tool::paint) tilt: (bool, u8, u8),
    pub(in crate::tool::paint) km_mixing: bool,
}

impl WetEngineFacts {
    /// The session's `applied` init — the engine's REAL boot (`WetKnobs::ENGINE_BOOT` == `Engine::new`),
    /// NOT the tool's product default (`WetKnobs::DEFAULT`). The first reconcile then sees the authored
    /// knobs differ from this baseline and pushes them; collapsing the two would skip the product values.
    pub(in crate::tool::paint) const BOOT: Self = Self {
        knobs: WetKnobs::ENGINE_BOOT,
        tilt: (true, 4, 3),
        km_mixing: false,
    };
}

pub(in crate::tool::paint) struct WetSession {
    /// O motor (grid + sim + tuning), do tamanho da tela da camada — e **onde
    /// ele está**: na thread do frame ou com o worker da sim
    /// ([`super::offthread`]). Pergunte por [`Self::eng`], nunca pelo slot.
    pub(in crate::tool::paint) engine: EngineSlot,
    /// A thread da sim, criada no primeiro `hand_off_sim`. `None` = o motor
    /// nunca saiu daqui (sessão recém-nascida, ou um teste que só dirige).
    pub(in crate::tool::paint) worker: Option<SimWorker>,
    /// Quantos passos do worker este tool já COMPOSITOU. É o que responde
    /// *"há algo novo na tela?"* sem trazer o motor para casa — sem isto o tick
    /// buscaria o engine a cada frame e o worker perderia o núcleo esperando.
    pub(in crate::tool::paint) seen_steps: u64,
    /// The canvas frozen at session start — every composite renders over THIS.
    /// `pub(super)` so [`PainterTool::wet_splat_gates`] can serve it as the
    /// alpha-lock's α reference (the frozen base the composite reads).
    pub(in crate::tool::paint) base: Arc<Vec<u8>>,
    /// The exact `canvas_rgba` allocation OUR last composite produced (or the
    /// one the session started from). A mismatch = foreign mutation = session
    /// over. `pub(super)` so the doc-21 commit door can re-arm it after an
    /// OWNED authoring write (`wetpaint_commit::wetpaint_rearm_after_own_write`).
    ///
    /// ⚠️ **`Weak`, never `Arc` — the guard asks about IDENTITY, not ownership,
    /// and a second strong handle is not free.** `wetpaint_composite` ends in
    /// `Arc::make_mut(&mut self.canvas_rgba)`, which hands back the slice only
    /// to a SOLE owner and **copies the whole canvas** otherwise: measured,
    /// this handle turned every pointer move into a full document copy —
    /// **9.86 ms at 4096²**, the one cost that made the Wet Paint move scale
    /// with the canvas while the other three media stayed flat. A `Weak` costs
    /// **0.0000 ms** (`make_mut` moves the 24-byte `Vec` header to a fresh
    /// allocation and leaves the pixels where they are) — measured, not cited.
    ///
    /// And it is the *right* handle for a second reason: a `Weak` keeps the
    /// allocation alive, so no later `Arc` can be born at that address — the
    /// ABA that makes a raw-pointer identity check unsound (the ADR-0124
    /// lesson: **ask the value, never `as_ptr()` of something you don't pin**).
    pub(in crate::tool::paint) canvas: Weak<Vec<u8>>,
    /// Session-persistent pigment scratch (`w*h*4`); the region render fully
    /// overwrites the rect it is asked for, so stale bytes outside are inert.
    pub(super) pigment: Vec<u8>,
    /// Per-LANE stroke state (one lane per symmetry copy / tile offset,
    /// matched geometrically): last dab centre (the chord source) + the last
    /// fresh-ink colour sent (Randomize change detector). Cleared per stroke.
    pub(super) lanes: Vec<Lane>,
    /// A direct stroke is open in the engine (pen-down .. pen-up).
    pub(super) stroke_open: bool,
    /// What the engine's PAPER plane was last seeded from: `None` = the
    /// engine's own preset tile (session birth / paper disarmed); `Some` =
    /// the artist's Paper slot, keyed by everything the seed reads. The
    /// session **reconciles** against this every dab batch — paper is
    /// substrate under live water, but the SLOT is authored state, and the
    /// session spans strokes: seeding only at session birth left every later
    /// adjustment (Brightness/Contrast, any knob, the arm itself) silently
    /// ignored until the bake (Enio 2026-07-21, "os ajustes ... não estão
    /// sendo levados em consideração").
    pub(super) paper_key: Option<PaperKey>,
    /// The [`WetEngineFacts`] last pushed into the engine — starts at the
    /// boot constant (the engine boots with exactly it, gate-pinned) and the
    /// reconcile pushes the authored values when they differ. Same law as
    /// `paper_key`: the facts are authored state, the session is not.
    pub(super) applied: WetEngineFacts,
}

/// Everything the paper seed reads — the reconcile re-seeds exactly when one
/// of these moves, and an unchanged key costs one small struct compare per
/// batch. The image rides by its monotonic VERSION, never the `Arc` address
/// (an address survives a content swap and a content survives an address
/// swap — the ADR-0124 cache-identity trap).
#[derive(Clone, Copy, PartialEq)]
pub(super) struct PaperKey {
    pub(super) tex: ph2d_painter_brush::TextureSettings,
    pub(super) image_version: u64,
    pub(super) period: [f32; 2],
}

impl WetSession {
    /// Push the authored FACTS into the engine when they moved (W3, grown by
    /// doc 22). ONE door for the stamp AND the tick — Dry Speed and Gravity
    /// act on water already sitting on the canvas, so a knob must land while
    /// the sim runs, not only at the next stroke. Unchanged facts cost one
    /// struct compare. The tuning knobs go through `Engine::set_knob` (the
    /// door that reacts to what a knob invalidates: bristle rebuild lazy,
    /// paper re-bake, repaint) — and the paper re-bake only fires while the
    /// ENGINE's own tile is the tooth source (an armed artist Paper slot
    /// overrides it; its three physical knobs hide in the panel then).
    pub(super) fn reconcile_facts(&mut self, f: WetEngineFacts) {
        use ph2d_wet_paint::tuning::KNOB_DEFS;
        if self.applied == f {
            return;
        }
        let a = self.applied;
        let own_paper = self.paper_key.is_none();
        self.bring_home();
        let e = &mut *self.engine;
        if f.knobs.water != a.knobs.water {
            e.sliders.water = f.knobs.water;
        }
        if f.knobs.erase != a.knobs.erase {
            e.sliders.erase = f.knobs.erase;
        }
        let moved = KNOB_DEFS.iter().zip(&f.knobs.knobs).zip(&a.knobs.knobs);
        for ((def, new), old) in moved {
            if new != old {
                e.set_knob(def.knob, *new);
            }
        }
        if f.tilt != a.tilt {
            let (on, ring, spoke) = f.tilt;
            let d = ph2d_wet_paint::sim::tilt_dir_for_spoke(spoke);
            e.sim.tilt_on = on;
            e.sim.tilt_dir_x = d[0];
            e.sim.tilt_dir_y = d[1];
            e.sim.tilt_scale = f64::from(ring) / 4.0;
        }
        if f.km_mixing != a.km_mixing {
            e.sim.km_mixing = f.km_mixing;
        }
        if own_paper && e.paper_dirty() {
            e.rebake_paper();
        }
        self.applied = f;
    }
}

/// One replication lane of the open stroke — see [`WetSession::lanes`].
pub(super) struct Lane {
    pub(super) pos: [f32; 2],
    pub(super) ink: [f32; 3],
}
