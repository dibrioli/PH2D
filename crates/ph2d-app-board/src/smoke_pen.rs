//! Cena 5 — a CANETA e o «Rascunho ↔ Final» (W4), módulo filho do roteador [`super`].

use super::{default_name, put_arrow, put_shape};
use ph2d_board_edit::{Tool, world_points};
use ph2d_board_model::{
    Anchor, BoardOp, BoardSet, Element, End, Ink, Pen, Rgba, STICKY_COLORS, ShapeType, Style,
};
use ph2d_editor_core::HeroScreen;
use ph2d_editor_core::screens::hero::board_view::{self, default_style};
use ph2d_editor_core::screens::hero::document_tabs;
use ph2d_editor_core::widget::panel_chrome::HIGHLIGHTER_RGBA;

/// Um traço de caneta pelos pontos `pts` (mundo).
fn put_ink(
    doc: &mut ph2d_board_model::BoardDoc,
    pts: &[[f64; 2]],
    color: Rgba,
    width: f64,
    pen: Pen,
) {
    let mut style = Style::new(None, Some(color), color);
    style.stroke_width = width;
    let world: Vec<[f64; 3]> = pts.iter().map(|p| [p[0], p[1], 0.5]).collect();
    let (ink, bx) = Ink::from_world(&world, style, pen, false);
    let el = Element::new_ink(doc.mint_id(), doc.z_on_top(), ink, bx);
    debug_assert_eq!(world_points(&el).len(), pts.len());
    BoardOp::Put(el).apply(doc);
}

/// Uma volta à mão à roda de `[cx, cy]` (raios `rx, ry`), que passa um pouco do início — o círculo
/// que alguém faz com a caneta para chamar a atenção.
fn loop_around(cx: f64, cy: f64, rx: f64, ry: f64) -> Vec<[f64; 2]> {
    (0..=56)
        .map(|i| {
            let t = f64::from(i) / 48.0 * std::f64::consts::TAU - 0.6;
            let wob = 1.0 + 0.04 * (3.0 * t).sin();
            [cx + rx * wob * t.cos(), cy + ry * wob * t.sin()]
        })
        .collect()
}

/// Cena 5: o fluxograma das cenas 2–3, mais curto, num quadro EM RASCUNHO (formas e setas à mão,
/// preenchimentos às riscas), com desenhos da caneta por cima — uma volta vermelha à roda da
/// pergunta, o marcador amarelo sobre a ideia, um visto verde — e a caneta na mão (o painel dela
/// aberto). O botão ondulado ao fundo da barra da esquerda troca o quadro para FINAL e de volta.
pub(super) fn scene_pen(hero: &mut HeroScreen) {
    let mut set = BoardSet::default();
    let id = set.create(default_name(1));
    let board = set.get_mut(id).expect("acabou de nascer");
    board.sketch = true;
    let doc = &mut board.doc;
    let mut style = default_style();
    style.sketch = true;
    let fill = |i: usize| {
        let mut s = style.clone();
        let c = Rgba(HIGHLIGHTER_RGBA[i]);
        s.fill = Some(c);
        s.text_color = c.readable_ink();
        s
    };
    let (w, h, gap) = (190.0, 100.0, 120.0);
    let col = |i: f64| i * (w + gap);
    let start = put_shape(
        doc,
        ShapeType::Pill,
        fill(2),
        "board.smoke.pen.start",
        [col(0.0), 0.0, w, h],
    );
    let idea = put_shape(
        doc,
        ShapeType::Rectangle,
        fill(3),
        "board.smoke.pen.idea",
        [col(1.0), 0.0, w, h],
    );
    let worth = put_shape(
        doc,
        ShapeType::Diamond,
        fill(0),
        "board.smoke.pen.worth",
        [col(2.0), -20.0, w, h + 40.0],
    );
    let doit = put_shape(
        doc,
        ShapeType::Rectangle,
        fill(4),
        "board.smoke.pen.do",
        [col(3.0), 0.0, w, h],
    );
    let later = put_shape(
        doc,
        ShapeType::Ellipse,
        fill(1),
        "board.smoke.pen.later",
        [col(2.0), 230.0, w, h],
    );
    let at = |t| End::Bound {
        target: t,
        anchor: Anchor::Center,
    };
    put_arrow(doc, at(start), at(idea), &style, "", &[]);
    put_arrow(doc, at(idea), at(worth), &style, "", &[]);
    put_arrow(doc, at(worth), at(doit), &style, "board.smoke.pen.yes", &[]);
    put_arrow(doc, at(worth), at(later), &style, "board.smoke.pen.no", &[]);
    // As instruções, sem contorno (texto solto, em FINAL: o que se lê não treme).
    let mut hint = default_style();
    hint.stroke = None;
    put_shape(
        doc,
        ShapeType::Rectangle,
        hint.clone(),
        "board.smoke.pen.hint_sketch",
        [col(0.0), 420.0, 2.0 * w + gap, 90.0],
    );
    put_shape(
        doc,
        ShapeType::Rectangle,
        hint,
        "board.smoke.pen.hint_pen",
        [col(2.5), 420.0, 2.0 * w + gap, 90.0],
    );
    // A caneta por cima: uma volta vermelha à roda da pergunta…
    let red = Rgba(HIGHLIGHTER_RGBA[5]);
    put_ink(
        doc,
        &loop_around(
            col(2.0) + w / 2.0,
            h / 2.0 - 20.0 + 20.0,
            w * 0.68,
            h * 0.95,
        ),
        red,
        4.0,
        Pen::Pen,
    );
    // …o marcador amarelo sobre a ideia…
    let yellow = Rgba(HIGHLIGHTER_RGBA[8]);
    let band: Vec<[f64; 2]> = (0..=24)
        .map(|i| {
            [
                col(1.0) + 15.0 + f64::from(i) * 6.7,
                h + 22.0 + (f64::from(i) * 0.7).sin() * 2.0,
            ]
        })
        .collect();
    put_ink(doc, &band, yellow, 16.0, Pen::Highlighter);
    // …e um visto verde ao lado do «fazer».
    let green = Rgba(STICKY_COLORS[6]);
    let tick = [
        [col(3.0) + w + 20.0, 40.0],
        [col(3.0) + w + 34.0, 58.0],
        [col(3.0) + w + 44.0, 72.0],
        [col(3.0) + w + 62.0, 36.0],
        [col(3.0) + w + 80.0, 0.0],
    ];
    put_ink(doc, &tick, green, 6.0, Pen::Pen);
    // As três formas da captura do dono (07/10, `capturas_excalidraw/formas_finas_do_dono.png`),
    // sem preenchimento, na espessura FINA de nascença: o rascunho ao lado do que ele mostrou.
    let mut bare = style.clone();
    bare.fill = None;
    let mut round = bare.clone();
    round.round = true;
    let y0 = 560.0;
    put_shape(
        doc,
        ShapeType::Ellipse,
        bare,
        "",
        [col(0.0), y0 + 110.0, 345.0, 188.0],
    );
    put_shape(
        doc,
        ShapeType::Rectangle,
        round.clone(),
        "",
        [col(1.2), y0, 288.0, 100.0],
    );
    put_shape(
        doc,
        ShapeType::Diamond,
        round,
        "",
        [col(2.3), y0 + 128.0, 100.0, 128.0],
    );
    board.camera.center_x = col(1.5) + w / 2.0;
    board.camera.center_y = 380.0;
    board.camera.zoom = 0.7;
    document_tabs::load(hero, set);
    hero.documents.activate(Some(id));
    board_view::set_tool(hero, Tool::Pen(Pen::Pen));
}
