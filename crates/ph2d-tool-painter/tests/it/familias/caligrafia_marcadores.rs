//! **Família `caligrafia_marcadores`** — J3, catálogos D e E: Monoline, Script, Broad Nib, Shale; Marker Bullet, Marker
//! Liner, Marker Chisel, Dry Marker. Escreva aqui as receitas e os testes da família; nada fora deste ficheiro
//! (`docs/Painter/Pinceis/04_BANCADA.md` §9).

use super::prelude::*;

pub const WET: &[Receita] = &[
    // Marker › Marker Bullet: chapado, quase sem água, multiplica (Glaze 1) — a lei da tinta por pincel (prompt 42).
    Receita {
        meio: "Marker",
        grupo: "Marker",
        nome: "Marker Bullet",
        water: 0.1,
        opacity: 1.0,
        pigment: 800.0,
        pickup: 1.0,
        gate: 0.0,
        evaporation: 1.0,
        edge: 0.0,
        tilt: DIREITA,
        bristle_size: 2.0,
        bristle_count: 2000.0,
        hardness: 0.75,
        size_px: None,
        extras: &[
            ("glaze", 1.0),
            ("pressureSizeCurve", 0.25),
            ("pressureOpacity", 1.0),
            ("pressureOpacityCurve", 2.0),
        ],
        grao: None,
        achatado: None,
        pincel: &[],
        nasce_de: None,
    },
    // Marker › Marker Liner: o mesmo marcador, a foto é fina (18 px): a dureza
    // medida à lei de largura do PRÓPRIO preset (raio 9; doc 42 §6) — 0,9 dava a borda 0,874 (Rebelle 0,716).
    Receita {
        meio: "Marker",
        grupo: "Marker",
        nome: "Marker Liner",
        water: 0.1,
        opacity: 1.0,
        pigment: 800.0,
        pickup: 1.0,
        gate: 0.0,
        evaporation: 1.0,
        edge: 0.0,
        tilt: DIREITA,
        bristle_size: 2.0,
        bristle_count: 2000.0,
        hardness: 0.55,
        size_px: None,
        extras: &[
            ("glaze", 1.0),
            ("pressureSizeCurve", 0.35),
            ("pressureOpacity", 0.9),
            ("pressureOpacityCurve", 3.0),
        ],
        grao: None,
        achatado: None,
        pincel: &[],
        nasce_de: None,
    },
];

pub const MEIOS: &[ReceitaMeio] = &[];

// Orla 1 nos marcadores (como no Round): não conta.
#[rustfmt::skip]
pub const MIDIAS: &[Midia] = &[
    Midia { preset: "Marker Bullet", foto: "markerbullet", raio: 20.0, metricas: &[(Cobertura, 0.909), (Textura, 0.658), (Borda, 0.768), (Cruzamento, 0.016)] },
    Midia { preset: "Marker Liner", foto: "marker", raio: 9.0, metricas: &[(Cobertura, 0.910), (Textura, 2.116), (Borda, 0.716), (Cruzamento, 0.016)] },
];
