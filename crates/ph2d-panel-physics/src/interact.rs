//! **A tabela da seção INTERACTION** (W-Hand) — irmã de [`crate::rows`], e
//! separada dela por uma razão e não por tamanho: aquela edita `PhysicsSettings`
//! (o MUNDO, persistido no arquivo) e esta edita `InteractionSettings` (o
//! PONTEIRO, runtime-only). Duas structs, duas tabelas; uma só teria de carregar
//! um `get`/`set` que soubesse das duas.
//!
//! A lei de [`crate::rows`] vale igual: **UMA lista, quatro consumidores**
//! (`paint`, `populate`, `event`, o sweep de seam). Um knob é uma row, e nasce
//! pintado, registrado, vivo e varrido.
//!
//! ⚠️ **Uma row tem um `shown`**, e é a diferença estrutural em relação às rows de
//! mundo: os knobs desta seção pertencem a UMA ferramenta, e pintar a rigidez da
//! mola enquanto o artista segura a explosão seria um controle que o solver
//! ignora. O `shown` é a mesma pergunta que o `HoldMode::uses_spring_gains`
//! responde do lado do modelo — o painel pergunta para OFERECER, o wrapper para
//! HONRAR.

use ph2d_a11y::NodeId;
use ph2d_editor_core::ids;
use ph2d_physics_ecs::{
    HoldMode, InteractionSettings, InteractionTool, MAX_ATTRACT_FORCE, MAX_BLAST_IMPULSE,
    MAX_HOLD_DAMPING_RATIO, MAX_HOLD_STIFFNESS, MIN_HOLD_STIFFNESS, WORLD_REACH_M,
};

/// One slider+chip row of the Interaction section. Same shape as
/// [`crate::rows::Row`] plus the `shown` predicate.
pub struct IRow {
    /// i18n key for the label.
    pub label: &'static str,
    /// Slider track id.
    pub slider: NodeId,
    /// Linked value chip id.
    pub chip: NodeId,
    pub min: f32,
    pub max: f32,
    /// Drag-scrub step for the chip. ⚠️ Registered, and not decoration: without
    /// a range+step the chip derives its step from the buffer text and scrubs
    /// ~50 units per PIXEL, turning it into a min↔max switch.
    pub step: f64,
    pub decimals: usize,
    pub get: fn(&InteractionSettings) -> f32,
    pub set: fn(&mut InteractionSettings, f32),
    /// Is this row live for the tool/mode currently in hand?
    pub shown: fn(&InteractionSettings) -> bool,
}

impl IRow {
    /// Slider track (`0..=1`) → the value it means.
    pub fn value_of(&self, track: f32) -> f32 {
        self.min + track * (self.max - self.min)
    }

    /// The value → its slider track. Inverse of [`IRow::value_of`], and they have
    /// to stay inverses: the panel publishes a track from the settings every frame
    /// and reads a value back on every drag, so a mismatch is a control that
    /// creeps while you hold it
    /// ([[feedback_derived_coordinate_seed_must_match_sample]]).
    pub fn track_of(&self, value: f32) -> f32 {
        if self.max <= self.min {
            return 0.0;
        }
        ((value - self.min) / (self.max - self.min)).clamp(0.0, 1.0)
    }

    pub fn scale(&self) -> f32 {
        self.max - self.min
    }

    pub fn offset(&self) -> f32 {
        self.min
    }
}

