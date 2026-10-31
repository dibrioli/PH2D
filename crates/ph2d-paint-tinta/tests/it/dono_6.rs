//! O caso `dono_6.png` (ADR-0197 Contexto): Thick Impasto (óleo `m = 0,85`, seco), amarelo sobre azul
//! assentado (30 µm, metade aberto), grelha pigmento φ × água — a mesma da figura (a) do oráculo.
//! Com a mistura Oklab: nada mais escuro que o azul puro em L e nenhum matiz fora do arco
//! azul→amarelo (sem ferrugem). E a monotonia em N passadas (sem degraus).

use ph2d_paint_tinta::oklab::linear_para_oklab;
use ph2d_paint_tinta::{Bandas, Soma, camada, dilui, optica, s_fator};

const M: f32 = 0.85;
const V_AZUL: f32 = 30.0;
const ABERTURA: f32 = 0.5;
const CARGA: f32 = 40.0;
const EPS: f32 = 2.0e-6;
/// Abaixo deste croma Oklab o matiz não está definido (ruído de `f32` no neutro).
const CROMA_MIN: f32 = 0.01;

fn linspace(a: f32, b: f32, n: usize) -> Vec<f32> {
    (0..n)
        .map(|i| a + (b - a) * i as f32 / (n - 1) as f32)
        .collect()
}

fn pinta(phi: f32, agua: f32, n: u32) -> Bandas<3> {
    let s = s_fator(M, false);
    let fechado = Soma::de(crate::azul(), (1.0 - ABERTURA) * V_AZUL);
    let r_fech = camada(&optica(&fechado, s), &crate::PAPEL);
    let v_am = n as f32 * CARGA * dilui(phi, agua);
    let aberto = Soma::de(crate::azul(), ABERTURA * V_AZUL).junta(Soma::de(crate::amarelo(), v_am));
    camada(&optica(&aberto, s), &r_fech)
}

fn matiz(lab: [f32; 3]) -> f32 {
    libm::atan2f(lab[2], lab[1]).to_degrees().rem_euclid(360.0)
}

#[test]
fn amarelo_sobre_azul_nunca_escurece_nem_enferruja() {
    let (az, am) = (crate::azul().lab, crate::amarelo().lab);
    let (h_az, h_am) = (matiz(az), matiz(am));
    // O arco CURTO entre os dois matizes (azul ≈ 269°, amarelo ≈ 102°: passa pelo verde, 180°);
    // a ferrugem (laranja-vermelho, ~40°) fica fora dele.
    let (lo, hi) = (h_az.min(h_am), h_az.max(h_am));
    assert!(hi - lo < 180.0);
    let (mut pior_l, mut pior_arco, mut n) = (f32::INFINITY, f32::INFINITY, 0);
    for &agua in &linspace(0.0, 0.9, 8) {
        for &phi in &linspace(0.05, 0.6, 8) {
            for k in [1, 3, 6, 24] {
                let lab = linear_para_oklab(pinta(phi, agua, k).0);
                pior_l = pior_l.min(lab[0] - az[0]);
                if lab[1].hypot(lab[2]) >= CROMA_MIN {
                    let h = matiz(lab);
                    pior_arco = pior_arco.min((h - lo).min(hi - h));
                }
                n += 1;
            }
        }
    }
    println!(
        "dono_6: {n} células · min (L − L_azul) = {pior_l:+.4} · folga mínima ao arco \
         [{lo:.1}°, {hi:.1}°] = {pior_arco:+.2}°"
    );
    assert!(pior_l > 0.0, "uma célula ficou mais escura que o azul puro");
    assert!(
        pior_arco >= 0.0,
        "matiz fora do arco azul→amarelo (ferrugem)"
    );
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
