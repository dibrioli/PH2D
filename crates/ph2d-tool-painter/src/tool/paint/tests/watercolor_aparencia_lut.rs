//! B16.6 (PL-31): a aparência da aguada lê o tom linear da tabela quando o tom é um byte (a base
//! opaca, a vazia, o chão). Ao byte contra o caminho de antes (`APARENCIA_PELA_CONTA`, o mesmo
//! processo), e a sonda do move intercalada.

use super::measure_watercolor_cost::{move_ms, wash};
use super::*;
use crate::tool::paint::watercolor_render::APARENCIA_PELA_CONTA;
use ph2d_painter_brush::{TextureKind, TextureMapping, TextureSettings};
use std::sync::atomic::Ordering;

/// O Watercolor Basic de `watercolor_parity` (Cold, Warp, Rewet).
fn basic(t: &mut PainterTool) {
    t.paint.brush = BrushSpec {
        radius_px: 40.0,
        color: [0.24, 0.39, 0.63],
        spacing: 0.05,
        watercolor: true,
        fill: 0.12,
        depth: 1.2,
        edge_gain: 3.0,
        edge_spread: 12.0,
        warp: 6.0,
        granulation: 0.30,
        paper: TextureSettings {
            kind: TextureKind::PaperCold,
            mapping: TextureMapping::Tiled,
            ..TextureSettings::default()
        },
        wet_rewet: 0.3,
        ..Default::default()
    };
    t.paint.brush_by_mode.fill(t.paint.brush);
}

/// Dois traços que se cruzam sobre uma camada de alfa `alfa` (255 = opaca, 0 = vazia, o resto = a
/// orla que a tabela não serve), com o caminho de antes ou o novo.
fn pinta(alfa: u8, pela_conta: bool) -> Vec<u8> {
    let size = 256u32;
    let mut t = PainterTool::default();
    let px = [200u8, 180, 150, alfa];
    t.set_source(px.repeat((size * size) as usize), size, size);
    basic(&mut t);
    APARENCIA_PELA_CONTA.store(pela_conta, Ordering::Relaxed);
    for (y0, dy) in [(60.0f32, 4.0f32), (200.0, -4.0)] {
        assert!(t.on_canvas_pointer(cp([40.0, y0], PointerPhase::Down)));
        for i in 1..=30 {
            let p = [40.0 + i as f32 * 6.0, y0 + i as f32 * dy * 0.5];
            t.on_canvas_pointer(cp(p, PointerPhase::Move));
            frame(&mut t);
        }
        t.on_canvas_pointer(cp([220.0, y0 + 15.0 * dy], PointerPhase::Up));
        frame(&mut t);
    }
    APARENCIA_PELA_CONTA.store(false, Ordering::Relaxed);
    t.canvas_rgba.to_vec()
}

#[test]
fn a_aparencia_pela_tabela_pinta_o_byte_da_conta() {
    for alfa in [255u8, 0, 140] {
        let novo = pinta(alfa, false);
        let antes = pinta(alfa, true);
        let base = [200u8, 180, 150, alfa].repeat(novo.len() / 4);
        assert_ne!(novo, base, "alfa {alfa}: o traço tem de pintar");
        let dif = novo.iter().zip(&antes).filter(|(a, b)| a != b).count();
        assert_eq!(
            dif, 0,
            "alfa {alfa}: {dif} bytes diferem do caminho de antes"
        );
    }
}

/// O traço de `move_ms` com o tique do quadro (onde a aguada compõe): o mínimo do move + tique.
fn quadro_ms(t: &mut PainterTool, y: f32) -> f64 {
    assert!(t.on_canvas_pointer(cp([120.0, y], PointerPhase::Down)));
    frame(t);
    let mut m = f64::MAX;
    for i in 1..=20 {
        let t0 = std::time::Instant::now();
        t.on_canvas_pointer(cp([120.0 + i as f32 * 40.0, y], PointerPhase::Move));
        frame(t);
        m = m.min(t0.elapsed().as_secs_f64() * 1e3);
    }
    t.on_canvas_pointer(cp([920.0, y], PointerPhase::Up));
    frame(t);
    m
}

/// Sonda (smoke): o move de aquarela a 4096² (sem e com o tique do quadro), o caminho de antes × o
/// novo, intercalados, o mínimo de 7.
#[test]
#[ignore = "medição, não gate"]
fn sonda_a_aparencia_pela_tabela_no_move_4096() {
    const SIZE: u32 = 4096;
    const RADIUS: f32 = 100.0;
    let mut m = [[f64::MAX; 2]; 2];
    for r in 0..7 {
        for pela_conta in [r % 2 == 0, r % 2 != 0] {
            APARENCIA_PELA_CONTA.store(pela_conta, Ordering::Relaxed);
            let k = usize::from(pela_conta);
            let ms = move_ms(&mut wash(SIZE, RADIUS), SIZE, RADIUS);
            m[0][k] = m[0][k].min(ms);
            let ms = quadro_ms(&mut wash(SIZE, RADIUS), 2048.0);
            m[1][k] = m[1][k].min(ms);
        }
    }
    APARENCIA_PELA_CONTA.store(false, Ordering::Relaxed);
    for (i, nome) in ["move", "move + tique"].iter().enumerate() {
        println!(
            "[B16.6] {nome} 4096² r100, mín de 7: pela conta {:.3} ms → pela tabela {:.3} ms",
            m[i][1], m[i][0]
        );
    }
}
