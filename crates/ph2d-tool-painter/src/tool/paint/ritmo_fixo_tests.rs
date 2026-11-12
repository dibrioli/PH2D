//! B22.1 (decisão 8, ritmo fixo): o mesmo gesto pelo mesmo tempo de parede dá a mesma tela, a mesma
//! umidade e o mesmo encharque a 30, 60 e 144 Hz. A porta é o `on_tick` real (a do shell).

use crate::tool::PainterTool;
use crate::tool::paint::media::PaintMedia;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool, Tool};
use ph2d_painter_brush::{BrushSpec, Falloff};

const SIZE: u32 = 192;

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// Aquarela com reumedecimento (encharque sob a ponta parada) e secagem de 2 s (a mais rápida).
fn tela() -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (SIZE * SIZE * 4) as usize], SIZE, SIZE);
    let b = BrushSpec {
        radius_px: 18.0,
        hardness: 0.5,
        falloff: Falloff::Smooth,
        strength: 1.0,
        color: [0.2, 0.3, 0.8],
        space_attenuation: false,
        ..Default::default()
    };
    t.paint.brush = b;
    t.paint.brush_by_mode.fill(b);
    t.set_paint_media(PaintMedia::Watercolor);
    t.set_brush_wet_rewet(0.8);
    t.set_dry_time_s(2.0);
    t
}

/// `segundos` de quadros a `hz` pela porta do shell (`dt_ms` = o `wall_dt` do quadro).
fn quadros(t: &mut PainterTool, hz: u32, segundos: f32) {
    let n = (segundos * hz as f32).round() as u32;
    let dt_ms = 1000.0 / hz as f32;
    for _ in 0..n {
        t.on_tick(dt_ms);
    }
}

/// Traço curto, 0,5 s parado com a caneta em baixo (encharque), solta, 1 s a secar (tempos que as três taxas cobrem com quadros inteiros).
///
/// `moves = 0` toca e segura num ponto. Com `moves > 0` o estabilizador alcança a ponta nos passos
/// fixos parados (B22.1).
fn gesto(hz: u32, moves: u32) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
    let mut t = tela();
    let y = SIZE as f32 / 2.0;
    t.on_canvas_pointer(cp([40.0, y], PointerPhase::Down));
    for i in 1..=moves {
        t.on_canvas_pointer(cp([40.0 + i as f32 * 8.0, y], PointerPhase::Move));
    }
    quadros(&mut t, hz, 0.5);
    t.on_canvas_pointer(cp([40.0 + moves as f32 * 8.0, y], PointerPhase::Up));
    quadros(&mut t, hz, 1.0);
    (
        t.canvas_rgba.to_vec(),
        t.paint.canvas_wet.clone(),
        t.paint.wet_soak.clone(),
    )
}

fn diferentes(a: &[u8], b: &[u8]) -> usize {
    if a.len() != b.len() {
        return a.len().max(b.len());
    }
    a.iter().zip(b).filter(|(x, y)| x != y).count()
}

#[test]
fn o_mesmo_gesto_a_30_60_e_144_hz_da_a_mesma_tela() {
    let (tela60, umid60, soak60) = gesto(60, 0);
    assert!(tela60.iter().any(|&b| b != 255), "a cena tem de pintar");
    assert!(
        umid60.iter().any(|&b| b != 0),
        "a cena tem de deixar umidade a secar"
    );
    assert!(
        soak60.iter().any(|&b| b != 0),
        "a cena tem de encharcar sob a ponta parada"
    );
    for hz in [30u32, 144] {
        let (tela, umid, soak) = gesto(hz, 0);
        let (dt, du, ds) = (
            diferentes(&tela60, &tela),
            diferentes(&umid60, &umid),
            diferentes(&soak60, &soak),
        );
        assert_eq!(
            (dt, du, ds),
            (0, 0, 0),
            "{hz} Hz × 60 Hz pelo mesmo tempo: bytes diferentes (tela, umidade, encharque)"
        );
    }
}

/// O traço que anda e pára: tela, umidade e encharque iguais ao bit — o alcance do estabilizador e o
/// encharque andam no passo fixo, e o primeiro passo depois dos Moves é o «em movimento» a qualquer taxa.
#[test]
fn o_traco_que_anda_e_para_seca_igual_a_30_60_e_144_hz() {
    let (tela60, umid60, soak60) = gesto(60, 6);
    for hz in [30u32, 144] {
        let (tela, umid, soak) = gesto(hz, 6);
        let (dt, du, ds) = (
            diferentes(&tela60, &tela),
            diferentes(&umid60, &umid),
            diferentes(&soak60, &soak),
        );
        eprintln!(
            "B22.1 {hz} Hz × 60 Hz, traço que anda: tela {dt} · umidade {du} · encharque {ds}"
        );
        assert_eq!(
            (dt, du, ds),
            (0, 0, 0),
            "{hz} Hz × 60 Hz, traço que anda: bytes diferentes (tela, umidade, encharque)"
        );
    }
}
