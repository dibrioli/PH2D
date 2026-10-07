//! A CANETA e o RASCUNHO (W4) pelo caminho real do ecrã: o botão e o painel pelo despacho, o rato
//! pela porta da shell, as teclas pela do teclado.

use super::*;
use ph2d_board_model::Pen;

const NONE: Modifiers = Modifiers {
    shift: false,
    ctrl: false,
    alt: false,
    meta: false,
};

impl T {
    fn inks(&self) -> Vec<Element> {
        self.elements()
            .into_iter()
            .filter(|e| e.ink().is_some())
            .collect()
    }

    fn tool(&self) -> Tool {
        self.hero.documents.live.editor.as_ref().unwrap().tool
    }

    fn sketch(&self) -> bool {
        self.hero.documents.active_board().unwrap().sketch
    }

    fn press(&mut self, c: char) {
        let s = c.to_string();
        assert!(
            self.key(BoardKey::Char(c), NONE, Some(&s)),
            "a tecla {c} não foi do quadro"
        );
    }
}

/// ⭐ O botão da caneta abre o painel; uma predefinição escolhe-se; arrastar no quadro desenha um
/// traço com ela — e a caneta fica na mão (o idioma do Miro).
#[test]
fn the_pen_button_opens_its_panel_and_a_drag_draws_with_the_chosen_preset() {
    let mut t = T::new();
    assert!(
        t.find(node(Item::Pen(PenItem::Preset(1)))).is_none(),
        "o painel nasce fechado"
    );
    t.click_bar(Item::Tool(Tool::Pen(Pen::Pen)));
    assert_eq!(t.tool(), Tool::Pen(Pen::Pen));
    t.click_bar(Item::Pen(PenItem::Preset(1)));
    let want = t
        .hero
        .documents
        .live
        .editor
        .as_ref()
        .unwrap()
        .pen
        .current(Pen::Pen);
    t.drag(t.at(0.3, 0.5), t.at(0.6, 0.55));
    let inks = t.inks();
    assert_eq!(inks.len(), 1, "o arrasto não desenhou");
    assert_eq!(inks[0].style().stroke, Some(want.color));
    assert_eq!(inks[0].style().stroke_width, want.width);
    assert_eq!(t.tool(), Tool::Pen(Pen::Pen), "a caneta fica na mão");
    assert!(
        t.find(node(Item::Pen(PenItem::Laser))).is_some(),
        "e o painel aberto"
    );
}

/// O marcador, e a cor e a espessura do painel mudam a predefinição ACTIVA dele.
#[test]
fn the_colour_and_width_of_the_panel_edit_the_active_preset() {
    let mut t = T::new();
    t.click_bar(Item::Tool(Tool::Pen(Pen::Pen)));
    t.click_bar(Item::Pen(PenItem::Kind(Pen::Highlighter)));
    t.click_bar(Item::Pen(PenItem::Color(Some(6))));
    t.click_bar(Item::Pen(PenItem::Width(4)));
    t.drag(t.at(0.3, 0.4), t.at(0.5, 0.4));
    let ink = t.inks().pop().expect("um traço");
    assert_eq!(ink.ink().unwrap().pen, Pen::Highlighter);
    assert_eq!(ink.style().stroke, Some(Rgba(HIGHLIGHTER_RGBA[6])));
    assert_eq!(ink.style().stroke_width, ph2d_board_edit::PEN_WIDTHS[4]);
}

/// `P` caneta (a última usada), `E` borracha, `K` laser.
#[test]
fn p_e_and_k_pick_the_pen_the_eraser_and_the_laser() {
    let mut t = T::new();
    t.press('e');
    assert_eq!(t.tool(), Tool::Eraser { precise: false });
    t.press('k');
    assert_eq!(t.tool(), Tool::Laser);
    t.press('p');
    assert_eq!(t.tool(), Tool::Pen(Pen::Pen));
    t.click_bar(Item::Pen(PenItem::Kind(Pen::Highlighter)));
    t.press('v');
    t.press('p');
    assert_eq!(
        t.tool(),
        Tool::Pen(Pen::Highlighter),
        "o P volta a pegar na última caneta"
    );
}

/// ⭐ O botão «Rascunho ↔ Final» sem selecção troca o quadro INTEIRO, e o que nasce depois nasce
/// à mão; outra vez, tudo volta a final.
#[test]
fn the_sketch_button_flips_the_whole_board_and_what_is_born_after_it() {
    let mut t = T::new();
    t.press('r');
    t.drag(t.at(0.2, 0.2), t.at(0.35, 0.35));
    t.press('v');
    t.key(BoardKey::Escape, NONE, None);
    assert!(!t.sketch());
    t.click_bar(Item::Sketch);
    assert!(t.sketch(), "o quadro passou a rascunho");
    assert!(t.elements().iter().all(|e| e.style().sketch));
    t.press('r');
    t.drag(t.at(0.5, 0.5), t.at(0.65, 0.65));
    assert!(
        t.elements().iter().all(|e| e.style().sketch),
        "a forma nova nasceu à mão"
    );
    t.press('v');
    t.key(BoardKey::Escape, NONE, None);
    // Outro botão antes: dois cliques seguidos no mesmo são um duplo-clique no relógio do teste.
    t.click_bar(Item::Tool(Tool::Select));
    t.click_bar(Item::Sketch);
    assert!(!t.sketch());
    assert!(t.elements().iter().all(|e| !e.style().sketch));
}

/// Um traço seleccionado mostra a barra de estilo DOS TRAÇOS; a espessura dela muda o traço e não o
/// estilo das próximas formas.
#[test]
fn a_selected_stroke_gets_the_ink_style_bar_and_its_width() {
    let mut t = T::new();
    t.press('p');
    let (a, b) = (t.at(0.3, 0.5), t.at(0.6, 0.5));
    t.drag(a, b);
    t.press('v');
    let mid = ((a.0 + b.0) / 2.0, a.1);
    t.drag(mid, mid);
    assert!(
        t.find(node(Item::Pen(PenItem::InkWidth(3)))).is_some(),
        "a barra dos traços não abriu"
    );
    let before = t
        .hero
        .documents
        .live
        .editor
        .as_ref()
        .unwrap()
        .style
        .stroke_width;
    t.click_bar(Item::Pen(PenItem::InkWidth(3)));
    assert_eq!(
        t.inks()[0].style().stroke_width,
        ph2d_board_edit::PEN_WIDTHS[3]
    );
    let after = t
        .hero
        .documents
        .live
        .editor
        .as_ref()
        .unwrap()
        .style
        .stroke_width;
    assert_eq!(
        after, before,
        "as próximas formas não herdam a espessura da caneta"
    );
}
