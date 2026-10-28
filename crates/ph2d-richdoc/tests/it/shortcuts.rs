use crate::texts;
use ph2d_richdoc::{BlockKind as K, Editor, Mark, MarkSpan, RichDoc, Shortcut};

fn typed(keys: &str) -> Editor {
    let mut e = Editor::new(RichDoc::new());
    for ch in keys.chars() {
        e.type_text(&ch.to_string());
    }
    e
}

#[test]
fn shortcuts_block_prefixes_convert_the_paragraph() {
    let cases = [
        ("# ", K::heading(1)),
        ("## ", K::heading(2)),
        ("### ", K::heading(3)),
        ("- ", K::Bullet),
        ("* ", K::Bullet),
        ("1. ", K::Numbered),
        ("12. ", K::Numbered),
        ("[] ", K::Check(false)),
        ("[ ] ", K::Check(false)),
        ("[x] ", K::Check(true)),
        ("> ", K::Quote),
        ("``` ", K::Code(String::new())),
        ("```rust ", K::Code("rust".into())),
    ];
    for (keys, kind) in cases {
        let e = typed(&format!("{keys}hi"));
        assert_eq!(texts(e.doc()), vec![(kind, "hi".to_owned())], "{keys:?}");
    }
}

#[test]
fn shortcuts_not_triggered_mid_text_or_on_non_paragraphs() {
    let e = typed("a # b");
    assert_eq!(texts(e.doc()), vec![(K::Paragraph, "a # b".to_owned())]);
    let e = typed("# # x");
    assert_eq!(texts(e.doc()), vec![(K::heading(1), "# x".to_owned())]);
    let e = typed("1.5 ");
    assert_eq!(e.doc().blocks()[0].kind, K::Paragraph);
    // `[] ` dentro de um item com marcador torna-o checklist.
    let e = typed("- [] x");
    assert_eq!(texts(e.doc()), vec![(K::Check(false), "x".to_owned())]);
}

#[test]
fn shortcuts_three_dashes_become_a_rule_and_the_caret_moves_below() {
    let mut e = typed("---");
    assert_eq!(
        texts(e.doc()),
        vec![(K::Rule, String::new()), (K::Paragraph, String::new())]
    );
    e.type_text("x");
    assert_eq!(e.doc().blocks()[1].inline.text(), "x");
}

#[test]
fn shortcuts_enter_on_fence_or_dashes() {
    let mut e = typed("```js");
    assert_eq!(e.enter(), Some(Shortcut::Block(K::Code("js".into()))));
    e.type_text("a*b*");
    assert_eq!(
        texts(e.doc()),
        vec![(K::Code("js".into()), "a*b*".to_owned())]
    );
}

fn marks(e: &Editor) -> Vec<MarkSpan> {
    e.doc().blocks()[0].inline.marks().to_vec()
}

#[test]
fn shortcuts_inline_marks_wrap_the_content_and_typing_continues_unmarked() {
    let cases = [
        ("**bold**", Mark::Bold),
        ("*it*", Mark::Italic),
        ("~~gone~~", Mark::Strike),
        ("`code`", Mark::Code),
    ];
    for (keys, mark) in cases {
        let mut e = typed(&format!("x {keys}"));
        let inner = keys.trim_matches(|c| c == '*' || c == '~' || c == '`');
        assert_eq!(
            e.doc().blocks()[0].inline.text(),
            format!("x {inner}"),
            "{keys}"
        );
        let span = MarkSpan {
            start: 2,
            end: 2 + inner.len(),
            mark,
        };
        assert_eq!(marks(&e), vec![span.clone()], "{keys}");
        e.type_text("y");
        assert_eq!(
            marks(&e),
            vec![span],
            "{keys}: o texto seguinte sai sem a marca"
        );
    }
}

#[test]
fn shortcuts_inline_bad_cases_stay_literal() {
    for keys in ["a * b *", "** x**", "**", "a*", "``"] {
        let e = typed(keys);
        assert!(marks(&e).is_empty(), "{keys:?}");
        assert_eq!(e.doc().blocks()[0].inline.text(), keys);
    }
    // `**bo*` não fecha um itálico com metade do negrito.
    let e = typed("**bo*");
    assert!(marks(&e).is_empty());
}

#[test]
fn shortcuts_toggle_with_collapsed_caret_sets_pending_marks() {
    let mut e = typed("a");
    e.toggle_mark(Mark::Bold);
    e.type_text("b");
    e.type_text("c");
    e.toggle_mark(Mark::Bold);
    e.type_text("d");
    let want = vec![MarkSpan {
        start: 1,
        end: 3,
        mark: Mark::Bold,
    }];
    assert_eq!(marks(&e), want);
    assert_eq!(e.active_marks(), vec![]);
}

#[test]
fn shortcuts_paste_does_not_trigger_shortcuts_and_mentions_insert() {
    let mut e = Editor::new(RichDoc::new());
    e.paste("# no\n**x**");
    assert_eq!(
        texts(e.doc()),
        vec![
            (K::Paragraph, "# no".to_owned()),
            (K::Paragraph, "**x**".to_owned())
        ]
    );
    e.insert_mention("bea");
    assert_eq!(e.doc().blocks()[1].inline.text(), "**x**@bea ");
    e.toggle_check(1);
    assert_eq!(
        e.doc().blocks()[1].kind,
        K::Paragraph,
        "só alterna checklists"
    );
}
