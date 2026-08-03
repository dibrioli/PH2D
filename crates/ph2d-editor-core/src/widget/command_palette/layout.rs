//! **Quanto mede, e onde cada coisa cai** — a metade GEOMÉTRICA da paleta, irmã de
//! [`super`] pelo teto de 500 LOC dos primitivos de widget.
//!
//! O corte é por responsabilidade, e ele já estava escrito no desenho do widget: medir uma
//! categoria (quantas pílulas cabem numa largura, que altura o cartão pede) é uma pergunta
//! **pura sobre texto e espaço**, respondida antes de um pixel existir; pintar é o que se faz
//! com a resposta. Manter as duas juntas foi o que levou o arquivo a 661.
//!
//! Nada aqui desenha. Nada aqui registra hit. As consts de medida moram no pai — um filho
//! enxerga os privados do pai, então o corte não move visibilidade nenhuma.

use super::*;

/// Abaixo de três colunas o arranjo não é uma grade — ele empilha.
const MIN_COLS_FOR_GRID: f32 = 3.0; // LITERAL-PX-OK: CONTAGEM de colunas, não medida

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

/// Lay the category cards out (MEASURE only — no painting), in card-content-LOCAL y with the top at 0,
/// like the approved mockup: small categories stack in two narrow columns on the left; the first BIG
/// category (>= [`MIN_WIDE_ITEMS`], in display order) is a wide card on the right (its sub-clusters grid
/// inside it); any further big categories are full-width cards below. Derived from category SIZE +
/// display order, so a new/renamed category never breaks a hardcoded map. A narrow window (< 3 columns)
/// falls back to a single stacked column. Returns each placed card `(layout, ox, oy, width)` and the
/// total content height (so [`paint`] can size the card to it, leaving no dead space below).
pub(super) fn arrange(
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
    if content_w >= MIN_COL_W * MIN_COLS_FOR_GRID && !big.is_empty() {
        // 4 units wide: [narrow][narrow][wide = 2 units]. Small cards fill the two narrow columns;
        // the first big card takes the wide slot on the right.
        let unit_w = (content_w - COL_GAP * 3.0) / 4.0; // LITERAL-PX-OK: CONTAGENS da grade de 4 unidades (3 vãos)
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
