//! ⭐⭐⭐ **A COSTURA: o quadro assente passa pelo dispositivo** (`docs/Render3d/05` §36).
//!
//! # As três condições, e nenhuma é opcional
//!
//! 1. **há adaptador** — sem GPU o módulo corre como sempre;
//! 2. **o quadro é o ASSENTE** — o de movimento fica **byte-idêntico** ao de hoje, que é a cerca
//!    que impede uma regressão no gesto (a lição do §32);
//! 3. ⭐ **a peça é DESENHÁVEL lá** ([`ph2d_field_gpu::supports`]) — e desde 2026-09-15 uma
//!    ESCULTURA já é: ela atravessa como grade ([`ph2d_field_gpu::sculpt`]), e o que a porta
//!    pergunta é se a folha amostrada sabe entregá-la.
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
    doc: &ph2d_field::FieldDoc,
    reg: &ph2d_field_eval::hybrid::Registry,
) -> bool {
    tracer.is_some() && ph2d_field_gpu::supports(doc, reg)
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
    let cabem = lamps_that_fit(tracer, w, h);
    let (campo, fita, setup) = pedido(
        doc,
        reg,
        cam,
        lamps,
        cabem,
        Sonda::default(),
        w,
        h,
        antialias,
    )?;
    let screen = ph2d_field_render::Screen::new(w, h, cam.half_extent);
    let dev = tracer
        .lock()
        .ok()?
        .frame(&fita, campo.sculpts(), setup, w, h);
    Some(dev.to_cpu(cam, screen))
}

/// ⭐⭐⭐ **A IMAGEM, pintada no dispositivo** — o quadro inteiro sem o G-buffer atravessar o
/// barramento. `None` pelas mesmas razões do [`march`].
///
/// ⚠️ **As luzes chegam como [`ph2d_field_render::PointLamp`]** e não como posições: o pintor
/// precisa da radiância, e derivá-la noutro sítio seria a segunda resposta à mesma pergunta.
/// ⛔ Acima de [`ph2d_field_gpu::trace::MAX_LAMPS`] devolve `None` e o chamador cai na CPU.
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
    paint_com(
        tracer,
        doc,
        reg,
        cam,
        points,
        surfaces,
        look,
        background,
        w,
        h,
        antialias,
        Sonda::default(),
    )
}

/// ⭐⭐ **O que a SONDA de calibração pode desligar — e nada disto é um caminho de produto.**
///
/// ⚠️ *Um número que ninguém consegue voltar a medir é um palpite com data* — esta porta existe para
/// que a comparação entre as duas ordens da fita se possa refazer noutra máquina, na mesma corrida.
///
/// ⛔⛔ **Ela tinha um segundo campo, `tecto`, e ele MORREU com o tecto** (2026-09-15): a cerca que
/// mandava uma fita larga para a CPU saiu quando a medição que a sustentava foi corrigida — ver a
/// nota no lugar do `MAX_GUARDADOS` na [`ph2d_field_gpu`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Sonda {
    /// `false` = a fita sai na ordem **CRUA** da travessia — ver
    /// [`ph2d_field_eval::tape_schedule`].
    pub escalonar: bool,
}

impl Default for Sonda {
    /// O caminho do produto: escalonada.
    fn default() -> Self {
        Self { escalonar: true }
    }
}

/// ⭐ **O mesmo, com o que a [`Sonda`] desliga por ARGUMENTO** — a porta da calibração.
#[must_use]
#[allow(clippy::too_many_arguments)]
pub fn paint_com(
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
    sonda: Sonda,
) -> Option<ph2d_field_gpu::trace::Pintado> {
    let mundos: Vec<[f32; 3]> = points.iter().map(|l| l.world).collect();
    // ⛔ **A placa tem de ter armazéns para o passe que pinta** — ver
    // [`ph2d_field_gpu::paint::ARMAZENS`]. Sem eles o quadro cai na CPU, em vez de a `wgpu` recusar
    // o layout a meio.
    if tracer
        .lock()
        .is_ok_and(|t| t.storage_slots() < ph2d_field_gpu::paint::ARMAZENS)
    {
        return None;
    }
    let cabem = lamps_that_fit(tracer, w, h);
    let (campo, fita, setup) = pedido(doc, reg, cam, &mundos, cabem, sonda, w, h, antialias)?;
    // ⚠️ **As duas listas nascem do MESMO `points`**, e é por isso que a ordem não pode divergir:
    // a posição da lâmpada `l` viaja no `MarchSetup` e a radiância dela aqui.
    let mut lamp_radiance = [[0.0f32; 3]; ph2d_field_gpu::trace::MAX_LAMPS];
    for (dst, l) in lamp_radiance.iter_mut().zip(points) {
        *dst = l.radiance_at_one;
    }
    let materiais = packed(surfaces.all);
    let tabelas = crate::studio_wgsl::tables();
    let pintor = ph2d_field_gpu::paint::PaintSetup {
        owners: surfaces.owners,
        materials: &materiais,
        env_source: crate::studio_wgsl::SOURCE,
        env_consts: &crate::studio_wgsl::constants(),
        env_tables: &tabelas,
        lamp_radiance,
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
            .painted_frame(&fita, campo.sculpts(), setup, &pintor, w, h),
    )
}

