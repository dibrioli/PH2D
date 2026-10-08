//! **Sonda (ignorada): a cena do dono das gotas** (report de 08/10, foto do S e do L): o Wet Round de
//! fábrica num traço LARGO, longo e curvo, na tela no máximo, com e sem «Easy Drips». Grava
//! `gotas_dono_<variante>.ppm` em `PH2D_SONDA_OUT` e imprime as gotas que cruzam uma linha abaixo de cada
//! traço. Corre-se à mão:
//!
//! ```text
//! PH2D_SONDA_OUT=/tmp/x cargo test -p ph2d-tool-painter --profile smoke --test it sonda_gotas_do_dono -- --ignored --nocapture
//! ```

use super::gotas_pela_lei_do_rebelle::inclina;
use ph2d_brush_presets::{Biblioteca, Volumes};
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
use ph2d_tool_painter::{PaintMedia, PainterTool};

pub const W: u32 = 1400;
pub const H: u32 = 900;

/// O S do dono: sobe da esquerda, faz um arco, desce a um vale e sobe à direita.
pub fn o_s() -> Vec<[f32; 2]> {
    let mut v = Vec::new();
    for k in 0..=300 {
        let t = k as f32 / 300.0;
        let x = 130.0 + 860.0 * t;
        let y = 330.0 - 150.0 * (t * 6.0).sin() - 260.0 * t * t + 170.0 * (1.0 - t) * (1.0 - t);
        v.push([x, y]);
    }
    v
}

/// O L do dono: vai para a direita e sobe.
pub fn o_l() -> Vec<[f32; 2]> {
    let mut v: Vec<[f32; 2]> = (0..=60).map(|k| [1000.0 + 3.0 * k as f32, 560.0]).collect();
    v.extend((1..=60).map(|k| [1180.0 + 2.0 * k as f32, 560.0 - 3.5 * k as f32]));
    v
}

fn ponto(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// Um gesto pelos pontos, a passo de rato (o tick de 8 ms por evento, como a bancada).
pub fn gesto(t: &mut PainterTool, pts: &[[f32; 2]]) {
    t.on_canvas_pointer(ponto(pts[0], PointerPhase::Down));
    for &p in &pts[1..] {
        t.on_canvas_pointer(ponto(p, PointerPhase::Move));
        t.on_tick(8.0);
    }
    t.on_canvas_pointer(ponto(*pts.last().unwrap(), PointerPhase::Up));
}

/// A cena: tela 1400×900, o pincel de fábrica `nome` a `size_px`, a tela no anel `anel`, `easy`.
pub fn cena(nome: &str, size_px: f64, agua: f64, anel: u8, easy: bool) -> Vec<u8> {
    cena_com(
        nome,
        size_px,
        agua,
        Some(anel),
        ph2d_wet_paint::drips::DripPaper {
            easy,
            ..ph2d_wet_paint::drips::DripPaper::DEFAULT
        },
    )
}

/// [`cena`] com a inclinação (`None` = tela direita) e as gotas do papel inteiras.
pub fn cena_com(
    nome: &str,
    size_px: f64,
    agua: f64,
    anel: Option<u8>,
    gotas: ph2d_wet_paint::drips::DripPaper,
) -> Vec<u8> {
    let raiz = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
    let b = Biblioteca::carregar(&raiz, None);
    let e = b
        .entradas
        .iter()
        .find(|e| e.preset.parametros.name == nome)
        .expect("o pincel de fábrica");
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (W * H * 4) as usize], W, H);
    t.set_paint_media(PaintMedia::WetPaint);
    t.set_wet_relogio_fixo(true);
    inclina(&mut t, anel);
    t.set_gotas(gotas);
    assert!(t.wet_preset_aplicar(&e.preset).is_empty());
    t.wet_preset_aplicar_volumes(&Volumes {
        size_px: Some(size_px),
        water: Some(agua),
        ..Volumes::default()
    });
    t.set_brush_color_srgb8([23, 101, 179]);
    gesto(&mut t, &o_s());
    gesto(&mut t, &o_l());
    for _ in 0..30 * 60 {
        t.on_tick(1000.0 / 60.0);
    }
    t.run_full().0
}

fn grava(out: &str, nome: &str, px: &[u8]) {
    let mut f = format!("P6\n{W} {H}\n255\n").into_bytes();
    for p in px.as_chunks::<4>().0 {
        f.extend_from_slice(&p[..3]);
    }
    std::fs::write(format!("{out}/gotas_dono_{nome}.ppm"), f).unwrap();
}

#[test]
#[ignore = "sonda: a cena do report do dono; corre-se à mão"]
fn sonda_gotas_do_dono() {
    let out = std::env::var("PH2D_SONDA_OUT").unwrap_or_else(|_| "/tmp".into());
    for (nome, anel, easy) in [
        ("normal", 8u8, false),
        ("easy", 8, true),
        ("meio", 4, false),
    ] {
        let px = cena("Wet Round", 140.0, 1.0, anel, easy);
        grava(&out, nome, &px);
        // Fios que cruzam a linha y = 700 (abaixo de tudo menos das gotas).
        let y = 700usize;
        let mut fios = 0;
        let mut dentro = false;
        for x in 0..W as usize {
            let i = (y * W as usize + x) * 4;
            let tinta = 255 - px[i..i + 3].iter().copied().min().unwrap_or(255) > 25;
            if tinta && !dentro {
                fios += 1;
            }
            dentro = tinta;
        }
        println!("[dono] {nome}: {fios} fios cruzam y = {y}");
    }
}

/// Os fios que cruzam a linha `y` (corridas de tinta).
pub fn fios_na_linha(px: &[u8], y: usize) -> usize {
    let mut n = 0;
    let mut dentro = false;
    for x in 0..W as usize {
        let i = (y * W as usize + x) * 4;
        let tinta = 255 - px[i..i + 3].iter().copied().min().unwrap_or(255) > 25;
        if tinta && !dentro {
            n += 1;
        }
        dentro = tinta;
    }
    n
}
