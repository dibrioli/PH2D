//! ⭐⭐⭐ **A COSTURA: o quadro do MATCAP passa pelo dispositivo** (`docs/Render3d/05` §36).
//!
//! # As duas condições
//!
//! 1. **há adaptador** — sem GPU o módulo traça na CPU, como sempre;
//! 2. ⭐ **a peça é DESENHÁVEL lá** ([`ph2d_field_gpu::supports`]) — uma ESCULTURA atravessa como
//!    grade ([`ph2d_field_gpu::sculpt`]), e o que a porta pergunta é se a folha amostrada sabe
//!    entregá-la.
//!
//! ⚠️ O pintor de material do Render traçado, as luzes, o chão e a borda mole viviam aqui e saíram
//! em 03/10: o modo Render desenha por malha ([`crate::malha_render_quadro`]).
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

/// ⭐⭐⭐⭐ **O TRAÇADOR PARA A THREAD QUE DESENHA** — e é uma porta DIFERENTE do [`shared`].
///
/// # ⛔⛔⛔ O que isto separa, e porque tem de ser separado (medido 2026-09-23)
///
/// A thread que responde a um pedido de quadro nasce **DESANEXADA** (`std::thread::spawn`, sem
/// `JoinHandle`) — de propósito: é isso que mantém a janela a 60 Hz enquanto a peça traça. ⇒
/// *trabalho na placa pode sobreviver ao processo.*
///
/// Reproduzido `3` de `3` com o modo de omissão aberto ao dispositivo, num teste de unidade COMUM
/// que desenha um quadro:
///
/// ```text
/// test result: ok               ← o teste PASSA
/// NVVM compilation failed: 3    ← o driver, DEPOIS
/// (signal: 11, SIGSEGV)         ← o PROCESSO não consegue sair
/// ```
///
/// ⚠️⚠️ **O defeito NÃO é do matcap e não é desta wave:** o discriminador é *a placa vista de uma
/// thread que ninguém espera*. Os **77** gates de placa desta crate nunca estouraram porque chamam
/// as portas do dispositivo **em série, na thread do teste** — elas bloqueiam no
/// `device.poll(wait)` e devolvem antes de o teste acabar. *O que mata é o DESANEXADO, nunca a
/// placa.* As duas sondas que o atribuem são o [`testes::diag_o_quadro_em_voo_mata_o_processo`] e o
/// irmão que o cura por espera.
///
/// # ⇒ A lei
///
/// **A placa é do PRODUTO; um teste de unidade comum não a entrega à thread que desenha.** É a
/// mesma lei que o `CLAUDE.md` já escreve para os gates de GPU (*são `#[ignore]` e precisam de
/// adaptador*), aqui aplicada ao único consumidor que a pedia **sem pedir**.
///
/// ⏳ **E o defeito de produto fica NOMEADO, com a reprodução na mão:** ao fechar a janela com um
/// quadro em voo o app pode morrer da mesma maneira, e isso já era verdade no `Render` antes desta
/// wave. A cura é o processo **drenar** os quadros em voo antes de sair — uma thread de desenho que
/// o processo POSSUI em vez de N desanexadas —, e ela é wave própria.
#[must_use]
pub fn para_o_quadro() -> Option<&'static SharedTracer> {
    #[cfg(test)]
    if !testes::a_placa_vai_ao_quadro() {
        return None;
    }
    shared()
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

/// ⭐⭐⭐ **Este quadro vai para o dispositivo?** — as duas condições da nota do módulo.
#[must_use]
pub fn takes_the_frame(
    tracer: Option<&SharedTracer>,
    doc: &ph2d_field::FieldDoc,
    reg: &ph2d_field_eval::hybrid::Registry,
) -> bool {
    tracer.is_some() && ph2d_field_gpu::supports(doc, reg)
}

/// ⭐ **O G-buffer marchado no dispositivo, trazido à CPU** — a porta das paridades com o traçado
/// de CPU (o produto pinta o Matcap na placa, sem o trazer de volta). `None` quando alguma coisa
/// faltar.
#[must_use]
// A peça, o registo, a vista, a tela e a bandeira — nenhuma pertence a outra.
#[allow(clippy::too_many_arguments)]
pub fn march(
    tracer: &SharedTracer,
    doc: &ph2d_field::FieldDoc,
    reg: &ph2d_field_eval::hybrid::Registry,
    cam: &ph2d_field_render::Orbit,
    w: u32,
    h: u32,
    bordas: bool,
) -> Option<ph2d_field_render::Gbuffer> {
    let sonda = Sonda {
        bordas,
        ..Sonda::default()
    };
    let (campo, fita, setup) = pedido(doc, reg, cam, sonda, w, h)?;
    let screen = ph2d_field_render::Screen::new(w, h, cam.half_extent);
    let dev = tracer
        .lock()
        .ok()?
        .frame(&fita, campo.sculpts(), setup, w, h);
    Some(dev.to_cpu(cam, screen))
}

