//! **Família `csp_tinta_grossa`** — o lote A do estudo CSP (`docs/Painter/ESTUDO_CLIP_STUDIO_PINCEIS_2026-11-07.md`
//! §3, §6 lote 2): o grupo «Thick paint» do Clip Studio no Digital, com o Color mixing **Wet blend**
//! (`docs/Painter/CSP_COMO_MISTURA_2026-11-08.md`). Números lidos (Amount/Density/Stretch, tamanho, dureza, passo);
//! nenhuma ponta nem textura da Celsys copiada: as cerdas são o Shape `Bristles` do motor e os papéis os nossos
//! procedurais. Os pincéis vivem em `Digital/Oil/Thick paint` (a categoria Oil do navegador; uma pasta nova pedia uma
//! categoria nova no app).
//!
//! O Spacing de fábrica fica 0,1 + Continuous (a regra da fábrica); com a mistura ligada o percurso anda a 3 %
//! (`BrushSpec::spacing_do_traco`). A pressão: o rato do oráculo é pressão 1, as curvas `P(n)` do CSP entram como o
//! peso da pressão no Amount/Density (`1 − n/100`).

use super::prelude::*;

pub const WET: &[Receita] = &[];

/// O Color mixing de um pincel: `(Amount, Density, Stretch, peso da pressão no Amount, na Density)`.
macro_rules! mistura {
    ($a:literal, $d:literal, $s:literal, $ap:literal, $dp:literal) => {
        concat!(
            r#"{"modo": "WetBlend", "paint_amount": "#,
            $a,
            r#", "paint_density": "#,
            $d,
            r#", "color_stretch": "#,
            $s,
            r#", "pigment": 0.0, "amount_pressure": "#,
            $ap,
            r#", "density_pressure": "#,
            $dp,
            "}"
        )
    };
}

/// As cerdas do Shape `Bristles`: params `[Count, Strength, Size, Felt]` na unidade do slider
/// (`ph2d_wet_paint::brush::BRISTLE_UNIT_KNOBS`). Com o Continuous Spacing cada fio arrasta-se ao longo do traço
/// (as estrias do CSP); a silhueta vai ao tecto da cobertura, então cobre.
macro_rules! cerdas {
    ($c:literal, $s:literal, $z:literal, $f:literal) => {
        concat!(
            r#"{"angle_deg": 0, "flow": false, "kind": "Bristles", "mapping": "ViewPlane", "offset": [0.0, 0.0], "params": ["#,
            $c, ", ", $s, ", ", $z, ", ", $f,
            r#", 0.5, 0.5], "rake": false, "size": [1.0, 1.0], "stencil_angle_deg": 0, "stencil_offset": [0.0, 0.0], "stencil_size": [0.5, 0.5]}"#
        )
    };
}
/// O papel do guache (CSP «Paper», densidade 85–100): o nosso papel prensado a frio, preso à tela.
const PAPEL: &str = r#"{"angle_deg": 0, "flow": false, "kind": "PaperCold", "mapping": "Tiled", "offset": [0.0, 0.0], "params": [0.5, 0.5, 0.5, 0.5, 0.5, 0.5], "rake": false, "size": [128.0, 128.0], "stencil_angle_deg": 0, "stencil_offset": [0.0, 0.0], "stencil_size": [0.5, 0.5]}"#;

