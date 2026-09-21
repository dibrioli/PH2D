//! The Painter dock's **Watercolor** section — the wet-media look, grouped into technique cards
//! (redesign 2026-07-07, `docs/Painter/12_…`): an **Enable** master toggle gates the section; when on
//! it paints three bordered cards named for what the painter controls —
//! **Wash** (how the stroke dries: Body · Concentration · Edge Darkening · Bleed · Ragged Edge),
//! **Brush** (what's on the brush: Charge · Dilution · Pull), and
//! **Water** (interaction with paint already down: Rewet · Smudge). Names mirror the
//! industry vocabulary (Rebelle "Edge Darkening" / "Re-wet", Corel "Concentration", Procreate
//! Charge/Dilution/Pull).
//!
//! ⚠️ **O `Pigment` SAIU do cartão Water em 2026-09-20** (ordem do dono, *«ligue o digital»*): a
//! mistura subtractiva deixou de ser da aguada e vive hoje no cartão **`Mixing`**
//! ([`crate::paint_pigment`]), acima da secção do meio, porque governa três deles. *Este cartão
//! voltou a ser só o que o nome dele diz.*
//!
//! All controls are fixed-id, tool-global widgets (registered in [`crate::populate`]); this module only
//! paints them off the published [`BrushSettings`] snapshot. The number fields forward the real value as
//! `SetValue` (routed via [`ph2d_editor_core::ids::PAINTER_WATERCOLOR_FIELDS`] + `is_param_field`); the
//! Enable toggle + section reset forward as `PanelEvent::Click`.

use crate::card::{card_frame, card_row};
use crate::number_field;
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::{SegmentedAdaptive, SegmentedOption, paint_segmented_adaptive};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_tokens::ROW_H_PX;
use ph2d_tool_painter::BrushSettings;

/// Slider range bounds — the parameter domains (matching the tool's `set_brush_*` clamps), not design
/// tokens. The `0..1` params (Charge / Dilution / …) use the allowlisted `0.0`/`1.0` inline.
const EDGE_MAX: f32 = 8.0; // LITERAL-PX-OK: watercolor Edge-darkening gain range bound (parameter domain)
const SPREAD_MIN: f32 = 1.0; // LITERAL-PX-OK: watercolor Bleed blur-radius min (px)
const SPREAD_MAX: f32 = 48.0; // LITERAL-PX-OK: watercolor Bleed blur-radius max (px; 48 so big brushes aren't capped dry)
const DEPTH_MIN: f32 = 0.1; // LITERAL-PX-OK: Beer–Lambert optical-depth min (parameter domain)
const DEPTH_MAX: f32 = 8.0; // LITERAL-PX-OK: Beer–Lambert optical-depth max (parameter domain)
const WARP_MAX: f32 = 48.0; // LITERAL-PX-OK: watercolor Ragged-Edge displacement max (px; range pair of SPREAD_MAX)
const DRY_TIME_MIN: f32 = 2.0; // LITERAL-PX-OK: Drying-Time slider min (seconds; matches DRY_TIME_MIN_S clamp)
const DRY_TIME_MAX: f32 = 60.0; // LITERAL-PX-OK: Drying-Time slider max (seconds; matches DRY_TIME_MAX_S clamp)

