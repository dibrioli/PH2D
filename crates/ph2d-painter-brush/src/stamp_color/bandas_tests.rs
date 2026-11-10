//! Gates do esqueleto de bandas único (B52.R26) e das curas B29.A2 / B29.A3 sobre ele.

use super::*;
use crate::spec::BrushSpec;
use crate::texture::{ImageMask, TexDabBasis, TextureKind, TextureMapping, TextureSettings};
use std::sync::atomic::Ordering::Relaxed;

const L0: [u8; 4] = [255, 255, 255, 255];
const L1: [u8; 4] = [0, 255, 255, 0];
const L2: [u8; 4] = [255, 0, 0, 255];

fn mask(lum: &[u8]) -> ImageMask<'_> {
    ImageMask {
        lum,
        width: 2,
        height: 2,
    }
}

fn shape_image_spec() -> BrushSpec {
    let mut spec = BrushSpec::default();
    spec.shape.kind = TextureKind::Image;
    spec
}

fn carimbos(lados: &[u32]) -> Vec<ColorStampMask> {
    let spec = shape_image_spec();
    let cols = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
    let ls = [L0, L1, L2];
    lados
        .iter()
        .enumerate()
        .map(|(i, &s)| {
            render_color_stamp_mask(
                &spec,
                &[mask(&ls[i % 3])],
                &[cols[i % 3]],
                &[],
                &[],
                None,
                None,
                s,
            )
        })
        .collect()
}

