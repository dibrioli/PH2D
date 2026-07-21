//! Engine facade (port of `painter.js`): everything a shell (or the
//! acceptance suite) drives. Owns the layers (up to 4 grids sharing one
//! paper), the per-layer history rings, the stroke engine + trail, dab
//! parameterization (SPEC §9), tool dispatch, canvas-wide actions, and the
//! render dirty-rect bookkeeping.

use crate::brush::{BristleKnobs, BrushShape, build_bristle_texture};
use crate::grid::{
    Grid, GridSnapshot, clear_canvas, dry_canvas, restore_grid, snapshot_grid, wet_canvas,
};
use crate::jsmath::clamp01;
use crate::paper::{PaperKnobs, PaperPreset, bake_paper, generate_paper_tile};
use crate::sim::{Sim, sim_fast_dry, sim_step};
use crate::stroke::{Stroke, StrokeEvent};
use crate::tools::{TOOL_HARDNESS, apply_blow, apply_dry, apply_erase, apply_smear, apply_wet};
use crate::trail::{Dab, TouchedRect, Trail, TrailMode};
use crate::tuning::{Knob, Rebuild, Tuning};

pub const MAX_LAYERS: usize = 4;
pub const HISTORY_DEPTH: usize = 6;

pub const NAMED_PIGMENTS: [(&str, [f64; 3]); 8] = [
    ("Hansa Yellow", [250.0, 205.0, 40.0]),
    ("Pyrrole Red", [220.0, 45.0, 40.0]),
    ("Quinacridone Rose", [220.0, 55.0, 120.0]),
    ("Phthalo Blue", [10.0, 70.0, 150.0]),
    ("Ultramarine", [45.0, 70.0, 165.0]),
    ("Viridian", [20.0, 120.0, 100.0]),
    ("Burnt Sienna", [150.0, 75.0, 40.0]),
    ("Payne's Grey", [55.0, 65.0, 85.0]),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tool {
    Paint,
    Erase,
    Smear,
    Blend,
    Wet,
    Dry,
    Blow,
}

/// The four user-facing brush sliders (boot defaults, SPEC §16).
#[derive(Clone, Copy)]
pub struct Sliders {
    pub size: f64,
    pub pressure: f64,
    pub water: f64,
    pub erase: f64,
}

impl Default for Sliders {
    fn default() -> Self {
        Sliders { size: 0.7, pressure: 0.7, water: 1.0, erase: 0.4 }
    }
}

pub struct Layer {
    pub grid: Grid,
    pub opacity: f64,
    pub visible: bool,
    undo: Vec<GridSnapshot>,
    redo: Vec<GridSnapshot>,
}

/// Pending render invalidation (cell coords, [1..W] x [1..H]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dirty {
    Clean,
    Full,
    Rect { x0: i32, y0: i32, x1: i32, y1: i32 },
}

pub struct Engine {
    pub width: usize,
    pub height: usize,
    pub tuning: Tuning,
    pub sim: Sim,
    pub layers: Vec<Layer>,
    pub active_layer: usize,
    pub paper_preset: PaperPreset,
    pub paper_sheet: u32,
    pub paper_tile: Vec<f32>,
    paper_dirty: bool,
    brush_tex: Option<Vec<f32>>,
    pub trail: Trail,
    pub stroke: Stroke,
    stroke_events: Vec<StrokeEvent>,
    // Tool state (boot defaults, SPEC §16).
    pub tool: Tool,
    pub shape: BrushShape,
    pub sliders: Sliders,
    pub color: [f64; 3],
    /// Render-side experimental checkbox (K–M glaze stacking).
    pub km_glaze: bool,
    pub show_wet: bool,
    // Stroke bookkeeping.
    stroke_down: bool,
    has_prev_dab: bool,
    prev_dab_x: f64,
    prev_dab_y: f64,
    stroke_tool_backup: Option<Tool>,
    /// Test hook: (pressure, dab) per dispatched dab.
    pub on_dab: Option<Box<dyn FnMut(f64, &Dab)>>,
    pub dirty: Dirty,
}

