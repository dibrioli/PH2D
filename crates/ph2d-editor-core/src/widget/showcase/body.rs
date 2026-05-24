//! Showcase orchestrator: paint_showcase_body.
//!
//! Extracted from showcase/mod.rs in Wave 6+7 Phase 1.C.

use super::*;

pub fn paint_showcase_body(
    rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
) {
    paint_panel_surface(rect, scene, theme);
    // Drag pill + resize gripper hit zones. Visuals are inside
    // `paint_panel_surface` / `paint_panel_corner_dot`; we register
    // the hits here against the gallery's own NodeIds so the
    // BlenderHit dispatch (`DragHandle` / `ResizeHandle`) drives
    // `GAL_PANEL` independently of the Inspector.
    let drag_handle_rect = panel_drag_handle_rect(
        rect,
        crate::widget::panel_chrome::PANEL_HEADER_H_DEFAULT,
        crate::widget::panel_chrome::PANEL_HEADER_CLOSE_RESERVE,
    );
    let resize_handle_rect = panel_resize_handle_rect(rect);
    let resize_handle_bl_rect = crate::widget::panel_chrome::panel_resize_handle_rect_bl(rect);
    hit_index.register(ids::GAL_DRAG_HANDLE, drag_handle_rect);
    hit_index.register(ids::GAL_RESIZE_HANDLE, resize_handle_rect);
    hit_index.register(ids::GAL_RESIZE_HANDLE_BL, resize_handle_bl_rect);

    // Header: title + subtitle + divider. Canonical panel title
    // (single source of truth — `panel_chrome::paint_panel_title`);
    // reserve ≈ICON_BTN_SIZE on the right for the Close button.
    let title_y = rect.y + PANEL_TITLE_BASELINE;
    let title_size = paint_panel_title(rect, "Widget Gallery", 40.0, scene, text_system, theme); // LITERAL-PX-OK: Close-button reserve
    paint_text(
        text_system,
        scene,
        "Canonical widget showcase \u{00b7} reference for peripheral agents",
        rect.x + PANEL_HEAD_PAD,
        title_y + title_size + Spacing::Xs.px(),
        TypeToken::Xs.px() - 1.0,
        rect.w - PANEL_HEAD_PAD * 2.0,
        resolve(ColorToken::Text3, theme),
    );
    // Close (X) at top-right of the header strip.
    let close_size = Spacing::Xl2.px();
    let close_rect = Rect::new(
        rect.x + rect.w - PANEL_HEAD_PAD - close_size,
        title_y - 2.0,
        close_size,
        close_size,
    );
    hit_index.register(ids::GAL_CLOSE, close_rect);
    crate::paint::paint_icon(
        scene,
        IconId::Close,
        close_rect,
        resolve(ColorToken::Text2, theme),
        StrokeToken::Default.px(),
    );

    let div_y = title_y + title_size + TypeToken::Xs.px() + Spacing::Xl.px();
    let div = Rect::new(
        rect.x + PANEL_HEAD_PAD,
        div_y,
        rect.w - PANEL_HEAD_PAD * 2.0,
        1.0,
    );
    scene.fill_rect(rect_to_vello(div), resolve(ColorToken::Border, theme));

    // Body clipped to panel rect with wheel-driven scroll offset
    // routed through `GAL_PANEL` (independent of `INSP_PANEL`).
    // Reserve room for the scrollbar even when it isn't visible so
    // the section content width is stable.
    let content_top = div_y + Spacing::Sm.px();
    let content_bottom = rect.y + rect.h - Spacing::Xs.px();
    let scroll_y = store.panel_scroll(ids::GAL_PANEL).max(0.0);
    let clip = ph2d_vector::Rect::new(
        rect.x as f64,
        content_top as f64,
        (rect.x + rect.w) as f64,
        content_bottom as f64,
    );
    scene.push_clip(&clip);

    let inner_x = rect.x + BODY_PAD;
    let scrollbar_reserve = crate::widget::SCROLLBAR_W + Spacing::Sm.px();
    let inner_w = (rect.w - BODY_PAD * 2.0 - scrollbar_reserve).max(0.0);
    let body_top_y = content_top - scroll_y + Spacing::Xs.px();
    let mut y = body_top_y;
    // Publish the body's screen-Y origin so the right-click dispatch
    // can convert screen-y → body-y when computing `before_section`
    // for a new note (`section_index_below_body_y`). Inspector's live
    // paint also writes this thread-local, but the gallery paints
    // AFTER inspector in `paint_hero_screen`, so the gallery's value
    // wins for the next dispatch tick — correct for clicks on the
    // gallery body.
    LAST_BODY_TOP_SCREEN_Y.with(|c| c.set(content_top + Spacing::Xs.px()));

    // Notes — read once and partition by `before_section`. Notes
    // tagged with `Some(i)` paint immediately above `SECTION_IDS[i]`;
    // notes with `None` paint at the tail after the last section.
    let all_notes = store.notes_for_panel(ids::GAL_PANEL).to_vec();
    let mut notes_per_section: [Vec<(usize, NoteData)>; 10] = Default::default();
    let mut trailing_notes: Vec<(usize, NoteData)> = Vec::new();
    for (idx, note) in all_notes.into_iter().enumerate() {
        match note.before_section {
            Some(i) if (i as usize) < notes_per_section.len() => {
                notes_per_section[i as usize].push((idx, note));
            }
            _ => trailing_notes.push((idx, note)),
        }
    }

    // Body-relative top-Y of each section header — captured so the
    // right-click dispatch can map a click to "which section the
    // user is targeting" for note insertion.
    let mut section_tops_y: Vec<f32> = Vec::with_capacity(SECTION_IDS.len());
    let mut section_idx: usize = 0;
    macro_rules! paint_pending_notes {
        () => {
            for (slot, note) in &notes_per_section[section_idx] {
                paint_one_note(
                    scene,
                    text_system,
                    hit_index,
                    store,
                    inner_x,
                    inner_w,
                    &mut y,
                    note,
                    *slot,
                );
            }
        };
    }
    // Section macro: paints the section, then the colored outline (if
    // the user picked one via right-click → "Section outline"), then
    // any notes anchored to THIS section (at the end of the section,
    // BEFORE the separator — UI canon post-2026-05-24), then the
    // separator. Each iteration also records the section's body-
    // relative top y so `section_index_below_body_y` works.
    //
    // Pre-canon, notes painted ABOVE the section header (separator
    // between section and note). User complaint 2026-05-24: notes
    // should belong VISUALLY to the section the user right-clicked,
    // grouped INSIDE it before the separator that ends it.
    macro_rules! section {
        ($f:ident, $section_id:expr) => {
            let y_before = y;
            push_section_top_y(&mut section_tops_y, y_before - body_top_y);
            let new_y = $f(
                scene,
                text_system,
                theme,
                hit_index,
                store,
                inner_x,
                inner_w,
                y,
            );
            if let Some(color_idx) = store.section_outline_color($section_id) {
                let rgba = HIGHLIGHTER_RGBA[color_idx.min(4) as usize];
                let pad = Spacing::Xs.px();
                let block = Rect::new(
                    inner_x - pad,
                    y_before - pad,
                    inner_w + pad * 2.0,
                    (new_y - y_before + pad * 2.0).max(0.0),
                );
                let outline_color =
                    ph2d_vector::Color::from_rgba8(rgba[0], rgba[1], rgba[2], rgba[3]); // LITERAL-COLOR-OK: user-color — showcase preview outline from user-stored ColorValue
                crate::paint::stroke_rounded_rect(
                    scene,
                    block,
                    Radius::Md.px(),
                    StrokeToken::Thick.px(),
                    outline_color,
                );
            }
            y = new_y;
            paint_pending_notes!();
            y = paint_section_separator(scene, theme, inner_x, inner_w, y);
            #[allow(unused_assignments)]
            {
                section_idx += 1;
            }
        };
    }
    section!(paint_inputs_section, ids::INSP_SECTION_INPUTS);
    section!(paint_slider_section, ids::INSP_SECTION_SLIDER);
    section!(paint_switches_section, ids::INSP_SECTION_SWITCHES);
    section!(paint_lists_section, ids::INSP_SECTION_LISTS);
    section!(paint_vector_section, ids::INSP_SECTION_VECTOR);
    section!(paint_status_section, ids::INSP_SECTION_STATUS);
    section!(paint_color_section, ids::INSP_SECTION_COLOR);
    section!(paint_actions_section, ids::INSP_SECTION_ACTIONS);
    section!(paint_identity_section, ids::INSP_SECTION_IDENTITY);
    section!(paint_card_section, ids::INSP_SECTION_CARD);
    // Trailing notes (anchor = None or out-of-range section index)
    // paint at the bottom after all sections.
    for (slot, note) in &trailing_notes {
        paint_one_note(
            scene,
            text_system,
            hit_index,
            store,
            inner_x,
            inner_w,
            &mut y,
            note,
            *slot,
        );
    }
    LAST_SECTION_TOPS_Y.with(|t| *t.borrow_mut() = section_tops_y);
    // Publish content + visible heights so the host can clamp the
    // wheel-scroll bound and so the scrollbar's thumb sizes itself
    // correctly. Mirror of the live Inspector's `set_last_inspector_*`
    // pair — kept separate so the two panels can scroll independently.
    let content_h = (y - body_top_y).max(0.0);
    let visible_h = (content_bottom - content_top).max(0.0);
    set_last_gallery_content_h(content_h);
    set_last_gallery_visible_h(visible_h);

    // Scrollbar — same widget as Inspector / Hierarchy, but routed
    // via `GALLERY_SCROLLBAR_ID` so `dispatch::scrollbar_panel_for_id`
    // sends drag-thumb moves to `GAL_PANEL`.
    if crate::widget::scrollbar_is_needed(content_h, visible_h) {
        let body = Rect::new(rect.x, content_top, rect.w, visible_h);
        let track = crate::widget::scrollbar_track_rect(body);
        let thumb = crate::widget::scrollbar_thumb_rect(track, scroll_y, content_h, visible_h);
        let is_active = matches!(store.scrollbar_drag(), Some(d) if d.panel == ids::GAL_PANEL);
        crate::widget::paint_scrollbar(
            body, scroll_y, content_h, visible_h, is_active, scene, theme,
        );
        hit_index.register(crate::widget::GALLERY_SCROLLBAR_ID, thumb);
    }

    // Late-paint phase: open Dropdown popover sits on top of every
    // section that ran before it. `take_pending_dropdown_chip` is a
    // thread_local owned by the showcase; the live Inspector never
    // paints dropdowns so there's no contention.
    if let Some((sel_idx, chip)) = take_pending_dropdown_chip() {
        let labels = ["Front", "Side", "Top"];
        let selected_label = labels.get(sel_idx).copied().unwrap_or("Front");
        let dd = Dropdown::new(
            ids::INSP_SAMPLE_DROPDOWN,
            "View",
            vec![
                DropdownOption::new(ids::INSP_SAMPLE_DD_OPT_A, "front", "Front"),
                DropdownOption::new(ids::INSP_SAMPLE_DD_OPT_B, "side", "Side"),
                DropdownOption::new(ids::INSP_SAMPLE_DD_OPT_C, "top", "Top"),
            ],
        )
        .selected(selected_label)
        .open(true);
        crate::widget::paint_dropdown_popover(&dd, chip, scene, text_system, theme);
        for (i, opt) in dd.options.iter().enumerate() {
            hit_index.register(opt.id, dd.option_rect(chip, i));
        }
    }

    scene.pop_layer();
    paint_panel_corner_dot(rect, scene, theme);
    crate::widget::panel_chrome::paint_panel_corner_dot_bl(rect, scene, theme);
    // End-of-frame re-registration of the title-bar drag handle so it
    // sits z-on-top of any body widget that scrolled into the header
    // band (prevents click-through behind the title — DIRETRIZ chip-
    // canon work, 2026-05-24). Close button must be re-registered AFTER
    // drag so it wins over the drag rect on its small overlap zone
    // AND wins over any body widget that scrolled into its position
    // (bug reported 2026-05-24: close stopped working after scroll).
    hit_index.register(ids::GAL_DRAG_HANDLE, drag_handle_rect);
    hit_index.register(ids::GAL_RESIZE_HANDLE, resize_handle_rect);
    hit_index.register(ids::GAL_RESIZE_HANDLE_BL, resize_handle_bl_rect);
    hit_index.register(
        ids::GAL_CLOSE,
        crate::widget::panel_chrome::panel_close_button_rect(rect),
    );
}
