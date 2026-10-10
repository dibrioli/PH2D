//! **Sonda (ignorada, W1e F1+F5): a lei da gota contra o Rebelle, numa varredura no MESMO processo.**
//! A bancada do doc 06 §2 (quatro traços de 150 px a 40 px, água 1 · 0,7 · 0,4 · 0,2, 30 s, relógio
//! fixo) com o Wet Round, para cada `G:H` de `PH2D_SONDA_VARRE` (a gravidade de TODOS os pincéis e a
//! `drip_head` do motor) e cada anel (`off`, 4, 8); grava `v_<G>_<H>_<anel>.ppm` em `PH2D_SONDA_OUT`.
//! A régua: `docs/Painter/NewFeaturesWetPaint/ferramentas/fios.py`.
use crate::tool::PainterTool;
use ph2d_brush_presets::{Biblioteca, Volumes};
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
use ph2d_wet_paint::tuning::Knob;

const W: usize = 1000;
const H: usize = 640;

fn ponto(x: f32) -> CanvasPointer {
    CanvasPointer {
        pos: [x, 150.0],
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase: PointerPhase::Move,
    }
}

/// A bancada: devolve a tela depois de 30 s.
pub(super) fn bancada(
    gravidade: f64,
    gota: f64,
    knobs: &[(Knob, f64)],
    anel: Option<u8>,
    aguas: [f64; 4],
) -> Vec<u8> {
    bancada_com(gravidade, gota, knobs, anel, aguas, &|_| {})
}

/// [`bancada`] com `ajusta` sobre o motor da sessão antes dos traços (as variantes da W1f).
pub(super) fn bancada_com(
    gravidade: f64,
    gota: f64,
    knobs: &[(Knob, f64)],
    anel: Option<u8>,
    aguas: [f64; 4],
    ajusta: &dyn Fn(&mut ph2d_wet_paint::sim::Sim),
) -> Vec<u8> {
    let raiz = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
    let b = Biblioteca::carregar(&raiz, None);
    let p = &b
        .entradas
        .iter()
        .find(|e| {
            e.preset.parametros.name
                == std::env::var("PH2D_SONDA_PINCEL").unwrap_or_else(|_| "Wet Round".into())
        })
        .expect("o pincel de fábrica")
        .preset;
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; W * H * 4], W as u32, H as u32);
    t.set_paint_tool_mode("wetpaint");
    t.set_wet_relogio_fixo(true);
    assert!(t.wet_preset_aplicar(p).is_empty());
    t.paint.wetpaint.knobs.set(Knob::Gravity, gravidade);
    for (k, v) in knobs {
        t.paint.wetpaint.knobs.set(*k, *v);
    }
    t.paint.wetpaint.tilt_on = anel.is_some();
    t.paint.wetpaint.tilt_spoke = 3;
    t.paint.wetpaint.tilt_ring = anel.unwrap_or(4);
    t.set_brush_color_srgb8([23, 101, 179]);
    assert!(t.ensure_wet_session());
    t.paint
        .wetpaint
        .session
        .as_mut()
        .expect("sessão")
        .engine
        .sim
        .drip_head = gota;
    ajusta(
        &mut t
            .paint
            .wetpaint
            .session
            .as_mut()
            .expect("sessão")
            .engine
            .sim,
    );
    // `PH2D_SONDA_POCA=n`: cada traço passa n vezes no mesmo sítio (a POÇA de muitos traços).
    let passes: usize = std::env::var("PH2D_SONDA_POCA")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(1);
    for (i, agua) in aguas
        .iter()
        .enumerate()
        .flat_map(|e| std::iter::repeat_n(e, passes))
    {
        t.wet_preset_aplicar_volumes(&Volumes {
            size_px: Some(40.0),
            water: Some(*agua),
            ..Volumes::default()
        });
        let x0 = 60.0 + i as f32 * 240.0;
        t.on_canvas_pointer(CanvasPointer {
            phase: PointerPhase::Down,
            ..ponto(x0)
        });
        for k in 1..=50 {
            t.on_canvas_pointer(ponto(x0 + 3.0 * k as f32));
            t.on_tick(8.0);
        }
        t.on_canvas_pointer(CanvasPointer {
            phase: PointerPhase::Up,
            ..ponto(x0 + 150.0)
        });
    }
    for k in 0..30 * 60 {
        t.on_tick(1000.0 / 60.0);
        if std::env::var("PH2D_SONDA_DIAG").is_ok() && [0, 30, 120, 600, 1799].contains(&k) {
            // A água do 1.º traço (x 60–210): a maior altura por linha, da orla para baixo.
            let s = t.paint.wetpaint.session.as_mut().expect("sessão");
            s.bring_home();
            let g = s.engine.active_grid();
            let linha = |y: usize| {
                (61..211)
                    .map(|x| g.film[x + y * g.s])
                    .fold(0.0f32, f32::max)
            };
            let marcas = (175..230)
                .flat_map(|y| (61..211).map(move |x| x + y * g.s))
                .filter(|&i| g.wet[i] != 0)
                .count();
            let molhadas = (175..230)
                .flat_map(|y| (61..211).map(move |x| x + y * g.s))
                .filter(|&i| g.film[i] > 1e-3)
                .count();
            println!(
                "[diag] marcas de humidade y175–230: {marcas} · com filme: {molhadas} · drip_head do motor {}",
                s.engine.sim.drip_head
            );
            println!(
                "[diag] G {gravidade} H {gota} anel {anel:?} t {:.1}s · filme máx y160 {:.4} y166 {:.4} y170 {:.4} y175 {:.4} y185 {:.4} y200 {:.4}",
                k as f32 / 60.0,
                linha(160),
                linha(166),
                linha(170),
                linha(175),
                linha(185),
                linha(200)
            );
        }
    }
    t.run_full().0
}

#[test]
#[ignore = "sonda: a varredura da lei da gota; corre-se à mão"]
fn sonda_gota() {
    let out = std::env::var("PH2D_SONDA_OUT").unwrap_or_else(|_| "/tmp".into());
    let varre = std::env::var("PH2D_SONDA_VARRE").unwrap_or_else(|_| "0.005:0".into());
    // Cada variante: `G:H` ou `G:H:chave=v/chave=v` (as chaves do registo, `KNOB_DEFS`).
    for par in varre.split(',') {
        let mut it = par.split(':');
        let g: f64 = it.next().expect("G").parse().expect("G");
        let h: f64 = it.next().expect("H").parse().expect("H");
        let extra = it.next().unwrap_or("");
        let knobs: Vec<(Knob, f64)> = extra
            .split('/')
            .filter(|e| !e.is_empty())
            .map(|e| {
                let (k, v) = e.split_once('=').expect("chave=v");
                let def = ph2d_wet_paint::tuning::KNOB_DEFS
                    .iter()
                    .find(|d| d.key == k)
                    .expect("knob conhecido");
                (def.knob, v.parse().expect("valor"))
            })
            .collect();
        let rotulo = if extra.is_empty() {
            String::new()
        } else {
            format!("_{}", extra.replace('/', "_"))
        };
        for (nome, anel) in [("off", None), ("meio", Some(4)), ("max", Some(8))] {
            let px = bancada(g, h, &knobs, anel, [1.0, 0.7, 0.4, 0.2]);
            let mut f = format!("P6\n{W} {H}\n255\n").into_bytes();
            for p in px.as_chunks::<4>().0 {
                f.extend_from_slice(&p[..3]);
            }
            std::fs::write(format!("{out}/v_{g}_{h}{rotulo}_{nome}.ppm"), f).expect("grava");
        }
        println!("[gota] G {g} H {h}");
    }
}
