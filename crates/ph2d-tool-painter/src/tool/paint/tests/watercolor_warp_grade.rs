//! B16.6 (PL-31): o warp do Ragged Edge lê os `hash2` de uma grelha preenchida 1× por composite
//! (`GradeDoWarp`). Ao byte contra a conta (`WARP_PELA_CONTA`, o mesmo processo), e a sonda 4096².

use super::measure_watercolor_cost::{move_ms, wash};
use super::*;
use crate::tool::paint::watercolor_noise::WARP_PELA_CONTA;
use ph2d_painter_brush::{TextureKind, TextureMapping, TextureSettings};
use std::sync::atomic::Ordering;

/// Dois traços que se cruzam (Watercolor Basic: Cold, Warp 6, Rewet), sem ou com o Tiling dos dois
/// eixos (o `wrap_cell` da grelha), pela grelha ou pela conta.
fn pinta(tiling: bool, warp: f32, pela_conta: bool) -> Vec<u8> {
    let size = 256u32;
    let mut t = PainterTool::default();
    t.set_source(
        [200u8, 180, 150, 255].repeat((size * size) as usize),
        size,
        size,
    );
    t.paint.tiling = [tiling, tiling];
    t.paint.brush = BrushSpec {
        radius_px: 40.0,
        color: [0.24, 0.39, 0.63],
        spacing: 0.05,
        watercolor: true,
        fill: 0.12,
        depth: 1.2,
        edge_gain: 3.0,
        edge_spread: 12.0,
        warp,
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
    WARP_PELA_CONTA.store(pela_conta, Ordering::Relaxed);
    // O 2.º traço sai pela borda (o Tiling dá a volta) e cruza o 1.º.
    for (y0, dy) in [(60.0f32, 4.0f32), (240.0, -4.0)] {
        assert!(t.on_canvas_pointer(cp([10.0, y0], PointerPhase::Down)));
        for i in 1..=30 {
            let p = [10.0 + i as f32 * 8.0, y0 + i as f32 * dy * 0.5];
            t.on_canvas_pointer(cp(p, PointerPhase::Move));
            frame(&mut t);
        }
        t.on_canvas_pointer(cp([252.0, y0 + 15.0 * dy], PointerPhase::Up));
        frame(&mut t);
    }
    WARP_PELA_CONTA.store(false, Ordering::Relaxed);
    t.canvas_rgba.to_vec()
}

#[test]
fn o_warp_pela_grelha_de_hash2_pinta_o_byte_da_conta() {
    for (tiling, warp) in [(false, 6.0f32), (true, 6.0), (false, 24.0), (true, 24.0)] {
        let novo = pinta(tiling, warp, false);
        let antes = pinta(tiling, warp, true);
        assert_ne!(
            novo,
            pinta(tiling, 0.0, false),
            "tiling {tiling}: o warp tem de mexer na borda"
        );
        let dif = novo.iter().zip(&antes).filter(|(a, b)| a != b).count();
        assert_eq!(
            dif, 0,
            "tiling {tiling} warp {warp}: {dif} bytes diferem da conta"
        );
    }
}

/// O move com o tique do quadro (onde a aguada compõe): o mínimo de 20.
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

/// Sonda (smoke): o move + tique de aquarela a 4096² r100, a conta × a grelha, intercalados, mín de 9.
#[test]
#[ignore = "medição, não gate"]
fn sonda_o_warp_pela_grelha_no_move_4096() {
    const SIZE: u32 = 4096;
    const RADIUS: f32 = 100.0;
    let mut m = [[f64::MAX; 2]; 2];
    for r in 0..9 {
        for pela_conta in [r % 2 == 0, r % 2 != 0] {
            WARP_PELA_CONTA.store(pela_conta, Ordering::Relaxed);
            let k = usize::from(pela_conta);
            m[0][k] = m[0][k].min(move_ms(&mut wash(SIZE, RADIUS), SIZE, RADIUS));
            m[1][k] = m[1][k].min(quadro_ms(&mut wash(SIZE, RADIUS), 2048.0));
        }
    }
    WARP_PELA_CONTA.store(false, Ordering::Relaxed);
    for (i, nome) in ["move", "move + tique"].iter().enumerate() {
        println!(
            "[B16.6 hash2] {nome} 4096² r100, mín de 9: pela conta {:.3} ms → pela grelha {:.3} ms",
            m[i][1], m[i][0]
        );
    }
}
