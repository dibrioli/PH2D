//! As NOTAS (W3) pelo caminho de quem as usa — no idioma do Miro (fontes em `notes.rs`).

use super::*;
use ph2d_board_model::{Mark, STICKY_COLORS, STICKY_DEFAULT_COLOR, STICKY_SIDE, STICKY_WIDE};

impl World {
    /// Uma nota com um clique da ferramenta, em `c`.
    fn note(&mut self, c: [f64; 2]) -> ElementId {
        self.ed.tool = Tool::Shape(ShapeType::Sticky);
        self.click(c, NONE);
        *self
            .ed
            .selection()
            .iter()
            .next()
            .expect("nasceu seleccionada")
    }

    fn text(&self, id: ElementId) -> String {
        self.el(id).shape().expect("forma").text.as_str().to_owned()
    }

    fn write(&mut self, s: &str) {
        self.ed.text_input(&mut self.doc, &mut self.ts, s);
    }

    fn key(&mut self, k: TextKey) {
        self.ed
            .text_key(&mut self.doc, &mut self.h, &mut self.ts, k);
    }
}

#[test]
fn a_click_with_the_note_tool_makes_the_miro_default_note() {
    let mut w = world();
    let id = w.note([0.0, 0.0]);
    let el = w.el(id);
    assert_eq!(
        [el.x, el.y, el.w, el.h],
        [-99.5, -99.5, STICKY_SIDE, STICKY_SIDE]
    );
    let s = el.shape().unwrap();
    assert_eq!(s.kind, ShapeType::Sticky);
    assert_eq!(
        s.style.fill,
        Some(Rgba(STICKY_COLORS[STICKY_DEFAULT_COLOR]))
    );
    assert_eq!(s.style.stroke, None, "a nota do Miro não tem contorno");
    assert_eq!(s.style.font_size, ph2d_board_model::DEFAULT_FONT_SIZE);
    assert_eq!(w.ed.tool, Tool::Select);
    assert!(w.cmd(Command::Undo));
    assert_eq!(w.doc.live_len(), 0);
}

#[test]
fn the_last_colour_and_size_chosen_are_the_next_notes() {
    let mut w = world();
    let red = Rgba(STICKY_COLORS[11]);
    let a = w.note([0.0, 0.0]);
    w.ed.set_note_color(&mut w.doc, &mut w.h, red);
    w.ed.set_note_size(&mut w.doc, &mut w.h, 2);
    let el = w.el(a);
    assert_eq!(el.style().fill, Some(red));
    assert!(close(el.w, STICKY_SIDE * NOTE_SCALES[2]), "{}", el.w);
    assert!(close(el.style().font_size, 20.0 * NOTE_SCALES[2]));
    let b = w.note([1000.0, 0.0]);
    assert_eq!(
        w.el(b).style().fill,
        Some(red),
        "a próxima nasce da última cor"
    );
    assert!(close(w.el(b).w, STICKY_SIDE * NOTE_SCALES[2]));
}

#[test]
fn typing_on_a_selected_note_writes_at_the_end_even_a_shortcut_letter() {
    let mut w = world();
    let id = w.note([0.0, 0.0]);
    assert!(w.ed.type_into_note(&mut w.doc, &mut w.ts, "n"));
    assert!(w.ed.is_editing_text());
    w.write("ota");
    w.key(TextKey::Commit);
    assert_eq!(w.text(id), "nota");
    // Seleccionada outra vez: o texto que lá está não se perde, escreve-se a seguir.
    w.ed.select(&w.doc, [id]);
    assert!(w.ed.type_into_note(&mut w.doc, &mut w.ts, "!"));
    w.key(TextKey::Commit);
    assert_eq!(w.text(id), "nota!");
    // Numa forma, uma letra é atalho (recusa da W1): não começa a escrever.
    let r = w.rect([500.0, 0.0, 100.0, 50.0]);
    w.ed.select(&w.doc, [r]);
    assert!(!w.ed.type_into_note(&mut w.doc, &mut w.ts, "r"));
}

#[test]
fn a_note_grows_down_with_its_text_and_shrinks_back_to_its_birth_height() {
    let mut w = world();
    let id = w.note([0.0, 0.0]);
    let top = w.el(id).y;
    w.ed.type_into_note(&mut w.doc, &mut w.ts, "uma ideia");
    for _ in 0..30 {
        w.write(" muito comprida");
    }
    let grown = w.el(id).h;
    assert!(grown > STICKY_SIDE * 1.5, "não cresceu: {grown}");
    assert_eq!(
        w.el(id).w,
        STICKY_SIDE,
        "cresce na VERTICAL (decisão 2 do dono)"
    );
    assert_eq!(w.el(id).y, top, "o topo fica");
    w.key(TextKey::SelectAll);
    w.key(TextKey::Backspace { word: false });
    assert_eq!(w.el(id).h, STICKY_SIDE, "volta à altura de nascença");
    // E numa nota que JÁ cresceu numa edição anterior: a de nascença, não a de quando se abriu.
    w.write(&"comprida ".repeat(40));
    w.key(TextKey::Commit);
    assert!(w.el(id).h > STICKY_SIDE * 1.5);
    w.ed.begin_text(&mut w.doc, &mut w.ts, id, None);
    w.key(TextKey::Backspace { word: false });
    assert_eq!(w.el(id).h, STICKY_SIDE, "ficou presa à altura de quando se abriu");
}

