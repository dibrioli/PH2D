//! **As MOLDURAS das secções FACTORY e LIFECYCLE** (TOP-20 #11 e #12, W3) — irmão do
//! [`super::paint_optional`] por CAP de ficheiro de painel.
//!
//! ⚠️ **O corte é por RESPONSABILIDADE e não por tamanho:** estas duas são as únicas do orquestrador
//! que leem o MESMO snapshot e o filtram por metades diferentes (`factory.is_some()` /
//! `lifecycle.is_some()`), e ficam melhor uma ao lado da outra.

use super::paint_frame::{begin_section, finish_section};
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{HitIndex, WidgetStore};
use ph2d_editor_core::widget::section_cards::close_section;
use ph2d_text::TextSystem;
use ph2d_vector::VectorScene;

/// **A secção FACTORY** — moldura e tudo (TOP-20 #11, W3). ⚠️ Sem estado de painel, como a da
/// câmera: um objecto tem UMA fábrica, então não há linha aberta a lembrar.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_factory_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: ph2d_tokens::Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    section_tops_y: &mut Vec<f32>,
    inner_x: f32,
    inner_w: f32,
    body_top_y: f32,
    mut y: f32,
    header_h: f32,
    info: Option<&ph2d_editor_core::screens::hero::InspectorFactoryInfo>,
) -> f32 {
    // ⚠️ **A secção só existe se o objecto TIVER a fábrica** — ADR-0166. Um objecto com só uma
    // vida entra pelo irmão de baixo, e nenhum dos dois paga o cabeçalho do outro.
    let Some(info) = info.filter(|i| i.factory.is_some()) else {
        return y;
    };
    y = close_section(scene, theme, inner_x, inner_w, y);
    let y_before = y;
    begin_section(
        section_tops_y,
        hit_index,
        inner_x,
        inner_w,
        body_top_y,
        y_before,
        ids::INSP_LIVE_FACTORY_SECTION,
        header_h,
    );
    let new_y = crate::sections::factory::paint_factory_section(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        inner_x,
        inner_w,
        y,
        info,
    );
    finish_section(
        scene,
        text_system,
        hit_index,
        store,
        inner_x,
        inner_w,
        ids::INSP_LIVE_FACTORY_SECTION,
        y_before,
        new_y,
        &[],
    )
}

/// **A secção LIFECYCLE** — moldura e tudo (TOP-20 #12, W3).
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_lifecycle_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: ph2d_tokens::Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    section_tops_y: &mut Vec<f32>,
    inner_x: f32,
    inner_w: f32,
    body_top_y: f32,
    mut y: f32,
    header_h: f32,
    info: Option<&ph2d_editor_core::screens::hero::InspectorFactoryInfo>,
) -> f32 {
    let Some(info) = info.filter(|i| i.lifecycle.is_some()) else {
        return y;
    };
    y = close_section(scene, theme, inner_x, inner_w, y);
    let y_before = y;
    begin_section(
        section_tops_y,
        hit_index,
        inner_x,
        inner_w,
        body_top_y,
        y_before,
        ids::INSP_LIVE_LIFECYCLE_SECTION,
        header_h,
    );
    let new_y = crate::sections::factory::paint_lifecycle_section(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        inner_x,
        inner_w,
        y,
        info,
    );
    finish_section(
        scene,
        text_system,
        hit_index,
        store,
        inner_x,
        inner_w,
        ids::INSP_LIVE_LIFECYCLE_SECTION,
        y_before,
        new_y,
        &[],
    )
}

