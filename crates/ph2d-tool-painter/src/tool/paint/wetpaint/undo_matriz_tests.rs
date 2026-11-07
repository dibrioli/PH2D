//! O undo vivo em todas as grades: os testes dele nasceram com `grid_ratio = flow_ratio = 1`, a única razão em que a
//! grade de FLUXO coincide com a fina, e por isso não viram o pânico do smoke do dono (07/11, `flow_x` fatiado com o
//! `s` da grade fina). Cada célula da matriz corre-os outra vez, com as razões postas em [`razoes`].

use super::undo_tinta_morre_tests::{AGUADA, razoes, tela, traco};

type Teste = (&'static str, fn());

macro_rules! testes {
    ($($m:ident::$n:ident),* $(,)?) => {
        &[$((stringify!($n), super::$m::$n as fn())),*]
    };
}

const TESTES: &[Teste] = testes![
    undo_tinta_viva_tests::undo_de_outro_traco_nao_mata_o_oleo_soluvel,
    undo_tinta_viva_tests::undo_de_outro_traco_nao_seca_a_aguarela_molhada,
    undo_tinta_viva_tests::redo_nao_mata_a_tinta_viva,
    undo_tinta_viva_tests::undo_devolve_a_ladeira_do_oleo_de_antes,
    undo_tinta_viva_tests::undo_redo_deixa_a_tinta_viva_a_andar,
    undo_tinta_viva_tests::undo_redo_com_o_worker_deixa_a_tinta_viva,
    undo_tinta_viva_tests::undo_que_nao_se_aplica_nao_drena_o_passo_em_voo,
    undo_tinta_morre_tests::desfazer_fundo_nao_mata_a_tinta_que_ficou,
    undo_tinta_morre_tests::o_primeiro_traco_desfeito_e_refeito_fica_vivo,
    undo_tinta_morre_tests::o_tecto_conta_os_registos_e_as_fotos,
    undo_tinta_morre_tests::para_la_do_tecto_so_o_traco_desfeito_acaba,
    undo_tinta_morre_tests::para_la_do_tecto_o_traco_que_cruza_nao_deixa_nada,
    undo_tinta_morre_tests::desfazer_depois_de_comprimir_nao_muda_nada,
    undo_tinta_morre_tests::para_la_do_tecto_os_passos_que_restam_ficam_esquecidos,
    undo_auditoria_tests::a_caixa_da_os_mesmos_planos_que_a_grade_inteira,
    undo_auditoria_tests::a_acao_de_tela_inteira_entra_no_registo_do_traco,
    undo_auditoria_tests::o_tecto_conta_o_que_cada_enterro_retem,
    undo_auditoria_tests::refazer_para_la_do_tecto_recusa_a_imagem_no_ar,
    undo_auditoria_tests::o_passo_que_o_documento_ja_largou_sai_do_tecto,
    undo_auditoria_tests::para_la_do_tecto_a_tinta_longe_fica_ao_bit,
    undo_drip_tests::undoing_a_wet_stroke_takes_the_drip_the_sim_painted_after_the_pen_up,
    undo_drip_tests::undoing_back_through_a_drip_that_was_painted_between_two_strokes,
    adormecida_tests::desfazer_o_traco_antes_de_voltar_mantem_a_tinta_viva,
    adormecida_tests::comprimir_a_tinta_adormecida_nao_muda_nada,
    adormecida_tests::a_agua_comprimida_corre_igual,
    adormecida_tests::a_agua_volta_a_correr,
];

fn corre(grid: u8, flow: u8) {
    let _razoes = razoes::com(grid, flow);
    // A matriz só prova algo se as razões chegam mesmo à sessão.
    let mut t = tela(300, 200, AGUADA, 20.0);
    traco(&mut t, 40.0, 260.0, 60.0);
    t.wet_bring_home();
    let s = t.paint.wetpaint.session.as_ref().expect("a sessão");
    assert_eq!(s.ratio, grid, "a razão da grade não chegou à sessão");
    assert_eq!(
        s.engine.flow_ratio,
        usize::from(flow),
        "a razão do fluxo não chegou à sessão"
    );
    let mut reprovados = Vec::new();
    for (nome, teste) in TESTES {
        if std::panic::catch_unwind(teste).is_err() {
            reprovados.push(*nome);
        }
        // O que um teste que reprova deixou posto, para não contaminar o seguinte.
        super::undo_tinta_morre_tests::teto::set(None);
        super::undo_auditoria_tests::grade_inteira::set(false);
    }
    assert!(
        reprovados.is_empty(),
        "grade {grid} × fluxo {flow}: {reprovados:?}"
    );
}

#[test]
fn undo_vivo_grade_1_fluxo_2() {
    corre(1, 2);
}

#[test]
fn undo_vivo_grade_1_fluxo_3() {
    corre(1, 3);
}

#[test]
fn undo_vivo_grade_1_fluxo_4() {
    corre(1, 4);
}

#[test]
fn undo_vivo_grade_2_fluxo_1() {
    corre(2, 1);
}

#[test]
fn undo_vivo_grade_2_fluxo_2() {
    corre(2, 2);
}

#[test]
fn undo_vivo_grade_4_fluxo_4() {
    corre(4, 4);
}

/// ⭐ O pânico do smoke do dono (07/11): tela de 2048 de largura, fluxo 2, um traço e o desfazer. O erro dele dizia
/// `range end index 1052676 out of range for slice of length 1051650`, na thread da água.
#[test]
fn desfazer_com_o_fluxo_mais_grosso_a_2048_nao_entra_em_panico() {
    let _razoes = razoes::com(1, 2);
    let mut t = tela(2048, 2048, AGUADA, 40.0);
    traco(&mut t, 200.0, 800.0, 200.0);
    traco(&mut t, 200.0, 800.0, 300.0);
    assert!(t.undo_last(), "havia um traço para desfazer");
    t.ticks_em_blocos(4);
    t.wet_bring_home();
    let g = &t
        .paint
        .wetpaint
        .session
        .as_ref()
        .expect("a sessão")
        .engine
        .layers[0]
        .grid;
    assert_eq!(g.flow_x.len(), g.flow.cells, "o plano mudou de tamanho");
    assert_eq!(g.flow_y.len(), g.flow.cells, "o plano mudou de tamanho");
}

/// A fotografia devolve o fluxo ao bit, na geometria da grade de FLUXO.
#[test]
fn a_foto_devolve_o_fluxo_ao_bit_em_toda_razao() {
    for flow in [1, 2, 3, 4, 16] {
        let mut e = ph2d_wet_paint::painter::Engine::with_flow_ratio(203, 150, flow);
        let g = &mut e.layers[0].grid;
        let (fx, fy): (Vec<f32>, Vec<f32>) = (0..g.flow.cells)
            .map(|i| ((i as f32).sin(), (i as f32).cos()))
            .unzip();
        g.flow_x.clone_from(&fx);
        g.flow_y.clone_from(&fy);
        let foto = super::undo_vivo::FotoDoMotor::de_para_sonda(&e);
        let g = &mut e.layers[0].grid;
        g.flow_x.fill(0.0);
        g.flow_y.fill(0.0);
        foto.repoe_para_sonda(&mut e);
        let g = &e.layers[0].grid;
        assert!(g.flow_x == fx, "fluxo {flow}: flow_x não voltou");
        assert!(g.flow_y == fy, "fluxo {flow}: flow_y não voltou");
    }
}