#[test]
fn tab_while_writing_makes_the_next_note_to_the_right_already_writing() {
    let mut w = world();
    let red = Rgba(STICKY_COLORS[11]);
    w.ed.notes.color = red;
    let a = w.note([0.0, 0.0]);
    w.ed.type_into_note(&mut w.doc, &mut w.ts, "um");
    assert!(w.ed.next_note(&mut w.doc, &mut w.h));
    let b = w.ed.editing_id().expect("já a escrever na seguinte");
    assert_ne!(a, b);
    let (ea, eb) = (w.el(a).clone(), w.el(b).clone());
    assert_eq!(w.text(a), "um");
    assert!(w.text(b).is_empty());
    assert_eq!(eb.style().fill, Some(red));
    assert_eq!([eb.w, eb.h, eb.y], [ea.w, ea.h, ea.y]);
    let gap = STICKY_SIDE * NOTE_GAP_K;
    assert!(
        close(eb.x, ea.x + ea.w + gap),
        "{} vs {}",
        eb.x,
        ea.x + ea.w + gap
    );
    w.write("dois");
    w.key(TextKey::Commit);
    // Três passos: o texto da 1.ª, a 2.ª nascer, o texto da 2.ª.
    assert!(w.cmd(Command::Undo));
    assert!(w.text(b).is_empty());
    assert!(w.cmd(Command::Undo));
    assert!(w.doc.get(b).is_none());
    assert!(w.cmd(Command::Undo));
    assert!(w.text(a).is_empty());
}

#[test]
fn tab_skips_a_place_already_taken() {
    let mut w = world();
    let a = w.note([0.0, 0.0]);
    let x_next = w.el(a).x + STICKY_SIDE * (1.0 + NOTE_GAP_K);
    w.rect([x_next + 10.0, 0.0, 50.0, 50.0]);
    w.ed.select(&w.doc, [a]);
    w.ed.type_into_note(&mut w.doc, &mut w.ts, "x");
    w.ed.next_note(&mut w.doc, &mut w.h);
    let b = w.ed.editing_id().unwrap();
    assert!(
        w.el(b).x > x_next + 50.0,
        "caiu em cima da caixa: {}",
        w.el(b).x
    );
}

#[test]
fn bulk_mode_turns_each_line_into_a_note_in_a_row_as_one_step() {
    let mut w = world();
    assert!(w.ed.begin_bulk(&mut w.doc, &mut w.h, [0.0, 0.0]));
    assert_eq!(w.doc.live_len(), 1, "o rascunho");
    w.write("primeira\nsegunda\n\n  terceira  ");
    w.key(TextKey::Commit);
    let notes = w.doc.live_in_z_order();
    assert_eq!(
        notes.len(),
        3,
        "o rascunho saiu e cada linha com texto é uma nota"
    );
    let texts: Vec<String> = notes
        .iter()
        .map(|e| e.shape().unwrap().text.as_str().to_owned())
        .collect();
    assert_eq!(texts, ["primeira", "segunda", "terceira"]);
    assert!(
        notes
            .iter()
            .all(|e| e.shape().unwrap().kind == ShapeType::Sticky)
    );
    assert!(
        notes[0].y == notes[1].y && notes[1].y == notes[2].y,
        "em fila"
    );
    assert!(notes[0].x < notes[1].x && notes[1].x < notes[2].x);
    // A linha em branco não deixa um buraco na fila (o vão entre as três é o mesmo).
    let step = notes[1].x - notes[0].x;
    assert!(close(notes[2].x - notes[1].x, step), "buraco na fila");
    assert_eq!(w.ed.selection().len(), 3);
    assert!(w.cmd(Command::Undo));
    assert_eq!(w.doc.live_len(), 0, "UM passo, e o rascunho não volta");
    // Um rascunho vazio não deixa nada nem passo.
    w.ed.begin_bulk(&mut w.doc, &mut w.h, [0.0, 0.0]);
    w.key(TextKey::Commit);
    assert_eq!(w.doc.live_len(), 0);
    assert!(!w.cmd(Command::Undo));
}

