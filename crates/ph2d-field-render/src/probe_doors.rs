//! ⭐⭐ **AS PORTAS QUE SÓ AS SONDAS ABREM** — as variantes do traçado com um parâmetro ESCOLHIDO.
//!
//! ⚠️ **Elas existem por uma lei desta linha:** *nenhuma leitura de relógio desta workstation vale
//! nada acima de `load ~5`*, e entre duas corridas o mesmo passe já deu `11,36` e `5,50 ms`. ⇒ os
//! dois lados de um A/B têm de correr no **mesmo processo**, intercalados — e para isso a constante
//! sob teste tem de entrar pelo argumento em vez de vir do módulo.
//!
//! ⚠️ **O corte é por ASSUNTO e nasceu de um tecto:** o `lib.rs` passou os `700` LOC do HR-18, e o
//! que saiu foi o que **nenhum caminho do produto chama**. *Uma porta de sonda no meio das portas do
//! produto faz o ficheiro crescer até quem lê deixar de distinguir umas das outras.*

// ⚠️ Glob de propósito: estas portas são as MESMAS funções que estavam no `lib.rs`, e uma lista de
// importações escrita à mão seria uma segunda resposta a *«o que uma porta do traçado precisa?»*.
use super::*;

/// ⭐ **A marcha por ladrilho com o lado ESCOLHIDO** — a porta que a sonda do `TILE` dirige.
#[doc(hidden)]
#[must_use]
#[allow(clippy::too_many_arguments)]
pub fn trace_tiled_for_test(
    doc: &FieldDoc,
    reg: &ph2d_field_eval::hybrid::Registry,
    cam: &Orbit,
    width: u32,
    height: u32,
    tile: usize,
    slabs: usize,
    antialias: bool,
    parallel: bool,
) -> Option<Gbuffer> {
    let shape = ph2d_field_eval::hybrid::Hybrid::new(doc, reg);
    let rc = ph2d_field_eval::RegionCompiler::new(doc);
    let bbox = ph2d_field_eval::bounds::bounding_ball(doc, reg)
        .map(ph2d_field_eval::bounds_clip::march_clip)?;
    if shape.sampled_count() != 0 || !rc.is_worth_it() {
        return None;
    }
    let plane = Screen::new(width, height, cam.half_extent);
    let scene = Scene {
        shape: &shape,
        cam,
        basis: cam.basis(),
        sharp: Sharpness::for_frame(cam.half_extent, (width as usize).min(height as usize)),
        clip: Some(bbox),
        step: ph2d_field_eval::safe_march_step(doc),
        shrink: ph2d_field_eval::field_shrink(doc, reg),
        stencil: NORMAL_STENCIL,
    };
    Some(tiled_trace(
        doc, &rc, &scene, plane, bbox, parallel, antialias, None, tile, slabs, None,
    ))
}

/// ⭐⭐ **A marcha com o PASSO escolhido** — a porta que a sonda do passo dirige.
///
/// ⚠️ Ela existe para que as duas respostas sejam medidas no **mesmo processo**: entre duas corridas
/// desta workstation a montagem — que não depende do passo — mexeu-se `14,4 -> 22,1 ms`, e um A/B
/// nessas condições mede o relógio da máquina, não a mudança.
#[doc(hidden)]
#[must_use]
pub fn trace_stepped_for_test(
    doc: &FieldDoc,
    reg: &ph2d_field_eval::hybrid::Registry,
    cam: &Orbit,
    width: u32,
    height: u32,
    step: f32,
) -> Gbuffer {
    trace_inner_tiles(
        doc,
        reg,
        cam,
        width,
        height,
        true,
        true,
        None,
        true,
        step,
        NORMAL_STENCIL,
        None,
    )
}

/// ⭐⭐ **A marcha com a CACHE escolhida** — a porta que a sonda da cache dirige.
///
/// ⚠️ Pela mesma razão da [`trace_stepped_for_test`]: os dois lados do A/B têm de correr no **mesmo
/// processo**, porque o que se compara é um arrasto inteiro contra outro arrasto inteiro.
#[doc(hidden)]
#[must_use]
pub fn trace_cached_for_test(
    doc: &FieldDoc,
    reg: &ph2d_field_eval::hybrid::Registry,
    cam: &Orbit,
    width: u32,
    height: u32,
    antialias: bool,
    cache: Option<&TapeCache>,
) -> Gbuffer {
    trace_inner(doc, reg, cam, width, height, true, antialias, None, cache)
}

