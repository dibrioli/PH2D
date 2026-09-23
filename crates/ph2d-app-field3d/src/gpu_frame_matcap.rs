//! ⭐⭐⭐⭐ **A PORTA DO MATCAP NO DISPOSITIVO** — o modo de **OMISSÃO** do modelador.
//!
//! ⚠️ **Ele saiu do [`super::gpu_frame`] por um TECTO DE LOC** (`702` contra `700`) — e a fronteira
//! que o tecto forçou é a certa: *as duas leis de pintura são dois assuntos*. O irmão guarda a do
//! MATERIAL (que precisa dos materiais, do céu, do olhar e das lâmpadas) e este a do matcap, que
//! precisa da normal de vista e de uma fotografia. ⛔ Corte por responsabilidade, nunca isenção.

use super::{SharedTracer, Sonda, lamps_that_fit, pedido};

/// ⭐⭐⭐⭐ **O MATCAP, pintado no dispositivo** — o modo de **OMISSÃO** do modelador.
///
/// # ⛔⛔⛔ O report que isto fecha, e a leitura errada que ele custou
///
/// *«ao arrastar fica grosseiro ainda»* (dono, 2026-09-23). O `#[default]` do
/// [`crate::shading::Shading`] é o **matcap**, e até aqui só o `Render` ia à placa ⇒ ao abrir uma
/// cena o arrasto era **todo** de CPU: `90,17 ms` e o prévio a escolher `D=3` — um **nono** dos
/// píxeis — contra `16,63 ms` e `D=1`.
///
/// ⚠️⚠️ **A primeira cura que eu desenhei era ABRIR A MARCHA ao matcap, e ela PIORAVA:** o
/// [`march`] devolve o G-buffer pelo barramento (`49,8 MB` a `1920×1080`, `119`–`123 ms`), que é
/// **mais lento do que a CPU inteira**. *O ganho nunca foi a marcha estar na placa — é a IMAGEM não
/// atravessar o barramento.*
///
/// # ⭐⭐ Porque ele não passa pelo [`paint`]
///
/// Ver a nota do [`ph2d_field_gpu::matcap`]: o pintor de material liga **`9`** armazéns contra o
/// piso de **`8`** do WebGPU, e este liga **`8`** ⇒ *o modo de omissão corre em toda placa
/// conforme*. Mais: um matcap **não lê o campo da peça**, logo o texto do shader não muda quando o
/// artista acrescenta uma forma.
///
/// ⚠️ **Zero lâmpadas é o caso NORMAL aqui** — ver o `exige_luz` do [`pedido`].
#[must_use]
// A peça, o registo, a vista, a fotografia, o olhar, o fundo e a tela — sete coisas independentes.
#[allow(clippy::too_many_arguments)]
pub fn pinta_matcap(
    tracer: &SharedTracer,
    doc: &ph2d_field::FieldDoc,
    reg: &ph2d_field_eval::hybrid::Registry,
    cam: &ph2d_field_render::Orbit,
    mc: &ph2d_field_gpu::matcap::MatcapSetup<'_>,
    w: u32,
    h: u32,
) -> Option<ph2d_field_gpu::trace::Pintado> {
    // ⛔ **Sem fotografia não há matcap**, e a CPU trata esse caso pintando o fundo (ver o
    // `m.side == 0` do [`ph2d_field_render::shade_with`]). *Recusar aqui é mais honesto do que
    // pintar um quadro que a referência não pinta.*
    if mc.side == 0 {
        return None;
    }
    // ⚠️ **A placa tem de ligar os armazéns do passe** — a mesma cerca que o [`paint_com`] aplica,
    // com o número DESTE passe: sem ela a `wgpu` recusaria o layout a meio de um quadro.
    if tracer
        .lock()
        .is_ok_and(|t| t.storage_slots() < ph2d_field_gpu::matcap::ARMAZENS)
    {
        return None;
    }
    let cabem = lamps_that_fit(tracer, w, h);
    // ⚠️ **A SONDA é a de omissão** — o matcap não tem chão, ricochete nem borda mole, logo os
    // passageiros daquela bandeira não o alcançam; o que dela importa aqui é o `bordas`, e ele é a
    // segunda passagem da silhueta, que **todo** quadro re-amostra desde a `W7c`.
    let sonda = Sonda::default();
    let (campo, fita, setup) = pedido(doc, reg, cam, &[], None, cabem, sonda, w, h, None, false)?;
    let mut guarda = tracer.lock().ok()?;
    Some(guarda.matcap_frame(&fita, campo.sculpts(), setup, mc, w, h))
}
