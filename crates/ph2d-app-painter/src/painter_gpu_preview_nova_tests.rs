//! B18.4 — a camada nova transparente na placa: o 1.º toque envia só o sujo, e o slot é o do caminho antigo.
use super::*;

/// **O 1.º toque da camada nova envia à placa só a caixa do traço, não a camada inteira, ao bit.** Duas pistas
/// com o mesmo documento e os mesmos toques: a antiga (`NOVA_SEM_LIMPAR`, a camada sobe inteira no 1.º toque)
/// é o controlo e o oráculo do slot em cada quadro.
#[test]
#[ignore = "needs a real GPU device; run with --ignored on the GPU lane"]
fn o_primeiro_toque_da_camada_nova_envia_so_a_caixa_ao_bit() {
    use crate::painter_preview_pipeline_tests::{assert_screen_equals, cp, impasto_tool};
    use ph2d_editor_core::tool::{CanvasPaintTool, PointerPhase};
    let Ok(gpu) = GpuContext::new(GpuContext::default_instance(), None) else {
        return;
    };
    let lado = 240u32;
    let pista = || {
        let t = impasto_tool(lado);
        let r = SpriteRenderer::new(
            gpu.clone(),
            ph2d_render::GameRt::FORMAT,
            ph2d_render::TextureAtlas::dummy(&gpu),
            8,
        );
        (t, r, None, None, ToastQueue::default())
    };
    let mut pistas = [pista(), pista()];
    let enviados =
        |s: &Option<PainterGpuPreview>| s.as_ref().map_or(0, |s| s.compositor.bytes_enviados());
    let mut primeiro = [0u64; 2];
    // 0..=2 = um traço na base (o relevo é o que dá a tela à placa); 3 = o «+»; 4.. = o traço na
    // camada nova (4 = o 1.º toque).
    for passo in 0u8..=11 {
        let mut telas = Vec::new();
        for (antigo, (t, r, s, g, toasts)) in pistas.iter_mut().enumerate() {
            NOVA_SEM_LIMPAR.set(antigo == 1);
            if passo == 3 {
                t.add_raster_layer("nova".to_string());
            } else {
                let fase = match passo {
                    0 | 4 => PointerPhase::Down,
                    2 | 11 => PointerPhase::Up,
                    _ => PointerPhase::Move,
                };
                let y = if passo < 3 { 60.0 } else { 120.0 };
                t.on_canvas_pointer(cp([40.0 + 16.0 * f32::from(passo), y], fase));
            }
            let antes = enviados(s);
            assert!(
                try_drive(s, r, t, Some(1), g, toasts),
                "a placa é dona (passo {passo})"
            );
            if passo == 4 {
                primeiro[antigo] = enviados(s) - antes;
            }
            let id = g.expect("o slot da placa").texture_id;
            telas.push(r.individual().readback(&gpu, id).expect("readback").2);
        }
        NOVA_SEM_LIMPAR.set(false);
        assert_screen_equals(&telas[0], &telas[1], lado, &format!("passo {passo}"));
    }
    let camada = u64::from(lado) * u64::from(lado) * 4;
    assert!(
        primeiro[1] >= camada,
        "controlo: o caminho antigo envia {} < a camada {camada}",
        primeiro[1]
    );
    assert!(
        primeiro[0] * 8 < camada,
        "o 1.º toque da camada nova enviou {} bytes (a camada inteira = {camada})",
        primeiro[0]
    );
}