/// Paint the Watercolor section starting at `y`, returning the next `y`.
pub(crate) fn paint_watercolor_section(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    brush: BrushSettings,
) -> f32 {
    let (mut y, fold) = crate::paint_brush_top::paint_collapsible_section(
        ctx,
        theme,
        x,
        content_w,
        y,
        tr("panel.painter_layers.watercolor.watercolor"),
        ph2d_tool_painter::ids::PAINTER_WATERCOLOR_SECTION,
        ph2d_tool_painter::ids::PAINTER_WATERCOLOR_SECTION_COLOR,
        ph2d_tool_painter::ids::PAINTER_WATERCOLOR_RESET,
    );
    let Some(fold) = fold else {
        return y;
    };

    // ⚠️ The master enable is the **Paint Mode** dropdown at the head of the appearance half
    // (2026-07-22): this section is painted only while Watercolor is the selected medium, so a checkbox
    // here would be a second door to the same fact. The guard stays as the belt to that braces —
    // `paint_media` derives the medium from this flag, so they cannot disagree.
    if !brush.watercolor {
        return crate::paint_brush_top::end_fold(ctx, fold, y);
    }

    // Three technique cards — each paints its rows off the snapshot and returns
    // the next `y`. Split out to keep this fn under the panel LOC cap.
    y = paint_wash_card(ctx, theme, x, content_w, y, &brush);
    y = paint_brush_card(ctx, theme, x, content_w, y, &brush);
    y = paint_water_card(ctx, theme, x, content_w, y, &brush);
    y = paint_wetness_card(ctx, theme, x, content_w, y, &brush);

    let out = y;
    crate::paint_brush_top::end_fold(ctx, fold, out)
}

/// Card 4: WETNESS — canvas-level moisture controls (doc 13 #9-#11), NOT brush params: the
/// **Drying Time** slider (how long the paper stays mergeable) + **Dry** (end the wet session now,
/// the bake becomes permanent) / **Wet** (re-moisten the canvas so strokes made now fuse). The
/// slider reads `brush.dry_time_s` (carried in the display snapshot; it maps to the canvas rate).
fn paint_wetness_card(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    brush: &BrushSettings,
) -> f32 {
    let (ix, iw, mut ry, next_y) = card_frame(
        ctx,
        theme,
        x,
        content_w,
        y,
        tr("panel.painter_layers.watercolor.wetness"),
        3,
    );
    ry = card_row(
        ctx,
        theme,
        ix,
        iw,
        ry,
        "panel.painter_layers.watercolor.drying_time",
        ph2d_tool_painter::ids::PAINTER_WATERCOLOR_DRY_TIME,
        brush.dry_time_s,
        DRY_TIME_MIN,
        DRY_TIME_MAX,
        1.0, // whole-second scrub step
        0,   // whole seconds
    );
    ry = card_row(
        ctx,
        theme,
        ix,
        iw,
        ry,
        "panel.painter_layers.watercolor.preview",
        ph2d_tool_painter::ids::PAINTER_WATERCOLOR_WET_PREVIEW,
        brush.wet_preview,
        0.0,
        1.0,
        number_field::FINE_STEP,
        2,
    );
    let _ = wetness_button_row(
        ctx,
        theme,
        ix,
        iw,
        ry,
        &[
            (
                ph2d_tool_painter::ids::PAINTER_WATERCOLOR_DRY_NOW,
                tr("panel.painter_layers.watercolor.dry"),
            ),
            (
                ph2d_tool_painter::ids::PAINTER_WATERCOLOR_WET_NOW,
                tr("panel.painter_layers.watercolor.wet"),
            ),
        ],
    );
    next_y
}

/// A row of momentary action buttons (none selected) — the Dry/Wet canvas actions. Mirrors
/// `paint_deform`'s `seg_group` (a `SegmentedAdaptive` reused as a button row); each option forwards
/// a plain `Click` via the `PAINTER_WATERCOLOR_CLICKS` membership in the panel's `event.rs`.
fn wetness_button_row(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    options: &[(ph2d_a11y::NodeId, &str)],
) -> f32 {
    let opts: Vec<SegmentedOption> = options
        .iter()
        .map(|(id, label)| SegmentedOption::new(*id, *label))
        .collect();
    let seg = SegmentedAdaptive::new(
        ph2d_a11y::NodeId(0),
        tr("panel.painter_layers.watercolor.wetness_actions"),
        opts,
    )
    .selected(usize::MAX);
    let scene = &mut *ctx.scene;
    let text_system = &mut *ctx.text_system;
    let (store, hit_index) = ctx.host.store_and_hit_index_mut();
    let used = paint_segmented_adaptive(
        &seg,
        Rect::new(x, y, content_w, ROW_H_PX),
        scene,
        text_system,
        theme,
        store,
        hit_index,
    );
    y + used + ph2d_tokens::control_gap_px()
}

