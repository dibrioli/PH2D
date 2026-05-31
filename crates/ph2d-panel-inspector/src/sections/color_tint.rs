//! Color & Tint — Inspector section painter (split from sections.rs,
//! architecture_panel_loc_cap). Logic verbatim; behavior unchanged.

use super::*;

/// Paint one labeled tint swatch (label left, swatch right) inside
/// `cell`. The swatch fill reads `widget_color(swatch_id)` (kept in
/// sync with the live `Sprite` channel by `sync.rs`), falling back to
/// `fallback_rgba` on the cold paint. The hit rect is the swatch only,
/// so the picker opens from the colored chip — matching grid-snap's
/// color row and the Widget Gallery "Tint" sample.
#[allow(clippy::too_many_arguments)]
fn paint_tint_swatch_cell(
    cell: Rect,
    label: &str,
    swatch_id: NodeId,
    fallback_rgba: [u8; 4],
    mixed: bool,
    store: &WidgetStore,
    hit_index: &mut HitIndex,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    let label_font = TypeToken::Sm.px();
    let swatch_px = SwatchSize::Sm.px();
    let swatch_rect = Rect::new(
        cell.x + cell.w - swatch_px,
        cell.y + (cell.h - swatch_px) * 0.5,
        swatch_px,
        swatch_px,
    );
    paint_text(
        text_system,
        scene,
        label,
        cell.x,
        cell.y + (cell.h - label_font) * 0.5,
        label_font,
        (cell.w - swatch_px - Spacing::Sm.px()).max(0.0),
        resolve(ColorToken::Text2, theme),
    );
    paint_swatch_or_mixed(
        swatch_rect,
        swatch_id,
        fallback_rgba,
        mixed,
        store,
        scene,
        theme,
    );
    hit_index.register(swatch_id, swatch_rect);
}

/// Paint a color swatch, or — when the value diverges across a
/// multi-selection (`mixed`) — a neutral chip with a centered dash
/// (BulkSelect, audit F5). A swatch can't go blank like a NumberInput,
/// so the dash reuses the Indeterminate-checkbox "Mixed" language to warn
/// the user that picking a color will collapse the diverging tints. The
/// hit rect is still registered (clicking opens the picker → applies to
/// all) — the dash is the warning, not a lock.
fn paint_swatch_or_mixed(
    rect: Rect,
    swatch_id: NodeId,
    fallback_rgba: [u8; 4],
    mixed: bool,
    store: &WidgetStore,
    scene: &mut VectorScene,
    theme: Theme,
) {
    if mixed {
        paint_mixed_swatch_rect(rect, scene, theme);
    } else {
        let rgba = store.widget_color(swatch_id).unwrap_or(fallback_rgba);
        let swatch = ColorSwatch::new(swatch_id, "", rgba).size(SwatchSize::Sm);
        paint_color_swatch(&swatch, rect, scene, theme);
    }
}

/// The "Mixed" swatch visual (BulkSelect): a neutral chip with a centered
/// dash, reusing the Indeterminate-checkbox language. Size-agnostic (fills
/// whatever `rect` it's given), so both the Tint/Self cells and the 2×2
/// per-corner grid share it.
fn paint_mixed_swatch_rect(rect: Rect, scene: &mut VectorScene, theme: Theme) {
    let radius = Radius::Sm.px();
    fill_rounded_rect(scene, rect, radius, resolve(ColorToken::Bg2, theme));
    stroke_rounded_rect(scene, rect, radius, 1.0, resolve(ColorToken::Border, theme));
    paint_icon(
        scene,
        IconId::Minus,
        rect,
        resolve(ColorToken::Text2, theme),
        2.0,
    );
}

