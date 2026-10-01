//! The panel body. Walks [`crate::rows::SECTIONS`] — the same table `populate`
//! and `event` walk — so what is drawn, what is registered and what dispatches
//! cannot disagree.

use ph2d_editor_core::ids;
use ph2d_editor_core::paint::{paint_text_block, resolve};
use ph2d_editor_core::panel::{PaintCtx, Panel};
use ph2d_editor_core::widget::panel_chrome::{
    PANEL_HEAD_PAD, PANEL_HEADER_CLOSE_RESERVE, PANEL_TITLE_BASELINE, paint_panel_close_button,
    paint_panel_surface, paint_panel_title,
};
use ph2d_editor_core::widget::{PHYSICS_SCROLLBAR_ID, paint_slider_with_chip_layout_adaptive};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_tokens::{ColorToken, ROW_H_PX, Spacing, Theme, TypeToken};

use crate::state::{self, PhysicsPanelState, set_last_content_h, set_last_visible_h};
use crate::{PhysicsPanel, rows};

mod body;
mod interact;
mod joint;
mod matrix;
mod plano;

pub(crate) fn paint(_state: &mut PhysicsPanelState, ctx: &mut PaintCtx) {
    if !ctx.host.panel_visible(PhysicsPanel::ID) {
        // Symmetric stale-rect cleanup, so `panel_at` stops returning
        // PHYSICS_PANEL the moment the panel is closed.
        ctx.host.store_mut().clear_panel_rect(ids::PHYSICS_PANEL);
        return;
    }

    let rect: Rect = ctx.slot;
    let theme = ctx.host.theme();
    let snapshot = state::current();
    // A lente do pintor recomeça a cada quadro — ver `state::PAINTED_SECTION_HEADERS`.
    state::begin_painted_section_headers();

    // Publish the rect so wheel/click dispatch can route to this panel.
    ctx.host
        .store_mut()
        .set_panel_rect(ids::PHYSICS_PANEL, rect);

    paint_panel_surface(rect, ctx.scene, theme);

    // Dock-slot drag + resize handles. Reuse the Inspector ids because the
    // right dock slot is shared — the resize delta should persist when the
    // artist switches between Inspector and a world panel.
    {}

    let title_size = paint_panel_title(
        rect,
        PhysicsPanel::TITLE.tr(),
        PANEL_HEADER_CLOSE_RESERVE,
        ctx.scene,
        ctx.text_system,
        theme,
    );
    paint_panel_close_button(
        rect,
        crate::ids::PHYSICS_CLOSE,
        ctx.host.hit_index_mut(),
        ctx.scene,
        theme,
    );

    let body_top = rect.y + PANEL_TITLE_BASELINE + title_size + Spacing::Md.px();
    let body_h = (rect.y + rect.h - body_top - PANEL_HEAD_PAD).max(0.0);
    let body_rect = Rect::new(rect.x, body_top, rect.w, body_h);
    // ⭐ A PORTA da rolagem (spec `04_a_rolagem_unica`): recorte do desenho e do clique, as duas
    // alturas, o clamp e a barra com a TRILHA registada.
    let area = ph2d_editor_core::panel::scroll_area::open(
        ctx,
        ids::PHYSICS_PANEL,
        PHYSICS_SCROLLBAR_ID,
        body_rect,
    );
    // ⭐⭐ **O corpo pinta-se DENTRO de cartões** (2026-09-30, *«siga com os outros painéis»*) — o
    //    livro que o tema por secção recolore. O par abre DEPOIS do `open` da rolagem, para o corpo
    //    devolvido cair dentro do recorte dela; e o topo desce a folga do cartão, senão a borda de
    //    cima do primeiro é cortada pelo recorte.
    let body_paint_top = area.top() + ph2d_tokens::card_pad_px();
    ph2d_editor_core::widget::section_cards::begin_section_cards(ctx.scene, theme, body_paint_top);
    let y_after = plano::paint_sections(
        ctx,
        theme,
        &snapshot,
        rect.x + PANEL_HEAD_PAD,
        (rect.w - PANEL_HEAD_PAD * 2.0).max(0.0),
        body_paint_top,
    );
    ph2d_editor_core::widget::section_cards::end_section_cards(ctx.scene);
    let content_h = (y_after + area.scroll()) - body_top + PANEL_HEAD_PAD;
    set_last_content_h(content_h);
    set_last_visible_h(body_h);
    ph2d_editor_core::panel::scroll_area::close(area, ctx, content_h);
}

