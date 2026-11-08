//! O ponto fixo do spray (dono 08/11, «Fixed dab size»): fora do `marca_tests.rs` pelo tecto de LOC.

use super::*;

/// ⭐ **O ponto fixo com a contagem que cresce** (dono 08/11, «Fixed dab size»): com `particle_px` e
/// `particle_ref_px`, Size 40 / 80 / 160 dão o MESMO raio de marca e a MESMA densidade (marcas por px² da faixa
/// `comprimento × diâmetro`); sem a referência a contagem é a de sempre (ao bit: o Pastel).
#[test]
fn o_ponto_fixo_mantem_o_tamanho_e_a_densidade_em_todo_size() {
    let mede = |diametro: f32, referencia: f32| {
        let mut s = com(|j| {
            j.particle_px = 26.4;
            j.particle_ref_px = referencia;
        });
        s.radius_px = 0.5 * diametro;
        s.jitter_scale = 0.0;
        s.spacing = 0.1;
        let d = recto(s, 7);
        let r: Vec<f32> = d.iter().map(|d| d.radius_px).collect();
        let lo = r.iter().copied().fold(f32::MAX, f32::min);
        let hi = r.iter().copied().fold(0.0, f32::max);
        (lo, hi, d.len() as f32 / (880.0 * diametro), d)
    };
    let (a_lo, a_hi, a_dens, _) = mede(40.0, 80.0);
    let (b_lo, b_hi, b_dens, _) = mede(80.0, 80.0);
    let (c_lo, c_hi, c_dens, _) = mede(160.0, 80.0);
    for (lo, hi) in [(a_lo, a_hi), (b_lo, b_hi), (c_lo, c_hi)] {
        assert!(
            (lo - 13.2).abs() < 1e-4 && (hi - 13.2).abs() < 1e-4,
            "raio {lo}..{hi}"
        );
    }
    // Medido (semente 7): 40 → a contagem 3·¼ arredonda a 1 — a tolerância é a desse arredondamento.
    for (nome, dens) in [("40", a_dens), ("160", c_dens)] {
        let razao = dens / b_dens;
        assert!(
            (0.75..=1.34).contains(&razao),
            "densidade a {nome} ÷ a 80 = {razao} ({dens} × {b_dens})"
        );
    }
    // A referência desligada: a contagem não cresce, o traço é o de antes ao bit.
    let (_, _, _, sem) = mede(160.0, 0.0);
    let mut s = com(|j| j.particle_px = 26.4);
    s.radius_px = 80.0;
    s.jitter_scale = 0.0;
    s.spacing = 0.1;
    assert_eq!(sem, recto(s, 7));
    let (_, _, sem_dens, _) = mede(160.0, 0.0);
    assert!(
        sem_dens < 0.5 * c_dens,
        "sem a referência a densidade cai: {sem_dens} × {c_dens}"
    );
}
