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
/// Espessura da barra de frames: o rectângulo de acerto É a trilha (o despacho deriva o valor de
/// `rect.x`/`rect.w`), logo é o alvo do ponteiro. Em 6 px era uma linha de leitura, não um alvo.
const BAR_H: f32 = 10.0; // LITERAL-PX-OK: altura de trilha de slider, é o alvo do ponteiro
/// A parte da largura da lista que o resumo de uma linha pode ocupar, no máximo (1 menos isto).
const SUMMARY_COL_FRAC: f32 = 0.55; // LITERAL-PX-OK: fração de layout

/// A pré-visualização da célula no ecrã — só quando a sprite tem pixels em memória.
pub(crate) fn paint_preview(
    scene: &mut VectorScene,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorAnimInfo,
) -> f32 {
    if info.card.sheet.is_none() {
        return y;
    }
    let h = ROW_H * PREVIEW_ROWS;
    // Cada quadro pinta-se pela imagem da folha DELE (plano GEF 26); um vazio não pinta.
    if let Some((sheet, cols, rows)) = info.card.sheet_of(info.frame.sheet) {
        paint_sheet_cell(
            scene,
            sheet,
            cols,
            rows,
            info.frame.cell,
            Rect::new(x, y, w, h),
        );
    }
    y + h + ph2d_tokens::control_gap_px()
}

