use super::*;
use ph2d_editor::tool::RasterEditTool;

fn sourced_tool() -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![0u8; 4 * 4 * 4], 4, 4);
    t
}

/// A single-layer canvas with a real impasto stroke on it. Big enough that a brush actually deposits
/// relief — a 4x4 canvas would leave `heights` empty and every gate below would pass by doing
/// nothing.
fn sculpted_tool() -> PainterTool {
    use ph2d_editor::tool::{CanvasPaintTool, CanvasPointer, PointerPhase};
    let cp = |pos: [f32; 2], phase: PointerPhase| CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    };
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; 48 * 48 * 4], 48, 48);
    t.toggle_brush_impasto();
    t.set_brush_impasto_depth(0.8);
    t.set_brush_size_px(9.0);
    t.on_canvas_pointer(cp([14.0, 24.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([34.0, 24.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([34.0, 24.0], PointerPhase::Up));
    t
}

#[test]
fn trivial_stack_stays_on_the_cpu_path() {
    // A single plain raster IS GPU-representable (flatten = Some), but the
    // gate must bow out FIRST: the CPU path is zero-composite + partial
    // bbox upload there, and the fluid E4 texture mode (trivial-only)
    // must never share the frame with a GPU layer recomposite.
    let t = sourced_tool();
    assert!(t.preview_is_trivial_stack());
    assert!(
        super::super::painter_gpu_flatten::flatten_for_gpu(t.layers()).is_some(),
        "precondition: the trivial stack is representable (the gate, not \
         the flatten, must reject it)"
    );
    assert!(gpu_eligible(&t).is_none(), "trivial stack must stay CPU");
}

/// **Repeat Image keeps the CPU producer** (Enio 2026-07-20, "em impasto Tiling as imagens
/// repetidas desaparecem"): the 3×3 tile preview blits the CPU composite (`PainterPreview`),
/// and when the GPU owns the slot the bridge clears that cache — the tiles silently vanish.
/// The fixture is the sculpted document (GPU-eligible since the light port — exactly the case
/// the report came from); the presence half proves the refusal is the TOGGLE's, not the stack's.
#[test]
fn repeat_image_keeps_the_cpu_producer() {
    let mut t = sculpted_tool();
    assert!(
        gpu_eligible(&t).is_some(),
        "precondition: the sculpted document is GPU-eligible with Repeat Image off"
    );
    t.toggle_repeat_image();
    assert!(
        gpu_eligible(&t).is_none(),
        "with Repeat Image on the CPU lane must produce (the tile preview blits its bytes)"
    );
    t.toggle_repeat_image();
    assert!(
        gpu_eligible(&t).is_some(),
        "toggling Repeat Image back off returns the slot to the GPU"
    );
}

#[test]
fn non_trivial_representable_stack_is_gpu_eligible() {
    // Opacity < 1 on the single layer breaks triviality without leaving
    // GPU-representability → the GPU path owns the preview.
    let mut t = sourced_tool();
    let active = t.layers().active().expect("set_source creates Layer 1");
    t.set_layer_opacity(active, 0.5);
    assert!(!t.preview_is_trivial_stack());
    let (ops, _luts) = gpu_eligible(&t).expect("representable non-trivial stack -> GPU");
    assert!(
        matches!(ops[..], [LayerOp::Layer { opacity, .. }] if (opacity - 0.5).abs() < 1e-6),
        "single half-opacity layer flattens to one Layer op: {ops:?}"
    );
}

/// A sculpted SINGLE-LAYER document — the most ordinary way to use Impasto — must now take the GPU
/// path. It is the whole point of the light port, and it is the case the old `impasto_visible()` bail
/// sent to the CPU compositor together with every layer, blend mode and adjustment in the document.
///
/// The premise this rests on is checked here rather than assumed: the stack IS trivial, so without
/// the relief the gate would (correctly) bow out.
#[test]
fn a_sculpted_document_goes_to_the_gpu_even_when_the_stack_is_trivial() {
    let t = sculpted_tool();
    assert!(
        t.preview_is_trivial_stack(),
        "precondition: one plain raster layer — the relief is the ONLY reason this goes GPU"
    );
    assert!(
        t.impasto_visible(),
        "precondition: the canvas carries relief"
    );
    assert!(
        gpu_eligible(&t).is_some(),
        "a sculpted canvas belongs on the GPU: the CPU lane is not zero-composite here \
         (runtime.rs refuses its fast path when impasto_visible), so it pays a full composite \
         AND a full CPU light every dirty frame"
    );
}

/// …and the light has something to hand the shader. A gate on eligibility alone would stay green if
/// the planes came back `None`, which is the shape of "the GPU path claims the frame and draws it
/// unlit" — the exact failure the old bail existed to prevent.
#[test]
fn a_gpu_eligible_sculpted_document_hands_over_planes() {
    let t = sculpted_tool();
    let p = t
        .impasto_gpu_planes()
        .expect("a lit, sculpted canvas must produce planes for the shader");
    assert_eq!(p.relief.len(), (p.width as usize) * (p.height as usize));
    assert!(
        p.cover.iter().any(|c| *c > 0),
        "…and the planes must carry actual paint, not an empty canvas"
    );
    assert!(!p.lamps.is_empty(), "…lit by at least one lamp");
}

/// Switching the pass OFF must put the canvas back on the CPU's zero-composite fast lane. Otherwise
/// hiding the relief would silently cost a full GPU recomposite per frame to draw a picture the CPU
/// hands over by cloning an `Arc`.
#[test]
fn hiding_the_relief_returns_a_trivial_document_to_the_cpu() {
    let mut t = sculpted_tool();
    t.toggle_impasto_show();
    assert!(!t.impasto_visible());
    assert!(
        gpu_eligible(&t).is_none(),
        "no relief on screen -> the trivial bow-out applies again"
    );
}

/// **The masked document goes to the GPU now** — the headline of
/// `docs/Painter/25_avaliacao_gpu.md` achado A, asserted where the routing
/// actually happens rather than one layer down in the flatten.
///
/// Measured before this landed: a single mask took a 4096² adjustment drag
/// from 0,738 ms to 652,9 ms — 885×, triggered by a checkbox, with nothing on
/// screen to say why. A mask is how you paint inside a shape; it is not an
/// exotic feature to leave on the slow producer.
#[test]
fn a_masked_stack_is_gpu_eligible_now() {
    let mut t = sourced_tool();
    t.add_mask_to_active().expect("mask on Layer 1");
    assert!(!t.preview_is_trivial_stack());
    assert!(
        gpu_eligible(&t).is_some(),
        "a per-layer mask is an op now — it must not force the CPU producer"
    );
}

/// The refusal that REMAINS, so "eligible" does not quietly become "always".
///
/// A clipped GROUP stays on the CPU deliberately: the CPU reference's Group
/// arm reads neither `mask` nor `clipping`, and a GPU that honoured them would
/// make the picture depend on which producer won the frame.
#[test]
fn a_clipped_group_is_still_not_gpu_eligible() {
    let mut t = sourced_tool();
    let g = t.add_group().expect("group");
    t.set_layer_clipping(g, true);
    assert!(
        gpu_eligible(&t).is_none(),
        "a clipped group must stay on the CPU while the CPU ignores the flag"
    );
}
