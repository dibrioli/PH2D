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
        size_px: Some(40.0),
        achatado: Some((0.65, 0)),
        extras: &[("dabRotation", 1.0)],
        pincel: &[("dab_rotation_lag", "1.5"), ("dab_quina", "0.5")],
        nasce_de: Some("Filbert"),
        ..FILBERT
    },
    // O carvão COMPRIMIDO: o bastão de quina viva a ângulo fixo, mais carga que o Charcoal e as pontas cortadas.
    Receita {
        nome: "Compressed Charcoal",
        pigment: 110.0,
        size_px: Some(30.0),
        achatado: Some((0.6, 30)),
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
        size_px: Some(30.0),
        achatado: Some((0.85, 45)),
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
        pigment: 90.0,
        size_px: Some(34.0),
        achatado: Some((0.5, 0)),
        extras: &[("dabRotation", 1.0), ("extDryBrush", 1.0)],
        pincel: &[("dab_rotation_lag", "0.6"), ("dab_quina", "0.4")],
        nasce_de: Some("Filbert"),
        ..FILBERT
    },
    // O chato de cerdas: a lâmina larga de riscas que gira com o gesto (fina no recto ao comprido, cheia ao través).
    Receita {
        size_px: Some(46.0),
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
        size_px: Some(28.0),
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

/// **Sonda (ignorada) da prancha CAP-01**: cada receita de [`WET`] (o ficheiro + a receita, sem regerar) pinta um recto
/// horizontal, um vertical e um S largo; grava `<nome>.ppm` em `PH2D_SONDA_OUT` (a prancha monta-se com `magick`).
#[test]
#[ignore = "sonda: a prancha da CAP-01; corre-se à mão"]
fn sonda_cap01() {
    use crate::sonda_gestos::{AZUL, Relogio, gesto, passa_o_tempo, tela};
    use ph2d_editor_core::tool::RasterEditTool;
    use std::io::Write;
    let out = std::env::var("PH2D_SONDA_OUT").unwrap_or_else(|_| "/tmp/ph2d_cap01".into());
    std::fs::create_dir_all(&out).unwrap();
    let so = std::env::var("PH2D_SONDA_PRESET").ok();
    for r in WET
        .iter()
        .filter(|r| so.as_deref().is_none_or(|s| s == r.nome))
    {
        let mut p = crate::midias_de_fabrica::de_fabrica(r.nome);
        p.parametros = crate::presets_de_fabrica::parametros(r);
        let mut t = tela();
        t.set_wet_relogio_fixo(true);
        assert!(t.wet_preset_aplicar(&p).is_empty());
        if p.parametros.volumes.size_px.is_none() {
            t.set_brush_size_px(40.0);
        }
        t.set_brush_color_srgb8(AZUL);
        let cheio = |_: f32| 1.0f32;
        gesto(&mut t, &[[60.0, 90.0], [560.0, 90.0]], 3.0, cheio);
        gesto(&mut t, &[[200.0, 200.0], [200.0, 600.0]], 3.0, cheio);
        let s: Vec<[f32; 2]> = (0..=120)
            .map(|k| {
                let u = k as f32 / 120.0;
                let a = u * std::f32::consts::TAU;
                [760.0 + 150.0 * a.sin(), 70.0 + 500.0 * u]
            })
            .collect();
        gesto(&mut t, &s, 3.0, cheio);
        gesto(&mut t, &[[330.0, 560.0], [560.0, 220.0]], 3.0, cheio);
        passa_o_tempo(&mut t, 240, Relogio::DeUmaVez);
        let (px, w, h) = t.run_full();
        let mut f =
            std::fs::File::create(std::path::Path::new(&out).join(format!("{}.ppm", r.nome)))
                .unwrap();
        write!(f, "P6\n{w} {h}\n255\n").unwrap();
        let rgb: Vec<u8> = px
            .as_chunks::<4>()
            .0
            .iter()
            .flat_map(|c| [c[0], c[1], c[2]])
            .collect();
        f.write_all(&rgb).unwrap();
        println!("[cap01] {} {w}x{h}", r.nome);
    }
}
