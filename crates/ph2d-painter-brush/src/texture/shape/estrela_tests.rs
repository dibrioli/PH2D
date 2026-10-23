//! ⭐ **O Stars pequeno não tem degraus** (o dono, 23/10: «o pincel Stars fica com baixa resolução quando o
//! tamanho do brush é reduzido»). A ponta de imagem REDUZIDA lê-se pela área do píxel nas duas rotas — a por
//! píxel (sem pedir a lei `area`) e a máscara em cache (o nível da pirâmide do tamanho do dab). A régua é a
//! ÁREA EXACTA do píxel: a média de 16×16 amostras pontuais da mesma ponta.

use super::super::{ImageMask, TextureKind, TextureMapping, TextureSettings, dab_basis};
use super::{sample_shape, sample_shape_silhouette_unit};
use crate::footprint::FootprintDeform;

const LADO: u32 = 128;
const SUB: usize = 16;

/// A estrela do Stars no que importa: miolo `178` e um ARO claro de 2 texels (`255`) — o traço fino que, lido
/// num ponto por píxel, acende e apaga em degraus.
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

fn ponta() -> TextureSettings {
    TextureSettings {
        kind: TextureKind::Image,
        mapping: TextureMapping::ViewPlane,
        ..Default::default()
    }
}

/// `(máx, média)` do erro absoluto de `ler(px, py)` contra a área exacta do píxel, na caixa do dab.
fn erro(r: f32, ler: impl Fn(i64, i64) -> f32) -> (f32, f32) {
    let lum = aro();
    let img = ImageMask {
        lum: &lum,
        width: LADO,
        height: LADO,
    };
    let s = ponta();
    let b = dab_basis(&s, &mut 0u64, [1.0, 1.0], FootprintDeform::identity());
    let c = [100.3_f32, 100.7];
    let (mut max, mut soma, mut n) = (0.0_f32, 0.0_f32, 0usize);
    let (lo, hi) = ((c[0] - r).floor() as i64, (c[0] + r).ceil() as i64);
    for py in lo..=hi {
        for px in lo..=hi {
            let mut area = 0.0;
            for j in 0..SUB {
                for i in 0..SUB {
                    let u = (px as f32 + (i as f32 + 0.5) / SUB as f32 - c[0]) / r;
                    let v = (py as f32 + (j as f32 + 0.5) / SUB as f32 - c[1]) / r;
                    area += sample_shape_silhouette_unit(&s, &b, u, v, Some(&img));
                }
            }
            let e = (ler(px, py) - area / (SUB * SUB) as f32).abs();
            max = max.max(e);
            soma += e;
            n += 1;
        }
    }
    (max, soma / n as f32)
}

fn por_pixel(r: f32) -> (f32, f32) {
    let lum = aro();
    let img = ImageMask {
        lum: &lum,
        width: LADO,
        height: LADO,
    };
    let s = ponta();
    let b = dab_basis(&s, &mut 0u64, [1.0, 1.0], FootprintDeform::identity());
    erro(r, |px, py| {
        sample_shape(&s, &b, px, py, [100.3, 100.7], r, Some(&img))
    })
}

/// O erro máximo tolerado contra a área exacta. O bilinear de uma borda DURA a ≤ 2:1 já difere da caixa até ~0,19
/// (medido: o raio 54, 1,18 texel por píxel, é a amostra no centro ao bit e dá 0,185); os degraus são ≥ 0,4
/// (o ponto por píxel: 0,55 a raio 5, 0,43 a raio 10).
const TOL_MAX: f32 = 0.2;

/// ⭐ A rota POR PÍXEL (a que o Stars toma: Accumulate desligado) sem a lei `area` no preset: reduzida, cada
/// píxel é a área dele — o aro fino não acende e apaga em degraus.
#[test]
fn a_estrela_pequena_nao_tem_degraus_por_pixel() {
    for r in [5.0_f32, 10.0, 20.0, 54.0] {
        let (max, med) = por_pixel(r);
        eprintln!("ESTRELA por_pixel r={r}: max={max:.3} media={med:.4}");
        assert!(
            max <= TOL_MAX,
            "raio {r}: erro máx {max} contra a área do píxel"
        );
    }
    // O CONTROLO: a amostra num ponto por píxel (a lei de antes) tem os degraus a raio 5 — a régua vê o defeito.
    let lum = aro();
    let img = ImageMask {
        lum: &lum,
        width: LADO,
        height: LADO,
    };
    let s = ponta();
    let b = dab_basis(&s, &mut 0u64, [1.0, 1.0], FootprintDeform::identity());
    let r = 5.0;
    let (max0, _) = erro(r, |px, py| {
        let (u, v) = ((px as f32 + 0.5 - 100.3) / r, (py as f32 + 0.5 - 100.7) / r);
        sample_shape_silhouette_unit(&s, &b, u, v, Some(&img))
    });
    assert!(
        max0 > TOL_MAX,
        "CONTROLO: o ponto por píxel não mostra degraus ({max0})"
    );
}

