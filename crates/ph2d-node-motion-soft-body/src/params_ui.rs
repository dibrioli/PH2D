//! The node's **param UI metadata** — labels, ranges, widgets, units. Split from
//! `lib.rs` at the HR-18 LOC cap, on the seam the emitter already uses
//! (`ph2d-node-motion-emitter/src/params_ui.rs`): none of this is behaviour, so the
//! node computes exactly the same result whatever a slider looks like.

use ph2d_node_registry::{ParamUiHint, ParamUnit, ParamUnitDecl, ParamWidget};

pub(crate) static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "rows",
        label: "Rows",
        min: 2.0,
        max: 512.0,
        step: 1.0,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "cols",
        label: "Cols",
        min: 2.0,
        max: 512.0,
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
