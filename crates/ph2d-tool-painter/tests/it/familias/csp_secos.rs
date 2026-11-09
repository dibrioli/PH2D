//! **Família `csp_secos`** — lote B do estudo CSP (08/11, `docs/Painter/ESTUDO_CLIP_STUDIO_PINCEIS_2026-11-07.md` §3 e
//! `docs/Painter/CSP_COMO_MISTURA_2026-11-08.md`): os pincéis secos com mistura leve (Crayon, Pastel, Chalk), o
//! Pointillism, as aguadas que multiplicam (Wet Wash Mix, Watercolor Splash), a Brush Pen e o Finger Tip. Números de
//! fábrica do CSP LIDOS (`EditImageTool.todb`); nenhuma ponta ou textura da Celsys copiada — discos achatados, a 2.ª
//! ponta e o grão procedural tomam o lugar das imagens. A mistura é o Wet blend (`mistura_opaca`, Amount/Density/
//! Stretch do CSP em `0..1`). Folha CSP × PH2D: `docs/Painter/handoffs/material_2026-11-08/csp_lote_B_secos.png`.

use super::prelude::*;

pub const WET: &[Receita] = &[];

/// O papel áspero NOSSO dos secos (o «Rough textured» do CSP não foi copiado): `Noise` procedural preso à tela.
const PAPEL_ASPERO: &str = r#"{"angle_deg": 0, "flow": false, "kind": "Noise", "lei": {"depth_jitter": 0.5, "depth_min": 0.0, "depth_pressure": true, "mode": 0}, "mapping": "Tiled", "offset": [0.0, 0.0], "params": [0.5, 0.5, 0.5, 0.5, 0.5, 0.5], "rake": false, "size": [48.0, 48.0], "stencil_angle_deg": 0, "stencil_offset": [0.0, 0.0], "stencil_size": [0.5, 0.5]}"#;
/// O papel do Pastel (K1): `Noise` 32 px de 3 oitavas, contraste 0,4, lido pelo Height com maciez e ponta afundada.
const PAPEL_DO_PASTEL: &str = r#"{"angle_deg": 0, "flow": false, "kind": "Noise", "lei": {"depth_jitter": 0.8, "depth_min": 0.0, "depth_pressure": true, "mode": 4, "softness": 0.3, "tip_depth": true, "fill": 0.5}, "mapping": "Tiled", "offset": [0.0, 0.0], "params": [0.4, 0.5, 0.3, 0.5, 0.5, 0.5], "rake": false, "size": [32.0, 32.0], "stencil_angle_deg": 0, "stencil_offset": [0.0, 0.0], "stencil_size": [0.5, 0.5]}"#;
/// O papel de desenho do Chalk (mais grosso).
const PAPEL_DE_DESENHO: &str = r#"{"angle_deg": 0, "flow": false, "kind": "Noise", "lei": {"depth_jitter": 0.3, "depth_min": 0.0, "depth_pressure": true, "mode": 0}, "mapping": "Tiled", "offset": [0.0, 0.0], "params": [0.5, 0.5, 0.5, 0.5, 0.5, 0.5], "rake": false, "size": [80.0, 80.0], "stencil_angle_deg": 0, "stencil_offset": [0.0, 0.0], "stencil_size": [0.5, 0.5]}"#;
/// A nuvem manchada das aguadas (o «Dappled cloud» do CSP não foi copiado): `Clouds` procedural.
const NUVEM: &str = r#"{"angle_deg": 0, "flow": false, "kind": "Clouds", "mapping": "Tiled", "offset": [0.0, 0.0], "params": [0.5, 0.5, 0.5, 0.5, 0.5, 0.5], "rake": false, "size": [160.0, 160.0], "stencil_angle_deg": 0, "stencil_offset": [0.0, 0.0], "stencil_size": [0.5, 0.5]}"#;

