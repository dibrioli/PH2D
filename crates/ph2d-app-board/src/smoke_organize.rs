//! Cena 6 — ORGANIZAR um quadro grande (W5), módulo filho do roteador [`super`].
//!
//! Quatro frames de trabalho lado a lado (Pesquisa · Ideias · Votação · Plano, com «Semana 1»
//! DENTRO do Plano) e, por baixo, o frame gigante «Todas as ideias» com 2 000 notas. Abre com o
//! quadro inteiro enquadrado: de longe os frames mostram só o TÍTULO e as notas são blocos de cor;
//! o minimapa e o painel de Frames abertos — clicar num frame do painel vai até ele.
//! Na Pesquisa uma nota TRAVADA e duas com a etiqueta «cliente»; nas Ideias duas «urgente» e uma
//! com LINK para o Plano (o ícone no canto leva lá); na Votação três caixas AGRUPADAS.

use super::{default_name, put_note, put_shape};
use ph2d_board_model::{
    BoardDoc, BoardOp, BoardSet, Element, ElementId, Frame, Rgba, STICKY_SIDE, ShapeType, Style,
    Tag,
};
use ph2d_editor_core::HeroScreen;
use ph2d_editor_core::screens::hero::board_nav;
use ph2d_editor_core::screens::hero::board_view::default_style;
use ph2d_editor_core::screens::hero::document_tabs;
use ph2d_editor_core::widget::panel_chrome::HIGHLIGHTER_RGBA;

/// Um frame de trabalho: 16:9, e o vão entre dois.
const FRAME: [f64; 2] = [1600.0, 900.0];
const GAP: f64 = 400.0;
/// O frame gigante: colunas × linhas de notas M com o vão do `Tab`.
const BIG: (usize, usize) = (50, 40);
const NOTE_STEP: f64 = STICKY_SIDE * 1.1;

fn tr(key: &str) -> String {
    ph2d_i18n::tr(key).to_owned()
}

fn frame(doc: &mut BoardDoc, name: &str, bx: [f64; 4], home: Option<ElementId>) -> ElementId {
    let paper = Rgba(ph2d_board_model::FRAME_FILL);
    let f = Frame {
        name: name.to_owned(),
        style: Style::new(Some(paper), None, paper.readable_ink()),
    };
    let mut el = Element::new_frame(doc.mint_id(), doc.z_on_top(), f, bx);
    el.frame = home;
    let id = el.id;
    BoardOp::Put(el).apply(doc);
    id
}

/// Muda `id` com `f` (a casa, o travar, as etiquetas…).
fn edit(doc: &mut BoardDoc, id: ElementId, f: impl FnOnce(&mut Element)) {
    if let Some(mut el) = doc.get(id).cloned() {
        f(&mut el);
        BoardOp::Put(el).apply(doc);
    }
}

fn tag(name: &str, i: usize) -> Tag {
    Tag {
        name: name.to_owned(),
        color: Rgba(HIGHLIGHTER_RGBA[i]),
    }
}

/// Notas `keys` numa grelha de `cols` colunas dentro do frame `f` (canto `[x, y]`), na cor `c`.
fn notes_in(
    doc: &mut BoardDoc,
    f: ElementId,
    [x, y]: [f64; 2],
    cols: usize,
    c: usize,
    keys: &[&str],
) -> Vec<ElementId> {
    keys.iter()
        .enumerate()
        .map(|(i, k)| {
            let at = [
                x + (i % cols) as f64 * NOTE_STEP * 1.2,
                y + (i / cols) as f64 * NOTE_STEP * 1.2,
            ];
            let id = put_note(
                doc,
                ShapeType::Sticky,
                c,
                super::note_text(k),
                [at[0], at[1], STICKY_SIDE],
            );
            edit(doc, id, |el| el.frame = Some(f));
            id
        })
        .collect()
}

