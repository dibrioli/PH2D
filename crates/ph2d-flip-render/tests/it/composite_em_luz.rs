//! ⭐ ADR-0177 P4 — as camadas do Flip juntam-se em LUZ. O rasterizador junta os traços de UMA
//! camada em linear 16F; uma camada por cima tem de se juntar como um traço na mesma camada (o
//! report do Painter, ao contrário: lá o pincel é codificado e as camadas passaram a sê-lo). A P1
//! mudou o compositor partilhado para tons de ecrã e o Flip herdou-o em silêncio: branco a 50 %
//! numa camada por cima de preto dava `0,216` linear, na mesma camada `0,5`.

use super::composite_blend::{Dummy, GAME_RT, H, W, cleared_target, gpu, pixel_camera, readback};
use ph2d_core::Vec2;
use ph2d_flip::{Fill, FlipDrawing, FlipStroke, Point, Rgba};
use ph2d_flip_render::{FlipCompose, FlipRenderer, compositor_do_flip, pack_drawing};
use ph2d_gpu::GpuContext;
use ph2d_painter_effects::BlendMode;
use ph2d_render::layer_compositor::{LayerCompositeError, LayerOp, Region};

/// Um quadrado preenchido com a opacidade `op` no preenchimento e no contorno (o `filled_square`
/// do `composite_blend` com a opacidade como parâmetro).
fn quadrado(min: Vec2, max: Vec2, color: Rgba, op: f32) -> FlipDrawing {
    let mut s = FlipStroke::new();
    for pos in [min, Vec2::new(max.x, min.y), max, Vec2::new(min.x, max.y)] {
        s.push_point(Point {
            pos,
            width: 0.5,
            opacity: op,
            color,
        });
    }
    s.closed = true;
    s.hardness = 1.0;
    s.fill = Some(Fill { color, opacity: op });
    let mut d = FlipDrawing::default();
    d.strokes.push(s);
    d
}

/// Compõe `camadas` (Normal, opacidade 1) pela porta do Flip e devolve o alvo 16F linear.
fn compoe(gpu: &GpuContext, camadas: &[FlipDrawing]) -> Vec<f32> {
    let mut fr = FlipRenderer::new(&gpu.device, GAME_RT);
    let mut fc = FlipCompose::new(&gpu.device, GAME_RT);
    let mut comp = compositor_do_flip(gpu);
    let ops: Vec<LayerOp> = (1..=camadas.len() as u64)
        .map(|key| LayerOp::Layer {
            mask: None,
            clipping: false,
            key,
            blend_mode: BlendMode::Normal.to_u8(),
            opacity: 1.0,
        })
        .collect();
    let cam = pixel_camera();
    for (key, d) in (1u64..).zip(camadas) {
        let data = pack_drawing(d);
        let slice = fc.stage_layer(&gpu.device, &gpu.queue, &mut fr, &cam, &data, (W, H));
        comp.inject_slice_from_texture(gpu, &ops, key, slice, W, H, (0, 0, W, H), 0)
            .expect("inject");
    }
    let dummy = vec![0u8; (W * H * 4) as usize];
    comp.composite(gpu, &ops, &Dummy { px: &dummy }, W, H, Region::full(W, H))
        .expect("composite");
    let (alvo, vista) = cleared_target(gpu);
    fc.blit(
        &gpu.device,
        &gpu.queue,
        comp.output_texture().expect("saída"),
        &vista,
    );
    readback(gpu, &alvo)
}

#[test]
#[ignore = "precisa de adapter GPU; roda com --ignored"]
fn a_camada_de_cima_junta_se_como_um_traco_na_mesma_camada() {
    let Some(gpu) = gpu() else {
        eprintln!("sem adapter GPU — pulando");
        return;
    };
    let fundo = quadrado(
        Vec2::new(4.0, 4.0),
        Vec2::new(60.0, 60.0),
        Rgba::new(0.0, 0.0, 0.0, 1.0),
        1.0,
    );
    let branco = quadrado(
        Vec2::new(16.0, 16.0),
        Vec2::new(48.0, 48.0),
        Rgba::new(1.0, 1.0, 1.0, 1.0),
        0.5,
    );
    let mut na_mesma = fundo.clone();
    na_mesma.strokes.extend(branco.strokes.iter().cloned());
    let uma = compoe(&gpu, &[na_mesma]);
    let duas = compoe(&gpu, &[fundo, branco]);
    let i = ((32 * W + 32) * 4) as usize;
    // Na mesma camada: `0,5` linear (premult-over em 16F). A fatia de 8 bits e a ida e volta sRGB
    // da camada de cima custam ≤ 1 degrau de sRGB8 em 0,5 (≈ 0,0045 linear).
    assert!((uma[i] - 0.5).abs() < 0.01, "na mesma camada = {}", uma[i]);
    assert!(
        (duas[i] - uma[i]).abs() < 0.01,
        "a camada de cima = {} × a mesma camada = {} (em tons de ecrã seria ~0,216)",
        duas[i],
        uma[i]
    );
}

/// A porta do Flip junta CAMADAS: um ajuste nela seria lido contra um acumulador em luz que ele
/// não conhece — recusa-se alto, nunca em silêncio.
#[test]
#[ignore = "precisa de adapter GPU; roda com --ignored"]
fn a_porta_do_flip_recusa_um_ajuste() {
    let Some(gpu) = gpu() else {
        eprintln!("sem adapter GPU — pulando");
        return;
    };
    let mut comp = compositor_do_flip(&gpu);
    let ops = [LayerOp::Adjustment {
        mask: None,
        kind: 2,
        params: [0.0; 3],
        blend_mode: 0,
        opacity: 1.0,
    }];
    let dummy = vec![0u8; (W * H * 4) as usize];
    assert_eq!(
        comp.composite(&gpu, &ops, &Dummy { px: &dummy }, W, H, Region::full(W, H)),
        Err(LayerCompositeError::AdjustmentInLightSpace)
    );
}
