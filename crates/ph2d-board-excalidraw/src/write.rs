//! A IDA (X1, X2, X4): um quadro → um ficheiro `.excalidraw`.

use std::collections::BTreeMap;

use ph2d_board_model::{
    Anchor, Board, Dash, Element, ElementId, ElementKind, End, FillStyle, Route, Shape, ShapeType,
    Style, TextAlign, VAlign,
};
use ph2d_board_route::RouteCache;
use ph2d_vector::{BezPath, PathEl};
use serde_json::{Map, Value, json};

use crate::{ID, LINE_HEIGHT, PH2D, font_family, head_name, hex};

fn id(e: ElementId) -> String {
    format!("{ID}{}", e.0)
}

/// Os pontos de um caminho aplanado (`tolerance` em unidades do mundo).
fn flat(path: &BezPath, tolerance: f64) -> Vec<[f64; 2]> {
    let mut pts = Vec::new();
    ph2d_vector::flatten(path.elements().iter().copied(), tolerance, |e| match e {
        PathEl::MoveTo(p) | PathEl::LineTo(p) => pts.push([p.x, p.y]),
        _ => {}
    });
    pts
}

/// O frame de TOPO de `el` (o Excalidraw não aninha frames: o `frameId` é o de fora).
fn top_frame(board: &Board, el: &Element) -> Option<ElementId> {
    board.doc.frame_chain(el.id).last().copied()
}

/// Os campos que todo elemento leva, com o estilo do nosso.
fn base(board: &Board, el: &Element, kind: &str, groups: &[String]) -> Map<String, Value> {
    let st = el.style();
    let mut m = Map::new();
    m.insert("id".into(), json!(id(el.id)));
    m.insert("type".into(), json!(kind));
    m.insert("x".into(), json!(el.x));
    m.insert("y".into(), json!(el.y));
    m.insert("width".into(), json!(el.w));
    m.insert("height".into(), json!(el.h));
    m.insert("angle".into(), json!(el.angle));
    m.insert("strokeColor".into(), json!(hex(st.stroke)));
    m.insert("backgroundColor".into(), json!(hex(st.fill)));
    m.insert(
        "fillStyle".into(),
        json!(fill_style_name(st.look.fill_style)),
    );
    m.insert("strokeWidth".into(), json!(st.stroke_width));
    let dash = match st.dash {
        Dash::Solid => "solid",
        Dash::Dashed => "dashed",
        Dash::Dotted => "dotted",
    };
    m.insert("strokeStyle".into(), json!(dash));
    m.insert("roughness".into(), json!(roughness(el)));
    m.insert("opacity".into(), json!(st.opacity));
    m.insert("roundness".into(), Value::Null);
    m.insert("seed".into(), json!(el.seed()));
    m.insert("version".into(), json!(el.version.max(1)));
    m.insert("versionNonce".into(), json!(el.nonce));
    m.insert("isDeleted".into(), json!(false));
    let mut g: Vec<String> = groups.to_vec();
    g.extend(el.groups().iter().map(|x| format!("{ID}g{}", x.0)));
    m.insert("groupIds".into(), json!(g));
    m.insert(
        "frameId".into(),
        top_frame(board, el)
            .filter(|f| *f != el.id)
            .map_or(Value::Null, |f| json!(id(f))),
    );
    m.insert("boundElements".into(), json!([]));
    m.insert("updated".into(), json!(1));
    m.insert("link".into(), Value::Null);
    m.insert("locked".into(), json!(el.locked()));
    m
}

