//! Os **hints de UI** desta forma — que widget cada param veste, e em que ordem
//! o painel os pinta.
//!
//! Saíram do `lib.rs` no teto de LOC, por assunto: o pai fica com **o que a forma
//! É** (o manifesto, o descritor, a chave de conteúdo) e este irmão com **como
//! ela se AUTORA**. `pub(crate)` porque só o `register` os consome.

use super::{KIND_LABELS, param};
use ph2d_node_registry::{ParamUiHint, ParamWidget};

/// The param rows: a real dropdown for the shape family (the segmented `Enum`
/// widget the Vector panel uses for Cap/Join), then the geometry sliders. Every
/// row past `size` is gated by [`PARAM_GATES`], so the panel shows ONLY the
/// controls the current `kind` uses.
pub(crate) static PARAM_HINTS: &[ParamUiHint] = &[
    // **O TRAÇO** (doc 89 folha 14, P0) — o controle que separa *forma* de
    // *silhueta*. `0` = sem traço ⇒ a forma que sempre shipou.
    ParamUiHint {
        param: param::STROKE_WIDTH,
        label: "Stroke Width",
        min: 0.0,
        max: 1.0,
        step: 0.005,
        widget: ParamWidget::Slider,
    },
    // ⚠️ **Um SWATCH, nunca quatro sliders lineares** — a lei que o `motion.tint`
    // escreve ao lado do dele: *"nunca sliders lineares crus, um `0.5` linear lê
    // como cinza claro"*. A hint ancora no primeiro canal e nomeia os quatro; o
    // bridge lê o pick de volta (sRGB→linear).
    ParamUiHint {
        param: param::STROKE_R,
        label: "Stroke",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Color {
            channels: [
                param::STROKE_R,
                param::STROKE_G,
                param::STROKE_B,
                param::STROKE_A,
            ],
        },
    },
    ParamUiHint {
        param: param::KIND,
        label: "Shape",
        min: 0.0,
        max: 0.0,
        step: 0.0,
        widget: ParamWidget::Enum {
            labels: KIND_LABELS,
        },
    },
    ParamUiHint {
        param: param::SIZE,
        label: "Size",
        min: 0.05,
        max: 10.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: param::ASPECT,
        label: "Aspect (H/W)",
        min: 0.1,
        max: 4.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: param::SIDES,
        label: "Sides / Points / Teeth",
        min: 3.0,
        max: 32.0,
        step: 1.0,
        widget: ParamWidget::IntSlider,
    },
    ParamUiHint {
        param: param::CORNER,
        label: "Corner Radius",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: param::STAR_DEPTH,
        label: "Point Depth",
        min: 0.05,
        max: 0.95,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: param::CLEFT,
        label: "Cleft",
        min: 0.02,
        max: 0.45,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: param::TOOTH_DEPTH,
        label: "Tooth Depth",
        min: 0.05,
        max: 0.6,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: param::HOLE,
        label: "Hole",
        min: 0.0,
        max: 0.9,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
];
