//! InputMap onda 3 item 7: a aba do som só rola na vertical — o deslizar de lado do trackpad não a toma.

use ph2d_editor_core::doc_surface::DocumentSurface;

#[test]
fn o_deslizar_de_lado_nao_toma_a_aba_do_som() {
    let mut d = ph2d_app_sfxlab::SfxLabDocs::default();
    let id = d.novo();
    let area = ph2d_editor_core::zones::Rect::new(0.0, 0.0, 1600.0, 1000.0); // LITERAL-PX-OK: a janela
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    let mut scene = ph2d_vector::VectorScene::new();
    d.paint(
        id,
        area,
        &mut scene,
        &mut ts,
        &ph2d_tokens::Theme::default(),
    );
    let antes = d.rect_de(id, ph2d_app_sfxlab::Alvo::Play);
    for i in 0..64u8 {
        let (x, y) = (
            100.0 + 200.0 * f32::from(i % 8),
            60.0 + 120.0 * f32::from(i / 8),
        );
        assert!(!d.wheel_sideways(id, x, y, -480.0), "({x}, {y})");
    }
    assert_eq!(d.rect_de(id, ph2d_app_sfxlab::Alvo::Play), antes);
}
