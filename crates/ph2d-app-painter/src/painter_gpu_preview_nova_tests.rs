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

/// **N7 onda 5 — apagar, duplicar e agrupar compõem na placa só o sujo com a lista NOVA, ao bit.** Duas pistas com
/// o mesmo documento e os mesmos gestos: a antiga (`SEMPRE_INTEIRA`, a tela inteira em todo quadro) é o oráculo do
/// slot (placa × placa). Controlo: a pista nova confinou com a lista nova; com `ESTRUTURA_RECOMPOE` (a mutação), não.
/// Sem adaptador (a CI sem placa) não há nada a afirmar e o teste sai; com placa corre por nome (240², segundos).
#[test]
fn apagar_duplicar_e_agrupar_compoem_na_placa_so_o_sujo_ao_bit() {
    use crate::painter_preview_pipeline_tests::assert_screen_equals;
    use ph2d_editor_core::tool::RasterEditTool as _;
    let Ok(gpu) = GpuContext::new(GpuContext::default_instance(), None) else {
        eprintln!("sem adaptador de placa: nada a afirmar");
        return;
    };
    let lado = 240u32;
    // Tinta só numa faixa de cada camada: a janela do gesto não é a tela inteira.
    let faixa = |semente: u32, y0: u32| -> Vec<u8> {
        let mut s = semente | 1;
        let mut px = vec![0u8; (lado * lado * 4) as usize];
        for (i, b) in px.iter_mut().enumerate() {
            s ^= s << 13;
            s ^= s >> 17;
            s ^= s << 5;
            let y = (i as u32 / 4) / lado;
            if (y0..y0 + 30).contains(&y) {
                *b = (s >> 24) as u8;
            }
        }
        px
    };
    for gesto in 0u8..3 {
        for controlo in [false, true] {
            ESTRUTURA_RECOMPOE.set(controlo);
            CONFINADAS_POR_ESTRUTURA.set(0);
            let pista = || {
                let mut t = PainterTool::default();
                t.set_source(vec![255u8; (lado * lado * 4) as usize], lado, lado);
                t.add_raster_layer_with_pixels("A", faixa(7, 20))
                    .expect("A");
                let b = t
                    .add_raster_layer_with_pixels("B", faixa(11, 100))
                    .expect("B");
                t.add_raster_layer_with_pixels("C", faixa(13, 180))
                    .expect("C");
                t.select_layer(b);
                let r = SpriteRenderer::new(
                    gpu.clone(),
                    ph2d_render::GameRt::FORMAT,
                    ph2d_render::TextureAtlas::dummy(&gpu),
                    8,
                );
                (t, b, r, None, None, ToastQueue::default())
            };
            let mut pistas = [pista(), pista()];
            // 0 = a semente; 1 = o gesto; 2 = undo; 3 = redo.
            for passo in 0u8..4 {
                let mut telas = Vec::new();
                for (antigo, (t, b, r, s, g, toasts)) in pistas.iter_mut().enumerate() {
                    SEMPRE_INTEIRA.set(antigo == 1);
                    match passo {
                        1 => match gesto {
                            0 => assert!(t.delete_layer(*b)),
                            1 => assert!(t.duplicate_layer(*b).is_some()),
                            _ => assert!(t.group_active().is_some()),
                        },
                        2 => assert!(t.undo_last()),
                        3 => assert!(t.redo_last()),
                        _ => {}
                    }
                    if passo == 0 {
                        // A 1.ª drenagem cria a sessão (a lista guardada nasce na 2.ª).
                        assert!(try_drive(s, r, t, Some(1), g, toasts), "a semente");
                        t.pede_a_composicao_inteira();
                    }
                    assert!(try_drive(s, r, t, Some(1), g, toasts), "a placa é dona");
                    let id = g.expect("o slot da placa").texture_id;
                    telas.push(r.individual().readback(&gpu, id).expect("readback").2);
                }
                SEMPRE_INTEIRA.set(false);
                let ctx = format!("gesto {gesto} · controlo={controlo} · passo {passo}");
                assert_screen_equals(&telas[0], &telas[1], lado, &ctx);
            }
            ESTRUTURA_RECOMPOE.set(false);
            let n = CONFINADAS_POR_ESTRUTURA.get();
            if controlo {
                assert_eq!(n, 0, "gesto {gesto}: o controlo confinou {n} drenagens");
            } else {
                assert!(
                    n >= 1,
                    "gesto {gesto}: a placa não confinou com a lista nova"
                );
            }
        }
    }
}
