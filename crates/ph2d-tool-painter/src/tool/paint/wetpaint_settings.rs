//! The Wet Paint **authored settings** — the checkbox (the persistent ARM), the FULL knob
//! table (doc 22: the W3 curated seven grew into the engine's whole registry, one storage),
//! the wet TOOL, the tilt dial, the canvas-action/overlay flags, and their panel routing.
//! Split from `wetpaint.rs` (workspace file-LOC cap), the exact sibling pattern of
//! `watercolor_settings.rs`: this file owns the AUTHORED state and its doors; `wetpaint.rs`
//! owns the SESSION (display-state) that reconciles against it.

use super::*;
use ph2d_wet_paint::tuning::{KNOB_COUNT, KNOB_DEFS, Knob, KnobGroup, knob_defaults};

/// The Wet Paint knob store — the engine's TWO slider knobs plus the whole
/// `KNOB_DEFS` registry, in ONE house (doc 22 §2.1: the basic section's
/// curated rows and the Tuning panel's table rows are two VIEWS of the same
/// value; a second store would be the two-doors disease). Authored,
/// persistent tool state — the session is display-state and dies on any
/// foreign mutation; knobs living in the engine would forget themselves on
/// every undo. The session **reconciles** the engine against this every
/// batch and tick, the same law as the paper.
///
/// `f64` on purpose — the engine's own precision, so [`WetKnobs::DEFAULT`]
/// can equal the engine's boot values EXACTLY (an `f32` copy of `0.4` is
/// `0.4000000059604645` in `f64`, and the reconcile would push that noise
/// into untouched knobs the first time any other knob moves). The panel's
/// `f32` rows convert at the paint boundary.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WetKnobs {
    /// Water per dab (engine `sliders.water`, `0..1`).
    pub water: f64,
    /// The wet eraser's lift per pass (engine `sliders.erase`, `0..1`).
    pub erase: f64,
    /// Every registry knob, indexed by `Knob as usize` (the table order).
    /// Values are pre-sanitized by [`WetKnobs::set`] (NaN falls back to the
    /// default, out-of-range clamps to the def's own bounds).
    pub knobs: [f64; KNOB_COUNT],
}

/// [`WetKnobs::DEFAULT`]'s knob array — the reference boot with Enio's five Wet Paint product tweaks
/// (2026-07-22). A `const fn` so `DEFAULT` stays a `const` (the panel's `FALLBACK_BRUSH` needs it).
/// Each override is inside its `KNOB_DEFS` range, so no clamp is skipped by writing the array directly.
const fn painter_default_knobs() -> [f64; KNOB_COUNT] {
    let mut k = knob_defaults();
    k[Knob::PigmentPerDab as usize] = 800.0; // LITERAL-OK: wet-paint product default (Enio), engine boot 600
    k[Knob::PaperGate as usize] = 0.4; // LITERAL-OK: wet-paint product default (Enio), engine boot 0.6
    k[Knob::Felt as usize] = 0.03; // LITERAL-OK: wet-paint product default (Enio), engine boot 0.01
    k[Knob::BristleSize as usize] = 2.0; // LITERAL-OK: wet-paint product default (Enio), engine boot 1.0
    k[Knob::BristleCount as usize] = 2000.0; // LITERAL-OK: wet-paint product default (Enio), engine boot 950
    k
}

impl WetKnobs {
    /// The reference model's boot — SPEC §16 (`Sliders::default` + `KNOB_DEFS`), the values the ENGINE
    /// actually boots with (`Engine::new`). This is the reconcile's **no-op baseline**: a session's
    /// `applied` field inits to this (via `WetEngineFacts::BOOT`), so the first reconcile measures the
    /// tool's authored delta against the engine's real starting state. It is NOT the app default — see
    /// [`Self::DEFAULT`] — and the two MUST stay distinct, or the reconcile's early-return would skip
    /// the product values (panel showing 800 over an engine still at 600).
    pub const ENGINE_BOOT: Self = Self {
        water: 1.0,
        erase: 0.4,
        knobs: knob_defaults(),
    };

