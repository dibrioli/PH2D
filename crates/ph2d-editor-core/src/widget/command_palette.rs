//! **Command palette** — a full-screen, centred modal that lists a large set of choices grouped by
//! coloured category (Motion Nodes' "Add Node", and reusable by any future browse-everything picker).
//!
//! This widget is **generic**: it knows only [`PaletteModel`] — groups of items, each item carrying a
//! `label` and an opaque `id` ([`NodeId`]). It does NOT know what an item *means*; the shell that
//! opened it maps the picked `id` back to a real action (mirroring how the [`super::blender_color_picker`]
//! is generic and the shell routes the read-back pick). So editor-core stays feature-agnostic.
//!
//! **Three layers, one paint.** A full-viewport **scrim** dims the app; a centred **card** holds the
//! header + grouped grid; and the click-through is the [`super::blender_color_picker`]'s trick —
//! the scrim registers a full-rect hit FIRST (so it loses the back-to-front walk to everything painted
//! after it), the card registers next (so its dead space beats the scrim → a click *inside* the card is
//! a no-op), and the close-X + item pills register LAST (so they win inside the card). A click on the
//! dimmed area outside the card therefore hits only the scrim → close.
//!
//! The paint is a pure function of the model + viewport; the chrome handler ([`crate::screens::hero::chrome`])
//! gates it on the store's open-state and routes its [`apply`]-side events. Nothing here mutates state.

use crate::interaction::HitIndex;
use crate::paint::{fill_rounded_rect, paint_text, resolve, stroke_rounded_rect};
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, Theme, TypeToken};
use ph2d_tool_registry::hash_node_id;
use ph2d_vector::{Color, VectorScene};

/// The close-X in the header.
pub const CMD_PALETTE_CLOSE: NodeId = hash_node_id("command_palette.close");
/// The full-viewport scrim barrier — a click here (outside the card) closes the palette.
pub const CMD_PALETTE_SCRIM: NodeId = hash_node_id("command_palette.scrim");
/// The card body — a click on the card's dead space is a no-op (consumed, so it never falls through
/// the scrim to close). Registered after the scrim, before the item pills.
pub const CMD_PALETTE_CARD: NodeId = hash_node_id("command_palette.card");

// ── Layout literals (LITERAL-PX-OK: modal geometry, not a measurement). ──
const CARD_MAX_W: f32 = 1160.0; // LITERAL-PX-OK: command palette max card width
const EDGE_MARGIN: f32 = 40.0; // LITERAL-PX-OK: min gap from the viewport edge to the card
const CARD_MAX_H_FRAC: f32 = 0.86; // LITERAL-PX-OK: card height fraction of the viewport
const HEADER_H: f32 = 44.0; // LITERAL-PX-OK: the "Add Node" + search + close band height
const CLOSE_W: f32 = 24.0; // LITERAL-PX-OK: close-X square
const MIN_COL_W: f32 = 200.0; // LITERAL-PX-OK: narrow column min width (below this, fewer columns)
const MAX_COLS: usize = 4; // LITERAL-PX-OK: CONTAGEM de colunas estreitas, nao medida
// A category with this many items reads as a pile-up when stacked in ONE narrow column, so it is
// laid as a full-width band instead (flows wide, stays short). Motion's catalog splits cleanly:
// the small categories cap at 15 items, the two big ones (Transform/Utility) have 39/41 — any
// threshold in [16, 38] is equivalent. This is a LAYOUT choice (readability), not a resource cap.
const MIN_WIDE_ITEMS: usize = 20; // LITERAL-PX-OK: CONTAGEM que promove uma categoria a faixa, nao medida
const COL_GAP: f32 = 20.0; // LITERAL-PX-OK: gap between masonry columns
const SECT_GAP: f32 = 18.0; // LITERAL-PX-OK: vertical gap between category sections in a column
const RULE_H: f32 = 2.0; // LITERAL-PX-OK: the coloured underline under a category header
const PILL_H: f32 = 26.0; // LITERAL-PX-OK: node pill height
const PILL_GAP: f32 = 6.0; // LITERAL-PX-OK: gap between pills
const PILL_PAD_X: f32 = 10.0; // LITERAL-PX-OK: pill horizontal padding around the dot + label
const DOT_R: f32 = 4.0; // LITERAL-PX-OK: the category-colour dot radius on a pill / header

