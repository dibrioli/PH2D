//! **O cartão «Animation»** (plano GameEngineFuncional 07, «Desenho aprovado» 1): a pré-visualização,
//! a lista com miniatura e ▶, «Edit Animations…», a grelha e «Who Picks» — Me · Script · Rules,
//! com a lista de regras em [`super::anim_rules`] (plano 08).
//!
//! ⚠️ Irmão de [`super::anim`] (o cabeçalho, a face sem tocador, «Playback» e o grupo «Advanced») e
//! de [`super::anim_rows`] (os campos da animação escolhida) pelo tecto de LOC dos painéis.

use super::*;
use ph2d_editor_core::screens::hero::{AnimChooser, InspectorAnimInfo, row_cells};
use ph2d_editor_core::widget::sheet_cell::paint_sheet_cell;
use ph2d_i18n::{tr, tr_with};

const ROW_H: f32 = ph2d_tokens::ROW_H_PX;
/// Quantas linhas de lista a pré-visualização ocupa (um número de linhas, não pixels).
const PREVIEW_ROWS: f32 = 4.0; // LITERAL-PX-OK: contagem de linhas da grelha do painel
/// Onde o resumo de uma linha da lista começa, como fração da largura.
const SUMMARY_COL_FRAC: f32 = 0.55; // LITERAL-PX-OK: fração de layout

/// A pré-visualização da célula no ecrã — só quando a sprite tem pixels em memória.
pub(crate) fn paint_preview(
    scene: &mut VectorScene,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorAnimInfo,
) -> f32 {
    let Some(sheet) = info.card.sheet.as_ref() else {
        return y;
    };
    let h = ROW_H * PREVIEW_ROWS;
    paint_sheet_cell(
        scene,
        sheet,
        info.card.hframes,
        info.card.vframes,
        info.frame,
        Rect::new(x, y, w, h),
    );
    y + h + ph2d_tokens::control_gap_px()
}

/// **A lista**: miniatura (a 1.ª célula da animação), nome, resumo e ▶. Clicar na linha escolhe o
/// que toca (a lei da §11); o ▶ escolhe E põe a tocar. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_list(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorAnimInfo,
    selected: usize,
) -> f32 {
    let font = TypeToken::Sm.px();
    let mut cur_y = y;
    if info.rows.is_empty() {
        paint_text(
            text_system,
            scene,
            tr("panel.inspector.animation.no_animations_yet"),
            x,
            cur_y,
            font,
            w,
            resolve(ColorToken::Text3, theme),
        );
        return cur_y + font + ph2d_tokens::control_gap_px();
    }
    let gap = ph2d_tokens::control_gap_px();
    for (i, ((row, &row_id), &play_id)) in info
        .rows
        .iter()
        .zip(ids::INSP_ANIM_ROW.iter())
        .zip(ids::INSP_ANIM_PLAY_ROW.iter())
        .enumerate()
    {
        let play = Rect::new(x + w - ROW_H, cur_y, ROW_H, ROW_H);
        let rect = Rect::new(x, cur_y, w - ROW_H - gap, ROW_H);
        hit_index.register(row_id, rect);
        ph2d_editor_core::widget::paint_row_stripe(
            scene,
            rect,
            theme,
            ph2d_editor_core::widget::section_cards::CardDepth::Section.token(),
            i,
        );
        let is_open = i == selected;
        ph2d_editor_core::widget::paint_row_highlight(
            scene,
            rect,
            theme,
            if is_open {
                ph2d_editor_core::widget::RowHighlight::Selected
            } else {
                ph2d_editor_core::widget::RowHighlight::None
            },
            0.0,
        );
        let cells = row_cells(row, info.cells);
        let mut text_x = x + Spacing::Sm.px();
        if let Some(sheet) = info.card.sheet.as_ref() {
            if let Some(&first) = cells.first() {
                paint_sheet_cell(
                    scene,
                    sheet,
                    info.card.hframes,
                    info.card.vframes,
                    first,
                    Rect::new(x, cur_y, ROW_H, ROW_H),
                );
            }
            text_x = x + ROW_H + Spacing::Sm.px();
        }
        let color = if info.current == row.name {
            resolve(ColorToken::Accent, theme)
        } else if is_open {
            resolve(ColorToken::Text1, theme)
        } else {
            resolve(ColorToken::Text2, theme)
        };
        paint_text(
            text_system,
            scene,
            &row.name,
            text_x,
            cur_y + (ROW_H - font) * 0.5,
            font,
            rect.w * SUMMARY_COL_FRAC - (text_x - x),
            color,
        );
        let plays = row.plays_on(info.cells);
        let summary = if plays {
            tr_with("panel.inspector.anim_card.frames", &[("n", &cells.len())])
        } else {
            tr("panel.inspector.animation.out_of_grid").to_owned()
        };
        paint_text(
            text_system,
            scene,
            &summary,
            x + rect.w * SUMMARY_COL_FRAC,
            cur_y + (ROW_H - font) * 0.5,
            font,
            rect.w * (1.0 - SUMMARY_COL_FRAC),
            resolve(
                if plays {
                    ColorToken::Text3
                } else {
                    ColorToken::Danger
                },
                theme,
            ),
        );
        // O ▶ é o ÍCONE (uma caixa de linha não tem respiro para o glifo de texto); a linha que está
        // no ecrã a tocar mostra ⏸ — a mesma pergunta que o clique faz (`row_shows_pause`).
        let (icon, label) = if crate::event_anim_card::row_shows_pause(info, i) {
            (IconId::Pause, tr("panel.inspector.anim_card.pause"))
        } else {
            (IconId::Play, tr("panel.inspector.anim_card.play"))
        };
        super::anim::icon_button(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            play,
            play_id,
            icon,
            label,
        );
        cur_y += ROW_H + ph2d_tokens::list_row_gap_px();
    }
    cur_y + ph2d_tokens::control_gap_px()
}