    /// The app's Wet Paint **default** — [`Self::ENGINE_BOOT`] with Enio's five product tweaks
    /// (2026-07-22: Pigment 800, Paper Gate 0.4, Felt 0.03, Bristle Size 2.0, Bristle Count 2000).
    /// Distinct from the reference boot ON PURPOSE: the engine stays a faithful 1:1 port of the JS
    /// reference (its fingerprint untouched, its `KNOB_DEFS` intact), and the painter opens the fluid on
    /// values chosen for the tool. The session's reconcile pushes these into the engine at birth, because
    /// they differ from the boot baseline it inits `applied` to. A `const` so the panel's
    /// `FALLBACK_BRUSH` (a `const` item) can carry it.
    pub const DEFAULT: Self = Self {
        water: 1.0,
        erase: 0.4,
        knobs: painter_default_knobs(),
    };

    /// Read one registry knob.
    #[must_use]
    pub fn get(&self, knob: Knob) -> f64 {
        self.knobs[knob as usize]
    }

    /// Write one registry knob: NaN (a garbled numeric input) falls back to
    /// the def's default — the model's own law — and everything else clamps
    /// to the def's bounds (const literals, table-order gated in the engine).
    pub fn set(&mut self, knob: Knob, v: f64) {
        let def = &KNOB_DEFS[knob as usize];
        self.knobs[knob as usize] = if v.is_nan() {
            // NaN (garbled input) falls back to the app DEFAULT, not the engine's SPEC boot — "default"
            // means the product value the artist opened with (== SPEC for every knob but Enio's five).
            Self::DEFAULT.knobs[knob as usize]
        } else {
            v.clamp(def.min, def.max)
        };
    }

    /// Restore one GROUP to the app DEFAULTS (the Tuning panel's header reset). Reads [`Self::DEFAULT`],
    /// not `KNOB_DEFS`, so a reset lands on the same values the section opened with — the product tweaks
    /// included (for every knob but Enio's five, `DEFAULT` and `KNOB_DEFS` agree, so this is a no-op change).
    pub fn reset_group(&mut self, group: KnobGroup) {
        for def in KNOB_DEFS.iter().filter(|d| d.group == group) {
            self.knobs[def.knob as usize] = Self::DEFAULT.knobs[def.knob as usize];
        }
    }

    // The W3 curated accessors — the basic section's five knob rows read the
    // SAME array the Tuning panel edits.
    /// Pigment per dab (`Knob::PigmentPerDab`).
    #[must_use]
    pub fn pigment(&self) -> f64 {
        self.get(Knob::PigmentPerDab)
    }
    /// Settled-paint pickup — the dirty brush (`Knob::Pickup`).
    #[must_use]
    pub fn pickup(&self) -> f64 {
        self.get(Knob::Pickup)
    }
    /// Evaporation multiplier — how fast the water dries (`Knob::Evaporation`).
    #[must_use]
    pub fn dry_speed(&self) -> f64 {
        self.get(Knob::Evaporation)
    }
    /// Edge darkening — the watercolor rim (`Knob::EdgeDarkening`).
    #[must_use]
    pub fn edge_darkening(&self) -> f64 {
        self.get(Knob::EdgeDarkening)
    }
    /// Gravity magnitude — the run/drip pull (`Knob::Gravity`).
    #[must_use]
    pub fn gravity(&self) -> f64 {
        self.get(Knob::Gravity)
    }
}

impl Default for WetKnobs {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// The wet TOOL (doc 22 §2.5 — the model's 7-button radio, minus Erase,
/// which is the rail eraser's other view). Selecting one USES it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum WetTool {
    #[default]
    Paint,
    Smear,
    Blend,
    Wet,
    Dry,
    Blow,
}

impl WetTool {
    /// The engine-side tool this maps to.
    pub(crate) fn engine(self) -> ph2d_wet_paint::painter::Tool {
        use ph2d_wet_paint::painter::Tool;
        match self {
            WetTool::Paint => Tool::Paint,
            WetTool::Smear => Tool::Smear,
            WetTool::Blend => Tool::Blend,
            WetTool::Wet => Tool::Wet,
            WetTool::Dry => Tool::Dry,
            WetTool::Blow => Tool::Blow,
        }
    }

    /// Paint and Blend deposit through the LANE trails (one per symmetry
    /// copy); the rest are per-dab grid ops through the direct-tool door.
    pub(crate) fn uses_lanes(self) -> bool {
        matches!(self, WetTool::Paint | WetTool::Blend)
    }
}

