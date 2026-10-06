//! Display gates, a metade que MEDE — o preço que cada produtor cobra pelo mesmo traço.
//!
//! Irmão do `painter_preview_handoff_tests.rs`, cortado dele pelo teto de LOC da shell (HR-18) e por
//! ASSUNTO: lá moram as coisas que o arquivo **afirma** (a dança de handoff, a paridade dos dois
//! produtores, o papel), aqui as que ele **mede**. As duas metades compartilham o harness — `app_frame`
//! e as fixtures continuam vindo dos vizinhos, e nenhum caminho de `use` de fora mudou.
//!
//! Todas `#[ignore]`: precisam de adapter, e rodam com `--release --ignored`.

use crate::painter_preview_handoff_tests::app_frame;
use crate::painter_preview_pipeline_tests::{cp, impasto_tool};
use ph2d_editor_core::tool::{CanvasPaintTool, PointerPhase};
use ph2d_preview_slot::PreviewGpu as PainterPreviewGpu;
use ph2d_tool_painter::PainterTool;

/// **Where a MASKED stroke's frame goes** — the census behind the mask-path FPS report.
///
/// A masked layer is GPU-representable (Ondas 1-2), so a stroke on it takes the GPU PRODUCER, not
/// the CPU trivial lane that Onda 5a made footprint-bound. This measures that producer's per-move
/// cost at two canvas sizes. If it grows ~4x with the canvas, the cost is plane-bound — the whole
/// changed layer re-uploaded per frame (`ensure_slice` -> full `write_texture`) plus the always-full
/// composite (`try_drive` passes `seed_full = true`), the two O(canvas) costs Onda 5b must make
/// partial. `#[ignore]`: needs a GPU adapter; run with `--release --ignored`.
/// **The sculpted stroke, priced on BOTH producers** — the number the routing was never given.
///
/// `gpu_eligible` sends a sculpted document to the GPU on a premise that is true as far as it goes: the
/// CPU lane refuses its zero-composite fast path when there is relief. What the routing never priced is
/// what the GPU lane pays to GET there. `impasto_gpu_planes` folds the composed relief **canvas-wide** on
/// the CPU every dirty frame so the shader has planes to read — measured alone, in
/// `ph2d_tool_painter`'s `measure_the_impasto_fold`, at **202 ms per frame at 4096²**, of which 180 ms is
/// the per-texel walk and 0.15 ms the allocation. The CPU lane composites and lights only the DIRTY RECT.
///
/// So the two lanes are measured on the same stroke, per move, at two canvas sizes. The comparison is the
/// point: a lane is only worth routing to if it is the faster one, and "the CPU pays a full composite" —
/// the sentence the routing rests on — is a claim about the CPU lane that this puts a number on.
///
/// `#[ignore]`: needs a GPU adapter; run with `--release --ignored`.
#[test]
#[ignore = "perf measurement (GPU adapter) — run with --release --ignored"]
fn measure_the_sculpted_stroke_on_both_producers() {
    let Ok(gpu) = ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None) else {
        eprintln!("no GPU adapter on this machine — nothing to measure");
        return;
    };
    /// A sculpted stroke driven move by move; `gpu` selects the producer.
    ///
    /// The CPU arm drains the tool and DROPS the Arc, which is what the shell does since Onda 5a gave
    /// it a buffer of its own — holding it would price the copy-on-write that fix removed and report a
    /// lane the product no longer takes.
    fn per_move(ctx: &ph2d_gpu::GpuContext, size: u32, on_gpu: bool) -> (f64, bool) {
        let mut renderer = ph2d_render::SpriteRenderer::new(
            ctx.clone(),
            ph2d_render::GameRt::FORMAT,
            ph2d_render::TextureAtlas::dummy(ctx),
            8,
        );
        let mut t = impasto_tool(size);
        let (mut session, mut preview, mut toasts) =
            (None, None, ph2d_editor_core::toast::ToastQueue::default());
        let mut preview_gpu: Option<PainterPreviewGpu> = None;
        let mid = (size / 2) as f32;
        t.on_canvas_pointer(cp([60.0, mid], PointerPhase::Down));
        let mut owns = false;
        let mut moves = Vec::new();
        for i in 1..=20u32 {
            let x = 60.0 + 40.0 * (i as f32);
            let t0 = std::time::Instant::now();
            t.on_canvas_pointer(cp([x, mid], PointerPhase::Move));
            if on_gpu {
                owns = app_frame(
                    &mut renderer,
                    &mut t,
                    &mut session,
                    &mut preview,
                    &mut preview_gpu,
                    &mut toasts,
                );
                // Make the queued GPU work COMPLETE, so the number includes device execution and not
                // just the CPU-side encode — the proxy gap that hid the mask path's real cost.
                let _ = ctx.device.poll(wgpu::PollType::wait_indefinitely());
            } else {
                let _ = t.take_preview_arc();
            }
            moves.push(t0.elapsed().as_secs_f64() * 1e3);
        }
        moves.sort_by(|a, b| a.partial_cmp(b).expect("finite"));
        (moves[moves.len() / 2], owns)
    }
    eprintln!("\n[sculpted-move] canvas    gpu ms   cpu ms   gpu/cpu   gpu_owns");
    for size in [2048u32, 4096] {
        let (g, owns) = per_move(&gpu, size, true);
        let (c, _) = per_move(&gpu, size, false);
        eprintln!(
            "[sculpted-move] {size:<8} {g:>7.3} {c:>8.3} {:>9.1}x   {owns}",
            g / c.max(1e-6)
        );
    }
}

