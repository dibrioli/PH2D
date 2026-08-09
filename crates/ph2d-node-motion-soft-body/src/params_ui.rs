//! The node's **param UI metadata** — labels, ranges, widgets, units. Split from
//! `lib.rs` at the HR-18 LOC cap, on the seam the emitter already uses
//! (`ph2d-node-motion-emitter/src/params_ui.rs`): none of this is behaviour, so the
//! node computes exactly the same result whatever a slider looks like.

use ph2d_node_registry::{ParamHardMax, ParamUiHint, ParamUnit, ParamUnitDecl, ParamWidget};

/// O teto que a MÁQUINA (ou o bom senso) impõe, alcançável por DIGITAÇÃO — o slider fica
/// onde a MÃO trabalha (soft/hard do Blender; doc 88 §11). O curso de antes é este número:
/// nada ficou inalcançável, só deixou de ser o que o dedo percorre.
pub(crate) static PARAM_HARD_MAX: &[ParamHardMax] = &[
    ParamHardMax {
        param: "rows",
        max: 512.0,
    },
    ParamHardMax {
        param: "cols",
        max: 512.0,
    },
    // MEASURED (`is_the_useful_pressure_coupled_to_stiffness`, a 8x8 body squeezed
    // by a force field for 5 s, worst area ratio reached): the term does what it
    // says up to **2** at every stiffness (0,985..1,058), starts overshooting at
    // **3** (1,30..1,49) and is plainly diverging by **6** (1,49..2,70). Four is
    // where it is wrong at every stiffness, so it is where the box stops — the
    // 2..4 strip is still typable on purpose, because an over-pressured balloon
    // that wobbles is a look somebody may want and the artist SEES it.
    ParamHardMax {
        param: "pressure",
        max: 4.0,
    },
    // The largest count any legal mesh can HONOUR: a cluster needs two particles
    // on an axis to have a frame fitted to it, so `counts` caps the split at
    // `side / 2`, and the biggest side this node allows is `MAX_SIDE`. Typing more
    // than this could never be obeyed by any body, which is the box accepting and
    // lying (doc 88 B2); typing less than this IS obeyed, by a mesh big enough.
    //
    // ⚠️ A given body clamps it further — a 4-row snake honours 2 — and that limit
    // is VISIBLE rather than silent: the body simply stops changing as the number
    // goes up.
    ParamHardMax {
        param: "clusters",
        max: 256.0,
    },
];

pub(crate) static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "rows",
        label: "Rows",
        min: 2.0,
        max: 64.0,
        step: 1.0,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "cols",
        label: "Cols",
        min: 2.0,
        max: 64.0,
        step: 1.0,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "spacing",
        label: "Spacing",
        min: 0.1,
        max: 4.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "gravity",
        label: "Gravity",
        min: 0.0,
        max: 40.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "stiffness",
        label: "Stiffness",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "stretch",
        label: "Stretch",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "damping",
        label: "Damping",
        min: 0.0,
        max: 0.5,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "pressure",
        label: "Pressure",
        // The slider is the FINGER's range, not the ceiling (doc 88 §11): 2 is
        // where the measured band of *does what it says* ends. The typable max
        // is twice that.
        min: 0.0,
        max: 2.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "clusters",
        label: "Clusters",
        // The finger's range is the band the measurement found USEFUL. Fitting a
        // rigid frame to a chord of an arc leaves an error that falls as the
        // square of the piece, and the arc probe reads it: 1 → 1,075 · 2 → 0,503 ·
        // 4 → 0,135 · 8 → 0,044 · 16 → 0,017. Past sixteen the curve has been
        // bought and what is left is the cost.
        min: 1.0,
        max: 16.0,
        step: 1.0,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "pin",
        label: "Pin Top",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["Free", "Pinned"],
        },
    },
];

/// **What each of this node's numbers IS** (doc 88, Wave A) — never how it is
/// shown. A `Length` is stored in world METRES and the panel resolves the face
/// the artist reads (`px` or `m`) from `ProjectSettings::display_unit`; a node
/// that could pin one would be overriding a setting it does not own.
///
/// Only params whose value is a world COORDINATE or a world DISTANCE are declared
/// here. A weight, a fraction, a rate and a count are left bare on purpose: a unit
/// that is wrong is worse than a unit that is missing, because the artist can read
/// a bare number but a mislabelled one teaches them something false.
pub(crate) static PARAM_UNITS: &[ParamUnitDecl] = &[ParamUnitDecl {
    param: "spacing",
    unit: ParamUnit::Length,
}];
