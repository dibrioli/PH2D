use super::*;

#[test]
fn chips_wrap_when_the_next_one_does_not_fit() {
    assert_eq!(chip_rows(&[], 232.0), 0);
    assert_eq!(chip_rows(&[100.0, 100.0], 232.0), 1);
    assert_eq!(chip_rows(&[100.0, 100.0, 100.0], 232.0), 2);
}

#[test]
fn initials_take_the_first_letter_of_two_words() {
    assert_eq!(initials(&Member("enio dibrioli".into())), "ED");
    assert_eq!(initials(&Member("Mia".into())), "M");
}

const HOUR: i64 = 3_600_000;

/// ⭐ O tom da entrega em três instantes: atrasada, a menos de 24 h, concluída — e sem data.
#[test]
fn the_due_badge_takes_the_tone_of_the_instant() {
    let z = Zone::utc();
    let now = 1_793_000_000_000;
    let tone = |due: Option<(i64, bool)>| due_badge(None, due, now, &z).map(|b| b.tone);
    assert_eq!(tone(Some((now - HOUR, false))), Some(DueTone::Overdue));
    assert_eq!(tone(Some((now + 2 * HOUR, false))), Some(DueTone::Soon));
    assert_eq!(tone(Some((now - HOUR, true))), Some(DueTone::Done));
    assert_eq!(tone(Some((now + 72 * HOUR, false))), Some(DueTone::Neutral));
    assert_eq!(tone(None), None);
}

#[test]
fn start_and_due_read_as_a_range_and_start_alone_says_starts() {
    let z = Zone::utc();
    // 2026-10-28 12:00 UTC.
    let now = 1_793_188_800_000;
    let range = due_badge(
        Some(now - 48 * HOUR),
        Some((now + 96 * HOUR, false)),
        now,
        &z,
    );
    assert_eq!(
        range.map(|b| b.text).as_deref(),
        Some("Oct 26 \u{2013} Nov 1")
    );
    let starts = due_badge(Some(now + 48 * HOUR), None, now, &z).expect("início");
    assert_eq!(
        (starts.text.as_str(), starts.tone),
        ("Starts: Oct 30", DueTone::Neutral)
    );
    let today = due_badge(None, Some((now + 2 * HOUR, false)), now, &z).expect("hoje");
    assert_eq!(
        today.text, "Today 2:00 PM",
        "a frente diz o prazo como o verso"
    );
}

#[test]
fn a_number_field_reads_as_written() {
    assert_eq!(number(3.0), "3");
    assert_eq!(number(2.5), "2.5");
}

/// O chip de rótulo tem a largura do nome: o orçamento nunca fica abaixo da medida. O controlo mostra
/// que a subtracção de antes ficava (um ulp) em algum nome de alguma fonte.
/// *Mutação: `chip_room` volta a `w − 2·respiro`.*
#[test]
fn a_label_chip_never_budgets_less_than_the_name_that_sized_it() {
    use ph2d_tokens::{UiFont, UiTextStyle};
    let mut ts = TextSystem::without_system_fonts();
    let mut curtos_na_subtraccao = 0;
    for font in UiFont::ALL {
        ph2d_text::set_active_text_style(UiTextStyle {
            font,
            ..UiTextStyle::default()
        });
        for i in 0..200 {
            let nome = format!("Priority {i}");
            let medido = ts.prefix_width(&nome, m::SMALL_PX);
            let w = chip_w(&mut ts, &nome);
            assert!(chip_room(&mut ts, &nome, w) >= medido, "{font:?} {nome}");
            if w - 2.0 * m::LABEL_PAD_X < medido {
                curtos_na_subtraccao += 1;
            }
        }
    }
    ph2d_text::set_active_text_style(UiTextStyle::default());
    assert!(
        curtos_na_subtraccao > 0,
        "o controlo: a varredura apanha o ulp"
    );
}
