//! **The row table — the single source every other module reads.**
//!
//! A world setting shows up in four places: it is painted, it is registered so
//! the click is not dropped, it is turned back into a value when the artist
//! drags it, and it is swept by the seam test. Four hand-written lists drift,
//! and the drift is silent — a row that is painted but not registered is dead
//! on click, and a row nobody swept is a knob nobody proved.
//!
//! So there is ONE list. `paint`, `populate`, `event` and `tests/seam.rs` all
//! iterate [`ROWS`]; adding a knob is adding a row, and it is wired, focusable,
//! live and swept by construction.
//!
//! This is also the answer to "the fullest card rots"
//! ([[feedback_the_fullest_card_premise_rots]]): the sweep does not pick a
//! representative section, it asks every row in the table.

use ph2d_a11y::NodeId;
use ph2d_editor_core::ids;
use ph2d_physics_ecs::{
    GRAVITY_LIMIT, MAX_AIR_DRAG, MAX_CONTACT_HZ, MAX_DAMPING, MAX_SLEEP_THRESHOLD,
    MAX_SOLVER_ITERATIONS, MAX_SUBSTEPS, MAX_TIME_UNTIL_SLEEP, MIN_CONTACT_HZ, PhysicsSettings,
};

/// One slider+chip row: which setting it edits, and the range it edits it over.
pub struct Row {
    /// i18n key for the label.
    pub label: &'static str,
    /// Slider track id.
    pub slider: NodeId,
    /// Linked value chip id.
    pub chip: NodeId,
    /// Domain minimum (the value at track 0).
    pub min: f32,
    /// Domain maximum (the value at track 1).
    pub max: f32,
    /// Drag-scrub step for the chip. ⚠️ Not decoration: without a registered
    /// range+step the chip derives its step from the buffer text and scrubs
    /// ~50 units per PIXEL, which turns it into a min↔max switch (the bug the
    /// Flip panel documented — typing always worked, only dragging was broken).
    pub step: f64,
    /// How many decimals the readout shows. Integer knobs show none, because a
    /// "4.0" sub-step invites a 4.5 that does not exist.
    pub decimals: usize,
    /// Read this row's value out of the settings.
    pub get: fn(&PhysicsSettings) -> f32,
    /// Write this row's value into the settings.
    pub set: fn(&mut PhysicsSettings, f32),
}

impl Row {
    /// Slider track (`0..=1`) → the value it means.
    pub fn value_of(&self, track: f32) -> f32 {
        self.min + track * (self.max - self.min)
    }

    /// The value → its slider track. The inverse of [`Row::value_of`], and they
    /// have to stay inverses: the panel publishes a track from the settings
    /// every frame and reads a value back out of the track on every drag, so a
    /// mismatch is a control that creeps while you hold it
    /// ([[feedback_derived_coordinate_seed_must_match_sample]]).
    pub fn track_of(&self, value: f32) -> f32 {
        if self.max <= self.min {
            return 0.0;
        }
        ((value - self.min) / (self.max - self.min)).clamp(0.0, 1.0)
    }

    /// `link_slider_number_mapped` expresses the same mapping as
    /// `display = track * scale + offset`.
    pub fn scale(&self) -> f32 {
        self.max - self.min
    }

    /// See [`Row::scale`].
    pub fn offset(&self) -> f32 {
        self.min
    }
}

/// A titled group of rows. The section list IS the paint order.
pub struct Section {
    /// Collapsible header id.
    pub id: NodeId,
    /// i18n key for the header.
    pub title: &'static str,
    /// The rows under it.
    pub rows: &'static [Row],
}

/// World: the two gravity axes.
static WORLD: &[Row] = &[
    Row {
        label: "panel.physics.gravity_x",
        slider: ids::PHYSICS_GRAVITY_X,
        chip: ids::PHYSICS_GRAVITY_X_NUM,
        min: -GRAVITY_LIMIT,
        max: GRAVITY_LIMIT,
        step: 0.1, // LITERAL-PX-OK: drag step in m/s^2 (physical unit, not a design metric)
        decimals: 2,
        get: |s| s.gravity_x,
        set: |s, v| s.gravity_x = v,
    },
    Row {
        label: "panel.physics.gravity_y",
        slider: ids::PHYSICS_GRAVITY_Y,
        chip: ids::PHYSICS_GRAVITY_Y_NUM,
        min: -GRAVITY_LIMIT,
        max: GRAVITY_LIMIT,
        step: 0.1, // LITERAL-PX-OK: drag step in m/s^2 (physical unit, not a design metric)
        decimals: 2,
        get: |s| s.gravity_y,
        set: |s, v| s.gravity_y = v,
    },
];

