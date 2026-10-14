//! ⭐ doc 121 §9.14 (c) — **O HALO DO `fx.glow` PELA ROTA DO QUADRO** (o elo que vivia no
//! `present_placa_tests` da shell até o passe sair dela). A LEI (que listas, que buffer) é a
//! `motion_glow_layer::halo_do_quadro`, com gates puros; este é o elo: o passe de isolamento recebe o
//! buffer do DISPOSITIVO e as formas entram pelo redesenho da camada. *Mutações: passar `None` em vez
//! do buffer (num quadro do dispositivo as sprites não brilham) · não redesenhar as formas (a forma
//! deixa de brilhar na placa).*
#[test]
fn o_halo_recebe_o_buffer_da_placa_e_a_camada_das_formas() {
    let fx = include_str!("motion_glow_passe.rs");
    assert!(fx.contains("halo_do_quadro(motion, formas.desenhou())"));
    assert!(fx.contains("        &halo.cpu,\n        halo.placa,"));
    assert!(fx.contains(
        "if halo.formas_da_placa && !formas.redesenha_em(gpu, g.motion_fx.rt_view(), tamanho)"
    ));
}
