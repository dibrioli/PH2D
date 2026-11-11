//! B31.1 — **o aro branco à volta do Impasto no papel** (AP-7, O2-50): a parede da TINTA, fora da tinta, acendia com a
//! presença do papel (`body = 1`), e o papel nu ali ia a 255. A lei: fora da tinta só o dente inclina a luz.
use super::*;
use crate::Region;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};

/// A cor do papel da foto do defeito (coluna x=780: 227,214,201).
const PAPEL: [u8; 4] = [227, 214, 201, 255];

fn papel(w: u32, h: u32) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(PAPEL.repeat((w * h) as usize), w, h);
    t.set_substrate_depth(0.5);
    t
}

fn crista(t: &mut PainterTool, w: u32, h: u32) {
    t.toggle_brush_impasto();
    t.set_brush_size_px(h as f32 * 0.3);
    let at = |x: f32, phase| CanvasPointer {
        pos: [x, h as f32 * 0.5],
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    };
    t.on_canvas_pointer(at(w as f32 * 0.2, PointerPhase::Down));
    t.on_canvas_pointer(at(w as f32 * 0.5, PointerPhase::Move));
    t.on_canvas_pointer(at(w as f32 * 0.8, PointerPhase::Up));
}

/// A tela acesa e o quanto a luz CLAREIA cada píxel (máximo dos 3 canais, em níveis) sobre a tela ainda sem luz.
fn clareia(t: &PainterTool) -> (Vec<u8>, Vec<i32>) {
    let (w, h) = t.source_size;
    let mut rgba = t.canvas_rgba.to_vec();
    t.apply_impasto_light(&mut rgba, Region { x: 0, y: 0, w, h });
    let d = rgba
        .as_chunks::<4>()
        .0
        .iter()
        .zip(t.canvas_rgba.as_chunks::<4>().0)
        .map(|(l, u)| {
            (0..3)
                .map(|c| i32::from(l[c]) - i32::from(u[c]))
                .max()
                .unwrap_or(0)
        })
        .collect();
    (rgba, d)
}

/// ⭐ **Sonda B31.1 (vermelha até a cura):** fora da tinta (corpo 0), ao lado da crista, a luz não clareia o papel mais
/// do que o dente sozinho o clareia em parte nenhuma da tela (+4 níveis), e nenhum píxel vira branco puro. Medido 11/11
/// (160×120, papel 0,5): 2 494 píxeis vistos, pior +54 contra +29 do dente, 944 brancos puros. A cura é a inclinação do
/// dente separada da da tinta (`∇dente + corpo·∇tinta`), na CPU e na placa (o shader só recebe a altura somada).
#[test]
#[ignore = "sonda B31.1: vermelha até a cura (o gradiente do dente separado, CPU e GPU) — BACKLOG B31.1"]
fn fora_da_tinta_o_papel_so_acende_pelo_dente() {
    let (w, h) = (160, 120);
    let mut t = papel(w, h);
    let (alt0, _) = t.composed_relief_plane();
    let so_dente = clareia(&t).1.into_iter().max().unwrap_or(0);
    crista(&mut t, w, h);
    let (alt, cob) = t.composed_relief_plane();
    let (rgba, d) = clareia(&t);
    let wu = w as usize;
    let (mut vistos, mut pior, mut brancos) = (0, i32::MIN, 0);
    for i in wu..alt.len() - wu {
        let (x, y) = (i % wu, i / wu);
        if x == 0 || x + 1 >= wu || y == 0 || y + 1 >= h as usize || cob[i] > 0.0 {
            continue;
        }
        if ![i - 1, i + 1, i - wu, i + wu]
            .iter()
            .any(|&j| alt[j] != alt0[j])
        {
            continue; // a parede da tinta não chega aqui
        }
        vistos += 1;
        pior = pior.max(d[i]);
        brancos += usize::from(rgba[i * 4..i * 4 + 3] == [255, 255, 255]);
    }
    println!(
        "papel nu ao lado da crista: {vistos} píxeis, pior +{pior}, {brancos} brancos puros (o dente sozinho: +{so_dente})"
    );
    assert!(vistos > 20, "a sonda tem de ver papel nu ao lado da crista");
    assert!(
        brancos == 0 && pior <= so_dente + 4,
        "aro: o papel nu ao lado da crista clareia +{pior}, {brancos} brancos puros (o dente sozinho: +{so_dente})"
    );
}
