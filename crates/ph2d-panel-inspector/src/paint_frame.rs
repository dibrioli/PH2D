//! The chrome every live Inspector section is wrapped in, lifted out of
//! `paint.rs`.
//!
//! `live_section!` is a macro **defined inside** `paint_inspector` (it closes
//! over a dozen per-frame locals), so every line of its body counts toward
//! that function's length — and `paint_inspector` sits at a frozen LOC
//! allowance that may only shrink. Adding §11 Physics Body pushed it over, so
//! the macro now does the two things only a macro can (capture the locals,
//! run the caller's block) and delegates the rest here.
//!
//! This is the split the LOC gate itself prescribes: helpers that take the
//! per-frame mutables plus `y` and hand `y` back.

use ph2d_a11y::NodeId;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{HitIndex, WidgetStore};
use ph2d_editor_core::paint::stroke_rounded_rect;
use ph2d_editor_core::widget::section_cards::close_section;
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{Radius, Spacing, StrokeToken};
use ph2d_vector::VectorScene;

use ph2d_editor_core::interaction::NoteData;
use ph2d_editor_core::paint::{paint_text, resolve};
use ph2d_editor_core::widget::INSPECTOR_SCROLLBAR_ID;
use ph2d_editor_core::widget::showcase::LAST_SECTION_TOPS_Y;
use ph2d_editor_core::widget::{self};
use ph2d_tokens::{ColorToken, TypeToken};

use crate::state::{set_last_inspector_content_h, set_last_inspector_visible_h};
use ph2d_editor_core::widget::panel_chrome::HIGHLIGHTER_RGBA;
use ph2d_editor_core::widget::showcase::{paint_one_note, push_section_top_y};

/// Record where a section starts and make its header clickable.
#[allow(clippy::too_many_arguments)]
pub(crate) fn begin_section(
    section_tops_y: &mut Vec<f32>,
    hit_index: &mut HitIndex,
    inner_x: f32,
    inner_w: f32,
    body_top_y: f32,
    y_before: f32,
    section_id: NodeId,
    header_h: f32,
) {
    push_section_top_y(section_tops_y, y_before - body_top_y);
    hit_index.register(section_id, Rect::new(inner_x, y_before, inner_w, header_h));
}

/// As notas que não estão ancoradas a seção nenhuma — pintadas no fim do corpo.
///
/// ⚠️ Saiu do `paint_inspector` por CAP: ela estava exactamente na catraca (387) e a §12 não
/// cabia. Este bloco é o candidato óbvio — *não olha para seção nenhuma*.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_trailing_notes(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    inner_x: f32,
    inner_w: f32,
    y: &mut f32,
    trailing: &[(usize, NoteData)],
) {
    for (slot, note) in trailing {
        paint_one_note(
            scene,
            text_system,
            hit_index,
            store,
            inner_x,
            inner_w,
            y,
            note,
            *slot,
        );
    }
}

