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
/// O papel de desenho do Chalk (mais grosso).
const PAPEL_DE_DESENHO: &str = r#"{"angle_deg": 0, "flow": false, "kind": "Noise", "lei": {"depth_jitter": 0.3, "depth_min": 0.0, "depth_pressure": true, "mode": 0}, "mapping": "Tiled", "offset": [0.0, 0.0], "params": [0.5, 0.5, 0.5, 0.5, 0.5, 0.5], "rake": false, "size": [80.0, 80.0], "stencil_angle_deg": 0, "stencil_offset": [0.0, 0.0], "stencil_size": [0.5, 0.5]}"#;
/// A nuvem manchada das aguadas (o «Dappled cloud» do CSP não foi copiado): `Clouds` procedural.
const NUVEM: &str = r#"{"angle_deg": 0, "flow": false, "kind": "Clouds", "mapping": "Tiled", "offset": [0.0, 0.0], "params": [0.5, 0.5, 0.5, 0.5, 0.5, 0.5], "rake": false, "size": [160.0, 160.0], "stencil_angle_deg": 0, "stencil_offset": [0.0, 0.0], "stencil_size": [0.5, 0.5]}"#;
/// A 2.ª ponta da Brush Pen: cerdas finas que rasgam a orla.
const CERDAS_DA_PENA: &str = r#"{"angulo": 0, "atraves": 0.15, "contagem": 4, "dureza": 0.6, "escala": 0.35, "espalhar": 0.25, "forma": {"angle_deg": 0, "flow": false, "kind": "None", "mapping": "ViewPlane", "offset": [0.0, 0.0], "params": [0.5, 0.5, 0.5, 0.5, 0.5, 0.5], "rake": false, "size": [1.0, 1.0], "stencil_angle_deg": 0, "stencil_offset": [0.0, 0.0], "stencil_size": [0.5, 0.5]}, "modo": 0, "passo": 0.1, "rotacao": 1.0}"#;