/// ⭐⭐ **A marcha por ladrilho COM CACHE e com o lado escolhido** — a porta que a varredura do
/// `TILE` da W88 dirige.
///
/// ⚠️ Ela existe porque a varredura anterior do `TILE` (§82.10) é **anterior à cache**: ali um
/// ladrilho pequeno pagava uma compilação a mais, e hoje a fita dele também é reusada. *Uma
/// varredura sem a cache mede um mundo que já não existe.*
#[doc(hidden)]
#[must_use]
#[allow(clippy::too_many_arguments)]
pub fn trace_tiled_with_cache_for_test(
    doc: &FieldDoc,
    reg: &ph2d_field_eval::hybrid::Registry,
    cam: &Orbit,
    width: u32,
    height: u32,
    tile: usize,
    slabs: usize,
    cache: Option<&TapeCache>,
) -> Option<Gbuffer> {
    let shape = ph2d_field_eval::hybrid::Hybrid::new(doc, reg);
    let rc = ph2d_field_eval::RegionCompiler::new(doc);
    let bbox = ph2d_field_eval::bounds::bounding_ball(doc, reg)
        .map(ph2d_field_eval::bounds_clip::march_clip)?;
    if shape.sampled_count() != 0 || !rc.is_worth_it() {
        return None;
    }
    if let Some(c) = cache {
        let (pw, ph) = (width as usize, height as usize);
        c.begin(doc, pw.div_ceil(tile) * ph.div_ceil(tile) * (slabs + 2));
    }
    let plane = Screen::new(width, height, cam.half_extent);
    let scene = Scene {
        shape: &shape,
        cam,
        basis: cam.basis(),
        sharp: Sharpness::for_frame(cam.half_extent, (width as usize).min(height as usize)),
        clip: Some(bbox),
        step: ph2d_field_eval::safe_march_step(doc),
        shrink: ph2d_field_eval::field_shrink(doc, reg),
        stencil: NORMAL_STENCIL,
    };
    Some(tiled_trace(
        doc, &rc, &scene, plane, bbox, true, false, None, tile, slabs, cache,
    ))
}

/// ⭐ **Quantas fatias de profundidade o produto reparte** — ver [`tiles::SLABS`].
///
/// ⚠️ Ela existe porque um binário de teste não alcança um `pub(crate)` e por isso **escolhia um
/// número**: o `tape_budget` media com `2` desde a W70, e o produto ship `4` desde a W71. *Um gate
/// que escolhe a configuração mede a configuração que escolheu.*
#[doc(hidden)]
#[must_use]
pub const fn slabs_for_test() -> usize {
    SLABS
}

/// ⭐ **O lado do ladrilho que o produto usa** — ver [`tiles::TILE`], e pela mesma razão do
/// [`slabs_for_test`].
#[doc(hidden)]
#[must_use]
pub const fn tile_for_test() -> usize {
    TILE
}

/// ⭐⭐ **A marcha com o ESTÊNCIL escolhido** — a porta que a sonda da normal dirige.
///
/// ⚠️ Pela mesma razão da [`trace_stepped_for_test`]: as duas respostas têm de ser medidas no
/// **mesmo processo**, e a comparação que interessa é entre as duas IMAGENS — o ângulo entre as
/// normais, pixel a pixel, que não depende do relógio da máquina.
#[doc(hidden)]
#[must_use]
pub fn trace_stencil_for_test(
    doc: &FieldDoc,
    reg: &ph2d_field_eval::hybrid::Registry,
    cam: &Orbit,
    width: u32,
    height: u32,
    stencil: Stencil,
) -> Gbuffer {
    trace_inner_tiles(
        doc,
        reg,
        cam,
        width,
        height,
        true,
        false,
        None,
        true,
        ph2d_field_eval::safe_march_step(doc),
        stencil,
        None,
    )
}

/// ⭐ **A MARCHA DE LINHA, forçada** — a porta que o gate de paridade dirige.
///
/// ⚠️ Ela existe porque, com um perfil no documento, o caminho por ladrilho passa a ser o **único**
/// alcançável — e uma paridade que não consegue chamar as duas metades não é uma paridade.
#[doc(hidden)]
#[must_use]
pub fn trace_by_rows_for_test(
    doc: &FieldDoc,
    reg: &ph2d_field_eval::hybrid::Registry,
    cam: &Orbit,
    width: u32,
    height: u32,
) -> Gbuffer {
    trace_inner_tiles(
        doc,
        reg,
        cam,
        width,
        height,
        true,
        true,
        None,
        false,
        ph2d_field_eval::safe_march_step(doc),
        NORMAL_STENCIL,
        None,
    )
}
