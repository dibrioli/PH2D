//! ⭐ O RASCUNHO medido pela TINTA no ecrã, pelo caminho REAL (auditoria 21/10, smoke do dono na cena
//! 28: «sketch não tem aspecto de rascunho», «apertei e nada mudou»). A cena abre a ~0,29: o tremor
//! do rascunho era de MUNDO e lá caía abaixo de 1 px (doc 17 §9, «Auditoria do Sketch»). A régua: o
//! que o pintor GRAVOU, cada traço posto em contorno, varrido em colunas de ecrã — nada de campos.

use super::*;
use ph2d_vector::{
    BezPath, DrawOp, ParamCurveNearest, PathEl, Point, Shape as _, StrokeOpts, expand_stroke,
    flatten,
};

/// O zoom com que a cena 28 abre a 1366 (o frame «Draw here», 700 de largura, em ~205 px).
const FAR: f64 = 0.293;
/// A faixa de cada lado de uma aresta onde se procura a tinta dela (px de ecrã).
const BAND: f64 = 12.0;
/// Dois troços de tinta separados por menos que isto lêem-se como um (px de ecrã).
const SEEN_GAP: f64 = 0.75;

impl T {
    fn set_zoom(&mut self, z: f64) {
        self.hero
            .documents
            .active_board_mut()
            .expect("um quadro")
            .camera
            .zoom = z;
        self.paint();
    }

    fn deselect(&mut self) {
        let doc = self
            .hero
            .documents
            .active_board()
            .expect("um quadro")
            .doc
            .clone();
        self.board_editor().select(&doc, []);
        self.paint();
    }

    fn board_sketch(&mut self, on: bool) {
        self.hero
            .documents
            .active_board_mut()
            .expect("um quadro")
            .sketch = on;
    }

    /// A tinta pintada, em contornos de ECRÃ (o traço expandido pela espessura dele, como a placa o
    /// enche); sem os pontos da grelha.
    fn ink(&mut self) -> Vec<BezPath> {
        let mut scene = VectorScene::new();
        scene.record();
        let vp = Rect::new(0.0, 0.0, self.vp_w, self.vp_h);
        paint_hero_screen(&mut self.hero, vp, &mut scene, &mut self.ts);
        scene
            .take_record()
            .into_iter()
            .filter_map(|op| match op {
                DrawOp::Fill {
                    transform, path, ..
                } => Some(transform * path),
                DrawOp::Stroke {
                    style,
                    transform,
                    path,
                    ..
                } => Some(transform * expand_stroke(path, &style, &StrokeOpts::default(), 0.005)),
                _ => None,
            })
            .filter(|p| {
                let b = p.bounding_box();
                b.width().max(b.height()) > 3.0
            })
            .collect()
    }

    /// A caixa no ecrã do elemento `el` (sem rodar).
    fn el_screen_box(&self, el: &Element) -> kurbo_rect::R {
        let b = self.hero.documents.active_board().expect("um quadro");
        let c = self.hero.last_canvas;
        let v = ph2d_board_render::view(&b.camera, board_view::area_of(c));
        let p0 = v * Point::new(el.x, el.y);
        let p1 = v * Point::new(el.x + el.w, el.y + el.h);
        kurbo_rect::R {
            x0: p0.x,
            y0: p0.y,
            x1: p1.x,
            y1: p1.y,
        }
    }
}

mod kurbo_rect {
    #[derive(Clone, Copy, Debug)]
    pub struct R {
        pub x0: f64,
        pub y0: f64,
        pub x1: f64,
        pub y1: f64,
    }
}

/// A tinta CONTIDA na caixa `bx` com a folga [`BAND`] (o fundo do quadro e a interface ficam de fora).
fn near(ink: &[BezPath], bx: kurbo_rect::R) -> Vec<BezPath> {
    ink.iter()
        .filter(|p| {
            let b = p.bounding_box();
            b.x0 >= bx.x0 - BAND
                && b.x1 <= bx.x1 + BAND
                && b.y0 >= bx.y0 - BAND
                && b.y1 <= bx.y1 + BAND
        })
        .cloned()
        .collect()
}