/// Um botão de linha inteira (o padrão `caixa_do_botao`). Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
pub(crate) fn button_row(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    id: NodeId,
    label: &str,
) -> f32 {
    let r = ph2d_editor_core::property_row::caixa_do_botao(text_system, x, w, y, label);
    hit_index.register(id, r);
    paint_button(
        &Button::new(id, label)
            .kind(ButtonKind::Default)
            .visual(store.button_visual(id)),
        r,
        scene,
        text_system,
        theme,
    );
    ph2d_editor_core::property_row::abaixo_do_botao(r)
}

/// Uma frase de ajuda em `Text3`, QUEBRADA em tantas linhas quantas pede (nunca elidida). Devolve o
/// `y` seguinte.
#[allow(clippy::too_many_arguments)]
#[track_caller]
fn hint(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    x: f32,
    w: f32,
    y: f32,
    text: &str,
    color: ColorToken,
) -> f32 {
    super::rows::aviso(scene, text_system, theme, x, w, y, text, color)
}

/// **«Who Picks»** e o que cada escolha mostra. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_chooser(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorAnimInfo,
    sec: ph2d_editor_core::property_row::Seccao,
) -> f32 {
    let chosen = info.card.chooser;
    let labels = [
        tr("panel.inspector.anim_card.me"),
        tr("panel.inspector.anim_card.script"),
        tr("panel.inspector.anim_card.rules"),
    ];
    let segments: Vec<(&str, bool, NodeId)> = AnimChooser::ALL
        .iter()
        .zip(labels)
        .zip(ids::INSP_ANIM_CHOOSER)
        .map(|((&c, text), id)| (text, c == chosen, id))
        .collect();
    let mut cur_y = ph2d_editor_core::property_row::paint_choice_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        y,
        tr("panel.inspector.anim_card.who_picks"),
        &segments,
        sec,
    );
    let key = match chosen {
        AnimChooser::Me => "panel.inspector.anim_card.me_hint",
        AnimChooser::Script => "panel.inspector.anim_card.script_hint",
        AnimChooser::Rules => "panel.inspector.anim_card.rules_hint",
    };
    cur_y = hint(
        scene,
        text_system,
        theme,
        x,
        w,
        cur_y,
        tr(key),
        ColorToken::Text3,
    );
    if chosen == AnimChooser::Rules {
        return super::anim_rules::rule_rows(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            info,
            sec,
        );
    }
    cur_y
}