/// One selectable choice. `id` is opaque to this widget — the shell that built the model maps it back
/// to a real action. `label` is the already-localised, English display string (HR-15).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PaletteItem {
    pub label: String,
    pub id: NodeId,
}

/// A named cluster inside a category (e.g. Transform's "Forces & Physics"). `title == None` means the
/// items hang directly under the category header with no sub-header (the common case).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PaletteSub {
    pub title: Option<String>,
    pub items: Vec<PaletteItem>,
}

/// A coloured category. `color` is the header tint (the `node-cat-*` tokens for Motion); the dot on
/// every item in the group wears it too, so the colour *teaches the library map* (plan §2.4).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PaletteGroup {
    pub title: String,
    pub color: ColorToken,
    pub subs: Vec<PaletteSub>,
}

/// The whole palette: a title + the coloured category groups. Held by the [`crate::interaction::WidgetStore`]
/// while open (set once on open, mirroring `open_onion_modal`'s value-seeding — never rebuilt per frame).
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct PaletteModel {
    pub title: String,
    pub groups: Vec<PaletteGroup>,
}

impl PaletteModel {
    /// Total item count (for the header's "N nodes" readout).
    #[must_use]
    pub fn item_count(&self) -> usize {
        self.groups
            .iter()
            .flat_map(|g| &g.subs)
            .map(|s| s.items.len())
            .sum()
    }

    /// Is `id` one of the palette's item ids? The chrome handler asks this to decide whether a click
    /// is a pick (vs the close-X, the scrim, or an unrelated widget) — the model is the single source,
    /// so there is no second list of ids to drift.
    #[must_use]
    pub fn is_item(&self, id: NodeId) -> bool {
        self.groups
            .iter()
            .flat_map(|g| &g.subs)
            .flat_map(|s| &s.items)
            .any(|it| it.id == id)
    }
}

// ── Internal placement: a group laid out at column-local coordinates (origin at the group's top-left),
//    plus its total height so the masonry can pick the shortest column before painting. ──
enum Placed {
    /// The category header: dot + title + "· N" count + a coloured underline rule.
    Header { title: String, count: usize, y: f32 },
    /// A sub-cluster header (grey, small).
    Sub { title: String, y: f32 },
    /// A node pill: its rect (column-local) + the item behind it.
    Pill {
        rect: Rect,
        id: NodeId,
        label: String,
    },
}

struct GroupLayout {
    color: ColorToken,
    placed: Vec<Placed>,
    height: f32,
}

/// Lay one group out into a column of width `col_w`, measuring pill widths from their labels. Pills wrap
/// to a new row when they would overflow the column. Returns the placement (column-local) + total height.
fn layout_group(ts: &mut TextSystem, group: &PaletteGroup, col_w: f32) -> GroupLayout {
    let font = TypeToken::Sm.px();
    let sub_font = TypeToken::Xs.px();
    let mut placed = Vec::new();
    let mut y = 0.0_f32;

    let count: usize = group.subs.iter().map(|s| s.items.len()).sum();
    placed.push(Placed::Header {
        title: group.title.clone(),
        count,
        y,
    });
    y += HEADER_H;

    for sub in &group.subs {
        if let Some(t) = &sub.title {
            placed.push(Placed::Sub {
                title: t.clone(),
                y,
            });
            y += sub_font + Spacing::Xs.px();
        }
        // Wrap pills across rows within the column.
        let mut x = 0.0_f32;
        let mut row_started = false;
        for it in &sub.items {
            let text_w = ts.prefix_width(&it.label, font);
            let pill_w = (DOT_R * 2.0 + Spacing::Xs.px() + text_w + PILL_PAD_X * 2.0).min(col_w);
            if row_started && x + pill_w > col_w {
                // wrap
                x = 0.0;
                y += PILL_H + PILL_GAP;
            }
            placed.push(Placed::Pill {
                rect: Rect::new(x, y, pill_w, PILL_H),
                id: it.id,
                label: it.label.clone(),
            });
            x += pill_w + PILL_GAP;
            row_started = true;
        }
        if row_started {
            y += PILL_H + PILL_GAP;
        }
    }
    GroupLayout {
        color: group.color,
        placed,
        height: y,
    }
}

