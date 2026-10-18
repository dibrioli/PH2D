//! Cena 8 — PLANEAR (W6, `docs/MiroClone/04_planear_mapa_e_modelos.md` §5), módulo filho do
//! roteador [`super`]: um mapa mental começado — «Plan» com «Ideas» (dois filhos) e «Risks»
//! (três filhos, RECOLHIDO: o número à vista) à direita, «Next steps» à esquerda — e, por baixo, uma
//! nota por lição, com os nomes dos nós que ela usa (a 6.ª aponta o botão dos modelos).
//! ⚠️ A vista NÃO se escolhe aqui: o 1.º quadro enquadra o quadro inteiro (handoff W5b §10.1). Por
//! isso a galeria não abre sozinha: aberta, tapava o ramo da esquerda (foto de 08/10).
//! O mapa faz-se com o editor do quadro (o texto dos nós mede-se), e o gate
//! `every_mind_lesson_does_what_it_says` conduz cada lição.

use super::default_name;
use ph2d_board_edit::Editor;
use ph2d_board_model::{
    BoardDoc, BoardOp, BoardSet, Element, ElementId, History, Rgba, STICKY_COLORS,
    STICKY_DEFAULT_COLOR, STICKY_SIDE, Shape, ShapeType, Side, Style,
};
use ph2d_editor_core::HeroScreen;
use ph2d_editor_core::screens::hero::board_view::{default_style, metrics};
use ph2d_editor_core::screens::hero::{board_nav, document_tabs};
use ph2d_text::TextSystem;

/// Os nós e as notas que o gate conduz.
#[cfg_attr(
    not(test),
    expect(dead_code, reason = "só o gate da cena lê as lições")
)]
pub(crate) struct Lessons {
    pub root: ElementId,
    pub ideas: ElementId,
    pub sketch: ElementId,
    pub risks: ElementId,
    pub next: ElementId,
    pub notes: [ElementId; 6],
}

/// O respiro do frame à volta das notas da cena (o enquadrar já desconta os painéis).
const PAD: f64 = 90.0;

/// O vão entre o mapa e as notas, e entre as notas.
const GAP: f64 = 40.0;
const NOTE_GAP: f64 = 16.0;

fn tr(key: &str) -> String {
    ph2d_i18n::tr(key).to_owned()
}

/// O texto de uma lição com os nomes dos nós que ela usa (`{node}`, `{to}` → a chave do nome).
pub(crate) fn lesson(key: &str, names: &[(&str, &str)]) -> String {
    names
        .iter()
        .fold(tr(key), |t, (slot, name)| t.replace(slot, &tr(name)))
}

