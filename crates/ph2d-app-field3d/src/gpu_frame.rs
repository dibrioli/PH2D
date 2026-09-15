//! ⭐⭐⭐ **A COSTURA: o quadro assente passa pelo dispositivo** (`docs/Render3d/05` §36).
//!
//! # As três condições, e nenhuma é opcional
//!
//! 1. **há adaptador** — sem GPU o módulo corre como sempre;
//! 2. **o quadro é o ASSENTE** — o de movimento fica **byte-idêntico** ao de hoje, que é a cerca
//!    que impede uma regressão no gesto (a lição do §32);
//! 3. ⛔ **a peça não tem ESCULTURA** ([`ph2d_field_gpu::supports`]) — ela compila para espaço
//!    vazio na fita, e sem esta condição ela desapareceria em silêncio.
//!
//! # ⚠️ O traçador VIVE entre quadros
//!
//! Abrir o dispositivo e compilar o shader a cada chamada custa mais do que a CPU inteira (medido:
//! `130 ms` contra `13`). ⇒ ele mora num `Arc<Mutex<_>>` do módulo e atravessa a fronteira da
//! thread como ponteiro, como a tabela de materiais e a cache de fitas já fazem.

use std::sync::{Arc, Mutex};

/// O traçador de dispositivo do módulo — `None` quando não há adaptador.
///
/// ⚠️ **Um por MÓDULO e não por viewport:** o que ele guarda é o dispositivo e o cache de
/// pipelines, e os dois são por-máquina. *Quatro vistas da mesma peça partilham a estrutura, logo
/// partilham o pipeline.*
pub type SharedTracer = Arc<Mutex<ph2d_field_gpu::trace::Tracer>>;

/// ⭐⭐⭐ **O traçador da MÁQUINA, aberto uma vez e para sempre.**
///
/// ⛔⛔ **Ele NÃO vive no [`crate::smoke::Smoke`], e a primeira versão vivia.** O `Smoke` é um
/// `thread_local`, e um `wgpu::Device` lá dentro é destruído **na saída da thread** — onde outros
/// `thread_local` já morreram. Resultado medido: *«cannot access a Local Storage value during or
/// after destruction»*, e a suíte inteira do módulo a devolver **`0 passaram · 0 falharam`**.
///
/// ⚠️ *Um binário que não corre teste nenhum lê-se, num relatório, quase como um que passou.*
///
/// ⭐ E o sítio certo não é uma correcção de conveniência: **o dispositivo é da MÁQUINA**, não do
/// módulo nem do viewport. Quatro vistas da mesma peça partilham a estrutura, logo partilham o
/// pipeline — e nada nele é estado que o artista tenha pousado.
#[must_use]
pub fn shared() -> Option<&'static SharedTracer> {
    static TRACER: std::sync::OnceLock<Option<SharedTracer>> = std::sync::OnceLock::new();
    TRACER
        .get_or_init(|| {
            enabled()
                .then(ph2d_field_gpu::trace::Tracer::new)
                .flatten()
                .map(|t| Arc::new(Mutex::new(t)))
        })
        .as_ref()
}

/// ⚠️ **A porta de bissecção.** `PH2D_FIELD_GPU=0` devolve o módulo ao traçado de CPU inteiro —
/// e é ela que responde a *«piorou»* sem ninguém ter de adivinhar qual metade.
#[must_use]
pub fn enabled() -> bool {
    static LIGADO: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *LIGADO.get_or_init(|| {
        std::env::var("PH2D_FIELD_GPU").is_ok_and(|v| v != "0")
            || std::env::var("PH2D_FIELD_GPU").is_err()
    })
}

/// ⭐⭐⭐ **Este quadro vai para o dispositivo?** — as três condições da nota do módulo.
#[must_use]
pub fn takes_the_frame(
    tracer: Option<&SharedTracer>,
    assente: bool,
    doc: &ph2d_field::FieldDoc,
) -> bool {
    tracer.is_some() && assente && ph2d_field_gpu::supports(doc)
}

/// O G-buffer e a luz, marchados no dispositivo. `None` quando alguma coisa faltar — e o chamador
/// cai na CPU, que é o caminho de sempre.
#[must_use]
pub fn march(
    tracer: &SharedTracer,
    doc: &ph2d_field::FieldDoc,
    reg: &ph2d_field_eval::hybrid::Registry,
    cam: &ph2d_field_render::Orbit,
    lamps: &[[f32; 3]],
    w: u32,
    h: u32,
) -> Option<(ph2d_field_render::Gbuffer, ph2d_field_render::Shadows)> {
    let campo = ph2d_field_eval::Field::new(doc);
    let fita = campo.tape_wgsl()?;
    let bola = ph2d_field_eval::bounds::bounding_ball(doc, reg)?;
    let (right, up, fwd) = cam.basis();
    let screen = ph2d_field_render::Screen::new(w, h, cam.half_extent);
    let sharp = ph2d_field_render::Sharpness::for_frame(cam.half_extent, w.min(h) as usize);
    let passo = ph2d_field_eval::safe_march_step(doc);
    let shrink = ph2d_field_eval::field_shrink(doc, reg);
    let setup = ph2d_field_gpu::trace::MarchSetup {
        half_extent: cam.half_extent,
        half_px: screen.half(),
        target: cam.target,
        right,
        up,
        fwd,
        ortho_start: ph2d_field_render::ORTHO_START,
        eye_distance: cam.eye_distance().unwrap_or(0.0),
        hit_eps: sharp.hit,
        normal_eps: sharp.normal,
        step: passo,
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        budget: ((ph2d_field_render::MAX_STEPS as f32) * shrink.max(1.0)
            / passo.clamp(f32::EPSILON, 1.0))
        .ceil() as u32,
        t_max: ph2d_field_render::T_MAX,
        // ⚠️ **UMA lâmpada por enquanto** — o shader tem um canal de sombra. Com duas, a segunda
        // ficaria sem sombra **em silêncio**, e é por isso que o chamador cai na CPU em vez de a
        // ignorar.
        lamp: *lamps.first()?,
        ball_center: bola.center,
        ball_radius: bola.radius,
        ao_rays: ph2d_field_render::OCCLUSION_PASSES,
        ao_reach: ph2d_field_render::OCCLUSION_REACH * cam.half_extent,
        edge_cos: ph2d_field_render::EDGE_COS,
    };
    let dev = tracer.lock().ok()?.frame(&fita, setup, w, h);
    Some(dev.to_cpu(cam, screen))
}

#[cfg(test)]
#[path = "gpu_frame_tests.rs"]
mod tests;