impl Engine {
    pub fn new(width: usize, height: usize) -> Self {
        let mut engine = Engine {
            width,
            height,
            tuning: Tuning::default(),
            sim: Sim::default(),
            layers: Vec::new(),
            active_layer: 0,
            paper_preset: PaperPreset::Cold,
            paper_sheet: 0,
            paper_tile: Vec::new(),
            paper_dirty: false,
            brush_tex: None,
            trail: Trail::default(),
            stroke: Stroke::default(),
            stroke_events: Vec::new(),
            tool: Tool::Paint,
            shape: BrushShape::Round,
            sliders: Sliders::default(),
            color: [50.0, 140.0, 210.0],
            km_glaze: false,
            show_wet: false,
            stroke_down: false,
            has_prev_dab: false,
            prev_dab_x: 0.0,
            prev_dab_y: 0.0,
            stroke_tool_backup: None,
            on_dab: None,
            dirty: Dirty::Clean,
        };
        engine.layers.push(Layer {
            grid: Grid::new(width, height),
            opacity: 1.0,
            visible: true,
            undo: Vec::new(),
            redo: Vec::new(),
        });
        engine.rebake_paper();
        engine.brush_tex = Some(build_bristle_texture(engine.bristle_knobs()));
        engine
    }

    pub fn active_grid(&self) -> &Grid {
        &self.layers[self.active_layer].grid
    }

    pub fn active_grid_mut(&mut self) -> &mut Grid {
        &mut self.layers[self.active_layer].grid
    }

    fn bristle_knobs(&self) -> BristleKnobs {
        BristleKnobs {
            felt: self.tuning.get(Knob::Felt),
            bristle_count: self.tuning.get(Knob::BristleCount),
            bristle_strength: self.tuning.get(Knob::BristleStrength),
            bristle_size: self.tuning.get(Knob::BristleSize),
        }
    }

    fn paper_knobs(&self) -> PaperKnobs {
        PaperKnobs {
            contrast: self.tuning.get(Knob::PaperContrast),
            fibres: self.tuning.get(Knob::PaperFibres),
            grooves: self.tuning.get(Knob::PaperGrooves),
        }
    }

    /// Write a knob and react to what it invalidates (the JS onChange).
    pub fn set_knob(&mut self, knob: Knob, v: f64) {
        if let Some(def) = self.tuning.set(knob, v) {
            self.react_to_knob_change(def);
        }
    }

    fn react_to_knob_change(&mut self, def: &'static crate::tuning::KnobDef) {
        match def.rebuild {
            Some(Rebuild::Brush) => self.brush_tex = None, // lazy rebuild
            Some(Rebuild::Paper) => self.paper_dirty = true, // re-bake on release
            Some(Rebuild::Render) => self.mark_dirty_full(),
            None => {}
        }
    }

    /// Reset a whole knob group, reacting to every changed knob exactly as
    /// [`Engine::set_knob`] would — the JS fires onChange per knob from
    /// resetGroup; dropping those left a stale brush texture / unbaked paper
    /// (port-verify finding). The panel's group-reset button goes HERE.
    pub fn reset_knob_group(&mut self, group: crate::tuning::KnobGroup) {
        for def in self.tuning.reset_group(group) {
            self.react_to_knob_change(def);
        }
    }

    /// The shell calls this on slider release when a paper knob changed.
    pub fn paper_dirty(&self) -> bool {
        self.paper_dirty
    }

    fn texture(&mut self) -> &[f32] {
        if self.brush_tex.is_none() {
            self.brush_tex = Some(build_bristle_texture(self.bristle_knobs()));
        }
        self.brush_tex.as_deref().unwrap()
    }

    pub fn rebake_paper(&mut self) {
        self.paper_tile =
            generate_paper_tile(self.paper_preset, self.paper_sheet, self.paper_knobs());
        for l in &mut self.layers {
            bake_paper(&mut l.grid, &self.paper_tile);
            l.grid.paper_preset = self.paper_preset;
            l.grid.paper_sheet = self.paper_sheet;
        }
        self.paper_dirty = false;
        self.mark_dirty_full();
    }

    pub fn set_paper(&mut self, preset: PaperPreset, sheet: Option<u32>) {
        self.paper_preset = preset;
        if let Some(s) = sheet {
            self.paper_sheet = s & 3; // 4 sheet indices
        }
        self.rebake_paper();
    }

    // -----------------------------------------------------------------------
    // Dab parameters (SPEC §9) + dispatch
    // -----------------------------------------------------------------------

