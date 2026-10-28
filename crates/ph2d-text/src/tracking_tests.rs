//! O espaçamento entre letras: `0` não muda nenhuma resposta; `> 0` abre o texto por letra, e medir
//! e pintar dão a mesma largura.

use super::*;

const PX: f32 = 15.0;

#[test]
fn zero_tracking_measures_the_same_as_before_to_the_bit() {
    let mut ts = TextSystem::without_system_fonts();
    for (text, w) in [
        ("BACKLOG", FontWeight::SEMI_BOLD),
        ("Short film — doing ", FontWeight::MEDIUM),
        ("IN REVIEW", FontWeight::SEMI_BOLD),
    ] {
        let before = ts.prefix_width_weighted(text, PX, w);
        let laid = if text.ends_with(' ') {
            before
        } else {
            ts.layout_with_weight(text, PX, f32::INFINITY, w).width()
        };
        let tracked = ts.prefix_width_tracked(text, PX, w, 0.0);
        assert_eq!(tracked.to_bits(), before.to_bits(), "{text}");
        assert_eq!(
            laid.to_bits(),
            before.to_bits(),
            "{text}: a porta antiga, por outro caminho"
        );
        let painted = ts
            .layout_for_rendering_tracked(text, PX, f32::INFINITY, w, active_text_rendering(), 0.0)
            .width();
        let old = ts
            .layout_for_rendering(text, PX, f32::INFINITY, w, active_text_rendering())
            .width();
        assert_eq!(painted.to_bits(), old.to_bits(), "{text}");
    }
}

#[test]
fn tracking_opens_every_letter_and_measure_equals_paint() {
    let mut ts = TextSystem::without_system_fonts();
    let text = "BACKLOG";
    let w = FontWeight::SEMI_BOLD;
    let em = 0.04;
    let plain = ts.prefix_width_tracked(text, PX, w, 0.0);
    let measured = ts.prefix_width_tracked(text, PX, w, em);
    let painted = ts
        .layout_for_rendering_tracked(text, PX, f32::INFINITY, w, active_text_rendering(), em)
        .width();
    assert_eq!(measured.to_bits(), painted.to_bits(), "medir == pintar");
    let per_letter = em * PX;
    let gained = measured - plain;
    let n = text.chars().count() as f32;
    assert!(
        gained >= (n - 1.0) * per_letter - 0.01 && gained <= n * per_letter + 0.01,
        "{gained} px por {n} letras de {per_letter} px"
    );
}
