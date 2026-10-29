//! **As propriedades do activo** (Frame Properties, Cel Properties, Tag Properties do Aseprite), à
//! direita da grelha: a duração do frame activo; a opacidade e o z do cel activo (se há cel); e a
//! direcção, as repetições e o apagar da tag que cobre o frame activo.

use crate::{PixelTimelinePanelState, event, ids};
use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::paint::{paint_text, rect_to_vello, resolve};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::pixel_panel::{PixelCel, PixelDirection, PixelPanelInfo};
use ph2d_editor_core::property_row;
use ph2d_editor_core::widget::{
    Button, Dropdown, DropdownOption, Seccao, Unit, paint_button, paint_dropdown_chip,
};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::{tr, tr_with};
use ph2d_tokens::{ColorToken, ROW_H_PX, Spacing, TypeToken};

/// Quantas colunas de propriedades: Frame · Cel · Tag.
pub(crate) const COLUNAS: usize = 3;

/// As opções do menu da direcção, na ordem do `.ase`.
pub(crate) fn direction_options() -> Vec<DropdownOption<PixelDirection>> {
    PixelDirection::ALL
        .iter()
        .enumerate()
        .map(|(i, &d)| {
            DropdownOption::new(ids::direction_option_id(i), d, tr(ids::direction_keys(d).0))
        })
        .collect()
}

/// O cel da camada activa no frame activo.
pub(crate) fn cel_activo(info: &PixelPanelInfo) -> Option<PixelCel> {
    let (l, f) = (info.active_layer?, info.active_frame?);
    info.cels
        .iter()
        .find(|c| c.layer == l && c.frame == f)
        .copied()
}

/// Quantas colunas de propriedades cabem em `w` (cada uma com a largura mínima de um painel).
pub(crate) fn colunas_em(w: f32) -> usize {
    let min = ph2d_tokens::chrome::PANEL_MIN_W_PX + Spacing::Md.px();
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "quantas colunas cabem, ≥ 0"
    )]
    let n = (w / min).floor().max(0.0) as usize;
    n.clamp(1, COLUNAS) // CLAMP-OK: 1 ≤ COLUNAS (const)
}

/// A largura das propriedades com `n` colunas.
pub(crate) fn largura(n: usize) -> f32 {
    #[expect(clippy::cast_precision_loss, reason = "≤ três colunas")]
    let n = n as f32;
    ph2d_tokens::chrome::PANEL_MIN_W_PX * n + Spacing::Md.px() * (n - 1.0)
}

