//! O campo preguiçoso do material (W2 E3): a mistura pela massa, o Lock, e o undo do motor (instantâneo e recorte).

use super::super::{
    Grid, clear_canvas, restore_grid, restore_grid_region, snapshot_grid, snapshot_grid_region,
};
use super::*;

fn metal() -> MaterialCelula {
    MaterialCelula {
        metal: 1.0,
        rugosidade: 0.25,
        ..MaterialCelula::NEUTRO
    }
}

#[test]
fn a_tinta_neutra_nao_aloca_o_campo() {
    let mut g = Grid::new(16, 12);
    pousa(&mut g, 40, 0.0, MaterialCelula::NEUTRO, 100.0, false);
    assert!(g.material.is_empty());
    assert_eq!(g.material_da_celula(40), MaterialCelula::NEUTRO);
}

/// ⭐ Metal sobre a mesma massa de tinta comum = meio metal; sobre o papel limpo, o metal inteiro.
#[test]
fn o_material_mistura_pela_massa() {
    let mut g = Grid::new(16, 12);
    pousa(&mut g, 40, 0.0, metal(), 100.0, false);
    assert_eq!(g.material_da_celula(40), metal());
    assert_eq!(g.material_da_celula(41), MaterialCelula::NEUTRO);
    pousa(&mut g, 41, 300.0, metal(), 100.0, false);
    let m = g.material_da_celula(41);
    assert!((m.metal - 0.25).abs() < 1e-6, "{m:?}");
    assert!((m.rugosidade - 0.4375).abs() < 1e-6, "{m:?}");
}

/// ⭐ Lock Reflectivity: a célula com tinta guarda o seu material; o papel limpo recebe o do pincel.
#[test]
fn o_lock_preserva_o_material_pousado() {
    let mut g = Grid::new(16, 12);
    pousa(&mut g, 40, 0.0, metal(), 100.0, false);
    pousa(&mut g, 40, 100.0, MaterialCelula::NEUTRO, 500.0, true);
    assert_eq!(g.material_da_celula(40), metal());
    pousa(&mut g, 42, 0.0, metal(), 100.0, true);
    assert_eq!(g.material_da_celula(42), metal());
}

#[test]
fn o_instantaneo_e_o_recorte_levam_o_material() {
    let mut g = Grid::new(16, 12);
    let s = g.s;
    let (dentro, fora) = (4 * s + 4, 10 * s + 12);
    let vazio = snapshot_grid(&g);
    let recorte_vazio = snapshot_grid_region(&g, 2, 2, 6, 6).unwrap();
    garante(&mut g)[dentro] = metal();
    g.material[fora] = metal();
    let cheio = snapshot_grid(&g);
    let recorte_cheio = snapshot_grid_region(&g, 2, 2, 6, 6).unwrap();
    restore_grid_region(&mut g, &recorte_vazio);
    assert_eq!(
        g.material_da_celula(dentro),
        MaterialCelula::NEUTRO,
        "a janela volta a NEUTRO"
    );
    assert_eq!(
        g.material_da_celula(fora),
        metal(),
        "fora da janela não mexe"
    );
    restore_grid_region(&mut g, &recorte_cheio);
    assert_eq!(
        g.material_da_celula(dentro),
        metal(),
        "o refazer devolve o material"
    );
    restore_grid(&mut g, &vazio);
    assert!(
        g.material.is_empty(),
        "o campo que não existia volta a não existir"
    );
    restore_grid(&mut g, &cheio);
    assert_eq!(g.material_da_celula(fora), metal());
    clear_canvas(&mut g);
    assert!(g.material.is_empty(), "limpar a folha devolve o campo");
}

/// ⭐ Os defaults dos knobs do material SÃO o NEUTRO (os presets que não os trazem pintam a tinta de sempre e o canal
/// fica vazio), e o Lock nasce desligado.
#[test]
fn o_neutro_do_material_e_o_default_dos_knobs() {
    use crate::tuning::{KNOB_DEFS, Knob};
    let d = |k: Knob| KNOB_DEFS[k as usize].default as f32;
    let n = MaterialCelula::NEUTRO;
    assert_eq!(
        [
            d(Knob::Metallic),
            d(Knob::Roughness),
            d(Knob::IorLevel),
            d(Knob::Coat),
            d(Knob::CoatRoughness),
            d(Knob::Sheen),
            d(Knob::Subsurface),
            d(Knob::SubsurfaceRadius)
        ],
        [
            n.metal,
            n.rugosidade,
            n.nivel_ior,
            n.verniz,
            n.verniz_rugosidade,
            n.veludo,
            n.subsuperficie,
            n.raio_subsuperficie
        ]
    );
    assert_eq!(d(Knob::LockMaterial), 0.0);
}
