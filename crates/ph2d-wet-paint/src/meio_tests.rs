use super::*;

#[test]
fn sem_meio_as_leis_do_liquido_sao_as_da_agua_ao_bit() {
    assert_eq!(mobilidade(0.0).to_bits(), 1f64.to_bits());
    assert_eq!(evaporacao(0.0).to_bits(), 1f64.to_bits());
    for r in [0.0, 0.3, 0.97, 1.0] {
        assert_eq!(retencao(r, 0.0).to_bits(), f64::to_bits(r));
    }
}

#[test]
fn o_oleo_corre_mais_devagar_seca_mais_devagar_e_acaba_por_parar() {
    assert!((mobilidade(1.0) - 1.0 / ETA_OLEO).abs() < 1e-12);
    assert!(mobilidade(0.5) < 1.0 && mobilidade(0.5) > mobilidade(1.0));
    assert!((evaporacao(1.0) - ENDURECE).abs() < 1e-12);
    assert!(
        evaporacao(1.0) > 0.0,
        "sem perda nenhuma o solver corria para sempre"
    );
    let r = 0.97;
    assert!(retencao(r, 1.0) > r && retencao(r, 1.0) < 1.0);
}

#[test]
fn a_aguada_sobre_o_oleo_seco_e_agua_e_o_oleo_grosso_no_papel_limpo_e_oleo() {
    // Óleo seco (nada molhado) recebe a aguada: o meio passa a ser o dela.
    assert_eq!(mistura(1.0, 0.0, 0.0, 5.0, 2.0), 0.0);
    // Papel limpo recebe óleo grosso (quase sem água).
    assert_eq!(mistura(0.0, 0.0, 1.0, 3000.0, 0.0), 1.0);
    // Metade molhada de cada: o meio do meio.
    let m = mistura(0.0, 1000.0, 1.0, 1000.0, 0.0);
    assert!((m - 0.5).abs() < 1e-6);
}

#[test]
fn a_pasta_do_dab_e_continua_no_limiar_e_nula_na_agua() {
    // Água: nada é pasta, ao bit.
    assert_eq!(pasta(0.0, 600.0, 0.3).to_bits(), 0f64.to_bits());
    // Sem líquido livre (o tubo): tudo é pasta — o ramo da ponta seca.
    assert_eq!(pasta(1.0, 600.0, 0.0), 1.0);
    // Dilution 0,127 (J7, P600 × W0,5): ~76 % prende-se; Dilution 0,9: ~1 %.
    let dab = |d: f64| {
        let phi = 1.0 - d;
        pasta(
            1.0,
            phi,
            liquido_livre(phi, 1.0) / crate::painter::receita::K_W,
        )
    };
    assert!((dab(0.127) - 0.764).abs() < 0.01, "{}", dab(0.127));
    assert!(dab(0.9) < 0.015, "{}", dab(0.9));
    // Contínua: logo acima da pasta quase tudo continua pasta.
    assert!(dab(0.1001) > 0.99);
}

#[test]
fn o_oleo_do_tubo_e_pasta_e_so_o_diluente_a_mais_corre() {
    assert_eq!(liquido_livre(0.4, 0.0).to_bits(), (1.0f64 - 0.4).to_bits());
    assert_eq!(
        liquido_livre(0.95, 1.0),
        0.0,
        "a tinta do tubo não tem líquido livre"
    );
    let limiar = 1.0 / (1.0 + LIGADO_NO_OLEO);
    assert_eq!(liquido_livre(limiar, 1.0), 0.0);
    assert!(
        liquido_livre(0.5, 1.0) > 0.4,
        "o óleo diluído tem líquido que corre"
    );
    // Contínua: sem degrau no limiar.
    assert!(liquido_livre(limiar - 1e-6, 1.0) < 1e-5);
}
