//! B49.2: a base do grão tira-se UMA vez por dab e por chamada (antes, a cada par de dabs: N²), ao bit.

use super::{BASES_DO_GRAO, UmaPassagem};
use crate::tool::PainterTool;
use crate::tool::paint::media::PaintMedia;
use ph2d_editor_core::tool::RasterEditTool as _;
use ph2d_painter_brush::{Dab, TextureKind, TextureMapping};

const LADO: u32 = 256;

fn pincel(mapping: TextureMapping) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (LADO * LADO * 4) as usize], LADO, LADO);
    t.set_paint_media(PaintMedia::Digital);
    t.set_brush_size_px(40.0);
    t.set_brush_texture_kind(TextureKind::Noise.to_u8());
    t.paint.brush.texture.mapping = mapping;
    t
}

/// `n` dabs de raio 40 a 1 px uns dos outros: todo centro é alcançado por todos (o pior caso do 1.º diâmetro).
#[allow(clippy::cast_precision_loss)]
fn dabs(n: usize) -> Vec<Dab> {
    (0..n)
        .map(|i| Dab {
            center: [96.0 + i as f32 * 1.0, 128.0 + (i % 5) as f32 * 0.7],
            radius_px: 40.0,
            coverage: 0.6,
            color: [0.2, 0.3, 0.9],
            color_shift: [0.0; 3],
            rotation: [1.0, 0.0],
            dir: [1.0, 0.0],
            arc_len: i as f32,
            stroke_radius_px: 40.0,
            step: [0.0, 0.0],
            ante: None,
            tilt: None,
        })
        .collect()
}

fn passagem(n: usize) -> UmaPassagem<f32> {
    let mut p = UmaPassagem::<f32>::default();
    p.dabs = dabs(n);
    p
}

/// A cobertura da régua: varia por texel para que o máximo dependa do grão de cada centro.
#[allow(clippy::cast_precision_loss)]
fn cobertura(i: usize) -> f32 {
    0.25 + ((i * 2_654_435_761) % 997) as f32 / 1994.0
}

#[test]
fn a_base_do_grao_tira_se_uma_vez_por_dab() {
    const N: usize = 64;
    for mapping in [TextureMapping::Tiled, TextureMapping::ViewPlane] {
        let t = pincel(mapping);
        assert!(
            t.grao_que_o_nivel_le(),
            "{mapping:?}: a régua precisa de um grão que o nível lê"
        );
        let mut p = passagem(N);
        BASES_DO_GRAO.with(|n| n.set(0));
        let nivel = t.nivel_no_grao_medio(&mut p, cobertura);
        let bases = BASES_DO_GRAO.with(std::cell::Cell::get);
        assert!(nivel.is_some(), "{mapping:?}: o nível tem de ler-se");
        assert!(
            bases <= 1 + N,
            "{mapping:?}: {bases} bases do grão com {N} dabs (tecto 1 + N = {})",
            1 + N
        );
    }
}

/// O caminho de antes do B49.2, VERBATIM (só `self` → `t` e o `p.dabs` passados), como oráculo ao bit.
mod antigo {
    use super::super::{AMOSTRAS_DO_GRAO, GRAO_MINIMO, LADO_DA_MEDIA_TILED};
    use crate::tool::PainterTool;
    use ph2d_det_math::DetMath as _;
    use ph2d_painter_brush::Dab;
    use ph2d_painter_brush::texture::TextureMapping;

    fn grao_do_dab(t: &PainterTool, d: &Dab, x: i64, y: i64) -> f32 {
        let b = &t.paint.brush;
        let dims = [t.source_size.0 as f32, t.source_size.1 as f32];
        let base = ph2d_painter_brush::texture::dab_basis(
            &b.texture,
            &mut 0u64,
            dims,
            b.dab_footprint_of(d),
        );
        let imagem = t.paint.texture_image.as_ref().map(|i| i.as_mask());
        ph2d_painter_brush::dab::grain_at(b, &base, imagem.as_ref(), x, y, d.center, d.radius_px)
    }

    pub(super) fn grao_medio(t: &PainterTool, d: &Dab) -> f32 {
        let b = &t.paint.brush;
        let tiled = b.texture.mapping == TextureMapping::Tiled;
        let r = d.radius_px.max(0.5);
        let (a1, a2) = (0.754_877_7_f32, 0.569_840_3_f32);
        let (mut soma, mut peso) = (0.0f64, 0.0f64);
        for k in 0..AMOSTRAS_DO_GRAO {
            let (u, v) = ((0.5 + a1 * k as f32).fract(), (0.5 + a2 * k as f32).fract());
            let (x, y, w) = if tiled {
                (u * LADO_DA_MEDIA_TILED, v * LADO_DA_MEDIA_TILED, 1.0)
            } else {
                let (dx, dy) = ((2.0 * u - 1.0) * r, (2.0 * v - 1.0) * r);
                let t = dx.det_hypot(dy) / r;
                if t >= 1.0 {
                    continue;
                }
                (d.center[0] + dx, d.center[1] + dy, b.falloff_weight(t))
            };
            let g = grao_do_dab(t, d, x.floor() as i64, y.floor() as i64);
            soma += f64::from(w * g);
            peso += f64::from(w);
        }
        if peso > 0.0 {
            (soma / peso) as f32
        } else {
            1.0
        }
    }