/// Card 1: WASH — how the stroke dries (the flat glaze + its dried character).
fn paint_wash_card(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    brush: &BrushSettings,
) -> f32 {
    let (ix, iw, mut ry, next_y) = card_frame(
        ctx,
        theme,
        x,
        content_w,
        y,
        tr("panel.painter_layers.watercolor.wash"),
        7,
    );
    ry = card_row(
        ctx,
        theme,
        ix,
        iw,
        ry,
        "panel.painter_layers.watercolor.body",
        ph2d_tool_painter::ids::PAINTER_WATERCOLOR_FILL,
        brush.fill,
        0.0,
        1.0,
        number_field::FINE_STEP,
        2,
    );
    ry = card_row(
        ctx,
        theme,
        ix,
        iw,
        ry,
        "panel.painter_layers.watercolor.concentration",
        ph2d_tool_painter::ids::PAINTER_WATERCOLOR_DEPTH,
        brush.depth,
        DEPTH_MIN,
        DEPTH_MAX,
        number_field::FINE_STEP,
        2,
    );
    ry = card_row(
        ctx,
        theme,
        ix,
        iw,
        ry,
        "panel.painter_layers.watercolor.opacity",
        ph2d_tool_painter::ids::PAINTER_WATERCOLOR_OPACITY,
        brush.opacity,
        0.0,
        1.0,
        number_field::FINE_STEP,
        2,
    );
    ry = card_row(
        ctx,
        theme,
        ix,
        iw,
        ry,
        "panel.painter_layers.watercolor.edge_darkening",
        ph2d_tool_painter::ids::PAINTER_WATERCOLOR_EDGE,
        brush.edge_gain,
        0.0,
        EDGE_MAX,
        number_field::FINE_STEP,
        2,
    );
    ry = card_row(
        ctx,
        theme,
        ix,
        iw,
        ry,
        "panel.painter_layers.watercolor.bleed",
        ph2d_tool_painter::ids::PAINTER_WATERCOLOR_SPREAD,
        brush.edge_spread,
        SPREAD_MIN,
        SPREAD_MAX,
        number_field::SIZE_STEP,
        1,
    );
    ry = card_row(
        ctx,
        theme,
        ix,
        iw,
        ry,
        "panel.painter_layers.watercolor.ragged_edge",
        ph2d_tool_painter::ids::PAINTER_WATERCOLOR_WARP,
        brush.warp,
        0.0,
        WARP_MAX,
        number_field::SIZE_STEP,
        1,
    );
    // Smooth Edges (BUGS #16): screen-space AA of the silhouette — the default look; off restores
    // the pre-AA hard/serrated edge as a deliberate style.
    let _ = crate::paint_brush_top::paint_checkbox_row(
        ctx,
        theme,
        ix,
        iw,
        ry,
        ph2d_tool_painter::ids::PAINTER_WATERCOLOR_SMOOTH_EDGES,
        "panel.painter_layers.watercolor.smooth_edges",
        brush.smooth_edges,
    );
    next_y
}