#[test]
#[ignore = "perf measurement (GPU adapter) — run with --release --ignored"]
fn measure_the_masked_stroke_on_the_gpu_producer() {
    let Ok(gpu) = ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None) else {
        eprintln!("no GPU adapter on this machine — nothing to measure");
        return;
    };
    fn per_move(gpu: &ph2d_gpu::GpuContext, size: u32) -> (f64, bool) {
        use ph2d_editor_core::tool::RasterEditTool;
        let mut renderer = ph2d_render::SpriteRenderer::new(
            gpu.clone(),
            ph2d_render::GameRt::FORMAT,
            ph2d_render::TextureAtlas::dummy(gpu),
            8,
        );
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
        t.set_brush_size_px(16.0);
        t.add_mask_to_active().expect("a mask on the active layer");
        let (mut session, mut preview, mut toasts) =
            (None, None, ph2d_editor_core::toast::ToastQueue::default());
        let mut preview_gpu: Option<PainterPreviewGpu> = None;
        let mid = (size / 2) as f32;
        t.on_canvas_pointer(cp([40.0, mid], PointerPhase::Down));
        let mut owns = false;
        let mut moves = Vec::new();
        for i in 1..=24u32 {
            let x = 40.0 + 20.0 * (i as f32);
            let t0 = std::time::Instant::now();
            t.on_canvas_pointer(cp([x, mid], PointerPhase::Move));
            owns = app_frame(
                &mut renderer,
                &mut t,
                &mut session,
                &mut preview,
                &mut preview_gpu,
                &mut toasts,
            );
            // Force the queued GPU work (composite compute + the slot copy) to COMPLETE, so the
            // measurement includes GPU execution and not just the CPU-side encode + staging. Without
            // this, `queue.submit` returns immediately and the per-frame full-canvas composite + full
            // 64 MiB slot copy `try_drive` issues (`seed_full = true`) are invisible — the proxy gap
            // that hid the real mask-path cost behind a "fast" CPU-side number.
            let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
            moves.push(t0.elapsed().as_secs_f64() * 1e3);
        }
        moves.sort_by(|a, b| a.partial_cmp(b).expect("finite"));
        (moves[moves.len() / 2], owns)
    }
    let (ms_2k, owns_2k) = per_move(&gpu, 2048);
    let (ms_4k, owns_4k) = per_move(&gpu, 4096);
    eprintln!(
        "[masked-move] 2048²={ms_2k:.3} ms (gpu_owns={owns_2k})  4096²={ms_4k:.3} ms \
         (gpu_owns={owns_4k})  ratio {:.1}x",
        ms_4k / ms_2k.max(1e-6)
    );
}