// Revisão v3 (08/11, CSP corrido em 3 fundos, `docs/Painter/REVISAO_CSP_V3_2026-11-08.md`): Amount/Density, dureza,
// tamanho do Gouache blender (80) e as cerdas do Thick oil (chapado no CSP) afinados pela folha.
pub const MEIOS: &[ReceitaMeio] = &[
    // Round mixing brush (CSP 50 · dureza 100 · 10 % contínuo · 80/50/100, Alpha P(90)): o redondo que apanha e arrasta.
    ReceitaMeio {
        pincel: &[
            ("hardness", "0.5"),
            ("spacing", "0.1"),
            ("continuous_spacing", "true"),
            ("stabilizer", "0.02"),
            ("mistura_opaca", mistura!(0.25, 0.8, 1.0, 0.0, 0.1)),
        ],
        mao: &[("pressureSize", 0.8), ("pressureSizeCurve", 1.0)],
        size_px: Some(50.0),
        opacity: Some(1.0),
        ..ReceitaMeio::digital("Oil", "Thick paint", "Round Mixing")
    },
    // Gouache (CSP 80 · cerda @45° · Paper 100 · 70/45/15, Mix/Alpha P(10)): cobre com estrias e esquece depressa.
    ReceitaMeio {
        pincel: &[
            ("hardness", "0.5"),
            ("spacing", "0.1"),
            ("continuous_spacing", "true"),
            ("dab_angle_deg", "45"),
            ("shape", cerdas!(1.0, 1.0, 0.4, 1.0)),
            ("mistura_opaca", mistura!(0.35, 0.7, 0.3, 0.9, 0.9)),
        ],
        mao: &[("pressureSize", 0.9), ("pressureSizeCurve", 1.0)],
        size_px: Some(80.0),
        opacity: Some(1.0),
        ..ReceitaMeio::digital("Oil", "Thick paint", "Gouache")
    },
    // Dry Gouache (CSP 80 · ponta seca @45° · Paper 100 · 80/80/0, flow 50): pincel seco que racha sobre o papel.
    ReceitaMeio {
        pincel: &[
            ("hardness", "0.9"),
            ("spacing", "0.1"),
            ("continuous_spacing", "true"),
            ("dab_angle_deg", "45"),
            ("shape", cerdas!(0.5, 1.0, 0.5, 0.0)),
            ("flow", "0.5"),
            ("mistura_opaca", mistura!(0.35, 0.8, 0.3, 0.9, 0.5)),
        ],
        mao: &[("pressureSize", 0.7), ("pressureSizeCurve", 1.0)],
        size_px: Some(80.0),
        opacity: Some(1.0),
        ..ReceitaMeio::digital("Oil", "Thick paint", "Dry Gouache")
    },
    // Thin Gouache (CSP 20 · @63° · Paper 85 · 100/55/70): guache diluído — só a cor do pincel, que vela e arrasta.
    ReceitaMeio {
        pincel: &[
            ("hardness", "0.5"),
            ("spacing", "0.1"),
            ("continuous_spacing", "true"),
            ("dab_angle_deg", "63"),
            ("texture", PAPEL),
            ("grain_depth", "0.3"),
            ("mistura_opaca", mistura!(0.3, 0.55, 0.7, 0.9, 0.9)),
        ],
        mao: &[("pressureSize", 0.9), ("pressureSizeCurve", 1.0)],
        size_px: Some(20.0),
        opacity: Some(1.0),
        ..ReceitaMeio::digital("Oil", "Thick paint", "Thin Gouache")
    },
    // Gouache blender (CSP Mix 0/0, stretch 20, papel 20, intervalo 20): só arrasta a tinta, não pinta o vazio;
    // amostra todas as camadas visíveis (o esbatedor numa camada nova, dono 08/11)
    // (Amount 0); a Density é a força do arrasto (no CSP «0» com o modo de aguarela, que o nosso não tem).
    ReceitaMeio {
        pincel: &[
            ("hardness", "0.7"),
            ("spacing", "0.1"),
            ("continuous_spacing", "true"),
            ("texture", PAPEL),
            ("grain_depth", "0.2"),
            (
                "mistura_opaca",
                r#"{"modo": "WetBlend", "paint_amount": 0.0, "paint_density": 0.7, "color_stretch": 0.8, "pigment": 0.0, "amount_pressure": 0.0, "density_pressure": 0.5, "sample": "AllVisibleLayers"}"#,
            ),
        ],
        mao: &[("pressureSize", 0.5), ("pressureSizeCurve", 1.0)],
        size_px: Some(80.0),
        opacity: Some(1.0),
        ..ReceitaMeio::digital("Oil", "Thick paint", "Gouache Blender")
    },
    // Oil paint (CSP 80 · cerda @115° · Canvas 02 densidade 12 · 61/81/87 · HSV por dab 3/50/16): arrasta longe.
    ReceitaMeio {
        pincel: &[
            ("hardness", "0.9"),
            ("spacing", "0.1"),
            ("continuous_spacing", "true"),
            ("dab_angle_deg", "115"),
            ("shape", cerdas!(1.0, 0.8, 0.5, 0.6)),
            ("color_jitter_enabled", "true"),
            ("color_jitter_val", "0.02"),
            ("mistura_opaca", mistura!(0.35, 0.7, 0.87, 0.9, 0.9)),
        ],
        mao: &[("pressureSize", 0.9), ("pressureSizeCurve", 1.0)],
        size_px: Some(80.0),
        opacity: Some(1.0),
        ..ReceitaMeio::digital("Oil", "Thick paint", "Oil Paint")
    },
    // Paint and apply (CSP 40 · dureza 40 · 80/100/50 · entrada/saída 20, Flow P(40)): redondo macio que cobre.
    ReceitaMeio {
        pincel: &[
            ("hardness", "0.0"),
            ("falloff", "\"Smooth\""),
            ("spacing", "0.1"),
            ("continuous_spacing", "true"),
            ("mistura_opaca", mistura!(0.3, 0.5, 0.5, 0.5, 0.0)),
        ],
        mao: &[
            ("pressureSize", 0.0),
            ("pressureOpacity", 0.6),
            ("pressureOpacityCurve", 1.0),
        ],
        size_px: Some(40.0),
        opacity: Some(1.0),
        ..ReceitaMeio::digital("Oil", "Thick paint", "Paint And Apply")
    },
    // Thick oil paint (CSP 60 · 2 % contínuo · @70° espessura 60 · Canvas 02 densidade 50 · 85/50/60 · ponta dupla ·
    // valor −3): óleo grosso que cobre com a cor do pincel, a 2.ª ponta rasga a orla.
    ReceitaMeio {
        pincel: &[
            ("hardness", "0.7"),
            ("spacing", "0.1"),
            ("continuous_spacing", "true"),
            ("dab_angle_deg", "70"),
            ("dab_flatten", "0.4"),
            ("color_jitter_enabled", "true"),
            ("color_jitter_val", "0.03"),
            ("mistura_opaca", mistura!(0.45, 0.9, 0.6, 0.7, 0.0)),
        ],
        mao: &[("pressureSize", 0.7), ("pressureSizeCurve", 1.0)],
        size_px: Some(60.0),
        opacity: Some(1.0),
        ..ReceitaMeio::digital("Oil", "Thick paint", "Thick Oil Paint")
    },
];

pub const MIDIAS: &[Midia] = &[];