/// As secções (Frame · Cel · Tag) em `n` colunas, em blocos contíguos; as que partilham uma coluna
/// empilham-se (recortadas à banda — nunca cortadas na largura).
pub(crate) fn paint(
    st: &mut PixelTimelinePanelState,
    ctx: &mut PaintCtx,
    info: &PixelPanelInfo,
    area: Rect,
    n: usize,
) {
    let Some(pos) = event::activo(info) else {
        return;
    };
    ctx.scene.push_clip(&rect_to_vello(area));
    ctx.host.hit_index_mut().push_clip(area);
    let nomes = [
        tr("panel.pixel.timeline.duration"),
        tr("panel.pixel.timeline.opacity"),
        tr("panel.pixel.timeline.z"),
        tr("panel.pixel.timeline.direction"),
        tr("panel.pixel.timeline.repeat"),
    ];
    let sec = Seccao::medida(ctx.text_system, 1, &nomes);
    let gap = Spacing::Md.px();
    let cw = ph2d_tokens::chrome::PANEL_MIN_W_PX;
    #[expect(clippy::cast_precision_loss, reason = "≤ três colunas")]
    let col = |k: usize| Rect::new(area.x + k as f32 * (cw + gap), area.y, cw, area.h);
    let tag = event::tag_activa(info);
    let tem_cel = cel_activo(info).is_some();
    // As secções repartem-se em blocos contíguos: a `i` de `m` vai para a coluna `i·n/m`.
    let m = 1 + usize::from(tem_cel) + usize::from(tag.is_some());
    let mut ys = [area.y; COLUNAS];
    let mut seccao = 0;
    let mut proxima = |ys: &mut [f32; COLUNAS]| {
        let k = (seccao * n / m).min(n - 1);
        seccao += 1;
        let c = col(k);
        let y = if ys[k] > c.y {
            ys[k] + ph2d_tokens::section_gap_px()
        } else {
            ys[k]
        };
        (k, Rect::new(c.x, y, c.w, c.h))
    };

    let (k, c) = proxima(&mut ys);
    let num = (pos + 1).to_string();
    let y = linha(
        ctx,
        c,
        &tr_with("panel.pixel.timeline.frame_props", &[("n", &num)]),
        c.y,
    );
    ys[k] = campo(ctx, c, nomes[0], ids::FRAME_DURATION, sec, y);
    if tem_cel {
        let (k, c) = proxima(&mut ys);
        let y = linha(ctx, c, tr("panel.pixel.timeline.cel_props"), c.y);
        let y = campo(ctx, c, nomes[1], ids::CEL_OPACITY, sec, y);
        ys[k] = campo(ctx, c, nomes[2], ids::CEL_Z, sec, y);
    }
    if let Some(t) = tag {
        let (_, c) = proxima(&mut ys);
        let tag = &info.tags[t];
        let titulo = tr_with("panel.pixel.timeline.tag_props", &[("name", &tag.name)]);
        let y = linha(ctx, c, &titulo, c.y);
        let y = direcao(st, ctx, c, nomes[3], tag.direction, y);
        let y = campo(ctx, c, nomes[4], ids::TAG_REPEAT, sec, y);
        let rotulo = tr("panel.pixel.timeline.tag_delete");
        let r = property_row::caixa_do_botao(ctx.text_system, c.x, c.w, y, rotulo);
        let theme = ctx.host.theme();
        let visual = ctx.host.store().button_visual(ids::TAG_DELETE);
        let b = Button::new(ids::TAG_DELETE, rotulo).visual(visual);
        paint_button(&b, r, ctx.scene, ctx.text_system, theme);
        ctx.host.hit_index_mut().register(ids::TAG_DELETE, r);
    }
    ctx.host.hit_index_mut().pop_clip();
    ctx.scene.pop_layer();
}

fn linha(ctx: &mut PaintCtx, area: Rect, texto: &str, y: f32) -> f32 {
    let theme = ctx.host.theme();
    paint_text(
        ctx.text_system,
        ctx.scene,
        texto,
        area.x,
        y + (ROW_H_PX - TypeToken::Sm.px()) * 0.5,
        TypeToken::Sm.px(),
        area.w.max(1.0),
        resolve(ColorToken::Text3, theme),
    );
    y + ROW_H_PX
}

fn campo(
    ctx: &mut PaintCtx,
    area: Rect,
    nome: &str,
    id: ph2d_a11y::NodeId,
    sec: Seccao,
    y: f32,
) -> f32 {
    let unit = (id == ids::FRAME_DURATION).then_some(Unit::Milliseconds);
    let theme = ctx.host.theme();
    let (store, hit) = ctx.host.store_and_hit_index_mut();
    property_row::paint_field_row(
        ctx.scene,
        ctx.text_system,
        theme,
        hit,
        store,
        area.x,
        area.w,
        y,
        nome,
        id,
        1.0,
        unit,
        sec,
    )
    .0
}

/// O rótulo e o chip do menu da direcção (a lista abre por cima, no fim do quadro).
fn direcao(
    st: &mut PixelTimelinePanelState,
    ctx: &mut PaintCtx,
    area: Rect,
    nome: &str,
    cur: PixelDirection,
    y: f32,
) -> f32 {
    // O nome numa linha e o chip na coluna inteira por baixo: as direcções são PALAVRAS («Ping-pong
    // Reverse»), e a coluna do valor de uma linha de campo é a de um número.
    let theme = ctx.host.theme();
    let y = linha(ctx, area, nome, y);
    let r = Rect::new(area.x, y, area.w, ROW_H_PX);
    let open = matches!(
        ctx.host.store().get(ids::TAG_DIRECTION),
        Some(InteractiveState::Dropdown { open: true, .. })
    );
    let dd = Dropdown::new(ids::TAG_DIRECTION, "", direction_options())
        .selected(cur)
        .open(open)
        .visual(ctx.host.store().dropdown_visual(ids::TAG_DIRECTION));
    paint_dropdown_chip(&dd, r, ctx.scene, ctx.text_system, theme);
    ctx.host.hit_index_mut().register(ids::TAG_DIRECTION, r);
    if open {
        st.direction_open = Some((r, cur));
    }
    y + ROW_H_PX + ph2d_tokens::control_gap_px()
}