/// §11 Physics Body + §12 Physics Joint + §13 Pulley Wheel, frame and all.
///
/// Lifted out of `paint_inspector` for the same reason the section frame and
/// phase B were lifted before it: that orchestrator is under a ratcheting LOC
/// cap, and §12 pushed it past the line §11 had already ratcheted down to. The
/// three live here together because they are one family — the joint section's
/// creation gesture is a button in the body section, and the wheel's is a button
/// in the joint section — and because keeping them adjacent is what makes their
/// three note slots obviously distinct.
///
/// Returns the new `y`.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_physics_sections(
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
    physics: Option<&ph2d_editor_core::screens::hero::InspectorPhysicsInfo>,
    joint: Option<&ph2d_editor_core::screens::hero::InspectorJointInfo>,
    wheel: Option<&ph2d_editor_core::screens::hero::InspectorWheelInfo>,
    // §14 Platform Player (W5) — a quarta da família, e a única cujo assunto é
    // COMPORTAMENTO em vez de corpo.
    player: Option<&ph2d_editor_core::screens::hero::InspectorPlayerInfo>,
    // Os slots de nota da PANELA inteira: a família indexa os seus (9, 10, 11)
    // aqui dentro, em vez de o orquestrador passar um por seção. É o mesmo
    // corte que trouxe a pintura para cá — quem é dono das três seções é dono
    // dos três slots.
    notes: &[Vec<(usize, NoteData)>],
) -> f32 {
    let slot = |i: usize| notes.get(i).map_or(&[][..], |v| &v[..]);
    // §11 Physics Body — offered for ANY Transform-bearing entity, with or
    // without a body: the empty state is the Add button, and without it a
    // sprite could never become physical (ADR-0131 D8).
    if let Some(phys) = physics {
        y = close_section(scene, theme, inner_x, inner_w, y);
        let y_before = y;
        begin_section(
            section_tops_y,
            hit_index,
            inner_x,
            inner_w,
            body_top_y,
            y_before,
            ids::INSP_LIVE_PHYSICS_SECTION,
            header_h,
        );
        let new_y = crate::sections::paint_physics_section(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            inner_x,
            inner_w,
            y,
            phys,
        );
        y = finish_section(
            scene,
            text_system,
            hit_index,
            store,
            inner_x,
            inner_w,
            ids::INSP_LIVE_PHYSICS_SECTION,
            y_before,
            new_y,
            slot(10),
        );
    }
    // §12 Physics Joint — only for an entity that IS a joint. Unlike §11 it has
    // no empty face: there is nothing to offer on an object that is not a
    // joint, and the gesture that creates one lives in §11, where the two
    // bodies you want to join are what you are looking at.
    if let Some(j) = joint {
        y = close_section(scene, theme, inner_x, inner_w, y);
        let y_before = y;
        begin_section(
            section_tops_y,
            hit_index,
            inner_x,
            inner_w,
            body_top_y,
            y_before,
            ids::INSP_LIVE_JOINT_SECTION,
            header_h,
        );
        let new_y = crate::sections::paint_joint_section(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            inner_x,
            inner_w,
            y,
            j,
        );
        y = finish_section(
            scene,
            text_system,
            hit_index,
            store,
            inner_x,
            inner_w,
            ids::INSP_LIVE_JOINT_SECTION,
            y_before,
            new_y,
            slot(11),
        );
    }
    // §13 Pulley Wheel — só para uma entidade que É uma roldana. Como a §12 e
    // pelo mesmo motivo, ela não tem face vazia: não há o que oferecer num
    // objeto que não é roldana, e o gesto que cria uma mora na §12, onde está a
    // corda que vai atravessá-la.
    if let Some(wh) = wheel {
        y = close_section(scene, theme, inner_x, inner_w, y);
        let y_before = y;
        begin_section(
            section_tops_y,
            hit_index,
            inner_x,
            inner_w,
            body_top_y,
            y_before,
            ids::INSP_LIVE_WHEEL_SECTION,
            header_h,
        );
        let new_y = crate::sections::paint_wheel_section(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            inner_x,
            inner_w,
            y,
            wh,
        );
        y = finish_section(
            scene,
            text_system,
            hit_index,
            store,
            inner_x,
            inner_w,
            ids::INSP_LIVE_WHEEL_SECTION,
            y_before,
            new_y,
            slot(12),
        );
    }
    // §14 Platform Player — para todo corpo Dynamic, COM ou SEM o componente:
    // a face vazia é o botão que faz o comportamento existir, e sem ela ele
    // seria alcançável só onde já existe (a lição da §11 do W2a).
    if let Some(pl) = player {
        y = close_section(scene, theme, inner_x, inner_w, y);
        let y_before = y;
        begin_section(
            section_tops_y,
            hit_index,
            inner_x,
            inner_w,
            body_top_y,
            y_before,
            ids::INSP_LIVE_PLAYER_SECTION,
            header_h,
        );
        let new_y = crate::sections::paint_player_section(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            inner_x,
            inner_w,
            y,
            pl,
        );
        y = finish_section(
            scene,
            text_system,
            hit_index,
            store,
            inner_x,
            inner_w,
            ids::INSP_LIVE_PLAYER_SECTION,
            y_before,
            new_y,
            slot(13),
        );
    }
    y
}

