use super::*;
use ph2d_board_model::Marks;

fn bold() -> Marks {
    Marks {
        bold: true,
        ..Marks::default()
    }
}

#[test]
fn a_note_becomes_title_and_markdown_and_comes_back() {
    let text = "Fix jump\nThe hero falls\nthrough the floor";
    let hero = text.find("hero").unwrap() as u32;
    let rich = RichText::with_spans(
        text.into(),
        &[Span {
            start: hero,
            end: hero + 4,
            marks: bold(),
        }],
    );
    let c = card_from_note(&rich);
    assert_eq!(c.title, "Fix jump");
    assert_eq!(c.description, "The **hero** falls\n\nthrough the floor\n");
    let back = note_from_card(&c.title, &c.description);
    assert_eq!(back.as_str(), text);
    assert_eq!(back.spans(), rich.spans(), "o negrito volta");
}

#[test]
fn an_empty_or_one_line_note_has_no_description() {
    assert_eq!(
        card_from_note(&RichText::plain("\n  Idea  \n\n")).title,
        "Idea"
    );
    assert!(
        card_from_note(&RichText::plain("Idea"))
            .description
            .is_empty()
    );
    assert!(card_from_note(&RichText::plain("")).title.is_empty());
}

/// Cada cor de nota vai à etiqueta do mesmo matiz e volta à MESMA nota (a de nascença do Miro, a
/// cinza-branca, não pede etiqueta).
#[test]
fn note_colours_map_to_the_nearest_label_and_back() {
    assert_eq!(label_for_note(Rgba(STICKY_COLORS[0])), None);
    let yellow = label_for_note(Rgba(STICKY_COLORS[STICKY_DEFAULT_COLOR])).unwrap();
    assert_eq!(yellow.hue, Hue::Yellow);
    let mut same = 0;
    for c in &STICKY_COLORS[1..] {
        let l = label_for_note(Rgba(*c)).unwrap();
        if note_color_for(Some(l)).0 == *c {
            same += 1;
        }
    }
    // Medido 28/10: 11 de 15 — as 4 restantes partilham matiz com outra nota (dois verdes, dois
    // rosas…) e voltam à vizinha. Mudar a paleta muda este número: confira as cores à mão.
    assert_eq!(same, 11, "ida e volta: {same} de 15 voltam à mesma cor");
    assert_eq!(
        note_color_for(None),
        Rgba(STICKY_COLORS[STICKY_DEFAULT_COLOR])
    );
}