    fn dispatch_dab(&mut self, x: f64, y: f64, b: f64, dir_x: f64, dir_y: f64) {
        let p = self.sim.gather_params(&self.tuning);
        let s = self.sliders;
        let pressure_base = 0.2 + 0.7 * s.pressure; // even minimum pressure deposits
        let mut intensity = b * 0.5 * pressure_base;
        intensity = intensity.clamp(0.0, 3.0);
        intensity *= p.k(Knob::Intensity); // knob applied post-clamp
        let water_unit = s.water * 0.35;
        // Pressure-INVERSE water: a light touch is wetter, a heavy stroke
        // denser.
        let water_amount =
            (2.0 * water_unit * water_unit) / (pressure_base + 0.01) * p.k(Knob::WaterPerDab);
        let r = 2.0 + s.size * 33.0;
        let mut h = 0.5 * b;
        if h < 1.0 {
            h = 1.0;
        } else if h > 3.0 {
            h = 3.0;
        }
        if h > r / 2.0 {
            h = r / 2.0;
        }
        let hardness = h * h; // slow => 9 crisp flat-top, fast => ~2 soft puff
        let dry_gate = clamp01(1.0 - water_amount / 1.5) * clamp01((11.0 - b - 4.0) / 4.0);
        let dab = Dab {
            x,
            y,
            r,
            hardness,
            intensity,
            water_amount,
            dry_gate,
            shape: self.shape,
            dir_x,
            dir_y,
        };
        if let Some(hook) = self.on_dab.as_mut() {
            hook(b, &dab);
        }

        let ext_bypass = self.sim.ext_bypass;
        // Materialize the (lazily invalidated) bristle texture, then take it
        // out for the duration of the dispatch — the trail/tools need it
        // alongside a mutable borrow of the layer grid.
        self.texture();
        let tex = self.brush_tex.take().unwrap();
        let layer = &mut self.layers[self.active_layer];
        let rect: Option<TouchedRect> = match self.tool {
            Tool::Paint => {
                if self.trail.accumulate_paint(&mut layer.grid, &p, &tex, &dab, ext_bypass) {
                    self.trail.transfer_paint(&mut layer.grid, &p)
                } else {
                    None
                }
            }
            Tool::Blend => {
                // Blend is a tool (fixed hardness 4.8, intensity clamped to 3)
                // that routes through the trail's saturating mask window.
                let bd = Dab { hardness: TOOL_HARDNESS, intensity: intensity.min(3.0), ..dab };
                if self.trail.accumulate_blend(&layer.grid, &p, &tex, &bd) {
                    self.trail.transfer_blend(&mut layer.grid, &p)
                } else {
                    None
                }
            }
            Tool::Erase => apply_erase(&mut layer.grid, &p, &tex, &dab, s.erase),
            Tool::Wet => apply_wet(&mut layer.grid, &p, &tex, &dab, water_unit),
            Tool::Dry => apply_dry(&mut layer.grid, &p, &tex, &dab),
            Tool::Blow => {
                let (px, py) =
                    if self.has_prev_dab { (self.prev_dab_x, self.prev_dab_y) } else { (x, y) };
                apply_blow(&mut layer.grid, &p, &tex, &dab, px, py)
            }
            Tool::Smear => {
                if self.has_prev_dab {
                    apply_smear(&mut layer.grid, &p, &tex, &dab, self.prev_dab_x, self.prev_dab_y)
                } else {
                    None // skip the stroke's first dab (no displacement)
                }
            }
        };
        self.brush_tex = Some(tex);
        self.prev_dab_x = x;
        self.prev_dab_y = y;
        self.has_prev_dab = true;
        if let Some(r) = rect {
            self.merge_dirty(r.x0, r.y0, r.x1, r.y1);
        }
    }

    /// Drain the stroke engine's per-tick events into trail/dab dispatch —
    /// the JS fires these as closures during the tick; the ORDER is
    /// preserved (segment before its dabs, windows roll mid-tick).
    fn drain_stroke_events(&mut self) {
        let mut events = std::mem::take(&mut self.stroke_events);
        for ev in events.drain(..) {
            match ev {
                StrokeEvent::Segment { chord } => {
                    if self.tool == Tool::Paint || self.tool == Tool::Blend {
                        let spacing = self.tuning.get(Knob::Spacing);
                        self.trail.on_segment(chord, spacing);
                    }
                }
                StrokeEvent::Dab { x, y, pressure, dir_x, dir_y } => {
                    self.dispatch_dab(x, y, pressure, dir_x, dir_y);
                }
            }
        }
        self.stroke_events = events; // hand the allocation back
    }