/// The chrome that wraps EVERY live section: the highlighter outline a user
/// can paint over a section, and the sticky notes anchored to it.
#[allow(clippy::too_many_arguments)]
pub(crate) fn finish_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    inner_x: f32,
    inner_w: f32,
    section_id: ph2d_a11y::NodeId,
    y_before: f32,
    new_y: f32,
    notes: &[(usize, NoteData)],
) -> f32 {
    let mut new_y = new_y;
    if let Some(color_idx) = store.section_outline_color(section_id) {
        let rgba = HIGHLIGHTER_RGBA[color_idx.min(4) as usize];
        let pad = Spacing::Xs.px();
        let block = Rect::new(
            inner_x - pad,
            y_before - pad,
            inner_w + pad * 2.0,
            (new_y - y_before + pad * 2.0).max(0.0),
        );
        let outline_color = ph2d_vector::Color::from_rgba8(rgba[0], rgba[1], rgba[2], rgba[3]); // LITERAL-COLOR-OK: HIGHLIGHTER_RGBA palette
        stroke_rounded_rect(
            scene,
            block,
            Radius::Md.px(),
            StrokeToken::Thick.px(),
            outline_color,
        );
    }
    for (slot, note) in notes {
        paint_one_note(
            scene,
            text_system,
            hit_index,
            store,
            inner_x,
            inner_w,
            &mut new_y,
            note,
            *slot,
        );
    }
    new_y
}

/// Is there anything at all to show, or is the Inspector empty?
///
/// A named question rather than a disjunction buried mid-function: it decides
/// whether the panel paints its "nothing selected" placeholder, and every new
/// section has to be remembered here — a fact that is easier to keep true
/// when it has a name and a signature that changes when you forget.
#[allow(clippy::too_many_arguments, clippy::fn_params_excessive_bools)]
pub(crate) fn any_live_section(flags: [bool; 11]) -> bool {
    flags.iter().any(|&b| b)
}

/// **As notas, distribuídas por seção.**
///
/// Mora aqui pelo mesmo argumento que trouxe a pintura da família de física: o
/// orquestrador de seções está num cap de LOC com CATRACA cujo texto diz, sobre
/// si mesmo, *"está na linha: a próxima seção divide de novo"* — e a §14 foi
/// ela. Um bloco autocontido que não olha para seção nenhuma é o corte honesto.
///
/// ⚠️ **O tamanho do array é uma slot por seção viva.** Dimensionado errado, uma
/// nota ancorada na ÚLTIMA seção cai em silêncio no `trailing` em vez de onde o
/// autor a pôs — e o silêncio é o problema, não o deslocamento.
pub(crate) type SectionNotes = ([Vec<(usize, NoteData)>; 15], Vec<(usize, NoteData)>);

pub(crate) fn split_notes(store: &WidgetStore) -> SectionNotes {
    // ⚠️ **QUINZE desde 2026-08-21** — a §12 Sockets/Anchors entrou no slot 14, DEPOIS da
    // família da física, que é onde ela é pintada. Pô-la no slot da spec (12) obrigaria a
    // empurrar a família outra vez; pô-la no fim mantém **índice == ordem visual**, que é o que
    // `before_section` significa.
    // ⚠️ **CATORZE antes disso** — a §5 9-Slice entrou no slot 6 e empurrou todas as que
    // vêm depois. O array estava CHEIO (0..12): renumerar sem o crescer punha a §10 Blend no
    // slot 9, que é o do Physics Body, e as duas passavam a partilhar as mesmas notas em
    // silêncio. *Um índice que soma entre seções conta-se; não se escolhe um livre que não há.*
    let mut per_section: [Vec<(usize, NoteData)>; 15] = Default::default();
    let mut trailing: Vec<(usize, NoteData)> = Vec::new();
    for (idx, note) in store
        .notes_for_panel(ids::INSP_PANEL)
        .to_vec()
        .into_iter()
        .enumerate()
    {
        match note.before_section {
            Some(i) if (i as usize) < per_section.len() => {
                per_section[i as usize].push((idx, note));
            }
            _ => trailing.push((idx, note)),
        }
    }
    (per_section, trailing)
}