/// Monta o mapa e as notas em `doc`.
pub(crate) fn build(doc: &mut BoardDoc) -> Lessons {
    let mut ed = Editor::new(default_style(), metrics());
    let (mut h, mut ts) = (History::default(), TextSystem::without_system_fonts());
    let root = ed.mind_new_map(doc, &mut h, &mut ts, [0.0, 0.0]);
    ed.text_input(doc, &mut ts, &tr("board.smoke.mind.root"));
    let mut child = |ed: &mut Editor, doc: &mut BoardDoc, parent, side, key: &str| {
        ed.commit_text(doc, &mut h);
        ed.select(doc, [parent]);
        ed.mind_child(doc, &mut h, &mut ts, side);
        ed.text_input(doc, &mut ts, &tr(key));
        let id = ed.mind_focus(doc).expect("o filho novo");
        ed.commit_text(doc, &mut h);
        id
    };
    let ideas = child(
        &mut ed,
        doc,
        root,
        Some(Side::After),
        "board.smoke.mind.ideas",
    );
    let risks = child(
        &mut ed,
        doc,
        root,
        Some(Side::After),
        "board.smoke.mind.risks",
    );
    let next = child(
        &mut ed,
        doc,
        root,
        Some(Side::Before),
        "board.smoke.mind.next",
    );
    let sketch = child(&mut ed, doc, ideas, None, "board.smoke.mind.sketch");
    child(&mut ed, doc, ideas, None, "board.smoke.mind.proto");
    for k in [
        "board.smoke.mind.r1",
        "board.smoke.mind.r2",
        "board.smoke.mind.r3",
    ] {
        child(&mut ed, doc, risks, None, k);
    }
    child(&mut ed, doc, next, None, "board.smoke.mind.ask");
    ed.select(doc, [risks]);
    ed.mind_collapse(doc, &mut h, None);

    // As lições, por baixo do mapa: duas filas de três notas, centradas nele.
    let map: Vec<ElementId> = std::iter::once(root)
        .chain(doc.mind_descendants(root))
        .collect();
    let bb = map
        .iter()
        .filter_map(|id| doc.get(*id))
        .map(Element::aabb)
        .fold(
            [
                f64::INFINITY,
                f64::INFINITY,
                f64::NEG_INFINITY,
                f64::NEG_INFINITY,
            ],
            |a, b| {
                [
                    a[0].min(b[0]),
                    a[1].min(b[1]),
                    a[2].max(b[2]),
                    a[3].max(b[3]),
                ]
            },
        );
    let texts = [
        lesson(
            "board.smoke.mind.l1",
            &[("{node}", "board.smoke.mind.ideas")],
        ),
        lesson("board.smoke.mind.l2", &[]),
        lesson("board.smoke.mind.l3", &[]),
        lesson(
            "board.smoke.mind.l4",
            &[("{node}", "board.smoke.mind.risks")],
        ),
        lesson(
            "board.smoke.mind.l5",
            &[
                ("{node}", "board.smoke.mind.sketch"),
                ("{to}", "board.smoke.mind.next"),
            ],
        ),
        lesson("board.smoke.mind.l6", &[]),
    ];
    let row = 3.0 * STICKY_SIDE + 2.0 * NOTE_GAP;
    let x0 = (bb[0] + bb[2]) / 2.0 - row / 2.0;
    let y0 = bb[3] + GAP;
    let notes: [ElementId; 6] = std::array::from_fn(|i| {
        let (c, r) = ((i % 3) as f64, (i / 3) as f64);
        let at = [
            x0 + c * (STICKY_SIDE + NOTE_GAP),
            y0 + r * (STICKY_SIDE + NOTE_GAP),
        ];
        note(doc, &texts[i], at)
    });
    // Tudo dentro de um frame com margem: o 1.º quadro enquadra o quadro inteiro na JANELA (os
    // painéis e a barra de estado tapam as bordas — fotos de 08/10), e a margem do frame é o que
    // fica por baixo deles.
    let mut inside: Vec<ElementId> = map.clone();
    inside.extend(notes);
    let all = inside
        .iter()
        .filter_map(|id| doc.get(*id))
        .map(Element::aabb)
        .fold(
            [
                f64::INFINITY,
                f64::INFINITY,
                f64::NEG_INFINITY,
                f64::NEG_INFINITY,
            ],
            |a, b| {
                [
                    a[0].min(b[0]),
                    a[1].min(b[1]),
                    a[2].max(b[2]),
                    a[3].max(b[3]),
                ]
            },
        );
    let (mx, my) = (PAD, PAD);
    let bx = [
        all[0] - mx,
        all[1] - my,
        all[2] - all[0] + 2.0 * mx,
        all[3] - all[1] + 2.0 * my,
    ];
    let f = super::organize::frame(doc, &tr("board.smoke.mind.frame"), bx, None);
    for id in inside {
        if let Some(mut el) = doc.get(id).cloned() {
            el.frame = Some(f);
            BoardOp::Put(el).apply(doc);
        }
    }
    Lessons {
        root,
        ideas,
        sketch,
        risks,
        next,
        notes,
    }
}

/// Uma nota amarela (a de nascença do Miro) com `text`, o canto em `at`.
fn note(doc: &mut BoardDoc, text: &str, at: [f64; 2]) -> ElementId {
    let color = Rgba(STICKY_COLORS[STICKY_DEFAULT_COLOR]);
    let style = Style::new(Some(color), None, color.readable_ink());
    let shape = Shape {
        kind: ShapeType::Sticky,
        style,
        text: text.into(),
    };
    let el = Element::new_shape(
        doc.mint_id(),
        doc.z_on_top(),
        shape,
        [at[0], at[1], STICKY_SIDE, STICKY_SIDE],
    );
    let id = el.id;
    BoardOp::Put(el).apply(doc);
    id
}

pub(super) fn scene_mind(hero: &mut HeroScreen) {
    let mut set = BoardSet::default();
    let id = set.create(default_name(1));
    let board = set.get_mut(id).expect("acabou de nascer");
    build(&mut board.doc);
    document_tabs::load(hero, set);
    hero.documents.activate(Some(id));
    board_nav::show(hero, false, false);
}

#[cfg(test)]
#[path = "smoke_mind_tests.rs"]
mod tests;
