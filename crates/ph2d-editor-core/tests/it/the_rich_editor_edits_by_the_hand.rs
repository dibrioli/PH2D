//! O editor rico do verso do cartão (Kanban W3) conduzido pela mão: cliques nas caixas que o
//! MESMO layout pintou, teclas, atalhos markdown, a barra, o IME — e o documento do
//! `ph2d-richdoc` a responder.

use ph2d_editor_core::doc_surface::{Compose, Key, KeyInput};
use ph2d_editor_core::widget::{
    Marker, RICH_TOOLS, RichEditor, RichEditorEvent, RichEditorLabels, RichTool, paint_rich_editor,
};
use ph2d_editor_core::zones::Rect;
use ph2d_host::Modifiers;
use ph2d_richdoc::{Block, BlockKind, Caret, Inline, Mark, RichDoc, Selection};
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;

fn labels() -> RichEditorLabels {
    RichEditorLabels {
        placeholder: String::from("p"),
        tools: std::array::from_fn(|i| format!("t{i}")),
        styles: std::array::from_fn(|i| format!("s{i}")),
    }
}

fn rect() -> Rect {
    Rect::new(20.0, 20.0, 420.0, 600.0)
}

fn editor(doc: RichDoc) -> RichEditor {
    RichEditor::new(ph2d_a11y::NodeId(5), doc, labels())
}

fn none() -> Modifiers {
    Modifiers::default()
}

fn mods(ctrl: bool, shift: bool) -> Modifiers {
    Modifiers {
        ctrl,
        shift,
        ..Modifiers::default()
    }
}

fn press(ed: &mut RichEditor, ts: &mut TextSystem, k: Key, m: Modifiers, text: Option<&str>) {
    let input = KeyInput {
        key: k,
        mods: m,
        pressed: true,
        text,
        paste: None,
    };
    ed.key(ts, rect(), &input);
}

fn type_str(ed: &mut RichEditor, ts: &mut TextSystem, s: &str) {
    for ch in s.chars() {
        let t = ch.to_string();
        let k = if ch == ' ' { Key::Space } else { Key::Char(ch) };
        press(ed, ts, k, none(), Some(&t));
    }
}

/// O centro do caret `c`, tal como o editor o pinta.
fn at(ed: &mut RichEditor, ts: &mut TextSystem, c: Caret) -> (f32, f32) {
    let r = ed.layout(ts, rect()).caret_rect(c);
    (r.x + r.w / 2.0, r.y + r.h / 2.0)
}

#[test]
fn a_click_in_reading_enters_editing_and_markdown_shortcuts_make_blocks() {
    let mut ts = TextSystem::without_system_fonts();
    let mut ed = editor(RichDoc::new());
    let (x, y) = at(&mut ed, &mut ts, Caret::new(0, 0));
    assert_eq!(
        ed.pointer_down(&mut ts, rect(), x, y, none(), 1),
        Some(RichEditorEvent::EnterEdit)
    );
    assert!(ed.editing);
    type_str(&mut ed, &mut ts, "# Title");
    assert_eq!(ed.doc().blocks()[0].kind, BlockKind::Heading(1));
    assert_eq!(ed.doc().blocks()[0].inline.text(), "Title");
    press(&mut ed, &mut ts, Key::Enter, none(), None);
    type_str(&mut ed, &mut ts, "- item");
    assert_eq!(ed.doc().blocks()[1].kind, BlockKind::Bullet);
    press(&mut ed, &mut ts, Key::Enter, none(), None);
    type_str(&mut ed, &mut ts, "[] todo");
    assert_eq!(ed.doc().blocks()[2].kind, BlockKind::Check(false));
}