/// W2 Sprite Inspector v2 — Color & Tint section (anatomia §03 §3.6).
/// Sub-tabs `[Tint] [Self] [Corners] [Effects]` (§3.0 D11 density fix),
/// one body at a time: Tint / Self Tint modulate swatches (OKLCH picker),
/// the per-corner 2×2 gradient grid (+ live preview + Equalize), and the
/// Effects body (Opacity slider-with-chip + Tint Fill silhouette). Every
/// channel is render-ready (`RenderInstance.tint` / per-corner / opacity).
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_color_tint_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    let field_h = ROW_H_PX;
    let row_gap = Spacing::Sm.px();
    let header_h = TypeToken::Md.px() + Spacing::Md.px(); // LITERAL-PX-OK: section header band height
    let collapsed = store.is_collapsed(ids::INSP_LIVE_COLOR_SECTION);
    let color_id = ids::INSP_LIVE_COLOR_COLOR;
    let rgba = store
        .widget_color(color_id)
        .unwrap_or([0x88, 0x88, 0x88, 0xff]); // LITERAL-COLOR-OK: neutral default for unconfigured section accent
    let header = SectionHeader::new(ids::INSP_LIVE_COLOR_SECTION, "Color & Tint")
        .collapsible(!collapsed)
        .color(rgba);
    let header_rect = Rect::new(x, y, w, header_h);
    paint_section_header(&header, header_rect, scene, text_system, theme);
    if let Some(circle_rect) = ph2d_editor_core::widget::color_circle_hit_rect(&header, header_rect)
    {
        hit_index.register(color_id, circle_rect);
    }
    if collapsed {
        return y + header_h;
    }
    let mut cur_y = y + header_h;

    let sp = crate::state::current_inspector_sprite();

    // All Color & Tint controls stacked + visible at once (user
    // 2026-05-31: faster to reach than behind the old [Tint][Self]
    // [Corners][Effects] sub-tabs). Order: Tint · Self Tint · Per-corner
    // grid + Equalize · Opacity · Tint Fill. Each color swatch opens the
    // shared BlenderColorPicker (OKLCH); the chosen color round-trips via
    // `widget_color(id)` and is dispatched as a `SpriteFieldEdit` from
    // `sync.rs`. WHITE fallback covers the cold paint before first sync.

    // Tint — inherited modulate (cascades to children).
    let tint_seed = sp
        .as_ref()
        .map(|s| crate::state::tint_f32_to_u8(s.tint))
        .unwrap_or([0xff, 0xff, 0xff, 0xff]); // LITERAL-COLOR-OK: WHITE = tint default
    paint_tint_swatch_cell(
        Rect::new(x, cur_y, w, field_h),
        "Tint",
        ids::INSP_SPRITE_TINT_SWATCH,
        tint_seed,
        sp.as_ref().is_some_and(|s| s.mixed.tint),
        store,
        hit_index,
        scene,
        text_system,
        theme,
    );
    cur_y += field_h + row_gap;

    // Self Tint — local modulate (does NOT cascade).
    let self_seed = sp
        .as_ref()
        .map(|s| crate::state::tint_f32_to_u8(s.self_tint))
        .unwrap_or([0xff, 0xff, 0xff, 0xff]); // LITERAL-COLOR-OK: WHITE = self_tint default
    paint_tint_swatch_cell(
        Rect::new(x, cur_y, w, field_h),
        "Self Tint",
        ids::INSP_SPRITE_SELF_TINT_SWATCH,
        self_seed,
        sp.as_ref().is_some_and(|s| s.mixed.self_tint),
        store,
        hit_index,
        scene,
        text_system,
        theme,
    );
    cur_y += field_h + row_gap;

    // Per-corner — 2×2 swatch grid + live bilinear gradient preview +
    // Equalize. Renders via the shader's @location(9..12) attributes.
    cur_y = paint_per_corner_tab(
        scene, text_system, theme, hit_index, store, x, w, cur_y,
        sp.as_ref(),
    );
    cur_y += row_gap;

    // Opacity slider-with-chip.
    let (_, op_value) = store
        .slider(ids::INSP_SPRITE_OPACITY)
        .unwrap_or((SliderState::Normal, 1.0));
    let opacity_h = paint_slider_with_chip(
        Rect::new(x, cur_y, w, field_h),
        "Opacity",
        op_value,
        ids::INSP_SPRITE_OPACITY,
        ids::INSP_SPRITE_OPACITY_CHIP,
        store,
        hit_index,
        scene,
        text_system,
        theme,
    );
    cur_y += opacity_h + row_gap;

    // Tint Fill silhouette toggle.
    let cb_h = 18.0_f32; // LITERAL-PX-OK: matches Checkbox visual height
    let (tf_state, tf_value) = store
        .checkbox(ids::INSP_SPRITE_TINT_FILL)
        .unwrap_or((CheckboxState::Normal, CheckboxValue::Unchecked));
    let tf_rect = Rect::new(x, cur_y, w, cb_h);
    hit_index.register(ids::INSP_SPRITE_TINT_FILL, tf_rect);
    paint_checkbox(
        &Checkbox::new(ids::INSP_SPRITE_TINT_FILL, "Tint Fill")
            .state(tf_state)
            .value(tf_value),
        tf_rect,
        scene,
        text_system,
        theme,
    );
    cur_y += cb_h + row_gap;

    cur_y + SECTION_BOTTOM_PAD_PX
}