/// Card 2: BRUSH — what's on the brush (the Wet Mix reservoir: pickup, water, carry).
fn paint_brush_card(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    brush: &BrushSettings,
) -> f32 {
    let (ix, iw, mut ry, next_y) = card_frame(
        ctx,
        theme,
        x,
        content_w,
        y,
        tr("panel.painter_layers.watercolor.brush"),
        // ⚠️ TRÊS linhas: Charge · Dilution · Pull. O `card_frame` dimensiona a moldura por este
        //    número — uma linha a mais do que ele diz é pintada FORA do cartão, e nada no desenho
        //    reclama. O gate `o_numero_de_linhas_que_um_cartao_declara_e_o_que_ele_pinta` é quem o
        //    apanha (report do dono, 2026-09-20).
        3,
    );
    ry = card_row(
        ctx,
        theme,
        ix,
        iw,
        ry,
        "panel.painter_layers.watercolor.charge",
        ph2d_tool_painter::ids::PAINTER_WATERCOLOR_CHARGE,
        brush.wet_charge,
        0.0,
        1.0,
        number_field::FINE_STEP,
        2,
    );
    ry = card_row(
        ctx,
        theme,
        ix,
        iw,
        ry,
        "panel.painter_layers.watercolor.dilution",
        ph2d_tool_painter::ids::PAINTER_WATERCOLOR_DILUTION,
        brush.wet_dilution,
        0.0,
        1.0,
        number_field::FINE_STEP,
        2,
    );
    // ⛔⛔ **QUEM ACRESCENTAR UMA LINHA AQUI TEM DE TROCAR ESTE `let _ =` POR `ry =`, E SUBIR O
    //    `n_rows` ACIMA.** O `y` de retorno de uma `card_row` é o `y` da linha SEGUINTE, logo
    //    deitá-lo fora só é honesto na ÚLTIMA — e em 2026-09-20 uma linha nova (`Self Pickup`,
    //    entretanto retirada por ordem do dono) foi pintada EM CIMA desta, porque este `let _ =`
    //    continuou a ser o que era quando o `Pull` fechava o cartão. *Uma linha correcta virou
    //    defeito sem ser tocada.*
    let _ = card_row(
        ctx,
        theme,
        ix,
        iw,
        ry,
        "panel.painter_layers.watercolor.pull",
        ph2d_tool_painter::ids::PAINTER_WATERCOLOR_PULL,
        brush.wet_pull,
        0.0,
        1.0,
        number_field::FINE_STEP,
        2,
    );
    next_y
}

/// Card 3: WATER — interaction with paint already on the canvas (rewet / smear / subtractive mix).
fn paint_water_card(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    brush: &BrushSettings,
) -> f32 {
    // ⭐⭐⭐ **A 3.ª fileira (o `Pigment`) SAIU deste cartão em 2026-09-20, e quem a mandou sair foi
    //    um CENSO.** A mistura subtractiva deixou de ser da aguada (ordem do dono, *«ligue o
    //    digital»*) e passou a ter de desaparecer nos gestos que não depositam cor (report dele,
    //    *«confira se funciona para Blur e Smear»* — não funciona). A 1.ª redacção derivou o
    //    `n_rows` daquela condição, e o `o_numero_de_linhas_que_um_cartao_declara_e_o_que_ele_pinta`
    //    reprovou em voz alta: *«o `n_rows` não é um literal»*.
    //
    //    ⚠️ **Ele tinha razão, e a cura barata era cegá-lo.** Um cartão de altura variável não é
    //    verificável por aquela régua, e ensiná-la a ler um `if` tornaria-a um parser — a mesma
    //    régua que existe porque um cartão dimensionado para `3` com `4` dentro já shipou uma vez.
    //    ⇒ a fileira mudou-se para o cartão **`Mixing`** ([`crate::paint_pigment`]), que é UM
    //    hospedeiro para os três meios e some inteiro quando não é oferecido. *Este cartão volta a
    //    ser o que o nome dele diz — o que o traço faz com a ÁGUA — e a declarar um literal.*
    let (ix, iw, mut ry, next_y) = card_frame(
        ctx,
        theme,
        x,
        content_w,
        y,
        tr("panel.painter_layers.watercolor.water"),
        2,
    );
    ry = card_row(
        ctx,
        theme,
        ix,
        iw,
        ry,
        "panel.painter_layers.watercolor.rewet",
        ph2d_tool_painter::ids::PAINTER_WATERCOLOR_WET,
        brush.wet_rewet,
        0.0,
        1.0,
        number_field::FINE_STEP,
        2,
    );
    ry = card_row(
        ctx,
        theme,
        ix,
        iw,
        ry,
        "panel.painter_layers.watercolor.smudge",
        ph2d_tool_painter::ids::PAINTER_WATERCOLOR_SMUDGE,
        brush.wet_smudge,
        0.0,
        1.0,
        number_field::FINE_STEP,
        2,
    );
    let _ = ry; // a 3.ª fileira saiu deste cartão (ver o bloco do `card_frame` acima)
    next_y
}
