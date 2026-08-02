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
const MIN_COL_W: f32 = 200.0; // LITERAL-PX-OK: min narrow-column width (below 3x this: stacked fallback)
// A category with this many items gets the wide card + gridded sub-clusters (Transform/Utility),
// instead of stacking in a narrow column. Motion's catalog splits cleanly:
// the small categories cap at 15 items, the two big ones (Transform/Utility) have 39/41 — any
// threshold in [16, 38] is equivalent. This is a LAYOUT choice (readability), not a resource cap.
const MIN_WIDE_ITEMS: usize = 20; // LITERAL-PX-OK: CONTAGEM que promove uma categoria a 2 colunas, nao medida
const COL_GAP: f32 = 16.0; // LITERAL-PX-OK: gap between category cards (columns and rows)
const SECT_GAP: f32 = 12.0; // LITERAL-PX-OK: vertical gap between stacked cards in a column
const CARD_PAD: f32 = 12.0; // LITERAL-PX-OK: inner padding of a category card
const SUB_MIN_W: f32 = 210.0; // LITERAL-PX-OK: min sub-column width inside a card (below this, fewer)
const SUB_GAP: f32 = 16.0; // LITERAL-PX-OK: gap between sub-clusters inside a card
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

/// Does `label` match the live search `query`? Case-insensitive substring. This is the SINGLE predicate
/// the palette's on-screen filter and its `Enter` top-match both call — so what the artist SEES filtered
/// and what `Enter` adds can never disagree. An empty query matches everything.
#[must_use]
pub fn item_matches(query: &str, label: &str) -> bool {
    query.is_empty() || label.to_lowercase().contains(&query.to_lowercase())
}

/// The first item (in display order) whose label matches `query`, or `None`. The shell calls this on
/// `Enter` to add the top result — through the same [`item_matches`] the filter paints with. An EMPTY
/// query returns `None` (Enter with no search is a no-op, never "add the first random node").
#[must_use]
pub fn top_match(model: &PaletteModel, query: &str) -> Option<NodeId> {
    if query.is_empty() {
        return None;
    }
    model
        .groups
        .iter()
        .flat_map(|g| &g.subs)
        .flat_map(|s| &s.items)
        .find(|it| item_matches(query, &it.label))
        .map(|it| it.id)
}

/// A copy of `model` keeping only items whose label matches `query`, dropping any sub-cluster or category
/// that ends up empty — so a filtered library never shows a coloured header with nothing under it. An
/// empty query is handled by the caller (it uses the model unfiltered); here every item still matches.
fn filter_model(model: &PaletteModel, query: &str) -> PaletteModel {
    let groups = model
        .groups
        .iter()
        .filter_map(|g| {
            let subs: Vec<PaletteSub> = g
                .subs
                .iter()
                .filter_map(|s| {
                    let items: Vec<PaletteItem> = s
                        .items
                        .iter()
                        .filter(|it| item_matches(query, &it.label))
                        .cloned()
                        .collect();
                    (!items.is_empty()).then(|| PaletteSub {
                        title: s.title.clone(),
                        items,
                    })
                })
                .collect();
            (!subs.is_empty()).then(|| PaletteGroup {
                title: g.title.clone(),
                color: g.color,
                subs,
            })
        })
        .collect();
    PaletteModel {
        title: model.title.clone(),
        groups,
    }
}

// ── Internal placement: a category card laid out at card-local coordinates (origin at the card's
//    top-left), plus its total height so the arrangement can stack cards before painting. ──
enum Placed {
    /// The category header: dot + title + "· N" count + a coloured underline rule (drawn at CARD_PAD).
    Header { title: String, count: usize, y: f32 },
    /// A sub-cluster header (grey, small), at a card-local (x, y).
    Sub { title: String, x: f32, y: f32 },
    /// A node pill: its card-local rect + the item behind it.
    Pill {
        rect: Rect,
        id: NodeId,
        label: String,
    },
}

struct CardLayout {
    color: ColorToken,
    placed: Vec<Placed>,
    height: f32,
}