/// Every Interaction row, in paint order.
///
/// The order is the reading order of the section: what the tool is, then how it
/// holds, then the numbers of whichever tool is in hand.
pub static IROWS: &[IRow] = &[
    // ── The Hand: the spring's two gains ────────────────────────────────────
    IRow {
        label: "panel.physics.hold_stiffness",
        slider: ids::PHYSICS_HOLD_STIFFNESS,
        chip: ids::PHYSICS_HOLD_STIFFNESS_NUM,
        min: MIN_HOLD_STIFFNESS,
        max: MAX_HOLD_STIFFNESS,
        step: 10.0, // LITERAL-PX-OK: drag step in acceleration-per-metre (physical unit)
        decimals: 0,
        get: |s| s.stiffness,
        set: |s, v| s.stiffness = v,
        shown: |s| s.tool == InteractionTool::Hand && s.hold.uses_spring_gains(),
    },
    IRow {
        label: "panel.physics.hold_damping",
        slider: ids::PHYSICS_HOLD_DAMPING,
        chip: ids::PHYSICS_HOLD_DAMPING_NUM,
        min: 0.0,
        max: MAX_HOLD_DAMPING_RATIO,
        step: 0.05, // LITERAL-PX-OK: drag step of a dimensionless ratio, not a design metric
        decimals: 2,
        get: |s| s.damping_ratio,
        set: |s, v| s.damping_ratio = v,
        shown: |s| s.tool == InteractionTool::Hand && s.hold.uses_spring_gains(),
    },
    IRow {
        label: "panel.physics.hold_slack",
        slider: ids::PHYSICS_HOLD_SLACK,
        chip: ids::PHYSICS_HOLD_SLACK_NUM,
        min: 0.0,
        max: WORLD_REACH_M,
        step: 0.1, // LITERAL-PX-OK: drag step in metres (physical unit)
        decimals: 2,
        get: |s| s.slack,
        set: |s, v| s.slack = v,
        shown: |s| s.tool == InteractionTool::Hand && s.hold.uses_slack(),
    },
    // ── The Blast ───────────────────────────────────────────────────────────
    IRow {
        label: "panel.physics.blast_radius",
        slider: ids::PHYSICS_BLAST_RADIUS,
        chip: ids::PHYSICS_BLAST_RADIUS_NUM,
        min: 0.0,
        max: WORLD_REACH_M,
        step: 0.1, // LITERAL-PX-OK: drag step in metres (physical unit)
        decimals: 2,
        get: |s| s.blast_radius,
        set: |s, v| s.blast_radius = v,
        shown: |s| s.tool == InteractionTool::Explode,
    },
    IRow {
        label: "panel.physics.blast_force",
        slider: ids::PHYSICS_BLAST_FORCE,
        chip: ids::PHYSICS_BLAST_FORCE_NUM,
        min: 0.0,
        max: MAX_BLAST_IMPULSE,
        step: 0.5, // LITERAL-PX-OK: drag step in N*s (physical unit)
        decimals: 1,
        get: |s| s.blast_impulse,
        set: |s, v| s.blast_impulse = v,
        shown: |s| s.tool == InteractionTool::Explode,
    },
    // ── The Pull ────────────────────────────────────────────────────────────
    IRow {
        label: "panel.physics.pull_radius",
        slider: ids::PHYSICS_PULL_RADIUS,
        chip: ids::PHYSICS_PULL_RADIUS_NUM,
        min: 0.0,
        max: WORLD_REACH_M,
        step: 0.1, // LITERAL-PX-OK: drag step in metres (physical unit)
        decimals: 2,
        get: |s| s.attract_radius,
        set: |s, v| s.attract_radius = v,
        shown: |s| s.tool == InteractionTool::Attract,
    },
    // ⚠️ The only row of this family whose domain crosses zero, and the sign IS
    // the direction: negative REPELS. So the minimum is `-MAX`, not `0`, and the
    // panel must not clamp it away (the `AreaTorque` precedent — a negative
    // torque is a direction, not an invalid value).
    IRow {
        label: "panel.physics.pull_force",
        slider: ids::PHYSICS_PULL_FORCE,
        chip: ids::PHYSICS_PULL_FORCE_NUM,
        min: -MAX_ATTRACT_FORCE,
        max: MAX_ATTRACT_FORCE,
        step: 1.0,
        decimals: 0,
        get: |s| s.attract_force,
        set: |s, v| s.attract_force = v,
        shown: |s| s.tool == InteractionTool::Attract,
    },
];

/// The row a widget id belongs to, if any (the slider or its chip).
pub fn irow_for(id: NodeId) -> Option<&'static IRow> {
    IROWS.iter().find(|r| r.slider == id || r.chip == id)
}

/// The tool a segmented option id names, if any.
pub fn tool_for(id: NodeId) -> Option<InteractionTool> {
    ids::PHYSICS_INTERACT_TOOL_OPT
        .iter()
        .position(|&o| o == id)
        .map(|i| InteractionTool::ALL[i])
}

/// The hold mode a segmented option id names, if any.
pub fn hold_for(id: NodeId) -> Option<HoldMode> {
    ids::PHYSICS_HOLD_MODE_OPT
        .iter()
        .position(|&o| o == id)
        .map(|i| HoldMode::ALL[i])
}
