//! **As CINCO seções OPCIONAIS** — a §11 Animation, a §12 Sockets/Anchors, a TIMERS, a SIGNAL
//! ACTIONS e a AUDIO.
//!
//! ⚠️ **Irmão de [`super::paint_frame_shared`] por CAP de FICHEIRO** (600): a TIMERS levou-o a 658,
//! e o corte por responsabilidade estava à mão porque a família já estava reunida lá dentro. *Um
//! cap de ficheiro e um cap de função medem grandezas diferentes*, e extrair para o mesmo ficheiro
//! curaria um e estouraria o outro — a lição que o par de PRECISAO pagou em 2026-08-20.
//!
//! ⚠️ **Elas andam juntas por uma PROPRIEDADE, não por vizinhança:** são as que só existem quando
//! o objecto **anexou** o componente (ADR-0166), e é isso que decide que elas se pintam DEPOIS das
//! que todo objecto tem.
//!
//! ⚠️⚠️ **O ficheiro chamava-se `paint_stateful` e a propriedade era outra** — *«as que dependem de
//! qual LINHA está aberta»*. Ela descrevia quatro das cinco: a AUDIO (TOP-20 #4) não tem lista e
//! por isso não tem linha aberta, mas é tão opcional como as outras e pinta-se no mesmo sítio.
//! *Quando a propriedade que dá nome a um ficheiro deixa de descrever o que ele contém, o que se
//! corrige é o nome — senão o próximo a chegar acredita nele.*
//!
//! ⚠️ **O `paint_anchor_section` FICA no irmão**, e não é inconsistência: ele é chamado daqui, e
//! trazê-lo devolveria o ficheiro-pai a 458 e este a 274 — dois ficheiros a meio do caminho em vez
//! de um corte por responsabilidade. A §12 é a mais antiga das três e vive onde nasceu.

use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{HitIndex, NoteData, WidgetStore};
use ph2d_editor_core::widget::section_cards::close_section;
use ph2d_text::TextSystem;
use ph2d_vector::VectorScene;

use super::paint_frame::{begin_section, finish_section};
use super::paint_frame_shared::paint_anchor_section;

/// **§11 Animation** — moldura e tudo. Irmã da `paint_anchor_section`, e igual a ela na única
/// coisa que as distingue das outras: ela também precisa do **estado do painel** (qual animação
/// está aberta no editor).
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_anim_section(
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
    anim: Option<&ph2d_editor_core::screens::hero::InspectorAnimInfo>,
    selected: &mut usize,
) -> f32 {
    let Some(an) = anim else {
        return y;
    };
    *selected = (*selected).min(an.rows.len().saturating_sub(1));
    y = close_section(scene, theme, inner_x, inner_w, y);
    let y_before = y;
    begin_section(
        section_tops_y,
        hit_index,
        inner_x,
        inner_w,
        body_top_y,
        y_before,
        ids::INSP_LIVE_ANIM_SECTION,
        header_h,
    );
    let new_y = crate::sections::anim::paint_anim_section(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        inner_x,
        inner_w,
        y,
        an,
        *selected,
    );
    // ⚠️ **Sem slot de NOTA**, e é deliberado: os slots são uma lista posicional que as doze
    // seções partilham, e acrescentar um a meio renumeraria as notas que os artistas já colaram.
    // A §11 nasce sem nota; dar-lhe uma é um passo à parte, com a renumeração feita de uma vez.
    finish_section(
        scene,
        text_system,
        hit_index,
        store,
        inner_x,
        inner_w,
        ids::INSP_LIVE_ANIM_SECTION,
        y_before,
        new_y,
        &[],
    )
}

/// **A secção TIMERS** — moldura e tudo. Irmã da `paint_anim_section` na única coisa que as
/// distingue das outras: ela também precisa do **estado do painel** (qual timer está aberto).
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_timer_section(
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
    timer: Option<&ph2d_editor_core::screens::hero::InspectorTimerInfo>,
    selected: &mut usize,
) -> f32 {
    let Some(tm) = timer else {
        return y;
    };
    *selected = (*selected).min(tm.rows.len().saturating_sub(1));
    y = close_section(scene, theme, inner_x, inner_w, y);
    let y_before = y;
    begin_section(
        section_tops_y,
        hit_index,
        inner_x,
        inner_w,
        body_top_y,
        y_before,
        ids::INSP_LIVE_TIMER_SECTION,
        header_h,
    );
    let new_y = crate::sections::timers::paint_timer_section(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        inner_x,
        inner_w,
        y,
        tm,
        *selected,
    );
    // ⚠️ **Sem slot de NOTA**, e é a mesma decisão da §11: os slots são uma lista posicional que as
    // seções partilham, e acrescentar um a meio renumeraria as notas que os artistas já colaram.
    finish_section(
        scene,
        text_system,
        hit_index,
        store,
        inner_x,
        inner_w,
        ids::INSP_LIVE_TIMER_SECTION,
        y_before,
        new_y,
        &[],
    )
}

