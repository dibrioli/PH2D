//! Cena 9 — FACILITAR (W7, `docs/MiroClone/05_facilitacao.md` §5), módulo filho do roteador [`super`]: uma
//! retrospectiva já com ideias — sete notas de cores e etiquetas variadas num frame «Ideas», duas com o
//! resultado de uma votação anterior (os selos 3 e 1) e duas com carimbos — e, por baixo, uma nota por
//! lição na ORDEM REAL (o cronómetro · votar · os turnos · carimbar · agrupar). O cartão do cronómetro
//! abre armado a 5:00. Tudo num frame com a margem do que os painéis e a barra de estado tapam (a lição
//! da cena 8). O gate `every_facilitation_lesson_does_what_it_says` conduz cada lição.

use super::default_name;
use ph2d_board_model::{
    BoardDoc, BoardOp, BoardSet, Element, ElementId, Rgba, STICKY_COLORS, STICKY_DEFAULT_COLOR,
    STICKY_SIDE, Shape, ShapeType, Stamp, Style, Tag,
};
use ph2d_editor_core::HeroScreen;
use ph2d_editor_core::screens::hero::{board_nav, document_tabs};

/// As notas que o gate conduz.
#[cfg_attr(
    not(test),
    expect(dead_code, reason = "só o gate da cena lê as lições")
)]
pub(crate) struct Lessons {
    pub ideas_frame: ElementId,
    pub ideas: [ElementId; 7],
    pub notes: [ElementId; 5],
    /// A votação de antes (W13: uma sessão do histórico, mostrada — os selos à vista).
    pub before: std::collections::BTreeMap<ElementId, u32>,
}

/// O respiro do frame à volta das notas da cena (o enquadrar já desconta os painéis).
const OUTER_PAD: f64 = 90.0;
/// A margem do frame das ideias, o vão entre as notas e entre as ideias e as lições. O vão das ideias é
/// o do modelo do Miro medido (`vote_05`: 78–80 numa nota de 236 = 0,33 do lado): com a metade, o selo
/// de 3 votos tapava o canto da nota de baixo (1.ª foto da cena 9).
const PAD: f64 = 60.0;
const NOTE_GAP: f64 = 0.33 * STICKY_SIDE;
const GAP: f64 = 60.0;
/// As ideias: `(chave do texto, cor das 16 do Miro, etiquetas, votos de antes, carimbos)`.
type Idea = (
    &'static str,
    usize,
    &'static [&'static str],
    u32,
    &'static [(Stamp, i64)],
);
const IDEAS: [Idea; 7] = [
    (
        "board.smoke.fac.i1",
        1,
        &["board.smoke.fac.tag_speed"],
        3,
        &[],
    ),
    (
        "board.smoke.fac.i2",
        1,
        &["board.smoke.fac.tag_speed"],
        0,
        &[],
    ),
    (
        "board.smoke.fac.i3",
        8,
        &["board.smoke.fac.tag_speed", "board.smoke.fac.tag_team"],
        0,
        &[(Stamp::Heart, 2)],
    ),
    (
        "board.smoke.fac.i4",
        4,
        &["board.smoke.fac.tag_team"],
        1,
        &[],
    ),
    (
        "board.smoke.fac.i5",
        4,
        &["board.smoke.fac.tag_team"],
        0,
        &[],
    ),
    ("board.smoke.fac.i6", 8, &[], 0, &[(Stamp::Party, 1)]),
    ("board.smoke.fac.i7", 12, &[], 0, &[]),
];
/// As cores das etiquetas (as do documento: o vermelho e o azul do Miro).
const TAG_RED: Rgba = Rgba(STICKY_COLORS[11]);
const TAG_BLUE: Rgba = Rgba(STICKY_COLORS[13]);

fn tr(key: &str) -> String {
    ph2d_i18n::tr(key).to_owned()
}

