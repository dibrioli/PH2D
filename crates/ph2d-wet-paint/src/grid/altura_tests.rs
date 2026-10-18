//! O campo preguiçoso da altura (W3) no undo do motor: o instantâneo inteiro e o recorte.

use super::super::{
    Grid, clear_canvas, restore_grid, restore_grid_region, snapshot_grid, snapshot_grid_region,
};
use super::*;

fn grade() -> Grid {
    Grid::new(16, 12)
}

#[test]
fn o_instantaneo_leva_o_campo_e_a_ausencia_dele() {
    let mut g = grade();
    let vazio = snapshot_grid(&g);
    let s = g.s;
    garante(&mut g)[3 * s + 4] = 1.5;
    let cheio = snapshot_grid(&g);
    restore_grid(&mut g, &vazio);
    assert!(
        g.altura.is_empty(),
        "o campo que não existia volta a não existir"
    );
    restore_grid(&mut g, &cheio);
    assert_eq!(em(&g, 3 * s + 4), 1.5);
}

#[test]
fn o_recorte_tirado_sem_o_campo_zera_a_janela_e_o_resto_fica() {
    let mut g = grade();
    let antes = snapshot_grid_region(&g, 2, 2, 6, 6).unwrap();
    let s = g.s;
    let campo = garante(&mut g);
    campo[4 * s + 4] = 2.0; // dentro da janela
    campo[10 * s + 12] = -1.0; // fora dela
    let depois = snapshot_grid_region(&g, 2, 2, 6, 6).unwrap();
    restore_grid_region(&mut g, &antes);
    assert_eq!(em(&g, 4 * s + 4), 0.0, "a janela volta a zero");
    assert_eq!(em(&g, 10 * s + 12), -1.0, "fora da janela não mexe");
    restore_grid_region(&mut g, &depois);
    assert_eq!(em(&g, 4 * s + 4), 2.0, "o refazer devolve a altura");
}

#[test]
fn sem_o_campo_o_recorte_nao_o_aloca() {
    let mut g = grade();
    let p = snapshot_grid_region(&g, 1, 1, 5, 5).unwrap();
    restore_grid_region(&mut g, &p);
    assert!(g.altura.is_empty());
    let s = g.s;
    garante(&mut g)[2 * s + 2] = 1.0;
    clear_canvas(&mut g);
    assert!(g.altura.is_empty(), "limpar a folha devolve o campo");
}
