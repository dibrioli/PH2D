//! **Os primitivos de ROW que o card Brush empresta aos outros** — o rótulo de coluna fixa, a row
//! `rótulo + chip` de dropdown, e o chip nu.
//!
//! ⚠️ **Eles moram aqui porque têm SETE consumidores** (Stroke, Shape, Texture, Paper, Ramp, Line,
//! e o próprio Brush): enquanto viviam dentro do `paint_brush.rs` o arquivo crescia por causa de
//! quem o importava, não de quem ele é — e foi assim que ele cruzou o teto de LOC. O corte é por
//! RESPONSABILIDADE: *a seção Brush* de um lado, *as peças de linha que qualquer seção usa* do
//! outro.

use ph2d_editor_core::IconId;
use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::paint::{fill_rounded_rect, paint_icon, paint_text, resolve};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::DropdownState;
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_tokens::{ColorToken, ROW_H_PX, Radius, Spacing, StrokeToken, TypeToken};

/// ⭐⭐⭐ **AS COLUNAS de uma linha deste painel, derivadas da SECÇÃO a que a chave pertence.**
///
/// ⛔⛔ Até 2026-09-16 a coluna do rótulo era o literal `LABEL_W = 60,0`, escrito no sítio de
/// pintura — o que a spec §3 proíbe por escrito (*«uma largura FIXA está errada por construção: a
/// coluna docada é arrastável»*). Medido nesse dia, com `60`:
///
/// - o rótulo **`Paint Mode`** mede `65,3 px` e saía **cortado em TODA largura de painel**;
/// - e os rótulos destas linhas ficavam encostados à ESQUERDA enquanto as caixas de marcar do mesmo
///   cartão já viviam na coluna da secção ⇒ **duas colunas de nome, alternando linha sim linha
///   não**, que é o defeito que o §6-quinquies da spec existe para matar.
///
/// ⚠️ **O censo que devia ter apanhado isto é CEGO à grafia:** o
/// `the_label_column_is_one_answer` procura `label_col` e este chamava-se `LABEL_W`. *A quinta
/// grafia da mesma pergunta* (§34.6) — a régua foi alargada no mesmo commit.
pub(crate) fn linha_da_chave(
    ctx: &mut PaintCtx,
    x: f32,
    content_w: f32,
    y: f32,
    chave: &str,
) -> ph2d_editor_core::widget::PropertyRow {
    let seccao = crate::seccoes::seccao_da_chave(ctx.text_system, chave);
    ph2d_editor_core::widget::colunas_da_linha(x, content_w, y, ROW_H_PX, seccao)
}

/// O rótulo de uma linha de propriedade: **alinhado à direita da coluna e elidido** (spec §4).
///
/// ⛔ `paint_text` perde as duas coisas — era com ele que este painel pintava, e é por isso que o
/// `Paint Mode` cortava sem reticências e sem ninguém ver.
pub(crate) fn label(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    text: &str,
    row: &ph2d_editor_core::widget::PropertyRow,
    font: f32,
) {
    ph2d_editor_core::widget::paint_property_label(
        ctx.text_system,
        ctx.scene,
        text,
        row.label.x,
        row.label.y + (row.label.h - font) * 0.5,
        font,
        row.label.w,
        resolve(ColorToken::Text2, theme),
    );
}

/// Paint a "label + dropdown chip" row. Returns `(next_y, Some(chip_rect))` when
/// the chip is open (the caller stashes the rect into the matching pending slot).
/// `pub(crate)` so the Stroke section reuses it for Method + Jitter Unit.
///
/// ⭐ **Ela recebe a CHAVE do rótulo, não o texto** — pela mesma razão que a linha de marcar
/// ([`crate::paint_brush_top::paint_checkbox_row`]): *quem só tem o texto traduzido não sabe a que
/// secção pertence*, e a coluna é uma resposta da secção.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_dropdown_row(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    chave: &str,
    id: ph2d_a11y::NodeId,
    cur_value: u8,
    cur_label: &str,
) -> (f32, Option<Rect>) {
    let row = linha_da_chave(ctx, x, content_w, y, chave);
    label(ctx, theme, tr(chave), &row, TypeToken::Sm.px());
    let rect = row.control;
    let open = paint_dropdown_chip(ctx, theme, id, cur_value, cur_label, rect);
    (y + ph2d_tokens::row_pitch_px(), open.then_some(rect))
}

/// Paint a dropdown chip (registered as a `Dropdown` for the generic open/close
/// dispatch). Returns whether it is open. Shared by the Blend + Falloff chips.
pub(crate) fn paint_dropdown_chip(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    id: ph2d_a11y::NodeId,
    cur_value: u8,
    cur_label: &str,
    rect: Rect,
) -> bool {
    ctx.host.store_mut().register_if_absent(
        id,
        InteractiveState::Dropdown {
            state: DropdownState::Normal,
            open: false,
            selected_index: Some(cur_value as usize),
        },
    );
    let open = matches!(
        ctx.host.store().get(id),
        Some(InteractiveState::Dropdown { open: true, .. })
    );

    // ⭐ Raio e moldura pela porta do TEMA, com o `Feel` do estado do dropdown — a mesma porta do
    //    `Dropdown` da casa (`dropdown_feel`), pela mesma razão do `chip_border_color` abaixo.
    let radius = ph2d_editor_core::paint::frame_radius(theme, Radius::Sm.px());
    fill_rounded_rect(
        ctx.scene,
        rect,
        radius,
        resolve(
            ph2d_editor_core::widget::section_cards::CardDepth::Subsection.token(),
            theme,
        ),
    );
    // ⚠️ **Pela porta do widget, e não por uma quarta cópia da lei.** Este chip é desenhado à
    // mão (não constrói um `Dropdown`) e por isso carregava a sua própria regra de borda — que
    // não conhecia `BorderEmph` e portanto **nunca acendia sob o ponteiro**.
    let (dd_state, dd_t) = ctx.host.store().dropdown_visual(id);
    ph2d_editor_core::paint::stroke_frame(
        ctx.scene,
        rect,
        radius,
        theme,
        ph2d_editor_core::widget::dropdown_feel(dd_state),
        StrokeToken::Default.px(),
        ph2d_editor_core::widget::chip_border_color(dd_state, dd_t, theme),
    );

    let chevron = Spacing::Md.px();
    let pad = Spacing::Sm.px();
    let chevron_rect = Rect::new(
        rect.x + rect.w - pad - chevron,
        rect.y + (rect.h - chevron) * 0.5,
        chevron,
        chevron,
    );
    let icon = if open {
        IconId::ChevronUp
    } else {
        IconId::ChevronDown
    };
    paint_icon(
        ctx.scene,
        icon,
        chevron_rect,
        resolve(ColorToken::Text2, theme),
        StrokeToken::Default.px(),
    );

    let font = TypeToken::Sm.px();
    let text_x = rect.x + pad;
    let text_w = (chevron_rect.x - Spacing::Xs.px() - text_x).max(0.0);
    paint_text(
        ctx.text_system,
        ctx.scene,
        cur_label,
        text_x,
        rect.y + (rect.h - font) * 0.5,
        font,
        text_w,
        resolve(ColorToken::Text1, theme),
    );

    ctx.host.hit_index_mut().register(id, rect);
    open
}