/// **A secção TOP-DOWN PLAYER** — moldura e tudo (TOP-20 #13, W3).
///
/// ⚠️ Ela mora neste ficheiro e não no [`super::paint_optional`] pela mesma razão das duas acima:
/// o orquestrador está no tecto de LOC, e as molduras de secção opcional cabem melhor juntas.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_topdown_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: ph2d_tokens::Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    section_tops_y: &mut Vec<f32>,
    inner_x: f32,
    inner_w: f32,
    body_top_y: f32,
    mut y: f32,
    header_h: f32,
    info: Option<&ph2d_editor_core::topdown_edits::InspectorTopDownInfo>,
) -> f32 {
    // ⚠️ **A secção só existe se o objecto TIVER o mover** — ADR-0166.
    let Some(info) = info else {
        return y;
    };
    y = close_section(scene, theme, inner_x, inner_w, y);
    let y_before = y;
    begin_section(
        section_tops_y,
        hit_index,
        inner_x,
        inner_w,
        body_top_y,
        y_before,
        ids::INSP_LIVE_TOPDOWN_SECTION,
        header_h,
    );
    let new_y = crate::sections::topdown::paint_topdown_section(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        inner_x,
        inner_w,
        y,
        info,
    );
    finish_section(
        scene,
        text_system,
        hit_index,
        store,
        inner_x,
        inner_w,
        ids::INSP_LIVE_TOPDOWN_SECTION,
        y_before,
        new_y,
        &[],
    )
}

/// **A secção PROJECTILE MOTION** — moldura e tudo (TOP-20 #14, W3).
///
/// ⚠️ Ela mora neste ficheiro e não no [`super::paint_optional`] pela mesma razão das duas acima:
/// o orquestrador está no tecto de LOC, e as molduras de secção opcional cabem melhor juntas.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_projectile_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: ph2d_tokens::Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    section_tops_y: &mut Vec<f32>,
    inner_x: f32,
    inner_w: f32,
    body_top_y: f32,
    mut y: f32,
    header_h: f32,
    info: Option<&ph2d_editor_core::projectile_edits::InspectorProjectileInfo>,
) -> f32 {
    // ⚠️ **A secção só existe se o objecto TIVER o projéctil** — ADR-0166.
    let Some(info) = info else {
        return y;
    };
    y = close_section(scene, theme, inner_x, inner_w, y);
    let y_before = y;
    begin_section(
        section_tops_y,
        hit_index,
        inner_x,
        inner_w,
        body_top_y,
        y_before,
        ids::INSP_LIVE_PROJECTILE_SECTION,
        header_h,
    );
    let new_y = crate::sections::projectile::paint_projectile_section(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        inner_x,
        inner_w,
        y,
        info,
    );
    finish_section(
        scene,
        text_system,
        hit_index,
        store,
        inner_x,
        inner_w,
        ids::INSP_LIVE_PROJECTILE_SECTION,
        y_before,
        new_y,
        &[],
    )
}

/// **A secção STATE MACHINE** — moldura e tudo (TOP-20 #15, W3).
///
/// ⚠️ Ela mora aqui pela mesma razão das irmãs acima: o orquestrador está no tecto de LOC, e as
/// molduras de secção opcional cabem melhor juntas.
///
/// ⚠️ **As duas selecções vêm de fora** — elas são estado do PAINEL, e quem o tem é o
/// [`crate::InspectorState`]. Lê-las aqui de um `thread_local` seria a segunda porta.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_statemachine_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: ph2d_tokens::Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    section_tops_y: &mut Vec<f32>,
    inner_x: f32,
    inner_w: f32,
    body_top_y: f32,
    mut y: f32,
    header_h: f32,
    info: Option<&ph2d_editor_core::statemachine_edits::InspectorStateMachineInfo>,
    state_sel: usize,
    trans_sel: usize,
) -> f32 {
    // ⚠️ **A secção só existe se o objecto TIVER o cérebro** — ADR-0166.
    let Some(info) = info else {
        return y;
    };
    y = close_section(scene, theme, inner_x, inner_w, y);
    let y_before = y;
    begin_section(
        section_tops_y,
        hit_index,
        inner_x,
        inner_w,
        body_top_y,
        y_before,
        ids::INSP_LIVE_SM_SECTION,
        header_h,
    );
    let new_y = crate::sections::statemachine::paint_statemachine_section(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        inner_x,
        inner_w,
        y,
        info,
        state_sel,
        trans_sel,
    );
    finish_section(
        scene,
        text_system,
        hit_index,
        store,
        inner_x,
        inner_w,
        ids::INSP_LIVE_SM_SECTION,
        y_before,
        new_y,
        &[],
    )
}

