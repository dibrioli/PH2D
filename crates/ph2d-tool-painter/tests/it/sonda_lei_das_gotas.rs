//! **Sondas (ignoradas) da lei das gotas** (W1c) — as variantes da [`DripLaw`] no MESMO processo
//! (CLAUDE.md §2: nunca uma `const` recompilada por variante), sobre o S do oráculo
//! ([`super::borda_do_s`]) e a bancada dos traços retos ([`super::gotas_pela_lei_do_rebelle`]). A régua
//! que as lê é a do portão ([`super::borda_do_s::mede_borda`]) ou `ferramentas/borda.py` sobre os PPM.
//!
//! ```text
//! PH2D_SONDA_OUT=/tmp/x PH2D_S_VARIANTES="g2:g=2;recede=0.3" cargo test -p ph2d-tool-painter --profile smoke --test it sonda_borda_do_s -- --ignored --nocapture
//! PH2D_B_LEIS="vol=3000,notch=0" cargo test -p ph2d-tool-painter --profile smoke --test it sonda_bancada_da_lei -- --ignored --nocapture
//! PH2D_WD_AGUAS="0.49,0.7" cargo test -p ph2d-tool-painter --profile smoke --test it sonda_wet_drip -- --ignored --nocapture
//! ```

use super::borda_do_s::{cena, grava};
use super::gotas_pela_lei_do_rebelle::inclina;
use ph2d_brush_presets::{Biblioteca, Volumes};
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::RasterEditTool;
use ph2d_tool_painter::PaintMedia;
use ph2d_wet_paint::drips::{DripLaw, DripPaper};

#[test]
#[ignore = "sonda: a cena do S do oráculo; corre-se à mão"]
fn sonda_borda_do_s() {
    let out = std::env::var("PH2D_SONDA_OUT").unwrap_or_else(|_| "/tmp".into());
    let sem = DripPaper {
        create: false,
        ..DripPaper::DEFAULT
    };
    // `PH2D_S_VARIANTES="nome:g=2;edgeDarkening=100,…"`: `g` = a força da lei (o ângulo mínimo e o que o
    // papel segura na mesma escala: só a água muda); `off` = Create Drips desligado; as outras chaves são
    // knobs do motor no preset. Além das três de sempre.
    let variantes = std::env::var("PH2D_S_VARIANTES").unwrap_or_default();
    type Cena = (
        String,
        Option<u8>,
        DripPaper,
        Option<DripLaw>,
        Vec<(String, f64)>,
        u32,
    );
    let mut cenas: Vec<Cena> = vec![
        (
            "s_direita".into(),
            None,
            DripPaper::DEFAULT,
            None,
            vec![],
            30,
        ),
        (
            "s_lei".into(),
            Some(8u8),
            DripPaper::DEFAULT,
            None,
            vec![],
            30,
        ),
        ("s_natural".into(), Some(8), sem, None, vec![], 30),
    ];
    for v in variantes.split(',').filter(|v| !v.is_empty()) {
        let (nome, resto) = v.split_once(':').expect("nome:chaves");
        let (lei, gotas, knobs, seg) = lei_de(resto);
        cenas.push((format!("s_{nome}"), Some(8), gotas, Some(lei), knobs, seg));
    }
    for (nome, anel, gotas, lei, knobs, seg) in cenas {
        let ratio = knobs.iter().find(|k| k.0 == "ratio").map(|k| k.1);
        let px = cena(
            anel,
            gotas,
            &|t| {
                if let Some(l) = lei {
                    t.set_wet_drip_law(l);
                }
                if let Some(r) = ratio {
                    t.set_wet_grid_ratio(r);
                }
            },
            &|p| {
                for (k, x) in knobs.iter().filter(|k| k.0 != "ratio") {
                    p.parametros.engine.insert(k.clone(), *x);
                }
            },
            seg,
        );
        grava(&out, &nome, &px);
        println!("[s] {nome}");
    }
}

/// A bancada dos traços retos (doc 06) com a lei variada: `PH2D_B_LEIS="vol=3000,recede=0.3"` ([`lei_de`]),
/// `PH2D_B_SIZE=5|10` (Size/Length do papel). Imprime a gota mais longa e os fios a +60 de cada traço.
#[test]
#[ignore = "sonda: a bancada com a lei variada; corre-se à mão"]
fn sonda_bancada_da_lei() {
    use super::gotas_pela_lei_do_rebelle::{bancada, fios, gota_mais_longa};
    let sz: u8 = std::env::var("PH2D_B_SIZE")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(5);
    for v in std::env::var("PH2D_B_LEIS")
        .unwrap_or_default()
        .split(',')
        .filter(|v| !v.is_empty())
    {
        let (lei, _, _, _) = lei_de(v);
        let px = bancada(30, &|t| {
            t.set_gotas(DripPaper {
                size: sz,
                length: sz,
                ..DripPaper::DEFAULT
            });
            super::gotas_pela_lei_do_rebelle::inclina(t, Some(8));
            t.set_wet_drip_law(lei);
        });
        let g: Vec<u32> = (0..4).map(|c| gota_mais_longa(&px, c)).collect();
        let f: Vec<Vec<u32>> = (0..4).map(|c| fios(&px, c, 235)).collect();
        let b: Vec<usize> = (0..2)
            .map(|c| super::gotas_pela_lei_do_rebelle::brechas(&px, c))
            .collect();
        println!("[bancada] {v} size {sz}: gota {g:?} fios {f:?} brechas {b:?}");
    }
}