    pub(super) fn nivel_no_grao_medio(
        tool: &PainterTool,
        dabs: &[Dab],
        medio: f32,
        cobertura: impl Fn(usize) -> f32,
    ) -> Option<f32> {
        let b = &tool.paint.brush;
        let linear = tool.cobertura_tapada(b);
        let (w, h) = (tool.source_size.0 as i64, tool.source_size.1 as i64);
        let mut melhor: Option<f32> = None;
        for c in dabs {
            let (x, y) = (c.center[0].floor() as i64, c.center[1].floor() as i64);
            if x < 0 || y < 0 || x >= w || y >= h {
                continue;
            }
            let v = cobertura((y * w + x) as usize);
            if v <= 0.0 {
                continue;
            }
            let (px, py) = (x as f32 + 0.5, y as f32 + 0.5);
            let (mut soma, mut peso) = (0.0f32, 0.0f32);
            for d in dabs {
                let t = (px - d.center[0]).det_hypot(py - d.center[1]) / d.radius_px.max(0.5);
                if t < 1.0 {
                    let k = b.falloff_weight(t);
                    soma += k * grao_do_dab(tool, d, x, y);
                    peso += k;
                }
            }
            if peso <= 0.0 || soma / peso < GRAO_MINIMO {
                continue;
            }
            let razao = medio / (soma / peso);
            let n = if linear {
                v * razao
            } else {
                1.0 - (1.0 - v.min(1.0)).det_powf(razao)
            };
            melhor = Some(melhor.map_or(n, |m: f32| m.max(n)));
        }
        melhor.map(|n| n.clamp(0.0, 1.0))
    }
}

/// Paridade ao bit com o caminho de antes: o grão médio e o nível, Tiled e View, para vários comprimentos do 1.º diâmetro.
#[test]
fn o_nivel_no_grao_medio_e_o_antigo_ao_bit() {
    for mapping in [TextureMapping::Tiled, TextureMapping::ViewPlane] {
        let t = pincel(mapping);
        let mut lidos = 0usize;
        for n in [1usize, 2, 7, 33, 64] {
            let mut p = passagem(n);
            let novo = t.nivel_no_grao_medio(&mut p, cobertura);
            let medio = antigo::grao_medio(&t, &p.dabs[0]);
            assert_eq!(
                p.grao_medio.map(f32::to_bits),
                Some(medio.to_bits()),
                "{mapping:?} n={n}: grão médio"
            );
            let velho = antigo::nivel_no_grao_medio(&t, &p.dabs, medio, cobertura);
            assert_eq!(
                novo.map(f32::to_bits),
                velho.map(f32::to_bits),
                "{mapping:?} n={n}: nível"
            );
            lidos += usize::from(velho.is_some());
        }
        assert!(
            lidos >= 3,
            "{mapping:?}: só {lidos} de 5 comprimentos leram um nível"
        );
    }
}

/// O custo de uma leitura com 64 dabs, o antigo e o novo intercalados no mesmo processo (o MÍNIMO de 9).
#[test]
#[ignore = "medição: cargo-test-narrow.sh ph2d-tool-painter --profile smoke -- --ignored mede_a_base_do_grao --nocapture"]
fn mede_a_base_do_grao() {
    let t = pincel(TextureMapping::Tiled);
    let p0 = passagem(64);
    let medio = antigo::grao_medio(&t, &p0.dabs[0]);
    let (mut a, mut b) = (f64::INFINITY, f64::INFINITY);
    for _ in 0..9 {
        let t0 = std::time::Instant::now();
        std::hint::black_box(antigo::nivel_no_grao_medio(&t, &p0.dabs, medio, cobertura));
        a = a.min(t0.elapsed().as_secs_f64() * 1e3);
        let mut p = passagem(64);
        p.grao_medio = Some(medio);
        let t0 = std::time::Instant::now();
        std::hint::black_box(t.nivel_no_grao_medio(&mut p, cobertura));
        b = b.min(t0.elapsed().as_secs_f64() * 1e3);
    }
    let base = {
        let d = p0.dabs[0];
        let t0 = std::time::Instant::now();
        for _ in 0..100_000 {
            std::hint::black_box(t.base_do_dab(std::hint::black_box(&d)));
        }
        t0.elapsed().as_secs_f64() * 1e9 / 100_000.0
    };
    eprintln!("[B49.2] 64 dabs: antigo {a:.3} ms · novo {b:.3} ms · dab_basis {base:.0} ns");
}
