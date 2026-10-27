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
