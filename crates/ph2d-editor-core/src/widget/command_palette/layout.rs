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
    // ⚠️ **A grade deixou de exigir uma categoria GRANDE** (F3 / ADR-0166), e a condição antiga
    //    (`&& !big.is_empty()`) não tinha razão escrita: com ela, um modelo de dez categorias
    //    médias caía na coluna ÚNICA de largura total — uma parede de itens muito mais alta que a
    //    grade, e transbordando por muito mais. O que a largura permite, a largura decide; o
    //    tamanho de uma categoria decide só quantos vãos ela ocupa.
    if content_w >= MIN_COL_W * MIN_COLS_FOR_GRID {
        // ⭐ **MASONRY sobre as quatro unidades** — cada categoria cai no vão mais CURTO, larga
        //    ocupa dois (F3 / ADR-0166).
        //
        // ⚠️ **A lei anterior era POSICIONAL** — *«as pequenas nas duas colunas da esquerda, a
        //    primeira grande no vão largo da direita»* — e ela não olhava para altura nenhuma. No
        //    catálogo de componentes com *Show all* (72 items) isso dava uma coluna a transbordar o
        //    ecrã enquanto o canto direito ficava **vazio** por baixo de uma categoria baixa e
        //    larga: foi a foto do Enio de 25/08.
        //
        // ⚠️ **O `shortest_slot` já existia, com o parâmetro `span` e tudo** — ele era usado só
        //    para os sub-clusters DENTRO de um cartão, e o arranjo de topo, um nível acima,
        //    ignorava-o. *A peça que falta pode já estar construída.*
        //
        // ⚠️ **A ordem de visita é a de EXIBIÇÃO, e as largas vão primeiro**: uma larga precisa de
        //    dois vãos adjacentes, e semeá-la depois de as estreitas encherem a grade obriga-a a
        //    um par desnivelado. É o idioma de todo empacotador — o item grande primeiro.
        let unit_w = (content_w - COL_GAP * 3.0) / 4.0; // LITERAL-PX-OK: CONTAGENS da grade de 4 unidades (3 vãos)
        let slot_x = |c: usize| content_x + c as f32 * (unit_w + COL_GAP);
        let span_w = |span: usize| unit_w * span as f32 + COL_GAP * (span as f32 - 1.0);
        let mut bottom = [0.0_f32; 4];
        for (g, span) in big
            .iter()
            .map(|g| (*g, 2usize))
            .chain(small.iter().map(|g| (*g, 1usize)))
        {
            let c = shortest_slot(&bottom, span);
            let top = bottom[c..c + span].iter().copied().fold(0.0_f32, f32::max);
            let w = span_w(span);
            let cl = layout_card(ts, g, w);
            let h = cl.height;
            placed.push((cl, slot_x(c), top, w));
            for b in &mut bottom[c..c + span] {
                *b = top + h + SECT_GAP;
            }
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
