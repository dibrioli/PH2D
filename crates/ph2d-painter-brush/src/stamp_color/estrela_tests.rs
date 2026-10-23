//! ⭐ **O carimbo COLORIDO pequeno não tem degraus** (o Stars pequeno, 23/10, na rota de «Use Texture Colors» /
//! cor por camada). O lote assa a máscara colorida do MAIOR dab; cada kernel lê nela o nível da pirâmide do
//! tamanho de CADA dab ([`ColorStampMask::nivel_para`], escolhido uma vez por dab). A régua é a ÁREA EXACTA do píxel
//! (16×16 amostras pontuais da mesma ponta) contra a cobertura que cada kernel escreve, num lote com um dab
//! grande (que obriga as bandas) e o dab pequeno medido.

use super::*;
use crate::spec::BrushSpec;
use crate::texture::{
    ImageMask, TextureKind, TextureMapping, TextureSettings, dab_basis,
    sample_shape_silhouette_unit,
};

const LADO: u32 = 128;
const SUB: usize = 16;
const W: u32 = 600;
const H: u32 = 260;
const C: [f32; 2] = [100.3, 100.7];
/// O mesmo tecto do `texture::shape::estrela_tests`: o bilinear de uma borda dura a ≤ 2:1 já difere da caixa
/// até ~0,19; os degraus são ≥ 0,3.
const TOL_MAX: f32 = 0.2;

/// A estrela do Stars no que importa: miolo `178`, um aro claro de 2 texels (`255`).
fn aro() -> Vec<u8> {
    (0..LADO * LADO)
        .map(|k| {
            let (x, y) = (
                (k % LADO) as f32 + 0.5 - 64.0,
                (k / LADO) as f32 + 0.5 - 64.0,
            );
            match x.hypot(y) {
                d if d < 46.0 => 178,
                d if d < 48.0 => 255,
                _ => 0,
            }
        })
        .collect()
}

fn spec() -> BrushSpec {
    BrushSpec {
        shape: TextureSettings {
            kind: TextureKind::Image,
            mapping: TextureMapping::ViewPlane,
            ..Default::default()
        },
        ..Default::default()
    }
}

/// `(máx, média)` do erro de `cov(px, py)` (`0..1`) contra a área exacta do píxel, na caixa do dab de raio `r`.
fn erro(lum: &[u8], r: f32, cov: impl Fn(usize) -> f32) -> (f32, f32) {
    let img = ImageMask {
        lum,
        width: LADO,
        height: LADO,
    };
    let s = spec().shape;
    let b = dab_basis(
        &s,
        &mut 0u64,
        [1.0, 1.0],
        crate::footprint::FootprintDeform::identity(),
    );
    let (mut max, mut soma, mut n) = (0.0_f32, 0.0_f32, 0usize);
    let (lo, hi) = ((C[0] - r).floor() as i64, (C[0] + r).ceil() as i64);
    for py in lo..=hi {
        for px in lo..=hi {
            let mut area = 0.0;
            for j in 0..SUB {
                for i in 0..SUB {
                    let u = (px as f32 + (i as f32 + 0.5) / SUB as f32 - C[0]) / r;
                    let v = (py as f32 + (j as f32 + 0.5) / SUB as f32 - C[1]) / r;
                    area += sample_shape_silhouette_unit(&s, &b, u, v, Some(&img));
                }
            }
            let e = (cov(py as usize * W as usize + px as usize) - area / (SUB * SUB) as f32).abs();
            max = max.max(e);
            soma += e;
            n += 1;
        }
    }
    (max, soma / n as f32)
}

