use super::*;

#[test]
fn text_wraps_to_the_width_and_grows_in_height() {
    let mut ts = TextSystem::without_system_fonts();
    let one = shape(&mut ts, "uma ideia", 20.0, 1000.0);
    let narrow = shape(&mut ts, "uma ideia muito comprida que não cabe", 20.0, 80.0);
    assert_eq!(one.lines().count(), 1);
    assert!(narrow.lines().count() > 2, "não quebrou");
    assert!(narrow.height() > one.height() * 2.0);
    assert!(
        narrow.width() <= 80.0 + 0.5,
        "passou da largura: {}",
        narrow.width()
    );
}

#[test]
fn the_cache_reshapes_only_when_something_changed() {
    let mut ts = TextSystem::without_system_fonts();
    let mut c = TextCache::default();
    for _ in 0..10 {
        c.get(&mut ts, (1, 1), "olá", 20.0, 100.0);
        c.end_frame();
    }
    assert_eq!(c.shaped(), 1, "um ecrã parado remoldou");
    c.get(&mut ts, (1, 1), "olá!", 20.0, 100.0);
    c.get(&mut ts, (1, 1), "olá!", 20.0, 120.0);
    assert_eq!(c.shaped(), 3);
    for _ in 0..(KEEP_FRAMES * 2) {
        c.end_frame();
    }
    assert!(c.is_empty(), "o que ninguém usa fica para sempre");
}

#[test]
fn editing_starts_with_everything_selected_and_typing_replaces_it() {
    let mut ts = TextSystem::without_system_fonts();
    let mut e = TextEdit::new(&mut ts, "antigo", 20.0, 200.0);
    assert_eq!(e.selected().as_deref(), Some("antigo"));
    e.insert(&mut ts, "novo");
    assert_eq!(e.text(), "novo");
    e.insert(&mut ts, "\nlinha");
    e.motion(&mut ts, Move::TextStart, false);
    e.motion(&mut ts, Move::WordRight, true);
    assert_eq!(e.selected().as_deref(), Some("novo"));
    e.backspace(&mut ts, false);
    assert_eq!(e.text(), "\nlinha");
    let _ = e.layout(&mut ts);
    let (_, caret) = e.decorations(1.0);
    assert!(caret.is_some(), "sem cursor");
}
