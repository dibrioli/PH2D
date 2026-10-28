use crate::{at, doc, texts};
use ph2d_richdoc::{BlockKind as K, Caret, Mark, RichDoc, Selection};

fn s(t: &str) -> String {
    t.to_owned()
}

#[test]
fn edit_insert_text_with_newlines_splits_blocks_but_code_keeps_them() {
    let mut d = doc(&[(K::Paragraph, "ab")]);
    let c = d.insert_text(at(0, 1), "x\ny", &[]);
    assert_eq!(
        texts(&d),
        vec![(K::Paragraph, s("ax")), (K::Paragraph, s("yb"))]
    );
    assert_eq!(c, Caret::new(1, 1));
    let mut d = doc(&[(K::Code("rs".into()), "ab")]);
    d.insert_text(at(0, 1), "x\ny", &[Mark::Bold]);
    assert_eq!(d.blocks()[0].inline.text(), "ax\nyb");
    assert!(
        d.blocks()[0].inline.marks().is_empty(),
        "código não leva marcas"
    );
}

#[test]
fn edit_delete_selection_across_blocks_keeps_the_first_kind() {
    let mut d = doc(&[
        (K::heading(2), "Title"),
        (K::Paragraph, "mid"),
        (K::Bullet, "tail"),
    ]);
    let c = d.delete_selection(Selection::new(Caret::new(2, 2), Caret::new(0, 3)));
    assert_eq!(texts(&d), vec![(K::heading(2), s("Titil"))]);
    assert_eq!(c, Caret::new(0, 3));
}

#[test]
fn edit_backspace_at_block_start_unlists_then_outdents_then_joins() {
    let mut d = doc(&[(K::Bullet, "a"), (K::Bullet, "b")]);
    d.indent(at(1, 0));
    assert_eq!(d.blocks()[1].indent, 1);
    d.delete_backward(at(1, 0));
    assert_eq!(d.blocks()[1].kind, K::Paragraph, "1.º: deixa de ser item");
    assert_eq!(d.blocks()[1].indent, 1);
    d.delete_backward(at(1, 0));
    assert_eq!(d.blocks()[1].indent, 0, "2.º: desindenta");
    let c = d.delete_backward(at(1, 0));
    assert_eq!(texts(&d), vec![(K::Bullet, s("ab"))], "3.º: junta");
    assert_eq!(c, Caret::new(0, 1));
}

#[test]
fn edit_backspace_and_delete_remove_a_whole_mention_and_whole_chars() {
    let mut d = RichDoc::new();
    let c = d.insert_mention(at(0, 0), "ana");
    assert_eq!(d.blocks()[0].inline.text(), "@ana ");
    let c = d.delete_backward(Selection::caret(c));
    let c = d.delete_backward(Selection::caret(c));
    assert_eq!(d.blocks()[0].inline.text(), "");
    assert_eq!(c, Caret::new(0, 0));
    let mut d = doc(&[(K::Paragraph, "çé")]);
    let c = d.delete_forward(at(0, 0));
    assert_eq!((d.blocks()[0].inline.text(), c.byte), ("é", 0));
}

#[test]
fn edit_delete_forward_at_end_joins_and_swallows_a_following_rule() {
    let mut d = doc(&[(K::Paragraph, "a"), (K::Rule, ""), (K::Quote, "b")]);
    d.delete_forward(at(0, 1));
    assert_eq!(texts(&d), vec![(K::Paragraph, s("a")), (K::Quote, s("b"))]);
    d.delete_forward(at(0, 1));
    assert_eq!(texts(&d), vec![(K::Paragraph, s("ab"))]);
}

#[test]
fn edit_enter_splits_and_new_check_is_unchecked_and_heading_tail_is_paragraph() {
    let mut d = doc(&[(K::Check(true), "buy milk")]);
    let c = d.enter(at(0, 3));
    assert_eq!(
        texts(&d),
        vec![(K::Check(true), s("buy")), (K::Check(false), s(" milk"))]
    );
    assert_eq!(c, Caret::new(1, 0));
    let mut d = doc(&[(K::heading(1), "Hello")]);
    d.enter(at(0, 5));
    assert_eq!(
        texts(&d),
        vec![(K::heading(1), s("Hello")), (K::Paragraph, s(""))]
    );
    let mut d = doc(&[(K::heading(1), "Hello")]);
    let c = d.enter(at(0, 0));
    assert_eq!(
        texts(&d),
        vec![(K::Paragraph, s("")), (K::heading(1), s("Hello"))]
    );
    assert_eq!(c, Caret::new(1, 0));
}

