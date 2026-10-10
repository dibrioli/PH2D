//! **Sonda (ignorada) do Tilt do preset** (09/10, dono: o escorrimento é a água, o pigmento e o círculo do
//! Tilt; o preset configura também o Tilt; os valores saem do Rebelle). A bancada do doc 06 §2 (quatro
//! traços de 150 px a 40 px, água 1,0 · 0,7 · 0,4 · 0,2, 30 s de água) no motor de hoje, com o Wet Round
//! de fábrica e a gravidade dele variada; e os quatro pincéis de fábrica com a água DELES — tudo no MESMO
//! processo, com a tela desligada, a meio (anel 4) e no máximo (anel 8). A régua corre sobre os PPM
//! (`ferramentas/tilt_rebelle.py`, a mesma conta sobre as fotos do Rebelle).
//!
//! ```text
//! PH2D_SONDA_OUT=/tmp/x cargo test -p ph2d-tool-painter --profile smoke --test it sonda_tilt_do_preset -- --ignored --nocapture
//! ```

use super::sonda_gestos::{H, W, tela};
use ph2d_brush_presets::{Biblioteca, Preset, Volumes};
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{
    CanvasPaintTool, CanvasPointer, PanelEvent, PointerPhase, RasterEditTool,
};
use ph2d_tool_painter::{PaintMedia, PainterTool, ids};

const AGUAS: [f64; 4] = [1.0, 0.7, 0.4, 0.2];
const Y: f32 = 150.0;

fn ponto(x: f32, phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos: [x, Y],
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// A inclinação: `None` = desligada; `Some(anel)` = para baixo, no anel `0..=8`.
fn inclina(t: &mut PainterTool, anel: Option<u8>) {
    let ligada = t.brush_settings().wet_tilt_on;
    if ligada != anel.is_some() {
        t.handle_panel_event(PanelEvent::Click(ids::PAINTER_WETPAINT_TILT_TOGGLE));
    }
    if let Some(a) = anel {
        t.handle_panel_event(PanelEvent::SetValue(ids::PAINTER_WETPAINT_TILT_SPOKE, 3.0));
        t.handle_panel_event(PanelEvent::SetValue(
            ids::PAINTER_WETPAINT_TILT_RING,
            f64::from(a),
        ));
    }
    assert_eq!(t.brush_settings().wet_tilt_on, anel.is_some());
}

fn preset(nome: &str) -> Preset {
    let raiz = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
    Biblioteca::carregar(&raiz, None)
        .entradas
        .iter()
        .find(|e| e.preset.parametros.name == nome)
        .expect("o pincel de fábrica")
        .preset
        .clone()
}

/// Os quatro traços com o pincel `p`; `aguas` = a água de cada traço (`None` = a do preset).
fn bancada(p: &Preset, anel: Option<u8>, aguas: [Option<f64>; 4]) -> Vec<u8> {
    let mut t = tela();
    t.set_paint_media(PaintMedia::WetPaint);
    t.set_wet_relogio_fixo(true);
    assert!(t.wet_preset_aplicar(p).is_empty());
    inclina(&mut t, anel);
    t.set_brush_color_srgb8([23, 101, 179]);
    for (i, agua) in aguas.iter().enumerate() {
        t.wet_preset_aplicar_volumes(&Volumes {
            size_px: Some(40.0),
            water: agua.or(p.parametros.volumes.water),
            ..Volumes::default()
        });
        let x0 = 60.0 + i as f32 * 240.0;
        t.on_canvas_pointer(ponto(x0, PointerPhase::Down));
        for k in 1..=50 {
            t.on_canvas_pointer(ponto(x0 + 3.0 * k as f32, PointerPhase::Move));
            t.on_tick(8.0);
        }
        t.on_canvas_pointer(ponto(x0 + 150.0, PointerPhase::Up));
    }
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
    std::fs::write(format!("{out}/{nome}.ppm"), f).unwrap();
}

#[test]
#[ignore = "sonda: o Tilt do preset contra o Rebelle; corre-se à mão"]
fn sonda_tilt_do_preset() {
    let out = std::env::var("PH2D_SONDA_OUT").unwrap_or_else(|_| "/tmp".into());
    let aneis: [(&str, Option<u8>); 3] = [("off", None), ("meio", Some(4)), ("max", Some(8))];
    for g in [0.005, 0.01] {
        let mut p = preset("Wet Round");
        p.parametros.engine.insert("gravity".into(), g);
        for (nome, anel) in aneis {
            let px = bancada(&p, anel, AGUAS.map(Some));
            grava(&out, &format!("round_g{g}_{nome}"), &px);
            println!("[tilt] round g {g} {nome}");
        }
    }
    for (nome, anel) in &aneis[1..] {
        let mut px = Vec::new();
        for (k, pincel) in ["Wet Round", "Wet Wash", "Wet Dry Brush", "Wet Drip"]
            .iter()
            .enumerate()
        {
            // Um pincel por coluna: o traço `k` da bancada com o pincel `k` e a água DELE.
            let mut aguas = [Some(0.0); 4];
            aguas[k] = None;
            let um = bancada(&preset(pincel), *anel, aguas);
            if px.is_empty() {
                px = vec![255u8; um.len()];
            }
            let (x0, x1) = (k * 240 + 30, k * 240 + 250);
            for y in 0..H as usize {
                let o = (y * W as usize + x0) * 4;
                let o1 = (y * W as usize + x1.min(W as usize)) * 4;
                px[o..o1].copy_from_slice(&um[o..o1]);
            }
        }
        grava(&out, &format!("pinceis_{nome}"), &px);
        println!("[tilt] pinceis {nome}");
    }
}