/// One slider+chip row, from the table.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_row(
    ctx: &mut PaintCtx,
    theme: Theme,
    row: &rows::Row,
    value: f32,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    let scene = &mut *ctx.scene;
    let text_system = &mut *ctx.text_system;
    let (store, hit_index) = ctx.host.store_and_hit_index_mut();

    // The published value is the truth; the stored track is only what the drag
    // left behind. Seeding the track from the value every frame is what keeps
    // the slider honest when the settings change from anywhere else (Reset to
    // Defaults, a project load, an undo).
    let track = row.track_of(value);
    let display = f64::from(value);
    let text = format!("{display:.*}", row.decimals);
    let label = tr(row.label);

    paint_slider_with_chip_layout_adaptive(
        Rect::new(x, y, w, ROW_H_PX),
        label,
        track,
        display,
        Some(&text),
        row.slider,
        row.chip,
        ph2d_editor_core::widget::property_label_col_w(x, w),
        ph2d_editor_core::widget::NUMBER_INPUT_MIN_W_PX,
        store,
        hit_index,
        scene,
        text_system,
        theme,
    )
}

/// One Interaction slider+chip row, from the Interaction table. The twin of
/// [`paint_row`] — the two tables have the same row shape and different owners,
/// so this is a delegation rather than a second layout.
pub(crate) fn paint_irow(
    ctx: &mut PaintCtx,
    theme: Theme,
    row: &crate::interact::IRow,
    value: f32,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    let scene = &mut *ctx.scene;
    let text_system = &mut *ctx.text_system;
    let (store, hit_index) = ctx.host.store_and_hit_index_mut();
    let track = row.track_of(value);
    let display = f64::from(value);
    let text = format!("{display:.*}", row.decimals);
    paint_slider_with_chip_layout_adaptive(
        Rect::new(x, y, w, ROW_H_PX),
        tr(row.label),
        track,
        display,
        Some(&text),
        row.slider,
        row.chip,
        ph2d_editor_core::widget::property_label_col_w(x, w),
        ph2d_editor_core::widget::NUMBER_INPUT_MIN_W_PX,
        store,
        hit_index,
        scene,
        text_system,
        theme,
    )
}

/// **Uma linha de dica** — texto puro, hit-indexado por ninguém (é um fato, não
/// um controle, e uma affordance que ele não pode honrar seria pior). Devolve o
/// `y` seguinte.
///
/// ⚠️ **O avanço é MEDIDO, não `ROW_H_PX`.** Uma dica é a única coisa deste
/// painel cujo comprimento é livre — ela quebra em duas linhas num painel
/// estreito ou num idioma mais comprido —, e um avanço fixo escreve a linha
/// seguinte por cima dela (foi exatamente o que o smoke da seção Joints mostrou).
/// `paint_text_block` pinta e devolve a altura da MESMA passada de layout, então
/// não há como as duas discordarem.
pub(crate) fn paint_hint(
    ctx: &mut PaintCtx,
    theme: Theme,
    key: &str,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    let font = TypeToken::Sm.px();
    let used = paint_text_block(
        ctx.text_system,
        ctx.scene,
        tr(key),
        x,
        y + (ROW_H_PX - font) * 0.5,
        font,
        w,
        resolve(ColorToken::Text2, theme),
    );
    // O piso é a altura de row: uma dica de uma linha continua ocupando o mesmo
    // espaço que sempre ocupou, e só o excedente da quebra é acrescentado.
    y + (ROW_H_PX - font).mul_add(0.5, used).max(ROW_H_PX) + ph2d_tokens::control_gap_px()
}