/// Solver: the two knobs W2a measured against the interpenetration artifact,
/// plus iteration count.
static SOLVER: &[Row] = &[
    Row {
        label: "panel.physics.substeps",
        slider: ids::PHYSICS_SUBSTEPS,
        chip: ids::PHYSICS_SUBSTEPS_NUM,
        min: 1.0,
        max: MAX_SUBSTEPS as f32,
        step: 1.0,
        decimals: 0,
        get: |s| s.substeps as f32,
        set: |s, v| s.substeps = v.round().max(1.0) as u32,
    },
    Row {
        label: "panel.physics.iterations",
        slider: ids::PHYSICS_ITERATIONS,
        chip: ids::PHYSICS_ITERATIONS_NUM,
        min: 1.0,
        max: MAX_SOLVER_ITERATIONS as f32,
        step: 1.0,
        decimals: 0,
        get: |s| s.solver_iterations as f32,
        set: |s, v| s.solver_iterations = v.round().max(1.0) as u32,
    },
    Row {
        label: "panel.physics.contact_hz",
        slider: ids::PHYSICS_CONTACT_HZ,
        chip: ids::PHYSICS_CONTACT_HZ_NUM,
        min: MIN_CONTACT_HZ,
        max: MAX_CONTACT_HZ,
        step: 1.0,
        decimals: 0,
        get: |s| s.contact_hz,
        set: |s, v| s.contact_hz = v,
    },
];

/// Air drag — the SIZE-AWARE model (`F = k·L·|v|·v`, so `a ∝ v²/s`).
static AIR: &[Row] = &[Row {
    label: "panel.physics.air_drag",
    slider: ids::PHYSICS_AIR_DRAG,
    chip: ids::PHYSICS_AIR_DRAG_NUM,
    min: 0.0,
    max: MAX_AIR_DRAG,
    step: 0.05, // LITERAL-PX-OK: drag step of a dimensionless physical knob, not a design metric
    decimals: 2,
    get: |s| s.air_drag,
    set: |s, v| s.air_drag = v,
}];

/// Uniform damping — mass and size cannot enter it. A different model from
/// [`AIR`], not a tuning of it.
static DAMPING: &[Row] = &[
    Row {
        label: "panel.physics.linear_damping",
        slider: ids::PHYSICS_LINEAR_DAMPING,
        chip: ids::PHYSICS_LINEAR_DAMPING_NUM,
        min: 0.0,
        max: MAX_DAMPING,
        step: 0.05, // LITERAL-PX-OK: drag step of a dimensionless physical knob, not a design metric
        decimals: 2,
        get: |s| s.linear_damping,
        set: |s, v| s.linear_damping = v,
    },
    Row {
        label: "panel.physics.angular_damping",
        slider: ids::PHYSICS_ANGULAR_DAMPING,
        chip: ids::PHYSICS_ANGULAR_DAMPING_NUM,
        min: 0.0,
        max: MAX_DAMPING,
        step: 0.05, // LITERAL-PX-OK: drag step of a dimensionless physical knob, not a design metric
        decimals: 2,
        get: |s| s.angular_damping,
        set: |s, v| s.angular_damping = v,
    },
];

/// When a body is allowed to stop being simulated.
static SLEEP: &[Row] = &[
    Row {
        label: "panel.physics.sleep_speed",
        slider: ids::PHYSICS_SLEEP_SPEED,
        chip: ids::PHYSICS_SLEEP_SPEED_NUM,
        min: 0.0,
        max: MAX_SLEEP_THRESHOLD,
        step: 0.05, // LITERAL-PX-OK: drag step of a dimensionless physical knob, not a design metric
        decimals: 2,
        get: |s| s.sleep_linear_threshold,
        set: |s, v| s.sleep_linear_threshold = v,
    },
    Row {
        label: "panel.physics.sleep_spin",
        slider: ids::PHYSICS_SLEEP_SPIN,
        chip: ids::PHYSICS_SLEEP_SPIN_NUM,
        min: 0.0,
        max: MAX_SLEEP_THRESHOLD,
        step: 0.05, // LITERAL-PX-OK: drag step of a dimensionless physical knob, not a design metric
        decimals: 2,
        get: |s| s.sleep_angular_threshold,
        set: |s, v| s.sleep_angular_threshold = v,
    },
    Row {
        label: "panel.physics.sleep_delay",
        slider: ids::PHYSICS_SLEEP_DELAY,
        chip: ids::PHYSICS_SLEEP_DELAY_NUM,
        min: 0.0,
        max: MAX_TIME_UNTIL_SLEEP,
        step: 0.1, // LITERAL-PX-OK: drag step in seconds (physical unit, not a design metric)
        decimals: 2,
        get: |s| s.time_until_sleep,
        set: |s, v| s.time_until_sleep = v,
    },
];

/// Every section, in paint order.
pub static SECTIONS: &[Section] = &[
    Section {
        id: ids::PHYSICS_SEC_WORLD,
        title: "panel.physics.section.world",
        rows: WORLD,
    },
    Section {
        id: ids::PHYSICS_SEC_SOLVER,
        title: "panel.physics.section.solver",
        rows: SOLVER,
    },
    Section {
        id: ids::PHYSICS_SEC_AIR,
        title: "panel.physics.section.air",
        rows: AIR,
    },
    Section {
        id: ids::PHYSICS_SEC_DAMPING,
        title: "panel.physics.section.damping",
        rows: DAMPING,
    },
    Section {
        id: ids::PHYSICS_SEC_SLEEP,
        title: "panel.physics.section.sleep",
        rows: SLEEP,
    },
];

/// Every row, flattened — what `populate`, `event` and the seam sweep walk.
pub fn rows() -> impl Iterator<Item = &'static Row> {
    SECTIONS.iter().flat_map(|s| s.rows.iter())
}

/// The row a widget id belongs to, if any (the slider or its chip).
pub fn row_for(id: NodeId) -> Option<&'static Row> {
    rows().find(|r| r.slider == id || r.chip == id)
}
