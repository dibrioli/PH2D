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

/// n.º 9 item 3 — **a troca com a tinta viva do Wet dobra na placa só a parte do relevo, ao bit.** A tinta que adormece
/// (e a que acorda) muda a figura na janela dela e não o relevo: o slot da pista nova é, em cada troca, desfazer e
/// refazer, o da antiga (`SEMPRE_INTEIRA`, a tela inteira em todo quadro), com o papel aceso. Controlo: a nova dobrou só
/// a parte do relevo; com `DOBRA_A_REGIAO` (a mutação), não — e o slot continua o mesmo. Os passos 6 (pinta e troca) e 8
/// (troca e pinta) não drenam no meio: a parte do relevo é a do traço.
///
/// n.º 10 F5: corre com `TINTA_MARCA_COMO_ANTES` a 1 (a tinta que adormece marca a janela: o caminho do item 3, controlo
/// `≥ 1`) e a 0 (a troca pura não marca nada: nenhuma dobra pela figura, `n == 0`, e as telas ao bit na mesma).
#[test]
fn a_troca_com_a_tinta_viva_dobra_na_placa_so_o_relevo_ao_bit() {
    use crate::painter_preview_pipeline_tests::{assert_screen_equals, cp};
    use ph2d_editor_core::tool::{CanvasPaintTool, PointerPhase, RasterEditTool as _};
    let Ok(gpu) = GpuContext::new(GpuContext::default_instance(), None) else {
        eprintln!("sem adaptador de placa: nada a afirmar");
        return;
    };
    let lado = 160u32;
    let traco = |t: &mut PainterTool, y: f32| {
        t.set_brush_size_px(10.0);
        t.set_brush_color_srgb8([40, 90, 220]);
        t.on_canvas_pointer(cp([20.0, y], PointerPhase::Down));
        for k in 1..=12 {
            t.on_canvas_pointer(cp([20.0 + 10.0 * k as f32, y], PointerPhase::Move));
            ph2d_editor_core::Tool::on_tick(t, 1000.0 / 60.0);
        }
        t.on_canvas_pointer(cp([140.0, y], PointerPhase::Up));
        ph2d_editor_core::Tool::on_tick(t, 1000.0 / 60.0);
    };
    struct Repoe(u8);
    impl Drop for Repoe {
        fn drop(&mut self) {
            TINTA_MARCA_COMO_ANTES.store(self.0, std::sync::atomic::Ordering::Relaxed);
        }
    }
    use ph2d_tool_painter::tool::TINTA_MARCA_COMO_ANTES;
    let _repoe = Repoe(TINTA_MARCA_COMO_ANTES.load(std::sync::atomic::Ordering::Relaxed));
    for (marca, controlo) in [(1u8, false), (1, true), (0, false), (0, true)] {
        TINTA_MARCA_COMO_ANTES.store(marca, std::sync::atomic::Ordering::Relaxed);
        DOBRA_A_REGIAO.set(controlo);
        DOBRAS_PELA_FIGURA.set(0);
        let pista = || {
            let mut t = PainterTool::default();
            t.set_source(vec![255u8; (lado * lado * 4) as usize], lado, lado);
            t.set_wet_relogio_fixo(true);
            let a = t.add_raster_layer("A").expect("A");
            let b = t.add_raster_layer("B").expect("B");
            t.set_substrate_depth(1.0);
            t.set_paint_tool_mode("wetpaint");
            t.select_layer(a);
            traco(&mut t, 40.0);
            t.select_layer(b);
            traco(&mut t, 110.0);
            let r = SpriteRenderer::new(
                gpu.clone(),
                ph2d_render::GameRt::FORMAT,
                ph2d_render::TextureAtlas::dummy(&gpu),
                8,
            );
            ((t, a, b), r, None, None, ToastQueue::default())
        };
        let mut pistas = [pista(), pista()];
        // 0 = a semente; depois trocas, um undo, um redo e um traço no meio.
        for passo in 0u8..9 {
            let mut telas = Vec::new();
            for (antigo, ((t, a, b), r, s, g, toasts)) in pistas.iter_mut().enumerate() {
                SEMPRE_INTEIRA.set(antigo == 1);
                match passo {
                    1 | 3 => t.select_layer(*a),
                    2 | 7 => t.select_layer(*b),
                    4 => assert!(t.undo_last()),
                    5 => assert!(t.redo_last()),
                    // A tinta nova e a troca no MESMO quadro: o relevo do traço dobra-se, o resto não.
                    _ => {
                        if passo == 6 {
                            traco(t, 75.0);
                            t.select_layer(*b);
                        } else if passo == 8 {
                            t.select_layer(*a);
                            traco(t, 60.0);
                        }
                    }
                }
                if passo == 0 {
                    assert!(try_drive(s, r, t, Some(1), g, toasts), "a semente");
                    t.pede_a_composicao_inteira();
                }
                assert!(try_drive(s, r, t, Some(1), g, toasts), "a placa é dona");
                let id = g.expect("o slot da placa").texture_id;
                telas.push(r.individual().readback(&gpu, id).expect("readback").2);
            }
            SEMPRE_INTEIRA.set(false);
            let ctx = format!("marca={marca} · controlo={controlo} · passo {passo}");
            assert_screen_equals(&telas[0], &telas[1], lado, &ctx);
        }
        DOBRA_A_REGIAO.set(false);
        let n = DOBRAS_PELA_FIGURA.get();
        eprintln!("marca={marca} controlo={controlo}: {n} dobras pela figura");
        if controlo || marca == 0 {
            assert_eq!(
                n, 0,
                "marca={marca} controlo={controlo}: dobrou pela figura {n} vezes"
            );
        } else {
            assert!(n >= 1, "a placa não dobrou só o relevo em nenhuma troca");
        }
    }
}