/// O texto de uma forma ou de uma seta, num `text` preso a `container` (ou solto num grupo, quando o
/// contentor não o aceita).
fn text(
    board: &Board,
    owner: &Element,
    content: &str,
    st: &Style,
    rect: [f64; 4],
    container: Option<&str>,
    groups: &[String],
) -> Value {
    let (font, color) = (st.font_size, st.text_color);
    let lines = content.split('\n').count().max(1) as f64;
    let h = lines * font * LINE_HEIGHT;
    // Onde o bloco assenta na caixa: o MESMO `verticalAlign` que escrevemos (auditoria W9: o y era
    // sempre o do meio e o ficheiro dizia «top»).
    let pad = ph2d_board_geom::TEXT_PADDING;
    let y = match st.look.valign {
        VAlign::Top => rect[1] + pad,
        VAlign::Middle => rect[1] + (rect[3] - h) / 2.0,
        VAlign::Bottom => rect[1] + rect[3] - h - pad,
    };
    let mut m = base(board, owner, "text", groups);
    m.insert("id".into(), json!(format!("{}-text", id(owner.id))));
    m.insert("x".into(), json!(rect[0]));
    m.insert("y".into(), json!(y));
    m.insert("width".into(), json!(rect[2]));
    m.insert("height".into(), json!(h));
    m.insert("strokeColor".into(), json!(hex(Some(color))));
    m.insert("backgroundColor".into(), json!("transparent"));
    m.insert("text".into(), json!(content));
    m.insert("originalText".into(), json!(content));
    m.insert("fontSize".into(), json!(font));
    m.insert("fontFamily".into(), json!(font_family(st)));
    let align = match st.look.align {
        TextAlign::Center => "center",
        TextAlign::Left => "left",
        TextAlign::Right => "right",
    };
    m.insert("textAlign".into(), json!(align));
    let valign = match st.look.valign {
        VAlign::Middle => "middle",
        VAlign::Top => "top",
        VAlign::Bottom => "bottom",
    };
    m.insert("verticalAlign".into(), json!(valign));
    m.insert("containerId".into(), json!(container));
    m.insert("lineHeight".into(), json!(LINE_HEIGHT));
    m.insert("autoResize".into(), json!(true));
    m.insert(
        "customData".into(),
        json!({ PH2D: { "text_of": owner.id.0 } }),
    );
    Value::Object(m)
}

fn native(kind: ShapeType) -> Option<&'static str> {
    match kind {
        ShapeType::Rectangle
        | ShapeType::Sticky
        | ShapeType::StickyStack
        | ShapeType::StickyWide => Some("rectangle"),
        ShapeType::Ellipse => Some("ellipse"),
        ShapeType::Diamond => Some("diamond"),
        _ => None,
    }
}

/// Os pontos relativos ao 1.º (o `x, y` dele) e a largura e a altura da CAIXA deles (a do Excalidraw:
/// a leitura compara-a para saber se o elemento foi redimensionado lá).
fn points_rel(pts: &[[f64; 2]]) -> (f64, f64, Value, f64, f64) {
    let [x0, y0] = pts.first().copied().unwrap_or([0.0, 0.0]);
    let (mut lo, mut hi) = ([f64::MAX; 2], [f64::MIN; 2]);
    let rel: Vec<Value> = pts
        .iter()
        .map(|p| {
            for k in 0..2 {
                lo[k] = lo[k].min(p[k]);
                hi[k] = hi[k].max(p[k]);
            }
            json!([p[0] - x0, p[1] - y0])
        })
        .collect();
    let (w, h) = if pts.is_empty() {
        (0.0, 0.0)
    } else {
        (hi[0] - lo[0], hi[1] - lo[1])
    };
    (x0, y0, Value::Array(rel), w, h)
}

/// O contorno de uma forma que o Excalidraw não tem, fechado, no mundo (antes de rodar).
pub(crate) fn polygon(el: &Element, s: &Shape) -> Vec<[f64; 2]> {
    let o = ph2d_board_geom::outline(s, el.w, el.h);
    let mut pts = flat(&o.fill, 0.25);
    if let Some(&p0) = pts.first() {
        pts.push(p0);
    }
    pts.iter().map(|p| [p[0] + el.x, p[1] + el.y]).collect()
}

/// A forma vai como contentor nativo (`rectangle`, `ellipse`, `diamond`)?
pub(crate) fn is_native(kind: ShapeType) -> bool {
    native(kind).is_some()
}