/// **A secção SIGNAL ACTIONS** — moldura e tudo. Irmã da `paint_timer_section`.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_action_section(
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
    action: Option<&ph2d_editor_core::screens::hero::InspectorActionInfo>,
    selected: &mut usize,
) -> f32 {
    let Some(ac) = action else {
        return y;
    };
    *selected = (*selected).min(ac.rows.len().saturating_sub(1));
    y = close_section(scene, theme, inner_x, inner_w, y);
    let y_before = y;
    begin_section(
        section_tops_y,
        hit_index,
        inner_x,
        inner_w,
        body_top_y,
        y_before,
        ids::INSP_LIVE_ACTION_SECTION,
        header_h,
    );
    let new_y = crate::sections::actions::paint_action_section(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        inner_x,
        inner_w,
        y,
        ac,
        *selected,
    );
    finish_section(
        scene,
        text_system,
        hit_index,
        store,
        inner_x,
        inner_w,
        ids::INSP_LIVE_ACTION_SECTION,
        y_before,
        new_y,
        &[],
    )
}

/// **As CINCO seções OPCIONAIS**, na ordem em que se pintam.
///
/// ⚠️ **Elas andam juntas por uma PROPRIEDADE, não por vizinhança:** só existem quando o objecto
/// anexou o componente. Ver o doc do módulo — quatro das cinco também precisam do estado do painel;
/// a AUDIO não.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_optional_sections(
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
    anim: Option<&ph2d_editor_core::screens::hero::InspectorAnimInfo>,
    anim_selected: &mut usize,
    anchor: Option<&ph2d_editor_core::screens::hero::InspectorAnchorInfo>,
    anchor_selected: &mut usize,
    timer: Option<&ph2d_editor_core::screens::hero::InspectorTimerInfo>,
    timer_selected: &mut usize,
    action: Option<&ph2d_editor_core::screens::hero::InspectorActionInfo>,
    action_selected: &mut usize,
    audio: Option<&ph2d_editor_core::screens::hero::InspectorAudioInfo>,
    camera: Option<&ph2d_editor_core::screens::hero::InspectorCameraInfo>,
    factory: Option<&ph2d_editor_core::screens::hero::InspectorFactoryInfo>,
    topdown: Option<&ph2d_editor_core::topdown_edits::InspectorTopDownInfo>,
    tags: Option<&ph2d_editor_core::screens::hero::InspectorTagsInfo>,
    notes: &[Vec<(usize, NoteData)>],
) -> f32 {
    y = paint_anim_section(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        section_tops_y,
        inner_x,
        inner_w,
        body_top_y,
        y,
        header_h,
        anim,
        anim_selected,
    );
    y = paint_anchor_section(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        section_tops_y,
        inner_x,
        inner_w,
        body_top_y,
        y,
        header_h,
        anchor,
        anchor_selected,
        notes,
    );
    y = paint_timer_section(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        section_tops_y,
        inner_x,
        inner_w,
        body_top_y,
        y,
        header_h,
        timer,
        timer_selected,
    );
    y = paint_action_section(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        section_tops_y,
        inner_x,
        inner_w,
        body_top_y,
        y,
        header_h,
        action,
        action_selected,
    );
    y = paint_audio_section(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        section_tops_y,
        inner_x,
        inner_w,
        body_top_y,
        y,
        header_h,
        audio,
    );
    y = paint_camera_section(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        section_tops_y,
        inner_x,
        inner_w,
        body_top_y,
        y,
        header_h,
        camera,
    );
    // ⚠️ **`y = ` na irmã de cima, e não uma chamada solta**: a CAMERA deixou de ser a última desta
    // cadeia, e uma chamada cujo `y` se deita fora empilha a secção seguinte por cima dela — o
    // defeito exacto que o gate `two_sections_never_stack` existe para apanhar.
    y = crate::paint_optional_factory::paint_factory_section(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        section_tops_y,
        inner_x,
        inner_w,
        body_top_y,
        y,
        header_h,
        factory,
    );
    y = crate::paint_optional_factory::paint_lifecycle_section(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        section_tops_y,
        inner_x,
        inner_w,
        body_top_y,
        y,
        header_h,
        factory,
    );
    // ⚠️ **`y = `, e a razão é o gate `two_sections_never_stack`**: uma chamada cujo `y` se deita
    // fora empilha a secção seguinte por cima dela.
    y = crate::paint_optional_factory::paint_topdown_section(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        section_tops_y,
        inner_x,
        inner_w,
        body_top_y,
        y,
        header_h,
        topdown,
    );
    paint_tags_section(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        section_tops_y,
        inner_x,
        inner_w,
        body_top_y,
        y,
        header_h,
        tags,
    )
}

/// **A secção TAGS** — moldura e tudo. ⚠️ Sem estado de painel, como a do áudio e a da câmera: não
/// há «a tag aberta», e o `open` da caixa de escolha vive no store como o de todas as outras.
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
