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
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_text::TextSystem;
use ph2d_tokens::Spacing;
use ph2d_vector::VectorScene;

use ph2d_editor_core::paint::{paint_text_block, resolve};
use ph2d_tokens::{ColorToken, TypeToken};

use crate::state::{set_last_inspector_content_h, set_last_inspector_visible_h};

/// Record where a section starts and make its header clickable.
#[allow(clippy::too_many_arguments)]
pub(crate) fn begin_section(
    hit_index: &mut HitIndex,
    inner_x: f32,
    inner_w: f32,
    y_before: f32,
    section_id: NodeId,
    header_h: f32,
) {
    let head = Rect::new(inner_x, y_before, inner_w, header_h);
    hit_index.register(section_id, head);
    // ⭐⭐ **A PEGA regista-se AQUI, e não em cada secção** (2026-09-29): as quarenta molduras
    //    passam todas por esta porta, logo nenhuma nasce com a pega pintada e morta sob o rato.
    //    ⚠️ DEPOIS do cabeçalho — o hit-index resolve o último registado primeiro, e a pega
    //    fica por cima do rect que dobra a secção.
    if let Some(grip) = ids::grip_of(section_id)
        && !ph2d_editor_core::interaction::SECCOES_FIXAS.contains(&section_id)
    {
        hit_index.register(
            grip,
            ph2d_editor_core::widget::section_grip::grip_hit_rect(head),
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
/// in the joint section.
///
/// Returns the new `y`.
#[allow(clippy::too_many_arguments)]
pub(crate) fn push_physics_sections<'a>(
    plano: &mut crate::plano::Plano<'a>,
    store: &'a WidgetStore,
    inner_x: f32,
    inner_w: f32,
    header_h: f32,
    physics: Option<&'a ph2d_editor_core::screens::hero::InspectorPhysicsInfo>,
    joint: Option<&'a ph2d_editor_core::screens::hero::InspectorJointInfo>,
    wheel: Option<&'a ph2d_editor_core::screens::hero::InspectorWheelInfo>,
    // §14 Platform Player (W5) — a quarta da família, e a única cujo assunto é
    // COMPORTAMENTO em vez de corpo.
    player: Option<&'a ph2d_editor_core::screens::hero::InspectorPlayerInfo>,
) {
    use crate::plano::emoldurada;
    // §11 Physics Body — offered for ANY Transform-bearing entity, with or without a body: the
    // empty state is the Add button (ADR-0131 D8).
    if let Some(phys) = physics {
        emoldurada(
            plano,
            ids::INSP_LIVE_PHYSICS_SECTION,
            store,
            inner_x,
            inner_w,
            header_h,
            move |c, t, y| {
                crate::sections::paint_physics_section(
                    c.scene, c.text, t, c.hit, store, inner_x, inner_w, y, phys,
                )
            },
        );
    }
    // §12 Physics Joint — only for an entity that IS a joint (no empty face: the gesture that
    // creates one lives in §11).
    if let Some(j) = joint {
        emoldurada(
            plano,
            ids::INSP_LIVE_JOINT_SECTION,
            store,
            inner_x,
            inner_w,
            header_h,
            move |c, t, y| {
                crate::sections::paint_joint_section(
                    c.scene, c.text, t, c.hit, store, inner_x, inner_w, y, j,
                )
            },
        );
    }
    // §13 Pulley Wheel — só para uma entidade que É uma roldana.
    if let Some(wh) = wheel {
        emoldurada(
            plano,
            ids::INSP_LIVE_WHEEL_SECTION,
            store,
            inner_x,
            inner_w,
            header_h,
            move |c, t, y| {
                crate::sections::paint_wheel_section(
                    c.scene, c.text, t, c.hit, store, inner_x, inner_w, y, wh,
                )
            },
        );
    }
    // §14 Platform Player — para todo corpo Dynamic, COM ou SEM o componente.
    if let Some(pl) = player {
        emoldurada(
            plano,
            ids::INSP_LIVE_PLAYER_SECTION,
            store,
            inner_x,
            inner_w,
            header_h,
            move |c, t, y| {
                crate::sections::paint_player_section(
                    c.scene, c.text, t, c.hit, store, inner_x, inner_w, y, pl,
                )
            },
        );
    }
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
) -> f32 {
    // ⭐⭐ A porta partilhada (2026-10-01): as notas DESTA secção — pela identidade dela, onde quer
    //    que o artista a tenha arrastado — e o contorno POR FORA das duas. Antes o contorno era
    //    pintado primeiro e ficava curto, com as notas de fora.
    ph2d_editor_core::widget::showcase::notes_chrome::fecha_seccao(
        scene,
        text_system,
        hit_index,
        store,
        ids::INSP_PANEL,
        section_id,
        inner_x,
        inner_w,
        y_before,
        new_y,
    )
}

/// Is there anything at all to show, or is the Inspector empty?
///
/// A named question rather than a disjunction buried mid-function: it decides
/// whether the panel paints its "nothing selected" placeholder, and every new
/// section has to be remembered here — a fact that is easier to keep true
/// when it has a name and a signature that changes when you forget.
#[allow(clippy::too_many_arguments, clippy::fn_params_excessive_bools)]
pub(crate) fn any_live_section(flags: [bool; 35]) -> bool {
    flags.iter().any(|&b| b)
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
}

/// Phase B of the Inspector paint: the empty-state placeholder and the scroll
/// bookkeeping the host reads next frame (the scrollbar is the door's, in `close_body`).
///
/// Lifted out of `paint_inspector` for the same reason as the section frame —
/// its LOC allowance is frozen and every new section costs ~18 lines. The
/// vector panel splits its own paint the same way (`seed_and_publish`).
pub(crate) fn publish_and_finish(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: ph2d_tokens::Theme,
    f: PanelFinish,
) {
    if !f.any_section {
        let placeholder = if f.has_selection {
            tr("panel.inspector.panel.no_properties_yet_for_the")
        } else {
            tr("panel.inspector.panel.select_an_entity_in_the")
        };
        // ⛔⛔ **A FRASE QUEBRA, não é CORTADA** (2026-09-18, achado pela varredura das elisões):
        //    ela é a primeira coisa que o artista lê ao abrir o app, e com o `paint_text` — que
        //    elide para UMA linha desde 06/09 — ela saía **`…to inspec…`** numa coluna de `252 px`.
        //    *Uma dica cortada a meio da palavra é pior que nenhuma dica: ela ensina que o app
        //    está partido.* ⇒ `paint_text_block`, que é o pintor que existe PARA quebrar.
        let x = f.inner_x + Spacing::Md.px();
        let largura = (f.inner_w - Spacing::Xl.px()).max(80.0); // LITERAL-PX-OK: minimum placeholder text width
        let fonte = TypeToken::Sm.px();
        // ⭐⭐ **A régua da altura é o PRÓPRIO PINTOR, numa cena que se deita fora.** O doc do
        //    `paint_text_block` proíbe por escrito uma função de medição à parte (*«parley duas
        //    vezes e as duas respostas podem divergir»* — o defeito da dica de duas linhas do
        //    painel de física), e é por isso que aqui se pergunta a ELE. A cena descartada custa
        //    uma passada de layout de uma frase, no único quadro em que o painel está vazio.
        let alta = paint_text_block(
            text_system,
            &mut VectorScene::new(),
            placeholder,
            x,
            0.0,
            fonte,
            largura,
            resolve(ColorToken::Text3, theme),
        );
        let center_y = f.content_top + (f.content_bottom - f.content_top - alta) * 0.5;
        paint_text_block(
            text_system,
            scene,
            placeholder,
            x,
            center_y,
            fonte,
            largura,
            resolve(ColorToken::Text3, theme),
        );
    }

    let content_h = (f.y - f.body_top_y).max(0.0);
    let visible_h = (f.content_bottom - f.content_top).max(0.0);
    set_last_inspector_content_h(content_h);
    set_last_inspector_visible_h(visible_h);

    // A barra pinta-se na porta (`close_body` → `scroll_area::close_parts`).
}

/// ⭐ **Os snapshots vivos do quadro** — irmão por `#[path]`, cortado em 2026-09-14 pelo tecto de
/// 600 LOC deste ficheiro (a secção TAGS levou-o a `606`).
///
/// ⚠️ **O corte segue a fronteira que o doc do bloco já escrevia:** *«ler os snapshots e decidir se
/// alguma secção está viva é uma pergunta só»*, e ela não é a mesma que *«a moldura em que cada
/// secção é embrulhada»*, que é o que sobra aqui. ⛔ Subir o número seria adiar com juros.
#[path = "paint_frame_snapshots.rs"]
mod snapshots;
pub(crate) use snapshots::LiveSnapshots;
