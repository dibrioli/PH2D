//! ⭐ **O material da tinta por célula** (Óleo Real W2 E3): o pincel pousa o Principled, ele mistura-se no molhado com o
//! pigmento pela massa, o Lock preserva o pousado, o undo do motor devolve-o, e sem pincel com material o canal não existe.

use ph2d_wet_paint::painter::Engine;
use ph2d_wet_paint::tuning::Knob;

const W: usize = 96;
const H: usize = 96;

fn traco(e: &mut Engine, y: f64, cor: [f64; 3]) {
    e.color = cor;
    e.begin_direct_stroke(0, 20.0, y);
    let mut x = 20.0;
    while x < 76.0 {
        x += 4.0;
        e.direct_segment(0, 4.0);
        e.dispatch_pressure_dab_lane(0, x, y, 5.0, 10.0, None, None, None);
    }
    e.end_direct_stroke();
}

fn metal_no_pincel(e: &mut Engine, metal: f64) {
    e.set_knob(Knob::Metallic, metal);
    e.set_knob(Knob::Roughness, if metal > 0.0 { 0.2 } else { 0.5 });
}

/// As células com tinta na faixa `y0..=y1` (coordenadas da grade): o índice de cada.
fn com_tinta(e: &Engine, y0: usize, y1: usize) -> Vec<usize> {
    let g = e.active_grid();
    (0..g.cells)
        .filter(|&k| (y0..=y1).contains(&(k / g.s)) && g.susp[k] + g.sett[k] > 0.0)
        .collect()
}

fn metal(e: &Engine, k: usize) -> f32 {
    e.active_grid().material_da_celula(k).metal
}

#[test]
fn sem_pincel_com_material_o_canal_fica_vazio() {
    let mut e = Engine::new(W, H);
    traco(&mut e, 48.0, [220.0, 45.0, 40.0]);
    assert!(e.active_grid().material.is_empty());
}

/// ⭐ Metal ao lado e por cima de tinta comum, no molhado: no traço só de metal, metal 1 em toda a tinta; onde o metal
/// cruza a comum molhada, a mistura fica ENTRE as duas (pela massa).
#[test]
fn metal_ao_lado_de_tinta_comum_mistura_no_molhado() {
    let mut e = Engine::new(W, H);
    traco(&mut e, 48.0, [220.0, 45.0, 40.0]);
    metal_no_pincel(&mut e, 1.0);
    traco(&mut e, 48.0, [200.0, 170.0, 60.0]);
    traco(&mut e, 20.0, [200.0, 170.0, 60.0]);
    let so_metal = com_tinta(&e, 8, 32);
    assert!(!so_metal.is_empty());
    for &k in &so_metal {
        assert!(
            (metal(&e, k) - 1.0).abs() < 1e-6,
            "célula {k}: {}",
            metal(&e, k)
        );
    }
    let cruzadas = com_tinta(&e, 36, 60);
    let mistas = cruzadas
        .iter()
        .filter(|&&k| (0.05..0.95).contains(&metal(&e, k)))
        .count();
    println!(
        "[material] só metal {} células · cruzadas {} · mistas {mistas}",
        so_metal.len(),
        cruzadas.len()
    );
    assert!(mistas > 0, "nenhuma célula com a mistura das duas tintas");
}

/// ⭐ Lock material: um traço comum por cima do metal muda a cor, não o metal; sem o Lock o metal dilui-se.
#[test]
fn o_lock_material_preserva_o_metal_pousado() {
    let pinta = |trava: f64| {
        let mut e = Engine::new(W, H);
        metal_no_pincel(&mut e, 1.0);
        traco(&mut e, 48.0, [200.0, 170.0, 60.0]);
        let antes: Vec<usize> = com_tinta(&e, 36, 60);
        metal_no_pincel(&mut e, 0.0);
        e.set_knob(Knob::LockMaterial, trava);
        traco(&mut e, 48.0, [30.0, 40.0, 200.0]);
        let min = antes.iter().map(|&k| metal(&e, k)).fold(1.0f32, f32::min);
        let azul: f32 = antes
            .iter()
            .map(|&k| e.active_grid().susp_rgb[k][2] - e.active_grid().susp_rgb[k][0])
            .sum();
        (min, azul)
    };
    let (com, azul) = pinta(1.0);
    let (sem, _) = pinta(0.0);
    assert!(
        (com - 1.0).abs() < 1e-6,
        "o Lock deixou o metal mudar: {com}"
    );
    assert!(sem < 0.95, "sem o Lock o metal não se misturou: {sem}");
    assert!(
        azul > 0.0,
        "com o Lock o pigmento devia mudar (azul − vermelho {azul})"
    );
}

/// ⭐ O undo do motor devolve o canal (e a ausência dele).
#[test]
fn o_material_viaja_no_undo_do_motor() {
    let mut e = Engine::new(W, H);
    e.capture_history();
    metal_no_pincel(&mut e, 1.0);
    traco(&mut e, 48.0, [200.0, 170.0, 60.0]);
    let i = com_tinta(&e, 36, 60)[0];
    let depois = e.active_grid().material_da_celula(i);
    assert!(depois.metal > 0.99);
    assert!(e.undo());
    assert!(
        e.active_grid().material.is_empty(),
        "o desfazer não tirou o canal"
    );
    assert!(e.redo());
    assert_eq!(e.active_grid().material_da_celula(i), depois);
}

/// ⭐ A ponta que pega tinta leva o material: um pincel comum arrastado do metal molhado para o papel limpo pousa lá
/// tinta com metal (a sujidade da ponta).
#[test]
fn a_ponta_suja_leva_o_metal() {
    let mut e = Engine::new(W, H);
    metal_no_pincel(&mut e, 1.0);
    traco(&mut e, 48.0, [200.0, 170.0, 60.0]);
    metal_no_pincel(&mut e, 0.0);
    e.set_knob(Knob::Pickup, 1.0);
    e.color = [30.0, 40.0, 200.0];
    e.begin_direct_stroke(0, 48.0, 30.0);
    let mut y = 30.0;
    while y < 84.0 {
        y += 4.0;
        e.direct_segment(0, 4.0);
        e.dispatch_pressure_dab_lane(0, 48.0, y, 5.0, 10.0, None, None, None);
    }
    e.end_direct_stroke();
    let alem: Vec<usize> = com_tinta(&e, 62, 90);
    let com_metal = alem.iter().filter(|&&k| metal(&e, k) > 0.0).count();
    println!(
        "[material] além da faixa: {} células com tinta, {com_metal} com metal",
        alem.len()
    );
    assert!(com_metal > 0, "a ponta não levou o metal para lá da faixa");
}
