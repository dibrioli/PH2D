//! **Família `cap01`** — CAP-01 (31/10): a ponta que GIRA com o gesto (L3, `BrushSpec::dab_rotation_lag`) e as pontas
//! chatas de ângulo fixo. Seis pincéis do Wet Paint; o Broad Nib, o Marker Chisel e o Flat Bristle saíram do estudo. Ordem
//! do dono: superar o Rebelle em beleza, não ser idêntico — ele afina no smoke. Prancha:
//! `docs/Painter/Pinceis/pranchas/cap01_seis_pinceis.png`.

use super::caligrafia_marcadores::SEM_FORMA;
use super::oleo_acrilica::{FILBERT, FLAT_BRISTLE};
use super::prelude::*;

pub const WET: &[Receita] = &[
    // A sombra deitada: o Filbert achatado que SEGUE o traço com inércia — a curva alarga e o recto fica limpo.
    Receita {
        nome: "Tilted Shade",
        achatado: Some((0.6, 0)),
        extras: &[("dabRotation", 1.0)],
        pincel: &[("dab_rotation_lag", "1.0"), ("dab_quina", "0.3")],
        nasce_de: Some("Filbert"),
        ..FILBERT
    },
    // O carvão COMPRIMIDO: o bastão de quina viva a ângulo fixo, mais carga que o Charcoal e as pontas cortadas.
    Receita {
        nome: "Compressed Charcoal",
        pigment: 70.0,
        achatado: Some((0.8, 30)),
        pincel: &[("dab_quina", "1.0"), ("pontas_rectas", "true")],
        nasce_de: Some("Charcoal"),
        ..CHARCOAL
    },
    // O marcador de bisel: 45° fixo, a tinta que multiplica do Marker Bullet, as pontas que esbatem.
    Receita {
        meio: "Marker",
        grupo: "Marker",
        nome: "Marker Chisel",
        hardness: 0.3,
        achatado: Some((0.9, 45)),
        extras: &[
            ("glaze", 1.0),
            ("tipClean", 1.0),
            ("drag", 0.0),
            ("pressureSize", 0.0),
            ("pressureOpacity", 1.0),
            ("pressureOpacityCurve", 3.0),
        ],
        pincel: &[
            ("shape", SEM_FORMA),
            ("dab_quina", "1.0"),
            ("pontas_rectas", "true"),
            ("pontas_alem", "0.3"),
            ("pontas_esbate", "4.0"),
        ],
        nasce_de: Some("Marker Bullet"),
        ..super::caligrafia_marcadores::WET[0]
    },
    // O Filbert SECO: pouca tinta e nenhuma água — a cerda falha no grão do papel; segue o traço devagar.
    Receita {
        nome: "Filbert Dry",
        water: 0.0,
        pigment: 120.0,
        achatado: Some((0.5, 0)),
        extras: &[("dabRotation", 1.0), ("extDryBrush", 0.7)],
        pincel: &[("dab_rotation_lag", "0.6"), ("dab_quina", "0.4")],
        nasce_de: Some("Filbert"),
        ..FILBERT
    },
    // O chato de cerdas: a lâmina larga de riscas que gira com o gesto (fina no recto ao comprido, cheia ao través).
    Receita {
        achatado: Some((0.7, 0)),
        extras: &[("dabRotation", 1.0)],
        pincel: &[("dab_rotation_lag", "0.8"), ("dab_quina", "1.0")],
        nasce_de: Some("Filbert"),
        ..FLAT_BRISTLE
    },
    // O bico largo da caligrafia: 30° fixo — a largura sai da direcção; a tinta lisa do Ink Bullet.
    Receita {
        meio: "Ink",
        grupo: "Nib",
        nome: "Broad Nib",
        water: 0.1,
        opacity: 1.0,
        pigment: 2000.0,
        pickup: 0.0,
        gate: 0.0,
        evaporation: 1.0,
        edge: 0.0,
        tilt: DIREITA,
        bristle_size: 2.0,
        bristle_count: 2000.0,
        hardness: 0.6,
        size_px: None,
        extras: &[("glaze", 0.0), ("tipClean", 1.0), ("drag", 0.0)],
        grao: None,
        achatado: Some((0.9, 30)),
        pincel: &[("shape", SEM_FORMA), ("dab_quina", "0.8")],
        nasce_de: Some("Ink Bullet"),
        real_oil: false,
    },
];

pub const MEIOS: &[ReceitaMeio] = &[];

pub const MIDIAS: &[Midia] = &[];
