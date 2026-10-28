use ph2d_richdoc::{Inline, Mark, MarkSpan};

fn span(start: usize, end: usize, mark: Mark) -> MarkSpan {
    MarkSpan { start, end, mark }
}

#[test]
fn inline_insert_inside_a_mark_splits_it_and_the_new_text_carries_only_its_marks() {
    let mut i = Inline::plain("abcd");
    i.add_mark(0..4, Mark::Bold);
    i.insert(2, "XY", &[Mark::Italic]);
    assert_eq!(i.text(), "abXYcd");
    assert_eq!(
        i.marks(),
        &[
            span(0, 2, Mark::Bold),
            span(2, 4, Mark::Italic),
            span(4, 6, Mark::Bold)
        ]
    );
    i.insert(2, "", &[]);
    i.add_mark(2..4, Mark::Bold);
    assert_eq!(
        i.marks(),
        &[span(0, 6, Mark::Bold), span(2, 4, Mark::Italic)]
    );
}

#[test]
fn inline_delete_shifts_and_clips_marks_and_a_cut_mention_loses_its_mark() {
    let mut i = Inline::plain("hello @ana bye");
    i.add_mark(0..5, Mark::Bold);
    i.add_mark(6..10, Mark::Mention);
    i.delete(3..7);
    assert_eq!(i.text(), "helana bye");
    assert_eq!(i.marks(), &[span(0, 3, Mark::Bold)]);
}

#[test]
fn inline_toggle_coverage_and_remove_cut_a_hole() {
    let mut i = Inline::plain("abcdef");
    i.add_mark(0..3, Mark::Strike);
    i.add_mark(3..6, Mark::Strike);
    assert_eq!(
        i.marks(),
        &[span(0, 6, Mark::Strike)],
        "adjacentes fundem-se"
    );
    assert!(i.covers(1..5, &Mark::Strike));
    i.remove_mark(2..4, &Mark::Strike);
    assert_eq!(
        i.marks(),
        &[span(0, 2, Mark::Strike), span(4, 6, Mark::Strike)]
    );
    assert!(!i.covers(0..6, &Mark::Strike));
}

#[test]
fn inline_a_link_replaces_another_link_and_two_urls_never_overlap() {
    let mut i = Inline::plain("abcdef");
    i.add_mark(0..4, Mark::Link("u".into()));
    i.add_mark(2..6, Mark::Link("v".into()));
    assert_eq!(
        i.marks(),
        &[
            span(0, 2, Mark::Link("u".into())),
            span(2, 6, Mark::Link("v".into()))
        ]
    );
    assert!(
        i.covers(0..6, &Mark::Link(String::new())),
        "qualquer url conta como link"
    );
}

#[test]
fn inline_typing_inherits_inclusive_marks_and_code_or_link_only_from_inside() {
    let mut i = Inline::plain("abcdef");
    i.add_mark(0..2, Mark::Bold);
    i.add_mark(2..4, Mark::Code);
    i.add_mark(4..6, Mark::Link("u".into()));
    assert_eq!(i.marks_for_typing(0), vec![]);
    assert_eq!(i.marks_for_typing(2), vec![Mark::Bold]);
    assert_eq!(i.marks_for_typing(3), vec![Mark::Code]);
    assert_eq!(i.marks_for_typing(4), vec![]);
    assert_eq!(i.marks_for_typing(5), vec![Mark::Link("u".into())]);
    assert_eq!(i.marks_for_typing(6), vec![]);
}

#[test]
fn inline_split_and_append_are_inverse_and_runs_cover_the_text() {
    let mut i = Inline::plain("one two three");
    i.add_mark(2..9, Mark::Italic);
    i.add_mark(4..7, Mark::Bold);
    let orig = i.clone();
    let tail = i.split_off(5);
    assert_eq!(tail.marks()[0], span(0, 2, Mark::Bold));
    i.append(tail);
    assert_eq!(i, orig);
    let runs = i.runs();
    let ranges: Vec<_> = runs.iter().map(|r| r.range.clone()).collect();
    assert_eq!(ranges, vec![0..2, 2..4, 4..7, 7..9, 9..13]);
    assert_eq!(runs[2].marks, vec![Mark::Bold, Mark::Italic]);
}

#[test]
fn inline_adjacent_mentions_stay_two_people() {
    let mut i = Inline::plain("@a@b");
    i.add_mark(0..2, Mark::Mention);
    i.add_mark(2..4, Mark::Mention);
    assert_eq!(i.marks().len(), 2);
    assert_eq!(i.mention_at(3), Some(2..4));
}
