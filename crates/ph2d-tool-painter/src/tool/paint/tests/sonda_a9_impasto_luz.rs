//! **Sonda do smoke do dono 15/10 (P39, Impasto × Voronoi × Shape Color × Continuous)**: a régua do A9
//! media a tela SEM a luz; o dono vê a tela ILUMINADA. Mede a tela acesa, o relevo e a cor crua, e atribui
//! a diferença trocando os ingredientes: a luz do ALVO sobre a cor do teste (só cor) e a luz do teste sobre
//! a cor do alvo (só relevo).
//! `PH2D_SONDA_OUT=<pasta> cargo test -p ph2d-tool-painter --profile smoke --lib sonda_a9_impasto_luz -- --ignored --nocapture`

use super::capsula::{
    H, Meio, W, acesa, dif_cor, dif_relevo, pincel, relevo, traco, traco_impasto, verde_laranja,
};
use ph2d_painter_brush::TextureKind;

const TAM: f32 = 40.0;

/// O traço do portão ([`traco_impasto`]); `fino` = as células miúdas da foto do dono (Shape Size 3).
fn corre(
    kind: TextureKind,
    cor: bool,
    tiled: bool,
    fino: bool,
    sp: f32,
    cont: bool,
) -> (crate::tool::PainterTool, Vec<u8>) {
    if !fino {
        return traco_impasto(kind, cor, tiled, sp, cont);
    }
    let mut t = pincel(Meio::Impasto, kind, TAM, sp, cont);
    t.paint.brush.shape.size = [3.0; 2];
    t.set_shape_color_ramp(verde_laranja());
    t.set_shape_ramp_enabled(cor);
    let seed = t.paint.brush;
    t.paint.brush_by_mode.fill(seed);
    let px = traco(&mut t);
    (t, px)
}

/// Quanto da variância do relevo no miolo é ENTRE linhas (listras ao longo do traço): 1 = só listras.
pub(super) fn listras_relevo(h: &[f32]) -> f32 {
    let pts = super::capsula::miolo(TAM / 2.0);
    let v: Vec<f32> = pts.iter().map(|&(x, y)| h[y * W as usize + x]).collect();
    let m = v.iter().sum::<f32>() / v.len() as f32;
    let total: f32 = v.iter().map(|a| (a - m) * (a - m)).sum();
    let mut linhas: std::collections::BTreeMap<usize, Vec<f32>> = Default::default();
    for (p, a) in pts.iter().zip(&v) {
        linhas.entry(p.1).or_default().push(*a);
    }
    let dentro: f32 = linhas
        .values()
        .map(|l| {
            let ml = l.iter().sum::<f32>() / l.len() as f32;
            l.iter().map(|a| (a - ml) * (a - ml)).sum::<f32>()
        })
        .sum();
    if total < 1e-12 {
        f32::NAN
    } else {
        1.0 - dentro / total
    }
}

fn grava(dir: &str, nome: &str, px: &[u8]) {
    let mut ppm = format!("P6 {W} {H} 255\n").into_bytes();
    for k in 0..(W * H) as usize {
        ppm.extend_from_slice(&px[k * 4..k * 4 + 3]);
    }
    std::fs::write(format!("{dir}/{nome}.ppm"), ppm).expect("grava");
}

#[test]
#[ignore = "sonda: o Impasto com Continuous na tela ILUMINADA (smoke do dono P39)"]
fn sonda_a9_impasto_luz() {
    let out = std::env::var("PH2D_SONDA_OUT").ok();
    let r = TAM / 2.0;
    println!(
        "| caso | S | caixa | Δacesa | Δcrua | Δ só cor | Δ só relevo | Δrelevo | corr relevo | listras relevo (alvo) |"
    );
    println!("|---|---:|---|---:|---:|---:|---:|---:|---:|---:|");
    for (nome, kind, cor, tiled, fino) in [
        ("Voronoi cor", TextureKind::Voronoi, true, false, false),
        ("Voronoi cor fino", TextureKind::Voronoi, true, false, true),
        ("Checker cor", TextureKind::Checker, true, false, false),
        ("Noise cor", TextureKind::Noise, true, false, false),
        ("Voronoi sem cor", TextureKind::Voronoi, false, false, false),
        ("Voronoi cor Tiled", TextureKind::Voronoi, true, true, false),
        ("None cor", TextureKind::None, true, false, false),
    ] {
        let (ta, ca) = corre(kind, cor, tiled, fino, 0.025, false);
        let (la, ha) = (acesa(&ta, &ca), relevo(&ta));
        let n = nome.replace(' ', "_");
        if let Some(d) = &out {
            grava(d, &format!("{n}_alvo_acesa"), &la);
        }
        for (sp, cont) in [(0.01f32, false), (0.25, false), (0.25, true)] {
            let (tt, ct) = corre(kind, cor, tiled, fino, sp, cont);
            let (lt, ht) = (acesa(&tt, &ct), relevo(&tt));
            let so_cor = dif_cor(&acesa(&ta, &ct), &la, r);
            let so_rel = dif_cor(&acesa(&tt, &ca), &la, r);
            let (dr, cr) = dif_relevo(&ht, &ha, r);
            println!(
                "| {nome} | {sp} | {} | {:.4} | {:.4} | {:.4} | {:.4} | {:.4} | {:.3} | {:.3} ({:.3}) |",
                if cont { "sim" } else { "não" },
                dif_cor(&lt, &la, r),
                dif_cor(&ct, &ca, r),
                so_cor,
                so_rel,
                dr,
                cr,
                listras_relevo(&ht),
                listras_relevo(&ha),
            );
            if let Some(d) = &out {
                grava(d, &format!("{n}_{sp}_{cont}_acesa"), &lt);
                grava(d, &format!("{n}_{sp}_{cont}_crua"), &ct);
            }
        }
    }
}
