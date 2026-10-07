//! As NOTAS (W3) pelo caminho real do ecrã: o botão e o painel da barra curta pelo despacho, o rato
//! pela porta da shell, as teclas pela do teclado.

use super::*;
use ph2d_board_model::{Mark, STICKY_COLORS, STICKY_SIDE};

const NONE: Modifiers = Modifiers {
    shift: false,
    ctrl: false,
    alt: false,
    meta: false,
};

impl T {
    fn typed(&mut self, s: &str) {
        for ch in s.chars() {
            let k = if ch == '\n' {
                BoardKey::Enter
            } else {
                BoardKey::Char(ch.to_ascii_lowercase())
            };
            let text = ch.to_string();
            self.key(k, NONE, (ch != '\n').then_some(text.as_str()));
        }
    }

    fn notes(&self) -> Vec<Element> {
        self.elements()
            .into_iter()
            .filter(|e| e.shape().is_some_and(|s| s.kind.is_note()))
            .collect()
    }

    fn editing(&self) -> Option<ph2d_board_model::ElementId> {
        self.hero.documents.live.editor.as_ref()?.editing_id()
    }

    fn paste(&mut self, text: &str) {
        board_keys::key(
            &mut self.hero,
            &mut self.ts,
            BoardKey::Char('v'),
            CTRL,
            true,
            None,
            Some(text.to_owned()),
        );
        self.paint();
    }
}

/// ⭐ O fluxo do brainstorm: o botão Nota abre o painel, uma cor escolhe-se, um clique põe a nota,
/// escrever escreve nela (mesmo o «n», que é atalho), `Tab` faz a seguinte à direita já a escrever.
#[test]
fn the_note_button_a_colour_a_click_typing_and_tab_make_a_row_of_ideas() {
    let mut t = T::new();
    assert!(
        t.find(node(Item::NoteColor(3))).is_none(),
        "o painel nasce aberto"
    );
    t.click_bar(Item::Tool(Tool::Shape(ShapeType::Sticky)));
    t.click_bar(Item::NoteColor(3));
    let p = t.at(0.4, 0.5);
    t.drag(p, p);
    let notes = t.notes();
    assert_eq!(notes.len(), 1, "o clique não pôs a nota");
    assert_eq!(notes[0].style().fill, Some(Rgba(STICKY_COLORS[3])));
    t.typed("nota");
    assert_eq!(t.notes()[0].shape().unwrap().text.as_str(), "nota");
    assert!(t.key(BoardKey::Tab, NONE, None), "o Tab não foi do quadro");
    t.typed("dois");
    t.key(BoardKey::Escape, NONE, None);
    let mut notes = t.notes();
    notes.sort_by(|a, b| a.x.total_cmp(&b.x));
    assert_eq!(notes.len(), 2);
    assert_eq!(notes[1].shape().unwrap().text.as_str(), "dois");
    assert_eq!(notes[1].style().fill, notes[0].style().fill, "a mesma cor");
    assert!(notes[1].x > notes[0].x + notes[0].w, "à direita");
    assert_eq!(notes[1].y, notes[0].y);
}

#[test]
fn the_n_key_picks_the_note_and_opens_its_panel() {
    let mut t = T::new();
    assert!(t.key(BoardKey::Char('n'), NONE, Some("n")));
    let ed = t.hero.documents.live.editor.as_ref().unwrap();
    assert_eq!(ed.tool, Tool::Shape(ShapeType::Sticky));
    assert!(t.find(node(Item::Bulk)).is_some(), "o painel não abriu");
}

/// A barra de estilo de uma nota seleccionada: tamanho, forma larga e negrito.
#[test]
fn the_style_bar_of_a_note_resizes_widens_and_bolds_it() {
    let mut t = T::new();
    t.key(BoardKey::Char('n'), NONE, Some("n"));
    let p = t.at(0.5, 0.6);
    t.drag(p, p);
    t.typed("ideia");
    t.key(BoardKey::Escape, NONE, None);
    t.click_bar(Item::NoteSize(2));
    let n = &t.notes()[0];
    assert!((n.w - STICKY_SIDE * ph2d_board_edit::NOTE_SCALES[2]).abs() < 1e-6);
    t.click_bar(Item::NoteWide(true));
    assert_eq!(t.notes()[0].shape().unwrap().kind, ShapeType::StickyWide);
    t.click_bar(Item::Mark(Mark::Bold));
    let txt = t.notes()[0].shape().unwrap().text.clone();
    assert!(
        txt.all(0..txt.len(), Mark::Bold),
        "o negrito da barra não pegou"
    );
    // Três passos de desfazer, um por clique.
    for _ in 0..3 {
        assert!(t.key(BoardKey::Char('z'), CTRL, None));
    }
    assert_eq!(t.notes()[0].shape().unwrap().kind, ShapeType::Sticky);
}