/// Os três snapshots da FAMÍLIA de física, buscados de uma vez.
///
/// Mora aqui pelo mesmo argumento que trouxe a pintura deles: §11, §12 e §13 são
/// uma família, e o orquestrador de seções está num cap de LOC com catraca (o
/// `architecture_panel_loc_cap` diz, textualmente, *"a próxima seção divide de
/// novo"*). Buscar os três numa linha é o que paga a §13 sem pedir uma exceção.
pub(crate) type PhysicsFamilyInfos = (
    Option<ph2d_editor_core::screens::hero::InspectorPhysicsInfo>,
    Option<ph2d_editor_core::screens::hero::InspectorJointInfo>,
    Option<ph2d_editor_core::screens::hero::InspectorWheelInfo>,
    Option<ph2d_editor_core::screens::hero::InspectorPlayerInfo>,
);

pub(crate) fn physics_family_infos() -> PhysicsFamilyInfos {
    (
        crate::state::current_inspector_physics(),
        crate::state::current_inspector_joint(),
        crate::state::current_inspector_wheel(),
        crate::state::current_inspector_player(),
    )
}

/// The per-frame numbers `publish_and_finish` needs, bundled — sixteen loose
/// parameters is a signature nobody can call correctly twice.
pub(crate) struct PanelFinish {
    pub any_section: bool,
    pub has_selection: bool,
    pub inner_x: f32,
    pub inner_w: f32,
    pub content_top: f32,
    pub content_bottom: f32,
    pub body_top_y: f32,
    pub y: f32,
    pub scroll_y: f32,
    pub rect: Rect,
}

/// Phase B of the Inspector paint: the empty-state placeholder, the scroll
/// bookkeeping the host reads next frame, and the scrollbar.
///
/// Lifted out of `paint_inspector` for the same reason as the section frame —
/// its LOC allowance is frozen and every new section costs ~18 lines. The
/// vector panel splits its own paint the same way (`seed_and_publish`).
pub(crate) fn publish_and_finish(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: ph2d_tokens::Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    f: PanelFinish,
    section_tops_y: Vec<f32>,
) {
    if !f.any_section {
        let placeholder = if f.has_selection {
            "No properties yet for the selected entity."
        } else {
            "Select an entity in the Hierarchy to inspect its properties."
        };
        let line_h = TypeToken::Sm.px() + Spacing::Xs.px();
        let center_y = f.content_top + (f.content_bottom - f.content_top) * 0.5 - line_h * 0.5;
        paint_text(
            text_system,
            scene,
            placeholder,
            f.inner_x + Spacing::Md.px(),
            center_y,
            TypeToken::Sm.px(),
            (f.inner_w - Spacing::Xl.px()).max(80.0), // LITERAL-PX-OK: minimum placeholder text width
            resolve(ColorToken::Text3, theme),
        );
    }

    let content_h = (f.y - f.body_top_y).max(0.0);
    let visible_h = (f.content_bottom - f.content_top).max(0.0);
    set_last_inspector_content_h(content_h);
    set_last_inspector_visible_h(visible_h);
    LAST_SECTION_TOPS_Y.with(|t| *t.borrow_mut() = section_tops_y);

    if widget::scrollbar_is_needed(content_h, visible_h) {
        let body = Rect::new(f.rect.x, f.content_top, f.rect.w, visible_h);
        let track = widget::scrollbar_track_rect(body);
        let thumb = widget::scrollbar_thumb_rect(track, f.scroll_y, content_h, visible_h);
        widget::paint_scrollbar(
            body,
            f.scroll_y,
            content_h,
            visible_h,
            store.scrollbar_visual(INSPECTOR_SCROLLBAR_ID),
            scene,
            theme,
        );
        hit_index.register(INSPECTOR_SCROLLBAR_ID, thumb);
    }
}