/// Per-corner tint sub-tab: a 2×2 swatch grid (`TL TR` / `BL BR`), a live
/// bilinear gradient preview to its right, and an "Equalize Corners"
/// button below. Each swatch opens the picker (dispatched as the whole
/// `PerCornerTint` array with one corner replaced — see `sync.rs`); the
/// preview re-bilerps the live (picker-overridden) corner colors so the
/// gradient updates while the user picks.
#[allow(clippy::too_many_arguments)]
fn paint_per_corner_tab(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    sp: Option<&InspectorSpriteInfo>,
) -> f32 {
    // Explanatory label — these four swatches are the per-corner (vertex)
    // tint; the renderer bilinearly interpolates them across the quad, so
    // distinct corners make a gradient (anatomia §3.6).
    let label_font = TypeToken::Sm.px();
    let label_h = label_font + Spacing::Xs.px();
    paint_text(
        text_system,
        scene,
        "Per-Corner Tint (vertex gradient)",
        x,
        y + (label_h - label_font) * 0.5,
        label_font,
        w,
        resolve(ColorToken::Text2, theme),
    );
    let y = y + label_h;
    let swatch_px = SwatchSize::Md.px();
    let gap = Spacing::Xs.px();
    let committed = sp
        .map(|s| s.per_corner_tint)
        .unwrap_or([[1.0, 1.0, 1.0, 1.0]; 4]); // WHITE = per-corner default (no gradient)
    let corner_ids = [
        ids::INSP_SPRITE_CORNER_TL,
        ids::INSP_SPRITE_CORNER_TR,
        ids::INSP_SPRITE_CORNER_BL,
        ids::INSP_SPRITE_CORNER_BR,
    ];
    let a11y = [
        "Top-left corner tint",
        "Top-right corner tint",
        "Bottom-left corner tint",
        "Bottom-right corner tint",
    ];
    // TL, TR, BL, BR positions in a 2×2 grid.
    let positions = [
        (x, y),
        (x + swatch_px + gap, y),
        (x, y + swatch_px + gap),
        (x + swatch_px + gap, y + swatch_px + gap),
    ];
    // Any per-corner divergence across a multi-selection (BulkSelect) →
    // all four show the Mixed treatment (a single flag covers the array).
    let per_corner_mixed = sp.is_some_and(|s| s.mixed.per_corner);
    let mut live = [[1.0_f32; 4]; 4];
    for i in 0..4 {
        let fallback = crate::state::tint_f32_to_u8(committed[i]);
        let rgba = store.widget_color(corner_ids[i]).unwrap_or(fallback);
        live[i] = crate::state::tint_u8_to_f32(rgba);
        let sr = Rect::new(positions[i].0, positions[i].1, swatch_px, swatch_px);
        if per_corner_mixed {
            paint_mixed_swatch_rect(sr, scene, theme);
        } else {
            let sw = ColorSwatch::new(corner_ids[i], a11y[i], rgba).size(SwatchSize::Md);
            paint_color_swatch(&sw, sr, scene, theme);
        }
        hit_index.register(corner_ids[i], sr);
    }
    let grid_w = swatch_px * 2.0 + gap;
    let grid_h = swatch_px * 2.0 + gap;
    // Live bilinear gradient preview, square, to the right of the grid.
    let preview = Rect::new(x + grid_w + Spacing::Md.px(), y, grid_h, grid_h);
    paint_corner_gradient_preview(preview, live, scene, theme);
    let mut cur_y = y + grid_h + Spacing::Sm.px();

    // Equalize Corners — copies TL → the other three (spec §3.6).
    let btn_h = ROW_H_PX;
    let eq_rect = Rect::new(x, cur_y, w, btn_h);
    let eq_state = store
        .button_state(ids::INSP_SPRITE_CORNER_EQUALIZE)
        .unwrap_or(ButtonState::Normal);
    hit_index.register(ids::INSP_SPRITE_CORNER_EQUALIZE, eq_rect);
    let eq = Button::new(ids::INSP_SPRITE_CORNER_EQUALIZE, "Equalize Corners")
        .kind(ButtonKind::Default)
        .state(eq_state);
    paint_button(&eq, eq_rect, scene, text_system, theme);
    cur_y += btn_h + Spacing::Sm.px();
    cur_y
}

/// Bilinearly sample a 4-corner color surface (`[TL, TR, BL, BR]`) at
/// `(u, v)` in the unit square — `u` left→right, `v` top→bottom. Mirrors
/// the GPU sprite path's per-corner bilerp so the preview matches what
/// renders.
fn corner_bilerp(corners: [[f32; 4]; 4], u: f32, v: f32) -> [f32; 4] {
    let lerp = |a: [f32; 4], b: [f32; 4], t: f32| {
        [
            a[0] + (b[0] - a[0]) * t,
            a[1] + (b[1] - a[1]) * t,
            a[2] + (b[2] - a[2]) * t,
            a[3] + (b[3] - a[3]) * t,
        ]
    };
    let top = lerp(corners[0], corners[1], u); // TL → TR
    let bot = lerp(corners[2], corners[3], u); // BL → BR
    lerp(top, bot, v)
}

