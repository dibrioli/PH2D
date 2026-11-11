//! ⭐ A pilha da aba ([`ph2d_panel_code::stack`]): numa aba curta os cartões encolhem pela ordem até
//! ao mínimo para o código guardar as suas linhas; com espaço ficam como querem.

use ph2d_panel_code::stack::{Want, fit};

const fn w(h: f32, least: f32) -> Want {
    Want { h, least }
}

#[test]
fn com_espaco_cada_cartao_fica_como_quer() {
    assert_eq!(
        fit(
            900.0,
            100.0,
            8.0,
            [w(120.0, 40.0), w(200.0, 50.0), w(150.0, 56.0)]
        ),
        [120.0, 200.0, 150.0]
    );
}

#[test]
fn numa_aba_curta_encolhem_pela_ordem_ate_ao_minimo_e_o_codigo_guarda_o_seu() {
    // 400 - 100 de código = 300 para cartões; querem 120 + 200 + 150 + 3 vãos de 8 = 494.
    let hs = fit(
        400.0,
        100.0,
        8.0,
        [w(120.0, 40.0), w(200.0, 50.0), w(150.0, 56.0)],
    );
    assert_eq!(hs[0], 40.0, "a Bancada encolhe primeiro, até ao mínimo");
    assert_eq!(hs[2], 150.0, "a faixa só encolhe se o resto não chega");
    let used: f32 = hs.iter().map(|h| h + 8.0).sum();
    assert!(
        (used - 300.0).abs() < 1e-3,
        "o código fica com os seus 100: {hs:?}"
    );
    // Nem os mínimos cabem: ficam os mínimos; um cartão ausente (0) não paga vão.
    assert_eq!(
        fit(
            120.0,
            100.0,
            8.0,
            [w(0.0, 40.0), w(200.0, 50.0), w(150.0, 56.0)]
        ),
        [0.0, 50.0, 56.0]
    );
}