fn shape(board: &Board, el: &Element, s: &Shape, out: &mut Vec<Value>) {
    let st = &s.style;
    let mut groups = Vec::new();
    let container;
    let mut m = if let Some(kind) = native(s.kind) {
        let mut m = base(board, el, kind, &[]);
        if st.round {
            let t = if kind == "rectangle" { 3 } else { 2 };
            m.insert("roundness".into(), json!({ "type": t }));
        }
        container = Some(id(el.id));
        m
    } else {
        // As formas que ele não tem: o NOSSO contorno como uma `line` fechada e cheia, e o texto num
        // `text` do mesmo grupo (uma `line` não leva texto preso).
        let local = polygon(el, s);
        let (x0, y0, rel, w, h) = points_rel(&local);
        groups.push(format!("{ID}s{}", el.id.0));
        let mut m = base(board, el, "line", &groups);
        m.insert("x".into(), json!(x0));
        m.insert("y".into(), json!(y0));
        m.insert("width".into(), json!(w));
        m.insert("height".into(), json!(h));
        m.insert("points".into(), rel);
        m.insert("startBinding".into(), Value::Null);
        m.insert("endBinding".into(), Value::Null);
        m.insert("startArrowhead".into(), Value::Null);
        m.insert("endArrowhead".into(), Value::Null);
        m.insert("lastCommittedPoint".into(), Value::Null);
        container = None;
        m
    };
    let custom = serde_json::to_value(el).unwrap_or(Value::Null);
    m.insert("customData".into(), json!({ PH2D: { "element": custom } }));
    let has_text = !s.text.is_empty();
    if has_text && container.is_some() {
        m.insert(
            "boundElements".into(),
            json!([{ "id": format!("{}-text", id(el.id)), "type": "text" }]),
        );
    }
    out.push(Value::Object(m));
    if has_text {
        out.push(text(
            board,
            el,
            s.text.as_str(),
            st,
            [el.x, el.y, el.w, el.h],
            container.as_deref(),
            &groups,
        ));
    }
}

fn arrow(board: &Board, el: &Element, routes: &RouteCache, out: &mut Vec<Value>) {
    let Some(c) = el.connector() else { return };
    let Some(r) = routes.get(el.id) else { return };
    let pts = match c.route {
        Route::Straight | Route::Elbow => r.polyline(),
        // A curva DELE pelos pontos: ele refá-la igual (XA1).
        Route::Spline => r.stations.clone(),
        Route::Curved => {
            let d = ph2d_board_route::drawn(r, [ph2d_board_model::Head::None; 2], 1.0);
            flat(&d.line, 0.5)
        }
    };
    if pts.len() < 2 {
        return;
    }
    let (x0, y0, rel, w, h) = points_rel(&pts);
    let mut m = base(board, el, "arrow", &[]);
    m.insert("x".into(), json!(x0));
    m.insert("y".into(), json!(y0));
    m.insert("width".into(), json!(w));
    m.insert("height".into(), json!(h));
    m.insert("points".into(), rel.clone());
    m.insert("lastCommittedPoint".into(), Value::Null);
    if matches!(c.route, Route::Curved | Route::Spline) {
        m.insert("roundness".into(), json!({ "type": 2 }));
    }
    let elbow = c.route == Route::Elbow;
    m.insert("elbowed".into(), json!(elbow));
    if elbow {
        let fixed = crate::elbow::fixed_segments(c, r, &pts, x0, y0);
        m.insert("fixedSegments".into(), fixed);
    }
    let ends = [&pts[0], &pts[pts.len() - 1]];
    let near = [pts[1], pts[pts.len() - 2]];
    for (k, key) in ["startBinding", "endBinding"].into_iter().enumerate() {
        let b = match c.ends()[k] {
            End::Bound { target, anchor } => board.doc.get(target).map(|t| {
                if elbow {
                    // O ponto do CONTORNO (a ponta está a `g` dele): num estranho o cotovelo vira um
                    // ponto fixo, e um ponto fora da caixa escalava o vão com a forma (auditoria W9).
                    let on = ph2d_board_geom::nearest_on_outline(t, *ends[k]).map_or(*ends[k], |q| q.0);
                    let fp = ph2d_board_route::normalized(t, on);
                    return json!({ "elementId": id(target), "focus": 0, "gap": 1, "fixedPoint": fp });
                }
                binding(t, anchor, near[k], *ends[k])
            }),
            End::Free(_) => None,
        };
        m.insert(key.into(), b.unwrap_or(Value::Null));
    }
    m.insert("startArrowhead".into(), json!(head_name(c.heads[0])));
    m.insert("endArrowhead".into(), json!(head_name(c.heads[1])));
    let custom = serde_json::to_value(el).unwrap_or(Value::Null);
    // Os pontos que ESCREVEMOS (a leitura sabe se foram mexidos lá: uma recta com pontos novos).
    let wrote = json!({ "x": x0, "y": y0, "points": rel });
    m.insert(
        "customData".into(),
        json!({ PH2D: { "element": custom, "wrote": wrote } }),
    );
    if !c.label.is_empty() {
        m.insert(
            "boundElements".into(),
            json!([{ "id": format!("{}-text", id(el.id)), "type": "text" }]),
        );
    }
    out.push(Value::Object(m));
    if !c.label.is_empty() {
        let st = &c.style;
        let w = ph2d_board_route::LABEL_WRAP;
        let rect = [
            r.mid[0] - w / 2.0,
            r.mid[1] - st.font_size,
            w,
            2.0 * st.font_size,
        ];
        let me = id(el.id);
        out.push(text(board, el, &c.label, st, rect, Some(&me), &[]));
    }
}