/// ⭐⭐⭐ **Quantas lâmpadas este quadro comporta no dispositivo** — ver
/// [`ph2d_field_gpu::trace::lamps_that_fit`].
///
/// ⚠️ **Ela pergunta à PLACA**, e é por isso que precisa do traçador: o limite de ligação é uma
/// propriedade do adaptador, não uma constante deste ficheiro.
#[must_use]
pub fn lamps_that_fit(tracer: &SharedTracer, w: u32, h: u32) -> usize {
    tracer.lock().map_or(0, |t| {
        ph2d_field_gpu::trace::lamps_that_fit(t.binding_limit(), w, h)
    })
}

/// ⭐ **O maior armazém que esta placa liga** — o número que o tecto das lâmpadas divide.
#[must_use]
pub fn binding_limit(tracer: &SharedTracer) -> u64 {
    tracer.lock().map_or(0, |t| t.binding_limit())
}

/// ⭐ **A lâmpada do rig, onde a wave da §25 a põe** — partilhada pelos gates deste módulo.
///
/// ⚠️ `#[cfg(test)]` porque ela é a fixtura de três gates e não uma porta do produto: quem acende a
/// cena é a [`crate::lights`], que lê o documento.
#[cfg(test)]
#[must_use]
pub fn tests_lampada(cam: &ph2d_field_render::Orbit) -> ph2d_field_render::PointLamp {
    let (right, up, toward_eye) = cam.basis();
    let ecra = [-0.5566703_f32, 0.6634139, 0.5];
    let r = 2.0 * cam.half_extent;
    ph2d_field_render::PointLamp {
        world: [0, 1, 2].map(|i| {
            cam.target[i] + r * (ecra[0] * right[i] + ecra[1] * up[i] + ecra[2] * toward_eye[i])
        }),
        radiance_at_one: [3.0, 3.0, 3.0],
    }
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
// A peça, o registo, a vista, as lâmpadas, o tecto delas, a tela e a bandeira — sete coisas
// independentes, e uma struct só as renomearia.
#[allow(clippy::too_many_arguments)]
fn pedido(
    doc: &ph2d_field::FieldDoc,
    reg: &ph2d_field_eval::hybrid::Registry,
    cam: &ph2d_field_render::Orbit,
    mundos: &[[f32; 3]],
    cabem: usize,
    sonda: Sonda,
    w: u32,
    h: u32,
    antialias: bool,
) -> Option<(
    ph2d_field_eval::device::DeviceField,
    ph2d_field_eval::wgsl::TapeWgsl,
    ph2d_field_gpu::trace::MarchSetup,
)> {
    // ⛔⛔ **ACIMA DO TECTO O DISPOSITIVO RECUSA, e não sombreia só as primeiras.** Deixar as
    // restantes sem sombra seria o defeito que esta cerca existe para impedir — e a CPU sabe
    // sombrear qualquer número.
    //
    // ⚠️ **O tecto é do QUADRO e não uma constante** — ver [`ph2d_field_gpu::trace::lamps_that_fit`]:
    // ele sai do tamanho de ligação que a placa oferece dividido pelos pixels desta tela.
    if mundos.is_empty() || mundos.len() > cabem {
        return None;
    }
    let mut lamps = [[0.0f32; 3]; ph2d_field_gpu::trace::MAX_LAMPS];
    for (dst, src) in lamps.iter_mut().zip(mundos) {
        *dst = *src;
    }
    // ⭐⭐⭐ **A PEÇA COM A ESCULTURA DENTRO** — ver [`ph2d_field_eval::device`]. `None` quando
    // alguma escultura não souber entregar a grade, e aí o chamador fica na CPU.
    let campo = ph2d_field_eval::device::DeviceField::new_com(doc, reg, sonda.escalonar)?;
    // ⛔⛔⛔ **AQUI VIVIA A CERCA DA LARGURA DA FITA, e ela saiu em 2026-09-15.** Ela mandava para a
    // CPU toda peça acima de um tecto, e os três números que o tecto teve saíram de uma sonda que
    // pedia aos dois motores trabalhos DIFERENTES. Medida de novo, a placa ganha em toda a faixa
    // (`1,3×`–`7,8×`, com um empate no penhasco de `192` arestas) e nas 17 cenas do produto
    // (`2,6×`–`98×`). ⇒ *uma cerca que nunca pode disparar não é uma cerca.* A nota com as tabelas
    // está no lugar do `MAX_GUARDADOS`, na [`ph2d_field_gpu`], e o que protege a propriedade agora é
    // o gate `a_placa_ganha_em_toda_a_faixa_medivel`.
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
        lamps,
        #[allow(clippy::cast_possible_truncation)]
        n_lamps: mundos.len() as u32,
        ball_center: bola.center,
        ball_radius: bola.radius,
        ao_rays: ph2d_field_render::OCCLUSION_PASSES,
        ao_reach: ph2d_field_render::OCCLUSION_REACH * cam.half_extent,
        edge_cos: ph2d_field_render::EDGE_COS,
    };
    Some((campo, fita, setup))
}

#[cfg(test)]
#[path = "gpu_frame_tests.rs"]
mod tests;

/// ⭐⭐⭐ **O PASSE QUE PINTA, nos dois motores** — irmão por assunto do gate do G-buffer.
#[cfg(test)]
#[path = "paint_parity_tests.rs"]
mod paint_parity_tests;
