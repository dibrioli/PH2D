//! B15.4 — um tique de Depth/Body/Smoothing/Push/Source no último traço recompõe só o que mudou, e o
//! quadro é o da recomposição total AO BIT.

use crate::tool::{DrainBranch, PainterTool};
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};

/// Papel com relevo + um traço de impasto vivo (Adjust Last Stroke), já drenado uma vez.
fn traco_vivo(w: u32, h: u32) -> PainterTool {
    let mut t = PainterTool::default();
    t.paint.impasto_live_edit = true;
    t.set_source(vec![255u8; (w * h * 4) as usize], w, h);
    t.set_substrate_depth(1.0);
    t.set_substrate_roughness(0.5);
    t.toggle_brush_impasto();
    t.set_brush_size_px(h as f32 * 0.2);
    t.set_brush_impasto_smoothing(0.4);
    let at = |x: f32, y: f32, phase| CanvasPointer {
        pos: [x * w as f32, y * h as f32],
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    };
    t.on_canvas_pointer(at(0.3, 0.4, PointerPhase::Down));
    let _ = t.take_preview_arc();
    t.on_canvas_pointer(at(0.6, 0.6, PointerPhase::Move));
    let _ = t.take_preview_arc();
    t.on_canvas_pointer(at(0.6, 0.6, PointerPhase::Up));
    t.reconcile_substrate();
    let _ = t.take_preview_arc().expect("o quadro do traço");
    t
}

/// MUT: devolver o `invalidate_composite()` ao `refresh_live_relief` ⇒ o ramo vira `FullComposite`
/// (vermelho). MUT: tirar o `mark_dirty` do `rebuild_live_layer_relief` ⇒ o tique não muda o quadro.
#[test]
fn o_tique_do_corpo_no_ultimo_traco_recompoe_so_o_que_mudou_ao_bit() {
    type Tique = (&'static str, fn(&mut PainterTool));
    let tiques: [Tique; 6] = [
        ("Depth", |t| t.set_brush_impasto_depth(0.35)),
        ("Body", |t| t.set_brush_impasto_body(0.2)),
        ("Smoothing", |t| t.set_brush_impasto_smoothing(0.9)),
        ("Push", |t| t.set_brush_impasto_push(0.6)),
        ("Source", |t| t.set_brush_impasto_source(1)),
        ("Depth de volta", |t| t.set_brush_impasto_depth(1.0)),
    ];
    let mut t = traco_vivo(160, 120);
    let mut antes = (*t.take_preview_arc().map_or_else(
        || {
            t.composited = None;
            t.preview_dirty = true;
            t.take_preview_arc().expect("recomposição").0
        },
        |a| a.0,
    ))
    .clone();
    let mut mudou = 0usize;
    for (nome, tique) in tiques {
        t.edited_since_bind = false;
        tique(&mut t);
        assert!(t.edited_since_bind, "{nome}: o tique é trabalho por gravar");
        // Nada mexeu (o Push de um traço que não empurrou nada) ⇒ nenhum quadro: fica o que está na tela.
        let parcial = match t.take_preview_arc() {
            Some((p, _, _)) => {
                assert_eq!(
                    t.preview_drain_diag().0,
                    DrainBranch::PartialComposite,
                    "{nome}: o tique só refaz a janela que mexeu, não a tela inteira"
                );
                (*p).clone()
            }
            None => antes.clone(),
        };
        t.composited = None;
        t.preview_dirty = true;
        let (total, _, _) = t.take_preview_arc().expect("recomposição total");
        assert!(
            *total == parcial,
            "{nome}: a composição parcial difere da recomposição total"
        );
        if parcial != antes {
            mudou += 1;
        }
        antes = parcial;
    }
    assert!(mudou >= 4, "os tiques têm de mudar o quadro ({mudou}/6)");
}