/// `k=v;k=v` → a lei (`g` = a força, com o ângulo e o que o papel segura na mesma escala), o papel
/// (`off` = Create Drips desligado), os knobs do motor no preset e os segundos de água.
pub fn lei_de(spec: &str) -> (DripLaw, DripPaper, Vec<(String, f64)>, u32) {
    let sem = DripPaper {
        create: false,
        ..DripPaper::DEFAULT
    };
    let (mut lei, mut gotas, mut knobs, mut seg) =
        (DripLaw::DEFAULT, DripPaper::DEFAULT, Vec::new(), 30);
    for kv in spec.split(';').filter(|k| !k.is_empty()) {
        if kv == "off" {
            gotas = sem;
            continue;
        }
        let (k, x) = kv.split_once('=').expect("k=v");
        let x: f64 = x.parse().expect("número");
        match k {
            "g" => {
                let e = x / DripLaw::DEFAULT.gravity;
                lei.gravity = x;
                lei.slide = DripLaw::DEFAULT.slide * e;
                lei.hold = DripLaw::DEFAULT.hold * e;
                lei.hold_easy = DripLaw::DEFAULT.hold_easy * e;
            }
            "hold" => lei.hold = x,
            "trail_alpha" => lei.trail_alpha = x,
            "catch" => lei.catch_per_size = x,
            "vol" => lei.basin_water = x,
            "slide" => lei.slide = x,
            "t" => seg = x as u32,
            "recede" => lei.recede = x,
            "feed" => lei.feed = x,
            "dren" => lei.drain_after = x > 0.0,
            "notch" => lei.notch_w = x as i32,
            "wmin" => lei.trail_min_px = x,
            "neck" => lei.neck_dark = x,
            _ => knobs.push((k.to_string(), x)),
        }
    }
    (lei, gotas, knobs, seg)
}

/// O Wet Drip de fábrica num traço da bancada (Size px 40) na tela no máximo, com a água
/// `PH2D_WD_AGUAS="0.49,0.7"`: a gota mais longa.
#[test]
#[ignore = "sonda: a água do Wet Drip; corre-se à mão"]
fn sonda_wet_drip() {
    use super::gotas_pela_lei_do_rebelle::gota_mais_longa;
    use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase};
    let raiz = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
    let b = Biblioteca::carregar(&raiz, None);
    let p = &b
        .entradas
        .iter()
        .find(|e| e.preset.parametros.name == "Wet Drip")
        .expect("o pincel de fábrica")
        .preset;
    let ponto = |x: f32, phase| CanvasPointer {
        pos: [x, 150.0],
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    };
    for a in std::env::var("PH2D_WD_AGUAS")
        .unwrap_or_default()
        .split(',')
    {
        let Ok(agua) = a.parse::<f64>() else { continue };
        for easy in [false, true] {
            let mut t = super::sonda_gestos::tela();
            t.set_paint_media(PaintMedia::WetPaint);
            t.set_wet_relogio_fixo(true);
            inclina(&mut t, Some(8));
            t.set_gotas(DripPaper {
                easy,
                ..DripPaper::DEFAULT
            });
            assert!(t.wet_preset_aplicar(p).is_empty());
            t.wet_preset_aplicar_volumes(&Volumes {
                size_px: Some(40.0),
                water: Some(agua),
                ..Volumes::default()
            });
            t.set_brush_color_srgb8([23, 101, 179]);
            t.on_canvas_pointer(ponto(60.0, PointerPhase::Down));
            for k in 1..=50 {
                t.on_canvas_pointer(ponto(60.0 + 3.0 * k as f32, PointerPhase::Move));
                t.on_tick(8.0);
            }
            t.on_canvas_pointer(ponto(210.0, PointerPhase::Up));
            for _ in 0..30 * 60 {
                t.on_tick(1000.0 / 60.0);
            }
            let px = t.run_full().0;
            println!(
                "[wet drip] água {agua} easy {easy}: gota {}",
                gota_mais_longa(&px, 0)
            );
        }
    }
}