/// Paint the palette over the whole `viewport` (a no-op decision is the chrome handler's; by the time we
/// are called the palette is open). Registers hit rects for the scrim (first, so it loses), the card
/// (next), and the close-X + every item pill (last, so they win inside the card).
pub fn paint(
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    model: &PaletteModel,
    viewport: Rect,
) {
    // ── Scrim: dim the whole app with the modal-backdrop token (heavy alpha). ──
    fill_rounded_rect(scene, viewport, 0.0, resolve(ColorToken::BgScrim, theme));
    hit_index.register(CMD_PALETTE_SCRIM, viewport);

    // ── Card: centred, capped to the viewport. ──
    let card_w = CARD_MAX_W
        .min(viewport.w - EDGE_MARGIN * 2.0)
        .max(MIN_COL_W);
    let card_h = (viewport.h * CARD_MAX_H_FRAC).min(viewport.h - EDGE_MARGIN * 2.0);
    let card_x = viewport.x + (viewport.w - card_w) * 0.5;
    let card_y = viewport.y + (viewport.h - card_h) * 0.5;
    let card = Rect::new(card_x, card_y, card_w, card_h);
    let radius = Radius::Lg.px();
    fill_rounded_rect(scene, card, radius, resolve(ColorToken::BgElev, theme));
    stroke_rounded_rect(scene, card, radius, 1.0, resolve(ColorToken::Border, theme));
    hit_index.register(CMD_PALETTE_CARD, card);

    let pad = Spacing::Md.px();
    let inner_x = card.x + pad;
    let inner_w = card.w - pad * 2.0;
    let font = TypeToken::Md.px();

    // ── Header band: title left, count centre-right, close-X right. ──
    let header_y = card.y + pad;
    paint_text(
        ts,
        scene,
        &model.title,
        inner_x,
        header_y + (HEADER_H - font) * 0.5,
        font,
        inner_w * 0.5,
        resolve(ColorToken::Text1, theme),
    );
    let count_str = format!("{} nodes", model.item_count());
    let count_w = ts.prefix_width(&count_str, TypeToken::Sm.px());
    let close_x = card.x + card.w - CLOSE_W - pad;
    paint_text(
        ts,
        scene,
        &count_str,
        close_x - count_w - Spacing::Sm.px(),
        header_y + (HEADER_H - TypeToken::Sm.px()) * 0.5,
        TypeToken::Sm.px(),
        count_w,
        resolve(ColorToken::Text2, theme),
    );
    let close_rect = Rect::new(
        close_x,
        header_y + (HEADER_H - CLOSE_W) * 0.5,
        CLOSE_W,
        CLOSE_W,
    );
    paint_text(
        ts,
        scene,
        "X",
        close_rect.x + CLOSE_W * 0.3,
        close_rect.y + (CLOSE_W - font) * 0.5,
        font,
        CLOSE_W,
        resolve(ColorToken::Text1, theme),
    );

    // ── Content: two regions, so the two big categories cannot pile up in one column and the small
    //    ones cannot leave gaps. SMALL categories pack into N balanced narrow columns (greedy
    //    shortest-column); BIG categories (>= MIN_WIDE_ITEMS) are full-width bands BELOW, where a
    //    40-node category is a few short rows across the whole card instead of a tall pile. This is
    //    the approved mockup's shape (narrow A/B columns + a wide Transform + a full-width Utility
    //    strip) derived from category SIZE, so a new/renamed category never breaks a hardcoded map. ──
    let content_x = inner_x;
    let content_y = header_y + HEADER_H + Spacing::Sm.px();
    let content_w = inner_w;

    let n_cols = ((content_w / MIN_COL_W).floor() as usize).clamp(1, MAX_COLS);
    let col_w = (content_w - COL_GAP * (n_cols as f32 - 1.0)) / n_cols as f32;

    // Small categories → balanced narrow columns, in display order (so Source leads at the
    // top-left, the pipeline reading order).
    let mut col_bottom = vec![content_y; n_cols];
    for g in model.groups.iter().filter(|g| group_count(g) < MIN_WIDE_ITEMS) {
        let gl = layout_group(ts, g, col_w);
        let c = shortest_column(&col_bottom);
        let ox = content_x + c as f32 * (col_w + COL_GAP);
        let oy = col_bottom[c];
        paint_group(scene, ts, theme, hit_index, &gl, ox, oy, col_w);
        col_bottom[c] = oy + gl.height + SECT_GAP;
    }

    // Big categories → full-width bands stacked below the narrow columns.
    let mut band_y = col_bottom.iter().copied().fold(content_y, f32::max);
    for g in model.groups.iter().filter(|g| group_count(g) >= MIN_WIDE_ITEMS) {
        let gl = layout_group(ts, g, content_w);
        paint_group(scene, ts, theme, hit_index, &gl, content_x, band_y, content_w);
        band_y += gl.height + SECT_GAP;
    }

    // Close-X last (wins its rect inside the card).
    hit_index.register(CMD_PALETTE_CLOSE, close_rect);
}

