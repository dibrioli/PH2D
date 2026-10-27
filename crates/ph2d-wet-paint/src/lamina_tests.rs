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

/// ⭐ **A crista é o rebordo e SÓ ele** (W3 K3a): a faca penteia o campo próprio [`crate::grid::Grid::crista`] com o
/// mesmo valor que deixa na `altura` (numa grade sem fios, os dois campos são iguais ao bit) — e o relevo de outro
/// pincel na `altura` não entra nela.
#[test]
fn a_crista_e_o_rebordo_da_lamina_e_so_dele() {
    let mut e = Engine::new(128, 128);
    arrasta(&mut e, [20.0, 30.0], [90.0, 30.0], 2.0);
    arrasta(&mut e, [90.0, 30.0], [90.0, 110.0], 2.0);
    let g = e.active_grid();
    assert_eq!(g.crista.len(), g.altura.len());
    assert!(g.crista == g.altura, "a crista ≠ a altura da lâmina");
    assert!(crate::grid::crista::em(g, 50 + 18 * g.s) > 1.0);
    // Os fios de outro pincel (a `altura` escrita fora da lâmina) não acordam o campo da crista.
    let mut f = Engine::new(64, 64);
    let s = f.active_grid().s;
    crate::grid::altura::garante(f.active_grid_mut())[10 + 10 * s] = 1.0;
    assert!(f.active_grid().crista.is_empty());
}

/// A massa com SULCOS através do traço (linhas alternadas 1000 / 4000), uma faixa de 64 × 30 em `y` 15..45.
fn sulcos(e: &mut Engine) {
    let g = e.active_grid_mut();
    for y in 15..45 {
        for x in 1..=g.w {
            g.sett[x + y * g.s] = if y % 2 == 0 { 1000.0 } else { 4000.0 };
        }
    }
}

fn espessura(e: &Engine, x: usize, y: usize) -> f64 {
    let g = e.active_grid();
    crate::grid::espessura::em(g, x + y * g.s)
}

/// ⭐ **A faca NIVELA** (W3 K3a, `knifeLevel`): arrastada sobre tinta com sulcos, a tinta EM PÉ sob a lâmina vai para
/// um planalto (os sulcos 1000/4000 somem: dentro de 1 %), a massa (a cor) fica ao bit; a `0` nada muda nem se aloca.
#[test]
fn a_faca_nivela_a_tinta_em_pe_num_planalto_sem_mudar_a_massa() {
    let mut e = Engine::new(64, 64);
    sulcos(&mut e);
    let massa: Vec<f32> = e.active_grid().sett.clone();
    // A `0`: nem o rebordo nem o nivelamento ⇒ a grade de antes.
    arrasta(&mut e, [6.0, 30.0], [58.0, 30.0], 0.0);
    assert!(e.active_grid().espessura.is_empty() && e.active_grid().crista.is_empty());
    e.tuning.set(crate::tuning::Knob::KnifeLevel, 1.0);
    arrasta(&mut e, [6.0, 30.0], [58.0, 30.0], 0.0);
    assert_eq!(e.active_grid().sett, massa, "o nivelamento mudou a massa");
    let corte: Vec<f64> = (20..41).map(|y| espessura(&e, 32, y)).collect();
    let (lo, hi) = corte
        .iter()
        .fold((f64::INFINITY, 0.0f64), |(a, b), &v| (a.min(v), b.max(v)));
    assert!(hi / lo < 1.01, "sulcos ficaram: {corte:?}");
    // O planalto é a média da tinta que estava sob a lâmina (as linhas 18..=42: 13 × 1000 e 12 × 4000), não o topo
    // nem o fundo dos sulcos.
    let media = (13.0 * 1000.0 + 12.0 * 4000.0) / 25.0;
    assert!(
        (corte[10] / media - 1.0).abs() < 0.01,
        "nível {} × média {media}",
        corte[10]
    );
    // Fora da lâmina (y 15) a tinta fica como estava.
    assert!((espessura(&e, 32, 15) - 4000.0).abs() < 1e-3);
}
