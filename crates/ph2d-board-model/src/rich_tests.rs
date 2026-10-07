use super::*;

fn bold() -> Marks {
    Marks {
        bold: true,
        ..Marks::default()
    }
}

#[test]
fn bolding_the_middle_word_leaves_one_span_over_it() {
    let mut r = RichText::plain("uma boa ideia");
    r.toggle(4..7, Mark::Bold);
    assert_eq!(
        r.spans(),
        &[Span {
            start: 4,
            end: 7,
            marks: bold()
        }]
    );
    assert!(r.all(4..7, Mark::Bold));
    assert!(!r.all(0..7, Mark::Bold), "metade a negrito não é «tudo»");
    r.toggle(4..7, Mark::Bold);
    assert!(r.spans().is_empty(), "o 2.º Ctrl+B desfaz");
}

#[test]
fn a_toggle_over_a_partly_bold_range_bolds_all_of_it_and_merges() {
    let mut r = RichText::plain("abcdef");
    r.toggle(1..3, Mark::Bold);
    r.toggle(0..6, Mark::Bold);
    assert_eq!(r.spans().len(), 1);
    assert_eq!(r.spans()[0].range(), 0..6);
}

#[test]
fn typing_inside_a_span_grows_it_and_after_it_shifts_it() {
    let mut r = RichText::plain("ab cd");
    r.toggle(0..2, Mark::Italic);
    let m = r.typing_marks(1);
    r.replace(1..1, "XY", m);
    assert_eq!(r.as_str(), "aXYb cd");
    assert_eq!(
        r.spans()[0].range(),
        0..4,
        "escrever dentro do itálico continua itálico"
    );
    let at_end = r.typing_marks(r.len());
    assert_eq!(at_end, Marks::default());
    r.replace(0..0, "»", Marks::default());
    assert_eq!(
        r.spans()[0].range(),
        2..6,
        "o « » tem 2 bytes: o trecho anda com o texto"
    );
}

#[test]
fn deleting_across_two_spans_keeps_the_survivors() {
    let mut r = RichText::plain("aaabbbccc");
    r.toggle(0..3, Mark::Bold);
    r.toggle(6..9, Mark::Underline);
    r.replace(2..7, "", Marks::default());
    assert_eq!(r.as_str(), "aacc");
    assert_eq!(r.spans()[0].range(), 0..2);
    assert!(r.spans()[0].marks.bold);
    assert_eq!(r.spans()[1].range(), 2..4);
    assert!(r.spans()[1].marks.underline);
}

#[test]
fn deleting_everything_leaves_no_span() {
    let mut r = RichText::plain("abc");
    r.toggle(0..3, Mark::Strike);
    r.replace(0..3, "", Marks::default());
    assert!(r.is_empty());
    assert!(r.spans().is_empty());
}

#[test]
fn a_range_in_the_middle_of_a_character_snaps_to_its_boundary() {
    let mut r = RichText::plain("olá!");
    // «á» ocupa os bytes 2..4: 3 está a meio dele.
    r.toggle(3..5, Mark::Bold);
    assert_eq!(r.spans()[0].range(), 2..5);
}

#[test]
fn colour_reports_one_colour_or_none_when_mixed() {
    let red = Some(Rgba([200, 0, 0, 255]));
    let mut r = RichText::plain("abcd");
    r.restyle(0..2, |m| m.color = red);
    assert_eq!(r.color(0..2), Some(red));
    assert_eq!(r.color(2..4), Some(None));
    assert_eq!(r.color(0..4), None, "duas cores: nenhuma está «escolhida»");
}

#[test]
fn spans_from_outside_are_normalised() {
    let r = RichText::with_spans(
        "abcdef".into(),
        &[
            Span {
                start: 4,
                end: 99,
                marks: bold(),
            },
            Span {
                start: 0,
                end: 4,
                marks: bold(),
            },
        ],
    );
    assert_eq!(r.spans().len(), 1, "dois vizinhos iguais são um");
    assert_eq!(r.spans()[0].range(), 0..6, "o fim passava do texto");
}
