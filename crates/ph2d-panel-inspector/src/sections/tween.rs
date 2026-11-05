//! ⭐⭐⭐ **A secção TWEEN** (suplente #22) — *«esta propriedade vai de A a B»*.
//!
//! ⚠️ **Irmã da [`super::timers`], e o molde é o dela pela mesma razão:** um `Tweens` é uma LISTA
//! cujo índice **é a identidade** (é ele que liga o tween ao timer do mesmo índice), e cada tween
//! tem cinco campos. ⇒ a **lista** escolhe qual está aberto, e um editor só, abaixo dela, mostra os
//! campos desse.
//!
//! ⛔ **Este ficheiro é a MOLDURA — a lista, os dois botões e o cabeçalho.** Quem desenha os campos
//! do tween aberto é o [`super::tween_editor`], e a fronteira é a que o parágrafo acima já nomeia:
//! *a lista escolhe; o editor mostra*. O corte veio do tecto de LOC do painel (`600`), e é **corte
//! por responsabilidade, nunca uma entrada nova no `FILE_OVERAGE_OK`**.

use super::*;
use ph2d_editor_core::tween_edits::InspectorTweenInfo;
use ph2d_editor_core::widget::SectionFold;
use ph2d_i18n::{tr, tr_with};
use ph2d_tween::Canal;

/// A linha de uma lista é a linha do app — pela porta, nunca por um literal que coincide.
pub(super) const ROW_H: f32 = ph2d_tokens::ROW_H_PX;

/// A lista. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
fn list(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorTweenInfo,
    selected: usize,
) -> f32 {
    let y = super::rows::antes_da_lista(y);
    let font = TypeToken::Sm.px();
    let mut cur_y = y;
    // ⚠️ **`zip` com o array de ids**: uma lista com mais tweens do que ids perde os excedentes em
    // vez de os pintar uns sobre os outros (e há gate a prender os dois comprimentos).
    for (i, (row, &id)) in info
        .rows
        .iter()
        .zip(crate::ids::INSP_TWEEN_ROW.iter())
        .enumerate()
    {
        let rect = Rect::new(x, cur_y, w, ROW_H);
        hit_index.register(id, rect);
        ph2d_editor_core::widget::paint_row_stripe(
            scene,
            rect,
            theme,
            ph2d_editor_core::widget::section_cards::CardDepth::Section.token(),
            i,
        );
        // ⚠️ **O `Hovered` está aqui de propósito:** uma linha que só acende ao ser escolhida não
        // diz que é clicável — e isso lê-se exactamente como um controlo morto sob o dedo.
        ph2d_editor_core::widget::paint_row_highlight(
            scene,
            rect,
            theme,
            if i == selected {
                ph2d_editor_core::widget::RowHighlight::Selected
            } else if store.hover_live(id) > 0.0 {
                ph2d_editor_core::widget::RowHighlight::Hovered
            } else {
                ph2d_editor_core::widget::RowHighlight::None
            },
            0.0,
        );
        // ⚠️ **Um tween com QUEIXA escreve-se em WARN**, e a razão é a mesma do irmão: *«nada
        // acontece»* é o sintoma mais caro deste componente, e as causas autoráveis dele são
        // invisíveis num resumo.
        let color = if row.queixa(info.tem_sprite).is_some() {
            resolve(ColorToken::Warn, theme)
        } else if i == selected {
            resolve(ColorToken::Text1, theme)
        } else {
            resolve(ColorToken::Text2, theme)
        };
        paint_text(
            text_system,
            scene,
            tr(Canal::from_tag(row.canal).label_key()),
            x + Spacing::Sm.px(),
            cur_y + super::rows::folga_da_linha(ROW_H, font),
            font,
            w,
            color,
        );
        cur_y += ROW_H;
    }
    // ⚠️ **A cauda da lista vem da PORTA** — há gate contra a segunda resposta.
    super::rows::depois_da_lista(cur_y)
}

/// Os dois botões da lista, lado a lado. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
fn buttons(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorTweenInfo,
) -> f32 {
    // ⚠️ **O `+` DESAPARECE no tecto**, e não fica cinzento a mentir: um botão que aceita o clique
    // e não faz nada é o defeito que a DIRETIVA §2 nomeia.
    let can_add = info.rows.len() < crate::ids::INSP_TWEEN_ROW.len();
    let can_remove = !info.rows.is_empty();
    super::rows::fila_de_botoes(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        y,
        &[
            (
                can_add,
                ids::INSP_TWEEN_ADD,
                tr("panel.inspector.tween.plus_add_tween"),
            ),
            (
                can_remove,
                ids::INSP_TWEEN_REMOVE,
                tr("panel.inspector.tween.x_remove_tween"),
            ),
        ],
    )
}

/// Pinta a secção. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_tween_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorTweenInfo,
    selected: usize,
) -> f32 {
    let header_h = TypeToken::Md.px() + Spacing::Md.px(); // LITERAL-PX-OK: banda do cabeçalho
    let title = if info.rows.is_empty() {
        String::from(tr("panel.inspector.tween.tween"))
    } else {
        tr_with(
            "panel.inspector.tween.title_count",
            &[("n", &info.rows.len())],
        )
    };
    let header = section_header(store, core_ids::INSP_LIVE_TWEEN_SECTION, &title);
    let header_rect = Rect::new(x, y, w, header_h);
    paint_section_header(&header, header_rect, scene, text_system, theme);
    let Some(fold) = SectionFold::begin(
        store,
        core_ids::INSP_LIVE_TWEEN_SECTION,
        x,
        w,
        y + header_h,
        scene,
        hit_index,
    ) else {
        return y + header_h;
    };
    let mut cur_y = y + header_h;
    let font = TypeToken::Sm.px();

    // ⚠️ **A SELECÇÃO MÚLTIPLA tem de se dizer** — o índice que uma edição carrega só significa
    // alguma coisa na lista da primária. Vem antes de tudo, porque *«em quem é que isto pega?»* é
    // anterior a qualquer controlo.
    if info.rows.is_empty() {
        paint_text(
            text_system,
            scene,
            tr("panel.inspector.tween.no_tweens_yet"),
            x,
            cur_y,
            font,
            w,
            resolve(ColorToken::Text3, theme),
        );
        cur_y += font + ph2d_tokens::control_gap_px();
    } else {
        cur_y = list(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            info,
            selected,
        );
    }
    cur_y = buttons(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        info,
    );
    if let Some(row) = info.rows.get(selected) {
        cur_y = super::tween_editor::editor(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            row,
            info.tem_sprite,
            selected,
        );
    }
    fold.finish(store, scene, hit_index, cur_y + SECTION_BOTTOM_PAD_PX)
}
