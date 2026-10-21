//! Cena 28 — W23 bloco 6 (`docs/MiroClone/17_bloco6_plano.md` §7), módulo filho do roteador
//! [`super`]: o quadro «Block 6» com quatro frames numerados, cada um com a lição dele, e o quadro
//! «Plan» por trás. ① o laço (seis ideias dentro, uma quase toda dentro, uma meio fora); ② um frame
//! vazio para desenhar e segurar; ③ uma nota com um fio do Rui que CHEGA ao abrir e chama quem está a
//! usar o PH2D (o aviso, a bolinha); ④ duas caixas ligadas por uma seta para «Move to board…».
//! O quadro abre em RASCUNHO, com um rectângulo da ferramenta e um da caneta no canto de ② nascidos ao
//! zoom de abertura a 1366 (auditoria do Sketch, 21/10: de longe o rascunho ficava liso).
//! O gate `the_block6_scene_arms_its_four_stations` conduz o que a cena arma.

use ph2d_board_model::{
    Anchor, BoardId, BoardSet, ElementId, End, Pin, STICKY_SIDE, ShapeType, Who,
};
use ph2d_editor_core::HeroScreen;
use ph2d_editor_core::screens::hero::board_nav::team::notify;
use ph2d_editor_core::screens::hero::board_view::default_style;
use ph2d_editor_core::screens::hero::{board_nav, document_tabs};
use ph2d_i18n::{tr, tr_with};

/// O que o gate conduz.
#[cfg_attr(not(test), expect(dead_code, reason = "só o gate da cena lê os ids"))]
pub(crate) struct Block6 {
    pub board: BoardId,
    pub plan: BoardId,
    pub lasso_frame: ElementId,
    pub ideas: Vec<ElementId>,
    pub most: ElementId,
    pub half: ElementId,
    pub send: [ElementId; 3],
    /// Os exemplos do rascunho em ②: o da ferramenta e o da caneta.
    pub sketched: [ElementId; 2],
}

const SIDE: f64 = 140.0;
const GAP: f64 = 30.0;
/// ① o laço: 3 × 2 ideias.
const F1: [f64; 4] = [
    0.0,
    0.0,
    3.0 * SIDE + 4.0 * GAP + SIDE,
    2.0 * SIDE + 3.0 * GAP,
];
const F2: [f64; 4] = [F1[2] + 200.0, 0.0, 700.0, F1[3]];
/// A 2.ª fila desce o que as lições da 1.ª ocupam (a foto de 20/10: a 320 tapavam os títulos).
const ROW2: f64 = F1[3] + 760.0;
const F3: [f64; 4] = [0.0, ROW2, F1[2], 420.0];
const F4: [f64; 4] = [F2[0], ROW2, F2[2], 420.0];
/// Os exemplos do rascunho no canto de baixo à direita de ②: a caixa e o vão.
const EXAMPLE: [f64; 2] = [220.0, 140.0];
const EXAMPLE_GAP: f64 = 20.0;
/// A escala de nascença dos exemplos: a do zoom com que a cena abre a 1366 (~0,293, um px = 3,4 do
/// mundo; quartos de oitava, `BirthScale::of_px`).
const OPEN_BIRTH: ph2d_board_model::BirthScale = ph2d_board_model::BirthScale(7);

