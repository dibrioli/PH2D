//! A VOLTA (X2, X3): um ficheiro `.excalidraw` → o documento de um quadro novo.

use std::collections::{BTreeMap, BTreeSet};

use ph2d_board_model::{
    Anchor, BoardDoc, BoardOp, Connector, Dash, Element, ElementId, End, FracKey, Frame, GroupId,
    Ink, Pen, Rgba, RichText, Route, Shape, ShapeType, Style,
};
use serde_json::Value;

use crate::{ID, PH2D, head_of, hex, parse_color};

/// O que a importação trouxe: o documento e o que SALTOU, contado por tipo (o aviso X3).
#[derive(Debug, Default)]
pub struct Import {
    pub doc: BoardDoc,
    pub skipped: BTreeMap<String, usize>,
}

/// O texto preso a um contentor (ou o texto de uma forma nossa que ele não tem).
#[derive(Clone)]
struct Text {
    text: String,
    size: f64,
    color: Option<Rgba>,
}

fn num(v: &Value, k: &str) -> Option<f64> {
    v[k].as_f64().filter(|x| x.is_finite())
}

fn text_of(v: &Value) -> Option<Text> {
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
    })
}

/// O estilo de um elemento de fora (os valores fora de faixa caem nos de nascença DELE, X3).
fn style_of(v: &Value) -> Style {
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
    st.opacity = num(v, "opacity").map_or(100, |o| o.clamp(0.0, 100.0) as u8); // CLAMP-OK: opacidade do ficheiro, 0..=100
    st.round = !v["roundness"].is_null();
    st.sketch = num(v, "roughness").unwrap_or(1.0) > 0.0;
    st
}

/// O elemento nosso que viajou no `customData`, se há.
fn carried(v: &Value) -> Option<Element> {
    serde_json::from_value(v["customData"][PH2D]["element"].clone()).ok()
}