    // -----------------------------------------------------------------------
    // Stroke lifecycle — feed exactly one sample per 40 Hz sim step.
    // The sim is PAUSED while the pointer is down, except for the blow tool.
    // -----------------------------------------------------------------------

    pub fn pointer_down(&mut self, x: f64, y: f64, tool_override: Option<Tool>) {
        self.capture_history(); // BEFORE each mutation
        if let Some(t) = tool_override {
            self.stroke_tool_backup = Some(self.tool);
            self.tool = t;
        }
        self.stroke_down = true;
        self.has_prev_dab = false;
        let mode = if self.tool == Tool::Blend { TrailMode::Blend } else { TrailMode::Paint };
        self.trail.start_stroke(x, y, self.color, mode);
        let spacing = self.tuning.get(Knob::Spacing);
        self.stroke.begin(x, y, spacing);
    }

    /// One per 40 Hz step while down.
    pub fn pointer_frame(&mut self, x: f64, y: f64) {
        if !self.stroke_down {
            return;
        }
        let mut events = std::mem::take(&mut self.stroke_events);
        self.stroke.tick(x, y, &mut events);
        self.stroke_events = events;
        self.drain_stroke_events();
    }

    pub fn pointer_up(&mut self) {
        if !self.stroke_down {
            return;
        }
        self.stroke_down = false;
        self.stroke.release();
    }

    /// Tick the release tail once per 40 Hz step with the hover position.
    /// Returns false when the stroke has fully finished (remainder dropped).
    pub fn release_frame(&mut self, x: f64, y: f64) -> bool {
        if !self.stroke.active {
            return false;
        }
        let mut events = std::mem::take(&mut self.stroke_events);
        let more = self.stroke.release_tick(x, y, &mut events);
        self.stroke_events = events;
        self.drain_stroke_events();
        if !more {
            self.end_stroke();
        }
        more
    }

    fn end_stroke(&mut self) {
        let mut events = std::mem::take(&mut self.stroke_events);
        self.stroke.finish(&mut events);
        self.stroke_events = events;
        self.drain_stroke_events();
        self.trail.drop_remainder(); // remainder is dropped by design
        if let Some(t) = self.stroke_tool_backup.take() {
            self.tool = t;
        }
    }

    /// Immediate full stop (used by tests and tool switches mid-air).
    pub fn abort_stroke(&mut self) {
        self.stroke_down = false;
        self.end_stroke();
    }

    /// Should the fixed-step loop advance the sim right now?
    pub fn sim_should_run(&self) -> bool {
        !self.stroke_down || self.tool == Tool::Blow
    }

    pub fn step_simulation(&mut self) -> bool {
        let idx = self.active_layer;
        let had_fluid = self.layers[idx].grid.has_fluid;
        let pre = if had_fluid {
            let g = &self.layers[idx].grid;
            Some((g.bx0, g.by0, g.bx1, g.by1))
        } else {
            None
        };
        let ran = sim_step(&mut self.sim, &mut self.layers[idx].grid, &self.tuning);
        if ran || had_fluid {
            if let Some((x0, y0, x1, y1)) = pre {
                self.merge_dirty(x0, y0, x1, y1);
            }
            let g = &self.layers[idx].grid;
            if g.has_fluid {
                let (x0, y0, x1, y1) = (g.bx0, g.by0, g.bx1, g.by1);
                self.merge_dirty(x0, y0, x1, y1);
            }
        }
        ran
    }

    // -----------------------------------------------------------------------
    // Canvas-wide actions (SPEC §12) — each captures history first.
    // -----------------------------------------------------------------------

    pub fn action_wet_canvas(&mut self) {
        self.capture_history();
        wet_canvas(self.active_grid_mut());
        self.mark_dirty_full(); // show-wet overlay may change everywhere
    }

    pub fn action_dry_canvas(&mut self) {
        self.capture_history();
        let mix = self.sim.gather_params(&self.tuning).mix;
        let g = self.active_grid_mut();
        dry_canvas(g, mix);
        g.bloom.fill(0); // backrun budgets recharge on dry
        self.mark_dirty_full();
    }

    pub fn action_fast_dry(&mut self) {
        self.capture_history();
        let idx = self.active_layer;
        sim_fast_dry(&mut self.sim, &mut self.layers[idx].grid, &self.tuning);
        self.mark_dirty_full();
    }

