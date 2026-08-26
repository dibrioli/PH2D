//! Inspector panel paint — ADR-0029 Phase C.1 port of
//! `ph2d_editor_core::screens::hero::inspector::{paint_thunk,
//! paint_inspector}`.
//!
//! `paint` is the typed entry point invoked by the `Panel` trait. It
//! gates on visibility (via [`PanelHostInternal::panel_visible`]),
//! runs the snapshot sync, paints the live Inspector body, and
//! publishes scroll bounds back to the store.

use crate::paint_frame::{PanelFinish, begin_section, finish_section, publish_and_finish};
use crate::state::{
    self, current_inspector_visibility_section, last_inspector_content_h, last_inspector_visible_h,
};
use crate::sync::sync_inspector_from_snapshots;
use crate::{InspectorPanel, sections};
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{HitIndex, WidgetStore};
use ph2d_editor_core::panel::{PaintCtx, Panel};
use ph2d_editor_core::screens::{HeroLayout, HeroSelection};
use ph2d_editor_core::widget::SCROLLBAR_W;
use ph2d_editor_core::widget::panel_chrome::{
    paint_panel_corner_dot, paint_panel_surface, panel_drag_handle_rect, panel_resize_handle_rect,
};
use ph2d_editor_core::widget::showcase::LAST_BODY_TOP_SCREEN_Y;
use ph2d_editor_core::widget::showcase::paint_section_separator;
use ph2d_text::TextSystem;
use ph2d_tokens::{ROW_H_PX, Spacing, Theme};
use ph2d_vector::VectorScene;

const BODY_PAD: f32 = 10.0; // LITERAL-PX-OK: inspector body inset
const SECTION_HEAD_H: f32 = ROW_H_PX;

pub(crate) fn paint(inspector_state: &mut state::InspectorState, ctx: &mut PaintCtx) {
    if !ctx.host.panel_visible(InspectorPanel::ID) {
        return;
    }
    sync_inspector_from_snapshots(inspector_state, ctx.host);
    let display_unit = ctx.host.project().display_unit;
    let ppm = ctx.host.project().pixels_per_meter;
    state::set_current_display_unit(display_unit, ppm);
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
            ctx.layout,
            selection_clone.as_ref(),
            ctx.scene,
            ctx.text_system,
            theme,
            hit_index,
            store,
            &mut inspector_state.anchor_selected,
            &mut inspector_state.anim_selected,
        );
    }
    state::set_current_display_unit(display_unit, ppm); // keep symmetric with legacy
    // Publish content_h + clamp scroll right after paint so
    // `dispatch_wheel` sees the new bounds on the very next event.
    let content_h = last_inspector_content_h();
    let visible_h = last_inspector_visible_h();
    let store = ctx.host.store_mut();
    store.set_panel_content_h(ids::INSP_PANEL, content_h);
    store.set_panel_visible_h(ids::INSP_PANEL, visible_h);
    let max_scroll = (content_h - visible_h).max(0.0);
    let cur = store.panel_scroll(ids::INSP_PANEL);
    if cur > max_scroll {
        store.set_panel_scroll(ids::INSP_PANEL, max_scroll);
    }
}

