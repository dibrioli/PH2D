//! **Família `aguarela`** — J4, catálogo I (aguarela, sumi-e): Wet Wash, Wet Dry Brush, Wet Drip, Wet Liner, Wet Flat.
//! Escreva aqui as receitas e os testes da família; nada fora deste ficheiro (`docs/Painter/Pinceis/04_BANCADA.md` §9).
//! ⛔ O Wet Round NÃO é desta família: é o pincel sem preset e o controlo dos portões (`receitas_de_fabrica::RECEITAS`).

use super::prelude::*;

/// O pigmento do Wet Flat (doc 44 §3, lote 3).
const PIGMENTO_DO_FLAT: f64 = 50.0;
/// A curva da pressão → opacidade (prompt 45, doc 45 §2): a rampa da cena seca contra o Rebelle — 0,41 (0,42).
const CURVA_DO_FLAT: f64 = 1.75;

pub const WET: &[Receita] = &[
    Receita {
        meio: "Watercolor",
        grupo: "Fluid",
        nome: "Wet Wash",
        water: 0.4,
        opacity: 1.0,
        pigment: 21.0,
        pickup: 0.6,
        gate: 0.3,
        evaporation: 0.7,
        edge: 20.0,
        tilt: DIREITA,
        bristle_size: 3.0,
        bristle_count: 3000.0,
        hardness: 0.0,
        size_px: None,
        extras: &[
            ("pressureSizeCurve", 0.35),
            ("pressureOpacity", 1.0),
            ("pressureOpacityCurve", 1.0),
        ],
        grao: None,
        achatado: None,
        pincel: &[],
        nasce_de: None,
    },
    Receita {
        meio: "Watercolor",
        grupo: "Fluid",
        nome: "Wet Dry Brush",
        water: 0.2,
        opacity: 1.0,
        pigment: 300.0,
        pickup: 1.6,
        gate: 0.4,
        evaporation: 1.5,
        edge: 56.25,
        tilt: DIREITA,
        bristle_size: 1.5,
        bristle_count: 1500.0,
        hardness: 0.0,
        size_px: None,
        extras: &[],
        grao: None,
        achatado: None,
        pincel: &[],
        nasce_de: None,
    },
    Receita {
        meio: "Watercolor",
        grupo: "Fluid",
        nome: "Wet Drip",
        water: 1.0,
        opacity: 1.0,
        pigment: 63.0,
        pickup: 0.8,
        gate: 0.2,
        evaporation: 0.6,
        edge: 20.0,
        tilt: Tilt {
            on: true,
            ring: 8,
            spoke: 3,
        },
        bristle_size: 2.0,
        bristle_count: 2000.0,
        hardness: 0.0,
        size_px: None,
        extras: &[],
        grao: None,
        achatado: None,
        pincel: &[],
        nasce_de: None,
    },
    // ⭐ O 1.º da Fase C (prompt 17, 05 §4b «Presets de todas as mídias»): o Liner do Rebelle
    // (`Watercolor/Liner`: duro, carregado, papel 0,10 — o grão quase não fura, 28,86 px). Medido contra
    // `fixtures/rebelle/wcliner.png` (cobertura 0,65 · textura 1,7 %) à largura 38; o Wet Round dá 0,19 · 8,1.
    Receita {
        meio: "Watercolor",
        grupo: "Fluid",
        nome: "Wet Liner",
        water: 0.5,
        opacity: 1.0,
        pigment: 100.0,
        pickup: 1.0,
        gate: 0.05,
        evaporation: 1.0,
        edge: 100.0,
        tilt: DIREITA,
        bristle_size: 2.0,
        bristle_count: 2000.0,
        hardness: 0.9,
        size_px: Some(29.0),
        extras: &[],
        grao: None,
        achatado: None,
        pincel: &[],
        nasce_de: None,
    },
    // ⭐ As FORMAS (prompt 44, doc 44): o Wet Round com a ponta RETA (quina 1, dura 0,9 — a ponta macia lê-se
    // redonda), um pouco mais alta que larga e FIXA na tela (o rato do Rebelle não roda a ponta), e a pressão
    // que ESMAECE em vez de estreitar. Watercolor › Flat do Rebelle (`wcflat`).
    Receita {
        meio: "Watercolor",
        grupo: "Fluid",
        nome: "Wet Flat",
        water: 1.0,
        opacity: 1.0,
        pigment: PIGMENTO_DO_FLAT,
        pickup: 1.0,
        gate: 0.4,
        evaporation: 1.0,
        edge: 12.0,
        tilt: DIREITA,
        bristle_size: 2.0,
        bristle_count: 2000.0,
        hardness: 0.9,
        size_px: None,
        extras: &[
            ("pressureSize", 0.3),
            ("pressureOpacity", 1.0),
            ("pressureOpacityCurve", CURVA_DO_FLAT),
        ],
        grao: None,
        achatado: Some((0.12, 90)),
        pincel: &[("dab_quina", "1.0")],
        nasce_de: None,
    },
];

pub const MEIOS: &[ReceitaMeio] = &[];

// Medido contra `fixtures/rebelle/wcliner.png` à largura dela; as formas (prompt 44, doc 44 §3) e a pressão que
// ESMAECE (prompt 45, doc 45).
#[rustfmt::skip]
pub const MIDIAS: &[Midia] = &[
    Midia { preset: "Wet Liner", foto: "wcliner", raio: 19.0, metricas: &[(Cobertura, 0.653), (Textura, 1.708)] },
    Midia { preset: "Wet Flat", foto: "wcflat", raio: 17.0, metricas: &[(Cobertura, 0.315), (Ponta, 0.096), (RampaLargura, 0.829), (RampaCobertura, 0.42), (Largura, 34.0), (Pontas, 0.9)] },
];
