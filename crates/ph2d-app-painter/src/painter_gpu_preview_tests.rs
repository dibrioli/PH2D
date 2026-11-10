use super::*;
use ph2d_editor_core::tool::RasterEditTool;

fn sourced_tool() -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![0u8; 4 * 4 * 4], 4, 4);
    t
}

/// A single-layer canvas with a real impasto stroke on it. Big enough that a brush actually deposits
fn sculpted_tool() -> PainterTool {
    esculpido_de(48, 48)
}

/// [`sculpted_tool`] numa tela `w × h` (o traço na faixa `y = 24`).
fn esculpido_de(w: u32, h: u32) -> PainterTool {
    use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase};
    let cp = |pos: [f32; 2], phase: PointerPhase| CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    };
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; w as usize * h as usize * 4], w, h);
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
    let t = sourced_tool();
    assert!(t.preview_is_trivial_stack());
    assert!(
        ph2d_tool_painter::flatten_for_gpu(t.layers()).is_some(),
        "precondition: the trivial stack is representable (the gate, not \
         the flatten, must reject it)"
    );
    assert!(gpu_eligible(&t).is_none(), "trivial stack must stay CPU");
}

/// **Repeat Image keeps the CPU producer** (Enio 2026-07-20, "em impasto Tiling as imagens
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