/// Height of the category header block inside a card (name row + gap + coloured rule + gap).
fn card_head_h() -> f32 {
    TypeToken::Sm.px() + Spacing::Xs.px() + RULE_H + Spacing::Sm.px()
}

/// Append pill placements for `items`, wrapping to new rows at `width`, starting at card-local
/// `(ox, oy)`. Returns the y just below the last row (including the trailing pill gap).
fn flow_pills(
    placed: &mut Vec<Placed>,
    ts: &mut TextSystem,
    items: &[PaletteItem],
    ox: f32,
    oy: f32,
    width: f32,
    font: f32,
) -> f32 {
    let mut x = 0.0_f32;
    let mut y = oy;
    let mut row_started = false;
    for it in items {
        let text_w = ts.prefix_width(&it.label, font);
        let pill_w = (DOT_R * 2.0 + Spacing::Xs.px() + text_w + PILL_PAD_X * 2.0).min(width);
        if row_started && x + pill_w > width {
            x = 0.0;
            y += PILL_H + PILL_GAP;
        }
        placed.push(Placed::Pill {
            rect: Rect::new(ox + x, y, pill_w, PILL_H),
            id: it.id,
            label: it.label.clone(),
        });
        x += pill_w + PILL_GAP;
        row_started = true;
    }
    if row_started {
        y += PILL_H + PILL_GAP;
    }
    y
}