/// A linha cujo nome está a ser renomeado (dois cliques) e onde devolver o rect do nome dela.
pub(crate) type Renomeia<'a> = Option<(usize, &'a mut Option<Rect>)>;

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
    mut renomeia: Renomeia<'_>,
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
    // O resumo manda na coluna da direita: mede-se (nunca se elide) até no máximo `1 - SUMMARY_COL_FRAC`
    // da linha; o nome fica com o que sobra, e o vão o deixa cortado longe do resumo.
    let resumos: Vec<(String, ColorToken)> = info
        .rows
        .iter()
        .map(|row| {
            let plays = row.plays_on(&info.sheets);
            let vazios = crate::cards_sprite_folhas::vazios(row);
            // Os quadros de uma folha apagada: a linha fica em alerta (plano GEF 26 §3.3).
            if vazios > 0 {
                let texto = tr_with("panel.inspector.anim_card.empty", &[("n", &vazios)]);
                return (texto, ColorToken::Warn);
            }
            let texto = if plays {
                tr_with(
                    "panel.inspector.anim_card.frames",
                    &[("n", &row_cells(row, &info.sheets).len())],
                )
            } else {
                tr("panel.inspector.animation.out_of_grid").to_owned()
            };
            (
                texto,
                if plays {
                    ColorToken::Text3
                } else {
                    ColorToken::Danger
                },
            )
        })
        .collect();
    let resumo_w = ph2d_editor_core::paint::label_column_width(
        text_system,
        font,
        resumos.iter().map(|(t, _)| t.as_str()),
    )
    .min((w - ROW_H - gap) * (1.0 - SUMMARY_COL_FRAC));
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
        let alerta = crate::cards_sprite_folhas::vazios(row) > 0;
        if alerta {
            ph2d_editor_core::paint::fill_rounded_rect(
                scene,
                rect,
                ph2d_card_stack::metrics::SEG_RADIUS,
                resolve(ColorToken::WarnSoft, theme),
            );
            crate::cards_sprite_folhas::marca_alerta(row_id);
        }
        let cells = row_cells(row, &info.sheets);
        let mut text_x = x + Spacing::Sm.px();
        if info.card.sheet.is_some() {
            // A miniatura é um quadrado do lado da linha; o texto anda na HORIZONTAL, não é um passo.
            let thumb = Rect::new(x, cur_y, ROW_H, ROW_H);
            if let Some(first) = cells.first()
                && let Some((sheet, cols, rows)) = info.card.sheet_of(first.sheet)
            {
                paint_sheet_cell(scene, sheet, cols, rows, first.cell, thumb);
            }
            text_x = thumb.x + thumb.w + Spacing::Sm.px();
        }
        let color = if alerta {
            resolve(ColorToken::Warn, theme)
        } else if info.current == row.name {
            resolve(ColorToken::Accent, theme)
        } else if is_open {
            resolve(ColorToken::Text1, theme)
        } else {
            resolve(ColorToken::Text2, theme)
        };
        // O vão deixa o nome cortado longe do resumo: no piso do painel, «crouc…4 frames» lia-se como uma palavra.
        let resumo_x = x + rect.w - resumo_w;
        let ate_o_resumo = (resumo_x - text_x - Spacing::Sm.px()).max(0.0);
        match &mut renomeia {
            Some((j, campo)) if *j == i => {
                **campo = Some(Rect::new(text_x, cur_y, ate_o_resumo, ROW_H));
            }
            _ => paint_text(
                text_system,
                scene,
                &if alerta {
                    tr_with(
                        "panel.inspector.anim_card.alert_name",
                        &[("name", &row.name)],
                    )
                } else {
                    row.name.clone()
                },
                text_x,
                cur_y + super::rows::folga_da_linha(ROW_H, font),
                font,
                ate_o_resumo,
                color,
            ),
        }
        let (summary, tinta) = &resumos[i];
        paint_text(
            text_system,
            scene,
            summary,
            resumo_x,
            cur_y + super::rows::folga_da_linha(ROW_H, font),
            font,
            resumo_w + 1.0, // LITERAL-PX-OK: folga de arredondamento da medida
            resolve(*tinta, theme),
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

/// A fileira do tocador: (⏮ só em `rewind`) · ▶/⏸ · a barra · «Frame 3 / 5». A posição da barra vem do
/// SNAPSHOT (`scrub_position`), nunca do store. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
pub(crate) fn transport_row(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorAnimInfo,
    rewind: bool,
) -> f32 {
    let font = TypeToken::Sm.px();
    let gap = ph2d_tokens::control_gap_px();
    let mut bx = x;
    if rewind {
        super::anim::icon_button(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            Rect::new(bx, y, ROW_H, ROW_H),
            ids::INSP_ANIM_REWIND,
            IconId::SkipBack,
            tr("panel.inspector.animation.rewind"),
        );
        bx += ROW_H + gap;
    }
    let (icon, label) = if info.playing {
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
        Rect::new(bx, y, ROW_H, ROW_H),
        ids::INSP_ANIM_PLAYING,
        icon,
        label,
    );
    if let Some((step, span)) = info.progress() {
        let text = tr_with(
            "panel.inspector.animation.frame_of",
            &[("n", &(step + 1)), ("span", &span)],
        );
        // O rótulo mede-se (nunca se elide); a barra fica com o que sobra entre os botões e ele.
        let text_w =
            ph2d_editor_core::paint::label_column_width(text_system, font, [text.as_str()]);
        let text_x = x + w - text_w;
        paint_text(
            text_system,
            scene,
            &text,
            text_x,
            y + (ROW_H - font) * 0.5,
            font,
            text_w + 1.0, // LITERAL-PX-OK: folga de arredondamento da medida
            resolve(ColorToken::Text2, theme),
        );
        let bar_x = bx + ROW_H + gap;
        let track = Rect::new(
            bar_x,
            y + (ROW_H - BAR_H) * 0.5,
            (text_x - gap - bar_x).max(0.0),
            BAR_H,
        );
        hit_index.register(ids::INSP_ANIM_FRAME_SCRUB, track);
        ph2d_editor_core::widget::paint_slider_track(
            track,
            info.scrub_position().unwrap_or(0.0),
            ph2d_editor_core::widget::SliderOrientation::Horizontal,
            store.slider_visual(ids::INSP_ANIM_FRAME_SCRUB),
            scene,
            theme,
        );
    }
    y + ROW_H + gap
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
    super::anim_rules::esquece_a_acesa();
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
    // O que cada escolha faz mora no balão de cada botão (`populate_anim`; ordem do dono 21/09).
    let cur_y = ph2d_editor_core::property_row::paint_choice_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        y,
        tr(super::anim_rules::rotulo_de_quem_escolhe(info)),
        &segments,
        sec,
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
