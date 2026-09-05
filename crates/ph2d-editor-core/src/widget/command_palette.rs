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
use crate::paint::{fill_rounded_rect, paint_text, resolve};
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, Theme, TypeToken};
use ph2d_tool_registry::hash_node_id;
use ph2d_vector::{Color, VectorScene};

/// **Quanto mede, e onde cada coisa cai** — irmã pelo teto de 500 LOC dos primitivos; o corte é
/// por responsabilidade: aqui fica o que a paleta DESENHA, lá o que ela MEDE.
mod layout;
use layout::arrange;

/// **QUANDO cada cartão chega** — a 3ª responsabilidade (desenha · mede · entra).
mod cascade;
pub use cascade::cascade_id;

/// **Quanto o conteúdo mede e o que dele SE VÊ** — a geometria do cartão, a régua da roda e o
/// traço indicador. Irmão pelo teto de 500 LOC, e o corte é por responsabilidade: aqui fica o que
/// se desenha, lá o que cabe.
mod scroll;
pub use scroll::max_scroll;
use scroll::{Metrics, metrics, paint_scroll_hint};

/// **A BANDA do cabeçalho** — título, busca, contagem, a caixa *Show all* e o X. A 4ª
/// responsabilidade, e o irmão que a F3 (ADR-0166) obrigou a existir: a caixa levaria este ficheiro
/// para lá dos 500 LOC dos primitivos, e *ficar no mesmo sítio não é encolher*.
mod header;
pub use header::{CMD_PALETTE_SHOW_ALL, PaletteToggle};

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
    /// ⭐ **A caixa da banda** — `None` para quem não tem nenhuma (ADR-0166 / F3). Ver
    /// [`PaletteToggle`]: acrescentar o campo é um erro de compilação nos três construtores, e é
    /// isso que se quer — um campo novo que se preenchesse sozinho seria a feature a desaparecer
    /// em silêncio no consumidor que ninguém reviu.
    pub toggle: Option<PaletteToggle>,
}

