//! The node's **param UI metadata** — labels, ranges, widgets, units. Split from
//! `lib.rs` at the HR-18 LOC cap, on the seam the siblings already use
//! (`ph2d-node-motion-soft-body/src/params_ui.rs`): none of this is behaviour, so
//! the rope computes exactly the same catenary whatever a slider looks like.

use ph2d_node_registry::{ParamHardMax, ParamUiHint, ParamUnit, ParamUnitDecl, ParamWidget};
/// **O teto DURO de `count` — MEDIDO** (doc 88 A1 · §0), enquanto o slider fica nos 200 que cobrem
/// uma corda de autoria confortável.
///
/// A relaxação é Gauss-Seidel por aresta — **sequencial por semântica**, mas LINEAR na contagem.
/// Medido pela porta do produto (`measure_the_count_ceiling`, com a aresta `pre` de estado ligada;
/// sem ela o `eval` semeia e a tabela reporta **300× menos**):
///
/// | partículas | cook |
/// |---|---|
/// | 10.000 | 2,040 ms |
/// | **50.000** | **~10 ms** (interpolado do linear) |
/// | 100.000 | 20,533 ms |
/// | 400.000 | 83,267 ms |
///
/// Cem mil já passa de um quadro de 60 fps; cinquenta mil fica em ~60% dele — 250× o que o slider
/// alcança. O teto é onde a medição parou de caber.
pub(crate) static PARAM_HARD_MAX: &[ParamHardMax] = &[
    ParamHardMax {
        param: "damping",
        max: 0.5,
    },
    ParamHardMax {
        param: "count",
        max: 50_000.0,
    },
];

pub(crate) static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "count",
        label: "Points",
        min: 2.0,
        max: 200.0,
        step: 1.0,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "length",
        label: "Length",
        min: 0.5,
        max: 40.0,
        step: 0.1,
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
        param: "iterations",
        label: "Stiffness",
        min: 1.0,
        max: 128.0,
        step: 1.0,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "damping",
        label: "Damping",
        min: 0.0,
        max: 0.2,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "pin_tail",
        label: "Pin Tail",
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
    param: "length",
    unit: ParamUnit::Length,
}];
