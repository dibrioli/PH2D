//! **Sonda do Bug #65 A3:** o Smooth Edges não anda com o Smear, o Blend e o Water: a tinta arrastada chega a células com
//! degrau 0 e a borda dela é dura. Decisão do dono. Corre-se à mão:
//! `PH2D_SONDA_OUT=<pasta> cargo test -p ph2d-tool-painter --lib sonda_smooth_edges_no_arrasto -- --ignored --nocapture`;
//! escreve `<ferramenta>_{antes,depois}.ppm` e imprime `ferramenta|células de borda|a degrau 0`.
use super::*;
use crate::tool::paint::wetpaint_preset::de_fabrica_ou_candidato;
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};

const W: usize = 360;
const H: usize = 200;

fn risca(t: &mut PainterTool, a: [f32; 2], b: [f32; 2]) {
    let p = |k: f32, phase| CanvasPointer {
        pos: [a[0] + (b[0] - a[0]) * k, a[1] + (b[1] - a[1]) * k],
        pressure: 0.9,
        tilt: [0.0, 0.0],
        phase,
    };
    t.on_canvas_pointer(p(0.0, PointerPhase::Down));
    for i in 1..=40 {
        t.on_canvas_pointer(p(i as f32 / 40.0, PointerPhase::Move));
        t.on_tick(1000.0 / 60.0);
    }
    t.on_canvas_pointer(p(1.0, PointerPhase::Up));
    for _ in 0..8 {
        t.on_tick(1000.0 / 60.0);
    }
}

fn grava(pasta: &str, nome: &str, t: &PainterTool) {
    let mut ppm = format!("P6\n{W} {H}\n255\n").into_bytes();
    for p in t.canvas_rgba.chunks(4) {
        ppm.extend_from_slice(&p[..3]);
    }
    std::fs::write(format!("{pasta}/{nome}.ppm"), ppm).expect("grava");
}

/// `(células de borda, as que estão a degrau 0)`: a tinta com um vizinho de papel limpo.
fn bordas(t: &PainterTool) -> (usize, usize) {
    let g = t
        .paint
        .wetpaint
        .session
        .as_ref()
        .expect("sessao")
        .engine
        .active_grid();
    let tinta = |i: usize| f64::from(g.susp[i]) + f64::from(g.sett[i]) > 0.5;
    let (mut n, mut zero) = (0, 0);
    for y in 2..g.h {
        for x in 2..g.w {
            let i = x + y * g.s;
            if tinta(i) && [i - 1, i + 1, i - g.s, i + g.s].iter().any(|j| !tinta(*j)) {
                n += 1;
                zero += usize::from(
                    ph2d_wet_paint::grid::aparencia::degrau_do_suave(g.aparencia[i].suave) == 0,
                );
            }
        }
    }
    (n, zero)
}

#[test]
#[ignore = "sonda: a prancha do A3 (decisão do dono)"]
fn sonda_smooth_edges_no_arrasto() {
    let pasta = std::env::var("PH2D_SONDA_OUT").expect("PH2D_SONDA_OUT");
    let e = de_fabrica_ou_candidato("Wet Liner");
    println!("ferramenta|bordas|a degrau 0");
    for (nome, indice) in [("smear", 2usize), ("blend", 3), ("water", 4)] {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; W * H * 4], W as u32, H as u32);
        t.set_paint_tool_mode("wetpaint");
        t.set_wet_relogio_fixo(true);
        assert!(t.wet_preset_aplicar(&e).is_empty());
        t.wet_preset_aplicar_volumes(&e.parametros.volumes);
        t.set_brush_size_px(40.0);
        t.set_brush_color_srgb8([30, 60, 200]);
        t.set_wet_smooth_edges(1.0);
        risca(&mut t, [40.0, 100.0], [320.0, 100.0]);
        t.wet_recomposite_full();
        grava(&pasta, &format!("{nome}_antes"), &t);
        let (n0, z0) = bordas(&t);
        t.pick_wet_tool(indice);
        t.set_brush_size_px(40.0);
        risca(&mut t, [180.0, 40.0], [180.0, 170.0]);
        for _ in 0..240 {
            t.on_tick(1000.0 / 60.0); // a água leva a tinta para fora do rasto
        }
        t.wet_recomposite_full();
        grava(&pasta, &format!("{nome}_depois"), &t);
        let (n, z) = bordas(&t);
        println!("{nome}|antes {n0}|{z0}");
        println!("{nome}|depois {n}|{z}");
    }
}