#[allow(clippy::too_many_arguments)]
fn paint_inspector(
    layout: &HeroLayout,
    selection: Option<&HeroSelection>,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    // §12: qual linha da lista de âncoras está aberta. **Estado do painel**, saturado dentro
    // de `paint_anchor_section` contra o tamanho da lista — apagar a última âncora não o pode
    // deixar a apontar para o vazio.
    anchor_selected: &mut usize,
    // §11: qual animação está aberta no editor. Mesmo contrato do `anchor_selected`.
    anim_selected: &mut usize,
) {
    let rect = layout.inspector;
    paint_panel_surface(rect, scene, theme);
    let drag_handle_rect = panel_drag_handle_rect(
        rect,
        ph2d_editor_core::widget::panel_chrome::PANEL_HEADER_H_DEFAULT,
        // Inspector has no close button — its visibility is governed
        // by the TopBar toggle. Reserve nothing on the right so the
        // drag area spans the full width.
        0.0,
    );
    let resize_handle_rect = panel_resize_handle_rect(rect);
    let resize_handle_bl_rect =
        ph2d_editor_core::widget::panel_chrome::panel_resize_handle_rect_bl(rect);
    hit_index.register(ids::INSP_DRAG_HANDLE, drag_handle_rect);
    hit_index.register(ids::INSP_RESIZE_HANDLE, resize_handle_rect);
    hit_index.register(ids::INSP_RESIZE_HANDLE_BL, resize_handle_bl_rect);

    // O cabeçalho — título, subtítulo, fechar e o divisor. Ver `paint_head`.
    let content_top =
        crate::paint_head::paint_panel_head(rect, scene, text_system, theme, hit_index, store);

    let content_bottom = rect.y + rect.h - Spacing::Xs.px();
    let scroll_y = store.panel_scroll(ids::INSP_PANEL).max(0.0);
    let clip = ph2d_vector::Rect::new(
        rect.x as f64,
        content_top as f64,
        (rect.x + rect.w) as f64,
        content_bottom as f64,
    );
    scene.push_clip(&clip);

    let inner_x = rect.x + BODY_PAD;
    let scrollbar_reserve = SCROLLBAR_W + Spacing::Sm.px();
    let inner_w = (rect.w - BODY_PAD * 2.0 - scrollbar_reserve).max(0.0);
    let body_top_y = content_top - scroll_y + Spacing::Xs.px();
    let mut section_tops_y: Vec<f32> = Vec::with_capacity(4);
    LAST_BODY_TOP_SCREEN_Y.with(|c| c.set(content_top + Spacing::Xs.px()));
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
        blend_info,
        physics_info,
        joint_info,
        wheel_info,
        player_info,
        name_present,
        any_section,
    } = crate::paint_frame::LiveSnapshots::fetch();
    let mut y = body_top_y + Spacing::Xs.px();

    let (notes_per_section, trailing_notes) = crate::paint_frame::split_notes(store);
    // Section macro: paints the section, then the outline (if any),
    // then notes anchored to THIS section (at the END, before the
    // separator the caller adds next). UI canon post-2026-05-24:
    // notes belong VISUALLY to the section the user right-clicked,
    // grouped inside it. Pre-canon notes painted ABOVE the header.
    macro_rules! live_section {
        ($section_id:expr, $section_idx:expr, $header_h:expr, $body:block) => {{
            let y_before = y;
            begin_section(
                &mut section_tops_y,
                hit_index,
                inner_x,
                inner_w,
                body_top_y,
                y_before,
                $section_id,
                $header_h,
            );
            let new_y: f32 = $body;
            finish_section(
                scene,
                text_system,
                hit_index,
                store,
                inner_x,
                inner_w,
                $section_id,
                y_before,
                new_y,
                &notes_per_section[$section_idx],
            )
        }};
    }

    if name_present {
        y = live_section!(ids::INSP_LIVE_NAME_SECTION, 0, ROW_H_PX, {
            sections::paint_entity_name_row(
                scene,
                text_system,
                theme,
                hit_index,
                store,
                inner_x,
                inner_w,
                y,
            )
        });
        y = paint_section_separator(scene, theme, inner_x, inner_w, y);
    }
    if visibility_info.is_some() {
        y = live_section!(ids::INSP_LIVE_VISIBILITY_SECTION, 1, ROW_H_PX, {
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
            // W3 §8: the optional-component controls (layer mask / clip /
            // mask / on-screen) sit directly below the Visible toggle.
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
        });
        y = paint_section_separator(scene, theme, inner_x, inner_w, y);
    }
    if transform_info.is_some() {
        y = live_section!(ids::INSP_LIVE_TRANSFORM_SECTION, 2, SECTION_HEAD_H, {
            sections::paint_transform_section(
                scene,
                text_system,
                theme,
                hit_index,
                store,
                inner_x,
                inner_w,
                y,
            )
        });
        y = paint_section_separator(scene, theme, inner_x, inner_w, y);
    }
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
    y = crate::paint_frame_shared::paint_stateful_sections(
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
    // **OS TRÊS POPOVERS DIFERIDOS**, pintados por último para ficarem acima de tudo.
    // ⚠️ Saíram daqui em 2026-08-23 para `paint_frame_shared`: o «Rides Parent Anchor» da §12
    // levou este orquestrador de 380 a 403 contra uma tolerância que **só desce**. Os três
    // andam juntos porque partilham UMA lei — o popover pinta-se fora da ordem das seções —
    // e levar só o novo deixaria o número onde estava, que não é encolher.
    crate::paint_frame_shared::paint_deferred_popovers(scene, text_system, theme, hit_index);

    scene.pop_layer();

    paint_panel_corner_dot(rect, scene, theme);
    ph2d_editor_core::widget::panel_chrome::paint_panel_corner_dot_bl(rect, scene, theme);
    hit_index.register(ids::INSP_DRAG_HANDLE, drag_handle_rect);
    hit_index.register(ids::INSP_RESIZE_HANDLE, resize_handle_rect);
    hit_index.register(ids::INSP_RESIZE_HANDLE_BL, resize_handle_bl_rect);
    // Re-register close AFTER drag so scrolled body widgets behind
    // the title can't shadow it (bug pattern reported 2026-05-24
    // for Widget Gallery; same surface here).
    hit_index.register(
        ids::INSP_CLOSE,
        ph2d_editor_core::widget::panel_chrome::panel_close_button_rect(rect),
    );
}

/// The section separator, callable from `paint_frame`'s extracted section
/// painters. A thin forward rather than a second import path, so there stays
/// exactly one place that knows what a separator looks like.
pub(crate) fn paint_section_separator_at(
    scene: &mut ph2d_vector::VectorScene,
    theme: ph2d_tokens::Theme,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    paint_section_separator(scene, theme, x, w, y)
}
