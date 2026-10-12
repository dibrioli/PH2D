//! W18 (ADR-0185) — **o vídeo de ida e volta**: o `embeddable` do Excalidraw com um `link` do YouTube
//! (o esquema medido no oráculo, `saidas/w18/w18_offline_Z04_gravado.excalidraw`: `type`, `link`,
//! `roundness: {type: 3}`, contorno e fundo transparentes). Um `embeddable` que não é um vídeo do YouTube
//! (outro site) continua a saltar com o aviso.

use ph2d_board_model::{Element, ElementId, Embed, FracKey, Style};
use serde_json::{Map, Value, json};

use crate::PH2D;
use crate::style::{num, style_of};

/// Um vídeo de FORA. `None` = o link não é um vídeo do YouTube.
pub(crate) fn foreign(v: &Value, id: ElementId, z: FracKey) -> Option<Element> {
    let link = v["link"].as_str()?;
    let video = ph2d_board_embed::youtube(link)?;
    let mut style: Style = style_of(v);
    (style.fill, style.stroke) = (None, None);
    style.round = !v["roundness"].is_null();
    let bx = [
        num(v, "x").unwrap_or(0.0),
        num(v, "y").unwrap_or(0.0),
        num(v, "width").unwrap_or(0.0).abs(),
        num(v, "height").unwrap_or(0.0).abs(),
    ];
    let e = Embed {
        link: link.trim().to_owned(),
        video,
        style,
    };
    let mut el = Element::new_embed(id, z, e, bx);
    el.angle = num(v, "angle").unwrap_or(0.0);
    Some(el)
}

/// O que o Excalidraw sabe editar num vídeo NOSSO que viajou: o link e os cantos. `false` = o link de lá
/// já não é um vídeo (o elemento salta).
pub(crate) fn edits(el: &mut Element, v: &Value) -> bool {
    let Some(e) = el.embed_mut() else {
        return true;
    };
    if let Some(link) = v["link"].as_str()
        && link.trim() != e.link
    {
        let Some(video) = ph2d_board_embed::youtube(link) else {
            return false;
        };
        (e.link, e.video) = (link.trim().to_owned(), video);
    }
    e.style.round = !v["roundness"].is_null();
    true
}

/// Um vídeo nosso → o `embeddable` dele (o nosso inteiro no `customData`).
pub(crate) fn write(el: &Element, e: &Embed, mut m: Map<String, Value>) -> Value {
    m.insert("link".into(), json!(e.link));
    m.insert(
        "roundness".into(),
        if e.style.round {
            json!({ "type": 3 })
        } else {
            Value::Null
        },
    );
    let custom = serde_json::to_value(el).unwrap_or(Value::Null);
    m.insert("customData".into(), json!({ PH2D: { "element": custom } }));
    Value::Object(m)
}
