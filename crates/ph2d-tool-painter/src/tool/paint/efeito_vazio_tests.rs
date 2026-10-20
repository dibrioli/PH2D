//! **L7 — o modo do pincel sobre o VAZIO** (`docs/Painter/Pinceis/06_LEIS_DAS_LACUNAS.md` §2: no papel sem tinta
//! o Effect do Rebelle pousa em NORMAL com o MESMO α, ± 0,005; o W3C geral também). Com `blend_sobre_vazio` um modo
//! de cor sobre a camada transparente pinta o que o `Mix` pinta; desligado (fábrica) é o de sempre: nada.

use crate::tool::PainterTool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
use ph2d_painter_brush::{BrushBlend, BrushSpec, Falloff};

const AZUL: [u8; 3] = [23, 101, 179];
const MODOS: [BrushBlend; 6] = [
    BrushBlend::Multiply,
    BrushBlend::Color,
    BrushBlend::Hue,
    BrushBlend::Luminosity,
    BrushBlend::Overlay,
    BrushBlend::LinearBurn,
];

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// Um toque Digital do pincel AZUL no modo `m` (força `alfa`) sobre `fundo`; o texel do centro (RGBA).
fn toque(fundo: [u8; 4], m: BrushBlend, alfa: f32, vazio: bool) -> [u8; 4] {
    let mut t = PainterTool::default();
    let (w, h) = (64u32, 64u32);
    let mut px = vec![0u8; (w * h * 4) as usize];
    for p in px.as_chunks_mut::<4>().0 {
        *p = fundo;
    }
    t.set_source(px, w, h);
    let b = BrushSpec {
        radius_px: 12.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: AZUL.map(|c| f32::from(c) / 255.0),
        blend_sobre_vazio: vazio,
        ..Default::default()
    };
    t.paint.brush = b;
    t.paint.brush_by_mode.fill(b);
    t.set_brush_blend(m as u8);
    t.set_brush_strength(alfa);
    t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Up));
    let o = ((32 * w + 32) * 4) as usize;
    [0, 1, 2, 3].map(|k| t.canvas_rgba[o + k])
}

/// ⭐ **O efeito sobre o vazio pousa a cor** (Digital): `α` do papel = `α` da tinta ± 0,01 e a cor é a do pincel
/// (± 2/255) — o mesmo texel que o `Mix`; sobre a tinta opaca o campo não muda um byte; desligado, o vazio fica
/// vazio (o de sempre).
#[test]
fn o_efeito_sobre_o_vazio_pousa_a_cor() {
    for alfa in [0.056f32, 0.216, 0.6] {
        let normal = toque([0; 4], BrushBlend::Mix, alfa, false);
        assert!(
            (f32::from(normal[3]) / 255.0 - alfa).abs() < 0.01,
            "o Mix pousa α {alfa}: {normal:?}"
        );
        for m in MODOS {
            let v = toque([0; 4], m, alfa, true);
            assert!(
                (f32::from(v[3]) / 255.0 - alfa).abs() < 0.01,
                "{m:?} α {alfa}: {v:?}"
            );
            for k in 0..3 {
                assert!(v[k].abs_diff(AZUL[k]) <= 2, "{m:?} α {alfa}: a cor {v:?}");
            }
            assert_eq!(
                toque([0; 4], m, alfa, false)[3],
                0,
                "{m:?} desligado pintou o vazio"
            );
            let base = [200, 120, 60, 255];
            assert_eq!(
                toque(base, m, alfa, true),
                toque(base, m, alfa, false),
                "{m:?} sobre a tinta"
            );
        }
    }
}

/// Um recto do Wet Paint no modo `m` sobre a camada TRANSPARENTE; a tela.
fn recto_wet(m: BrushBlend, vazio: bool) -> Vec<u8> {
    use ph2d_editor_core::Tool;
    let (w, h) = (160usize, 60usize);
    let mut t = PainterTool::default();
    t.set_source(vec![0u8; w * h * 4], w as u32, h as u32);
    t.set_paint_tool_mode("wetpaint");
    t.set_wet_relogio_fixo(true);
    t.set_brush_size_px(12.0);
    t.set_brush_color_srgb8(AZUL);
    t.paint.brush.blend_sobre_vazio = vazio;
    t.set_brush_blend(m as u8);
    let p = |x: f32, phase| cp([x, 30.0], phase);
    t.on_canvas_pointer(p(20.0, PointerPhase::Down));
    for k in 1..=30 {
        t.on_canvas_pointer(p(20.0 + 4.0 * k as f32, PointerPhase::Move));
        t.on_tick(8.0);
    }
    t.on_canvas_pointer(p(140.0, PointerPhase::Up));
    for _ in 0..20 {
        t.on_tick(1000.0 / 60.0);
    }
    t.canvas_rgba.to_vec()
}

/// ⭐ **O Wet tinha o MESMO buraco** (a sessão compõe-se no modo do pincel, `wetpaint/composite.rs`): um modo de
/// cor na camada transparente não pousava nada. Com o campo, o recto em Multiply/Color sobre o vazio é o recto
/// em Mix (± 2/255 por canal); desligado, o vazio fica vazio.
#[test]
fn o_efeito_sobre_o_vazio_pousa_a_cor_no_wet() {
    let normal = recto_wet(BrushBlend::Mix, false);
    assert!(
        normal.chunks(4).filter(|p| p[3] > 0).count() > 500,
        "o Mix do Wet não pintou"
    );
    for m in [BrushBlend::Multiply, BrushBlend::Color] {
        assert!(
            recto_wet(m, false).chunks(4).all(|p| p[3] == 0),
            "{m:?} desligado pintou o vazio"
        );
        let v = recto_wet(m, true);
        let pior = v
            .iter()
            .zip(&normal)
            .map(|(a, b)| a.abs_diff(*b))
            .max()
            .unwrap();
        assert!(pior <= 2, "{m:?} sobre o vazio desviou {pior}/255 do Mix");
    }
}
