//! **Família `tintas`** — J3, catálogo C (as tintas do Wet Paint): Dip Pen, Ink Bullet, Ink Brush, Fountain Pen, Reed
//! Pen, Gloop Pen, Ink Bleed, Gesinski Ink, Wood Stick. Escreva aqui as receitas e os testes da família; nada fora
//! deste ficheiro (`docs/Painter/Pinceis/04_BANCADA.md` §9).

use super::prelude::*;

// ⭐ A lei da tinta POR PINCEL (prompt 42, doc 42): tinta e marcador — o Glaze do pincel escurece o
// cruzamento como o do Rebelle (molhado: Ink 0,117 · Dip Pen 0,223 · Ink Brush 0,239 · marcadores 0,016).
// As receitas da sonda `sonda_midias` (b7: `z_IB2_g100`, `z_DP_g80`, `z_IK_g90`, `z_MB_h75`,
// `z_ML_h90`). As cerdas são as do Shape de fábrica: o feltro/cerdas que a sonda punha no mapa do motor
// não pintam (medido: a cena sem eles é a mesma ao byte).
pub const WET: &[Receita] = &[
    // Ink › Brush: corpo ralo e orla de nanquim, a água que se espalha devagar.
    Receita {
        meio: "Ink",
        grupo: "Brush",
        nome: "Ink Brush",
        water: 0.5,
        opacity: 1.0,
        pigment: 150.0,
        pickup: 1.0,
        gate: 0.0,
        evaporation: 3.0,
        edge: 0.0,
        tilt: DIREITA,
        bristle_size: 2.0,
        bristle_count: 2000.0,
        hardness: 0.1,
        size_px: None,
        // Glaze 0,8: a fracção multiplicada que o oráculo MEDIU nesta tinta (doc 42 §3: 0,76 · 0,82); o 1,0 de antes
        // compensava o halo e o azul arrastado para baixo (doc 42 §6).
        extras: &[
            ("extDiffusion", 0.01),
            ("glaze", 0.8),
            ("pressureSizeCurve", 1.0),
            ("pressureOpacity", 0.9),
            ("pressureOpacityCurve", 0.5),
        ],
        grao: None,
        achatado: None,
        pincel: &[],
        nasce_de: None,
    },
    // Ink › Pen › Dip Pen: duro e cheio, a tinta de cima ainda tapa um pouco (Glaze 0,8).
    Receita {
        meio: "Ink",
        grupo: "Pen",
        nome: "Dip Pen",
        water: 0.3,
        opacity: 1.0,
        pigment: 1500.0,
        pickup: 1.0,
        gate: 0.0,
        evaporation: 1.0,
        edge: 0.0,
        tilt: DIREITA,
        bristle_size: 2.0,
        bristle_count: 2000.0,
        hardness: 0.9,
        size_px: None,
        extras: &[
            ("glaze", 0.8),
            ("pressureSizeCurve", 1.0),
            ("pressureOpacity", 0.9),
            ("pressureOpacityCurve", 1.5),
        ],
        grao: None,
        achatado: None,
        pincel: &[],
        nasce_de: None,
    },
    // Ink › Pen › Ink Bullet: o mais carregado e seco da tinta.
    Receita {
        meio: "Ink",
        grupo: "Pen",
        nome: "Ink Bullet",
        water: 0.15,
        opacity: 1.0,
        pigment: 2000.0,
        pickup: 1.0,
        gate: 0.0,
        evaporation: 1.0,
        edge: 0.0,
        tilt: DIREITA,
        bristle_size: 2.0,
        bristle_count: 2000.0,
        hardness: 0.9,
        size_px: None,
        extras: &[
            ("glaze", 0.9),
            ("pressureSizeCurve", 0.25),
            ("pressureOpacity", 1.0),
            ("pressureOpacityCurve", 3.0),
        ],
        grao: None,
        achatado: None,
        pincel: &[],
        nasce_de: None,
    },
];

pub const MEIOS: &[ReceitaMeio] = &[];

// O Cruzamento é o da foto MOLHADA; a Borda e a Orla só onde o Round não as tem (Ink Bullet: borda 1,00 nos dois).
#[rustfmt::skip]
pub const MIDIAS: &[Midia] = &[
    Midia { preset: "Ink Bullet", foto: "ink", raio: 21.0, metricas: &[(Cobertura, 0.862), (Textura, 0.395), (Cruzamento, 0.117)] },
    Midia { preset: "Dip Pen", foto: "dippen", raio: 19.0, metricas: &[(Cobertura, 0.856), (Textura, 1.433), (Borda, 0.910), (Cruzamento, 0.223)] },
    Midia { preset: "Ink Brush", foto: "inkbrush", raio: 20.0, metricas: &[(Cobertura, 0.722), (Textura, 1.266), (Borda, 0.793), (Orla, 5.0), (Cruzamento, 0.239)] },
];