#[test]
fn edit_enter_on_empty_item_outdents_then_leaves_the_list() {
    let mut d = doc(&[(K::Numbered, "a"), (K::Numbered, "")]);
    d.indent(at(1, 0));
    d.enter(at(1, 0));
    assert_eq!(d.blocks()[1].indent, 0);
    assert_eq!(d.blocks()[1].kind, K::Numbered);
    d.enter(at(1, 0));
    assert_eq!(d.blocks()[1].kind, K::Paragraph);
}

#[test]
fn edit_enter_in_code_inserts_newline_and_twice_at_end_leaves() {
    let mut d = doc(&[(K::Code(String::new()), "x")]);
    let c = d.enter(at(0, 1));
    assert_eq!(d.blocks()[0].inline.text(), "x\n");
    let c = d.enter(Selection::caret(c));
    assert_eq!(
        texts(&d),
        vec![(K::Code(String::new()), s("x")), (K::Paragraph, s(""))]
    );
    assert_eq!(c, Caret::new(1, 0));
}

#[test]
fn edit_toggle_mark_adds_when_partial_and_removes_when_total_across_blocks() {
    let mut d = doc(&[
        (K::Paragraph, "abc"),
        (K::Code(String::new()), "c"),
        (K::Bullet, "def"),
    ]);
    d.toggle_mark(
        Selection::new(Caret::new(0, 0), Caret::new(0, 1)),
        &Mark::Bold,
    );
    let all = Selection::new(Caret::new(0, 1), Caret::new(2, 2));
    assert!(!d.mark_active(all, &Mark::Bold));
    assert!(d.toggle_mark(all, &Mark::Bold));
    assert!(d.mark_active(all, &Mark::Bold));
    assert!(
        d.blocks()[1].inline.marks().is_empty(),
        "código fica de fora"
    );
    assert_eq!(d.blocks()[0].inline.marks()[0].end, 3);
    d.toggle_mark(all, &Mark::Bold);
    assert_eq!(
        d.blocks()[0].inline.marks()[0].end,
        1,
        "o que estava antes fica"
    );
    assert!(d.blocks()[2].inline.marks().is_empty());
}

#[test]
fn edit_set_block_kind_and_code_strips_marks() {
    let mut d = doc(&[(K::Paragraph, "a"), (K::Paragraph, "b")]);
    d.toggle_mark(
        Selection::new(Caret::new(0, 0), Caret::new(0, 1)),
        &Mark::Italic,
    );
    d.set_block_kind(
        Selection::new(Caret::new(0, 0), Caret::new(1, 0)),
        K::Code("py".into()),
    );
    assert!(d.blocks().iter().all(|b| b.kind == K::Code("py".into())));
    assert!(d.blocks()[0].inline.marks().is_empty());
    d.set_block_kind(at(0, 0), K::Heading(9));
    assert_eq!(d.blocks()[0].kind, K::Heading(3), "título preso a 1..=3");
}

#[test]
fn edit_indent_never_skips_a_level_and_children_follow_an_outdent() {
    let mut d = doc(&[(K::Bullet, "a"), (K::Bullet, "b"), (K::Bullet, "c")]);
    d.indent(at(0, 0));
    assert_eq!(d.blocks()[0].indent, 0, "o primeiro não tem pai");
    d.indent(at(1, 0));
    d.indent(at(1, 0));
    assert_eq!(d.blocks()[1].indent, 1, "no máximo um nível abaixo do pai");
    d.indent(at(2, 0));
    d.indent(at(2, 0));
    assert_eq!(d.blocks()[2].indent, 2);
    d.outdent(at(1, 0));
    assert_eq!((d.blocks()[1].indent, d.blocks()[2].indent), (0, 1));
}

#[test]
fn edit_insert_rule_converts_an_empty_block_and_leaves_an_editable_paragraph() {
    let mut d = RichDoc::new();
    let c = d.insert_rule(at(0, 0));
    assert_eq!(texts(&d), vec![(K::Rule, s("")), (K::Paragraph, s(""))]);
    assert_eq!(c, Caret::new(1, 0));
    let c = d.delete_backward(Selection::caret(c));
    assert_eq!(texts(&d), vec![(K::Paragraph, s(""))]);
    assert_eq!(c, Caret::new(0, 0));
}

#[test]
fn edit_plain_text_and_caret_motion() {
    let d = doc(&[(K::heading(1), "T"), (K::Rule, ""), (K::Paragraph, "aé")]);
    assert_eq!(d.plain_text(), "T\naé");
    assert_eq!(d.move_right(Caret::new(0, 1)), Caret::new(1, 0));
    assert_eq!(d.move_left(Caret::new(2, 3)), Caret::new(2, 1));
    assert_eq!(
        d.clamp(Caret::new(9, 2)),
        Caret::new(2, 1),
        "fronteira de char"
    );
}