/// ⭐ **Uma ligação nas DUAS gramáticas** (W9, XV3: cada versão lê a sua e preserva a outra): a do next
/// (`mode` + `fixedPoint`) e a do 0.18.1 (`focus` + `gap`). O ponto fixo é o `inside` dele (a ponta É o
/// ponto); a órbita e o centro são `orbit` — o centro com o foco do next (`[0.5001, 0.5001]`, XV2). O
/// legado é o par que faz o 0.18.1, ao re-ligar a partir do ponto vizinho `from`, pôr a ponta em `tip`
/// (W10, XV7: [`crate::legacy_focus`]).
fn binding(t: &Element, anchor: Anchor, from: [f64; 2], tip: [f64; 2]) -> Value {
    let (focus, gap) = crate::legacy_focus::legacy(t, from, tip, ph2d_board_route::gap(t));
    let (mode, fp) = match anchor {
        Anchor::Fixed(fp) => ("inside", fp),
        Anchor::Orbit(fp) => ("orbit", fp),
        Anchor::Center => ("orbit", [CENTER_FOCUS, CENTER_FOCUS]),
    };
    json!({ "elementId": id(t.id), "mode": mode, "fixedPoint": fp, "focus": focus, "gap": gap })
}

/// O `fillStyle` dele.
fn fill_style_name(f: FillStyle) -> &'static str {
    match f {
        FillStyle::Solid => "solid",
        FillStyle::Hachure => "hachure",
        FillStyle::CrossHatch => "cross-hatch",
        FillStyle::Zigzag => "zigzag",
    }
}

/// A `roughness` dele: 0 no final, 1 no rascunho, 2 no rascunho «cartoonist» — e 0 numa nota, cujo
/// rascunho é só a letra (o papel desenha-se liso).
pub(crate) fn roughness(el: &Element) -> u8 {
    let st = el.style();
    if el.shape().is_some_and(|s| s.kind.is_note()) {
        return 0;
    }
    match (st.sketch, st.look.cartoonist) {
        (false, _) => 0,
        (true, false) => 1,
        (true, true) => 2,
    }
}

/// O foco do centro que o next escreve (XV2: `focus 0` ⇒ `[0.5001, 0.5001]`).
const CENTER_FOCUS: f64 = 0.5001;

