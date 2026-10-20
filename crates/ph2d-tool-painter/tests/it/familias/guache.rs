//! **Família `guache`** — J4, catálogo H (guache): Gouache Flat. Escreva aqui as receitas e os testes da família; nada
//! fora deste ficheiro (`docs/Painter/Pinceis/04_BANCADA.md` §9).

use super::prelude::*;

/// O pigmento da Gouache Flat (doc 44 §3, lote 3).
const PIGMENTO_DA_GUACHE: f64 = 100.0;
/// A curva da pressão → opacidade (prompt 45, doc 45 §2): a rampa da cena seca contra o Rebelle — 0,42 (0,43).
const CURVA_DA_GUACHE: f64 = 2.0;

pub const WET: &[Receita] = &[
    // A guache FLAT do Rebelle (`gouacheflat`, Gouache › Gouache Flat): a ponta do Wet Flat, mais alta que larga, e o
    // DOBRO da tinta (cobertura 0,50 × 0,32). A Gouache Round não entra: no Rebelle ela é o nosso Round (18 §4.3).
    Receita {
        meio: "Watercolor",
        grupo: "Gouache",
        nome: "Gouache Flat",
        water: 1.0,
        opacity: 1.0,
        pigment: PIGMENTO_DA_GUACHE,
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
            ("pressureOpacityCurve", CURVA_DA_GUACHE),
        ],
        grao: None,
        achatado: Some((0.2, 90)),
        pincel: &[("dab_quina", "1.0")],
        nasce_de: None,
    },
];

pub const MEIOS: &[ReceitaMeio] = &[];

// As formas (prompt 44, doc 44 §3): a ponta reta e a pressão que não estreita e ESMAECE (prompt 45, doc 45).
#[rustfmt::skip]
pub const MIDIAS: &[Midia] = &[
    Midia { preset: "Gouache Flat", foto: "gouacheflat", raio: 19.5, metricas: &[(Cobertura, 0.504), (Ponta, 0.083), (Chato, 0.782), (RampaLargura, 0.872), (RampaCobertura, 0.43), (Largura, 39.0), (Pontas, 0.87)] },
];