/// A cobertura que cada kernel colorido escreve para o dab de raio `r` em [`C`] (com um dab grande longe, no lote).
fn kernels(stamp: &ColorStampMask, r: f32) -> Vec<(&'static str, Vec<f32>)> {
    let len = (W * H) as usize;
    let lote = [
        FusedDab {
            center: [430.0, 130.0],
            radius: 110.0,
            coverage: 1.0,
        },
        FusedDab {
            center: C,
            radius: r,
            coverage: 1.0,
        },
    ];
    let stamps = std::slice::from_ref(stamp);
    let a = |v: &[u8], k: usize, passo: usize| {
        v.iter()
            .skip(k)
            .step_by(passo)
            .map(|&b| f32::from(b) / 255.0)
            .collect()
    };
    let mut out = Vec::new();
    let mut buf = vec![0u8; len * 4];
    let _ = blit_color_stamp(&mut buf, W, H, C, r, stamp, &spec(), 1.0, false, [0.0; 3]);
    out.push(("blit_color_stamp", a(&buf, 3, 4)));
    let mut cov = vec![0u8; len];
    let _ = accumulate_color_stamp_coverage(&mut cov, W, H, C, r, stamp, 1.0);
    out.push(("accumulate_color_stamp_coverage", a(&cov, 0, 1)));
    let mut covs = vec![vec![0u8; len]];
    let _ = accumulate_color_stamps_fused(&mut covs, W, H, C, r, stamps, 1.0);
    out.push(("accumulate_color_stamps_fused", a(&covs[0], 0, 1)));
    let mut covs = vec![vec![0u8; len]];
    let _ = accumulate_color_stamps_fused_batch(&mut covs, W, H, &lote, stamps);
    out.push(("accumulate_color_stamps_fused_batch", a(&covs[0], 0, 1)));
    let mut accs = vec![vec![0u8; len * 4]];
    let _ = accumulate_color_stamps_rgba_batch(&mut accs, W, H, &lote, stamps);
    out.push(("accumulate_color_stamps_rgba_batch", a(&accs[0], 3, 4)));
    out
}

/// ⭐ Cada kernel do carimbo colorido, num dab de 10, 20 e 40 px lido de uma máscara assada a 256 texels (a do
/// dab grande do lote), fica a [`TOL_MAX`] da área exacta. **Mutação que tem de sangrar:**
/// [`ColorStampMask::indice_do_nivel`] devolver `0` (o topo, a lei de antes).
#[test]
fn o_carimbo_colorido_pequeno_nao_tem_degraus() {
    let lum = aro();
    let img = ImageMask {
        lum: &lum,
        width: LADO,
        height: LADO,
    };
    let stamp = render_color_stamp_mask(
        &spec(),
        &[img],
        &[[1.0; 3]],
        &[None],
        &[1.0],
        None,
        None,
        256,
    );
    let mut falhas = Vec::new();
    for r in [5.0_f32, 10.0, 20.0] {
        let nivel = stamp.nivel_para(2.0 * r);
        if !(nivel.size() as f32 >= 2.0 * r && (nivel.size() as f32) < 4.0 * r) {
            falhas.push(format!("r={r}: o nível {} não é o do dab", nivel.size()));
        }
        for (nome, cov) in kernels(&stamp, r) {
            let (max, med) = erro(&lum, r, |i| cov[i]);
            eprintln!(
                "ESTRELA cor r={r} {nome}: max={max:.3} media={med:.4} (nível {})",
                nivel.size()
            );
            if max > TOL_MAX {
                falhas.push(format!("r={r} {nome}: {max:.3}"));
            }
        }
    }
    assert!(falhas.is_empty(), "degraus no carimbo colorido: {falhas:?}");
    // O CONTROLO: o topo (256 texels) lido num ponto por píxel tem os degraus — a régua vê o defeito.
    let r = 5.0;
    let (max0, _) = erro(&lum, r, |i| {
        let (px, py) = ((i % W as usize) as f32, (i / W as usize) as f32);
        sample_color_mask(&stamp, (px + 0.5 - C[0]) / r, (py + 0.5 - C[1]) / r).1
    });
    assert!(
        max0 > TOL_MAX,
        "CONTROLO: o topo não mostra degraus ({max0})"
    );
}