/// Os troços `[y0, y1]` de tinta na coluna `x`, dentro de `[lo, hi]` (regra não-nula por contorno, a
/// união de todos).
fn column(ink: &[BezPath], x: f64, lo: f64, hi: f64) -> Vec<[f64; 2]> {
    let mut runs: Vec<[f64; 2]> = Vec::new();
    for p in ink {
        let b = p.bounding_box();
        if x < b.x0 || x > b.x1 || b.y1 < lo || b.y0 > hi {
            continue;
        }
        let mut hits: Vec<(f64, i32)> = Vec::new();
        let (mut cur, mut start) = (Point::ZERO, Point::ZERO);
        flatten(p.iter(), 0.01, |el| match el {
            PathEl::MoveTo(q) => (cur, start) = (q, q),
            PathEl::LineTo(q) => {
                cross(cur, q, x, &mut hits);
                cur = q;
            }
            PathEl::ClosePath => {
                cross(cur, start, x, &mut hits);
                cur = start;
            }
            _ => {}
        });
        hits.sort_by(|a, b| a.0.total_cmp(&b.0));
        let mut wind = 0;
        for w in hits.windows(2) {
            wind += w[0].1;
            if wind != 0 {
                let (a, b) = (w[0].0.max(lo), w[1].0.min(hi));
                if b > a {
                    runs.push([a, b]);
                }
            }
        }
    }
    runs.sort_by(|a, b| a[0].total_cmp(&b[0]));
    let mut merged: Vec<[f64; 2]> = Vec::new();
    for r in runs {
        match merged.last_mut() {
            Some(m) if r[0] <= m[1] => m[1] = m[1].max(r[1]),
            _ => merged.push(r),
        }
    }
    merged
}

fn cross(a: Point, b: Point, x: f64, hits: &mut Vec<(f64, i32)>) {
    if (a.x <= x) != (b.x <= x) {
        let t = (x - a.x) / (b.x - a.x);
        hits.push((a.y + t * (b.y - a.y), if b.x > a.x { 1 } else { -1 }));
    }
}

/// A aresta de CIMA de `bx` lida na tinta, nas colunas do meio (25–75 %): (afastamento mediano entre
/// a tinta mais alta e a mais baixa, troços que se vêem separados — média, espessura mediana de um
/// troço), tudo em px de ecrã.
fn top_edge(ink: &[BezPath], bx: kurbo_rect::R) -> (f64, f64, f64) {
    let ink = &near(ink, bx);
    let (mut spread, mut seen, mut thick) = (Vec::new(), 0.0, Vec::new());
    let n = 41;
    for k in 0..n {
        let x = bx.x0 + (bx.x1 - bx.x0) * (0.25 + 0.5 * f64::from(k) / f64::from(n - 1));
        let runs = column(ink, x, bx.y0 - BAND, bx.y0 + BAND);
        if runs.is_empty() {
            continue;
        }
        spread.push(runs[runs.len() - 1][1] - runs[0][0]);
        seen += 1.0
            + runs
                .windows(2)
                .filter(|w| w[1][0] - w[0][1] >= SEEN_GAP)
                .count() as f64;
        thick.extend(runs.iter().map(|r| r[1] - r[0]));
    }
    assert!(
        !spread.is_empty(),
        "nenhuma tinta na aresta de cima de {bx:?}"
    );
    let median = |v: &mut Vec<f64>| {
        v.sort_by(f64::total_cmp);
        v[v.len() / 2]
    };
    let cols = spread.len() as f64;
    (median(&mut spread), seen / cols, median(&mut thick))
}

/// A ferramenta Retângulo arrasta um rectângulo de `w × h` px de ecrã; devolve-o.
fn tool_rect(t: &mut T, w: f32, h: f32) -> Element {
    t.board_editor().tool = ph2d_board_edit::Tool::Shape(ph2d_board_model::ShapeType::Rectangle);
    let a = t.at(0.15, 0.15);
    t.drag(a, (a.0 + w, a.1 + h));
    t.board_editor().tool = ph2d_board_edit::Tool::Select;
    t.key(board_keys::BoardKey::Escape, NO_MODS, None);
    t.elements().last().expect("o rectângulo").clone()
}

const NO_MODS: Modifiers = Modifiers {
    shift: false,
    ctrl: false,
    alt: false,
    meta: false,
};