/// O modo em massa: uma ideia por linha; `Esc` faz as notas.
#[test]
fn bulk_mode_from_the_panel_makes_one_note_per_line() {
    let mut t = T::new();
    t.click_bar(Item::Tool(Tool::Shape(ShapeType::Sticky)));
    t.click_bar(Item::Bulk);
    assert!(t.editing().is_some(), "o rascunho não abriu a escrever");
    t.typed("um\ndois\ntres");
    t.key(BoardKey::Escape, NONE, None);
    let notes = t.notes();
    assert_eq!(notes.len(), 3, "{:?}", t.elements().len());
    assert!(
        t.elements().iter().all(|e| e.style().dash == Dash::Solid),
        "o rascunho ficou"
    );
}

/// `Ctrl+V` de uma planilha (texto de FORA) = uma nota por célula.
#[test]
fn ctrl_v_of_a_spreadsheet_makes_one_note_per_cell() {
    let mut t = T::new();
    t.paste("a\tb\tc\nd\te\tf\n");
    assert_eq!(t.notes().len(), 6);
    // E o Ctrl+V do que o quadro copiou continua a colar os elementos.
    t.key(BoardKey::Char('c'), CTRL, None);
    t.paste("a\nb\nc\nd\ne\nf");
    assert_eq!(
        t.notes().len(),
        12,
        "a cópia do quadro não colou os elementos"
    );
}

/// A escrever numa forma, a barra mostra o texto: o negrito apanha só o seleccionado, e a cor da
/// letra existe (numa nota não — as notas do Miro não mudam a cor da letra).
#[test]
fn writing_in_a_shape_shows_the_text_bar_and_bold_takes_only_the_selection() {
    let mut t = T::new();
    t.key(BoardKey::Char('r'), NONE, Some("r"));
    let (a, b) = (t.at(0.3, 0.4), t.at(0.6, 0.6));
    t.drag(a, b);
    t.key(BoardKey::Enter, NONE, None);
    t.typed("um dois");
    let shift = Modifiers {
        shift: true,
        ctrl: true,
        ..NONE
    };
    t.key(BoardKey::Left, shift, None);
    assert!(
        t.find(node(Item::TextColor(None))).is_some(),
        "sem a cor da letra"
    );
    t.click_bar(Item::Mark(Mark::Bold));
    t.key(BoardKey::Escape, NONE, None);
    let txt = t.elements()[0].shape().unwrap().text.clone();
    assert_eq!(txt.spans().len(), 1);
    assert_eq!(&txt.as_str()[txt.spans()[0].range()], "dois");
    // Numa nota a escrever: as marcas, sem a cor.
    t.key(BoardKey::Char('n'), NONE, Some("n"));
    let p = t.at(0.2, 0.2);
    t.drag(p, p);
    t.typed("x");
    assert!(t.find(node(Item::Mark(Mark::Italic))).is_some());
    assert!(t.find(node(Item::TextColor(None))).is_none());
}

/// A pega de quatro pontos de uma selecção de notas arruma-as em grelha.
#[test]
fn the_four_dot_handle_arranges_the_notes() {
    let mut t = T::new();
    t.paste("1\t2\t3\t4");
    let before: Vec<f64> = t.notes().iter().map(|e| e.y).collect();
    assert!(before.windows(2).all(|w| w[0] == w[1]), "em fila");
    let (board, live) = t.hero.documents.active_parts().unwrap();
    let ed = live.editor.as_mut().unwrap();
    let corner = ed.overlay(&mut board.doc, &mut t.ts).grid.expect("há pega");
    let off = ph2d_board_edit::grid_handle_offset(ed.metrics()) as f32;
    let cam = board.camera;
    let area = board_view::area_of(t.hero.last_canvas);
    let s = cam.to_screen(area, corner);
    let handle = (s[0] as f32 + off, s[1] as f32 - off);
    assert!(
        t.hero
            .hit_index
            .hit(handle.0, handle.1)
            .and_then(item_of)
            .is_none(),
        "a barra de estilo tapa a pega da grelha"
    );
    let left = t.notes().iter().map(|e| e.x).fold(f64::INFINITY, f64::min);
    let w = t.notes()[0].w;
    // Puxada até à borda direita da 2.ª coluna: duas colunas, duas linhas.
    let gap = STICKY_SIDE * ph2d_board_edit::NOTE_GAP_K;
    let to = cam.to_screen(area, [left + 2.0 * w + gap, corner[1]]);
    t.drag(handle, (to[0] as f32, handle.1));
    let ys: std::collections::BTreeSet<i64> =
        t.notes().iter().map(|e| e.y.round() as i64).collect();
    assert_eq!(ys.len(), 2, "não ficou em duas linhas");
}