#[test]
fn a_long_bulk_line_fits_its_note_on_the_next_frame() {
    let mut w = world();
    w.ed.begin_bulk(&mut w.doc, &mut w.h, [0.0, 0.0]);
    w.write(&"palavra ".repeat(60));
    w.key(TextKey::Commit);
    let id = *w.ed.selection().iter().next().unwrap();
    let _ = w.ed.overlay(&mut w.doc, &mut w.ts);
    assert!(w.el(id).h > STICKY_SIDE, "a nota não cresceu para o texto");
}

#[test]
fn the_spreadsheet_parser_reads_tabs_lines_and_quoted_cells() {
    let t = "a\tb\tc\r\n1\t\t\"três\nlinhas \"\"x\"\"\"\n";
    let rows = parse_cells(t);
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0], ["a", "b", "c"]);
    assert_eq!(rows[1], ["1", "", "três\nlinhas \"x\""]);
    assert!(parse_cells("").is_empty());
    assert_eq!(parse_cells("só uma"), [["só uma"]]);
}

#[test]
fn pasting_a_spreadsheet_makes_one_note_per_cell_on_the_cells_grid() {
    let mut w = world();
    assert!(w.ed.paste_cells(&mut w.doc, &mut w.h, &mut w.ts, "a\tb\nc\t\n", [0.0, 0.0]));
    let mut notes = w.doc.live_in_z_order();
    assert_eq!(notes.len(), 3, "a célula vazia não é nota");
    notes.sort_by(|a, b| a.y.total_cmp(&b.y).then(a.x.total_cmp(&b.x)));
    let t = |e: &Element| e.shape().unwrap().text.as_str().to_owned();
    assert_eq!([t(notes[0]), t(notes[1]), t(notes[2])], ["a", "b", "c"]);
    assert_eq!(notes[0].y, notes[1].y);
    assert_eq!(notes[0].x, notes[2].x, "a coluna da célula");
    assert!(notes[2].y > notes[0].y + notes[0].h);
    assert_eq!(w.ed.selection().len(), 3);
    assert!(w.cmd(Command::Undo));
    assert_eq!(w.doc.live_len(), 0);
    assert!(
        !w.ed
            .paste_cells(&mut w.doc, &mut w.h, &mut w.ts, "\t\n\t", [0.0, 0.0])
    );
}

#[test]
fn a_foreign_clipboard_is_told_apart_from_the_boards_own_copy() {
    let mut w = world();
    assert!(w.ed.is_foreign_paste("de fora"), "nada copiado no quadro");
    assert!(!w.ed.is_foreign_paste("   "));
    let id = w.note([0.0, 0.0]);
    w.ed.type_into_note(&mut w.doc, &mut w.ts, "ideia");
    w.key(TextKey::Commit);
    w.ed.select(&w.doc, [id]);
    w.cmd(Command::Copy);
    assert_eq!(w.ed.copied_text(), Some("ideia"));
    assert!(!w.ed.is_foreign_paste("ideia"), "é o que a cópia lá pôs");
    assert!(w.ed.is_foreign_paste("a\tb"), "outro texto: veio de fora");
}

#[test]
fn the_four_dot_handle_arranges_the_selected_notes_in_columns() {
    let mut w = world();
    let ids: Vec<ElementId> = (0..4)
        .map(|i| w.note([f64::from(i) * 300.0, 0.0]))
        .collect();
    w.ed.select(&w.doc, ids.iter().copied());
    let g = w.ed.grid_handle(&w.doc, PX).expect("quatro notas: há pega");
    let x0 = w.el(ids[0]).x;
    let gap = STICKY_SIDE * NOTE_GAP_K;
    // Até a meio da 2.ª coluna: duas colunas.
    let to = [x0 + 1.5 * STICKY_SIDE + gap, g[1] + 200.0];
    w.drag(g, to, NONE);
    let e: Vec<Element> = ids.iter().map(|i| w.el(*i).clone()).collect();
    assert!(close(e[0].x, x0) && close(e[2].x, x0), "1.ª coluna");
    assert!(close(e[1].x, x0 + STICKY_SIDE + gap) && close(e[3].x, e[1].x));
    assert!(close(e[0].y, e[1].y) && close(e[2].y, e[0].y + STICKY_SIDE + gap));
    assert!(w.cmd(Command::Undo));
    assert!(close(w.el(ids[3]).x, x0 + 900.0), "desfazer devolve a fila");
    // Uma forma na selecção: sem pega.
    let r = w.rect([0.0, 600.0, 50.0, 50.0]);
    w.ed.select(&w.doc, ids.iter().copied().chain([r]));
    assert!(w.ed.grid_handle(&w.doc, PX).is_none());
}

