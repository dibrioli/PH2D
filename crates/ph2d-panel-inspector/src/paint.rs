//! Inspector panel paint — ADR-0029 Phase C.1 port of
//! `ph2d_editor_core::screens::hero::inspector::{paint_thunk,
//! paint_inspector}`.
//!
//! `paint` is the typed entry point invoked by the `Panel` trait. It
//! gates on visibility (via [`PanelHostInternal::panel_visible`]),
//! runs the snapshot sync, paints the live Inspector body, and
//! publishes scroll bounds back to the store.

use crate::paint_frame::{PanelFinish, publish_and_finish};
use crate::state::{
    self, current_inspector_visibility_section, last_inspector_content_h, last_inspector_visible_h,
};
use crate::state_popovers;
use crate::sync::sync_inspector_from_snapshots;
use crate::{InspectorPanel, sections};
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{HitIndex, WidgetStore};
use ph2d_editor_core::panel::{PaintCtx, Panel};
use ph2d_editor_core::screens::HeroSelection;
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ROW_H_PX, Spacing, Theme};
use ph2d_vector::VectorScene;

pub(crate) const BODY_PAD: f32 = 10.0; // LITERAL-PX-OK: inspector body inset
const SECTION_HEAD_H: f32 = ROW_H_PX;

pub(crate) fn paint(inspector_state: &mut state::InspectorState, ctx: &mut PaintCtx) {
    if !ctx.host.panel_visible(InspectorPanel::ID) {
        ctx.host.store_mut().clear_panel_rect(ids::INSP_PANEL);
        return;
    }
    // ⭐⭐⭐ **O RECT PUBLICADO É O DO ENCAIXE** — ver o irmão em `ph2d-panel-hierarchy/src/paint.rs`,
    // que traz o mecanismo inteiro. Este painel e a Hierarquia eram os dois únicos cujo rect era
    // publicado de FORA (`hero::paint`, com `layout.inspector`); os outros 20 publicam `ctx.slot`.
    ctx.host
        .store_mut()
        .set_panel_rect(ids::INSP_PANEL, ctx.slot);
    sync_inspector_from_snapshots(inspector_state, ctx.host);
    let display_unit = ctx.host.project().display_unit;
    let ppm = ctx.host.project().pixels_per_meter;
    let display_angle = ctx.host.project().display_angle;
    state::set_current_display_unit(display_unit, ppm);
    state::set_current_display_angle(display_angle);
    let theme = ctx.host.theme();
    // Splitting the borrows: paint_inspector wants &mut hit_index,
    // &WidgetStore, &mut Scene, &mut TextSystem — all from disjoint
    // refs on the host. Reborrow store via `store()` then `store_mut()`
    // sequentially below for the post-paint scroll publish.
    {
        // Build a transient owning copy of selection to avoid holding an
        // immutable borrow of host while also borrowing host's
        // hit_index mutably; selection is small + Clone. The combined
        // `store_and_hit_index_mut` accessor avoids the dyn-trait
        // aliasing dance for the &WidgetStore + &mut HitIndex pair.
        let selection_clone: Option<HeroSelection> = ctx.host.selection().cloned();
        let (store, hit_index) = ctx.host.store_and_hit_index_mut();
        paint_inspector(
            ctx.slot,
            selection_clone.as_ref(),
            ctx.scene,
            ctx.text_system,
            theme,
            hit_index,
            store,
            &mut inspector_state.anchor_selected,
            &mut inspector_state.anim_selected,
            &mut inspector_state.timer_selected,
            &mut inspector_state.action_selected,
        );
    }
    state::set_current_display_unit(display_unit, ppm); // keep symmetric with legacy
    state::set_current_display_angle(display_angle);
    // Publish content_h + clamp scroll right after paint so
    // `dispatch_wheel` sees the new bounds on the very next event.
    let content_h = last_inspector_content_h();
    let visible_h = last_inspector_visible_h();
    let store = ctx.host.store_mut();
    // ⭐ **O popover diferido publica o RECT dele aqui, e é o que o faz FECHAR ao clique fora.**
    //
    // ⚠️ A lei já existia no `dispatch::pointer_down` (*«se o usuário clicar fora do dropdown ele
    // deve se fechar»*, Enio 2026-06-24) e lê `store.dropdown_popover()` — que **nenhum dos
    // seletores deste painel publicava**, porque o passe diferido não tem `&mut WidgetStore`. Os
    // outros nove painéis do app publicam-no onde pintam; aqui o passe deposita e este sítio, que
    // é o primeiro com o store mutável depois da pintura, publica.
    if let Some((id, panel, content_h, visible_h)) = state_popovers::take_painted_popover() {
        store.set_dropdown_popover(id, panel);
        // ⚠️ **As duas alturas viajam com o rect**: são elas que dizem à roda e à barra até onde
        // rolar. Um popover clampado mais curto que a lista, publicado sem elas, fica cortado e
        // IMÓVEL.
        store.set_panel_content_h(id, content_h);
        store.set_panel_visible_h(id, visible_h);
        let max_scroll = (content_h - visible_h).max(0.0);
        if store.panel_scroll(id) > max_scroll {
            store.set_panel_scroll(id, max_scroll);
        }
    }
    store.set_panel_content_h(ids::INSP_PANEL, content_h);
    store.set_panel_visible_h(ids::INSP_PANEL, visible_h);
    let max_scroll = (content_h - visible_h).max(0.0);
    let cur = store.panel_scroll(ids::INSP_PANEL);
    if cur > max_scroll {
        store.set_panel_scroll(ids::INSP_PANEL, max_scroll);
    }
}