/// O mesmo gesto no ECRÃ, de perto (zoom 1) e de longe (`FAR`), num quadro em rascunho: (aresta de
/// perto, aresta de longe).
fn near_and_far(draw: impl Fn(&mut T) -> Element) -> ((f64, f64, f64), (f64, f64, f64)) {
    let mut out = Vec::new();
    for z in [1.0, FAR] {
        let mut t = T::new();
        t.set_zoom(z);
        t.board_sketch(true);
        t.board_editor().style.fill = None;
        let el = draw(&mut t);
        t.deselect();
        t.paint();
        let ink = t.ink();
        out.push(top_edge(&ink, t.el_screen_box(&el)));
    }
    (out[0], out[1])
}

fn reads_as_the_near_one(name: &str, near: (f64, f64, f64), far: (f64, f64, f64)) {
    // O rascunho de longe lê-se como o de perto: o afastamento das passagens e a espessura no ecrã.
    assert!(
        far.0 >= 0.75 * near.0 && far.0 >= 1.5,
        "{name}: afastamento das passagens no ecrã — perto {:.2} px, longe {:.2} px",
        near.0,
        far.0
    );
    assert!(
        far.2 >= 0.75 * near.2,
        "{name}: espessura de um troço no ecrã — perto {:.2} px, longe {:.2} px",
        near.2,
        far.2
    );
}

/// ⭐ Um rectângulo da FERRAMENTA desenhado com o quadro afastado (o zoom da cena 28) lê-se no ecrã
/// tão à mão como o mesmo gesto de perto — decisão do dono 21/10: «nasce no tamanho do ecrã».
#[test]
fn a_tool_shape_drawn_far_reads_as_sketched_as_one_drawn_near() {
    let (near, far) = near_and_far(|t| tool_rect(t, 200.0, 130.0));
    reads_as_the_near_one("ferramenta", near, far);
}

/// ⭐ O mesmo para a forma da CANETA + segurar.
#[test]
fn a_pen_shape_drawn_far_reads_as_sketched_as_one_drawn_near() {
    let (near, far) = near_and_far(|t| {
        let end = t.pen_hold(&hand::hand_rect(), false);
        t.release(end, false);
        t.board_editor().tool = ph2d_board_edit::Tool::Select;
        t.key(board_keys::BoardKey::Escape, NO_MODS, None);
        t.only()
    });
    reads_as_the_near_one("caneta", near, far);
}

/// ⭐ O CONTROLO, pela tinta (o «apertei e nada mudou» do dono): com o quadro afastado, um rectângulo
/// FINAL já desenhado, seleccionado, e o botão do rascunho — a aresta passa de um traço a várias
/// passagens à vista.
#[test]
fn the_sketch_button_visibly_changes_a_far_rectangle() {
    let mut t = T::new();
    t.set_zoom(FAR);
    t.board_editor().style.fill = None;
    let el = tool_rect(&mut t, 200.0, 130.0);
    assert!(!el.style().sketch);
    let bx = t.el_screen_box(&el);
    let ink = t.ink();
    let before = top_edge(&ink, bx);
    // Seleccionar pela aresta (o rectângulo é oco) e carregar no botão.
    let on_edge = ((bx.x0 + bx.x1) as f32 / 2.0, bx.y0 as f32);
    t.drag(on_edge, on_edge);
    assert_eq!(
        t.board_editor().selection().len(),
        1,
        "o rectângulo seleccionado"
    );
    t.click_bar(Item::Sketch);
    assert!(t.elements().iter().all(|e| e.style().sketch));
    t.deselect();
    t.paint();
    let ink = t.ink();
    let after = top_edge(&ink, bx);
    assert!(
        // De perto, o mesmo rectângulo à mão afasta as passagens ~4,5 px; final, uma linha só.
        after.0 - before.0 >= 2.5,
        "a tinta não mudou à vista: antes {before:?}, depois {after:?}"
    );
}