impl PainterTool {
    /// The Wet Paint checkbox's setter (the panel's Enable + the smoke's
    /// arm). Arming while holding the plain Brush enters the mode on the
    /// spot; disarming while wet exits to the plain Brush — and the exit's
    /// teardown ends the session, which IS the bake. From any other tool
    /// the flag just flips: the next `"brush"` honours it.
    pub fn set_wetpaint_armed(&mut self, on: bool) {
        if self.paint.wetpaint.armed == on {
            return;
        }
        self.paint.wetpaint.armed = on;
        match self.paint.paint_mode {
            PaintMode::Paint if on && !self.paint.eraser => self.set_paint_tool_mode("brush"),
            PaintMode::WetPaint if !on => self.set_paint_tool_mode("brush"),
            _ => {}
        }
    }

    /// Flip the arm. The panel reaches it through `set_paint_media` (the Paint Mode dropdown);
    /// the section reset and the smoke call it directly.
    pub fn toggle_wetpaint_armed(&mut self) {
        self.set_wetpaint_armed(!self.paint.wetpaint.armed);
    }

    /// The section reset — the Watercolor reset's exact semantics: restore
    /// the section's defaults INCLUDING the enable (disarming bakes the live
    /// water), the knob table, the tool, the tilt and the overlay flags
    /// (the reconcile carries the values into any live session's engine).
    pub fn reset_brush_wetpaint(&mut self) {
        let w = &mut self.paint.wetpaint;
        w.knobs = WetKnobs::default();
        w.tool = WetTool::default();
        w.tilt_on = true;
        w.tilt_ring = 4;
        w.tilt_spoke = 3;
        w.km_mixing = false;
        let need_recomposite = w.show_wet || w.paper_visual || w.km_glaze;
        w.show_wet = false;
        w.paper_visual = false;
        w.km_glaze = false;
        w.tuning_open = false;
        if need_recomposite {
            self.wet_recomposite_full();
        }
        self.set_wetpaint_armed(false);
    }

    /// The authored knob values (the panel snapshot's source).
    #[must_use]
    pub fn wet_knobs(&self) -> WetKnobs {
        self.paint.wetpaint.knobs
    }

    /// Pick a wet tool by its button index (the model's radio order:
    /// Paint · Erase · Smear · Blend · Wet · Dry · Blow). Picking USES the
    /// tool: Erase is the rail eraser's other view (the impasto tool-list
    /// precedent — two views of one radio), everything else lands on the
    /// brush wire, which resolves to the fluid while armed.
    pub fn pick_wet_tool(&mut self, index: usize) {
        let tool = match index {
            0 => WetTool::Paint,
            1 => {
                self.set_paint_tool_mode("eraser");
                return;
            }
            2 => WetTool::Smear,
            3 => WetTool::Blend,
            4 => WetTool::Wet,
            5 => WetTool::Dry,
            _ => WetTool::Blow,
        };
        self.paint.wetpaint.tool = tool;
        self.set_paint_tool_mode("brush");
    }

    /// **A razão da grade do fluido** (1..=30 px por célula) — a porta ÚNICA.
    ///
    /// ⚠️ **Trocar a razão encerra a sessão de água viva**, e isso é o desenho,
    /// não uma limitação: a grade tem dimensão, e uma sessão viva já a tem
    /// congelada em `WetSession::ratio`. Encerrar É o bake (a tinta que se vê
    /// está no `canvas_rgba`), então o artista não perde pintura — perde a água
    /// AINDA MOLHADA, que é exactamente o que "mudar a resolução do fluido"
    /// significa. A alternativa seria reamostrar catorze planos de `f32` para a
    /// resolução nova, inventando água que o solver nunca produziu.
    ///
    /// Sem mudança = **sem encerramento** (o guard de igualdade é o que torna
    /// seguro o chip numérico re-emitir o mesmo valor a cada frame de arrasto).
    pub fn set_wet_grid_ratio(&mut self, v: f64) {
        use super::wetpaint::grid_map;
        let want = grid_map::clamp_ratio(v.round().clamp(0.0, 255.0) as u8);
        if self.paint.wetpaint.grid_ratio == want {
            return;
        }
        self.paint.wetpaint.grid_ratio = want;
        // A sessão viva nasceu com a razão antiga; a próxima nasce com esta.
        self.wetpaint_end_session();
    }