/// ⭐ A MÁSCARA EM CACHE assada para o maior dab do lote (128 texels) e lida num dab de 20 px: o nível da
/// pirâmide do tamanho dele é a área; o CONTROLO — a máscara de cima lida num ponto por píxel (o que o blit
/// fazia) — tem os degraus, senão a régua não vê o defeito.
#[test]
fn a_estrela_pequena_nao_tem_degraus_na_mascara_em_cache() {
    use crate::spec::BrushSpec;
    use crate::stamp::{render_stamp_mask, sample_mask};
    let lum = aro();
    let img = ImageMask {
        lum: &lum,
        width: LADO,
        height: LADO,
    };
    let spec = BrushSpec {
        shape: ponta(),
        falloff: crate::Falloff::Constant,
        ..Default::default()
    };
    let mask = render_stamp_mask(&spec, None, Some(&img), None, 256);
    assert!(
        (mask.reach() - 1.0).abs() < 1e-6,
        "a ponta sem Offset cobre o raio"
    );
    let c = [100.3_f32, 100.7];
    for r in [5.0_f32, 10.0, 20.0] {
        fn ler(m: &crate::stamp::StampMask, c: [f32; 2], r: f32) -> impl Fn(i64, i64) -> f32 + '_ {
            move |px, py| {
                sample_mask(
                    m,
                    (px as f32 + 0.5 - c[0]) / r,
                    (py as f32 + 0.5 - c[1]) / r,
                )
            }
        }
        let nivel = mask.nivel_para(2.0 * r);
        let (max, med) = erro(r, ler(nivel, c, r));
        let (max0, med0) = erro(r, ler(&mask, c, r));
        eprintln!(
            "ESTRELA cache r={r}: nivel={} max={max:.3} media={med:.4} | topo={} max={max0:.3} media={med0:.4}",
            nivel.size(),
            mask.size()
        );
        assert!(nivel.size() as f32 >= 2.0 * r && (nivel.size() as f32) < 4.0 * r);
        assert!(
            max <= TOL_MAX,
            "raio {r}: erro máx {max} no nível {}",
            nivel.size()
        );
        assert!(
            max0 > TOL_MAX,
            "CONTROLO raio {r}: a máscara de cima não mostra degraus ({max0})"
        );
    }
}

/// ⭐ **O GRÃO de imagem pequeno também não tem degraus** (View): a mesma imagem como Grain, pela rota por píxel
/// ([`crate::texture::sample`]) e pela máscara em cache assada a 256 (o nível do dab), fica a [`TOL_MAX`] da área
/// exacta no miolo do dab (`0,9 r`, longe do recorte do falloff). O CONTROLO: o ponto por píxel tem os degraus.
/// **Mutação que tem de sangrar:** `grao_de_imagem_pela_area` devolver a amostra no centro.
#[test]
fn o_grao_de_imagem_pequeno_nao_tem_degraus() {
    use crate::spec::BrushSpec;
    use crate::stamp::{render_stamp_mask, sample_mask};
    use crate::texture::{sample, sample_unit};
    let lum = aro();
    let img = ImageMask {
        lum: &lum,
        width: LADO,
        height: LADO,
    };
    let s = ponta();
    let b = dab_basis(&s, &mut 0u64, [1.0, 1.0], FootprintDeform::identity());
    let spec = BrushSpec {
        texture: s,
        falloff: crate::Falloff::Constant,
        ..Default::default()
    };
    let mask = render_stamp_mask(&spec, Some(&img), None, None, 256);
    let c = [100.3_f32, 100.7];
    // `(máx)` de `ler(px, py)` contra a área exacta do grão, no miolo.
    let erro = |r: f32, ler: &dyn Fn(i64, i64) -> f32| {
        let mut max = 0.0_f32;
        let (lo, hi) = ((c[0] - r).floor() as i64, (c[0] + r).ceil() as i64);
        for py in lo..=hi {
            for px in lo..=hi {
                let (dx, dy) = (px as f32 + 0.5 - c[0], py as f32 + 0.5 - c[1]);
                if dx.hypot(dy) > 0.9 * r {
                    continue;
                }
                let mut area = 0.0;
                for j in 0..SUB {
                    for i in 0..SUB {
                        let u = (px as f32 + (i as f32 + 0.5) / SUB as f32 - c[0]) / r;
                        let v = (py as f32 + (j as f32 + 0.5) / SUB as f32 - c[1]) / r;
                        area += sample_unit(&s, &b, u, v, Some(&img));
                    }
                }
                max = max.max((ler(px, py) - area / (SUB * SUB) as f32).abs());
            }
        }
        max
    };
    let mut falhas = Vec::new();
    for r in [5.0_f32, 10.0, 20.0, 54.0] {
        let pp = erro(r, &|px, py| sample(&s, &b, px, py, c, r, Some(&img)));
        let nivel = mask.nivel_para(2.0 * r);
        let cache = erro(r, &|px, py| {
            sample_mask(
                nivel,
                (px as f32 + 0.5 - c[0]) / r,
                (py as f32 + 0.5 - c[1]) / r,
            )
        });
        eprintln!(
            "GRAO r={r}: por_pixel max={pp:.3} · cache (nível {}) max={cache:.3}",
            nivel.size()
        );
        if pp > TOL_MAX || cache > TOL_MAX {
            falhas.push(format!("r={r}: por píxel {pp:.3} · cache {cache:.3}"));
        }
    }
    assert!(falhas.is_empty(), "degraus no grão de imagem: {falhas:?}");
    let r = 5.0;
    let ponto = erro(r, &|px, py| {
        let (u, v) = ((px as f32 + 0.5 - c[0]) / r, (py as f32 + 0.5 - c[1]) / r);
        sample_unit(&s, &b, u, v, Some(&img))
    });
    assert!(
        ponto > TOL_MAX,
        "CONTROLO: o ponto por píxel não mostra degraus ({ponto})"
    );
}
