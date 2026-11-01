//! E10c: o lado espectral ligado às portas do motor (`docs/Painter/Arquitetura/E10b_pigment_mix.md` §6).

use super::super::{garante as garante_tinta, soma_da_massa};
use super::*;
use crate::solver::tinta_transporte::{gather_linear, junta_cantos, move_filme};

const AZUL: [f32; 3] = [40.0, 90.0, 220.0];
const AMARELO: [f32; 3] = [245.0, 205.0, 25.0];

/// Uma grade com a lei nova e duas células pintadas (o depósito da E6a no filme: soma + `Espectro` pelo `p`).
fn duas_tintas(p: f32) -> (Grid, usize, usize) {
    let mut g = Grid::new(4, 1);
    let _ = garante_tinta(&mut g);
    let (a, b) = (g.s + 2, g.s + 3);
    for (i, cor) in [(a, AZUL), (b, AMARELO)] {
        garante_para(&mut g, p);
        g.susp[i] = 600.0;
        g.tinta[i].filme = soma_da_massa(cor, 0.0, 0.0, 600.0);
        if existe(&g) {
            pousa(&mut g, (i, FILME), de_massa(cor, 0.0, 600.0, p), true);
        }
    }
    (g, a, b)
}

/// Jacobi (o gather linear do escoamento): `b ← ½ b + ½ a`, `a ← ½ a + ½ b`, lidos do início.
fn mistura_jacobi(g: &mut Grid, a: usize, b: usize) {
    gather_linear(g, [a, b].into_iter(), |_, i, t| {
        t.push((a, 0.5));
        t.push((b, 0.5));
        let _ = i;
    });
}

fn ecra(g: &Grid, i: usize) -> [f64; 3] {
    crate::render::tinta::ecra_da_celula(g, i).expect("tinta").0
}

/// (b) Azul + amarelo misturados no motor: `p = 1` dá VERDE no ecrã; `p = 0` o cinza Oklab de hoje (sem lado).
#[test]
fn pigment_mix_motor_azul_e_amarelo_dao_verde() {
    let (mut g1, a, b) = duas_tintas(1.0);
    mistura_jacobi(&mut g1, a, b);
    let (mut g0, _, _) = duas_tintas(0.0);
    mistura_jacobi(&mut g0, a, b);
    assert!(g0.pigmento.is_empty(), "p = 0 não aloca o lado");
    let (v, c) = (ecra(&g1, b), ecra(&g0, b));
    eprintln!("p=1 ecrã {v:?} · p=0 ecrã {c:?}");
    assert!(v[1] > v[0] && v[1] > v[2], "p=1 não é verde: {v:?}");
    assert!(!(c[1] > c[0] && c[1] > c[2]), "p=0 já era verde: {c:?}");
    // O espelho (`srgb_0_255`) da fatia diz o mesmo.
    let [Some(f), _] = espelho(&g1, b) else {
        panic!("sem filme")
    };
    eprintln!("p=1 espelho {f:?}");
    assert!(f[1] > f[0] && f[1] > f[2]);
}

/// (a) Com o lado vivo e `p = 0` em toda a tinta, o ecrã é o de hoje AO BIT depois das portas.
#[test]
fn pigment_mix_motor_lado_p0_igual_ao_bit() {
    let (mut g, a, b) = duas_tintas(0.0);
    let mut h = g.clone();
    let _ = garante(&mut h.pigmento, &h.tinta);
    for g in [&mut g, &mut h] {
        mistura_jacobi(g, a, b);
        move_filme(g, a, b, 0.3);
    }
    for i in [a, b] {
        assert_eq!(ecra(&g, i).map(f64::to_bits), ecra(&h, i).map(f64::to_bits));
    }
}

/// (c) Conservação: Σ do `Espectro` sobre a grade antes/depois de N passos de escoamento sem depósito.
#[test]
fn pigment_mix_motor_conserva_o_espectro() {
    let mut g = Grid::new(8, 3);
    let _ = garante_tinta(&mut g);
    garante_para(&mut g, 1.0);
    let cores = [AZUL, AMARELO, [200.0, 30.0, 60.0], [10.0, 10.0, 10.0]];
    let celulas: Vec<usize> = (2..9).map(|x| 2 * g.s + x).collect();
    for (k, &i) in celulas.iter().enumerate() {
        let cor = cores[k % cores.len()];
        let m = 200.0 + 100.0 * k as f32;
        g.tinta[i].filme = soma_da_massa(cor, 0.2, 0.0, m);
        g.tinta[i].aberto = soma_da_massa(cor, 0.2, 0.0, m * 0.5);
        g.pigmento[i] = [de_massa(cor, 0.2, m, 1.0), de_massa(cor, 0.2, m * 0.5, 0.7)];
    }
    let antes = soma_total(&g);
    for passo in 0..50 {
        for w in celulas.windows(2) {
            let (de, para) = if passo % 2 == 0 {
                (w[0], w[1])
            } else {
                (w[1], w[0])
            };
            move_filme(&mut g, de, para, 0.17);
        }
        let n = celulas.len();
        let cantos = [celulas[0], celulas[1], celulas[n - 2], celulas[n - 1]];
        junta_cantos(&mut g, celulas[n / 2], cantos, [0.1, 0.05, 0.08, 0.02]);
        let i = celulas[passo % n];
        let (c, l) = (&mut g.tinta[i], g.pigmento.get_mut(i));
        crate::drying::tinta::assenta(c, l, 0.11, &mut [0.0; 3]);
    }
    let depois = soma_total(&g);
    let pior = antes
        .iter()
        .zip(&depois)
        .map(|(a, d)| (a - d).abs() / a.abs().max(1e-9))
        .fold(0.0f64, f64::max);
    eprintln!("conservação: pior relativa {pior:.3e}");
    assert!(pior <= 1e-5, "o Espectro não se conservou: {pior:e}");
}

/// (d) `bytes_retidos` só cresce (112 B/célula) depois do primeiro depósito com `p > 0`.
#[test]
fn pigment_mix_motor_bytes_so_depois_de_p() {
    let mut g = Grid::new(16, 8);
    let _ = garante_tinta(&mut g);
    let b0 = g.bytes_retidos();
    garante_para(&mut g, 0.0);
    assert_eq!(g.bytes_retidos(), b0, "p = 0 não paga um byte");
    garante_para(&mut g, 0.5);
    let b1 = g.bytes_retidos();
    eprintln!("bytes: {b0} → {b1} ({} células)", g.cells);
    assert_eq!(b1 - b0, 112 * g.cells);
}
