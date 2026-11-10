//! **R12 · a revisão independente final**: o trilho vazio dos medidores do Try it é cinza (nunca o acento), a
//! coluna dos chips da lista não anda quando a busca estreita, e a prévia das 11 acções lê-se (escala ≥ 0,75).

use super::try_it_censo::cena;
use ph2d_app_input_map::view_try::{METER_TRACK, meter_fill};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ColorToken, Theme};

#[test]
fn o_trilho_vazio_do_medidor_e_cinza_e_a_forca_e_forca_vezes_trilho() {
    for th in [Theme::default(), Theme::Dark, Theme::Gray, Theme::Oled] {
        let track = METER_TRACK.resolve(th);
        for accent in [ColorToken::Accent, ColorToken::BorderEmph] {
            assert_ne!(track, accent.resolve(th), "{th:?}: o trilho não é o acento");
        }
        // Um cinza de superfície (o tom das teclas): muito mais perto do cartão que do acento do tema.
        let d = |a: ph2d_tokens::Color, b: ph2d_tokens::Color| {
            (i32::from(a.r) - i32::from(b.r)).abs()
                + (i32::from(a.g) - i32::from(b.g)).abs()
                + (i32::from(a.b) - i32::from(b.b)).abs()
        };
        let accent = ColorToken::Accent.resolve(th);
        let card = ColorToken::KanbanCard.resolve(th);
        assert!(
            d(track, card) * 3 < d(track, accent),
            "{th:?}: o trilho é um cinza de superfície ({track:?} × cartão {card:?} × acento {accent:?})"
        );
        assert_ne!(track, card, "{th:?}: o trilho vê-se a 0.00 sobre o cartão");
    }
    let bar = Rect::new(10.0, 4.0, 200.0, 6.0); // LITERAL-PX-OK: um trilho de prova
    assert_eq!(meter_fill(bar, 0.0), None, "a 0.00 só o trilho cinza");
    let f = meter_fill(bar, 0.55).expect("move_left a 0.55");
    assert!(
        (f.w - 0.55 * bar.w).abs() < 1e-3,
        "largura = força × trilho: {f:?}"
    );
    assert_eq!((f.x, f.y, f.h), (bar.x, bar.y, bar.h));
}

#[test]
fn escrever_na_busca_nunca_mexe_a_coluna_dos_chips() {
    let z_x = |m: &super::Mesa| {
        let xs: Vec<f32> = (m.tab().doc.census.texts.iter())
            .filter(|t| t.1 == "Z")
            .map(|t| t.0.x)
            .collect();
        assert_eq!(xs.len(), 1, "o chip Z do jump, uma vez: {xs:?}");
        xs[0]
    };
    let mut m = cena("A3");
    let filtered = z_x(&m);
    m.tab_mut().doc.list.search_key = None;
    m.frame();
    let all = z_x(&m);
    assert!(
        (filtered - all).abs() < 0.5,
        "o chip Z não anda com a busca: {filtered} × {all}"
    );
}

/// ⛔ R13: a área do `frame()` é mais larga que a aba real e dava verde com a foto `c1_1600` a ~5 px de texto. A
/// prova pinta na área que a aba RECEBE com a Hierarchy e o Inspector abertos (medida nas fotos c1_1600/c1_1280).
#[test]
fn a_previa_das_onze_accoes_le_se_a_pelo_menos_tres_quartos() {
    let tabs = [
        ("1600", Rect::new(308.0, 28.0, 988.0, 890.0)), // LITERAL-PX-OK: a aba medida na foto c1_1600
        ("1280", Rect::new(297.0, 28.0, 690.0, 604.0)), // LITERAL-PX-OK: a aba medida na foto c1_1280
    ];
    for (at, tab) in tabs {
        let mut m = cena("C1");
        assert_eq!(m.tab().doc.snap.map.len(), 11, "o mapa das 11 acções");
        m.paint_in(tab);
        let k = m.tab().doc.tryit.preview_scale;
        assert!(
            k >= 0.75 - 1e-4,
            "{at}: a prévia a {k:.3} (texto ilegível abaixo de 0,75)"
        );
    }
}

#[test]
fn as_receitas_cabem_numa_fila_ou_partem_se_ao_meio_nunca_uma_sozinha() {
    use ph2d_app_input_map::view_try::{recipe_cols, tile_w};
    let n = ph2d_input::recipes::Recipe::ALL.len();
    for w in (200..1600).step_by(7) {
        let w = w as f32;
        let k = recipe_cols(w);
        assert!(k == n || k == n.div_ceil(2) || k == 1, "{w}: {k} por fila");
        assert_ne!(n % k, 1, "{w}: uma receita sozinha numa fila");
        assert!(tile_w(w) * k as f32 <= w, "{w}: os ladrilhos cabem");
    }
}
