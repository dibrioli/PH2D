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
pub fn takes_the_frame(tracer: Option<&SharedTracer>, doc: &ph2d_field::FieldDoc) -> bool {
    tracer.is_some() && ph2d_field_gpu::supports(doc)
}

/// O G-buffer e a luz, marchados no dispositivo. `None` quando alguma coisa faltar — e o chamador
/// cai na CPU, que é o caminho de sempre.
#[must_use]
// A peça, o registo, a vista, as luzes, a tela e a bandeira — oito coisas que um quadro precisa,
// e nenhuma delas pertence a outra. Agrupá-las numa struct só as renomearia.
#[allow(clippy::too_many_arguments)]
pub fn march(
    tracer: &SharedTracer,
    doc: &ph2d_field::FieldDoc,
    reg: &ph2d_field_eval::hybrid::Registry,
    cam: &ph2d_field_render::Orbit,
    lamps: &[[f32; 3]],
    w: u32,
    h: u32,
    antialias: bool,
) -> Option<(ph2d_field_render::Gbuffer, ph2d_field_render::Shadows)> {
    let (fita, setup) = pedido(doc, reg, cam, *lamps.first()?, w, h, antialias)?;
    let screen = ph2d_field_render::Screen::new(w, h, cam.half_extent);
    let dev = tracer.lock().ok()?.frame(&fita, setup, w, h);
    Some(dev.to_cpu(cam, screen))
}

/// ⭐⭐⭐ **A IMAGEM, pintada no dispositivo** — o quadro inteiro sem o G-buffer atravessar o
/// barramento. `None` pelas mesmas razões do [`march`].
///
/// ⚠️ **As luzes chegam como [`ph2d_field_render::PointLamp`]** e não como posições: o pintor
/// precisa da radiância, e derivá-la noutro sítio seria a segunda resposta à mesma pergunta.
/// ⛔ **UMA lâmpada por enquanto** — o shader tem um canal de sombra. Com duas, a segunda ficaria
/// sem sombra **em silêncio**.
#[must_use]
// A peça, o registo, a vista, as luzes, os materiais, o olhar, o fundo, a tela e a bandeira.
#[allow(clippy::too_many_arguments)]
pub fn paint(
    tracer: &SharedTracer,
    doc: &ph2d_field::FieldDoc,
    reg: &ph2d_field_eval::hybrid::Registry,
    cam: &ph2d_field_render::Orbit,
    points: &[ph2d_field_render::PointLamp],
    surfaces: &ph2d_field_render::Surfaces<'_>,
    look: ph2d_view_transform::Look,
    background: [u8; 4],
    w: u32,
    h: u32,
    antialias: bool,
) -> Option<ph2d_field_gpu::trace::Pintado> {
    let lampada = points.first()?;
    let (fita, setup) = pedido(doc, reg, cam, lampada.world, w, h, antialias)?;
    let materiais = packed(surfaces.all);
    let tabelas = crate::studio_wgsl::tables();
    let pintor = ph2d_field_gpu::paint::PaintSetup {
        owners: surfaces.owners,
        materials: &materiais,
        env_source: crate::studio_wgsl::SOURCE,
        env_consts: &crate::studio_wgsl::constants(),
        env_tables: &tabelas,
        lamp_radiance: lampada.radiance_at_one,
        stops: look.exposure_stops,
        view: ph2d_view_transform::wgsl::view_code(look.view),
        background,
        // ⚠️ **A largura da fronteira de cor sai do [`ph2d_field_render::boundary_world`]**, que é
        // quem a deriva — o factor dela foi VARRIDO e mora lá, não aqui.
        pixel_world: ph2d_field_render::boundary_world(cam.half_extent, w.min(h)),
    };
    Some(
        tracer
            .lock()
            .ok()?
            .painted_frame(&fita, setup, &pintor, w, h),
    )
}

/// ⭐ **Os materiais no formato que o dispositivo lê** — o [`ph2d_material::wgsl::pack`] com o
/// encolhimento do lóbulo que o céu do produto pede.
///
/// ⚠️ **O `lobe_shrink` é `f64` e constante por MATERIAL**, logo viaja pronto: correr no
/// dispositivo o que já está calculado poria a mesma conta a dar o mesmo número dois milhões de
/// vezes ([`crate::studio_wgsl`]).
#[must_use]
pub fn packed(all: &[ph2d_material::Surface]) -> Vec<f32> {
    let mut v = Vec::with_capacity(all.len() * ph2d_material::wgsl::PACKED);
    for s in all {
        let (main, coat) = ph2d_material::wgsl::alphas(s);
        v.extend_from_slice(&ph2d_material::wgsl::pack(
            s,
            ph2d_material::wgsl::EnvLobe {
                main: crate::render_light::lobe_shrink(main),
                coat: crate::render_light::lobe_shrink(coat),
            },
        ));
    }
    v
}

/// O que a marcha precisa de saber, derivado uma vez — a fita e o pedido.
///
/// ⚠️ **Ele é partilhado pelo [`march`] e pelo [`paint`] de propósito:** os dois têm de marchar
/// exactamente a mesma coisa, senão o gate que compara as duas imagens mede também a geometria.
fn pedido(
    doc: &ph2d_field::FieldDoc,
    reg: &ph2d_field_eval::hybrid::Registry,
    cam: &ph2d_field_render::Orbit,
    lamp: [f32; 3],
    w: u32,
    h: u32,
    antialias: bool,
) -> Option<(
    ph2d_field_eval::wgsl::TapeWgsl,
    ph2d_field_gpu::trace::MarchSetup,
)> {
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
        antialias,
        step: passo,
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        budget: ((ph2d_field_render::MAX_STEPS as f32) * shrink.max(1.0)
            / passo.clamp(f32::EPSILON, 1.0))
        .ceil() as u32,
        t_max: ph2d_field_render::T_MAX,
        lamp,
        ball_center: bola.center,
        ball_radius: bola.radius,
        ao_rays: ph2d_field_render::OCCLUSION_PASSES,
        ao_reach: ph2d_field_render::OCCLUSION_REACH * cam.half_extent,
        edge_cos: ph2d_field_render::EDGE_COS,
    };
    Some((fita, setup))
}

#[cfg(test)]
#[path = "gpu_frame_tests.rs"]
mod tests;

/// ⭐⭐⭐ **O PASSE QUE PINTA, nos dois motores** — irmão por assunto do gate do G-buffer.
#[cfg(test)]
#[path = "paint_parity_tests.rs"]
mod paint_parity_tests;