#[test]
fn ctrl_b_marks_what_is_typed_next_and_shift_arrows_select() {
    let mut ts = TextSystem::without_system_fonts();
    let mut ed = editor(RichDoc::new());
    ed.editing = true;
    type_str(&mut ed, &mut ts, "ab");
    press(&mut ed, &mut ts, Key::Char('b'), mods(true, false), Some("b"));
    type_str(&mut ed, &mut ts, "cd");
    let inline = &ed.doc().blocks()[0].inline;
    assert_eq!(inline.text(), "abcd");
    assert!(inline.covers(2..4, &Mark::Bold));
    assert!(!inline.covers(0..2, &Mark::Bold));
    for _ in 0..3 {
        press(&mut ed, &mut ts, Key::Left, mods(false, true), None);
    }
    assert_eq!(ed.selected_text(), "bcd");
    // Ctrl+Shift+X risca a selecção; Ctrl+C devolve o texto.
    press(&mut ed, &mut ts, Key::Char('X'), mods(true, true), Some("X"));
    assert!(ed.doc().blocks()[0].inline.covers(1..4, &Mark::Strike));
    let copy = ed.key(
        &mut ts,
        rect(),
        &KeyInput {
            key: Key::Char('c'),
            mods: mods(true, false),
            pressed: true,
            text: Some("c"),
            paste: None,
        },
    );
    assert_eq!(copy.copy.as_deref(), Some("bcd"));
}

fn sample() -> RichDoc {
    let mut link = Inline::plain("see ");
    link.push("docs", &[Mark::Link(String::from("https://x.test"))]);
    RichDoc::from_blocks(vec![
        Block::paragraph("hello brave world"),
        Block::new(BlockKind::Check(false), Inline::plain("task")),
        Block::new(BlockKind::Paragraph, link),
    ])
}

#[test]
fn double_click_takes_the_word_and_the_painted_box_ticks_the_task() {
    let mut ts = TextSystem::without_system_fonts();
    let mut ed = editor(sample());
    ed.editing = true;
    let (x, y) = at(&mut ed, &mut ts, Caret::new(0, 8));
    ed.pointer_down(&mut ts, rect(), x, y, none(), 2);
    assert_eq!(ed.selected_text(), "brave");
    let mr = ed.layout(&mut ts, rect()).blocks[1].marker_rect;
    assert_eq!(ed.layout(&mut ts, rect()).blocks[1].marker, Marker::Check(false));
    let ev = ed.pointer_down(&mut ts, rect(), mr.x + 2.0, mr.y + mr.h / 2.0, none(), 1);
    assert_eq!(ev, Some(RichEditorEvent::Changed));
    assert_eq!(ed.doc().blocks()[1].kind, BlockKind::Check(true));
}

#[test]
fn reading_mode_opens_a_link_instead_of_editing() {
    let mut ts = TextSystem::without_system_fonts();
    let mut ed = editor(sample());
    let (x, y) = at(&mut ed, &mut ts, Caret::new(2, 6));
    assert_eq!(
        ed.pointer_down(&mut ts, rect(), x, y, none(), 1),
        Some(RichEditorEvent::OpenLink(String::from("https://x.test")))
    );
    assert!(!ed.editing);
}

