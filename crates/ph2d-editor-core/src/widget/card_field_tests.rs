use super::*;

#[test]
fn the_field_text_accepts_arithmetic_and_a_unit_suffix() {
    for (t, v) in [
        ("0.35*2", 0.7),
        ("120+16", 136.0),
        ("(1+2)/3", 1.0),
        ("-45", -45.0),
        ("12px", 12.0),
        ("45deg", 45.0),
        ("50 %", 50.0),
        ("2 * -3", -6.0),
        ("1,5", 1.5),
    ] {
        let got = eval_number(t).unwrap_or_else(|| panic!("{t} não avaliou"));
        assert!((got - v).abs() < 1e-9, "{t} = {got}, esperado {v}");
    }
    for t in ["", "abc", "1+", "(2", "1/0"] {
        assert_eq!(eval_number(t), None, "{t} devia recusar");
    }
}

#[test]
fn only_the_icon_cell_is_the_scrub_handle() {
    let f = Rect::new(10.0, 0.0, 120.0, 28.0);
    assert!(on_handle(f, 10.0));
    assert!(on_handle(
        f,
        10.0 + ph2d_card_stack::metrics::ICON_CELL - 0.5
    ));
    assert!(!on_handle(f, 10.0 + ph2d_card_stack::metrics::ICON_CELL));
    assert!(!on_handle(f, 9.0));
    assert_eq!(value_rect(f).x, handle_rect(f).x + handle_rect(f).w);
}

/// ⭐ Os algarismos da fonte de fábrica (Noto Sans) já são TABULARES — medido, não suposto: a
/// `ph2d-text` não tem API de features OpenType (`tnum`), e com a Noto Sans ela não é precisa.
#[test]
fn the_factory_font_digits_are_tabular() {
    let mut ts = ph2d_text::TextSystem::new();
    let w: Vec<f32> = (0..10)
        .map(|d| ts.prefix_width(&d.to_string().repeat(8), ph2d_card_stack::metrics::VALUE_PX))
        .collect();
    let (lo, hi) = w
        .iter()
        .fold((f32::MAX, 0.0f32), |(a, b), x| (a.min(*x), b.max(*x)));
    assert!(hi - lo < 0.01, "larguras de 8 algarismos 0..9: {w:?}");
}
