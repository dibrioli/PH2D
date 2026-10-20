//! **Família `lapis`** — J3, catálogo B (os lápis além da série de grafite): Colored 4B, Scratch, Tilted Shade, Rough
//! Sketch, Craft Flat, Erodible Pencil. Escreva aqui as receitas e os testes da família; nada fora deste ficheiro
//! (`docs/Painter/Pinceis/04_BANCADA.md` §9). As bases (`GRAPHITE_8B`, `CHARCOAL`) são partilhadas: não as mude daqui.

use super::prelude::*;

pub const WET: &[Receita] = &[
    // `lapisb_041`: o de cor é o 2B com o dobro da carga.
    lapis(
        "Colored",
        "Colored 4B",
        80.0,
        0.65,
        GRAO_MACIO,
        extras_do_lapis!(0.5, 3.0),
    ),
    // `lapisc_053`: o carvão mais ralo, o papel a meio.
    Receita {
        meio: "Pencil",
        grupo: "Scratch",
        nome: "Scratch",
        pigment: 20.0,
        gate: 0.5,
        size_px: Some(248.0),
        extras: &[
            ("felt", 0.0),
            ("drag", 0.02),
            MAIS_PERTO,
            ("pressureSizeCurve", 0.35),
            ("pressureOpacity", 0.75),
            ("pressureOpacityCurve", 0.75),
        ],
        nasce_de: Some("Charcoal"),
        ..CHARCOAL
    },
];

pub const MEIOS: &[ReceitaMeio] = &[];

#[rustfmt::skip]
pub const MIDIAS: &[Midia] = &[
    Midia { preset: "Colored 4B", foto: "pencil4bcolor", raio: 15.00, metricas: &[(Cobertura, 0.665), (Textura, 11.826), (Borda, 0.327), (Orla, 10.0), (Cruzamento, 0.730), (Largura, 25.0)] },
    Midia { preset: "Scratch", foto: "scratch", raio: 15.71, metricas: &[(Cobertura, 0.255), (Textura, 19.780), (Borda, 0.591), (Orla, 13.0), (Cv, 0.126), (Cruzamento, 0.652), (Largura, 24.0)] },
];
