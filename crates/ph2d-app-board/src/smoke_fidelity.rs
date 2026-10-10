//! Cena 11 — A FIDELIDADE AO EXCALIDRAW (W9, `docs/MiroClone/07_fidelidade_excalidraw.md` §6.3), módulo
//! filho do roteador [`super`]: o `bv.excalidraw` do PRÓPRIO dono aberto pelo leitor (as três setas
//! rectas, paradas antes das formas, como na foto `docs/MiroClone/bv_excalidraw.png`, o SVG do oráculo
//! `saidas/w9_dono_bv.next.svg` pelo `resvg` a 2×),
//! uma fila de setas curtas de 10 a 60 (a ponta encolhe), duas caixas para desenhar uma seta que pára
//! antes da forma, uma caixa com a letra do ficheiro que segue o botão «Rascunho ↔ Final» (W10, a lição 5
//! é ela mesma), e uma nota LARGA por lição na ORDEM REAL. O gate `every_fidelity_lesson_does_what_it_says`
//! conduz cada lição pelo mesmo caminho.

use super::default_name;
use ph2d_board_model::{
    Board, BoardOp, BoardSet, Connector, Element, ElementId, End, Head, Letter, Rgba, RichText,
    Route, STICKY_COLORS, STICKY_SIDE, STICKY_WIDE, Shape, ShapeType, Style,
};
use ph2d_editor_core::HeroScreen;
use ph2d_editor_core::screens::hero::{board_nav, document_tabs};

/// O ficheiro do dono (o `bv.excalidraw` do smoke 10/10, cópia de
/// `ferramentas/excalidraw_oracle/entradas/w9_dono_bv.excalidraw`) — o MESMO que a lição 2 manda abrir
/// pelo menu da aba. ⚠️ Nome curto: o caminho longo partia-se a meio na nota (foto da cena, 10/10).
pub(crate) const OWNER_FILE: &str = include_str!("../../../docs/MiroClone/bv.excalidraw");
/// Onde o desenho do dono assenta: o canto dele (≈ 734 × 256 no ficheiro) vai para ≈ (34, 56).
pub(crate) const OWNER_SHIFT: [f64; 2] = [-700.0, -200.0];
/// Os comprimentos das setas curtas (a lei XA2: a ponta mede `min(25, L/2)`).
pub(crate) const SHORT: [f64; 6] = [10.0, 20.0, 30.0, 40.0, 50.0, 60.0];
/// A espessura delas (a de nascença do Excalidraw, a do ficheiro do oráculo).
const SHORT_WIDTH: f64 = 2.0;
/// A fila das setas curtas: onde começa, e o passo entre elas.
const ROW: [f64; 2] = [20.0, 600.0];
const STEP: f64 = 110.0;
/// A margem do que os painéis e a barra de estado tapam (a da cena 10, medida lá).
const MARGIN_X: f64 = 450.0;
const MARGIN_Y: f64 = 90.0;
const GAP: f64 = 40.0;
const INK: Rgba = Rgba([30, 30, 30, 255]);

fn tr(key: &str) -> String {
    ph2d_i18n::tr(key).to_owned()
}

fn put(b: &mut Board, el: Element) -> ElementId {
    let id = el.id;
    BoardOp::Put(el).apply(&mut b.doc);
    id
}

/// O que o gate conduz.
#[cfg_attr(
    not(test),
    expect(dead_code, reason = "só o gate da cena lê as lições")
)]
pub(crate) struct Lessons {
    /// As três setas do dono, pela ordem do ficheiro.
    pub owner: Vec<ElementId>,
    /// As setas curtas, pela ordem de [`SHORT`].
    pub short: Vec<ElementId>,
    pub from: ElementId,
    pub to: ElementId,
    pub lessons: [ElementId; 4],
    /// A caixa da lição 5 (W10): à mão e com a letra do ficheiro fixa (`Letter::Sans`), como chega
    /// de um `.excalidraw` — o botão «Rascunho ↔ Final» devolve-a ao modo.
    pub letter: ElementId,
    /// O que a cena enquadra ao nascer: tudo, mais a margem do que os painéis tapam.
    pub view: [f64; 4],
}

/// Um texto solto (sem fundo nem contorno).
fn label(b: &mut Board, text: &str, bx: [f64; 4]) -> ElementId {
    let shape = Shape {
        kind: ShapeType::Rectangle,
        style: Style::new(None, None, INK),
        text: RichText::plain(text),
    };
    let el = Element::new_shape(b.doc.mint_id(), b.doc.z_on_top(), shape, bx);
    put(b, el)
}