    pub fn action_clear(&mut self) {
        self.capture_history();
        clear_canvas(self.active_grid_mut());
        self.mark_dirty_full();
    }

    // -----------------------------------------------------------------------
    // Layers (SPEC §15): up to 4, sharing one paper; only the active layer
    // simulates.
    // -----------------------------------------------------------------------

    pub fn add_layer(&mut self) -> bool {
        if self.layers.len() >= MAX_LAYERS {
            return false;
        }
        let mut g = Grid::new(self.width, self.height);
        bake_paper(&mut g, &self.paper_tile); // shared tooth (same bake)
        g.paper_preset = self.paper_preset;
        g.paper_sheet = self.paper_sheet;
        self.layers.push(Layer {
            grid: g,
            opacity: 1.0,
            visible: true,
            undo: Vec::new(),
            redo: Vec::new(),
        });
        self.select_layer(self.layers.len() - 1);
        true
    }

    pub fn remove_layer(&mut self, index: usize) -> bool {
        if self.layers.len() <= 1 || index >= self.layers.len() {
            return false;
        }
        self.layers.remove(index);
        if self.active_layer >= self.layers.len() {
            self.active_layer = self.layers.len() - 1;
        }
        self.select_layer(self.active_layer);
        self.mark_dirty_full();
        true
    }

    pub fn select_layer(&mut self, index: usize) {
        self.active_layer = index.min(self.layers.len() - 1);
        self.mark_dirty_full();
    }

    // -----------------------------------------------------------------------
    // History (SPEC §15): per-layer undo/redo ring, depth 6.
    // -----------------------------------------------------------------------

    pub fn capture_history(&mut self) {
        let layer = &mut self.layers[self.active_layer];
        layer.undo.push(snapshot_grid(&layer.grid));
        if layer.undo.len() > HISTORY_DEPTH {
            layer.undo.remove(0);
        }
        layer.redo.clear();
    }

    pub fn can_undo(&self) -> bool {
        !self.layers[self.active_layer].undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.layers[self.active_layer].redo.is_empty()
    }

    pub fn undo(&mut self) -> bool {
        let layer = &mut self.layers[self.active_layer];
        let Some(snap) = layer.undo.pop() else {
            return false;
        };
        layer.redo.push(snapshot_grid(&layer.grid));
        self.apply_snapshot(snap);
        true
    }

    pub fn redo(&mut self) -> bool {
        let layer = &mut self.layers[self.active_layer];
        let Some(snap) = layer.redo.pop() else {
            return false;
        };
        layer.undo.push(snapshot_grid(&layer.grid));
        self.apply_snapshot(snap);
        true
    }

    fn apply_snapshot(&mut self, snap: GridSnapshot) {
        let layer = &mut self.layers[self.active_layer];
        let paper_changed = restore_grid(&mut layer.grid, &snap);
        if paper_changed {
            self.paper_preset = layer.grid.paper_preset;
            self.paper_sheet = layer.grid.paper_sheet;
            self.rebake_paper();
        }
        self.mark_dirty_full();
    }

    // -----------------------------------------------------------------------
    // Dirty-rect bookkeeping (cell coords, [1..W] x [1..H])
    // -----------------------------------------------------------------------

    pub fn merge_dirty(&mut self, x0: i32, y0: i32, x1: i32, y1: i32) {
        if self.dirty == Dirty::Full {
            return;
        }
        let nx0 = (x0 - 1).max(1);
        let ny0 = (y0 - 1).max(1);
        let nx1 = (x1 + 1).min(self.width as i32);
        let ny1 = (y1 + 1).min(self.height as i32);
        match &mut self.dirty {
            Dirty::Clean => {
                self.dirty = Dirty::Rect { x0: nx0, y0: ny0, x1: nx1, y1: ny1 };
            }
            Dirty::Rect { x0, y0, x1, y1 } => {
                if nx0 < *x0 {
                    *x0 = nx0;
                }
                if ny0 < *y0 {
                    *y0 = ny0;
                }
                if nx1 > *x1 {
                    *x1 = nx1;
                }
                if ny1 > *y1 {
                    *y1 = ny1;
                }
            }
            Dirty::Full => {}
        }
    }

    pub fn mark_dirty_full(&mut self) {
        self.dirty = Dirty::Full;
    }

    /// Take and clear the pending dirty region.
    pub fn take_dirty(&mut self) -> Dirty {
        std::mem::replace(&mut self.dirty, Dirty::Clean)
    }
}