/// ⭐ Um ficheiro `.excalidraw` (o texto JSON) → o documento de um quadro novo e o que saltou.
///
/// # Errors
/// JSON que não se lê, ou sem `elements`.
pub fn read(json: &str) -> Result<Import, String> {
    let file: Value = serde_json::from_str(json).map_err(|e| e.to_string())?;
    let all = file["elements"]
        .as_array()
        .ok_or("this is not an Excalidraw file (no elements)")?;
    let live: Vec<&Value> = all
        .iter()
        .filter(|v| !v["isDeleted"].as_bool().unwrap_or(false))
        .collect();
    let mut imp = Import::default();
    // Os textos presos (a um contentor que exista) e os das nossas formas que ele não tem.
    let ids: BTreeSet<&str> = live.iter().filter_map(|v| v["id"].as_str()).collect();
    let mut texts: BTreeMap<String, Text> = BTreeMap::new();
    let mut consumed: BTreeSet<&str> = BTreeSet::new();
    for v in &live {
        if v["type"] != "text" {
            continue;
        }
        let own = v["customData"][PH2D]["text_of"]
            .as_u64()
            .map(|n| format!("{ID}{n}"));
        let host = v["containerId"].as_str().map(str::to_owned).or(own);
        if let (Some(h), Some(t), Some(me)) = (host, text_of(v), v["id"].as_str())
            && ids.contains(h.as_str())
        {
            texts.insert(h, t);
            consumed.insert(me);
        }
    }
    // Os ids: os nossos que viajaram (a 1.ª ocorrência fica com o seu; uma cópia feita lá ganha um
    // novo), e os de fora a seguir ao maior.
    let mut kept: Vec<&Value> = Vec::new();
    for v in &live {
        let kind = v["type"].as_str().unwrap_or("");
        let me = v["id"].as_str().unwrap_or("");
        if kind == "text" && consumed.contains(me) {
            continue; // voltou para o dono (o contentor, ou a nossa forma que ele não tem)
        }
        let ok = matches!(
            kind,
            "rectangle"
                | "ellipse"
                | "diamond"
                | "arrow"
                | "line"
                | "freedraw"
                | "frame"
                | "magicframe"
                | "text"
        );
        if ok {
            kept.push(v);
        } else {
            *imp.skipped.entry(kind.to_owned()).or_default() += 1;
        }
    }
    let mut taken = BTreeSet::new();
    let mut map: BTreeMap<String, ElementId> = BTreeMap::new();
    for v in &kept {
        if let Some(el) = carried(v)
            && taken.insert(el.id)
        {
            map.insert(v["id"].as_str().unwrap_or("").to_owned(), el.id);
        }
    }
    let mut next = taken.iter().map(|e| e.0).max().unwrap_or(0);
    for v in &kept {
        let k = v["id"].as_str().unwrap_or("").to_owned();
        map.entry(k).or_insert_with(|| {
            next += 1;
            ElementId(next)
        });
    }
    let frames: BTreeSet<ElementId> = kept
        .iter()
        .filter(|v| matches!(v["type"].as_str(), Some("frame" | "magicframe")))
        .filter_map(|v| map.get(v["id"].as_str().unwrap_or("")).copied())
        .collect();
    let mut groups: BTreeMap<String, GroupId> = BTreeMap::new();
    let mut next_group = 0u64;
    for v in &kept {
        for g in v["groupIds"].as_array().into_iter().flatten() {
            if let Some(n) = g
                .as_str()
                .and_then(|s| s.strip_prefix(ID)?.strip_prefix('g')?.parse().ok())
            {
                next_group = next_group.max(n);
                groups.insert(g.as_str().unwrap_or("").to_owned(), GroupId(n));
            }
        }
    }
    let mut z: Option<FracKey> = None;
    let mut out = Vec::new();
    let mut tops: Vec<(bool, Option<ElementId>)> = Vec::new();
    for v in &kept {
        let me = v["id"].as_str().unwrap_or("");
        let id = map[me];
        let key = FracKey::between(z.as_ref(), None);
        z = Some(key.clone());
        let came = carried(v).is_some();
        let mut el = match carried(v) {
            Some(mut ours) => {
                let fresh = ours.id != id;
                ours.id = id;
                ours.z = key;
                ours.deleted = false;
                edits(&mut ours, v, texts.get(me), &map, fresh);
                ours
            }
            None => match foreign(v, id, key, texts.get(me), &map) {
                Some(el) => el,
                None => {
                    *imp.skipped
                        .entry(v["type"].as_str().unwrap_or("").to_owned())
                        .or_default() += 1;
                    continue;
                }
            },
        };
        // O frame do Excalidraw (só um nível: o de topo) — resolvido depois de todos montados.
        let frame = v["frameId"]
            .as_str()
            .and_then(|f| map.get(f))
            .copied()
            .filter(|f| frames.contains(f) && *f != id);
        tops.push((came, frame));
        // Os grupos (sem o par forma+texto nosso).
        let mut gs = Vec::new();
        for g in v["groupIds"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
        {
            if g.strip_prefix(ID).is_some_and(|r| r.starts_with('s')) {
                continue;
            }
            let gid = *groups.entry(g.to_owned()).or_insert_with(|| {
                next_group += 1;
                GroupId(next_group)
            });
            gs.push(gid);
        }
        if el.groups() != gs.as_slice() {
            *el.groups_mut() = gs;
        }
        let locked = v["locked"].as_bool().unwrap_or(false);
        if el.locked() != locked {
            el.set_locked(locked);
        }
        out.push(el);
    }
    // Um nosso dentro de um frame ANINHADO viajou com o de topo: fica no seu se o de topo é o mesmo.
    let parent: BTreeMap<ElementId, Option<ElementId>> =
        out.iter().map(|e| (e.id, e.frame)).collect();
    let top = |mut f: ElementId| {
        let mut seen = BTreeSet::new();
        while let Some(Some(up)) = parent.get(&f) {
            if !seen.insert(f) {
                break;
            }
            f = *up;
        }
        f
    };
    for (el, (came, excal)) in out.iter_mut().zip(tops) {
        // O que NÓS escreveríamos: um frame vai sempre sem `frameId` (ele não os aninha); o resto com
        // o frame de topo. Igual = o Excalidraw não mexeu: fica o nosso (o aninhado incluído).
        let ours = if el.as_frame().is_some() {
            None
        } else {
            el.frame.map(top)
        };
        if !(came && ours == excal) {
            el.frame = excal;
        }
    }
    for el in out {
        BoardOp::Put(el).apply(&mut imp.doc);
    }
    Ok(imp)
}

/// A ponta de uma seta pela ligação dele (o id mapeado), ou solta no ponto.
fn end_of(v: &Value, key: &str, at: [f64; 2], map: &BTreeMap<String, ElementId>) -> End {
    let b = &v[key];
    let Some(target) = b["elementId"].as_str().and_then(|t| map.get(t)) else {
        return End::Free(at);
    };
    let anchor = match b["fixedPoint"].as_array() {
        Some(fp) if fp.len() == 2 => {
            Anchor::Fixed([fp[0].as_f64().unwrap_or(0.5), fp[1].as_f64().unwrap_or(0.5)])
        }
        _ => Anchor::Center,
    };
    End::Bound {
        target: *target,
        anchor,
    }
}

/// Os pontos de uma linha/seta/caneta no mundo (relativos a `x, y`, rodados à volta do centro da
/// caixa dos pontos — o `angle` dele).
fn world_points(v: &Value) -> Vec<[f64; 2]> {
    let (x, y) = (num(v, "x").unwrap_or(0.0), num(v, "y").unwrap_or(0.0));
    let pts: Vec<[f64; 2]> = v["points"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|p| Some([p[0].as_f64()? + x, p[1].as_f64()? + y]))
        .collect();
    let angle = num(v, "angle").unwrap_or(0.0);
    if angle == 0.0 || pts.is_empty() {
        return pts;
    }
    let (mut x0, mut y0, mut x1, mut y1) = (f64::MAX, f64::MAX, f64::MIN, f64::MIN);
    for p in &pts {
        (x0, y0, x1, y1) = (x0.min(p[0]), y0.min(p[1]), x1.max(p[0]), y1.max(p[1]));
    }
    let c = [(x0 + x1) / 2.0, (y0 + y1) / 2.0];
    pts.iter()
        .map(|&p| ph2d_board_model::rotate_about(p, c, angle))
        .collect()
}

/// As edições que o Excalidraw sabe fazer, aplicadas ao nosso elemento que viajou (X2): o que mudou
/// lá ganha; o que ficou igual fica exactamente o nosso.
fn edits(
    el: &mut Element,
    v: &Value,
    text: Option<&Text>,
    map: &BTreeMap<String, ElementId>,
    fresh: bool,
) {
    let (x, y) = (num(v, "x").unwrap_or(el.x), num(v, "y").unwrap_or(el.y));
    let near = |a: f64, b: f64| (a - b).abs() < 1e-6;
    match &el.kind {
        ph2d_board_model::ElementKind::Shape(s) if crate::write::is_native(s.kind) => {
            let (w, h) = (
                num(v, "width").unwrap_or(el.w),
                num(v, "height").unwrap_or(el.h),
            );
            if !(near(x, el.x) && near(y, el.y) && near(w, el.w) && near(h, el.h)) {
                (el.x, el.y, el.w, el.h) = (x, y, w, h);
            }
        }
        ph2d_board_model::ElementKind::Shape(s) => {
            let p0 = crate::write::polygon(el, s)
                .first()
                .copied()
                .unwrap_or([el.x, el.y]);
            if !(near(x, p0[0]) && near(y, p0[1])) {
                el.translate([x - p0[0], y - p0[1]]);
            }
        }
        ph2d_board_model::ElementKind::Frame(_) => {
            let (w, h) = (
                num(v, "width").unwrap_or(el.w),
                num(v, "height").unwrap_or(el.h),
            );
            (el.x, el.y, el.w, el.h) = (x, y, w, h);
            if let (Some(n), Some(f)) = (v["name"].as_str(), frame_mut(el)) {
                n.clone_into(&mut f.name);
            }
        }
        ph2d_board_model::ElementKind::Ink(_) => {
            let first = ph2d_board_edit::world_points(el)
                .first()
                .map_or([el.x, el.y], |p| [p[0], p[1]]);
            if !(near(x, first[0]) && near(y, first[1])) {
                el.translate([x - first[0], y - first[1]]);
            }
        }
        ph2d_board_model::ElementKind::Connector(_) => {
            let pts = world_points(v);
            if let (Some(c), Some(&a), Some(&b)) = (el.connector_mut(), pts.first(), pts.last()) {
                let ends = [
                    end_of(v, "startBinding", a, map),
                    end_of(v, "endBinding", b, map),
                ];
                for (k, e) in ends.into_iter().enumerate() {
                    let same = match (&c.ends()[k], &e) {
                        (End::Bound { target: t0, .. }, End::Bound { target: t1, .. }) => t0 == t1,
                        (End::Free(p), End::Free(q)) => near(p[0], q[0]) && near(p[1], q[1]),
                        _ => false,
                    };
                    if !same || fresh {
                        *c.end_mut(k) = e;
                    }
                }
            }
        }
    }
    if !matches!(
        el.kind,
        ph2d_board_model::ElementKind::Ink(_) | ph2d_board_model::ElementKind::Connector(_)
    ) {
        let a = num(v, "angle").unwrap_or(el.angle);
        if !near(a, el.angle) {
            el.angle = a;
        }
    }
    // O estilo: só o que difere do que escrevemos.
    let st = el.style_mut();
    for (k, slot) in [
        ("strokeColor", &mut st.stroke),
        ("backgroundColor", &mut st.fill),
    ] {
        if let Some(s) = v[k].as_str()
            && s != hex(*slot)
            && let Some(c) = parse_color(s)
        {
            *slot = c;
        }
    }
    if let Some(w) = num(v, "strokeWidth").filter(|w| !near(*w, st.stroke_width) && *w > 0.0) {
        st.stroke_width = w;
    }
    if let Some(o) = num(v, "opacity").filter(|o| !near(*o, f64::from(st.opacity))) {
        st.opacity = o.clamp(0.0, 100.0) as u8; // CLAMP-OK: opacidade do ficheiro, 0..=100
    }
    let rough = num(v, "roughness").unwrap_or(f64::from(u8::from(st.sketch))) > 0.0;
    if rough != st.sketch {
        st.sketch = rough;
    }
    if let (Some(t), Some(shape)) = (text, el.shape_mut())
        && t.text != shape.text.as_str()
    {
        shape.text = RichText::plain(&t.text);
    }
    if let (Some(t), Some(c)) = (text, el.connector_mut())
        && t.text != c.label
    {
        c.label.clone_from(&t.text);
    }
}

fn frame_mut(el: &mut Element) -> Option<&mut Frame> {
    match &mut el.kind {
        ph2d_board_model::ElementKind::Frame(f) => Some(f),
        _ => None,
    }
}

/// Um elemento de FORA (sem o nosso no `customData`) pelo mapeamento directo (X3).
fn foreign(
    v: &Value,
    id: ElementId,
    z: FracKey,
    text: Option<&Text>,
    map: &BTreeMap<String, ElementId>,
) -> Option<Element> {
    let kind = v["type"].as_str()?;
    let bx = [
        num(v, "x").unwrap_or(0.0),
        num(v, "y").unwrap_or(0.0),
        num(v, "width").unwrap_or(0.0).abs(),
        num(v, "height").unwrap_or(0.0).abs(),
    ];
    let mut st = style_of(v);
    let shape = |kind: ShapeType, mut st: Style, text: Option<&Text>| {
        if let Some(t) = text {
            st.font_size = t.size;
            if let Some(c) = t.color {
                st.text_color = c;
            }
        }
        Shape {
            kind,
            style: st,
            text: text.map_or_else(RichText::default, |t| RichText::plain(&t.text)),
        }
    };
    let mut el = match kind {
        "rectangle" | "ellipse" | "diamond" => {
            let k = match kind {
                "ellipse" => ShapeType::Ellipse,
                "diamond" => ShapeType::Diamond,
                _ => ShapeType::Rectangle,
            };
            Element::new_shape(id, z, shape(k, st, text), bx)
        }
        "text" => {
            // Texto solto: uma forma sem preenchimento nem contorno com o texto dentro.
            let t = text_of(v)?;
            st.fill = None;
            st.stroke = None;
            Element::new_shape(id, z, shape(ShapeType::Rectangle, st, Some(&t)), bx)
        }
        "frame" | "magicframe" => {
            let name = v["name"].as_str().unwrap_or("Frame").to_owned();
            st.fill = None;
            st.stroke = None;
            Element::new_frame(id, z, Frame { name, style: st }, bx)
        }
        "freedraw" => {
            let pr: Vec<f64> = v["pressures"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(Value::as_f64)
                .collect();
            let pts = world_points(v);
            if pts.is_empty() {
                return None;
            }
            let world: Vec<[f64; 3]> = pts
                .iter()
                .enumerate()
                .map(|(i, p)| [p[0], p[1], pr.get(i).copied().unwrap_or(0.5)])
                .collect();
            let measured = !v["simulatePressure"].as_bool().unwrap_or(true) && !pr.is_empty();
            let (ink, bx) = Ink::from_world(&world, st, Pen::Pen, measured);
            Element::new_ink(id, z, ink, bx)
        }
        "arrow" | "line" => {
            let pts = world_points(v);
            if pts.len() < 2 {
                return None;
            }
            let route = if v["elbowed"].as_bool().unwrap_or(false) {
                Route::Elbow
            } else if st.round {
                Route::Curved
            } else {
                Route::Straight
            };
            st.fill = None;
            st.round = false;
            if let Some(t) = text {
                st.font_size = t.size;
            }
            let mut c = Connector::new(
                end_of(v, "startBinding", pts[0], map),
                end_of(v, "endBinding", pts[pts.len() - 1], map),
                route,
                st,
            );
            c.heads = [
                head_of(v["startArrowhead"].as_str()),
                head_of(v["endArrowhead"].as_str()),
            ];
            if route != Route::Elbow {
                c.waypoints = pts[1..pts.len() - 1].to_vec();
            }
            if let Some(t) = text {
                c.label.clone_from(&t.text);
            }
            Element::new_connector(id, z, c)
        }
        _ => return None,
    };
    if !matches!(kind, "freedraw" | "arrow" | "line") {
        el.angle = num(v, "angle").unwrap_or(0.0);
    }
    Some(el)
}
