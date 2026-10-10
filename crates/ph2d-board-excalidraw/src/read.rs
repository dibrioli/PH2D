//! A VOLTA (X2, X3): um ficheiro `.excalidraw` → o documento de um quadro novo.

use std::collections::{BTreeMap, BTreeSet};

use ph2d_board_model::{
    BoardDoc, BoardOp, Connector, Element, ElementId, End, FracKey, Frame, GroupId, Ink, Letter,
    Pen, RichText, Route, Shape, ShapeType, Style,
};
use serde_json::Value;

use crate::bind::{Links, end_of};
use crate::style::{
    Text, dash_of, fill_style_of, num, opacity, seed_of, style_of, text_look, text_of,
};
use crate::{ID, PH2D, head_of, hex, parse_color};

/// O que a importação trouxe: o documento e o que SALTOU, contado por tipo (o aviso X3).
#[derive(Debug, Default)]
pub struct Import {
    pub doc: BoardDoc,
    pub skipped: BTreeMap<String, usize>,
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
    let by_id: BTreeMap<&str, &Value> = live
        .iter()
        .filter_map(|v| Some((v["id"].as_str()?, *v)))
        .collect();
    // Os textos presos (a um contentor que exista) e os das nossas formas que ele não tem.
    let ids: BTreeSet<&str> = live.iter().filter_map(|v| v["id"].as_str()).collect();
    let mut texts: BTreeMap<String, Text> = BTreeMap::new();
    let mut consumed: BTreeSet<&str> = BTreeSet::new();
    // As nossas formas que ele não tem (`line` com o nosso elemento), pelo grupo do par forma+texto:
    // uma cópia feita LÁ ganha grupos novos, e o texto dela fica com ela (a auditoria W8 viu o texto
    // da cópia ir parar ao original pelo `text_of`, que a cópia herda).
    let mut by_group: BTreeMap<&str, &str> = BTreeMap::new();
    for v in &live {
        if v["type"] == "line" && carried(v).is_some() {
            for g in v["groupIds"].as_array().into_iter().flatten() {
                if let (Some(g), Some(me)) = (g.as_str(), v["id"].as_str()) {
                    by_group.entry(g).or_insert(me);
                }
            }
        }
    }
    for v in &live {
        if v["type"] != "text" {
            continue;
        }
        let in_group = v["groupIds"]
            .as_array()
            .into_iter()
            .flatten()
            .find_map(|g| by_group.get(g.as_str()?))
            .map(|h| (*h).to_owned());
        let own = in_group.or_else(|| {
            v["customData"][PH2D]["text_of"]
                .as_u64()
                .map(|n| format!("{ID}{n}"))
        });
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
                | "stickynote"
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
                edits(&mut ours, v, texts.get(me), (&map, &by_id), fresh);
                ours
            }
            None => match foreign(v, id, key, texts.get(me), (&map, &by_id)) {
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
fn edits(el: &mut Element, v: &Value, text: Option<&Text>, links: Links<'_, '_>, fresh: bool) {
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
            // A forma que viajou como `line`: a caixa dos pontos que escrevemos contra a dele — um
            // redimensionar lá escala a nossa (a auditoria W8 viu-o reduzido a mover).
            let poly = crate::write::polygon(el, s);
            let p0 = poly.first().copied().unwrap_or([el.x, el.y]);
            let (pw, ph) = extent(&poly);
            let (w, h) = (
                num(v, "width").unwrap_or(pw),
                num(v, "height").unwrap_or(ph),
            );
            if !(near(w, pw) && near(h, ph)) && pw > 0.0 && ph > 0.0 {
                let (kx, ky) = (w / pw, h / ph);
                let (dx, dy) = (p0[0] - el.x, p0[1] - el.y);
                el.w *= kx;
                el.h *= ky;
                el.x = x - dx * kx;
                el.y = y - dy * ky;
            } else if !(near(x, p0[0]) && near(y, p0[1])) {
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
            let pts: Vec<[f64; 2]> = ph2d_board_edit::world_points(el)
                .iter()
                .map(|p| [p[0], p[1]])
                .collect();
            let first = pts.first().copied().unwrap_or([el.x, el.y]);
            let (pw, ph) = extent(&pts);
            let (w, h) = (
                num(v, "width").unwrap_or(pw),
                num(v, "height").unwrap_or(ph),
            );
            if !(near(w, pw) && near(h, ph)) && pw > 0.0 && ph > 0.0 {
                // Redimensionado lá: a caixa do traço escala (os pontos são relativos a ela).
                let (kx, ky) = (w / pw, h / ph);
                let (dx, dy) = (first[0] - el.x, first[1] - el.y);
                el.w *= kx;
                el.h *= ky;
                el.x = x - dx * kx;
                el.y = y - dy * ky;
            } else if !(near(x, first[0]) && near(y, first[1])) {
                el.translate([x - first[0], y - first[1]]);
            }
        }
        ph2d_board_model::ElementKind::Connector(_) => {
            let pts = world_points(v);
            if let (Some(c), Some(&a), Some(&b)) = (el.connector_mut(), pts.first(), pts.last()) {
                let n = pts.len();
                let ends = [
                    end_of(v, "startBinding", a, pts[1.min(n - 1)], links),
                    end_of(v, "endBinding", b, pts[n.saturating_sub(2)], links),
                ];
                // Uma RECTA (ou a curva DELE) com os pontos mexidos lá: os do meio são os novos pontos
                // de ajuste (a curva do Miro e o cotovelo são da nossa lei — recusa no estudo §3).
                let wrote = &v["customData"][PH2D]["wrote"];
                let moved = wrote["points"] != v["points"]
                    || num(wrote, "x").is_none_or(|w| !near(w, num(v, "x").unwrap_or(w)))
                    || num(wrote, "y").is_none_or(|w| !near(w, num(v, "y").unwrap_or(w)));
                let theirs = matches!(c.route, Route::Straight | Route::Spline);
                if theirs && moved && !wrote.is_null() {
                    c.waypoints = pts[1..pts.len() - 1].to_vec();
                }
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
    // A semente que ELE grava (uma cópia feita lá ganha outra): a nossa só fica se é a que escrevemos.
    if let Some(s) = seed_of(v).filter(|s| *s != el.seed()) {
        el.set_imported_seed(Some(s));
    }
    let ours = f64::from(crate::write::roughness(el));
    let note = el.shape().is_some_and(|s| s.kind.is_note());
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
    if num(v, "opacity").is_some_and(|o| !near(o, f64::from(st.opacity))) {
        st.opacity = opacity(v).unwrap_or(100);
    }
    if let Some(d) = dash_of(v).filter(|d| *d != st.dash) {
        st.dash = d;
    }
    if let Some(t) = text {
        if let Some(c) = t.color.filter(|c| *c != st.text_color) {
            st.text_color = c;
        }
        if !near(t.size, st.font_size) {
            st.font_size = t.size;
        }
    }
    let rough = num(v, "roughness").unwrap_or(ours);
    // O papel de uma nota não treme: a `roughness` dela não diz nada do modo.
    if !note && !near(rough, ours) {
        st.sketch = rough > 0.0;
        st.look.cartoonist = rough > 1.5;
    }
    if let Some(f) = fill_style_of(v).filter(|f| *f != st.look.fill_style) {
        st.look.fill_style = f;
    }

    if let Some(t) = text {
        if t.align != st.look.align {
            st.look.align = t.align;
        }
        if t.valign != st.look.valign {
            st.look.valign = t.valign;
        }
        // A letra só muda se ele gravou outra que a que lhe escrevemos (a do modo fica do modo).
        let wrote = crate::letter_of(Some(u64::from(crate::font_family(st))));
        if t.letter != Letter::Auto && t.letter != wrote {
            st.look.letter = t.letter;
        }
    }
    // Os cantos de uma forma NATIVA (as que viajam como `line` vão sempre sem `roundness`).
    if let Some(shape) = el.shape_mut()
        && crate::write::is_native(shape.kind)
    {
        let round = !v["roundness"].is_null();
        if round != shape.style.round {
            shape.style.round = round;
        }
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

/// A largura e a altura da caixa de `pts`.
fn extent(pts: &[[f64; 2]]) -> (f64, f64) {
    let (mut x0, mut y0, mut x1, mut y1) = (f64::MAX, f64::MAX, f64::MIN, f64::MIN);
    for p in pts {
        (x0, y0, x1, y1) = (x0.min(p[0]), y0.min(p[1]), x1.max(p[0]), y1.max(p[1]));
    }
    if pts.is_empty() {
        (0.0, 0.0)
    } else {
        (x1 - x0, y1 - y0)
    }
}

fn frame_mut(el: &mut Element) -> Option<&mut Frame> {
    match &mut el.kind {
        ph2d_board_model::ElementKind::Frame(f) => Some(f),
        _ => None,
    }
}

/// A pressão de um traço de largura constante (a do meio: o raio é o da espessura).
const CONSTANT_PRESSURE: f64 = 0.5;

/// Um elemento de FORA (sem o nosso no `customData`) pelo mapeamento directo (X3).
fn foreign(
    v: &Value,
    id: ElementId,
    z: FracKey,
    text: Option<&Text>,
    links: Links<'_, '_>,
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
            text_look(&mut st, t);
        }
        Shape {
            kind,
            style: st,
            text: text.map_or_else(RichText::default, |t| RichText::plain(&t.text)),
        }
    };
    let mut el = match kind {
        "rectangle" | "ellipse" | "diamond" | "stickynote" => {
            let k = match kind {
                "ellipse" => ShapeType::Ellipse,
                "diamond" => ShapeType::Diamond,
                "stickynote" => ShapeType::Sticky,
                _ => ShapeType::Rectangle,
            };
            if k == ShapeType::Sticky {
                // A nota do next (XV4): papel sem traço, a cor dele.
                st.stroke = None;
            }
            Element::new_shape(id, z, shape(k, st, text), bx)
        }
        "text" => {
            // Texto solto: uma forma sem preenchimento nem contorno com o texto dentro — a caixa
            // alarga a folga do texto de cada lado, e o texto fica onde ele o pôs (a caixa dele é a do
            // texto: sem isto a linha mais larga quebrava).
            let t = text_of(v)?;
            st.fill = None;
            st.stroke = None;
            let p = ph2d_board_geom::TEXT_PADDING;
            let bx = [bx[0] - p, bx[1] - p, bx[2] + 2.0 * p, bx[3] + 2.0 * p];
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
            let mut measured = !v["simulatePressure"].as_bool().unwrap_or(true) && !pr.is_empty();
            let mut world = world;
            // A caneta de largura CONSTANTE do next (XV5, `M_caneta`): a pressão do meio em todos os
            // pontos — o traço não afina.
            if v["strokeOptions"]["variability"] == "constant" {
                for p in &mut world {
                    p[2] = CONSTANT_PRESSURE;
                }
                measured = true;
            }
            let (ink, bx) = Ink::from_world(&world, st, Pen::Pen, measured);
            Element::new_ink(id, z, ink, bx)
        }
        "arrow" | "line" => {
            let pts = world_points(v);
            if pts.len() < 2 {
                return None;
            }
            // ⭐ A seta «redonda» é a curva DELE pelos pontos (XA1: recta com dois, Catmull-Rom com
            // mais) — ⛔ nunca a curva do Miro (ordem do dono, 10/10: «quero apenas que a importação
            // seja fiel»).
            let route = if v["elbowed"].as_bool().unwrap_or(false) {
                Route::Elbow
            } else if st.round {
                Route::Spline
            } else {
                Route::Straight
            };
            st.fill = None;
            st.round = false;
            if let Some(t) = text {
                st.font_size = t.size;
            }
            let n = pts.len();
            let mut c = Connector::new(
                end_of(v, "startBinding", pts[0], pts[1], links),
                end_of(v, "endBinding", pts[n - 1], pts[n - 2], links),
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
    el.set_imported_seed(seed_of(v));
    if !matches!(kind, "freedraw" | "arrow" | "line") {
        el.angle = num(v, "angle").unwrap_or(0.0);
    }
    Some(el)
}