#[path = "gpu_frame_matcap.rs"]
mod gpu_frame_matcap;
#[cfg(test)]
pub(crate) use gpu_frame_matcap::matcap_liso;
pub use gpu_frame_matcap::{pinta_matcap, pinta_matcap_com};

#[path = "gpu_frame_sonda.rs"]
mod gpu_frame_sonda;
pub use gpu_frame_sonda::Sonda;

/// O que a marcha precisa de saber, derivado uma vez — a fita e o pedido.
///
/// ⚠️ **Ele é partilhado pelo [`march`] e pelo [`pinta_matcap`] de propósito:** os dois têm de
/// marchar exactamente a mesma coisa, senão o gate que compara as duas imagens mede também a
/// geometria.
pub(super) fn pedido(
    doc: &ph2d_field::FieldDoc,
    reg: &ph2d_field_eval::hybrid::Registry,
    cam: &ph2d_field_render::Orbit,
    sonda: Sonda,
    w: u32,
    h: u32,
) -> Option<(
    ph2d_field_eval::device::DeviceField,
    ph2d_field_eval::wgsl::TapeWgsl,
    ph2d_field_gpu::trace::MarchSetup,
)> {
    let campo = ph2d_field_eval::device::DeviceField::new_com(doc, reg, sonda.escalonar)?;
    // ⛔⛔⛔ **AQUI VIVIA A CERCA DA LARGURA DA FITA, e ela saiu em 2026-09-15** porque a grandeza
    // dela não ordena os resultados: o contorno de `768` arestas perde `0,52×` de forma
    // reprodutível e os dois vizinhos — `512` e `1024` — ganham. *Um corte que apanhasse o mau
    // excluiria um bom.* A nota com as tabelas está no lugar do `MAX_GUARDADOS`, na
    // [`ph2d_field_gpu`]; o que protege a faixa do produto é o gate
    // `na_faixa_do_produto_a_placa_ganha_com_margem`.
    let fita = if sonda.fita_interpretada {
        campo.tape_interpretada()?
    } else {
        campo.tape_wgsl()?
    };
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
        // ⭐⭐⭐ **A SEGUNDA PASSAGEM DA SILHUETA CORRE EM TODO QUADRO** (`W7c`, 2026-09-19) — ela
        // saiu da bandeira do quadro assente e passou a ser propriedade do caminho, com o preço
        // medido no doc da [`Sonda::bordas`]. *Quem a quiser desligar passa pela sonda.*
        antialias: sonda.bordas,
        step: passo,
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        budget: ((ph2d_field_render::MAX_STEPS as f32) * shrink.max(1.0)
            / passo.clamp(f32::EPSILON, 1.0))
        .ceil() as u32,
        t_max: ph2d_field_render::T_MAX,
        edge_cos: ph2d_field_render::EDGE_COS,
        longe: a_caixa_da_marcha(bola, sonda.longe),
    };
    Some((campo, fita, setup))
}

/// ⭐⭐⭐⭐ **A CAIXA do pedido vive no irmão** — o recorte e a grade. ⛔ Corte por tecto de LOC e
/// por responsabilidade: *qual caixa* é um assunto, *o que o quadro pede* é outro.
#[path = "gpu_frame_caixa.rs"]
mod gpu_frame_caixa;
pub use gpu_frame_caixa::a_caixa_da_marcha;

#[cfg(test)]
#[path = "gpu_frame_tests.rs"]
mod tests;

/// ⭐⭐⭐⭐ **A ATRIBUIÇÃO do `SIGSEGV` à saída do processo** — ver o doc do [`para_o_quadro`].
#[cfg(test)]
#[path = "gpu_frame_desanexado_tests.rs"]
pub(crate) mod testes;

/// ⭐⭐⭐⭐ **O MATCAP nos dois motores** — o modo de OMISSÃO do modelador, byte a byte contra a lei
/// da CPU. ⛔ Irmão por assunto do [`paint_parity_tests`], e a razão de ele ser um ficheiro próprio
/// é a mesma: *a lei que ele mede é outra*.
#[cfg(test)]
#[path = "matcap_parity_tests.rs"]
mod matcap_parity_tests;

/// ⭐⭐⭐ **E a LEI do estilo nos dois motores, com as entradas ENTREGUES** — a outra metade da
/// atribuição: se a lei bate ao bit com a curvatura dada, toda divergência de pixel é da curvatura.
#[cfg(test)]
#[path = "estilo_lei_parity_tests.rs"]
pub(crate) mod estilo_lei_parity_tests;
