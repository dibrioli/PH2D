//! **Oráculo CORRIDO** (CLAUDE.md §0.9): o ImageMagick 7.1.2-32 compôs a nossa grelha de 7 cores
//! (49 pares fundo × fonte, opacos) nos 20 modos; `tests/oraculo/gera_fixture_imagemagick.sh` regera.
//! A lei é a do W3C *Compositing and Blending Level 1* — onde o IM diverge, o gate PREGA a
//! divergência medida (nunca muda a nossa lei para casar com ele).
//!
//! Medido em 2026-10-11 (|Δ| máximo sobre os 49 pares):
//! - concordam a ≤ 1,2e-7 (ruído f32): Normal, Multiply, Darken, LinearBurn, Lighten, Screen, Add,
//!   Overlay, HardLight, LinearLight, Difference, Exclusion;
//! - ColorBurn · ColorDodge · VividLight concordam fora do canto 0/0 (W3C: `Cb=1 ⇒ 1` no burn,
//!   `Cb=0 ⇒ 0` no dodge; o IM dá a vitória à fonte) — 1 · 1 · 2 pares, Δ = 1;
//! - SoftLight: o IM devolve `min(1, W3C + Cb)` em todos os 49 pares (o fundo somado mais uma vez);
//! - Hue · Saturate · Colorize · Luminize: o IM usa HCL com corte por canal, o W3C usa
//!   `SetLum`/`SetSat`/`ClipColor` sobre luma 0,3/0,59/0,11 — Δ até 0,58.

use ph2d_blend_mode::{BlendMode, apply};

const FIXTURE: &str = include_str!("../oraculo/imagemagick_compose.txt");

/// Ruído medido entre o nosso `f32` e o PFM float32 do IM: 1,2e-7.
const TOL: f32 = 1e-6;

/// Código do `BlendMode` → operador do IM (o mesmo mapa do script).
const MAPA: [&str; 20] = [
    "Over",
    "Multiply",
    "Darken",
    "ColorBurn",
    "LinearBurn",
    "Lighten",
    "Screen",
    "ColorDodge",
    "LinearDodge",
    "Overlay",
    "SoftLight",
    "HardLight",
    "VividLight",
    "LinearLight",
    "Difference",
    "Exclusion",
    "Hue",
    "Saturate",
    "Colorize",
    "Luminize",
];

const CONCORDAM: [u8; 12] = [0, 1, 2, 4, 5, 6, 8, 9, 11, 13, 14, 15];

struct Linha {
    nome: String,
    codigo: u8,
    ib: usize,
    is: usize,
    fundo: [f32; 3],
    fonte: [f32; 3],
    im: [f32; 3],
}

fn le() -> (Vec<[f32; 3]>, Vec<Linha>) {
    let mut cores = Vec::new();
    let mut linhas = Vec::new();
    for l in FIXTURE.lines().filter(|l| !l.starts_with('#')) {
        let t: Vec<&str> = l.split_whitespace().collect();
        let f = |s: &str| s.parse::<f32>().expect("número da fixture");
        if t[0] == "cor" {
            assert_eq!(t[1].parse::<usize>().unwrap(), cores.len());
            cores.push([f(t[2]), f(t[3]), f(t[4])]);
            continue;
        }
        let (ib, is) = (t[2].parse().unwrap(), t[3].parse().unwrap());
        linhas.push(Linha {
            nome: t[0].to_owned(),
            codigo: t[1].parse().unwrap(),
            ib,
            is,
            fundo: cores[ib],
            fonte: cores[is],
            im: [f(t[4]), f(t[5]), f(t[6])],
        });
    }
    (cores, linhas)
}

fn nosso(codigo: u8, fundo: [f32; 3], fonte: [f32; 3]) -> [f32; 3] {
    let o = apply(
        BlendMode::from_u8(codigo),
        [fundo[0], fundo[1], fundo[2], 1.0],
        [fonte[0], fonte[1], fonte[2], 1.0],
    );
    [o[0], o[1], o[2]]
}

fn delta(a: [f32; 3], b: [f32; 3]) -> f32 {
    (0..3).map(|k| (a[k] - b[k]).abs()).fold(0.0, f32::max)
}

/// `(max |Δ|, pares com |Δ| > 1e-4)` da nossa lei `codigo` contra as linhas IM do modo `de`.
fn mede(linhas: &[Linha], codigo: u8, de: u8) -> (f32, usize) {
    let ds: Vec<f32> = linhas
        .iter()
        .filter(|l| l.codigo == de)
        .map(|l| delta(nosso(codigo, l.fundo, l.fonte), l.im))
        .collect();
    (
        ds.iter().copied().fold(0.0, f32::max),
        ds.iter().filter(|&&d| d > 1e-4).count(),
    )
}