/// **O quadro de um documento COM PAPEL nos dois produtores** (2026-10-06): até aqui o papel mandava o
/// documento para a CPU (a GPU recusava-se); agora o compositor compõe sobre ele e a GPU produz. Os
/// dois produtores, no MESMO processo, por movimento de um traço, no Digital e no Impasto, a 2048² e
/// 4096² — três rodadas intercaladas, o mínimo das medianas com a mediana ao lado. A CPU é o «antes»
/// do produto, a GPU o «depois».
///
/// Medido 2026-10-06 (`smoke`, load 4,7–5,8), GPU · CPU em ms por movimento: Digital `0,505 · 0,274`
/// (2048²) e `1,157 · 0,255` (4096²); Impasto `0,970 · 1,482` e `1,948 · 1,495`. ⇒ uma camada LISA com
/// papel ficou na CPU (`preview_layer_stack_is_trivial`), e desde então a linha do Digital lê
/// `gpu produz = false`: a coluna da GPU é ali a pista que o produto toma.
///
/// `cargo test -p ph2d-app-painter --profile smoke --lib measure_o_quadro_com_papel -- --ignored --nocapture --test-threads=1`
#[test]
#[ignore = "perf measurement (GPU adapter) — run with --ignored"]
fn measure_o_quadro_com_papel_nos_dois_produtores() {
    let Ok(gpu) = ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None) else {
        eprintln!("no GPU adapter on this machine — nothing to measure");
        return;
    };
    fn por_move(ctx: &ph2d_gpu::GpuContext, size: u32, impasto: bool, na_gpu: bool) -> (f64, bool) {
        let mut renderer = ph2d_render::SpriteRenderer::new(
            ctx.clone(),
            ph2d_render::GameRt::FORMAT,
            ph2d_render::TextureAtlas::dummy(ctx),
            8,
        );
        let mut t = if impasto {
            impasto_tool(size)
        } else {
            let mut t = PainterTool::default();
            ph2d_editor_core::tool::RasterEditTool::set_source(
                &mut t,
                vec![255u8; (size * size * 4) as usize],
                size,
                size,
            );
            t.set_brush_size_px(40.0);
            t
        };
        ph2d_editor_core::tool::Tool::handle_panel_event(
            &mut t,
            ph2d_editor_core::tool::PanelEvent::SelectOption(
                ph2d_tool_painter::ids::PAINTER_WATERCOLOR_PAPER_COLOR_THUMB,
                "196,170,120".into(),
            ),
        );
        assert!(t.papel().is_some(), "controlo: o documento tem papel");
        let (mut session, mut preview, mut toasts) =
            (None, None, ph2d_editor_core::toast::ToastQueue::default());
        let mut preview_gpu: Option<PainterPreviewGpu> = None;
        let mid = (size / 2) as f32;
        t.on_canvas_pointer(cp([60.0, mid], PointerPhase::Down));
        let mut owns = false;
        let mut moves = Vec::new();
        for i in 1..=20u32 {
            let x = 60.0 + 40.0 * (i as f32);
            let t0 = std::time::Instant::now();
            t.on_canvas_pointer(cp([x, mid], PointerPhase::Move));
            if na_gpu {
                owns = app_frame(
                    &mut renderer,
                    &mut t,
                    &mut session,
                    &mut preview,
                    &mut preview_gpu,
                    &mut toasts,
                );
                let _ = ctx.device.poll(wgpu::PollType::wait_indefinitely());
            } else {
                let _ = t.take_preview_arc();
            }
            moves.push(t0.elapsed().as_secs_f64() * 1e3);
        }
        moves.sort_by(|a, b| a.partial_cmp(b).expect("finite"));
        (moves[moves.len() / 2], owns)
    }
    let leitura = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    eprintln!("\n[papel] loadavg {}", leitura.trim());
    let casos = [(false, 2048u32), (false, 4096), (true, 2048), (true, 4096)];
    let mut gpu_ms = vec![Vec::new(); casos.len()];
    let mut cpu_ms = vec![Vec::new(); casos.len()];
    let mut gpu_produz = vec![true; casos.len()];
    for _rodada in 0..3 {
        for (k, &(impasto, size)) in casos.iter().enumerate() {
            let (g, owns) = por_move(&gpu, size, impasto, true);
            let (c, _) = por_move(&gpu, size, impasto, false);
            gpu_ms[k].push(g);
            cpu_ms[k].push(c);
            gpu_produz[k] &= owns;
        }
    }
    let min_med = |v: &mut Vec<f64>| {
        v.sort_by(|a, b| a.partial_cmp(b).expect("finite"));
        (v[0], v[v.len() / 2])
    };
    eprintln!(
        "[papel] meio     canvas   gpu ms (mín · med)   cpu ms (mín · med)   gpu/cpu   gpu produz"
    );
    for (k, &(impasto, size)) in casos.iter().enumerate() {
        let (gm, gd) = min_med(&mut gpu_ms[k]);
        let (cm, cd) = min_med(&mut cpu_ms[k]);
        eprintln!(
            "[papel] {:<8} {size:<8} {gm:>7.3} · {gd:>7.3}      {cm:>7.3} · {cd:>7.3}     {:>5.2}×   {}",
            if impasto { "Impasto" } else { "Digital" },
            gm / cm.max(1e-6),
            gpu_produz[k]
        );
    }
    let leitura = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    eprintln!("[papel] loadavg {}", leitura.trim());
}