pub(super) fn scene_organize(hero: &mut HeroScreen) {
    let mut set = BoardSet::default();
    let id = set.create(default_name(1));
    let board = set.get_mut(id).expect("acabou de nascer");
    let doc = &mut board.doc;
    let col = |k: f64| k * (FRAME[0] + GAP);
    let fr = |k: f64| [col(k), 0.0, FRAME[0], FRAME[1]];
    let pad = 120.0;
    // 1 · Pesquisa: uma nota TRAVADA (as regras) e duas da etiqueta «cliente».
    let research = frame(doc, &tr("board.smoke.org.f_research"), fr(0.0), None);
    let r = notes_in(
        doc,
        research,
        [col(0.0) + pad, pad],
        3,
        3,
        &[
            "board.smoke.org.rules",
            "board.smoke.org.r1",
            "board.smoke.org.r2",
            "board.smoke.org.r3",
        ],
    );
    edit(doc, r[0], |el| el.set_locked(true));
    for n in &r[1..3] {
        edit(doc, *n, |el| {
            el.tags_mut()
                .push(tag(&tr("board.smoke.org.tag_client"), 3))
        });
    }
    // 2 · Ideias: duas «urgente» e uma com link para o Plano.
    let ideas = frame(doc, &tr("board.smoke.org.f_ideas"), fr(1.0), None);
    let i = notes_in(
        doc,
        ideas,
        [col(1.0) + pad, pad],
        4,
        1,
        &[
            "board.smoke.org.i1",
            "board.smoke.org.i2",
            "board.smoke.org.i3",
            "board.smoke.org.i4",
            "board.smoke.org.see_plan",
        ],
    );
    for n in &i[..2] {
        edit(doc, *n, |el| {
            el.tags_mut()
                .push(tag(&tr("board.smoke.org.tag_urgent"), 1))
        });
    }
    // 3 · Votação: três caixas AGRUPADAS (movem-se juntas; duplo-clique entra no grupo).
    let vote = frame(doc, &tr("board.smoke.org.f_vote"), fr(2.0), None);
    let g = doc.mint_group();
    let mut st = default_style();
    st.fill = Some(Rgba(HIGHLIGHTER_RGBA[2]));
    st.text_color = Rgba(HIGHLIGHTER_RGBA[2]).readable_ink();
    for (k, key) in [
        "board.smoke.org.v1",
        "board.smoke.org.v2",
        "board.smoke.org.v3",
    ]
    .iter()
    .enumerate()
    {
        let bx = [col(2.0) + pad + k as f64 * 420.0, 300.0, 360.0, 220.0];
        let s = put_shape(doc, ShapeType::Rectangle, st.clone(), key, bx);
        edit(doc, s, |el| {
            el.frame = Some(vote);
            el.groups_mut().push(g);
        });
    }
    // 4 · Plano, com «Semana 1» DENTRO dele (mover o Plano leva a semana e as notas dela).
    let plan = frame(doc, &tr("board.smoke.org.f_plan"), fr(3.0), None);
    let week = frame(
        doc,
        &tr("board.smoke.org.f_week"),
        [col(3.0) + pad, pad, 900.0, 600.0],
        Some(plan),
    );
    notes_in(
        doc,
        week,
        [col(3.0) + 2.0 * pad, 2.0 * pad],
        3,
        6,
        &["board.smoke.org.w1", "board.smoke.org.w2"],
    );
    edit(doc, i[4], |el| el.set_link(Some(plan)));
    // 5 · O frame gigante, por baixo: 2 000 notas nas 16 cores.
    let (cols, rows) = BIG;
    let top = FRAME[1] + GAP;
    let big = frame(
        doc,
        &tr("board.smoke.org.f_all"),
        [
            0.0,
            top,
            cols as f64 * NOTE_STEP + 2.0 * pad,
            rows as f64 * NOTE_STEP + 2.0 * pad,
        ],
        None,
    );
    for k in 0..cols * rows {
        let at = [
            pad + (k % cols) as f64 * NOTE_STEP,
            top + pad + (k / cols) as f64 * NOTE_STEP,
        ];
        let text = ph2d_i18n::tr_with("board.smoke.org.idea_n", &[("n", &(k + 1))]);
        let n = put_note(
            doc,
            ShapeType::Sticky,
            k % 16,
            text.into(),
            [at[0], at[1], STICKY_SIDE],
        );
        edit(doc, n, |el| el.frame = Some(big));
    }
    document_tabs::load(hero, set);
    hero.documents.activate(Some(id));
    board_nav::show(hero, true, true);
}