#[test]
fn a_fixture_cobre_os_20_modos_com_o_mapa_do_script() {
    let (cores, linhas) = le();
    assert_eq!(cores.len(), 7);
    assert_eq!(linhas.len(), 20 * 49);
    for c in 0..20u8 {
        let do_modo: Vec<&Linha> = linhas.iter().filter(|l| l.codigo == c).collect();
        assert_eq!(do_modo.len(), 49, "modo {c}");
        assert!(
            do_modo.iter().all(|l| l.nome == MAPA[c as usize]),
            "modo {c} ≠ {}",
            MAPA[c as usize]
        );
    }
}

#[test]
fn os_modos_separaveis_concordam_com_o_imagemagick() {
    let (_, linhas) = le();
    for c in CONCORDAM {
        let (mx, _) = mede(&linhas, c, c);
        assert!(mx <= TOL, "{:?}: |Δ| {mx} > {TOL}", BlendMode::from_u8(c));
    }
}

#[test]
fn burn_e_dodge_so_divergem_no_canto_zero_sobre_zero() {
    let (_, linhas) = le();
    // (código, pares (fundo, fonte) onde o W3C e o IM decidem 0/0 ao contrário)
    let cantos: [(u8, &[(usize, usize)]); 3] =
        [(3, &[(1, 0)]), (7, &[(0, 1)]), (12, &[(0, 1), (1, 0)])];
    for (c, esperados) in cantos {
        for l in linhas.iter().filter(|l| l.codigo == c) {
            let (ours, d) = (
                nosso(c, l.fundo, l.fonte),
                delta(nosso(c, l.fundo, l.fonte), l.im),
            );
            if esperados.contains(&(l.ib, l.is)) {
                // W3C: o fundo decide (burn Cb=1 ⇒ 1, dodge Cb=0 ⇒ 0); o IM devolve a fonte.
                assert_eq!(
                    ours,
                    l.fundo,
                    "{:?} canto {:?}: nosso",
                    BlendMode::from_u8(c),
                    (l.ib, l.is)
                );
                assert_eq!(
                    l.im,
                    l.fonte,
                    "{:?} canto {:?}: IM",
                    BlendMode::from_u8(c),
                    (l.ib, l.is)
                );
            } else {
                assert!(
                    d <= TOL,
                    "{:?} {:?}: |Δ| {d}",
                    BlendMode::from_u8(c),
                    (l.ib, l.is)
                );
            }
        }
    }
}

#[test]
fn soft_light_do_imagemagick_e_o_w3c_mais_o_fundo() {
    let (_, linhas) = le();
    let (mx, n) = mede(&linhas, 10, 10);
    assert!(
        (mx - 0.5775).abs() < 1e-4 && n == 35,
        "divergência mudou: |Δ| {mx}, {n} pares"
    );
    for l in linhas.iter().filter(|l| l.codigo == 10) {
        let w3c = nosso(10, l.fundo, l.fonte);
        let im_previsto = [0, 1, 2].map(|k| (w3c[k] + l.fundo[k]).min(1.0));
        assert!(
            delta(im_previsto, l.im) <= TOL,
            "{:?}: {im_previsto:?} ≠ IM {:?}",
            (l.ib, l.is),
            l.im
        );
    }
}

#[test]
fn os_modos_hsl_divergem_pelo_modelo_de_cor_medido() {
    let (_, linhas) = le();
    // (código, max |Δ|, pares > 1e-4) medidos em 2026-10-11.
    for (c, max, pares) in [
        (16, 0.560_870, 24),
        (17, 0.560_929, 30),
        (18, 0.576_216, 33),
        (19, 0.576_216, 33),
    ] {
        let (mx, n) = mede(&linhas, c, c);
        assert!(
            (mx - max).abs() < 1e-5 && n == pares,
            "{:?}: |Δ| {mx} em {n} pares (pregado {max} em {pares})",
            BlendMode::from_u8(c)
        );
    }
}

/// CONTROLO: o gate não é vácuo — trocar o mapa de modos ou os operandos TEM de reprovar.
#[test]
fn controlo_mapa_trocado_e_operandos_trocados_reprovam() {
    let (_, linhas) = le();
    for c in CONCORDAM {
        for de in (0..20u8).filter(|&d| d != c) {
            let (mx, _) = mede(&linhas, c, de);
            assert!(
                mx > 0.01,
                "nosso {c} casa com IM {} (|Δ| {mx}): grelha não distingue",
                MAPA[de as usize]
            );
        }
    }
    // Os não comutativos: fundo e fonte trocados.
    for c in [0u8, 3, 7, 9, 11, 12, 13] {
        let mx = linhas
            .iter()
            .filter(|l| l.codigo == c)
            .map(|l| delta(nosso(c, l.fonte, l.fundo), l.im))
            .fold(0.0, f32::max);
        assert!(
            mx > 0.01,
            "{:?} com operandos trocados passou (|Δ| {mx})",
            BlendMode::from_u8(c)
        );
    }
}