#[test]
fn the_toolbar_buttons_and_the_style_menu_change_the_document() {
    let mut ts = TextSystem::without_system_fonts();
    let mut ed = editor(sample());
    ed.editing = true;
    ed.set_selection(Selection::new(Caret::new(0, 0), Caret::new(0, 5)));
    let (bar, _) = ed.regions(rect());
    let bar = bar.expect("em edição há barra");
    let bold = RICH_TOOLS.iter().position(|t| *t == RichTool::Bold).unwrap();
    let b = ph2d_editor_core::widget::rich_editor_toolbar_button(bar, bold);
    let ev = ed.pointer_down(&mut ts, rect(), b.x + b.w / 2.0, b.y + b.h / 2.0, none(), 1);
    assert_eq!(ev, Some(RichEditorEvent::Changed));
    assert!(ed.doc().blocks()[0].inline.covers(0..5, &Mark::Bold));
    // «Aa» abre o menu; a 3.ª linha é o título 2.
    let aa = ph2d_editor_core::widget::rich_editor_toolbar_button(bar, 0);
    ed.pointer_down(&mut ts, rect(), aa.x + 1.0, aa.y + 1.0, none(), 1);
    assert!(ed.style_menu_open);
    let o = ph2d_editor_core::widget::rich_editor_style_option(bar, 2);
    ed.pointer_down(&mut ts, rect(), o.x + o.w / 2.0, o.y + o.h / 2.0, none(), 1);
    assert_eq!(ed.doc().blocks()[0].kind, BlockKind::Heading(2));
    // Ctrl+K pede o link ao chamador.
    let ev = ed.key(
        &mut ts,
        rect(),
        &KeyInput {
            key: Key::Char('k'),
            mods: mods(true, false),
            pressed: true,
            text: Some("k"),
            paste: None,
        },
    );
    assert_eq!(ev.event, Some(RichEditorEvent::LinkRequested));
    ed.apply_link("https://y.test");
    assert!(
        ed.doc().blocks()[0]
            .inline
            .covers(0..5, &Mark::Link(String::from("https://y.test")))
    );
}

#[test]
fn up_and_down_cross_blocks_and_a_long_line_wraps_softly() {
    let mut ts = TextSystem::without_system_fonts();
    let long = "word ".repeat(60);
    let mut ed = editor(RichDoc::from_blocks(vec![
        Block::paragraph("top"),
        Block::paragraph(long.trim_end()),
    ]));
    ed.editing = true;
    ed.set_selection(Selection::caret(Caret::new(0, 1)));
    press(&mut ed, &mut ts, Key::Down, none(), None);
    assert_eq!(ed.selection().head.block, 1);
    let lines = ed.layout(&mut ts, rect()).blocks[1].layout.len();
    assert!(lines > 1, "a linha longa quebra: {lines}");
    let end = ed.doc().end();
    let y_end = at(&mut ed, &mut ts, end).1;
    let y_start = at(&mut ed, &mut ts, Caret::new(1, 0)).1;
    assert!(y_end > y_start, "o fim está numa linha mais abaixo");
    press(&mut ed, &mut ts, Key::Up, none(), None);
    assert_eq!(ed.selection().head.block, 0);
    press(&mut ed, &mut ts, Key::Escape, none(), None);
    assert!(!ed.editing);
}

#[test]
fn the_ime_shows_the_preedit_and_commits_into_the_document() {
    let mut ts = TextSystem::without_system_fonts();
    let mut ed = editor(RichDoc::new());
    ed.editing = true;
    assert_eq!(ed.compose(Compose::Preedit("´", None)), None);
    assert_eq!(ed.preedit.as_deref(), Some("´"));
    assert_eq!(ed.compose(Compose::Commit('é')), Some(RichEditorEvent::Changed));
    assert_eq!(ed.preedit, None);
    assert_eq!(ed.doc().plain_text(), "é");
}

#[test]
fn every_block_and_mark_paints_in_every_theme_and_closes_its_clip() {
    let mut ts = TextSystem::without_system_fonts();
    let doc = ph2d_editor_core::widget::showcase::card_back::demo_doc();
    for theme in [Theme::Forge, Theme::Sunstone, Theme::Blueprint] {
        for editing in [false, true] {
            let mut ed = editor(doc.clone());
            ed.editing = editing;
            ed.style_menu_open = editing;
            ed.set_selection(Selection::new(Caret::new(1, 2), Caret::new(4, 3)));
            let h = ed.preferred_height(&mut ts, rect().w);
            assert!(h > 0.0);
            let mut scene = VectorScene::new();
            paint_rich_editor(&mut ed, rect(), &mut scene, &mut ts, theme);
            let enc = scene.inner().encoding();
            assert_eq!(enc.n_open_clips, 0, "{theme:?}");
            assert!(enc.n_paths > 0);
        }
    }
}
