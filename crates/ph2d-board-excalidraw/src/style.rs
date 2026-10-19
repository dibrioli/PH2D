//! O ESTILO de um elemento de fora (cortado do `read.rs` no fecho da W9, tecto de LOC): as cores, o
//! traço, a opacidade, o rascunho e o aspecto que vem do Excalidraw (o preenchimento, a rugosidade 2,
//! a semente, o alinhamento e a letra do texto).

use ph2d_board_model::{Dash, FillStyle, Letter, Rgba, Style, TextAlign, VAlign};
use serde_json::Value;

use crate::{letter_of, parse_color};

/// O texto preso a um contentor (ou o texto de uma forma nossa que ele não tem).
#[derive(Clone)]
pub(crate) struct Text {
    pub(crate) text: String,
    pub(crate) size: f64,
    pub(crate) color: Option<Rgba>,
    pub(crate) align: TextAlign,
    pub(crate) valign: VAlign,
    pub(crate) letter: Letter,
    /// W23 B5(2) — num texto preso a uma seta, a fracção da linha onde ele fica (o `labelPosition`).
    pub(crate) at: Option<f64>,
}

pub(crate) fn num(v: &Value, k: &str) -> Option<f64> {
    v[k].as_f64().filter(|x| x.is_finite())
}

pub(crate) fn text_of(v: &Value) -> Option<Text> {
    let text = v["originalText"]
        .as_str()
        .or_else(|| v["text"].as_str())?
        .to_owned();
    Some(Text {
        text,
        size: num(v, "fontSize")
            .filter(|s| *s > 0.0)
            .unwrap_or(ph2d_board_model::DEFAULT_FONT_SIZE),
        color: v["strokeColor"].as_str().and_then(parse_color).flatten(),
        align: match v["textAlign"].as_str() {
            Some("left") => TextAlign::Left,
            Some("right") => TextAlign::Right,
            _ => TextAlign::Center,
        },
        valign: match v["verticalAlign"].as_str() {
            Some("top") => VAlign::Top,
            Some("bottom") => VAlign::Bottom,
            _ => VAlign::Middle,
        },
        letter: letter_of(v["fontFamily"].as_u64()),
        at: crate::caption::label_position(v),
    })
}

/// O `fillStyle` dele (desconhecido = cheio).
pub(crate) fn fill_style_of(v: &Value) -> Option<FillStyle> {
    Some(match v["fillStyle"].as_str()? {
        "hachure" => FillStyle::Hachure,
        "cross-hatch" => FillStyle::CrossHatch,
        "zigzag" => FillStyle::Zigzag,
        _ => FillStyle::Solid,
    })
}

/// A `seed` dele (o tremor do rascunho), se é uma que a nossa conta aceita.
pub(crate) fn seed_of(v: &Value) -> Option<u32> {
    v["seed"]
        .as_u64()
        .and_then(|s| u32::try_from(s).ok())
        .filter(|s| *s > 0)
}

/// O estilo de um elemento de fora (os valores fora de faixa caem nos de nascença DELE, X3).
pub(crate) fn style_of(v: &Value) -> Style {
    let color = |k: &str, dflt: Option<Rgba>| v[k].as_str().and_then(parse_color).unwrap_or(dflt);
    let ink = Rgba([0x1e, 0x1e, 0x1e, 255]);
    let mut st = Style::new(
        color("backgroundColor", None),
        color("strokeColor", Some(ink)),
        ink,
    );
    st.stroke_width = num(v, "strokeWidth").filter(|w| *w > 0.0).unwrap_or(2.0);
    st.dash = match v["strokeStyle"].as_str() {
        Some("dashed") => Dash::Dashed,
        Some("dotted") => Dash::Dotted,
        _ => Dash::Solid,
    };
    st.opacity = opacity(v).unwrap_or(100);
    st.round = !v["roundness"].is_null();
    let rough = num(v, "roughness").unwrap_or(1.0);
    st.sketch = rough > 0.0;
    st.look.cartoonist = rough > 1.5;
    st.look.fill_style = fill_style_of(v).unwrap_or_default();
    st
}

/// O aspecto do texto `t` no estilo da forma que o leva.
pub(crate) fn text_look(st: &mut Style, t: &Text) {
    st.look.align = t.align;
    st.look.valign = t.valign;
    st.look.letter = t.letter;
}

/// A opacidade do ficheiro em `0..=100`; fora da faixa (o `restore` dele não a valida) = `None`, e
/// quem lê fica com a de nascença dele (100, X3).
pub(crate) fn opacity(v: &Value) -> Option<u8> {
    num(v, "opacity")
        .filter(|o| (0.0..=100.0).contains(o))
        .map(|o| o.round() as u8)
}

/// O traço do Excalidraw.
pub(crate) fn dash_of(v: &Value) -> Option<Dash> {
    Some(match v["strokeStyle"].as_str()? {
        "dashed" => Dash::Dashed,
        "dotted" => Dash::Dotted,
        "solid" => Dash::Solid,
        _ => return None,
    })
}
