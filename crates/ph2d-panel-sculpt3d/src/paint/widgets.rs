//! **OS WIDGETS DESTE PAINEL** — as cinco formas que as seções montam.
//!
//! Irmão (`#[path]`) do [`super::body`], e o corte é o que o cap de LOC do
//! painel pediu — mas ele é honesto por conta própria: aqui mora *como uma
//! fileira de chips, um cabeçalho dobrável ou um interruptor SE DESENHAM*, e lá
//! mora *que seções existem e em que ordem*. Os dois crescem por motivos
//! diferentes: este quando o painel ganha uma forma nova, aquele quando a cena
//! ganha um controle.

use ph2d_editor_core::paint::{paint_text, resolve};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::{
    Button, ButtonKind, ButtonState, SectionHeader, SegmentedAdaptive, SegmentedOption,
    paint_button, paint_section_header, paint_segmented_adaptive,
};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ColorToken, ROW_H_PX, Spacing, TypeToken};

/// Um cabeçalho dobrável. Devolve `(está_aberto, y_depois)`.
pub(super) fn header(
    ctx: &mut PaintCtx,
    id: ph2d_a11y::NodeId,
    title: &str,
    x: f32,
    w: f32,
    y: f32,
) -> (bool, f32) {
    let theme = ctx.host.theme();
    let h = TypeToken::Md.px() + Spacing::Md.px(); // LITERAL-PX-OK: altura da faixa de cabeçalho
    let collapsed = ctx.host.store().is_collapsed(id);
    let rect = Rect::new(x, y, w, h);
    let head = SectionHeader::new(id, title)
        .collapsible(!collapsed)
        .open_t(ctx.host.store().section_open_live(id));
    let scene = &mut *ctx.scene;
    let text_system = &mut *ctx.text_system;
    let (_, hit_index) = ctx.host.store_and_hit_index_mut();
    paint_section_header(&head, rect, scene, text_system, theme);
    hit_index.register(id, rect);
    (!collapsed, y + h + Spacing::Sm.px())
}

/// Um grupo segmentado sem rótulo (a lista de ferramentas).
#[allow(clippy::too_many_arguments)]
pub(super) fn seg(
    ctx: &mut PaintCtx,
    group: ph2d_a11y::NodeId,
    options: &[ph2d_a11y::NodeId],
    labels: &[&str],
    selected: usize,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    let theme = ctx.host.theme();
    let widget = SegmentedAdaptive::new(
        group,
        "",
        options
            .iter()
            .zip(labels)
            .map(|(&id, &l)| SegmentedOption::new(id, l))
            .collect(),
    )
    .selected(selected);
    let scene = &mut *ctx.scene;
    let text_system = &mut *ctx.text_system;
    let (store, hit_index) = ctx.host.store_and_hit_index_mut();
    let h = paint_segmented_adaptive(
        &widget,
        Rect::new(x, y, w, ROW_H_PX),
        scene,
        text_system,
        theme,
        store,
        hit_index,
    );
    y + h
}

/// Um grupo segmentado com rótulo em cima.
#[allow(clippy::too_many_arguments)]
pub(super) fn labelled_seg(
    ctx: &mut PaintCtx,
    label: &str,
    group: ph2d_a11y::NodeId,
    options: &[ph2d_a11y::NodeId],
    labels: &[&str],
    selected: usize,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    let theme = ctx.host.theme();
    let font = TypeToken::Sm.px();
    // `Md` e não `Xs`: o texto é pintado CENTRADO nesta faixa, então o respiro
    // que sobra abaixo dele é metade da folga — com `Xs` o rótulo encosta nos
    // chips e o olho lê a palavra como parte do primeiro botão (o número saiu do
    // smoke do painel de física).
    let label_h = font + Spacing::Md.px();
    paint_text(
        ctx.text_system,
        ctx.scene,
        label,
        x,
        y + (label_h - font) * 0.5,
        font,
        w,
        resolve(ColorToken::Text2, theme),
    );
    seg(ctx, group, options, labels, selected, x, w, y + label_h) + Spacing::Sm.px()
}

/// Dois botões lado a lado.
pub(super) fn row_of_two(
    ctx: &mut PaintCtx,
    left: (ph2d_a11y::NodeId, &str),
    right: (ph2d_a11y::NodeId, &str),
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    let gap = Spacing::Sm.px();
    let half = (w - gap) * 0.5;
    command(ctx, left.0, left.1, x, half, y);
    command(ctx, right.0, right.1, x + half + gap, half, y)
}

/// Um `Button` usado como toggle.
///
/// **Não é um `Checkbox`**: `Checkbox` emite `Toggled`, que o `event.rs` deste
/// painel não encaminha, então ele nasceria registrado e morto no clique — o
/// mesmo aviso que o `ph2d-panel-painter-layers` carrega pelo mesmo motivo.
pub(super) fn toggle(
    ctx: &mut PaintCtx,
    id: ph2d_a11y::NodeId,
    label: &str,
    on: bool,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    let theme = ctx.host.theme();
    let rect = Rect::new(x, y, w, ROW_H_PX);
    let state = if on {
        (ButtonState::Pressed, ph2d_editor_core::motion::SETTLED)
    } else {
        ctx.host.store().button_visual(id)
    };
    let kind = if on {
        ButtonKind::Accent
    } else {
        ButtonKind::Default
    };
    let scene = &mut *ctx.scene;
    let text_system = &mut *ctx.text_system;
    let (_, hit_index) = ctx.host.store_and_hit_index_mut();
    paint_button(
        &Button::new(id, label).kind(kind).visual(state),
        rect,
        scene,
        text_system,
        theme,
    );
    hit_index.register(id, rect);
    y + ROW_H_PX
}

/// Um botão de ação.
pub(super) fn command(
    ctx: &mut PaintCtx,
    id: ph2d_a11y::NodeId,
    label: &str,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    let theme = ctx.host.theme();
    let rect = Rect::new(x, y, w, ROW_H_PX);
    let state = ctx.host.store().button_visual(id);
    let scene = &mut *ctx.scene;
    let text_system = &mut *ctx.text_system;
    let (_, hit_index) = ctx.host.store_and_hit_index_mut();
    paint_button(
        &Button::new(id, label).visual(state),
        rect,
        scene,
        text_system,
        theme,
    );
    hit_index.register(id, rect);
    y + ROW_H_PX
}

/// Uma linha de texto. Hit-indexada por ninguém de propósito — é um FATO, não um
/// controle, e uma affordance que ele não pode honrar seria pior que texto puro.
pub(super) fn readout(ctx: &mut PaintCtx, text: &str, x: f32, w: f32, y: f32) -> f32 {
    let theme = ctx.host.theme();
    let font = TypeToken::Sm.px();
    paint_text(
        ctx.text_system,
        ctx.scene,
        text,
        x,
        y + (ROW_H_PX - font) * 0.5,
        font,
        w,
        resolve(ColorToken::Text2, theme),
    );
    y + ROW_H_PX
}