/// Um lote pequeno (rota serial) e um grande (rota em bandas), sobre uma tela `side`².
fn lotes() -> [(&'static str, u32, Vec<FusedDab>); 2] {
    let lote = |side: u32, rs: f32| {
        let k = side as f32 / 40.0;
        [
            ([18.3 * k, 19.7 * k], 9.0 * rs, 0.8),
            ([22.6 * k, 21.1 * k], 7.5 * rs, 1.0),
            ([15.0 * k, 24.4 * k], 11.2 * rs, 0.5),
            ([20.0 * k, 20.0 * k], 6.0 * rs, 0.35),
            ([2.4, 3.9], 8.0 * rs, 0.9),
        ]
        .iter()
        .map(|&(center, radius, coverage)| FusedDab {
            center,
            radius,
            coverage,
        })
        .collect()
    };
    [
        ("serial", 40, lote(40, 1.0)),
        ("bandas", 640, lote(640, 10.0)),
    ]
}

fn rect(r: Option<crate::dab::DirtyRect>) -> Option<(u32, u32, u32, u32)> {
    r.map(|r| (r.x, r.y, r.w, r.h))
}

/// O grão Tiled do papel (Clouds), com a base FIXA: a premissa da memória do grão.
fn spec_tiled() -> (BrushSpec, TexDabBasis) {
    let mut spec = shape_image_spec();
    spec.texture = TextureSettings {
        kind: TextureKind::Clouds,
        mapping: TextureMapping::Tiled,
        size: [40.0, 40.0],
        ..Default::default()
    };
    spec.grain_depth = 0.4;
    let basis = crate::texture::dab_basis(
        &spec.texture,
        &mut 7,
        [640.0, 640.0],
        spec.footprint_deform(),
    );
    (spec, basis)
}

fn dyn_lote(
    spec: &BrushSpec,
    gb: &TexDabBasis,
    raw: &[([f32; 2], f32, f32)],
    n: usize,
) -> Vec<DynDab> {
    let cols = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
    raw.iter()
        .map(|&(center, radius, coverage)| {
            let s = BrushSpec {
                radius_px: radius,
                ..*spec
            };
            let sb = crate::texture::shape_basis(
                &s.shape,
                &mut 3,
                [640.0, 640.0],
                s.footprint_deform(),
                crate::texture::ShapeFrame::Static,
            );
            DynDab {
                center,
                radius,
                spec: s,
                shape_basis: sb,
                grain_basis: Some(*gb),
                colors: cols[..n].to_vec(),
                color_shift: [0.0; 3],
                coverage,
            }
        })
        .collect()
}

/// Um traço de `n` dabs sobrepostos ao longo de x (passo `passo`, raio `r`).
fn traco(n: usize, x0: f32, y: f32, passo: f32, r: f32) -> Vec<([f32; 2], f32, f32)> {
    (0..n)
        .map(|i| ([x0 + passo * i as f32, y], r, 0.6))
        .collect()
}

/// ⭐ PARIDADE AO BIT (B52.R26): as três rotas em lote sobre o esqueleto único dão os mesmos mapas e o mesmo
/// retângulo que o código de antes (oráculo verbatim), na rota serial e na em bandas — e a do grão Tiled com a
/// memória dá o mesmo bit que a sem memória (B29.A3).
#[test]
fn o_esqueleto_de_bandas_unico_e_o_de_antes_ao_bit() {
    let stamps = carimbos(&[24, 24, 24]);
    let n = stamps.len();
    for (label, side, dabs) in lotes() {
        let len = (side * side) as usize;
        let (mut a, mut b): (Vec<Vec<u8>>, Vec<Vec<u8>>) = (
            (0..n).map(|_| vec![0; len]).collect(),
            (0..n).map(|_| vec![0; len]).collect(),
        );
        let ra = accumulate_color_stamps_fused_batch(&mut a, side, side, &dabs, &stamps);
        let rb = super::oraculo_batch::accumulate_color_stamps_fused_batch(
            &mut b, side, side, &dabs, &stamps,
        );
        assert_eq!(rect(ra), rect(rb), "{label}: fused rect");
        assert!(a == b, "{label}: fused maps");
        let (mut a, mut b): (Vec<Vec<u8>>, Vec<Vec<u8>>) = (
            (0..n).map(|_| vec![0; len * 4]).collect(),
            (0..n).map(|_| vec![0; len * 4]).collect(),
        );
        let ra = accumulate_color_stamps_rgba_batch(&mut a, side, side, &dabs, &stamps);
        let rb = super::oraculo_rgba_batch::accumulate_color_stamps_rgba_batch(
            &mut b, side, side, &dabs, &stamps,
        );
        assert_eq!(rect(ra), rect(rb), "{label}: rgba rect");
        assert!(a == b, "{label}: rgba maps");
    }
    let (spec, gb) = spec_tiled();
    let masks = [mask(&L0), mask(&L1), mask(&L2)];
    for (label, side, raw) in [
        ("serial", 640u32, traco(6, 40.0, 40.0, 3.0, 6.0)),
        ("bandas", 640, traco(24, 120.0, 300.0, 9.0, 70.0)),
    ] {
        let lote = dyn_lote(&spec, &gb, &raw, 3);
        let len4 = (side * side * 4) as usize;
        let (mut a, mut b): (Vec<Vec<u8>>, Vec<Vec<u8>>) = (
            (0..3).map(|_| vec![0; len4]).collect(),
            (0..3).map(|_| vec![0; len4]).collect(),
        );
        let ra =
            accumulate_shape_layers_rgba_batch(&mut a, side, side, &lote, &masks, &[], None, None);
        let rb = super::oraculo_batch::accumulate_shape_layers_rgba_batch(
            &mut b,
            side,
            side,
            &lote,
            &masks,
            &[],
            None,
            None,
        );
        assert_eq!(rect(ra), rect(rb), "{label}: dyn rect");
        assert!(
            a.iter().any(|m| m.iter().any(|&v| v > 0)),
            "{label}: pintou"
        );
        assert!(a == b, "{label}: dyn maps (memória do grão × sem memória)");
    }
}

/// B29.A2 (AU-11-1): um lote Per-Layer com carimbos de lados DIFERENTES pinta, em cada camada, o mesmo que o
/// carimbo dessa camada sozinho — na rota serial E na em bandas (antes a em bandas largava o lote em silêncio).
#[test]
fn lote_de_tamanhos_mistos_pinta_igual_nas_duas_rotas() {
    let mistos = carimbos(&[32, 16, 24]);
    for (label, side, dabs) in lotes() {
        let len = (side * side) as usize;
        let mut covs: Vec<Vec<u8>> = (0..3).map(|_| vec![0; len]).collect();
        let r = accumulate_color_stamps_fused_batch(&mut covs, side, side, &dabs, &mistos);
        let mut accs: Vec<Vec<u8>> = (0..3).map(|_| vec![0; len * 4]).collect();
        let r4 = accumulate_color_stamps_rgba_batch(&mut accs, side, side, &dabs, &mistos);
        assert!(r.is_some() && r4.is_some(), "{label}: o lote misto pintou");
        for (i, s) in mistos.iter().enumerate() {
            let so = std::slice::from_ref(s);
            let mut um = vec![vec![0u8; len]];
            let _ = accumulate_color_stamps_fused_batch(&mut um, side, side, &dabs, so);
            assert!(
                covs[i] == um[0],
                "{label}: camada {i} (lado {}) fused",
                s.size()
            );
            let mut um4 = vec![vec![0u8; len * 4]];
            let _ = accumulate_color_stamps_rgba_batch(&mut um4, side, side, &dabs, so);
            assert!(
                accs[i] == um4[0],
                "{label}: camada {i} (lado {}) rgba",
                s.size()
            );
            let mut serial = vec![0u8; len];
            for d in &dabs {
                let _ = accumulate_color_stamp_coverage(
                    &mut serial,
                    side,
                    side,
                    d.center,
                    d.radius,
                    s,
                    d.coverage,
                );
            }
            assert!(
                covs[i] == serial,
                "{label}: camada {i} = o kernel de uma camada"
            );
        }
    }
}

/// B29.A3 (AU-11-2): o grão Tiled da rota Per-Layer é amostrado ~uma vez por píxel do traço (a memória do Bug #84),
/// não uma vez por dab — nas duas rotas. Largura própria para o contador (os testes correm em paralelo).
#[test]
fn o_grao_tiled_da_rota_per_layer_e_amostrado_uma_vez_por_pixel() {
    let (spec, gb) = spec_tiled();
    let masks = [mask(&L0)];
    let (w, h) = (517u32, 400u32);
    super::bandas::sonda::LARGURA.store(w, Relaxed);
    for (label, raw) in [
        ("serial", traco(12, 30.0, 30.0, 1.0, 5.0)),
        ("bandas", traco(30, 100.0, 200.0, 6.0, 60.0)),
    ] {
        let lote = dyn_lote(&spec, &gb, &raw, 1);
        let mut accs = vec![vec![0u8; (w * h * 4) as usize]];
        super::bandas::sonda::AMOSTRAS.store(0, Relaxed);
        let _ = accumulate_shape_layers_rgba_batch(&mut accs, w, h, &lote, &masks, &[], None, None);
        let amostras = super::bandas::sonda::AMOSTRAS.load(Relaxed);
        // Píxeis distintos: a união das caixas dos dabs (≥ os que a silhueta toca).
        let mut visto = vec![false; (w * h) as usize];
        let mut visitas = 0u64;
        for &([cx, cy], r, _) in &raw {
            for py in ((cy - r).floor() as i64).max(0)..((cy + r).ceil() as i64 + 1).min(h as i64) {
                for px in
                    ((cx - r).floor() as i64).max(0)..((cx + r).ceil() as i64 + 1).min(w as i64)
                {
                    visto[(py * w as i64 + px) as usize] = true;
                    visitas += 1;
                }
            }
        }
        let distintos = visto.iter().filter(|&&v| v).count() as u64;
        eprintln!(
            "{label}: amostras {amostras} · píxeis distintos {distintos} · visitas {visitas}"
        );
        assert!(amostras > 0, "{label}: o grão foi amostrado");
        // O 1.º dab amostra sem memória (ela nasce quando a chave se repete); folga de um 2.º dab sem memória
        // se outro teste mexer no estado global entre dois dabs.
        assert!(
            amostras <= 3 * distintos,
            "{label}: {amostras} amostras para {distintos} píxeis distintos ({visitas} visitas)"
        );
    }
}