/// **A PROTEÇÃO mantém o produtor de CPU** (Enio 2026-08-07: *"a máscara funcionou, mas fica invisível
#[test]
fn a_live_protection_keeps_the_cpu_producer() {
    use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, Tool as _};
    let mut t = sculpted_tool();
    assert!(
        gpu_eligible(&t).is_some(),
        "precondition: o documento esculpido é elegível SEM proteção"
    );
    // A proteção pela porta do artista: modo Mask + um traço no scratch.
    t.handle_panel_event(ph2d_editor_core::tool::PanelEvent::SelectOption(
        ph2d_editor_core::ids::PAINTER_PAINT_MODE,
        "mask".to_string(),
    ));
    let cp = |pos: [f32; 2], phase: PointerPhase| CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    };
    t.on_canvas_pointer(cp([24.0, 24.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([24.0, 24.0], PointerPhase::Up));
    assert!(
        t.mask_scratch_active(),
        "precondition: o traço de máscara criou o scratch"
    );
    assert!(
        gpu_eligible(&t).is_none(),
        "com proteção viva a pista de CPU tem de produzir — é ela que desenha o overlay, e sem isto o \
         artista pinta sob uma máscara que não vê"
    );
}

#[test]
fn non_trivial_representable_stack_is_gpu_eligible() {
    // Opacity < 1 on the single layer breaks triviality without leaving
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

/// **Com PAPEL a GPU produz** (2026-10-06; era `o_papel_mantem_o_produtor_de_cpu`): o compositor
#[test]
fn com_papel_a_gpu_produz() {
    let mut t = sculpted_tool();
    assert!(
        gpu_eligible(&t).is_some(),
        "precondition: o documento esculpido é elegível para a GPU"
    );
    t.aplica_o_papel();
    assert!(t.papel().is_some(), "controlo: o papel foi aplicado");
    assert!(
        gpu_eligible(&t).is_some(),
        "com papel a GPU tem de produzir (ela compõe sobre ele)"
    );
    let mut so_papel = sourced_tool();
    assert!(
        gpu_eligible(&so_papel).is_none(),
        "precondition: uma camada lisa é a pista trivial da CPU"
    );
    so_papel.aplica_o_papel();
    assert!(so_papel.papel().is_some(), "controlo: o papel foi aplicado");
    assert!(
        gpu_eligible(&so_papel).is_none(),
        "uma camada lisa com papel fica na pista parcial da CPU (medida mais rápida)"
    );
}

/// **Com o VIDRO da aguada sob papel de cor a CPU produz** (doc 48, BUGS #46): o compositor da GPU só
#[test]
fn com_vidro_sob_papel_de_cor_a_cpu_produz() {
    use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PanelEvent, PointerPhase, Tool};
    let cp = |pos: [f32; 2], phase: PointerPhase| CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    };
    let castanho = |t: &mut PainterTool| {
        t.handle_panel_event(PanelEvent::SelectOption(
            ph2d_tool_painter::ids::PAINTER_WATERCOLOR_PAPER_COLOR_THUMB,
            "153,121,91".into(),
        ));
    };
    let esculpido = || {
        let mut t = PainterTool::default();
        t.bind_document(1, vec![255u8; 48 * 48 * 4], 48, 48);
        t.toggle_brush_impasto();
        t.set_brush_impasto_depth(0.8);
        t.set_brush_size_px(9.0);
        t.on_canvas_pointer(cp([14.0, 24.0], PointerPhase::Down));
        t.on_canvas_pointer(cp([34.0, 24.0], PointerPhase::Move));
        t.on_canvas_pointer(cp([34.0, 24.0], PointerPhase::Up));
        t
    };
    let mut t = esculpido();
    t.set_paint_media(ph2d_tool_painter::PaintMedia::Watercolor);
    t.on_canvas_pointer(cp([10.0, 10.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([38.0, 12.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([38.0, 12.0], PointerPhase::Up));
    assert!(
        gpu_eligible(&t).is_some(),
        "precondition: esculpido, com aguada, no papel branco, a GPU produz"
    );
    castanho(&mut t);
    assert!(t.papel_atravessa_vidro(), "controlo: a aguada deixou vidro");
    assert!(
        gpu_eligible(&t).is_none(),
        "com o vidro sob o papel castanho a GPU mostraria a aguada de um alfa só"
    );
    let mut sem_aguada = esculpido();
    castanho(&mut sem_aguada);
    assert!(
        gpu_eligible(&sem_aguada).is_some(),
        "sem aguada o papel castanho não tira o documento da GPU"
    );
}

/// **G20 (F36 §1) — no fim de um episódio sem memória o preview na placa refaz a sessão e o slot e
#[test]
#[ignore = "needs a real GPU device; run with --ignored on the GPU lane"]
fn the_end_of_an_out_of_memory_episode_recomposes_the_gpu_preview() {
    let Ok(gpu) = GpuContext::new(GpuContext::default_instance(), None) else {
        return;
    };
    let atlas = ph2d_render::TextureAtlas::new(&gpu, 64);
    let mut renderer = SpriteRenderer::new(
        gpu.clone(),
        ph2d_gpu::TextureFormat::Rgba8UnormSrgb,
        atlas,
        16,
    );
    let mut t = sculpted_tool();
    let (mut sessao, mut slot, mut toasts) = (None, None, ToastQueue::default());
    let mut quadro = |t: &mut PainterTool, sessao: &mut Option<PainterGpuPreview>, slot: &mut _| {
        try_drive(sessao, &mut renderer, t, Some(1), slot, &mut toasts)
    };
    assert!(
        quadro(&mut t, &mut sessao, &mut slot),
        "precondição: a tela vai à placa"
    );
    let antes = slot
        .map(|s: PainterPreviewGpu| s.texture_id)
        .expect("o 1.º quadro cria o slot");
    let g0 = sessao.as_ref().map(|s| s.geracao).expect("sessão");
    quadro(&mut t, &mut sessao, &mut slot);
    assert_eq!(
        slot.map(|s| s.texture_id),
        Some(antes),
        "um quadro parado refez o slot"
    );
    let v = &ph2d_gpu::memoria::VIGIA;
    v.anota_em(0);
    let _ = v.mudou_em(0);
    let _ = v.mudou_em(10 * ph2d_gpu::memoria::JANELA_MS);
    assert!(v.geracao() > g0, "o episódio devia ter acabado");
    quadro(&mut t, &mut sessao, &mut slot);
    assert_eq!(
        sessao.as_ref().map(|s| s.geracao),
        Some(v.geracao()),
        "a sessão da placa não se refez no fim do episódio"
    );
    assert!(
        slot.is_some_and(|s| s.texture_id != antes),
        "o slot do preview não se refez (o quadro sem pincelada não recompôs)"
    );
}

/// **B16.12 — a troca GPU→CPU sai do quadro do toque.** Com a aguada ARMADA sobre o papel de cor, a 1.ª
#[test]
fn troca_gpu_cpu_a_aguada_armada_no_papel_de_cor_sai_do_toque() {
    use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PanelEvent, PointerPhase, Tool};
    let cp = |pos: [f32; 2], phase: PointerPhase| CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    };
    let mut t = sculpted_tool();
    t.handle_panel_event(PanelEvent::SelectOption(
        ph2d_tool_painter::ids::PAINTER_WATERCOLOR_PAPER_COLOR_THUMB,
        "153,121,91".into(),
    ));
    assert!(
        gpu_eligible(&t).is_some(),
        "controlo: esculpido, no Digital, sobre o papel castanho, a GPU produz"
    );
    t.set_paint_media(ph2d_tool_painter::PaintMedia::Watercolor);
    assert!(!t.papel_atravessa_vidro(), "controlo: ainda sem vidro");
    let antes = gpu_eligible(&t).is_some();
    t.on_canvas_pointer(cp([10.0, 10.0], PointerPhase::Down));
    let _ = t.take_preview_arc();
    let no_toque = gpu_eligible(&t).is_some();
    t.on_canvas_pointer(cp([38.0, 12.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([38.0, 12.0], PointerPhase::Up));
    assert!(t.papel_atravessa_vidro(), "controlo: a aguada deixou vidro");
    assert_eq!(antes, no_toque, "o toque não pode trocar a pista");
    assert!(
        !antes,
        "a aguada armada no papel de cor já está na CPU antes do toque"
    );
    t.set_paint_media(ph2d_tool_painter::PaintMedia::Digital);
    assert!(
        gpu_eligible(&t).is_none(),
        "com o vidro, o Digital continua na CPU"
    );
}

/// **A placa larga o documento e a CPU compõe a tela inteira NESSE quadro** (B16.12): sem isto a pista
#[test]
fn troca_gpu_cpu_a_placa_que_larga_aquece_a_cpu_no_mesmo_quadro() {
    let mut t = sculpted_tool();
    let _ = t.take_preview_dirty(); // a pista GPU drenou: o composite da CPU caiu
    assert!(
        t.take_preview_arc().is_none(),
        "controlo: sem sujo a CPU não compõe"
    );
    let mut dona = false;
    larga_a_placa(Some(&mut dona), &mut t);
    assert!(
        t.take_preview_arc().is_none(),
        "sem a placa dona antes, nada a aquecer"
    );
    dona = true;
    larga_a_placa(Some(&mut dona), &mut t);
    assert!(!dona, "a placa deixou de ser dona");
    assert!(
        t.take_preview_arc().is_some(),
        "a CPU compõe no quadro da troca"
    );
    assert_eq!(
        t.take_preview_upload_bbox(),
        None,
        "a tela inteira sobe nesse quadro"
    );
}

/// **B20.A2 — a versão de uma camada é um relógio do DOCUMENTO** (`bump_layer_pixels`: sobe a cada toque, de
#[test]
fn salto_de_versao_com_dirty_sobe_a_camada_inteira() {
    let mut t = sculpted_tool();
    assert!(t.take_preview_dirty(), "controlo: o traço sujou a prévia");
    let key = t.layers().active().expect("a camada activa").0;
    let (versao, _, sujo) = t.preview_layer_pixels(key).expect("a activa");
    assert!(sujo.is_some(), "controlo: o instrumento dá um retângulo");
    let sem_anterior = PainterLayerProvider {
        tool: &t,
        anteriores: &[],
        novas: RefCell::default(),
    };
    assert_eq!(
        sem_anterior.layer_pixels(key).and_then(|l| l.dirty),
        None,
        "a camada não subiu na drenagem anterior: tem de subir inteira"
    );
    assert_eq!(sem_anterior.novas.into_inner(), vec![(key, versao)]);
    let anterior = [(key, versao.wrapping_sub(3))];
    let contigua = PainterLayerProvider {
        tool: &t,
        anteriores: &anterior,
        novas: RefCell::default(),
    };
    assert!(
        contigua.layer_pixels(key).and_then(|l| l.dirty).is_some(),
        "subiu na drenagem anterior: remenda só o retângulo, por mais que a versão salte"
    );
}

/// **B20.A1 — a falha da placa entrega o quadro à CPU.** Uma tela mais larga do que a placa aceita: o
#[test]
#[ignore = "needs a real GPU device; run with --ignored on the GPU lane"]
fn a_falha_da_placa_entrega_o_quadro_a_cpu() {
    let Ok(gpu) = GpuContext::new(GpuContext::default_instance(), None) else {
        return;
    };
    let largura = gpu.device.limits().max_texture_dimension_2d + 1;
    let atlas = ph2d_render::TextureAtlas::new(&gpu, 64);
    let mut renderer = SpriteRenderer::new(
        gpu.clone(),
        ph2d_gpu::TextureFormat::Rgba8UnormSrgb,
        atlas,
        16,
    );
    let mut t = esculpido_de(largura, 48);
    assert!(
        gpu_eligible(&t).is_some(),
        "precondição: a tela iria à placa"
    );
    let (mut sessao, mut slot, mut toasts) = (None, None, ToastQueue::default());
    let g0 = ph2d_gpu::memoria::geracao();
    for quadro in 0..2 {
        assert!(
            !try_drive(
                &mut sessao,
                &mut renderer,
                &mut t,
                Some(1),
                &mut slot,
                &mut toasts
            ),
            "quadro {quadro}: a placa recusou e ainda se diz dona"
        );
        assert!(slot.is_none(), "quadro {quadro}: sem slot da placa");
        if quadro == 0 {
            assert!(t.take_preview_arc().is_some(), "a CPU compõe nesse quadro");
            assert_eq!(t.take_preview_upload_bbox(), None, "a tela inteira sobe");
        }
    }
    let avisos: Vec<_> = toasts.iter().map(|t| t.message.clone()).collect();
    // Outro teste pode fechar um episódio sem memória no meio (`VIGIA` global): aí a placa tenta de novo.
    if ph2d_gpu::memoria::geracao() == g0 {
        assert_eq!(
            avisos.len(),
            1,
            "um aviso só, não um por quadro: {avisos:?}"
        );
    }
}

/// **B20.1 — a região suja (+1 px, e a luz +1 texel) dá o slot da semente inteira, ao bit.** Duas pistas
/// com o mesmo documento e os mesmos toques: a do caminho antigo (`SEMPRE_INTEIRA`) é o oráculo.
#[test]
#[ignore = "needs a real GPU device; run with --ignored on the GPU lane"]
fn a_regiao_suja_e_a_tela_inteira_ao_bit() {
    use crate::painter_preview_pipeline_tests::{assert_screen_equals, cp, impasto_tool};
    use ph2d_editor_core::tool::{CanvasPaintTool, PointerPhase};
    let Ok(gpu) = GpuContext::new(GpuContext::default_instance(), None) else {
        return;
    };
    for camadas in [1usize, 4] {
        let pista = || {
            let mut t = impasto_tool(240);
            for i in 1..camadas {
                t.add_raster_layer(format!("c{i}"));
            }
            let r = SpriteRenderer::new(
                gpu.clone(),
                ph2d_render::GameRt::FORMAT,
                ph2d_render::TextureAtlas::dummy(&gpu),
                8,
            );
            (t, r, None, None, ToastQueue::default())
        };
        let (mut pistas, mut parciais) = ([pista(), pista()], 0);
        for (y, x0) in [(90.0f32, 40.0f32), (150.0, 60.0), (100.0, 30.0)] {
            for passo in 0u8..=7 {
                let fase = match passo {
                    0 => PointerPhase::Down,
                    7 => PointerPhase::Up,
                    _ => PointerPhase::Move,
                };
                let mut telas = Vec::new();
                for (antigo, (t, r, s, g, toasts)) in pistas.iter_mut().enumerate() {
                    SEMPRE_INTEIRA.set(antigo == 1);
                    let dona = s.as_ref().is_some_and(|s: &PainterGpuPreview| s.dona);
                    t.on_canvas_pointer(cp([x0 + 24.0 * f32::from(passo), y], fase));
                    assert!(try_drive(s, r, t, Some(1), g, toasts), "a placa é dona");
                    parciais +=
                        usize::from(antigo == 0 && dona && t.preview_gpu_region().is_some());
                    let id = g.expect("o slot da placa").texture_id;
                    telas.push(r.individual().readback(&gpu, id).expect("readback").2);
                }
                SEMPRE_INTEIRA.set(false);
                let ctx = format!("{camadas} camadas, traço y={y}, passo {passo}");
                assert_screen_equals(&telas[0], &telas[1], 240, &ctx);
            }
        }
        assert!(parciais > 10, "controlo: só {parciais} quadros pela região");
    }
}