/// Lay one category out as a CARD of width `card_w`: header at the top, then pills — flowing directly
/// for a flat category, or as a masonry of sub-blocks across `inner_cols` sub-columns when the category
/// has named sub-clusters (so a big category reads as a tidy grid, not one tall list). All placements
/// are card-local (origin at the card's top-left).
fn layout_card(ts: &mut TextSystem, group: &PaletteGroup, card_w: f32) -> CardLayout {
    let font = TypeToken::Sm.px();
    let sub_font = TypeToken::Xs.px();
    let content_x = CARD_PAD;
    let content_w = (card_w - CARD_PAD * 2.0).max(PILL_H);

    let count: usize = group.subs.iter().map(|s| s.items.len()).sum();
    let mut placed = vec![Placed::Header {
        title: group.title.clone(),
        count,
        y: CARD_PAD,
    }];
    let top = CARD_PAD + card_head_h();

    let content_bottom = if group.subs.iter().any(|s| s.title.is_some()) {
        // Masonry the sub-clusters into sub-columns (widest a big category needs to stay short).
        let inner_cols = group
            .subs
            .len()
            .min(((content_w / SUB_MIN_W).floor() as usize).max(1));
        let sub_w = (content_w - SUB_GAP * (inner_cols as f32 - 1.0)) / inner_cols as f32;
        let mut sub_bottom = vec![top; inner_cols];
        for sub in &group.subs {
            let c = shortest_slot(&sub_bottom, 1);
            let ox = content_x + c as f32 * (sub_w + SUB_GAP);
            let mut sy = sub_bottom[c];
            if let Some(t) = &sub.title {
                placed.push(Placed::Sub {
                    title: t.clone(),
                    x: ox,
                    y: sy,
                });
                sy += sub_font + Spacing::Xs.px();
            }
            sy = flow_pills(&mut placed, ts, &sub.items, ox, sy, sub_w, font);
            sub_bottom[c] = sy + SUB_GAP;
        }
        sub_bottom.iter().copied().fold(top, f32::max) - SUB_GAP
    } else {
        let all: Vec<PaletteItem> = group
            .subs
            .iter()
            .flat_map(|s| s.items.iter().cloned())
            .collect();
        flow_pills(&mut placed, ts, &all, content_x, top, content_w, font) - PILL_GAP
    };

    CardLayout {
        color: group.color,
        placed,
        height: content_bottom + CARD_PAD,
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
    query: &str,
    viewport: Rect,
) {
    // ── The live search text filters the model (empty = show everything). Filtering keeps only the
    //    matching items and drops emptied sub-clusters / categories; `Enter` adds the same top match. ──
    let filtered;
    let model = if query.is_empty() {
        model
    } else {
        filtered = filter_model(model, query);
        &filtered
    };
    let no_match = model.groups.is_empty();

    // ── Card geometry independent of content height: width (centred) + the content box. ──
    let card_w = CARD_MAX_W
        .min(viewport.w - EDGE_MARGIN * 2.0)
        .max(MIN_COL_W);
    let card_x = viewport.x + (viewport.w - card_w) * 0.5;
    let pad = Spacing::Md.px();
    let content_x = card_x + pad;
    let content_w = card_w - pad * 2.0;
    let font = TypeToken::Md.px();

    // ── Measure the category-card arrangement (card-local y from 0), then size the card to it so there
    //    is no dead space below the last row — capped at CARD_MAX_H_FRAC of the viewport. ──
    let (placed, content_h) = arrange(ts, model, content_x, content_w);
    // With no matches the arrangement is empty (height 0); reserve a row for the "No matches" message so
    // the card doesn't collapse to just the header.
    let content_h = if no_match {
        TypeToken::Md.px() + Spacing::Lg.px()
    } else {
        content_h
    };
    let card_h = (pad * 2.0 + HEADER_H + Spacing::Sm.px() + content_h)
        .min(viewport.h * CARD_MAX_H_FRAC)
        .min(viewport.h - EDGE_MARGIN * 2.0);
    let card_y = viewport.y + (viewport.h - card_h) * 0.5;

    // ── Scrim: dim the whole app with the modal-backdrop token (heavy alpha). ──
    fill_rounded_rect(scene, viewport, 0.0, resolve(ColorToken::BgScrim, theme));
    hit_index.register(CMD_PALETTE_SCRIM, viewport);

    // ── Card: centred, sized to its content. ──
    let card = Rect::new(card_x, card_y, card_w, card_h);
    let radius = Radius::Lg.px();
    fill_rounded_rect(scene, card, radius, resolve(ColorToken::BgElev, theme));
    stroke_rounded_rect(scene, card, radius, 1.0, resolve(ColorToken::Border, theme));
    hit_index.register(CMD_PALETTE_CARD, card);

    // ── Header band: title left, search box in the middle, count centre-right, close-X right. ──
    let header_y = card_y + pad;
    let title_w = ts.prefix_width(&model.title, font).min(content_w * 0.35);
    paint_text(
        ts,
        scene,
        &model.title,
        content_x,
        header_y + (HEADER_H - font) * 0.5,
        font,
        title_w,
        resolve(ColorToken::Text1, theme),
    );
    let count_str = format!("{} nodes", model.item_count());
    let count_w = ts.prefix_width(&count_str, TypeToken::Sm.px());
    let close_x = card_x + card_w - CLOSE_W - pad;
    let count_x = close_x - count_w - Spacing::Sm.px();
    paint_text(
        ts,
        scene,
        &count_str,
        count_x,
        header_y + (HEADER_H - TypeToken::Sm.px()) * 0.5,
        TypeToken::Sm.px(),
        count_w,
        resolve(ColorToken::Text2, theme),
    );

    // ── Search box: a Bg3 rounded field showing the typed query (or a "Search" placeholder), the app's
    //    keyboard feeds it while the palette is open (a full-screen modal captures the keys). ──
    let sm = TypeToken::Sm.px();
    let search_h = (HEADER_H - Spacing::Sm.px()).max(PILL_H);
    let sb_x = content_x + title_w + Spacing::Md.px();
    let sb_w = (count_x - Spacing::Md.px() - sb_x).max(MIN_COL_W * 0.5);
    let sb = Rect::new(sb_x, header_y + (HEADER_H - search_h) * 0.5, sb_w, search_h);
    fill_rounded_rect(scene, sb, Radius::Sm.px(), resolve(ColorToken::Bg3, theme));
    stroke_rounded_rect(
        scene,
        sb,
        Radius::Sm.px(),
        1.0,
        resolve(ColorToken::Border, theme),
    );
    let (search_text, search_color) = if query.is_empty() {
        ("Search", resolve(ColorToken::Text2, theme))
    } else {
        (query, resolve(ColorToken::Text1, theme))
    };
    let text_y = sb.y + (sb.h - sm) * 0.5;
    paint_text(
        ts,
        scene,
        search_text,
        sb.x + Spacing::Sm.px(),
        text_y,
        sm,
        sb.w - Spacing::Sm.px() * 2.0,
        search_color,
    );
    // Caret at the end of the typed text (only while there IS a query — the placeholder needs none).
    if !query.is_empty() {
        let caret_x = (sb.x + Spacing::Sm.px() + ts.prefix_width(query, sm))
            .min(sb.x + sb.w - Spacing::Xs.px());
        fill_rounded_rect(
            scene,
            Rect::new(
                caret_x,
                sb.y + Spacing::Xs.px(),
                1.5,
                sb.h - Spacing::Xs.px() * 2.0,
            ), // LITERAL-PX-OK: 1.5px text caret
            0.0,
            resolve(ColorToken::Text1, theme),
        );
    }
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

    // ── Content: paint the measured category cards at the card's content origin — or a "No matches"
    //    message when the search filtered everything out. ──
    let content_y = header_y + HEADER_H + Spacing::Sm.px();
    if no_match {
        paint_text(
            ts,
            scene,
            "No matches",
            content_x,
            content_y,
            font,
            content_w,
            resolve(ColorToken::Text2, theme),
        );
    } else {
        for (cl, ox, oy, w) in &placed {
            paint_card(scene, ts, theme, hit_index, cl, *ox, content_y + *oy, *w);
        }
    }

    // Close-X last (wins its rect inside the card).
    hit_index.register(CMD_PALETTE_CLOSE, close_rect);
}

/// Lay the category cards out (MEASURE only — no painting), in card-content-LOCAL y with the top at 0,
/// like the approved mockup: small categories stack in two narrow columns on the left; the first BIG
/// category (>= [`MIN_WIDE_ITEMS`], in display order) is a wide card on the right (its sub-clusters grid
/// inside it); any further big categories are full-width cards below. Derived from category SIZE +
/// display order, so a new/renamed category never breaks a hardcoded map. A narrow window (< 3 columns)
/// falls back to a single stacked column. Returns each placed card `(layout, ox, oy, width)` and the
/// total content height (so [`paint`] can size the card to it, leaving no dead space below).
fn arrange(
    ts: &mut TextSystem,
    model: &PaletteModel,
    content_x: f32,
    content_w: f32,
) -> (Vec<(CardLayout, f32, f32, f32)>, f32) {
    let mut small: Vec<&PaletteGroup> = Vec::new();
    let mut big: Vec<&PaletteGroup> = Vec::new();
    for g in &model.groups {
        if group_count(g) >= MIN_WIDE_ITEMS {
            big.push(g);
        } else {
            small.push(g);
        }
    }

    let mut placed: Vec<(CardLayout, f32, f32, f32)> = Vec::new();
    if content_w >= MIN_COL_W * 3.0 && !big.is_empty() {
        // 4 units wide: [narrow][narrow][wide = 2 units]. Small cards fill the two narrow columns;
        // the first big card takes the wide slot on the right.
        let unit_w = (content_w - COL_GAP * 3.0) / 4.0;
        let narrow_w = unit_w;
        let wide_x = content_x + 2.0 * (unit_w + COL_GAP);
        let wide_w = 2.0 * unit_w + COL_GAP;

        let mut col_bottom = [0.0_f32, 0.0];
        for g in &small {
            let c = usize::from(col_bottom[1] < col_bottom[0]);
            let cx = content_x + c as f32 * (narrow_w + COL_GAP);
            let cl = layout_card(ts, g, narrow_w);
            let h = cl.height;
            placed.push((cl, cx, col_bottom[c], narrow_w));
            col_bottom[c] += h + SECT_GAP;
        }

        let mut col_c_bottom = 0.0_f32;
        if let Some(g) = big.first() {
            let cl = layout_card(ts, g, wide_w);
            let h = cl.height;
            placed.push((cl, wide_x, 0.0, wide_w));
            col_c_bottom = h + SECT_GAP;
        }

        // Any remaining big categories are full-width cards below the top region.
        let mut y = col_bottom[0].max(col_bottom[1]).max(col_c_bottom);
        for g in big.iter().skip(1) {
            let cl = layout_card(ts, g, content_w);
            let h = cl.height;
            placed.push((cl, content_x, y, content_w));
            y += h + SECT_GAP;
        }
    } else {
        // Fallback (narrow window / no big category): one stacked column of full-width cards.
        let mut y = 0.0_f32;
        for g in &model.groups {
            let cl = layout_card(ts, g, content_w);
            let h = cl.height;
            placed.push((cl, content_x, y, content_w));
            y += h + SECT_GAP;
        }
    }

    let content_h = placed
        .iter()
        .map(|(cl, _, oy, _)| oy + cl.height)
        .fold(0.0_f32, f32::max);
    (placed, content_h)
}

/// Item count of a group (decides the column span in [`paint`]: 1 column normally, 2 when big).
fn group_count(g: &PaletteGroup) -> usize {
    g.subs.iter().map(|s| s.items.len()).sum()
}

/// The starting column of the `span`-wide slot whose columns are currently shortest — where the
/// next category has the most room. Ties break to the leftmost slot.
fn shortest_slot(bottoms: &[f32], span: usize) -> usize {
    let mut best = 0;
    let mut best_h = f32::INFINITY;
    for c in 0..=bottoms.len().saturating_sub(span) {
        let h = bottoms[c..c + span].iter().copied().fold(0.0_f32, f32::max);
        if h < best_h {
            best_h = h;
            best = c;
        }
    }
    best
}

/// Paint one category card: a rounded `Bg2` box + `Border`, then its header (dot + name + count +
/// coloured rule), the grey sub-cluster headers, and the pills (each registering its hit rect).
#[allow(clippy::too_many_arguments)]
fn paint_card(
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    cl: &CardLayout,
    ox: f32,
    oy: f32,
    card_w: f32,
) {
    let radius = Radius::Md.px();
    let card = Rect::new(ox, oy, card_w, cl.height);
    fill_rounded_rect(scene, card, radius, resolve(ColorToken::Bg2, theme));
    stroke_rounded_rect(scene, card, radius, 1.0, resolve(ColorToken::Border, theme));

    let cat_color = resolve(cl.color, theme);
    let sm = TypeToken::Sm.px();
    let xs = TypeToken::Xs.px();
    let content_w = card_w - CARD_PAD * 2.0;
    for p in &cl.placed {
        match p {
            Placed::Header { title, count, y } => {
                let hx = ox + CARD_PAD;
                let hy = oy + *y;
                dot(scene, hx + DOT_R, hy + sm * 0.5, cat_color);
                let label = format!("{title} \u{00b7} {count}");
                paint_text(
                    ts,
                    scene,
                    &label,
                    hx + DOT_R * 2.0 + Spacing::Xs.px(),
                    hy,
                    sm,
                    content_w,
                    cat_color,
                );
                // Coloured underline rule across the card content width.
                fill_rounded_rect(
                    scene,
                    Rect::new(hx, hy + sm + Spacing::Xs.px(), content_w, RULE_H),
                    RULE_H * 0.5,
                    cat_color,
                );
            }
            Placed::Sub { title, x, y } => {
                paint_text(
                    ts,
                    scene,
                    title,
                    ox + *x,
                    oy + *y,
                    xs,
                    content_w,
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

// Gates for the search filter live in a `#[path]` sibling so they remain a CHILD module (reaching the
// private `filter_model`) while keeping this file under the LOC cap.
#[cfg(test)]
#[path = "command_palette_tests.rs"]
mod tests;