fn boxed(b: &mut Board, key: &str, color: usize, bx: [f64; 4]) -> ElementId {
    let shape = Shape {
        kind: ShapeType::Rectangle,
        style: Style::new(Some(Rgba(STICKY_COLORS[color])), Some(INK), INK),
        text: RichText::plain(&tr(key)),
    };
    let el = Element::new_shape(b.doc.mint_id(), b.doc.z_on_top(), shape, bx);
    put(b, el)
}

/// Monta a cena em `b` (o documento dele é SUBSTITUÍDO pelo do ficheiro do dono).
pub(crate) fn build(b: &mut Board) -> Lessons {
    // O ficheiro do dono pelo leitor de verdade; as formas deslizam, as setas seguem-nas.
    let imp = ph2d_board_excalidraw::read(OWNER_FILE).expect("o ficheiro do dono lê-se");
    b.doc = imp.doc;
    let owner: Vec<ElementId> = b
        .doc
        .live_in_z_order()
        .into_iter()
        .filter(|e| e.connector().is_some())
        .map(|e| e.id)
        .collect();
    let shapes: Vec<Element> = b
        .doc
        .live_in_z_order()
        .into_iter()
        .filter(|e| e.shape().is_some())
        .cloned()
        .collect();
    for mut el in shapes {
        el.translate(OWNER_SHIFT);
        put(b, el);
    }
    // As setas curtas, com o comprimento escrito por baixo.
    let mut st = Style::new(None, Some(INK), INK);
    st.stroke_width = SHORT_WIDTH;
    let short: Vec<ElementId> = SHORT
        .iter()
        .enumerate()
        .map(|(i, &l)| {
            let x = ROW[0] + i as f64 * STEP;
            let mut c = Connector::new(
                End::Free([x, ROW[1]]),
                End::Free([x + l, ROW[1]]),
                Route::Straight,
                st.clone(),
            );
            c.heads = [Head::None, Head::Arrow];
            let el = Element::new_connector(b.doc.mint_id(), b.doc.z_on_top(), c);
            let id = put(b, el);
            // O número centrado por baixo da seta (a 1.ª foto tinha-o à direita dela).
            label(
                b,
                &format!("{l}"),
                [x + l / 2.0 - 40.0, ROW[1] + 15.0, 80.0, 30.0],
            );
            id
        })
        .collect();
    // As duas caixas da lição de desenhar.
    let from = boxed(
        b,
        "board.smoke.fidelity.from",
        1,
        [800.0, 60.0, 180.0, 90.0],
    );
    let to = boxed(
        b,
        "board.smoke.fidelity.to",
        12,
        [1060.0, 300.0, 180.0, 110.0],
    );
    // A lição 5 é a própria caixa: o texto dela muda de letra com o botão.
    let mut st = Style::new(None, Some(INK), INK);
    st.sketch = true;
    st.look.letter = Letter::Sans;
    let shape = Shape {
        kind: ShapeType::Rectangle,
        style: st,
        text: RichText::plain(&tr("board.smoke.fidelity.l5")),
    };
    let el = Element::new_shape(
        b.doc.mint_id(),
        b.doc.z_on_top(),
        shape,
        [800.0, 440.0, 440.0, 150.0],
    );
    let letter = put(b, el);
    // As lições, por baixo, na ordem real.
    let y = ROW[1] + 100.0;
    let (w, h) = (STICKY_WIDE, STICKY_SIDE);
    let lessons: [ElementId; 4] = std::array::from_fn(|i| {
        let c = Rgba(STICKY_COLORS[1]);
        let shape = Shape {
            kind: ShapeType::StickyWide,
            style: Style::new(Some(c), None, c.readable_ink()),
            text: RichText::plain(&tr(&format!("board.smoke.fidelity.l{}", i + 1))),
        };
        let x = i as f64 * (w + GAP);
        let el = Element::new_shape(b.doc.mint_id(), b.doc.z_on_top(), shape, [x, y, w, h]);
        put(b, el)
    });
    let right = 4.0 * w + 3.0 * GAP;
    label(
        b,
        &tr("board.smoke.fidelity.title"),
        [0.0, -80.0, right, 40.0],
    );
    Lessons {
        owner,
        short,
        from,
        to,
        lessons,
        letter,
        view: [
            -MARGIN_X,
            -80.0 - MARGIN_Y,
            right + MARGIN_X,
            y + h + MARGIN_Y,
        ],
    }
}

/// A cena 11 no hero; devolve os ids dela (os testes conduzem-na pelo MESMO caminho do smoke).
pub(super) fn scene_fidelity(hero: &mut HeroScreen) -> Lessons {
    let mut set = BoardSet::default();
    let id = set.create(default_name(1));
    let l = build(set.get_mut(id).expect("acabou de nascer"));
    document_tabs::load(hero, set);
    hero.documents.activate(Some(id));
    board_nav::show_box(hero, false, false, l.view);
    l
}

#[cfg(test)]
#[path = "smoke_fidelity_tests.rs"]
mod tests;