#[test]
fn dragging_from_a_stack_takes_a_new_note_and_a_click_selects_the_stack() {
    let mut w = world();
    w.ed.notes.color = Rgba(STICKY_COLORS[7]);
    w.ed.tool = Tool::Shape(ShapeType::StickyStack);
    w.click([0.0, 0.0], NONE);
    let stack = *w.ed.selection().iter().next().unwrap();
    w.ed.select(&w.doc, []);
    w.drag([0.0, 0.0], [400.0, 50.0], NONE);
    assert_eq!(w.doc.live_len(), 2, "a pilha ficou e saiu uma nota");
    let note = *w.ed.selection().iter().next().unwrap();
    assert_ne!(note, stack);
    let n = w.el(note);
    assert_eq!(n.shape().unwrap().kind, ShapeType::Sticky);
    assert_eq!(n.style().fill, Some(Rgba(STICKY_COLORS[7])));
    assert!(close(n.center()[0], 400.0) && close(n.center()[1], 50.0));
    assert!(close(w.el(stack).center()[0], 0.0), "a pilha não andou");
    assert!(w.cmd(Command::Undo));
    assert_eq!(w.doc.live_len(), 1);
    // Um clique sem arrastar só selecciona a pilha; a seguir arrastá-la move-a.
    w.ed.select(&w.doc, []);
    w.click([0.0, 0.0], NONE);
    assert_eq!(
        w.ed.selection().iter().copied().collect::<Vec<_>>(),
        [stack]
    );
    w.drag([0.0, 0.0], [100.0, 0.0], NONE);
    assert!(close(w.el(stack).center()[0], 100.0));
    // E a pilha não se escreve.
    assert!(!w.ed.begin_text(&mut w.doc, &mut w.ts, stack, None));
}

#[test]
fn resizing_a_note_keeps_its_shape_and_scales_the_letter() {
    let mut w = world();
    let id = w.note([0.0, 0.0]);
    let e = w.el(id).clone();
    // A pega do canto inferior direito, puxada só na horizontal.
    w.drag([e.x + e.w, e.y + e.h], [e.x + 2.0 * e.w, e.y + e.h], NONE);
    let r = w.el(id);
    assert!(
        close(r.w, 2.0 * STICKY_SIDE) && close(r.h, 2.0 * STICKY_SIDE),
        "{} × {}",
        r.w,
        r.h
    );
    assert!(close(r.style().font_size, 40.0));
}

#[test]
fn square_and_wide_switch_the_width_and_keep_the_scale() {
    let mut w = world();
    let id = w.note([0.0, 0.0]);
    w.ed.set_note_wide(&mut w.doc, &mut w.h, true);
    let _ = w.ed.overlay(&mut w.doc, &mut w.ts);
    let e = w.el(id);
    assert_eq!(e.shape().unwrap().kind, ShapeType::StickyWide);
    assert!(close(e.w, STICKY_WIDE));
    assert!(
        close(e.h, STICKY_SIDE),
        "a larga do Miro tem a altura da quadrada"
    );
    assert!(close(e.center()[0], 0.0), "troca à volta do centro");
    assert_eq!(note_size(e), Some(NOTE_DEFAULT_SIZE));
}

#[test]
fn ctrl_b_bolds_the_whole_text_of_the_selection_or_the_selected_words_while_writing() {
    let mut w = world();
    let id = w.note([0.0, 0.0]);
    w.ed.type_into_note(&mut w.doc, &mut w.ts, "uma ideia");
    w.key(TextKey::Commit);
    w.ed.select(&w.doc, [id]);
    assert!(w.cmd(Command::Mark(Mark::Bold)));
    assert!(w.ed.has_mark(&w.doc, Mark::Bold));
    let t = w.el(id).shape().unwrap().text.clone();
    assert!(t.all(0..t.len(), Mark::Bold));
    assert!(w.cmd(Command::Undo), "um passo");
    assert!(!w.ed.has_mark(&w.doc, Mark::Bold));
    // A escrever: só a palavra seleccionada.
    w.ed.begin_text(&mut w.doc, &mut w.ts, id, None);
    w.key(TextKey::Move {
        to: Move::TextEnd,
        extend: false,
    });
    w.key(TextKey::Move {
        to: Move::WordLeft,
        extend: true,
    });
    w.cmd(Command::Mark(Mark::Italic));
    w.key(TextKey::Commit);
    let t = w.el(id).shape().unwrap().text.clone();
    assert_eq!(t.spans().len(), 1);
    assert_eq!(&t.as_str()[t.spans()[0].range()], "ideia");
    assert!(t.spans()[0].marks.italic);
}

#[test]
fn reading_order_goes_by_rows_then_left_to_right() {
    let mut w = world();
    let a = w.note([500.0, 0.0]);
    let b = w.note([0.0, 30.0]);
    let c = w.note([0.0, 400.0]);
    let els: Vec<Element> = [a, b, c].iter().map(|i| w.el(*i).clone()).collect();
    let order: Vec<ElementId> = reading_order(&els).iter().map(|e| e.id).collect();
    assert_eq!(order, [b, a, c]);
}
