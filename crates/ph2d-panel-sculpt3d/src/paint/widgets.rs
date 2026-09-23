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
    Button, ButtonKind, ButtonState, SectionFold, SectionHeader, paint_button, paint_section_header,
};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ColorToken, ROW_H_PX, Spacing, TypeToken};

/// Um cabeçalho dobrável. Devolve `(a_dobra, y_depois)` — `None` quando a secção está fechada
/// **e parada**, que é o único caso em que o corpo não é pintado.
///
/// ⚠️ **`Option<SectionFold>` e não o `bool` de antes**, e a diferença é o que a F4b entrega: o
/// `bool` vinha do `is_collapsed`, que vira no quadro do clique enquanto o `t` ainda desce — um
/// corpo gateado nele sumiria de repente por baixo de um chevron a rodar. Quem devolve a dobra
/// devolve também o escopo que a fecha, então esquecer o `finish` deixou de ser possível sem o
/// compilador (ou o `Drop` em debug) reclamar.
pub(super) fn header(
    ctx: &mut PaintCtx,
    id: ph2d_a11y::NodeId,
    title: &str,
    x: f32,
    w: f32,
    y: f32,
) -> (Option<SectionFold>, f32) {
    let theme = ctx.host.theme();
    let h = TypeToken::Md.px() + Spacing::Md.px(); // LITERAL-PX-OK: altura da faixa de cabeçalho
    let collapsed = ctx.host.store().is_collapsed(id);
    let rect = Rect::new(x, y, w, h);
    let head = SectionHeader::new(id, title)
        .collapsible(!collapsed)
        .open_t(ctx.host.store().section_open_live(id));
    let body_top = y + h + Spacing::Sm.px();
    let scene = &mut *ctx.scene;
    let text_system = &mut *ctx.text_system;
    let (store, hit_index) = ctx.host.store_and_hit_index_mut();
    paint_section_header(&head, rect, scene, text_system, theme);
    hit_index.register(id, rect);
    let fold = SectionFold::begin(store, id, x, w, body_top, scene, hit_index);
    (fold, body_top)
}

/// Fecha a dobra aberta pelo [`header`] e devolve o `y` de saída.
///
/// ⚠️ Existe porque o `finish` quer `&WidgetStore`, `&mut VectorScene` e `&mut HitIndex` ao mesmo
/// tempo, e num `PaintCtx` os três saem de campos disjuntos — a mesma dança que o `header` faz.
pub(super) fn end_fold(ctx: &mut PaintCtx, fold: SectionFold, y: f32) -> f32 {
    let scene = &mut *ctx.scene;
    let (store, hit_index) = ctx.host.store_and_hit_index_mut();
    fold.finish(store, scene, hit_index, y)
}

/// Uma escolha com nome — **pela porta da ESCOLHA** ([`ph2d_editor_core::property_row::paint_choice_row`]).
///
/// ⛔⛔ Até 2026-09-23 ela pintava o nome POR CIMA numa faixa `Sm + Md` e o grupo a toda a
/// largura: a varredura geométrica `nenhum_nome_por_cima_do_controlo` acusava onze escolhas deste
/// painel. Hoje o nome fica AO LADO quando o grupo cabe numa fileira da coluna do valor, e só vira
/// PALETA quando não cabe (a curva, o matcap, o alpha, a lista de filtros — dez a doze opções).
///
/// ⚠️ A coluna é a de omissão (`Seccao::apenas_campos(1)`), a MESMA `property_label_col_w` das
/// linhas deste painel. ⚠️ O ritmo é o do painel: a porta fecha com o vão da casa e esta família
/// separava a escolha da linha seguinte por um `Sm` — troca-se um pelo outro. ⚠️ O id do GRUPO é
/// da acessibilidade (o `populate` regista-o) e nunca entrou na pintura.
#[allow(clippy::too_many_arguments)]
pub(super) fn labelled_seg(
    ctx: &mut PaintCtx,
    label: &str,
    options: &[ph2d_a11y::NodeId],
    labels: &[&str],
    selected: usize,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    let theme = ctx.host.theme();
    let segs: Vec<(&str, bool, ph2d_a11y::NodeId)> = options
        .iter()
        .zip(labels)
        .enumerate()
        .map(|(i, (&id, &l))| (l, i == selected, id))
        .collect();
    let scene = &mut *ctx.scene;
    let text_system = &mut *ctx.text_system;
    let (store, hit_index) = ctx.host.store_and_hit_index_mut();
    let fim = ph2d_editor_core::property_row::paint_choice_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        y,
        label,
        &segs,
        ph2d_editor_core::widget::Seccao::apenas_campos(1),
    );
    fim - ph2d_tokens::control_gap_px() + Spacing::Sm.px()
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
    let gap = Spacing::Xs.px();
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