/// **Todos os snapshots vivos deste quadro, lidos de uma vez.**
///
/// ⚠️ **Extraído do `paint_inspector` em 2026-08-23**, quando a §11 o levou acima da tolerância.
/// Mas o corte não é só de LOC: ler os treze snapshots e decidir se **alguma** seção está viva é
/// uma pergunta só, e ela vivia espalhada por vinte e cinco linhas no meio do orquestrador.
///
/// ⚠️ **O `any_section` é DERIVADO aqui**, junto de quem o alimenta. Enquanto ele estava longe,
/// acrescentar um snapshot novo e esquecer a linha dele naquela lista não dava erro nenhum — dava
/// um painel que se declarava vazio com uma seção pintada dentro.
pub(crate) struct LiveSnapshots {
    pub transform_info: Option<ph2d_editor_core::screens::hero::InspectorTransformInfo>,
    pub sprite_info: Option<ph2d_editor_core::screens::hero::InspectorSpriteInfo>,
    pub visibility_info: Option<ph2d_editor_core::screens::hero::InspectorVisibilityInfo>,
    pub ordering_info: Option<ph2d_editor_core::screens::hero::InspectorOrderingInfo>,
    pub sampling_info: Option<ph2d_editor_core::screens::hero::InspectorSamplingInfo>,
    pub slice_info: Option<ph2d_editor_core::screens::hero::InspectorSliceInfo>,
    pub anchor_info: Option<ph2d_editor_core::screens::hero::InspectorAnchorInfo>,
    pub anim_info: Option<ph2d_editor_core::screens::hero::InspectorAnimInfo>,
    pub blend_info: Option<ph2d_editor_core::screens::hero::InspectorBlendInfo>,
    pub physics_info: Option<ph2d_editor_core::screens::hero::InspectorPhysicsInfo>,
    pub joint_info: Option<ph2d_editor_core::screens::hero::InspectorJointInfo>,
    pub wheel_info: Option<ph2d_editor_core::screens::hero::InspectorWheelInfo>,
    pub player_info: Option<ph2d_editor_core::screens::hero::InspectorPlayerInfo>,
    /// ⭐ **A seção COMPONENT** (ADR-0164 / F5) — `Some` só quando o selecionado é peça de
    /// uma cópia. ⚠️ **Não entra no `any_section`**, e pela razão das outras três: uma peça de
    /// instância tem sempre `Transform`, logo o `transform_info` já a representa — contá-la
    /// outra vez não mudaria a resposta.
    pub instance_info: Option<ph2d_editor_core::screens::hero::InspectorInstanceInfo>,
    /// ⭐ **O CARTÃO DE PROPRIEDADES** — `Some` quando o nome do objecto (ou o do mestre dele)
    /// declara alguma. ⚠️ **Não entra no `any_section`**, pela razão dos irmãos: quem tem
    /// propriedades tem `Transform`, logo já está representado.
    pub properties_info: Option<ph2d_editor_core::screens::hero::InspectorPropertiesInfo>,
    pub name_present: bool,
    /// Alguma seção viva? ⚠️ A §5 9-Slice, a §11 Animation e a §12 Sockets **não** entram nesta
    /// conta, e é deliberado: as três só existem sobre uma sprite, que já está representada pelo
    /// `sprite_info`. Contá-las outra vez não mudaria a resposta.
    pub any_section: bool,
}

impl LiveSnapshots {
    /// Lê os treze do estado do painel e decide o `any_section`.
    pub(crate) fn fetch() -> Self {
        let transform_info = crate::state::current_inspector_transform();
        let sprite_info = crate::state::current_inspector_sprite();
        let visibility_info = crate::state::current_inspector_visibility();
        let ordering_info = crate::state::current_inspector_ordering();
        let sampling_info = crate::state::current_inspector_sampling();
        let blend_info = crate::state::current_inspector_blend();
        let (physics_info, joint_info, wheel_info, player_info) = physics_family_infos();
        let instance_info = crate::state::current_inspector_instance();
        let properties_info = crate::state::current_inspector_properties();
        let name_present = crate::state::current_inspector_name_is_some();
        let any_section = any_live_section([
            transform_info.is_some(),
            sprite_info.is_some(),
            visibility_info.is_some(),
            ordering_info.is_some(),
            sampling_info.is_some(),
            blend_info.is_some(),
            physics_info.is_some(),
            joint_info.is_some(),
            wheel_info.is_some(),
            player_info.is_some(),
            name_present,
        ]);
        Self {
            transform_info,
            sprite_info,
            visibility_info,
            ordering_info,
            sampling_info,
            slice_info: crate::state::current_inspector_slice(),
            anchor_info: crate::state::current_inspector_anchor(),
            anim_info: crate::state::current_inspector_anim(),
            blend_info,
            physics_info,
            joint_info,
            wheel_info,
            player_info,
            instance_info,
            properties_info,
            name_present,
            any_section,
        }
    }
}