/// Monta a retrospectiva em `doc`.
pub(crate) fn build(doc: &mut BoardDoc) -> Lessons {
    let cols = 4.0;
    let step = STICKY_SIDE + NOTE_GAP;
    let mut before = std::collections::BTreeMap::new();
    let ideas: [ElementId; 7] = std::array::from_fn(|i| {
        let (key, color, tags, votes, stamps) = IDEAS[i];
        let at = [PAD + (i % 4) as f64 * step, PAD + (i / 4) as f64 * step];
        let mut el = note_el(doc, &tr(key), color, at);
        for t in tags {
            let c = if *t == "board.smoke.fac.tag_speed" {
                TAG_RED
            } else {
                TAG_BLUE
            };
            el.tags_mut().push(Tag {
                name: tr(t),
                color: c,
            });
        }
        for (s, n) in stamps {
            el.add_stamp(*s, *n);
        }
        let id = el.id;
        if votes > 0 {
            before.insert(id, votes);
        }
        BoardOp::Put(el).apply(doc);
        id
    });
    // O frame das ideias: o que se arrasta à volta delas para agrupar.
    let inner = [
        0.0,
        0.0,
        2.0 * PAD + cols * step - NOTE_GAP,
        2.0 * PAD + 2.0 * step,
    ];
    let ideas_frame = super::organize::frame(doc, &tr("board.smoke.fac.ideas_frame"), inner, None);
    for id in ideas {
        if let Some(mut el) = doc.get(id).cloned() {
            el.frame = Some(ideas_frame);
            BoardOp::Put(el).apply(doc);
        }
    }
    let y = inner[3] + GAP;
    let notes: [ElementId; 5] = std::array::from_fn(|i| {
        let key = ["l1", "l2", "l3", "l4", "l5"][i];
        let at = [i as f64 * (STICKY_SIDE + NOTE_GAP / 2.0), y];
        let el = note_el(
            doc,
            &tr(&format!("board.smoke.fac.{key}")),
            STICKY_DEFAULT_COLOR,
            at,
        );
        let id = el.id;
        BoardOp::Put(el).apply(doc);
        id
    });
    let mut inside = vec![ideas_frame];
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
    let bx = [
        all[0] - OUTER_PAD,
        all[1] - OUTER_PAD,
        all[2] - all[0] + 2.0 * OUTER_PAD,
        all[3] - all[1] + 2.0 * OUTER_PAD,
    ];
    let f = super::organize::frame(doc, &tr("board.smoke.fac.frame"), bx, None);
    for id in inside {
        if let Some(mut el) = doc.get(id).cloned() {
            el.frame = Some(f);
            BoardOp::Put(el).apply(doc);
        }
    }
    Lessons {
        ideas_frame,
        ideas,
        notes,
        before,
    }
}

/// Uma nota da cor `color` (das 16 do Miro) com `text`, o canto em `at` — ainda fora do documento.
fn note_el(doc: &mut BoardDoc, text: &str, color: usize, at: [f64; 2]) -> Element {
    let c = Rgba(STICKY_COLORS[color]);
    let shape = Shape {
        kind: ShapeType::Sticky,
        style: Style::new(Some(c), None, c.readable_ink()),
        text: text.into(),
    };
    Element::new_shape(
        doc.mint_id(),
        doc.z_on_top(),
        shape,
        [at[0], at[1], STICKY_SIDE, STICKY_SIDE],
    )
}

pub(super) fn scene_facilitate(hero: &mut HeroScreen) {
    let mut set = BoardSet::default();
    let id = set.create(default_name(1));
    let board = set.get_mut(id).expect("acabou de nascer");
    let l = build(&mut board.doc);
    // A votação de antes: «Voting 1» no histórico, sem data (a cena não sabe quando), mostrada.
    board.team.add_poll(l.before, "", 3, 0);
    document_tabs::load(hero, set);
    hero.documents.activate(Some(id));
    board_nav::show(hero, false, false);
    // O cartão do cronómetro abre armado (a 5:00).
    board_nav::facilitate::toggle_timer(hero);
}

#[cfg(test)]
#[path = "smoke_facilitate_tests.rs"]
mod tests;