fn freedraw(board: &Board, el: &Element, out: &mut Vec<Value>) {
    let Some(ink) = el.ink() else { return };
    let world = ph2d_board_edit::world_points(el);
    let pts: Vec<[f64; 2]> = world.iter().map(|p| [p[0], p[1]]).collect();
    let (x0, y0, rel, w, h) = points_rel(&pts);
    let mut m = base(board, el, "freedraw", &[]);
    m.insert("x".into(), json!(x0));
    m.insert("y".into(), json!(y0));
    m.insert("width".into(), json!(w));
    m.insert("height".into(), json!(h));
    m.insert("angle".into(), json!(0.0));
    m.insert("points".into(), rel);
    m.insert(
        "pressures".into(),
        json!(world.iter().map(|p| p[2]).collect::<Vec<_>>()),
    );
    m.insert("simulatePressure".into(), json!(!ink.pressure));
    m.insert("lastCommittedPoint".into(), Value::Null);
    let custom = serde_json::to_value(el).unwrap_or(Value::Null);
    m.insert("customData".into(), json!({ PH2D: { "element": custom } }));
    out.push(Value::Object(m));
}

/// ⭐ Um quadro → o texto de um ficheiro `.excalidraw` (X1–X4): os elementos pela ordem de z (o
/// array manda; sem `index`), os presos de cada contentor ligados nos dois sentidos.
#[must_use]
pub fn write(board: &Board) -> String {
    let mut routes = RouteCache::default();
    routes.sync(&board.doc);
    let mut els: Vec<&Element> = board.doc.live_with_hidden().collect();
    els.sort_by(|a, b| a.z.cmp(&b.z).then(a.id.cmp(&b.id)));
    // Os frames primeiro (atrás de tudo, como no nosso desenho), depois o resto pelo z.
    els.sort_by_key(|e| e.as_frame().is_none());
    let mut out = Vec::new();
    for el in &els {
        match &el.kind {
            ElementKind::Shape(s) => shape(board, el, s, &mut out),
            ElementKind::Connector(_) => arrow(board, el, &routes, &mut out),
            ElementKind::Ink(_) => freedraw(board, el, &mut out),
            ElementKind::Image(i) => {
                let m = base(board, el, "image", &[]);
                out.push(crate::image::write(board, el, i, m));
            }
            ElementKind::Embed(e) => {
                let m = base(board, el, "embeddable", &[]);
                out.push(crate::video::write(el, e, m));
            }
            ElementKind::Frame(f) => {
                let mut m = base(board, el, "frame", &[]);
                m.insert("name".into(), json!(f.name));
                m.insert("frameId".into(), Value::Null);
                let custom = serde_json::to_value(el).unwrap_or(Value::Null);
                m.insert("customData".into(), json!({ PH2D: { "element": custom } }));
                out.push(Value::Object(m));
            }
        }
    }
    // As setas presas: cada contentor diz quem se prende a ele (o `boundElements` do oráculo).
    let mut bound: BTreeMap<String, Vec<Value>> = BTreeMap::new();
    for v in &out {
        if v["type"] != "arrow" {
            continue;
        }
        for key in ["startBinding", "endBinding"] {
            if let Some(t) = v[key]["elementId"].as_str() {
                bound
                    .entry(t.to_owned())
                    .or_default()
                    .push(json!({ "id": v["id"], "type": "arrow" }));
            }
        }
    }
    for v in &mut out {
        let Some(extra) = v["id"].as_str().and_then(|i| bound.get(i)).cloned() else {
            continue;
        };
        if let Some(arr) = v["boundElements"].as_array_mut() {
            for e in extra {
                if !arr.contains(&e) {
                    arr.push(e);
                }
            }
        }
    }
    let file = json!({
        "type": "excalidraw",
        "version": 2,
        "source": "PH2D",
        "elements": out,
        "appState": { "gridSize": 20, "gridStep": 5, "gridModeEnabled": false, "viewBackgroundColor": "#ffffff" },
        "files": crate::image::files(board, &els),
    });
    // Compacto: o Excalidraw lê-o igual, e o `customData` de cada elemento não paga a indentação.
    serde_json::to_string(&file).unwrap_or_default()
}
