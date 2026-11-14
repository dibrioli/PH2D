//! **A ponta deitada chega ao carimbo nos CINCO meios** (W6, `ph2d_painter_brush::pen`): uma ponta quadrada
//! (a Shape de imagem, a que as caches de carimbo constante servem) com a caneta a rodá-la (PenTilt) e a
//! esticá-la — o traço com o azimute ao longo e através do traço tem de sair diferente em todas as rotas, e o
//! rato (`[0, 0]`) com a lei ligada é o traço de sem ela, ao byte. Mutação que sangra: as caches servirem com a
//! caneta a variar a pegada (`pen_varies_dabs` falso) ou a pegada ignorar a caneta (`dab_footprint_of`).

use super::*;
use ph2d_painter_brush::{DabRotation, PenTilt, StrokeMethod};
use ph2d_wet_paint::tuning::Knob;

/// `Some(meio)` ou o Composite (`None`).
fn ferramenta(meio: Option<PaintMedia>, lei: bool) -> PainterTool {
    let mut t = white_canvas(120, 9.0);
    match meio {
        Some(m) => t.set_paint_media(m),
        None => {
            t.toggle_composite();
            t.set_composite_layer_strength(0, 1.0);
        }
    }
    t.paint.brush.stroke_method = StrokeMethod::Space;
    // Na Aquarela a pegada do pincel (o Flatten, a Shape — e a ponta deitada, pela MESMA porta) só vale com o
    // Automatic desligado: o Automatic é o carimbo redondo próprio dela.
    t.paint.brush.watercolor_shape_auto = false;
    t.paint.brush.shape.size = [1.0, 1.0];
    let s = 16u32;
    let rgba: Vec<u8> = std::iter::repeat_n([40u8, 40, 40, 255], (s * s) as usize)
        .flatten()
        .collect();
    t.set_brush_shape_image_rgba(&rgba, s, s, None);
    t.toggle_brush_shape_alpha_from_image();
    if lei {
        t.paint.brush.pen_tilt = PenTilt {
            rotation: DabRotation::PenTilt,
            start_deg: 0.0,
            full_deg: 45.0,
            length: 2.0,
            ..PenTilt::default()
        };
        let k = &mut t.paint.wetpaint.knobs;
        k.set(Knob::DabRotation, 2.0);
        k.set(Knob::TiltStart, 0.0);
        k.set(Knob::TiltFull, 45.0);
        k.set(Knob::TiltLength, 2.0);
    }
    t
}

fn traco(meio: Option<PaintMedia>, lei: bool, tilt: [f32; 2]) -> Vec<u8> {
    let mut t = ferramenta(meio, lei);
    let cp = |x: f32, phase| CanvasPointer {
        pos: [x, 60.0],
        pressure: 1.0,
        tilt,
        phase,
    };
    t.on_canvas_pointer(cp(30.0, PointerPhase::Down));
    let mut x = 30.0;
    while x < 90.0 {
        x += 3.0;
        t.on_canvas_pointer(cp(x, PointerPhase::Move));
        t.on_tick(8.0);
    }
    t.on_canvas_pointer(cp(x, PointerPhase::Up));
    for _ in 0..30 {
        t.on_tick(1000.0 / 60.0);
    }
    t.run_full().0
}

#[test]
fn a_ponta_deitada_chega_ao_carimbo_nos_cinco_meios() {
    for meio in [
        Some(PaintMedia::Digital),
        Some(PaintMedia::Watercolor),
        Some(PaintMedia::Impasto),
        Some(PaintMedia::WetPaint),
        None,
    ] {
        let rato = traco(meio, false, [0.0, 0.0]);
        assert!(
            rato != vec![255u8; rato.len()],
            "{meio:?}: o traço não pintou"
        );
        assert!(
            rato == traco(meio, true, [0.0, 0.0]),
            "{meio:?}: o rato mudou com a lei ligada"
        );
        let ao_longo = traco(meio, true, [0.5, 0.0]);
        let atraves = traco(meio, true, [0.0, 0.5]);
        let diagonal = traco(meio, true, [0.35, 0.35]);
        assert!(
            ao_longo != rato,
            "{meio:?}: a caneta deitada ao longo não chegou ao carimbo"
        );
        assert!(
            ao_longo != atraves,
            "{meio:?}: o azimute não roda nem estica a ponta"
        );
        assert!(
            diagonal != ao_longo && diagonal != atraves,
            "{meio:?}: a diagonal é uma das outras"
        );
    }
}

/// B25.6: o Blur e o Clone decidem a pegada pela MESMA porta (`pegada_que_levanta`), e a caneta deitada conta
/// como textura nas duas. Um Blur redondo SEM Shape nem Grain, sobre listras, com a caneta deitada ao longo e
/// através do traço tem de borrar diferente; o rato fica o de sempre. Mutação que sangra: tirar `deitados` do
/// `textured` da porta (o redondo não lê a pegada: as duas canetas dão a mesma imagem — era o Blur até 14/11).
#[test]
fn o_blur_redondo_deita_com_a_caneta_como_o_clone() {
    let borra = |lei: bool, tilt: [f32; 2]| -> Vec<u8> {
        let mut t = white_canvas(120, 9.0);
        let listras: Vec<u8> = (0..120u32 * 120)
            .flat_map(|i| {
                let v = if (i % 120 / 3 + i / 120 / 3) % 2 == 0 {
                    20u8
                } else {
                    235
                };
                [v, v, v, 255]
            })
            .collect();
        t.set_source(listras, 120, 120);
        t.paint.paint_mode = PaintMode::Blur;
        t.paint.brush.stroke_method = StrokeMethod::Space;
        if lei {
            t.paint.brush.pen_tilt = PenTilt {
                rotation: DabRotation::PenTilt,
                start_deg: 0.0,
                full_deg: 45.0,
                length: 2.0,
                ..PenTilt::default()
            };
        }
        let cp = |x: f32, phase| CanvasPointer {
            pos: [x, 60.0],
            pressure: 1.0,
            tilt,
            phase,
        };
        t.on_canvas_pointer(cp(30.0, PointerPhase::Down));
        let mut x = 30.0;
        while x < 90.0 {
            x += 3.0;
            t.on_canvas_pointer(cp(x, PointerPhase::Move));
            t.on_tick(8.0);
        }
        t.on_canvas_pointer(cp(x, PointerPhase::Up));
        t.canvas_rgba.to_vec()
    };
    let rato = borra(false, [0.0, 0.0]);
    assert!(
        rato == borra(true, [0.0, 0.0]),
        "o rato mudou com a lei ligada"
    );
    let ao_longo = borra(true, [0.5, 0.0]);
    let atraves = borra(true, [0.0, 0.5]);
    assert!(ao_longo != rato, "a caneta deitada não chegou ao Blur");
    assert!(
        ao_longo != atraves,
        "o Blur não roda nem estica a ponta com o azimute"
    );
}