/// O que fica TINTA LIMPA da caneta + segurar (o triângulo qualquer) num quadro em rascunho pinta-se
/// à mão como as formas — e o botão alcança-o (num quadro final volta direito).
#[test]
fn the_pens_clean_triangle_is_sketched_in_a_sketch_board_and_the_button_reaches_it() {
    let worst = |t: &mut T| -> f64 {
        let el = t.only();
        let ink = el.ink().expect("tinta limpa").clone();
        let b = t.hero.documents.active_board().unwrap();
        let v = ph2d_board_render::view(&b.camera, board_view::area_of(t.hero.last_canvas))
            * ph2d_board_geom::to_world(&el);
        let exact: Vec<Point> = ink
            .points
            .iter()
            .map(|p| v * Point::new(f64::from(p[0]), f64::from(p[1])))
            .collect();
        // O pior afastamento entre a tinta pintada e a geometria limpa da lei (px de ecrã).
        let mut far = 0.0f64;
        let painted = t.ink();
        let bx = t.el_screen_box(&el);
        for p in &near(&painted, bx) {
            flatten(p.iter(), 0.05, |e| {
                if let PathEl::MoveTo(q) | PathEl::LineTo(q) = e {
                    let d = exact
                        .windows(2)
                        .map(|w| {
                            ph2d_vector::Line::new(w[0], w[1])
                                .nearest(q, 1e-6)
                                .distance_sq
                                .sqrt()
                        })
                        .fold(f64::INFINITY, f64::min);
                    far = far.max(d);
                }
            });
        }
        far
    };
    let mut t = T::new();
    t.board_sketch(true);
    let end = t.pen_hold(&hand::hand_triangle(), false);
    t.release(end, false);
    t.board_editor().tool = ph2d_board_edit::Tool::Select;
    t.key(board_keys::BoardKey::Escape, NO_MODS, None);
    assert!(
        t.only().ink().is_some(),
        "o triângulo qualquer fica tinta limpa"
    );
    let sketched = worst(&mut t);
    assert!(
        sketched >= 1.5,
        "a tinta limpa num quadro em rascunho saiu direita: {sketched:.2} px"
    );
    // O botão sem selecção (o quadro inteiro) alcança-a: volta direita.
    t.click_bar(Item::Sketch);
    assert!(
        !t.only().style().sketch,
        "o botão não alcançou a tinta limpa"
    );
    let plain = worst(&mut t);
    assert!(
        plain < 1.0,
        "final, a tinta limpa devia ficar sobre a lei: {plain:.2} px"
    );
}

/// O botão nunca fica MORTO: com uma selecção que ele não alcança (um traço livre da caneta), ele
/// troca o quadro como sem selecção — e a luz diz o que o clique vai trocar.
#[test]
fn the_sketch_button_with_only_freehand_ink_selected_toggles_the_board() {
    let mut t = T::new();
    t.board_editor().tool = ph2d_board_edit::Tool::Pen(ph2d_board_model::Pen::Pen);
    let (a, b) = (t.at(0.3, 0.5), t.at(0.6, 0.5));
    t.drag(a, b);
    t.board_editor().tool = ph2d_board_edit::Tool::Select;
    let mid = ((a.0 + b.0) / 2.0, a.1);
    t.drag(mid, mid);
    assert_eq!(
        t.board_editor().selection().len(),
        1,
        "o traço seleccionado"
    );
    let board = |t: &T| t.hero.documents.active_board().unwrap().sketch;
    let lit = |t: &mut T| {
        let b = t.hero.documents.active_board().unwrap().clone();
        crate::screens::hero::board_bar::look::sketch_on(t.board_editor(), &b.doc, b.sketch)
    };
    assert!(!board(&t) && !lit(&mut t));
    t.click_bar(Item::Sketch);
    assert!(board(&t), "o clique não fez nada (o botão morto)");
    assert!(lit(&mut t), "a luz não acompanhou o clique");
}

/// O `Ctrl+Z` do botão sem selecção desfaz também o MODO do quadro (a luz e o que nasce depois).
#[test]
fn undo_of_the_board_sketch_toggle_gives_the_mode_back() {
    let mut t = T::new();
    tool_rect(&mut t, 120.0, 80.0);
    t.click_bar(Item::Sketch);
    let board = |t: &T| t.hero.documents.active_board().unwrap().sketch;
    assert!(board(&t));
    let ctrl = Modifiers {
        ctrl: true,
        ..NO_MODS
    };
    assert!(t.key(board_keys::BoardKey::Char('z'), ctrl, None));
    assert!(t.elements().iter().all(|e| !e.style().sketch));
    assert!(
        !board(&t),
        "o Ctrl+Z deixou o quadro em rascunho com as formas finais"
    );
    let redo = Modifiers {
        shift: true,
        ..ctrl
    };
    assert!(t.key(board_keys::BoardKey::Char('z'), redo, None));
    assert!(board(&t), "o refazer não repôs o modo");
    assert!(t.elements().iter().all(|e| e.style().sketch));
}
