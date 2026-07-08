use super::*;

/// Top-level hero paint orchestrator. Clears + re-populates the
/// hit-index, then walks each region painter in z-order
/// (canvas → selection overlay → chrome → HUD).
pub fn paint_hero_screen(
    hero: &mut HeroScreen,
    viewport: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
) {
    // Publish the user-picked radius scale to the thread-local read
    // by `paint::fill_rounded_rect` / `stroke_rounded_rect`. Set
    // every frame so it stays in sync with the topbar's radius menu.
    crate::paint::set_radius_scale(hero.store.radius_scale());
    // Same pattern for the text-rendering strategy — read by
    // `paint_text*` via the `paint::text_rendering()` thread-local.
    crate::paint::set_text_rendering(hero.text_rendering);
    // Stash the viewport so chrome event handlers in `chrome/` can
    // make smart layout decisions (cascade submenu side-flip etc.).
    hero.last_viewport = viewport;

    // Rail width follows the user's Themes-menu rail-button-size
    // preset (Small / Medium / Large; default Small). Switching size
    // shifts Inspector/Hierarchy x-positions accordingly.
    let rail_w = hero.store.rail_button_size().rail_width_px();
    // Motion Nodes M0.T4: `center_split` is `None` for every non-Motion tool, so
    // this is identical to the legacy layout there; the Motion bridge sets a split
    // while its tool is active.
    let mut layout = HeroLayout::for_viewport_split(
        viewport,
        hero.view.ui_mirrored,
        rail_w,
        hero.view.center_split,
    );
    // Apply user-driven panel drag offsets to the Inspector +
    // Hierarchy rects. The offsets live on the WidgetStore's
    // `blender_picker_offset` side-table (panel-agnostic — the
    // dispatch's BlenderHitKind::DragHandle path stores the
    // offset under the `parent` NodeId regardless of widget kind).
    // Clamp helper lives in `style::clamp_panel_rect` so the floating
    // panel thunks (widget gallery, grid snap) share the same math.
    let insp_off = hero.store.blender_picker_offset(ids::INSP_PANEL);
    let hier_off = hero.store.blender_picker_offset(ids::HIER_PANEL);
    let insp_resize = hero.store.panel_resize_delta(ids::INSP_PANEL);
    let hier_resize = hero.store.panel_resize_delta(ids::HIER_PANEL);
    let (insp_rect, insp_clamped_off, insp_clamped_resize) =
        style::clamp_panel_rect(layout.inspector, insp_off, insp_resize, viewport);
    let (hier_rect, hier_clamped_off, hier_clamped_resize) =
        style::clamp_panel_rect(layout.hierarchy, hier_off, hier_resize, viewport);
    layout.inspector = insp_rect;
    layout.hierarchy = hier_rect;
    // Image-tool panels (BgRemoval, Padding, CEQ, Upscale, Equalize
    // Sizes) share the right-dock slot with Inspector. Mirror the
    // resized + dragged rect so they paint at the same position and
    // size when active. The handles inside those panels parent to
    // INSP_PANEL too (single dock-slot persistence — resizing CEQ
    // also resizes the Inspector when the user switches back).
    layout.bgremoval = insp_rect;
    layout.padding = insp_rect;
    // W2.T2.1 Day-7 follow-up: Painter sidebar shares Inspector slot too
    // (single dock-slot persistence). Sem este propagação, drag/resize não
    // afetavam o painter_sidebar visualmente + rect publicado divergia do
    // que dispatch hit-test usava → click vazava pra canvas atrás.
    layout.painter_sidebar = insp_rect;
    // W3.T3.4: Painter layers panel shares the Inspector dock slot too —
    // mirror the resized/dragged rect so its chrome + published panel rect
    // align with dispatch hit-test (else clicks leak to the canvas behind).
    layout.painter_layers = insp_rect;
    if (insp_clamped_off.0 - insp_off.0).abs() > f32::EPSILON
        || (insp_clamped_off.1 - insp_off.1).abs() > f32::EPSILON
    {
        hero.store.set_blender_picker_offset(
            ids::INSP_PANEL,
            insp_clamped_off.0,
            insp_clamped_off.1,
        );
    }
    if (hier_clamped_off.0 - hier_off.0).abs() > f32::EPSILON
        || (hier_clamped_off.1 - hier_off.1).abs() > f32::EPSILON
    {
        hero.store.set_blender_picker_offset(
            ids::HIER_PANEL,
            hier_clamped_off.0,
            hier_clamped_off.1,
        );
    }
    if (insp_clamped_resize.0 - insp_resize.0).abs() > f32::EPSILON
        || (insp_clamped_resize.1 - insp_resize.1).abs() > f32::EPSILON
    {
        hero.store.set_panel_resize_delta(
            ids::INSP_PANEL,
            insp_clamped_resize.0,
            insp_clamped_resize.1,
        );
    }
    if (hier_clamped_resize.0 - hier_resize.0).abs() > f32::EPSILON
        || (hier_clamped_resize.1 - hier_resize.1).abs() > f32::EPSILON
    {
        hero.store.set_panel_resize_delta(
            ids::HIER_PANEL,
            hier_clamped_resize.0,
            hier_clamped_resize.1,
        );
    }
    hero.hit_index.clear_for_frame();

    // M14.5: in live mode (`grid_view` published) the compositor pass
    // shows `game_rt` underneath wherever vello_rt has α=0, so we
    // **skip** the opaque canvas Bg1 fill. Chrome panels (BgElev,
    // panels, topbar) paint their own backdrops — verified in the
    // M14.5 audit. Fixture mode keeps the canvas tint so mockup
    // screenshots stay theme-correct.
    if hero.grid.view.is_none() {
        paint_canvas_bg(&layout, scene, hero.theme);
    }
    // M14.4b: world-space grid overlay. Painted between the canvas
    // background and the selection marquee so the marquee remains
    // legible over the grid. Skipped when toggle is off or host
    // hasn't published a camera view. We substitute the layout's
    // computed canvas rect into the view so the host doesn't have
    // to mirror layout math — it only owns camera + window dims.
    //
    // Layer-order toggle (2026-05-15): the compositor currently
    // composes `game_rt_ldr` UNDER `vello_intermediate` in a single
    // pass — chrome (including the grid) always lands on top of
    // sprites. Real "behind" rendering needs a second Vello
    // intermediate + a 3-layer compositor shader (TODO follow-up).
    // For now we approximate by halving the grid's effective opacity
    // when `grid_in_front == false`, which reads as "the grid is
    // farther / underneath" without changing the compositing path.
    if hero.view.grid_visible
        && let Some(view) = hero.grid.view
    {
        let view = crate::grid::GridView {
            canvas: layout.canvas,
            ..view
        };
        let mut state_for_paint = hero.grid.snap_state.clone();
        if !state_for_paint.grid_in_front {
            state_for_paint.opacity *= 0.4; // LITERAL-PX-OK: grid behind-canvas dim ratio (visual effect)
        }
        crate::grid_snap::render::paint(scene, &view, &state_for_paint, hero.theme);
    }
    // M14.4c: the legacy mockup selection marquee draws a fixed-size
    // dashed rect at the CANVAS center in screen pixels — it has no
    // world-space coupling and so doesn't follow pan/zoom. Skip it
    // when a `grid_view` is published (live ECS mode) so we don't
    // mislead users into thinking the marquee tracks an entity.
    // Fixture mode keeps the placeholder marquee for the mockup
    // screenshots.
    if hero.grid.view.is_none()
        && let Some(sel) = hero.selection.as_ref()
    {
        paint_selection_overlay(&layout, sel, scene, text_system, hero.theme);
    }
    // M14.7 B: live-mode sprite gizmo. The host publishes a
    // `gizmo_view` carrying the selected sprite's world-space bbox +
    // current camera; the painter projects to screen pixels with the
    // same math the grid uses (so the gizmo and grid stay aligned
    // across pan/zoom).
    if let Some(view) = hero.gizmo.view {
        crate::gizmo::paint_sprite_gizmo(scene, &view, hero.theme, &mut hero.hit_index);
    }
    // Onda 2C + z-order fix: the multi-selection extra + global gizmos
    // paint here — at the SAME layer as the primary gizmo, i.e. above the
    // scene but BELOW the floating panels (painted later in this fn). They
    // used to paint in the shell AFTER `paint_hero_screen` returned, which
    // put them visually on top of panels AND registered their hit rects
    // after the panel barriers (so handles were clickable through chrome).
    // Snapshot the `(bits, view)` pairs first so `hero.gizmo` isn't borrowed
    // while `&mut hero.hit_index` + `&mut hero.gizmo.gizmo_hit_map` are held.
    // Each pair carries its own bits, so a handle can never be registered
    // under a different sprite's identity (no zip against `extra_selection`).
    let extras_snapshot: Vec<(u64, crate::gizmo::GizmoView)> = hero.gizmo.extra_views.clone();
    for (bits, v) in extras_snapshot {
        crate::gizmo::paint_sprite_gizmo_keyed(
            scene,
            &v,
            hero.theme,
            &mut hero.hit_index,
            &mut hero.gizmo.gizmo_hit_map,
            crate::gizmo::GizmoTarget::ExtraIndividual(bits),
            1.0,
        );
    }
    if let Some(v) = hero.gizmo.global_view {
        crate::gizmo::paint_sprite_gizmo_keyed(
            scene,
            &v,
            hero.theme,
            &mut hero.hit_index,
            &mut hero.gizmo.gizmo_hit_map,
            crate::gizmo::GizmoTarget::Global,
            2.0, // LITERAL-PX-OK: global gizmo outline stroke width
        );
    }
    paint_top_bar(
        &layout,
        scene,
        text_system,
        hero.theme,
        &mut hero.hit_index,
        &hero.store,
        hero.image_edit.mode_on,
    );
    // Publish Inspector + Hierarchy panel rects so wheel-event
    // dispatch can route to them. Both are static (no drag offset).
    // When a panel is hidden via its left-rail toggle we DROP the
    // published rect so dispatch's "inside panel" tests don't match
    // a stale geometry.
    if hero.is_panel_visible("inspector") {
        hero.store.set_panel_rect(ids::INSP_PANEL, layout.inspector);
    } else {
        hero.store.clear_panel_rect(ids::INSP_PANEL);
    }
    if hero.is_panel_visible("hierarchy") {
        hero.store.set_panel_rect(ids::HIER_PANEL, layout.hierarchy);
    } else {
        hero.store.clear_panel_rect(ids::HIER_PANEL);
    }
    // Mirror the global picker's current value into the target
    // widget's `widget_colors` slot before either panel paints so
    // color circles inside the Inspector see this frame's value.
    if let Some(target) = hero.store.picker_target()
        && let Some((value, _, _, _)) = hero.store.blender_picker(ids::INSP_BLENDER_PICKER)
    {
        hero.store.set_widget_color(target, value.rgba);
        // Mirror Grid-Settings swatch edits back into the grid_snap
        // state so the canvas overlay re-paints with the new color.
        if target == crate::grid_snap::ids::GS_COLOR_PICKER {
            hero.grid.snap_state.color_rgba = value.rgba;
        }
    }
    // ADR-0029 Phase C.2: Hierarchy migrated to a typed Panel — selection
    // label is read via `host.selection()` inside the panel's `paint`;
    // live entries and rename-target live in panel-owned thread-local /
    // typed `HierarchyState` respectively. No host-side publish needed.
    //
    // Publish the picker's outer rect so dispatch's "is the click
    // inside the picker?" test can reason about its bounds.
    if hero.store.picker_target().is_some()
        && let Some(picker_rect) = color_picker_demo::current_picker_rect(&layout, &hero.store)
    {
        hero.store
            .set_panel_rect(ids::INSP_BLENDER_PICKER, picker_rect);
    } else {
        hero.store.clear_panel_rect(ids::INSP_BLENDER_PICKER);
    }

    // Wave 5 stage D — paint each panel via the PanelRegistry in
    // z-order. Bottom-first, so the panel most recently clicked /
    // dragged / opened sits on top. Panels that haven't been touched
    // yet inherit a default order at the bottom (fallback list below
    // also covers floating panels that have their own panel rects:
    // GAL_PANEL + GS_PANEL).
    //
    // INSP_BLENDER_PICKER is intentionally NOT in the panel
    // registry — it's painted out-of-band AFTER every floating panel
    // (see `paint_blender_picker_demo` below) so it sits on top of
    // every other panel regardless of z order.
    //
    // Each manifest's `paint_fn` owns its full per-frame logic:
    // visibility check + lazy default rect + drag/resize clamp +
    // chrome publish + actual paint + content_h publish + scroll
    // clamp + stale-rect cleanup on hide. Adding a new panel needs
    // zero edits to this iteration — drop `PANEL_MANIFEST` in the
    // panel module + 1 line in `panel_registry::PANEL_REGISTRY`.
    let mut z_order: Vec<ph2d_a11y::NodeId> = hero.store.panel_z_order().to_vec();
    for &fallback in &[
        ids::HIER_PANEL,
        ids::INSP_PANEL,
        // Geometry-graph smoke panel (ADR-0065): docks over the inspector rect
        // when `PH2D_VECTOR_GRAPH=1`. Its own `paint()` no-ops when hidden, so
        // this is inert in the normal app. After INSP_PANEL → paints on top.
        ids::VGRAPH_PANEL,
        ids::BGR_PANEL,
        ids::PAD_PANEL,
        ids::CEQ_PANEL,
        ids::EQS_PANEL,
        ids::UPS_PANEL,
        ids::PAINTER_SIDEBAR_PANEL,
        ids::VECTOR_INSPECTOR_PANEL,
        // Vector tool Style panel (ADR-0108 docked `ph2d-panel-vector`): docks
        // over the inspector slot while the `vector` tool is active. Its
        // `paint()` no-ops when hidden, so this is inert otherwise.
        ids::VECTOR_PANEL,
        // Motion Nodes docked panels (M0.T9): the graph-editor panel fills the
        // `motion_graph` split region, the params panel takes the inspector slot.
        // Both `paint()` no-op when the `motion` tool is inactive (bridge-driven
        // visibility), so they're inert otherwise. WITHOUT these entries a
        // registered+visible panel is never reached by this z-order walk → never
        // painted (the split would be invisible).
        ids::MOTION_GRAPH_PANEL,
        ids::MOTION_PARAMS_PANEL,
        // General timeline (docs/Timeline W2): bottom-docked, visibility toggled
        // by the `timeline` key. WITHOUT this entry the registered+visible panel
        // is never reached by the z-order walk → never painted.
        ids::TIMELINE_PANEL,
        ids::INSP_BLENDER_PICKER,
        ids::GAL_PANEL,
        ids::AUDIO_MIXER_PANEL,
        crate::grid_snap::ids::GS_PANEL,
    ] {
        if !z_order.contains(&fallback) {
            z_order.push(fallback);
        }
    }
    // ADR-0029 Phase D: legacy fn-pointer dispatch deleted. Every
    // in-tree panel lives in `crate::panel::PANEL_REGISTRY` as a
    // typed `Panel<State>`. The z-order walk resolves each id to its
    // typed entry; ids that don't match (e.g. `INSP_BLENDER_PICKER`,
    // painted out-of-band below) are silently skipped.
    crate::panel::with_registry_opt(|reg| {
        for panel_id in z_order {
            if let Some(idx) = reg.find_by_panel_node_id(panel_id) {
                // Hit barrier: register the panel rect BEFORE the
                // widgets inside `panel.paint()` so the gizmo's hit
                // rects (registered earlier this frame) don't bleed
                // through the panel surface. `HitIndex::hit()` walks
                // back-to-front, so internal panel widgets registered
                // by `paint()` below still outrank this barrier — only
                // empty panel area falls back to it. Enio 2026-05-25:
                // "alças do gizmo da sprite podem ser acessadas
                // através dos painéis. Isso não pode acontecer."
                if let Some(panel_rect) = hero.store.panel_rect(panel_id) {
                    hero.hit_index.register(panel_id, panel_rect);
                }
                let mut typed_ctx = crate::panel::PaintCtx {
                    host: hero,
                    layout: &layout,
                    viewport,
                    scene,
                    text_system,
                };
                reg.panels_mut()[idx].paint(&mut typed_ctx);
            }
        }
    });
    // hero/scene/text_system unborrowed for the
    // rest of paint_hero_screen (bottom HUD, picker overlay, tooltip,
    // context menu, drop overlay).
    //
    // Left rail painted AFTER the docked panels so its buttons — and the
    // Painter Shapes flyout, which extends over the Inspector/Hierarchy area —
    // sit ABOVE them, both visually and for hit-testing (HitIndex walks
    // back-to-front, so the rail chips registered here win any overlapping
    // click). Still below the bottom HUD / color picker / context menu, which
    // paint after this (unchanged). Painter mode = Image-Tools on AND the
    // active tool is the Painter (mirrored shell-side into `active_tool_id`),
    // which swaps the transform block for the paint tools.
    let painter_active =
        hero.image_edit.mode_on && hero.image_edit.active_tool_id == Some("painter");
    paint_left_rail(
        &layout,
        scene,
        text_system,
        hero.theme,
        &mut hero.hit_index,
        &hero.store,
        painter_active,
    );
    if hero.view.stats_visible {
        paint_bottom_hud(&layout, scene, text_system, hero.theme, hero.stats);
    }
    // W2.T2.3: the Painter color swatch lives INSIDE the Painter sidebar
    // panel (`ph2d-panel-painter-sidebar`), painted there alongside
    // Size/Opacity and registering hit `ids::PAINTER_COLOR_THUMB`. The
    // open-picker dispatch (pointer.rs) + the bridge read-back are keyed
    // on that hit id and are placement-agnostic, so nothing here paints
    // the swatch — the docked panel owns it (the earlier floating
    // top-right swatch was the wrong home and was removed).
    // BlenderColorPicker — painted AFTER every floating panel
    // (Inspector, Hierarchy, Widget Gallery, Grid Settings) so it
    // never sits visually behind one of them. The painter is a no-op
    // when `picker_target` is None.
    if hero.store.picker_target().is_some() {
        color_picker_demo::paint_blender_picker_demo(
            &layout,
            scene,
            text_system,
            hero.theme,
            &mut hero.hit_index,
            &hero.store,
        );
    }
    // Tooltip overlay on top of all chrome (Phase 3 polish).
    topbar::paint_hover_tooltip(scene, text_system, hero.theme, &hero.hit_index, &hero.store);
    // Context menu overlay — last so the floating menu sits above
    // every panel, including the floating BlenderColorPicker.
    context_menu_overlay::paint_context_menu_overlay(
        scene,
        text_system,
        hero.theme,
        &mut hero.hit_index,
        &hero.store,
        &hero.project,
        viewport,
    );
    // Fill (Bucket) "Fill adjust" modal — a floating, draggable card at the ColorDrop release point
    // (no-op when closed). Painted after the context menu so its hit rects sit above the canvas.
    chrome::paint_fill_adjust_modal(
        scene,
        text_system,
        hero.theme,
        &mut hero.hit_index,
        &hero.store,
        viewport,
    );
    // M14.4e: file-drop overlay sits above EVERY layer (chrome,
    // tooltips, context menus) so the user always sees the "Drop to
    // import" hint while the OS drag is active.
    if let Some((paths, cursor)) = hero.dragging_files.as_ref() {
        paint_drop_overlay(&layout, paths, *cursor, scene, text_system, hero.theme);
    }
}