/// A cena; devolve os ids que o gate conduz.
pub(crate) fn scene_block6(hero: &mut HeroScreen) -> Block6 {
    let mut set = BoardSet::default();
    let board = set.create(tr("board.smoke.b6.board").to_owned());
    let plan = set.create(tr("board.smoke.b6.plan").to_owned());
    // A tinta escura da cena da equipa: o estilo de nascença sai cinzento claro na foto (20/10).
    let mut style = default_style();
    style.stroke = Some(super::team::INK);
    style.text_color = super::team::INK;
    let doc = &mut set.get_mut(board).expect("acabou de nascer").doc;
    let lasso_frame = super::organize::frame(doc, tr("board.smoke.b6.f1"), F1, None);
    let mut ideas = Vec::new();
    for i in 0..6 {
        let (c, r) = (i % 3, i / 3);
        let x = GAP + c as f64 * (SIDE + GAP);
        let y = GAP + r as f64 * (SIDE + GAP);
        ideas.push(super::put_note(
            doc,
            ShapeType::Sticky,
            1 + i,
            super::note_text("board.smoke.b6.idea"),
            [x, y, SIDE],
        ));
    }
    // Uma quase toda dentro (95 % da área) e uma meio fora, na borda direita do frame.
    let edge = F1[0] + F1[2];
    let most = super::put_note(
        doc,
        ShapeType::Sticky,
        8,
        super::note_text("board.smoke.b6.edge_most"),
        [edge - 0.95 * SIDE, GAP, SIDE],
    );
    let half = super::put_note(
        doc,
        ShapeType::Sticky,
        9,
        super::note_text("board.smoke.b6.edge_half"),
        [edge - 0.5 * SIDE, GAP + SIDE + GAP, SIDE],
    );
    super::organize::frame(doc, tr("board.smoke.b6.f2"), F2, None);
    let sketched = examples(doc, &style);
    super::organize::frame(doc, tr("board.smoke.b6.f3"), F3, None);
    let launch = super::put_note(
        doc,
        ShapeType::Sticky,
        4,
        super::note_text("board.smoke.b6.launch"),
        [F3[0] + GAP, F3[1] + GAP, SIDE * 1.5],
    );
    super::organize::frame(doc, tr("board.smoke.b6.f4"), F4, None);
    // Notas (cor e letra que se vêem com o quadro inteiro à vista; as caixas saíam pálidas na foto).
    let at = |x: f64| [F4[0] + x, F4[1] + 90.0, SIDE * 1.4];
    let a = super::put_note(
        doc,
        ShapeType::Sticky,
        3,
        super::note_text("board.smoke.b6.a"),
        at(GAP),
    );
    let b = super::put_note(
        doc,
        ShapeType::Sticky,
        6,
        super::note_text("board.smoke.b6.b"),
        at(450.0),
    );
    style.stroke_width *= 3.0;
    let bound = |target| End::Bound {
        target,
        anchor: Anchor::Center,
    };
    let arrow = super::put_arrow(doc, bound(a), bound(b), &style, "", &[]);
    for (i, f) in [F1, F2, F3, F4].into_iter().enumerate() {
        let key = format!("board.smoke.b6.l{}", i + 1);
        super::put_note(
            doc,
            ShapeType::StickyWide,
            0,
            super::note_text(&key),
            [f[0], f[1] + f[3] + GAP, f[2].min(2.0 * STICKY_SIDE * 1.6)],
        );
    }
    set.get_mut(board).expect("acabou de nascer").sketch = true;
    document_tabs::load(hero, set);
    hero.documents.activate(Some(board));
    board_nav::show(hero, false, false);
    // ③ — o fio do Rui CHEGA agora (o que estava antes não avisaria): chama quem está a usar o PH2D —
    // pelo nome do computador, que a família lê na 1.ª pintura (a foto de 20/10: chamava «Me»).
    crate::io::author(hero);
    notify::seen_now(hero);
    let me = notify::me_of(&hero.documents);
    let text = tr_with(
        "board.smoke.b6.rui_says",
        &[("name", &Who::Person(me.clone()).label())],
    );
    if let Some(b) = hero.documents.board_mut(board) {
        let pin = Pin::Element {
            id: launch,
            at: [1.0, 0.0],
        };
        b.team
            .open_calling(pin, "Rui", &text, &[Who::Person(me)], now());
    }
    Block6 {
        board,
        plan,
        lasso_frame,
        ideas,
        most,
        half,
        send: [a, b, arrow],
        sketched,
    }
}

/// ② — o rectângulo da FERRAMENTA e o da CANETA + segurar, em rascunho, nascidos ao zoom de abertura:
/// o da caneta com a cor da caneta e o resto do estilo da ferramenta (a lei do `smart_style`).
fn examples(
    doc: &mut ph2d_board_model::BoardDoc,
    style: &ph2d_board_model::Style,
) -> [ElementId; 2] {
    let mut tool = style.clone();
    tool.fill = None;
    tool.sketch = true;
    tool.reborn(OPEN_BIRTH);
    let mut pen = tool.clone();
    let ink = ph2d_board_model::Rgba(ph2d_board_model::DEFAULT_INK);
    (pen.stroke, pen.text_color) = (Some(ink), ink);
    let y = F2[1] + F2[3] - EXAMPLE[1] - EXAMPLE_GAP;
    let x = |k: f64| F2[0] + F2[2] - (2.0 - k) * (EXAMPLE[0] + EXAMPLE_GAP);
    let at = |k: f64| [x(k), y, EXAMPLE[0], EXAMPLE[1]];
    // Sem rótulo: a esta distância a letra (de mundo) é um risco; a lição ② diz qual é qual.
    [(tool, 0.0), (pen, 1.0)].map(|(style, k)| {
        let shape = ph2d_board_model::Shape {
            kind: ShapeType::Rectangle,
            style,
            text: Default::default(),
        };
        let el = ph2d_board_model::Element::new_shape(doc.mint_id(), doc.z_on_top(), shape, at(k));
        let id = el.id;
        ph2d_board_model::BoardOp::Put(el).apply(doc);
        id
    })
}

fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| i64::try_from(d.as_secs()).unwrap_or(0))
}

#[cfg(test)]
#[path = "smoke_w23_block6_tests.rs"]
mod tests;