// Revisão v3 (08/11, CSP corrido em 3 fundos, `docs/Painter/REVISAO_CSP_V3_2026-11-08.md`): grão, dureza, Amount,
// espalhamento da Splash, o tubo do Wet wash (véu 4,5×), a Brush Pen sem 2.ª ponta e o Finger tip curto.
pub const MEIOS: &[ReceitaMeio] = &[
    // Crayon (CSP: 30 px, dureza 100, intervalo 20 contínuo, espessura 80, «Rough textured» dens 90, mistura
    // 60/100/15, HSV por dab 10/−10/−3, Size P·T(30), Flow P): a cera que apanha um pouco de cor e deixa o papel.
    ReceitaMeio {
        pincel: &[
            ("hardness", "0.9"),
            ("falloff", "\"Sharp\""),
            ("spacing", "0.1"),
            ("continuous_spacing", "true"),
            ("dab_flatten", "0.2"),
            ("texture", PAPEL_ASPERO),
            ("grain_depth", "0.3"),
            ("color_jitter_hue", "0.02"),
            ("color_jitter_val", "0.02"),
            (
                "mistura_opaca",
                r#"{"modo": "WetBlend", "paint_amount": 0.35, "paint_density": 1.0, "color_stretch": 0.15, "pigment": 0.0, "amount_pressure": 0.0, "density_pressure": 0.0}"#,
            ),
        ],
        mao: &[
            ("pressureSize", 0.7),
            ("pressureSizeCurve", 1.0),
            ("pressureOpacity", 1.0),
        ],
        size_px: Some(30.0),
        opacity: Some(1.0),
        ..ReceitaMeio::digital("Pastel", "Crayon", "Crayon")
    },
    // Pastel — receita K1 (09/11, medida contra o Krita Chalk Soft; aprovada pelo dono): sem spray, o papel
    // decide a cobertura pelo Height com borda macia e ponta que afunda
    // (`docs/Painter/handoffs/material_2026-11-09_pastel/README.md` FASE 3).
    ReceitaMeio {
        pincel: &[
            ("hardness", "0.75"),
            ("falloff", "\"Smooth\""),
            ("spacing", "0.1"),
            ("texture", PAPEL_DO_PASTEL),
            ("grain_depth", "1.0"),
            ("color_jitter_sat", "0.06"),
            (
                "mistura_opaca",
                r#"{"modo": "WetBlend", "paint_amount": 0.5, "paint_density": 0.7, "color_stretch": 0.4, "pigment": 0.0, "amount_pressure": 0.0, "density_pressure": 0.0}"#,
            ),
        ],
        mao: &[
            ("pressureSize", 0.6),
            ("pressureSizeCurve", 1.0),
            ("pressureOpacity", 0.3),
        ],
        size_px: Some(40.0),
        opacity: Some(1.0),
        ..ReceitaMeio::digital("Pastel", "Pastel", "Pastel")
    },
    // Chalk (CSP: 30 px, dureza 81, intervalo 10, espessura 50, «Drawing paper C» dens 80, mistura 41/100/15, HSV
    // 0/5/−10, Size P·T(50), Flow P·T(50)): o giz achatado e granulado.
    ReceitaMeio {
        pincel: &[
            ("hardness", "0.9"),
            ("falloff", "\"Sharp\""),
            ("spacing", "0.1"),
            ("dab_flatten", "0.2"),
            ("jitter", "0.02"),
            ("texture", PAPEL_DE_DESENHO),
            ("grain_depth", "0.3"),
            ("color_jitter_sat", "0.02"),
            ("color_jitter_val", "0.04"),
            (
                "mistura_opaca",
                r#"{"modo": "WetBlend", "paint_amount": 0.3, "paint_density": 1.0, "color_stretch": 0.15, "pigment": 0.0, "amount_pressure": 0.0, "density_pressure": 0.0}"#,
            ),
        ],
        mao: &[
            ("pressureSize", 0.5),
            ("pressureSizeCurve", 1.0),
            ("pressureOpacity", 0.5),
        ],
        size_px: Some(42.0),
        opacity: Some(1.0),
        ..ReceitaMeio::digital("Pastel", "Chalk", "Chalk")
    },
    // Pointillism (CSP: 80 px, intervalo 10, espessura 30 @45°, spray 30 desvio −100, mistura 50/50/15, HSV por
    // carimbo 30/20/10): pontos achatados com a cor do fundo e a variação de cor por ponto. O jitter de cor do motor
    // entra ANTES da mistura (na cor do pincel, a metade da tinta com Amount 0,5): a amplitude vai a dobrar do HSV
    // 30/20/10 lido como frações de 360/100/100 (matiz 0,04 — a 0,17 a folha de 08/11 virou arco-íris).
    // Fixed dab size (dono 08/11): o ponto de 0,33 × 80 = 26,4 px fica em todo Size e a contagem cresce com a área
    // (referência 80): no Size de fábrica a nuvem é a de antes.
    ReceitaMeio {
        pincel: &[
            ("hardness", "0.5"),
            ("borda_px", "1.0"),
            ("spacing", "0.1"),
            ("continuous_spacing", "false"),
            ("dab_flatten", "0.75"),
            ("dab_angle_deg", "0"),
            ("spray_count", "3"),
            ("jitter", "0.45"),
            ("jitter_scale", "0.3"),
            ("color_jitter_hue", "0.04"),
            ("color_jitter_sat", "0.08"),
            ("color_jitter_val", "0.12"),
            (
                "dab_jitter",
                r#"{"across": 0.0, "angle_per_stroke": 0.0, "count": 0.3, "flip_x": 0.0, "flip_y": 0.0, "opacity": 0.0, "particle": 0.33, "particle_px": 26.4, "particle_ref_px": 80.0, "roundness": 0.0}"#,
            ),
            (
                "mistura_opaca",
                r#"{"modo": "WetBlend", "paint_amount": 0.3, "paint_density": 1.0, "color_stretch": 0.15, "pigment": 0.0, "amount_pressure": 0.0, "density_pressure": 0.0}"#,
            ),
        ],
        mao: &[("pressureSize", 0.1), ("dabRotation", 1.0)],
        size_px: Some(80.0),
        opacity: Some(0.75),
        ..ReceitaMeio::digital("Texture", "Pointillism", "Pointillism")
    },
    // Wet wash (CSP: 30 px, intervalo ~1 contínuo, «Dappled cloud», mistura 70/50/10, spray 120 dens 4 desvio −50,
    // ponta dupla com spray, Opac P(10), composição Multiplicar): um véu largo e liso, maior que o tamanho nominal,
    // que multiplica (o traço é uma camada: não escurece a si mesmo). O nome «Wet Wash» é do Wet Painter. A largura
    // de ~3× o nominal (folha CSP de 08/11) vem do spray com véu 2× e espalhamento de 2 diâmetros.
    ReceitaMeio {
        pincel: &[
            ("hardness", "0.7"),
            ("falloff", "\"Sharp\""),
            ("spacing", "0.03"),
            ("continuous_spacing", "true"),
            ("blend", "\"Multiply\""),
            ("blend_sobre_vazio", "true"),
            ("flow", "0.5"),
            ("spray_count", "2"),
            ("jitter", "0.0"),
            ("jitter_scale", "0.4"),
            (
                "dab_jitter",
                r#"{"across": 0.0, "angle_per_stroke": 0.0, "count": 0.0, "flip_x": 0.0, "flip_y": 0.0, "opacity": 0.0, "particle": 1.0, "roundness": 1.0, "veu": 1.0, "veu_escala": 4.5, "veu_opacidade": 1.0}"#,
            ),
            ("texture", NUVEM),
            ("grain_depth", "0.15"),
            (
                "mistura_opaca",
                r#"{"modo": "WetBlend", "paint_amount": 0.3, "paint_density": 0.5, "color_stretch": 0.1, "pigment": 0.0, "amount_pressure": 0.0, "density_pressure": 0.0}"#,
            ),
        ],
        mao: &[("pressureSize", 0.0), ("pressureOpacity", 0.9)],
        size_px: Some(30.0),
        opacity: Some(1.0),
        ..ReceitaMeio::digital("Watercolor", "Wash", "Wet Wash Mix")
    },
    // Watercolor splash (CSP: 300 px, ~9,5 contínuo, spray 325, ponta dupla com spray 200, matiz ±15 por carimbo,
    // «Sand» dens 15, Opac P·R(40), composição Multiplicar): salpicos dispersos que multiplicam, de vários tamanhos;
    // a mancha irregular é a nuvem procedural funda sobre partículas de 0,4 do pincel (sem ponta de imagem).
    ReceitaMeio {
        pincel: &[
            ("hardness", "0.7"),
            ("falloff", "\"Sharp\""),
            ("spacing", "0.2"),
            ("continuous_spacing", "false"),
            ("blend", "\"Multiply\""),
            ("blend_sobre_vazio", "true"),
            ("spray_count", "2"),
            ("jitter", "0.1"),
            ("jitter_scale", "0.5"),
            ("jitter_rotate", "1.0"),
            ("color_jitter_hue", "0.0"),
            ("shape", PONTA_IMG),
            (
                "hose",
                r#"{"n": 3, "mode": 0, "pesos": [1, 1, 1, 1, 1, 1, 1, 1]}"#,
            ),
            ("texture", NUVEM),
            ("grain_depth", "0.3"),
            (
                "dab_jitter",
                r#"{"across": 0.8, "angle_per_stroke": 0.0, "count": 0.3, "flip_x": 0.5, "flip_y": 0.5, "opacity": 0.3, "particle": 0.5, "roundness": 0.0}"#,
            ),
        ],
        mao: &[("pressureSize", 0.5), ("pressureOpacity", 0.6)],
        size_px: Some(300.0),
        opacity: Some(1.0),
        texturas: &[
            ("shape", img_salpico_1),
            ("shape_2", img_salpico_2),
            ("shape_3", img_salpico_3),
        ],
        ..ReceitaMeio::digital("Watercolor", "Splash", "Watercolor Splash")
    },
    // Brush pen (CSP: 70 px, intervalo 2 contínuo, ponta dupla de cerdas, «Dappled cloud» dens 100, estab. 15,
    // Size P·V, Flow P·V(50)): o pincel de nanquim que afina com a pressão e rasga a orla em cerdas.
    ReceitaMeio {
        pincel: &[
            ("hardness", "0.9"),
            ("borda_px", "1.0"),
            ("spacing", "0.02"),
            ("continuous_spacing", "true"),
            ("stabilizer", "0.15"),
            ("texture", NUVEM),
            ("grain_depth", "0.25"),
        ],
        mao: &[
            ("pressureSize", 0.9),
            ("pressureSizeCurve", 1.0),
            ("pressureOpacity", 0.5),
            ("speedSize", 0.05),
        ],
        size_px: Some(70.0),
        opacity: Some(1.0),
        ..ReceitaMeio::digital("Ink", "Brush", "Brush Pen")
    },
    // Finger tip (CSP: 30 px, dureza 50, intervalo 3, mistura 0/0/70 Running color, estab. 4, Size P): o dedo que só
    // arrasta a tinta (Amount 0: não pinta o vazio).
    ReceitaMeio {
        pincel: &[
            ("hardness", "0.5"),
            ("spacing", "0.03"),
            ("stabilizer", "0.04"),
            (
                "mistura_opaca",
                r#"{"modo": "WetBlend", "paint_amount": 0.0, "paint_density": 1.0, "color_stretch": 0.8, "pigment": 0.0, "amount_pressure": 0.0, "density_pressure": 0.0, "sample": "AllVisibleLayers"}"#,
            ),
        ],
        mao: &[("pressureSize", 0.5), ("pressureOpacity", 0.6)],
        size_px: Some(30.0),
        opacity: Some(1.0),
        ..ReceitaMeio::digital("Blend", "Finger", "Finger Tip")
    },
];