    /// Route one W3/tilt `SetValue` to its clamped field. Knob ranges are
    /// the engine's own (`KNOB_DEFS`); the two engine sliders are `0..1`.
    fn set_wet_knob_value(&mut self, id: ph2d_a11y::NodeId, v: f64) -> bool {
        use ph2d_editor_core::ids as core_ids;
        // ⚠️ A razão da grade vem ANTES do empréstimo dos knobs: ela não é um
        // knob do motor (o motor nem sabe que ela existe — ver
        // `wetpaint::grid_map`), e o setter dela ENCERRA a sessão, o que precisa
        // de `self` inteiro.
        if id == core_ids::PAINTER_WETPAINT_GRID {
            self.set_wet_grid_ratio(v);
            return true;
        }
        let w = &mut self.paint.wetpaint;
        let k = &mut w.knobs;
        match id {
            x if x == core_ids::PAINTER_WETPAINT_WATER => k.water = v.clamp(0.0, 1.0),
            x if x == core_ids::PAINTER_WETPAINT_PIGMENT => k.set(Knob::PigmentPerDab, v),
            x if x == core_ids::PAINTER_WETPAINT_PICKUP => k.set(Knob::Pickup, v),
            x if x == core_ids::PAINTER_WETPAINT_DRY_SPEED => k.set(Knob::Evaporation, v),
            x if x == core_ids::PAINTER_WETPAINT_EDGE => k.set(Knob::EdgeDarkening, v),
            x if x == core_ids::PAINTER_WETPAINT_GRAVITY => k.set(Knob::Gravity, v),
            x if x == core_ids::PAINTER_WETPAINT_ERASE => k.erase = v.clamp(0.0, 1.0),
            // The tilt dial's two carriers (the pad converts its 2D drag to
            // ring/spoke and forwards them here). Touching the dial turns
            // the tilt ON — the model's "dragging the knob implies on".
            x if x == core_ids::PAINTER_WETPAINT_TILT_RING => {
                w.tilt_ring = (v.round().clamp(0.0, 8.0)) as u8;
                w.tilt_on = true;
            }
            x if x == core_ids::PAINTER_WETPAINT_TILT_SPOKE => {
                w.tilt_spoke = (v.round().rem_euclid(12.0)) as u8;
                w.tilt_on = true;
            }
            _ => return false,
        }
        true
    }

    /// Route the Wet Paint section controls (Enable + reset + tools + tilt +
    /// canvas actions + overlay checkboxes + the Tuning side panel's table)
    /// from the layers panel's generic channel — sibling of
    /// `route_brush_watercolor_event`.
    pub(crate) fn route_brush_wetpaint_event(
        &mut self,
        event: &ph2d_editor_core::tool::PanelEvent,
    ) -> bool {
        use ph2d_editor_core::ids as core_ids;
        use ph2d_editor_core::tool::PanelEvent;
        match event {
            PanelEvent::Click(id) if *id == core_ids::PAINTER_WETPAINT_RESET => {
                self.reset_brush_wetpaint();
                true
            }
            PanelEvent::Click(id) if core_ids::PAINTER_WETPAINT_TOOL_IDS.contains(id) => {
                let index = core_ids::PAINTER_WETPAINT_TOOL_IDS
                    .iter()
                    .position(|t| t == id)
                    .unwrap_or(0);
                self.pick_wet_tool(index);
                true
            }
            PanelEvent::Click(id) if *id == core_ids::PAINTER_WETPAINT_TILT_TOGGLE => {
                // Flips WITHOUT losing the dial's direction (the model).
                self.paint.wetpaint.tilt_on = !self.paint.wetpaint.tilt_on;
                true
            }
            PanelEvent::Click(id) if *id == core_ids::PAINTER_WETPAINT_WETCANVAS => {
                self.wetpaint_wet_canvas();
                true
            }
            PanelEvent::Click(id) if *id == core_ids::PAINTER_WETPAINT_DRYCANVAS => {
                self.wetpaint_dry_canvas();
                true
            }
            PanelEvent::Click(id) if *id == core_ids::PAINTER_WETPAINT_FASTDRY => {
                self.wetpaint_fast_dry();
                true
            }
            PanelEvent::Click(id) if *id == core_ids::PAINTER_WETPAINT_SHOWWET => {
                let on = !self.paint.wetpaint.show_wet;
                self.paint.wetpaint.show_wet = on;
                self.wet_recomposite_full();
                true
            }
            PanelEvent::Click(id)
                if *id == core_ids::PAINTER_WETPAINT_PAPER_VISUAL
                    || *id == core_ids::WET_TUNING_PAPER_EYE =>
            {
                // One fact, two views: the basic checkbox and the Tuning
                // panel's PAPER eye.
                self.paint.wetpaint.paper_visual = !self.paint.wetpaint.paper_visual;
                self.wet_recomposite_full();
                true
            }
            PanelEvent::Click(id) if *id == core_ids::PAINTER_WETPAINT_TUNING => {
                self.paint.wetpaint.tuning_open = !self.paint.wetpaint.tuning_open;
                true
            }
            PanelEvent::Click(id) if *id == core_ids::WET_TUNING_KM_MIXING => {
                // Sim-side: changes how FUTURE mixing happens; nothing on
                // screen moves until paint does.
                self.paint.wetpaint.km_mixing = !self.paint.wetpaint.km_mixing;
                true
            }
            PanelEvent::Click(id) if *id == core_ids::WET_TUNING_KM_GLAZE => {
                self.paint.wetpaint.km_glaze = !self.paint.wetpaint.km_glaze;
                self.wet_recomposite_full();
                true
            }
            PanelEvent::Click(id) => self.route_wet_tuning_click(*id),
            PanelEvent::SetValue(id, v) => {
                self.set_wet_knob_value(*id, *v) || self.route_wet_tuning_set(*id, *v)
            }
            _ => false,
        }
    }

