//! W23 B14 — [`paint_text_block_spans`]: a cor dos trechos chega aos glifos DELES, nenhum glifo se
//! perde nem se repete, e a altura é a do pintor de uma cor só.

use super::*;

const BASE: Color = Color::from_rgba8(20, 20, 20, 255);
const BLUE: Color = Color::from_rgba8(0, 90, 255, 255);
const RED: Color = Color::from_rgba8(230, 0, 0, 255);

struct Ink {
    height: f32,
    /// O nº de glifos de cada corrida, pela ordem.
    runs: Vec<usize>,
    glyphs: usize,
    draw_data: Vec<u32>,
}

fn ink(text: &str, width: f32, spans: &[(std::ops::Range<usize>, Color)]) -> Ink {
    let mut ts = TextSystem::without_system_fonts();
    let mut scene = VectorScene::new();
    let height = paint_text_block_spans(
        &mut ts, &mut scene, text, 0.0, 0.0, 13.0, width, BASE, spans,
    );
    let e = scene.inner().encoding();
    Ink {
        height,
        runs: e
            .resources
            .glyph_runs
            .iter()
            .map(|r| r.glyphs.len())
            .collect(),
        glyphs: e.resources.glyphs.len(),
        draw_data: e.draw_data.clone(),
    }
}

/// ⭐ «hi @Rui there» com o `@Rui` a azul: três corridas (3 · 4 · 6 glifos), os mesmos 13 glifos e a
/// mesma altura que sem trechos; e a cor do trecho é a que entra no desenho (azul ≠ vermelho).
#[test]
fn a_span_paints_exactly_its_glyphs_in_its_colour() {
    let text = "hi @Rui there";
    let plain = ink(text, f32::INFINITY, &[]);
    let blue = ink(text, f32::INFINITY, &[(3..7, BLUE)]);
    let red = ink(text, f32::INFINITY, &[(3..7, RED)]);
    assert_eq!(plain.runs, vec![13]);
    assert_eq!(
        blue.runs,
        vec![3, 4, 6],
        "o trecho leva os glifos dele e só esses"
    );
    assert_eq!(blue.glyphs, plain.glyphs);
    assert_eq!(blue.height.to_bits(), plain.height.to_bits());
    assert_ne!(
        blue.draw_data, red.draw_data,
        "a cor do trecho não chegou ao desenho"
    );
    let base = ink(text, f32::INFINITY, &[(3..7, BASE)]);
    assert_eq!(
        base.runs,
        vec![13],
        "um trecho da cor de base não parte a corrida"
    );
    assert_eq!(base.draw_data, plain.draw_data);
}

/// Um texto que quebra: o trecho na 2.ª linha pinta-se lá, e nada se perde na quebra.
#[test]
fn a_span_on_a_wrapped_line_keeps_every_glyph() {
    let text = "a long message that wraps and then @everyone at the end";
    let plain = ink(text, 120.0, &[]);
    assert!(
        plain.runs.len() > 1,
        "o texto tem de quebrar: {:?}",
        plain.runs
    );
    let at = text.find("@everyone").unwrap();
    let blue = ink(text, 120.0, &[(at..at + "@everyone".len(), BLUE)]);
    assert_eq!(blue.glyphs, plain.glyphs);
    assert_eq!(blue.height.to_bits(), plain.height.to_bits());
    assert!(blue.runs.contains(&"@everyone".len()), "{:?}", blue.runs);
    assert_eq!(
        blue.runs.iter().sum::<usize>(),
        plain.runs.iter().sum::<usize>()
    );
}
