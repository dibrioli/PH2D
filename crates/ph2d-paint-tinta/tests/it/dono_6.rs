//! O caso `dono_6.png` (ADR-0197 Contexto): Thick Impasto (óleo `m = 0,85`, seco), amarelo sobre azul
//! assentado (30 µm, metade aberto), grelha pigmento φ × água — a MESMA da figura (a) do oráculo.
//! E a monotonia em N passadas (sem degraus).

use ph2d_paint_tinta::{Bandas, camada, dilui, mistura, s_fator};

const M: f32 = 0.85;
const V_AZUL: f32 = 30.0;
const ABERTURA: f32 = 0.5;
const CARGA: f32 = 40.0;
const EPS: f32 = 2.0e-6;

fn linspace(a: f32, b: f32, n: usize) -> Vec<f32> {
    (0..n)
        .map(|i| a + (b - a) * i as f32 / (n - 1) as f32)
        .collect()
}

fn pinta(phi: f32, agua: f32, n: u32) -> Bandas<3> {
    let (azul, amarelo) = (crate::azul(), crate::amarelo());
    let s = s_fator(M, false);
    let r_fech = camada(
        &mistura(&[(&azul, (1.0 - ABERTURA) * V_AZUL)], s),
        &crate::PAPEL,
    );
    let v_am = n as f32 * CARGA * dilui(phi, agua);
    camada(
        &mistura(&[(&azul, ABERTURA * V_AZUL), (&amarelo, v_am)], s),
        &r_fech,
    )
}

#[test]
fn amarelo_sobre_azul_nunca_escurece_nem_enferruja() {
    let s = s_fator(M, false);
    let (azul_puro, amar_puro) = (crate::azul().masstone(s), crate::amarelo().masstone(s));
    let ferrugem_amarelo = (amar_puro.0[0] - amar_puro.0[1]).max(0.0);
    let (mut pior_escuro, mut pior_rg, mut pior_ferrugem) =
        (f32::INFINITY, f32::INFINITY, f32::NEG_INFINITY);
    for &agua in &linspace(0.0, 0.9, 8) {
        for &phi in &linspace(0.05, 0.6, 8) {
            for n in [1, 3, 6] {
                let r = pinta(phi, agua, n);
                // O critério do oráculo: nunca mais escuro que o azul puro nas três bandas.
                let d = (0..3)
                    .map(|b| r.0[b] - azul_puro.0[b])
                    .fold(f32::NEG_INFINITY, f32::max);
                pior_escuro = pior_escuro.min(d);
                // Por banda onde o amarelo é mais claro que o azul (R e G): R ≥ azul puro.
                for b in 0..2 {
                    pior_rg = pior_rg.min(r.0[b] - azul_puro.0[b]);
                }
                // Por banda em todas (o teorema): R ≥ min dos masstones (o fundo é azul sobre papel ≥ azul).
                for b in 0..3 {
                    assert!(
                        r.0[b] >= azul_puro.0[b].min(amar_puro.0[b]) - EPS,
                        "teorema, banda {b}"
                    );
                }
                pior_ferrugem = pior_ferrugem.max((r.0[0] - r.0[1]) - ferrugem_amarelo);
            }
        }
    }
    println!(
        "dono_6: min max_banda(R − azul) = {pior_escuro:+.4} · min_{{R,G}}(R − azul) = {pior_rg:+.4} · \
         max ferrugem = {pior_ferrugem:+.4}"
    );
    assert!(
        pior_escuro > 0.0,
        "uma célula ficou mais escura que o azul puro"
    );
    assert!(pior_rg >= -EPS, "R ou G abaixo do azul puro: {pior_rg}");
    assert!(pior_ferrugem <= 1e-6, "ferrugem: {pior_ferrugem}");
}

#[test]
fn n_passadas_sao_monotonas_por_banda() {
    let mut casos = 0;
    for &agua in &linspace(0.0, 0.9, 8) {
        for &phi in &linspace(0.05, 0.6, 8) {
            let seq: Vec<Bandas<3>> = (0..=24).map(|n| pinta(phi, agua, n)).collect();
            for b in 0..3 {
                let sobe = seq[24].0[b] >= seq[0].0[b];
                for w in seq.windows(2) {
                    let d = w[1].0[b] - w[0].0[b];
                    assert!(
                        if sobe { d >= -EPS } else { d <= EPS },
                        "degrau na banda {b}, φ={phi}, água={agua}: {seq:?}"
                    );
                }
            }
            casos += 1;
        }
    }
    println!("monotonia: {casos} células × 24 passadas × 3 bandas");
}