pub const MIDIAS: &[Midia] = &[];

// As imagens CC0 da Watercolor Splash (fonte e preparo: `assets/brush-images/watercolor/FONTES.md`).
/// A ponta em imagem (a mancha inteira no dab).
const PONTA_IMG: &str = r#"{"angle_deg": 0, "flow": false, "kind": "Image", "mapping": "ViewPlane", "offset": [0.0, 0.0], "params": [0.5, 0.5, 0.5, 0.5, 0.5, 0.5], "rake": false, "size": [1.0, 1.0], "stencil_angle_deg": 0, "stencil_offset": [0.0, 0.0], "stencil_size": [0.5, 0.5]}"#;

fn imagem_cc0(png: &[u8]) -> Vec<u8> {
    let (lum, w, h) = ph2d_brush_presets::ler_textura(png).expect("imagem CC0");
    ph2d_brush_presets::escrever_textura(&lum, w, h)
}
fn img_salpico_1() -> Vec<u8> {
    imagem_cc0(include_bytes!(
        "../../../../../assets/brush-images/watercolor/splash_1.png"
    ))
}
fn img_salpico_2() -> Vec<u8> {
    imagem_cc0(include_bytes!(
        "../../../../../assets/brush-images/watercolor/splash_2.png"
    ))
}
fn img_salpico_3() -> Vec<u8> {
    imagem_cc0(include_bytes!(
        "../../../../../assets/brush-images/watercolor/splash_3.png"
    ))
}