/// Paint a small bilinear preview of a 4-corner tint gradient (`TL, TR,
/// BL, BR`). Vello has no native 4-corner gradient, so we sample the
/// bilinear surface over an N×N cell grid — the standard honest CPU
/// preview. (The real GPU sprite path does true per-corner bilerp via
/// the shader's `@location(9..12)` attributes; this only mirrors it.)
fn paint_corner_gradient_preview(
    rect: Rect,
    corners: [[f32; 4]; 4],
    scene: &mut VectorScene,
    theme: Theme,
) {
    const CELLS: usize = 8;
    for j in 0..CELLS {
        for i in 0..CELLS {
            let u = (i as f32 + 0.5) / CELLS as f32;
            let v = (j as f32 + 0.5) / CELLS as f32;
            let c = corner_bilerp(corners, u, v);
            let b = crate::state::tint_f32_to_u8(c);
            let x0 = rect.x + rect.w * (i as f32 / CELLS as f32);
            let x1 = rect.x + rect.w * ((i as f32 + 1.0) / CELLS as f32);
            let y0 = rect.y + rect.h * (j as f32 / CELLS as f32);
            let y1 = rect.y + rect.h * ((j as f32 + 1.0) / CELLS as f32);
            // Half-pixel overlap right/down kills sub-pixel seams; later
            // cells overdraw earlier ones, so the bleed is harmless and
            // the final column/row is hidden under the border stroke below.
            let cell = Rect::new(x0, y0, x1 - x0 + 0.5, y1 - y0 + 0.5); // LITERAL-PX-OK: anti-seam overlap
            // The user's per-corner tint colors, bilerp'd — sprite content,
            // not chrome, so it lives outside the theme system.
            let fill = VelloColor::from_rgba8(b[0], b[1], b[2], b[3]); // LITERAL-COLOR-OK: bilerp'd sprite tint, not chrome
            scene.fill_rect(rect_to_vello(cell), fill);
        }
    }
    stroke_rounded_rect(
        scene,
        rect,
        Radius::Sm.px(),
        1.0,
        resolve(ColorToken::Border, theme),
    );
}

#[cfg(test)]
mod corner_gradient_tests {
    use super::corner_bilerp;

    const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
    const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

    #[test]
    fn uniform_corners_sample_uniform() {
        // No gradient configured (all WHITE) → every sample is WHITE.
        let c = [WHITE; 4];
        for &(u, v) in &[(0.0, 0.0), (1.0, 1.0), (0.5, 0.5), (0.3, 0.7)] {
            assert_eq!(corner_bilerp(c, u, v), WHITE);
        }
    }

    #[test]
    fn corners_sample_at_their_own_uv() {
        // [TL, TR, BL, BR]; the unit-square corners read back each corner.
        let c = [
            [1.0, 0.0, 0.0, 1.0], // TL red
            [0.0, 1.0, 0.0, 1.0], // TR green
            [0.0, 0.0, 1.0, 1.0], // BL blue
            WHITE,                // BR white
        ];
        assert_eq!(corner_bilerp(c, 0.0, 0.0), c[0]);
        assert_eq!(corner_bilerp(c, 1.0, 0.0), c[1]);
        assert_eq!(corner_bilerp(c, 0.0, 1.0), c[2]);
        assert_eq!(corner_bilerp(c, 1.0, 1.0), c[3]);
    }

    #[test]
    fn center_is_the_four_corner_average() {
        let c = [BLACK, WHITE, WHITE, BLACK]; // diagonal split
        // (0.5, 0.5) = mean of the four corners = 0.5 grey.
        let m = corner_bilerp(c, 0.5, 0.5);
        assert!((m[0] - 0.5).abs() < 1e-6, "got {m:?}");
        assert!((m[3] - 1.0).abs() < 1e-6, "alpha preserved");
    }

    #[test]
    fn top_edge_midpoint_interpolates_tl_tr_only() {
        let c = [BLACK, WHITE, [0.2, 0.2, 0.2, 1.0], [0.3, 0.3, 0.3, 1.0]];
        // v=0 top edge, u=0.5 → halfway TL↔TR, untouched by the bottom row.
        let s = corner_bilerp(c, 0.5, 0.0);
        assert!((s[0] - 0.5).abs() < 1e-6, "got {s:?}");
    }
}
