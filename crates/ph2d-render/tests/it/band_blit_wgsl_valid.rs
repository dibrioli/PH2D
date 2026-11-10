//! P13 L: o texto da colagem (com a luz na rampa à frente) e o da lei sozinha validam no naga.

#[test]
fn a_colagem_com_a_luz_na_rampa_valida() {
    for (nome, fonte) in [
        ("band_blit", ph2d_render::band_blit::BAND_BLIT_WGSL),
        ("luz_rampa", ph2d_render::luz_rampa::LUZ_RAMPA_WGSL),
    ] {
        let module = naga::front::wgsl::parse_str(fonte)
            .unwrap_or_else(|e| panic!("{nome}: {}", e.emit_to_string(fonte)));
        naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::all(),
        )
        .validate(&module)
        .unwrap_or_else(|e| panic!("{nome}: {e:?}"));
    }
}