/// Item count of a group (decides narrow-column vs full-width-band placement in [`paint`]).
fn group_count(g: &PaletteGroup) -> usize {
    g.subs.iter().map(|s| s.items.len()).sum()
}

fn shortest_column(bottoms: &[f32]) -> usize {
    let mut best = 0;
    for (i, b) in bottoms.iter().enumerate() {
        if *b < bottoms[best] {
            best = i;
        }
    }
    best
}

#[allow(clippy::too_many_arguments)]
fn paint_group(
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    gl: &GroupLayout,
    ox: f32,
    oy: f32,
    col_w: f32,
) {
    let cat_color = resolve(gl.color, theme);
    let sm = TypeToken::Sm.px();
    let xs = TypeToken::Xs.px();
    for p in &gl.placed {
        match p {
            Placed::Header { title, count, y } => {
                let hy = oy + *y;
                dot(scene, ox + DOT_R, hy + sm * 0.5, cat_color);
                let label = format!("{title} \u{00b7} {count}");
                paint_text(
                    ts,
                    scene,
                    &label,
                    ox + DOT_R * 2.0 + Spacing::Xs.px(),
                    hy,
                    sm,
                    col_w,
                    cat_color,
                );
                // Coloured underline rule across the column.
                fill_rounded_rect(
                    scene,
                    Rect::new(ox, hy + sm + Spacing::Xs.px(), col_w, RULE_H),
                    RULE_H * 0.5,
                    cat_color,
                );
            }
            Placed::Sub { title, y } => {
                paint_text(
                    ts,
                    scene,
                    title,
                    ox,
                    oy + *y,
                    xs,
                    col_w,
                    resolve(ColorToken::Text2, theme),
                );
            }
            Placed::Pill { rect, id, label } => {
                let r = Rect::new(ox + rect.x, oy + rect.y, rect.w, rect.h);
                fill_rounded_rect(scene, r, Radius::Sm.px(), resolve(ColorToken::Bg3, theme));
                dot(scene, r.x + PILL_PAD_X + DOT_R, r.y + r.h * 0.5, cat_color);
                paint_text(
                    ts,
                    scene,
                    label,
                    r.x + PILL_PAD_X + DOT_R * 2.0 + Spacing::Xs.px(),
                    r.y + (r.h - xs) * 0.5,
                    xs,
                    r.w - PILL_PAD_X * 2.0 - DOT_R * 2.0,
                    resolve(ColorToken::Text1, theme),
                );
                hit_index.register(*id, r);
            }
        }
    }
}

/// A small filled category dot centred at `(cx, cy)`.
fn dot(scene: &mut VectorScene, cx: f32, cy: f32, color: Color) {
    fill_rounded_rect(
        scene,
        Rect::new(cx - DOT_R, cy - DOT_R, DOT_R * 2.0, DOT_R * 2.0),
        DOT_R,
        color,
    );
}
