//! Os portões da lâmina e do rebordo ([`super`]).

use super::{FAIXA, perfil};
use crate::painter::Engine;

/// Uma lâmina rectangular de meia-largura `l` ATRAVÉS de `rumo` e meia-espessura `t` AO LONGO dele (a que gira
/// com o traço), centrada em `c`: cobertura 1 dentro, 0 fora.
fn lamina(c: [f64; 2], rumo: [f64; 2], l: f64, t: f64) -> impl Fn(i32, i32) -> f64 + Sync {
    move |x, y| {
        let d = [f64::from(x) - c[0], f64::from(y) - c[1]];
        let ao_longo = d[0] * rumo[0] + d[1] * rumo[1];
        let atraves = -d[0] * rumo[1] + d[1] * rumo[0];
        f64::from(u8::from(atraves.abs() <= l && ao_longo.abs() <= t))
    }
}

/// Arrasta a lâmina de `a` a `b` em passos de 1 célula, com o rebordo `h`.
fn arrasta(e: &mut Engine, a: [f64; 2], b: [f64; 2], h: f64) {
    let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
    let n = dx.hypot(dy);
    let rumo = [dx / n, dy / n];
    for k in 0..=n as usize {
        let c = [a[0] + rumo[0] * k as f64, a[1] + rumo[1] * k as f64];
        let sil = lamina(c, rumo, 12.0, 2.0);
        e.pentear_rebordo(c[0], c[1], [14.0, 14.0], &sil, rumo, h, 1.0);
    }
}

fn altura(e: &Engine, x: i32, y: i32) -> f32 {
    let g = e.active_grid();
    crate::grid::altura::em(g, x as usize + y as usize * g.s)
}

#[test]
fn o_perfil_e_zero_no_miolo_e_sobe_na_faixa_ate_a_borda() {
    assert_eq!(perfil(0.0), 0.0);
    assert_eq!(perfil(1.0 - FAIXA), 0.0);
    assert!((perfil(1.0 - FAIXA / 2.0) - 0.5).abs() < 1e-12);
    assert_eq!(perfil(1.0), 1.0);
    assert_eq!(perfil(-1.0), 1.0);
}

/// ⭐ **A lâmina que gira deixa o rebordo nas duas bordas dos DOIS braços de um traço em L** (horizontal e depois
/// vertical): a crista nas bordas laterais, o miolo liso, e o mesmo perfil nos dois braços — a meia-largura sai da
/// silhueta ATRAVÉS do rumo, então o braço vertical tem o rebordo à esquerda e à direita, não em cima e em baixo.
#[test]
fn a_lamina_em_l_deixa_o_rebordo_nas_bordas_dos_dois_bracos() {
    let mut e = Engine::new(128, 128);
    let h = 2.0;
    arrasta(&mut e, [20.0, 30.0], [90.0, 30.0], h);
    arrasta(&mut e, [90.0, 30.0], [90.0, 110.0], h);
    // Braço horizontal, a meio (x = 50): a crista nas linhas 30 ± 12, o miolo em 30.
    let (borda_h, miolo_h) = (
        altura(&e, 50, 18).min(altura(&e, 50, 42)),
        altura(&e, 50, 30),
    );
    // Braço vertical, a meio (y = 75): a crista nas colunas 90 ± 12.
    let (borda_v, miolo_v) = (
        altura(&e, 78, 75).min(altura(&e, 102, 75)),
        altura(&e, 90, 75),
    );
    assert!(
        borda_h > 0.5 * h as f32,
        "crista do braço horizontal {borda_h}"
    );
    assert!(
        borda_v > 0.5 * h as f32,
        "crista do braço vertical {borda_v}"
    );
    assert!(
        miolo_h.abs() < 1e-6 && miolo_v.abs() < 1e-6,
        "miolo {miolo_h} {miolo_v}"
    );
    assert!(
        (borda_h - borda_v).abs() < 0.05 * h as f32,
        "braços diferentes: {borda_h} × {borda_v}"
    );
    // No braço vertical as células de cima/baixo do eixo (ao longo do traço) não são borda: lisas.
    assert!(altura(&e, 90, 70).abs() < 1e-6);
    // Fora da lâmina nada se escreveu.
    assert_eq!(altura(&e, 50, 50), 0.0);
}

/// Sem rumo (o 1.º dab antes de o traço ter direcção) a porta não escreve nada nem aloca o campo.
#[test]
fn sem_rumo_o_rebordo_nao_toca_a_grade() {
    let mut e = Engine::new(64, 64);
    let sil = lamina([32.0, 32.0], [1.0, 0.0], 10.0, 2.0);
    e.pentear_rebordo(32.0, 32.0, [12.0, 12.0], &sil, [0.0, 0.0], 2.0, 1.0);
    assert!(e.active_grid().altura.is_empty());
}