#[allow(clippy::too_many_arguments)]
/// # Os TRÊS que são estado do PAINEL, e não da cena
///
/// ⚠️ A prosa deles mudou-se do meio da lista de parâmetros para aqui em 2026-08-31, quando o
/// terceiro empurrou a função sobre o teto de 200 LOC: comentários dentro do corpo contam, e o
/// idioma do Rust para documentar um parâmetro é o doc-comment da função. *A tolerância desceu
/// junto — ela só anda nessa direcção.*
///
/// - `anchor_selected` — §12: qual linha da lista de âncoras está aberta. Saturado dentro de
///   `paint_anchor_section` contra o tamanho da lista: apagar a última âncora não o pode deixar a
///   apontar para o vazio.
/// - `anim_selected` — §11: qual animação está aberta no editor. Mesmo contrato.
/// - `timer_selected` — TIMERS: qual timer está aberto no editor. Mesmo contrato.
/// - `action_selected` — SIGNAL ACTIONS: qual acção está aberta. Mesmo contrato.
/// - `editing_value` — qual eixo do cartão de propriedades está a ser **reescrito**; ver
///
/// # A moldura e o fecho
///
/// A abertura ([`crate::paint_body::open_body`]) traz superfície, alças, cabeçalho, clip e a
/// caixa interior; o fecho ([`crate::paint_body::close_body`]) traz os popovers diferidos, o
/// `pop_layer` daquele clip, os cantos, o re-registo dos hits e os cartões. ⚠️ **Nada disso é
/// orquestração de seção** — é a mesma razão pela qual o cabeçalho saiu para o `paint_head` em
/// 2026-08-23, e a razão pela qual esta prosa vive AQUI: *comentário dentro do corpo conta para
/// o teto de 200 LOC; doc-comment não.*
fn paint_inspector(
    slot: Rect,
    selection: Option<&HeroSelection>,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    anchor_selected: &mut usize,
    anim_selected: &mut usize,
    timer_selected: &mut usize,
    action_selected: &mut usize,
) {
    let crate::paint_body::BodyFrame {
        rect,
        content_top,
        content_bottom,
        scroll_y,
        inner_x,
        inner_w,
        body_top_y,
    } = crate::paint_body::open_body(slot, scene, text_system, theme, hit_index, store);
    let mut section_tops_y: Vec<f32> = Vec::with_capacity(4);
    // Os treze snapshots e o `any_section`, numa pergunta só. Ver `paint_frame::LiveSnapshots`.
    let crate::paint_frame::LiveSnapshots {
        transform_info,
        sprite_info,
        visibility_info,
        ordering_info,
        sampling_info,
        slice_info,
        anchor_info,
        anim_info,
        timer_info,
        action_info,
        audio_info,
        blend_info,
        physics_info,
        joint_info,
        wheel_info,
        player_info,
        instance_info,
        properties_info,
        name_present,
        any_section,
    } = crate::paint_frame::LiveSnapshots::fetch();
    // ⭐⭐ Os dois CARTÕES do topo — o porquê vive no cabeçalho de [`crate::paint_cards`].
    let mut y = crate::paint_cards::paint_top_cards(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        instance_info.as_ref(),
        properties_info.as_ref(),
        inner_x,
        inner_w,
        body_top_y + Spacing::Xs.px(),
    );
    let (notes_per_section, trailing_notes) = crate::paint_frame::split_notes(store);
    // ⛔ **O macro `live_section!` MORREU em 2026-09-09, e a morte dele é o ganho.**
    //
    // Ele existia para as três seções que TODO objecto tem — as únicas que ainda se pintavam aqui
    // dentro — e era ele que as prendia a este corpo: um macro captura os locais em volta, então
    // extraí-las obrigava a transformar essa captura em argumentos. Foi o que a
    // [`crate::paint_frame_shared::paint_core_sections`] fez, e o macro ficou sem um único
    // consumidor. *Uma abstracção que sobrevive ao último chamador é uma cerca sobre um campo
    // vazio.*
    // ⭐⭐ **As TRÊS que TODO objecto tem** — §1 Name, §8 Visibility, §2 Transform. Ver o cabeçalho
    // de [`crate::paint_frame_shared::paint_core_sections`]: elas saíram daqui quando a secção
    // SIGNAL ACTIONS empurrou este orquestrador contra a catraca dele.
    y = crate::paint_frame_shared::paint_core_sections(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        &mut section_tops_y,
        inner_x,
        inner_w,
        body_top_y,
        y,
        ROW_H_PX,
        SECTION_HEAD_H,
        name_present,
        visibility_info.is_some(),
        transform_info.is_some(),
        &notes_per_section,
    );
    // **As três seções da SPRITE** — §3 Render Source, §6 Color & Tint e §4 Sprite Sheet —
    // moram em `paint_frame_shared` pelo mesmo cap que levou lá as compartilhadas. Elas andam
    // juntas porque partilham a mesma porta: **só existem se houver sprite**.
    y = crate::paint_frame_shared::paint_sprite_sections(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        &mut section_tops_y,
        inner_x,
        inner_w,
        body_top_y,
        y,
        SECTION_HEAD_H,
        sprite_info.as_ref(),
        &notes_per_section,
    );
    // **As quatro seções COMPARTILHADAS** — §5 9-Slice, §7 Ordering, §9 Sampling e §10 Material
    // & Blend — moram em `paint_frame`, como a família da física e pela mesma razão: este
    // orquestrador está numa catraca que só desce, e a §5 (2026-08-21) empurrou-o para 436
    // contra 414. As quatro andam juntas porque partilham a mesma porta — qualquer entidade com
    // `Transform` — e porque os seus quatro slots de nota (6..9) ficam obviamente distintos ao
    // lado uns dos outros.
    y = crate::paint_frame_shared::paint_shared_sections(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        &mut section_tops_y,
        inner_x,
        inner_w,
        body_top_y,
        y,
        SECTION_HEAD_H,
        slice_info.as_ref(),
        ordering_info.as_ref(),
        sampling_info.as_ref(),
        blend_info.as_ref(),
        &notes_per_section,
    );
    y = crate::paint_frame::paint_physics_sections(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        &mut section_tops_y,
        inner_x,
        inner_w,
        body_top_y,
        y,
        SECTION_HEAD_H,
        physics_info.as_ref(),
        joint_info.as_ref(),
        wheel_info.as_ref(),
        player_info.as_ref(),
        &notes_per_section,
    );
    // **AS DUAS SEÇÕES COM ESTADO DE PAINEL** — a §11 Animation e a §12 Sockets/Anchors são as
    // únicas cuja pintura depende de qual LINHA está aberta, e por isso saíram juntas para
    // `paint_frame_shared::paint_stateful_sections`.
    //
    // ⚠️ Saíram porque a §11 levou este orquestrador de 348 a 365 contra uma tolerância que **só
    // desce** — e levar só a nova devolveria o número a 348 exactos, que é ficar no mesmo sítio.
    y = crate::paint_optional::paint_optional_sections(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        &mut section_tops_y,
        inner_x,
        inner_w,
        body_top_y,
        y,
        SECTION_HEAD_H,
        anim_info.as_ref(),
        anim_selected,
        anchor_info.as_ref(),
        anchor_selected,
        timer_info.as_ref(),
        timer_selected,
        action_info.as_ref(),
        action_selected,
        audio_info.as_ref(),
        &notes_per_section,
    );
    if any_section {
        crate::paint_frame::paint_trailing_notes(
            scene,
            text_system,
            hit_index,
            store,
            inner_x,
            inner_w,
            &mut y,
            &trailing_notes,
        );
    }
    publish_and_finish(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        PanelFinish {
            any_section,
            has_selection: selection.is_some(),
            inner_x,
            inner_w,
            content_top,
            content_bottom,
            body_top_y,
            y,
            scroll_y,
            rect,
        },
        section_tops_y,
    );
    crate::paint_body::close_body(scene, text_system, theme, hit_index, rect, store, layout);
}

