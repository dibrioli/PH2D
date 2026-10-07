//! A lei do MEIO-ÂNGULO ([`MisturaDoAngulo::MeioAngulo`], a do produto desde 2026-10-06): a fórmula,
//! a taxa onde os pesos se igualam, e a volta dos meios escolhida pela ÁRVORE DAS JUNTAS.

use super::MisturaDoAngulo;
use crate::{Skin, SkinBone, Xform};
use std::f64::consts::PI;

/// Um osso de `a` a `b` em repouso cuja pose roda `ang` (absoluto) e leva a ponta `a` a `a2`.
fn osso(a: [f64; 2], b: [f64; 2], ang: f64, a2: [f64; 2]) -> SkinBone {
    let (s, c) = ang.sin_cos();
    SkinBone {
        rest_a: a,
        rest_b: b,
        radius: 10.0,
        pose: Xform([
            c,
            s,
            -s,
            c,
            a2[0] - c.mul_add(a[0], -(s * a[1])),
            a2[1] - s.mul_add(a[0], c * a[1]),
        ]),
        sub: (0, 1),
        tendon: 0,
    }
}

/// Pai de `(−2, 0)` a `(0, 0)` parado, filho de `(0, 0)` a `(2, 0)` rodado `g` em torno da junta.
fn par(g: f64) -> Skin {
    Skin::new(vec![
        osso([-2.0, 0.0], [0.0, 0.0], 0.0, [-2.0, 0.0]),
        osso([0.0, 0.0], [2.0, 0.0], g, [0.0, 0.0]),
    ])
    .expect("pele")
}

/// O ângulo com que um ponto de pesos `w` roda em torno da junta `j` (posta em `j2`).
fn rodou(k: &Skin, w: &[f64], lei: MisturaDoAngulo, j: [f64; 2], j2: [f64; 2]) -> f64 {
    let q = k.blend_com([j[0] + 1.0, j[1]], w, lei);
    (q[1] - j2[1]).atan2(q[0] - j2[0])
}

/// ⭐⭐⭐ **GATE — a fórmula:** num par com dobra `g`, `θ̄ = 2·atan2(w·sin(g/2), (1−w) + w·cos(g/2))`
/// em torno da junta; e a TAXA em `w = ½` é `4·tan(g/4)` contra `2·tan(g/2)` da média em círculo —
/// o que fecha a tampa de fora numa dobra forte (`7,7×` a taxa linear a `170°` no círculo, `1,24×`
/// no meio-ângulo; report do dono de 2026-10-06).
#[test]
fn o_meio_angulo_e_o_dobro_do_argumento_da_media_dos_meios() {
    for gd in [30.0_f64, 90.0, 140.0, 170.0, 175.0] {
        let g = gd.to_radians();
        let k = par(g);
        for i in 0..=20 {
            let w = f64::from(i) / 20.0;
            let esperado = 2.0 * (w * (g / 2.0).sin()).atan2((1.0 - w) + w * (g / 2.0).cos());
            let t = rodou(
                &k,
                &[1.0 - w, w],
                MisturaDoAngulo::MeioAngulo,
                [0.0; 2],
                [0.0; 2],
            );
            assert!(
                (t - esperado).abs() < 1e-12,
                "g {gd}° w {w}: {t} contra {esperado}"
            );
        }
        let taxa = |lei| {
            let h = 1e-6;
            let a = rodou(&k, &[0.5 + h, 0.5 - h], lei, [0.0; 2], [0.0; 2]);
            let b = rodou(&k, &[0.5 - h, 0.5 + h], lei, [0.0; 2], [0.0; 2]);
            (b - a) / (2.0 * h)
        };
        let (m, c) = (
            taxa(MisturaDoAngulo::MeioAngulo),
            taxa(MisturaDoAngulo::Circulo),
        );
        assert!(
            (m - 4.0 * (g / 4.0).tan()).abs() < 1e-5,
            "g {gd}°: taxa {m}"
        );
        assert!(
            (c - 2.0 * (g / 2.0).tan()).abs() < 1e-4,
            "g {gd}°: taxa do círculo {c}"
        );
        if gd >= 170.0 {
            assert!(c > 6.0 * m, "g {gd}°: o círculo {c} contra o meio {m}");
        }
    }
}

/// ⭐⭐ **GATE — a volta dos meios é a da JUNTA, não a do `atan2`.** Pai a `+170°`, filho a `−170°`
/// (`+190°`: uma dobra REAL de `20°`). Os meios crus (`85°` e `−85°`) estão a `170°` um do outro e a
/// mistura iria pelo lado de fora (`0°`); desdobrados pela junta, a meio caminho roda `180°`.
#[test]
fn a_volta_dos_meios_e_a_da_junta_e_nao_a_do_atan2() {
    let j2 = {
        let (s, c) = (170.0_f64.to_radians()).sin_cos();
        [-2.0 + 2.0 * c, 2.0 * s]
    };
    let k = Skin::new(vec![
        osso([-2.0, 0.0], [0.0, 0.0], 170.0_f64.to_radians(), [-2.0, 0.0]),
        osso([0.0, 0.0], [2.0, 0.0], (-170.0_f64).to_radians(), j2),
    ])
    .expect("pele");
    let t = rodou(&k, &[0.5, 0.5], MisturaDoAngulo::MeioAngulo, [0.0; 2], j2);
    assert!((t.abs() - PI).abs() < 1e-9, "rodou {}°", t.to_degrees());
}

