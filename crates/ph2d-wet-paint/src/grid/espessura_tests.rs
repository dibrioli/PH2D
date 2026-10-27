use super::*;
use crate::grid::{restore_grid_region, snapshot_grid_region};

fn grade() -> Grid {
    Grid::new(16, 16)
}

#[test]
fn sem_aparar_o_campo_nao_se_aloca() {
    let mut g = grade();
    let i = g.s * 4 + 4;
    g.susp[i] = 1234.5;
    fixa(&mut g, i, 1234.5);
    assert!(g.espessura.is_empty());
    assert!((em(&g, i) - 1234.5).abs() < 1e-9);
}

#[test]
fn o_aparado_fica_e_sai_com_a_massa() {
    let mut g = grade();
    let i = g.s * 4 + 4;
    g.sett[i] = 3000.0;
    fixa(&mut g, i, 9000.0);
    assert!((em(&g, i) - 9000.0).abs() < 1e-3);
    // Quem tira metade da massa leva metade da espessura.
    g.sett[i] = 1500.0;
    assert!((em(&g, i) - 4500.0).abs() < 1e-3);
    // Levantar (assentado → suspenso) não a muda.
    g.sett[i] = 500.0;
    g.susp[i] = 1000.0;
    assert!((em(&g, i) - 4500.0).abs() < 1e-3);
}

#[test]
fn o_recorte_devolve_o_factor_e_a_janela_sem_campo_volta_a_um() {
    let mut g = grade();
    let i = g.s * 4 + 4;
    let antes = snapshot_grid_region(&g, 2, 2, 8, 8).expect("janela");
    g.sett[i] = 3000.0;
    fixa(&mut g, i, 6000.0);
    let depois = snapshot_grid_region(&g, 2, 2, 8, 8).expect("janela");
    restore_grid_region(&mut g, &antes);
    assert!((factor(&g, i) - 1.0).abs() < 1e-9);
    restore_grid_region(&mut g, &depois);
    assert!((factor(&g, i) - 2.0).abs() < 1e-6);
}

/// A faca que nivela baixa a tinta em pé abaixo da massa (factor < 1) sem tocar na massa; na célula sem tinta, nada.
#[test]
fn nivelar_desce_abaixo_da_massa_sem_mudar_a_massa() {
    let mut g = grade();
    let i = g.s * 4 + 4;
    g.sett[i] = 3000.0;
    nivela(&mut g, i, 1500.0);
    assert!((em(&g, i) - 1500.0).abs() < 1e-3);
    assert_eq!(g.sett[i], 3000.0);
    let vazia = g.s * 6 + 6;
    nivela(&mut g, vazia, 800.0);
    assert!((factor(&g, vazia) - 1.0).abs() < 1e-12);
    // Ao factor 1 o campo não se aloca.
    let mut h = grade();
    h.sett[i] = 100.0;
    nivela(&mut h, i, 100.0);
    assert!(h.espessura.is_empty());
}

/// ⛔ **O vale nivelado SOMA a tinta que lhe chega** (BUG do smoke 27/10): a célula quase sem massa (o vale do grão)
/// nivelada à média dos vizinhos fica com a altura deles; a massa que lhe chega depois (o assentar) SOMA-SE à
/// espessura — como factor multiplicava-se (`2000 ÷ 2 × 102 = 102 000`), e a média do dab seguinte ia a ∞. Na tinta
/// que cobre, a lei de sempre: a massa que sai leva a espessura na mesma proporção.
#[test]
fn o_vale_nivelado_soma_a_tinta_que_lhe_chega() {
    let mut g = grade();
    let vale = g.s * 4 + 4;
    g.sett[vale] = 2.0;
    nivela(&mut g, vale, 2000.0);
    assert!((em(&g, vale) - 2000.0).abs() < 1e-3);
    g.susp[vale] = 100.0;
    assert!(
        (em(&g, vale) - 2100.0).abs() < 1e-3,
        "o vale: {}",
        em(&g, vale)
    );
    assert!(factor(&g, vale) < 21.0);
    // A tinta que cobre: o factor de sempre (a metade da massa leva a metade da espessura).
    let cheia = g.s * 5 + 5;
    g.sett[cheia] = 3000.0;
    fixa(&mut g, cheia, 6000.0);
    assert!((factor(&g, cheia) - 2.0).abs() < 1e-6);
    g.sett[cheia] = 1500.0;
    assert!((em(&g, cheia) - 3000.0).abs() < 1e-3);
    // Sem massa, sem espessura.
    g.sett[vale] = 0.0;
    g.susp[vale] = 0.0;
    assert_eq!(em(&g, vale), 0.0);
}