/// **O corpo da §8 Visibility** — a caixa `Visible` mais os controlos do componente opcional.
///
/// ⚠️ **Função IRMÃ, e não um ficheiro novo:** o `paint.rs` está com folga larga no cap de
/// FICHEIRO, e o que estourou foi o cap de FUNÇÃO do orquestrador — os dois medem grandezas
/// diferentes, e extrair para aqui cura o que estourou sem tocar no outro.
///
/// ⚠️ **As duas metades andam juntas por uma LEI:** os controlos de camada/clip/máscara/on-screen
/// só fazem sentido *debaixo* da caixa que diz se o objecto se vê, e é essa adjacência que a §8
/// promete. Separá-las poria a pergunta e a qualificação dela em sítios diferentes do painel.
#[allow(clippy::too_many_arguments)]
pub(crate) fn visibility_body(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    inner_x: f32,
    inner_w: f32,
    y: f32,
) -> f32 {
    let mut yy = sections::paint_visibility_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        inner_x,
        inner_w,
        y,
    );
    // W3 §8: the optional-component controls (layer mask / clip / mask / on-screen) sit directly
    // below the Visible toggle.
    if let Some(vis) = current_inspector_visibility_section() {
        yy = sections::paint_visibility_section(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            inner_x,
            inner_w,
            yy,
            &vis,
        );
    }
    yy
}