impl PaletteModel {
    /// Total item count (for the header's "N items" readout).
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
        // ⚠️ A caixa ATRAVESSA o filtro: ela é da banda, não do conteúdo — filtrar por texto não a
        // pode desligar, senão escrever no campo de busca apagava o controlo que a mostra.
        toggle: model.toggle.clone(),
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

/// Paint the palette over the whole `viewport` (a no-op decision is the chrome handler's; by the time we
/// are called the palette is open). Registers hit rects for the scrim (first, so it loses), the card
/// (next), and the close-X + every item pill (last, so they win inside the card).
///
/// ⚠️ `scroll` é quanto a lista está rolada, em px — preso ao [`max_scroll`] **aqui dentro**, para
/// que um valor velho no store (o conteúdo encolheu com a busca) não mostre um cartão vazio.
#[allow(clippy::too_many_arguments)]
pub fn paint(
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    model: &PaletteModel,
    query: &str,
    viewport: Rect,
    motion: &crate::motion::UiMotion,
    scroll: f32,
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

    // ⭐ **A geometria vem da PORTA** — a mesma que a [`max_scroll`] usa. Duas contas para «que
    //    altura tem este cartão» seriam duas respostas, e a que decide se há rolagem seria a que
    //    envelhece: o artista via metade da lista e a roda não fazia nada.
    let Metrics {
        card,
        content_x,
        content_w,
        content_h,
        view_h,
        pad,
        font,
        placed,
    } = metrics(ts, model, viewport, no_match);
    let (card_x, card_y, card_w, card_h) = (card.x, card.y, card.w, card.h);

    // ── Scrim: dim the whole app with the modal-backdrop token (heavy alpha). ──
    fill_rounded_rect(scene, viewport, 0.0, resolve(ColorToken::BgScrim, theme));
    hit_index.register(CMD_PALETTE_SCRIM, viewport);

    // ── Card: centred, sized to its content. ──
    let card = Rect::new(card_x, card_y, card_w, card_h);
    let radius = crate::paint::frame_radius(theme, Radius::Lg.px());
    fill_rounded_rect(scene, card, radius, resolve(ColorToken::BgElev, theme));
    crate::paint::stroke_frame(
        scene,
        card,
        radius,
        theme,
        ph2d_tokens::visuals::Feel::Rest,
        1.0,
        resolve(ColorToken::Border, theme),
    );
    hit_index.register(CMD_PALETTE_CARD, card);

    // ── A banda do cabeçalho — título, busca, contagem, a caixa *Show all* e o X de fechar. Ela
    //    saiu para o irmão `header.rs` quando a caixa da F3 (ADR-0166) a levaria a passar o teto de
    //    500 LOC deste ficheiro: o corte é por responsabilidade — aqui fica o CARTÃO, lá a BANDA.
    let header_y = card_y + pad;
    let close_rect = header::paint_header(
        scene, ts, theme, hit_index, model, query, card_x, card_w, content_x, content_w, header_y,
        pad, font,
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
        // ⭐ **O CORPO É RECORTADO no CLIQUE e na PINTURA** (F3 / ADR-0166) — e são duas coisas
        //    diferentes: o `push_clip` do `HitIndex` decide quem RESPONDE; quem recorta pixels é a
        //    cena. Sem os dois, o conteúdo rolado desenha por cima do cabeçalho e por fora do
        //    cartão — que foi o report do Enio no 1.º smoke (a lista saía pela base do ecrã).
        //
        // ⚠️ **O `scroll` é preso AQUI**, e não onde a roda o escreve: a busca encolhe o conteúdo,
        //    e um valor velho mostraria um cartão vazio sem nada que explicasse como voltar.
        let scroll = crate::math::safe_clamp(scroll, 0.0, (content_h - view_h).max(0.0));
        let body = Rect::new(content_x, content_y, content_w, view_h);
        hit_index.push_clip(body);
        scene.push_clip(&ph2d_vector::Rect::new(
            f64::from(body.x),
            f64::from(body.y),
            f64::from(body.x + body.w),
            f64::from(body.y + body.h),
        ));
        // ⚠️ A CASCATA: o cartão `i` desenha-se subido por `cascade_rise(t)` e o hit regista na
        //    posição ASSENTE. É a mesma lei do `hover_lift` — o alvo que o dedo procura não pode
        //    estar noutro sítio do que o alvo que o olho vê —, e aqui ela morde mais forte, porque
        //    12 px de deslocamento poriam um clique apressado na row de cima.
        let travels = motion.travels();
        for (i, (cl, ox, oy, w)) in placed.iter().enumerate() {
            let t = motion.get(cascade_id(i)).unwrap_or(1.0);
            let settled_y = content_y + *oy - scroll;
            paint_card(
                scene,
                ts,
                theme,
                hit_index,
                cl,
                *ox,
                settled_y,
                *w,
                crate::motion::cascade_rise(t, travels),
            );
        }
        scene.pop_layer();
        hit_index.pop_clip();
        paint_scroll_hint(scene, theme, body, content_h, view_h, scroll);
    }

    // Close-X last (wins its rect inside the card).
    hit_index.register(CMD_PALETTE_CLOSE, close_rect);
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
    rise: f32,
) {
    // ⚠️ `oy` é a posição ASSENTE — a que o dedo procura — e `dy` a que o olho vê durante a
    //    entrada. Tudo o que DESENHA lê `dy`; o único `register` deste corpo lê `oy`.
    let dy = oy + rise;
    let radius = crate::paint::frame_radius(theme, Radius::Md.px());
    let card = Rect::new(ox, dy, card_w, cl.height);
    fill_rounded_rect(scene, card, radius, resolve(ColorToken::Bg2, theme));
    crate::paint::stroke_frame(
        scene,
        card,
        radius,
        theme,
        ph2d_tokens::visuals::Feel::Rest,
        1.0,
        resolve(ColorToken::Border, theme),
    );

    let cat_color = resolve(cl.color, theme);
    let sm = TypeToken::Sm.px();
    let xs = TypeToken::Xs.px();
    let content_w = card_w - CARD_PAD * 2.0;
    for p in &cl.placed {
        match p {
            Placed::Header { title, count, y } => {
                let hx = ox + CARD_PAD;
                let hy = dy + *y;
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
                    dy + *y,
                    xs,
                    content_w,
                    resolve(ColorToken::Text2, theme),
                );
            }
            Placed::Pill { rect, id, label } => {
                let r = Rect::new(ox + rect.x, dy + rect.y, rect.w, rect.h);
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
                // ⚠️ ASSENTE, nunca `r`: durante a entrada o cartão está 12 px abaixo, e um
                //    clique apressado cairia na row de cima.
                hit_index.register(*id, Rect::new(ox + rect.x, oy + rect.y, rect.w, rect.h));
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
mod tests;