/// **A secção TAGS** — moldura e tudo. ⚠️ Sem estado de painel, como a do áudio e a da câmera: não
/// há «a tag aberta», e o `open` da caixa de escolha vive no store como o de todas as outras.
///
/// ⚠️ **Mudou-se para cá em 2026-09-15**, pelo tecto de 600 LOC do irmão e **por
/// responsabilidade**: este ficheiro é onde as molduras de secção opcional vivem, e aquele é o
/// orquestrador. ⛔ A cura de um tecto é o CORTE, nunca uma entrada nova no `FILE_OVERAGE_OK`.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_tags_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: ph2d_tokens::Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    section_tops_y: &mut Vec<f32>,
    inner_x: f32,
    inner_w: f32,
    body_top_y: f32,
    mut y: f32,
    header_h: f32,
    tags: Option<&ph2d_editor_core::screens::hero::InspectorTagsInfo>,
) -> f32 {
    let Some(tg) = tags else {
        return y;
    };
    y = close_section(scene, theme, inner_x, inner_w, y);
    let y_before = y;
    begin_section(
        section_tops_y,
        hit_index,
        inner_x,
        inner_w,
        body_top_y,
        y_before,
        ids::INSP_LIVE_TAGS_SECTION,
        header_h,
    );
    let new_y = crate::sections::tags::paint_tags_section(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        inner_x,
        inner_w,
        y,
        tg,
    );
    // ⚠️ **Sem slot de NOTA**, e é a mesma decisão das quatro irmãs da família lógica: os slots são
    // uma lista posicional que as secções partilham, e acrescentar um a meio renumeraria as notas
    // que os artistas já colaram.
    finish_section(
        scene,
        text_system,
        hit_index,
        store,
        inner_x,
        inner_w,
        ids::INSP_LIVE_TAGS_SECTION,
        y_before,
        new_y,
        &[],
    )
}

/// **A secção CAMERA** — moldura e tudo. ⚠️ Sem estado de painel, como a do áudio: um objecto tem
/// UMA câmera, então não há linha aberta a lembrar.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_camera_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: ph2d_tokens::Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    section_tops_y: &mut Vec<f32>,
    inner_x: f32,
    inner_w: f32,
    body_top_y: f32,
    mut y: f32,
    header_h: f32,
    camera: Option<&ph2d_editor_core::screens::hero::InspectorCameraInfo>,
) -> f32 {
    let Some(cam) = camera else {
        return y;
    };
    y = close_section(scene, theme, inner_x, inner_w, y);
    let y_before = y;
    begin_section(
        section_tops_y,
        hit_index,
        inner_x,
        inner_w,
        body_top_y,
        y_before,
        ids::INSP_LIVE_CAMERA_SECTION,
        header_h,
    );
    let new_y = crate::sections::camera::paint_camera_section(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        inner_x,
        inner_w,
        y,
        cam,
    );
    finish_section(
        scene,
        text_system,
        hit_index,
        store,
        inner_x,
        inner_w,
        ids::INSP_LIVE_CAMERA_SECTION,
        y_before,
        new_y,
        &[],
    )
}

/// **A secção AUDIO** — moldura e tudo. ⚠️ **A única das cinco sem estado de painel** (ver o doc do
/// módulo): um objecto tem UMA fonte de som, então não há linha aberta a lembrar.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_audio_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: ph2d_tokens::Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    section_tops_y: &mut Vec<f32>,
    inner_x: f32,
    inner_w: f32,
    body_top_y: f32,
    mut y: f32,
    header_h: f32,
    audio: Option<&ph2d_editor_core::screens::hero::InspectorAudioInfo>,
) -> f32 {
    let Some(au) = audio else {
        return y;
    };
    y = close_section(scene, theme, inner_x, inner_w, y);
    let y_before = y;
    begin_section(
        section_tops_y,
        hit_index,
        inner_x,
        inner_w,
        body_top_y,
        y_before,
        ids::INSP_LIVE_AUDIO_SECTION,
        header_h,
    );
    let new_y = crate::sections::audio::paint_audio_section(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        inner_x,
        inner_w,
        y,
        au,
    );
    finish_section(
        scene,
        text_system,
        hit_index,
        store,
        inner_x,
        inner_w,
        ids::INSP_LIVE_AUDIO_SECTION,
        y_before,
        new_y,
        &[],
    )
}