/// ⭐⭐ **GATE — numa RAMIFICAÇÃO a volta vem do vizinho da junta, não do anterior da LISTA.** Na
/// lista `[A (0°), X (170°, outro ramo, longe), C (filho de A, −100°)]` a ordem da lista desdobraria
/// `C` para `260°` (perto de `X`) e a mistura de `A` com `C` iria pelo lado de fora (`130°`); pela
/// árvore das juntas `C` desdobra contra `A` e a meio caminho roda `−50°`.
#[test]
fn numa_ramificacao_a_volta_vem_do_vizinho_da_junta() {
    let k = Skin::new(vec![
        osso([-2.0, 0.0], [0.0, 0.0], 0.0, [-2.0, 0.0]),
        osso(
            [10.0, 10.0],
            [12.0, 10.0],
            170.0_f64.to_radians(),
            [10.0, 10.0],
        ),
        osso(
            [0.0, 0.0],
            [2.0, 0.0],
            (-100.0_f64).to_radians(),
            [0.0, 0.0],
        ),
    ])
    .expect("pele");
    let t = rodou(
        &k,
        &[0.5, 0.0, 0.5],
        MisturaDoAngulo::MeioAngulo,
        [0.0; 2],
        [0.0; 2],
    );
    assert!(
        (t.to_degrees() + 50.0).abs() < 1e-9,
        "rodou {}°",
        t.to_degrees()
    );
    // ⭐ E dois IRMÃOS na mesma junta (`C` a `−100°`, `D` a `+100°`): `D` desdobra contra o PAI
    // (`A`), não contra o irmão que entrou antes dele na árvore (que o levaria a `−260°`).
    let k2 = Skin::new(vec![
        osso([-2.0, 0.0], [0.0, 0.0], 0.0, [-2.0, 0.0]),
        osso(
            [0.0, 0.0],
            [2.0, 0.0],
            (-100.0_f64).to_radians(),
            [0.0, 0.0],
        ),
        osso([0.0, 0.0], [0.0, 2.0], 100.0_f64.to_radians(), [0.0, 0.0]),
    ])
    .expect("pele");
    let t2 = rodou(
        &k2,
        &[0.5, 0.0, 0.5],
        MisturaDoAngulo::MeioAngulo,
        [0.0; 2],
        [0.0; 2],
    );
    assert!(
        (t2.to_degrees() - 50.0).abs() < 1e-9,
        "irmão: rodou {}°",
        t2.to_degrees()
    );
    // ⛔ O CONTROLO: a ordem da lista (a da desdobrada) leva C para 260°.
    let d = rodou(
        &k,
        &[0.5, 0.0, 0.5],
        MisturaDoAngulo::Desdobrado,
        [0.0; 2],
        [0.0; 2],
    );
    assert!(
        (d.to_degrees() - 130.0).abs() < 1e-9,
        "desdobrada {}°",
        d.to_degrees()
    );
}

/// ⭐⭐ **GATE — a volta dos meios no ESPELHO** (pai a `−170°`, filho a `+170°`): desdobrado pela
/// junta o filho vai a `−190°` (a janela ALTA, `t − base > π`), e a meio caminho roda `180°`. ⛔ Sem
/// ela os meios crus (`−85°`, `85°`) dariam `0°` — o irmão de cima só exercita a janela BAIXA.
#[test]
fn a_volta_dos_meios_no_espelho_usa_a_janela_alta() {
    let j2 = {
        let (s, c) = ((-170.0_f64).to_radians()).sin_cos();
        [-2.0 + 2.0 * c, 2.0 * s]
    };
    let k = Skin::new(vec![
        osso(
            [-2.0, 0.0],
            [0.0, 0.0],
            (-170.0_f64).to_radians(),
            [-2.0, 0.0],
        ),
        osso([0.0, 0.0], [2.0, 0.0], 170.0_f64.to_radians(), j2),
    ])
    .expect("pele");
    let t = rodou(&k, &[0.5, 0.5], MisturaDoAngulo::MeioAngulo, [0.0; 2], j2);
    assert!((t.abs() - PI).abs() < 1e-9, "rodou {}°", t.to_degrees());
}

/// ⭐⭐ **GATE — a árvore das juntas é a MÍNIMA.** Cadeia `A (0°) → B (120°) → C (240°)`, juntas só
/// em `A–B` e `B–C` (`A` e `C` a `2` de distância). Pela árvore mínima `C` desdobra contra `B`
/// (`240°`) e o meio de `B` e `C` roda `180°`; ⛔ uma árvore pela MAIOR distância ligava `C` a `A`
/// (`−120°`) e o mesmo ponto rodava `0°`.
#[test]
fn a_arvore_das_juntas_e_a_minima() {
    let (s, c) = (120.0_f64.to_radians()).sin_cos();
    let j2 = [2.0 * c, 2.0 * s];
    let k = Skin::new(vec![
        osso([-2.0, 0.0], [0.0, 0.0], 0.0, [-2.0, 0.0]),
        osso([0.0, 0.0], [2.0, 0.0], 120.0_f64.to_radians(), [0.0, 0.0]),
        osso([2.0, 0.0], [4.0, 0.0], 240.0_f64.to_radians(), j2),
    ])
    .expect("pele");
    let t = rodou(
        &k,
        &[0.0, 0.5, 0.5],
        MisturaDoAngulo::MeioAngulo,
        [2.0, 0.0],
        j2,
    );
    assert!((t.abs() - PI).abs() < 1e-9, "rodou {}°", t.to_degrees());
}
