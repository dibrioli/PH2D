//! W14 — a **IMAGEM** no `.excalidraw`, pelo que o oráculo grava e desenha
//! (`ferramentas/excalidraw_oracle/saidas/w14/`): o elemento leva o `fileId`, o `scale` (`-1` = o
//! espelho), o `crop` em píxeis da imagem natural e a `roundness` (os cantos); os bytes vão uma vez em
//! `files`, num `dataURL`. Uma imagem sem ficheiro no `files` não se desenha lá no SVG — aqui salta,
//! com aviso (X3).

use std::collections::BTreeMap;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use ph2d_board_model::{
    Board, Crop, Element, ElementId, FracKey, Image, ImageFile, Ingest, Style,
};
use serde_json::{Map, Value, json};

use crate::PH2D;
use crate::style::{num, style_of};

/// Os ficheiros do `files` dele, já pela porta de entrada de imagens (`ingest`), pelo `fileId` dele.
/// Um `dataURL` que não se lê, ou que a porta recusa, não entra.
pub(crate) fn incoming(file: &Value, ingest: Ingest<'_>) -> BTreeMap<String, ImageFile> {
    let Some(files) = file["files"].as_object() else {
        return BTreeMap::new();
    };
    files
        .iter()
        .filter_map(|(k, f)| {
            let url = f["dataURL"].as_str()?;
            let b64 = url.split_once(";base64,")?.1;
            let bytes = STANDARD.decode(b64.trim()).ok()?;
            Some((k.clone(), ingest(&bytes)?))
        })
        .collect()
}

/// O ficheiro de um elemento `image` dele, se chegou.
fn file_of<'a>(v: &Value, files: &'a BTreeMap<String, ImageFile>) -> Option<&'a ImageFile> {
    files.get(v["fileId"].as_str()?)
}

/// O espelho (`scale` `-1`).
fn flip_of(v: &Value) -> [bool; 2] {
    let s = |i: usize| v["scale"][i].as_f64().is_some_and(|x| x < 0.0);
    [s(0), s(1)]
}

/// O `crop` dele em píxeis da imagem que entrou (a `naturalWidth` dele pode não ser a nossa).
fn crop_of(v: &Value, size: [u32; 2]) -> Option<Crop> {
    let c = &v["crop"];
    let (nw, nh) = (num(c, "naturalWidth")?, num(c, "naturalHeight")?);
    if nw <= 0.0 || nh <= 0.0 {
        return None;
    }
    let (kx, ky) = (f64::from(size[0]) / nw, f64::from(size[1]) / nh);
    Some(Crop {
        x: num(c, "x")? * kx,
        y: num(c, "y")? * ky,
        w: num(c, "width")? * kx,
        h: num(c, "height")? * ky,
    })
}

/// Uma imagem de FORA (sem o nosso no `customData`). `None` = o ficheiro não chegou.
pub(crate) fn foreign(
    v: &Value,
    id: ElementId,
    z: FracKey,
    files: &BTreeMap<String, ImageFile>,
) -> Option<Element> {
    let f = file_of(v, files)?;
    let mut st: Style = style_of(v);
    (st.fill, st.stroke) = (None, None);
    st.round = !v["roundness"].is_null();
    let mut img = Image::new(f.id(), st);
    img.crop = crop_of(v, f.size);
    img.flip = flip_of(v);
    let bx = [
        num(v, "x").unwrap_or(0.0),
        num(v, "y").unwrap_or(0.0),
        num(v, "width").unwrap_or(0.0).abs(),
        num(v, "height").unwrap_or(0.0).abs(),
    ];
    let mut el = Element::new_image(id, z, img, bx);
    el.angle = num(v, "angle").unwrap_or(0.0);
    Some(el)
}

/// O que o Excalidraw sabe editar numa imagem NOSSA que viajou: o ficheiro (o que ele traz), o recorte,
/// o espelho e os cantos. `false` = o ficheiro não chegou (a imagem salta).
pub(crate) fn edits(el: &mut Element, v: &Value, files: &BTreeMap<String, ImageFile>) -> bool {
    let Some(f) = file_of(v, files) else {
        return false;
    };
    let (id, size) = (f.id(), f.size);
    let Some(img) = el.image_mut() else {
        return true;
    };
    img.file = id;
    let crop = crop_of(v, size);
    let near = |a: Option<Crop>, b: Option<Crop>| match (a, b) {
        (Some(a), Some(b)) => [a.x - b.x, a.y - b.y, a.w - b.w, a.h - b.h]
            .iter()
            .all(|d| d.abs() < 1e-6),
        (a, b) => a.is_none() && b.is_none(),
    };
    if !near(crop, img.crop) {
        img.crop = crop;
    }
    img.flip = flip_of(v);
    img.style.round = !v["roundness"].is_null();
    true
}

/// Uma imagem nossa → o elemento `image` dele (o nosso inteiro no `customData`).
pub(crate) fn write(board: &Board, el: &Element, img: &Image, mut m: Map<String, Value>) -> Value {
    let file = board.file(img.file).filter(|f| f.mime().is_some());
    m.insert(
        "roundness".into(),
        if img.style.round {
            json!({ "type": 3 })
        } else {
            Value::Null
        },
    );
    m.insert(
        "status".into(),
        json!(if file.is_some() { "saved" } else { "pending" }),
    );
    m.insert("fileId".into(), json!(img.file.hex()));
    let s = |f: bool| if f { -1 } else { 1 };
    m.insert("scale".into(), json!([s(img.flip[0]), s(img.flip[1])]));
    let crop = match (img.crop, file) {
        (Some(c), Some(f)) => json!({
            "x": c.x, "y": c.y, "width": c.w, "height": c.h,
            "naturalWidth": f.size[0], "naturalHeight": f.size[1],
        }),
        _ => Value::Null,
    };
    m.insert("crop".into(), crop);
    let custom = serde_json::to_value(el).unwrap_or(Value::Null);
    m.insert("customData".into(), json!({ PH2D: { "element": custom } }));
    Value::Object(m)
}

/// O `files` dele: os ficheiros que as imagens vivas mostram, cada um uma vez.
pub(crate) fn files(board: &Board, els: &[&Element]) -> Value {
    let mut out = Map::new();
    for img in els.iter().filter_map(|e| e.image()) {
        let key = img.file.hex();
        if out.contains_key(&key) {
            continue;
        }
        let Some((f, mime)) = board
            .file(img.file)
            .and_then(|f| Some((f, f.mime()?)))
        else {
            continue;
        };
        let url = format!("data:{mime};base64,{}", STANDARD.encode(&f.bytes.0));
        out.insert(
            key.clone(),
            json!({ "mimeType": mime, "id": key, "dataURL": url, "created": 1 }),
        );
    }
    Value::Object(out)
}