    /// The Tuning side panel's per-knob RESET / per-group RESET clicks
    /// (dynamic id family — resolved through [`wet_tuning_id_map`]).
    fn route_wet_tuning_click(&mut self, id: ph2d_a11y::NodeId) -> bool {
        use ph2d_editor_core::ids as core_ids;
        if let Some(gi) = core_ids::WET_TUNING_GROUP_RESETS
            .iter()
            .position(|g| *g == id)
        {
            let group = [
                KnobGroup::Paint,
                KnobGroup::Water,
                KnobGroup::Physics,
                KnobGroup::Tools,
                KnobGroup::Paper,
            ][gi];
            self.paint.wetpaint.knobs.reset_group(group);
            return true;
        }
        if let Some(&(idx, kind)) = wet_tuning_id_map().get(&id)
            && kind == TuneWidget::Reset
        {
            let def = &KNOB_DEFS[idx];
            self.paint.wetpaint.knobs.set(def.knob, def.default);
            return true;
        }
        false
    }

    /// The Tuning side panel's knob `SetValue`s (slider or chip commit).
    fn route_wet_tuning_set(&mut self, id: ph2d_a11y::NodeId, v: f64) -> bool {
        if let Some(&(idx, kind)) = wet_tuning_id_map().get(&id)
            && kind != TuneWidget::Reset
        {
            self.paint.wetpaint.knobs.set(KNOB_DEFS[idx].knob, v);
            return true;
        }
        false
    }
}

/// Which face of a Tuning-panel knob row an id belongs to.
#[derive(Clone, Copy, PartialEq, Eq)]
enum TuneWidget {
    Slider,
    Chip,
    Reset,
}

/// The dynamic id family, resolved ONCE: every VISIBLE knob's slider / chip /
/// reset id (the hidden §17 group has no UI, exactly like the model). Built
/// from the same `KNOB_DEFS` + id-derive functions the panel paints from —
/// no table to drift.
fn wet_tuning_id_map() -> &'static std::collections::BTreeMap<ph2d_a11y::NodeId, (usize, TuneWidget)>
{
    use ph2d_editor_core::ids as core_ids;
    static MAP: std::sync::OnceLock<
        std::collections::BTreeMap<ph2d_a11y::NodeId, (usize, TuneWidget)>,
    > = std::sync::OnceLock::new();
    MAP.get_or_init(|| {
        let mut m = std::collections::BTreeMap::new();
        for (i, def) in KNOB_DEFS.iter().enumerate() {
            if def.group == KnobGroup::Hidden {
                continue;
            }
            m.insert(
                core_ids::wet_tuning_slider_id(def.key),
                (i, TuneWidget::Slider),
            );
            m.insert(core_ids::wet_tuning_chip_id(def.key), (i, TuneWidget::Chip));
            m.insert(
                core_ids::wet_tuning_reset_id(def.key),
                (i, TuneWidget::Reset),
            );
        }
        m
    })
}