pub const MEIOS: &[ReceitaMeio] = &[
    // Crayon (CSP: 30 px, dureza 100, intervalo 20 contínuo, espessura 80, «Rough textured» dens 90, mistura
    // 60/100/15, HSV por dab 10/−10/−3, Size P·T(30), Flow P): a cera que apanha um pouco de cor e deixa o papel.
    ReceitaMeio {
        pincel: &[
            ("hardness", "0.6"),
            ("falloff", "\"Sharp\""),
            ("spacing", "0.1"),
            ("continuous_spacing", "true"),
            ("dab_flatten", "0.2"),
            ("texture", PAPEL_ASPERO),
            ("grain_depth", "0.8"),
            ("color_jitter_hue", "0.02"),
            ("color_jitter_val", "0.02"),
            (
                "mistura_opaca",
                r#"{"modo": "WetBlend", "paint_amount": 0.6, "paint_density": 1.0, "color_stretch": 0.15, "pigment": 0.0, "amount_pressure": 0.0, "density_pressure": 0.0}"#,
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
    // Pastel (CSP: 40 px, dureza 1, intervalo 10, «Rough textured» dens 60, mistura 40/40/40, sat. +15 por dab,
    // Size P·T(40), Flow P(15)): o pó macio que esbate na cor de baixo — no CSP um
    // pontilhado esparso quase sem cor, aqui um spray de grãos de 2 px.
    ReceitaMeio {
        pincel: &[
            ("hardness", "1.0"),
            ("falloff", "\"Sharp\""),
            ("spacing", "0.1"),
            ("jitter", "0.5"),
            ("spray_count", "20"),
            (
                "dab_jitter",
                r#"{"across": 0.0, "angle_per_stroke": 0.0, "count": 0.5, "flip_x": 0.0, "flip_y": 0.0, "opacity": 0.5, "particle": 0.05, "particle_px": 2.0, "roundness": 0.0, "subpixel_area": true}"#,
            ),
            ("texture", PAPEL_ASPERO),
            ("grain_depth", "1.0"),
            ("color_jitter_sat", "0.06"),
            (
                "mistura_opaca",
                r#"{"modo": "WetBlend", "paint_amount": 0.4, "paint_density": 0.4, "color_stretch": 0.4, "pigment": 0.0, "amount_pressure": 0.0, "density_pressure": 0.0}"#,
            ),
        ],
        mao: &[
            ("pressureSize", 0.6),
            ("pressureSizeCurve", 1.0),
            ("pressureOpacity", 0.85),
        ],
        size_px: Some(40.0),
        opacity: Some(0.6),
        ..ReceitaMeio::digital("Pastel", "Pastel", "Pastel")
    },
    // Chalk (CSP: 30 px, dureza 81, intervalo 10, espessura 50, «Drawing paper C» dens 80, mistura 41/100/15, HSV
    // 0/5/−10, Size P·T(50), Flow P·T(50)): o giz achatado e granulado.
    ReceitaMeio {
        pincel: &[
            ("hardness", "0.45"),
            ("falloff", "\"Sharp\""),
            ("spacing", "0.1"),
            ("dab_flatten", "0.5"),
            ("jitter", "0.02"),
            ("texture", PAPEL_DE_DESENHO),
            ("grain_depth", "0.5"),
            ("color_jitter_sat", "0.02"),
            ("color_jitter_val", "0.04"),
            (
                "mistura_opaca",
                r#"{"modo": "WetBlend", "paint_amount": 0.41, "paint_density": 1.0, "color_stretch": 0.15, "pigment": 0.0, "amount_pressure": 0.0, "density_pressure": 0.0}"#,
            ),
        ],
        mao: &[
            ("pressureSize", 0.5),
            ("pressureSizeCurve", 1.0),
            ("pressureOpacity", 0.5),
        ],
        size_px: Some(42.0),
        opacity: Some(0.8),
        ..ReceitaMeio::digital("Pastel", "Chalk", "Chalk")
    },
    // Pointillism (CSP: 80 px, intervalo 10, espessura 30 @45°, spray 30 desvio −100, mistura 50/50/15, HSV por
    // carimbo 30/20/10): pontos achatados com a cor do fundo e a variação de cor por ponto. O jitter de cor do motor
    // entra ANTES da mistura (na cor do pincel, a metade da tinta com Amount 0,5): a amplitude vai a dobrar do HSV
    // 30/20/10 lido como frações de 360/100/100 (matiz 0,04 — a 0,17 a folha de 08/11 virou arco-íris).
    ReceitaMeio {
        pincel: &[
            ("hardness", "0.9"),
            ("borda_px", "1.0"),
            ("spacing", "0.1"),
            ("continuous_spacing", "false"),
            ("dab_flatten", "0.7"),
            ("dab_angle_deg", "135"),
            ("spray_count", "3"),
            ("jitter", "0.45"),
            ("jitter_scale", "0.3"),
            ("color_jitter_hue", "0.04"),
            ("color_jitter_sat", "0.08"),
            ("color_jitter_val", "0.12"),
            (
                "dab_jitter",
                r#"{"across": 0.0, "angle_per_stroke": 0.0, "count": 0.3, "flip_x": 0.0, "flip_y": 0.0, "opacity": 0.0, "particle": 0.22, "roundness": 0.0}"#,
            ),
            (
                "mistura_opaca",
                r#"{"modo": "WetBlend", "paint_amount": 0.3, "paint_density": 0.5, "color_stretch": 0.15, "pigment": 0.0, "amount_pressure": 0.0, "density_pressure": 0.0}"#,
            ),
        ],
        mao: &[("pressureSize", 0.1)],
        size_px: Some(80.0),
        opacity: Some(1.0),
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
            ("flow", "0.5"),
            ("spray_count", "3"),
            ("jitter", "0.8"),
            ("jitter_scale", "0.4"),
            (
                "dab_jitter",
                r#"{"across": 2.0, "angle_per_stroke": 0.0, "count": 0.0, "flip_x": 0.0, "flip_y": 0.0, "opacity": 0.0, "particle": 1.0, "roundness": 0.3, "veu": 0.85, "veu_escala": 2.0, "veu_opacidade": 1.5}"#,
            ),
            ("texture", NUVEM),
            ("grain_depth", "0.15"),
            (
                "mistura_opaca",
                r#"{"modo": "WetBlend", "paint_amount": 0.7, "paint_density": 0.5, "color_stretch": 0.1, "pigment": 0.0, "amount_pressure": 0.0, "density_pressure": 0.0}"#,
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
            ("hardness", "0.6"),
            ("falloff", "\"Sharp\""),
            ("spacing", "0.1"),
            ("continuous_spacing", "false"),
            ("blend", "\"Multiply\""),
            ("spray_count", "3"),
            ("jitter", "1.0"),
            ("jitter_scale", "0.9"),
            ("jitter_rotate", "1.0"),
            ("color_jitter_hue", "0.04"),
            ("texture", NUVEM),
            ("grain_depth", "0.9"),
            (
                "dab_jitter",
                r#"{"across": 2.0, "angle_per_stroke": 0.0, "count": 0.6, "flip_x": 0.0, "flip_y": 0.0, "opacity": 0.5, "particle": 0.4, "roundness": 0.4}"#,
            ),
        ],
        mao: &[("pressureSize", 0.5), ("pressureOpacity", 0.6)],
        size_px: Some(300.0),
        opacity: Some(0.65),
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
            ("dual", CERDAS_DA_PENA),
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
                r#"{"modo": "WetBlend", "paint_amount": 0.0, "paint_density": 1.0, "color_stretch": 0.7, "pigment": 0.0, "amount_pressure": 0.0, "density_pressure": 0.0}"#,
            ),
        ],
        mao: &[("pressureSize", 0.5), ("pressureOpacity", 0.6)],
        size_px: Some(30.0),
        opacity: Some(1.0),
        ..ReceitaMeio::digital("Blend", "Finger", "Finger Tip")
    },
];

pub const MIDIAS: &[Midia] = &[];
