//! O corpo rolado: a pilha de seções dobráveis.
//!
//! Irmão do `paint.rs` porque o cap de LOC de painel é 600 e as duas metades
//! crescem por motivos diferentes (chrome × conteúdo).
//!
//! **A ordem das seções é a ordem em que a mão as procura:** a FERRAMENTA
//! primeiro (é o que se troca a cada minuto), o PINCEL logo abaixo (os knobs da
//! ferramenta em mãos), o ESPELHO, a TOPOLOGIA (a resolução do barro), o
//! SOMBREAMENTO (como a forma é lida) e a CENA por último — que é a ordem do
//! SculptGL, e por um motivo que se verifica: quanto mais raro o gesto, mais
//! fundo ele pode estar.

use ph2d_editor_core::ids;
use ph2d_editor_core::paint::{paint_text, resolve};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::{
    Button, ButtonKind, ButtonState, SectionHeader, SegmentedAdaptive, SegmentedOption,
    paint_button, paint_section_header, paint_segmented_adaptive,
};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_sculpt3d::{Falloff, Verb};
use ph2d_tokens::{ColorToken, ROW_H_PX, Spacing, TypeToken};

use crate::rows;
use crate::state::Sculpt3dSnapshot;

/// Os rótulos dos três degraus de detalhe, na ordem do `DETAIL_STEPS` do shell.
const DETAIL_LABELS: [&str; 3] = [
    "panel.sculpt3d.detail.coarse",
    "panel.sculpt3d.detail.medium",
    "panel.sculpt3d.detail.fine",
];

/// Os rótulos das quatro primitivas, na ordem dos comandos `Add*`.
const ADD_LABELS: [&str; 4] = [
    "panel.sculpt3d.add.sphere",
    "panel.sculpt3d.add.cube",
    "panel.sculpt3d.add.cylinder",
    "panel.sculpt3d.add.torus",
];

/// Os rótulos das quatro operações de máscara, na ordem dos comandos `Mask*`.
const MASK_LABELS: [&str; 4] = [
    "panel.sculpt3d.mask.clear",
    "panel.sculpt3d.mask.invert",
    "panel.sculpt3d.mask.blur",
    "panel.sculpt3d.mask.sharpen",
];

/// Pinta todas as seções. Devolve o `y` em que terminou.
///
/// ⚠️ **Uma chamada por seção, e não um corpo só.** Ele já cruzou o cap de 200
/// LOC de `fn` uma vez, e o corte que o gate pediu é o mesmo que a leitura pede:
/// cada seção é um ASSUNTO, e o orquestrador aqui é a ORDEM em que a mão os
/// procura.
pub(super) fn paint_sections(
    ctx: &mut PaintCtx,
    snap: &Sculpt3dSnapshot,
    x: f32,
    w: f32,
    y_in: f32,
) -> f32 {
    let mut y = paint_tool(ctx, snap, x, w, y_in);
    y = knob_section(ctx, snap, &rows::SECTIONS[0], x, w, y, paint_brush_tail);
    y = paint_symmetry(ctx, snap, x, w, y);
    y = paint_topology(ctx, snap, x, w, y);
    y = knob_section(ctx, snap, &rows::SECTIONS[1], x, w, y, |_, _, _, _, y| y);
    paint_scene(ctx, snap, x, w, y)
}

/// **A FERRAMENTA** — os dezesseis verbos numa faixa que REFLUI.
///
/// É a mesma decisão que a lista de dez ferramentas do Impasto tomou: um grupo
/// segmentado com muitas opções quebra em linhas, e a alternativa (um dropdown)
/// esconde quinze ferramentas atrás de um clique para mostrar uma.
fn paint_tool(ctx: &mut PaintCtx, snap: &Sculpt3dSnapshot, x: f32, w: f32, y: f32) -> f32 {
    let (open, mut y) = header(
        ctx,
        ids::SCULPT3D_SEC_TOOL,
        tr("panel.sculpt3d.section.tool"),
        x,
        w,
        y,
    );
    if !open {
        return y;
    }
    let selected = Verb::ALL
        .iter()
        .position(|&v| v == snap.ui.brush.verb)
        .unwrap_or(0);
    let labels: Vec<&str> = Verb::ALL.iter().map(|v| v.label()).collect();
    y = seg(
        ctx,
        ids::SCULPT3D_SEC_TOOL,
        &ids::SCULPT3D_VERB,
        &labels,
        selected,
        x,
        w,
        y,
    );
    y + Spacing::Md.px()
}

/// O que vem DEPOIS dos knobs do pincel: a curva e as operações de máscara.
fn paint_brush_tail(ctx: &mut PaintCtx, snap: &Sculpt3dSnapshot, x: f32, w: f32, y: f32) -> f32 {
    // O falloff logo abaixo dos knobs: ele é a FORMA do peso, e a força é
    // quanto dele se aplica.
    let selected = Falloff::ALL
        .iter()
        .position(|&f| f == snap.ui.brush.falloff)
        .unwrap_or(0);
    let labels: Vec<&str> = Falloff::ALL.iter().map(|f| f.label()).collect();
    let y = labelled_seg(
        ctx,
        tr("panel.sculpt3d.falloff"),
        ids::SCULPT3D_SEC_BRUSH,
        &ids::SCULPT3D_FALLOFF,
        &labels,
        selected,
        x,
        w,
        y,
    );
    // As quatro operações de máscara moram AQUI, ao lado do verbo que a pinta —
    // um artista que acabou de pintar máscara procura o que fazer com ela onde
    // ele a pintou, não numa seção própria três rolagens abaixo.
    let mask: Vec<&str> = MASK_LABELS.iter().map(|k| tr(k)).collect();
    labelled_seg(
        ctx,
        tr("panel.sculpt3d.mask"),
        ids::SCULPT3D_SEC_BRUSH,
        &ids::SCULPT3D_MASK_OP,
        &mask,
        // ⚠️ Nenhum fica aceso, e `usize::MAX` é como se diz isso: as quatro são
        // GESTOS (executam e acabam), não um modo escolhido. Acender uma delas
        // afirmaria um estado que não existe.
        usize::MAX,
        x,
        w,
        y,
    )
}

/// **O ESPELHO** — três botões INDEPENDENTES.
///
/// Não é um rádio: um segmented é *um de N* por construção, e o ZBrush espelha
/// em dois eixos ao mesmo tempo.
fn paint_symmetry(ctx: &mut PaintCtx, snap: &Sculpt3dSnapshot, x: f32, w: f32, y: f32) -> f32 {
    let gap = Spacing::Sm.px();
    let (open, y) = header(
        ctx,
        ids::SCULPT3D_SEC_SYMMETRY,
        tr("panel.sculpt3d.section.symmetry"),
        x,
        w,
        y,
    );
    if !open {
        return y;
    }
    let third = (w - gap * 2.0) / 3.0; // LITERAL-PX-OK: sao TRES eixos de espelho, nao uma metrica
    for (i, (id, key, on)) in [
        (
            ids::SCULPT3D_SYM_X,
            "panel.sculpt3d.sym.x",
            snap.ui.symmetry.x,
        ),
        (
            ids::SCULPT3D_SYM_Y,
            "panel.sculpt3d.sym.y",
            snap.ui.symmetry.y,
        ),
        (
            ids::SCULPT3D_SYM_Z,
            "panel.sculpt3d.sym.z",
            snap.ui.symmetry.z,
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let bx = (third + gap).mul_add(i as f32, x);
        toggle(ctx, id, tr(key), on, bx, third, y);
    }
    y + ROW_H_PX + Spacing::Md.px()
}

/// **A TOPOLOGIA** — a resolução do barro.
fn paint_topology(ctx: &mut PaintCtx, snap: &Sculpt3dSnapshot, x: f32, w: f32, y: f32) -> f32 {
    let gap = Spacing::Sm.px();
    let (open, mut y) = header(
        ctx,
        ids::SCULPT3D_SEC_TOPOLOGY,
        tr("panel.sculpt3d.section.topology"),
        x,
        w,
        y,
    );
    if !open {
        return y;
    }
    y = toggle(
        ctx,
        ids::SCULPT3D_DYNTOPO,
        tr("panel.sculpt3d.dyntopo"),
        snap.dyntopo,
        x,
        w,
        y,
    ) + gap;
    let detail: Vec<&str> = DETAIL_LABELS.iter().map(|k| tr(k)).collect();
    y = labelled_seg(
        ctx,
        tr("panel.sculpt3d.detail"),
        ids::SCULPT3D_SEC_TOPOLOGY,
        &ids::SCULPT3D_DETAIL,
        &detail,
        snap.ui.detail as usize,
        x,
        w,
        y,
    );
    // O nível vivo é um FATO, e ele fica entre os dois botões que o movem — sem
    // ele, descer e subir são dois botões que não dizem onde você está (a malha
    // de baixo se PARECE com a de cima alisada).
    y = readout(
        ctx,
        &format!(
            "{}: {} / {}",
            tr("panel.sculpt3d.level"),
            snap.level,
            snap.level_count.saturating_sub(1)
        ),
        x,
        w,
        y,
    );
    y = row_of_two(
        ctx,
        (ids::SCULPT3D_LEVEL_DOWN, "-"),
        (ids::SCULPT3D_LEVEL_UP, "+"),
        x,
        w,
        y,
    ) + gap;
    y = row_of_two(
        ctx,
        (ids::SCULPT3D_SUBDIVIDE, tr("panel.sculpt3d.subdivide")),
        (ids::SCULPT3D_REVERSE, tr("panel.sculpt3d.reverse")),
        x,
        w,
        y,
    ) + gap;
    row_of_two(
        ctx,
        (ids::SCULPT3D_REMESH, tr("panel.sculpt3d.remesh")),
        (ids::SCULPT3D_CLOSE_HOLES, tr("panel.sculpt3d.close_holes")),
        x,
        w,
        y,
    ) + Spacing::Md.px()
}

/// **A CENA** — a lista de peças e os verbos que a mexem.
fn paint_scene(ctx: &mut PaintCtx, snap: &Sculpt3dSnapshot, x: f32, w: f32, y: f32) -> f32 {
    let gap = Spacing::Sm.px();
    let (open, mut y) = header(
        ctx,
        ids::SCULPT3D_SEC_SCENE,
        tr("panel.sculpt3d.section.scene"),
        x,
        w,
        y,
    );
    if !open {
        return y;
    }
    let add: Vec<&str> = ADD_LABELS.iter().map(|k| tr(k)).collect();
    y = labelled_seg(
        ctx,
        tr("panel.sculpt3d.add"),
        ids::SCULPT3D_SEC_SCENE,
        &ids::SCULPT3D_ADD,
        &add,
        usize::MAX, // gestos, não um modo
        x,
        w,
        y,
    );
    y = row_of_two(
        ctx,
        (ids::SCULPT3D_DUPLICATE, tr("panel.sculpt3d.duplicate")),
        (ids::SCULPT3D_DELETE, tr("panel.sculpt3d.delete")),
        x,
        w,
        y,
    ) + gap;
    // O Isolate é o único desta fileira com ESTADO — ele fica aceso enquanto a
    // cena está reduzida a uma peça, senão o artista perde quatro objetos e não
    // tem na tela nada que explique por quê.
    let half = (w - gap) * 0.5;
    toggle(
        ctx,
        ids::SCULPT3D_ISOLATE,
        tr("panel.sculpt3d.isolate"),
        snap.isolated,
        x,
        half,
        y,
    );
    y = command(
        ctx,
        ids::SCULPT3D_MERGE,
        tr("panel.sculpt3d.merge"),
        x + half + gap,
        half,
        y,
    ) + gap;
    y = readout(
        ctx,
        &format!(
            "{}: {}   {}: {}",
            tr("panel.sculpt3d.pieces"),
            snap.pieces,
            tr("panel.sculpt3d.verts"),
            snap.verts
        ),
        x,
        w,
        y,
    );
    y + gap
}

/// Uma seção de knobs da tabela, com um sufixo opcional (o falloff, a máscara).
fn knob_section(
    ctx: &mut PaintCtx,
    snap: &Sculpt3dSnapshot,
    section: &rows::Section,
    x: f32,
    w: f32,
    y_in: f32,
    tail: impl Fn(&mut PaintCtx, &Sculpt3dSnapshot, f32, f32, f32) -> f32,
) -> f32 {
    let (open, mut y) = header(ctx, section.id, tr(section.title), x, w, y_in);
    if !open {
        return y;
    }
    for row in section.rows {
        // ⚠️ A row condicional é PULADA, não desenhada apagada: um controle
        // apagado que ainda despacha mente, e um que não despacha é a affordance
        // morta que esta casa varre.
        if !(row.show)(&snap.ui) {
            continue;
        }
        let value = (row.get)(&snap.ui);
        let used = super::paint_row(ctx, row, value, x, w, y);
        y += used + Spacing::Sm.px();
    }
    y = tail(ctx, snap, x, w, y);
    y + Spacing::Md.px()
}

/// Um cabeçalho dobrável. Devolve `(está_aberto, y_depois)`.
fn header(
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
    let head = SectionHeader::new(id, title).collapsible(!collapsed);
    let scene = &mut *ctx.scene;
    let text_system = &mut *ctx.text_system;
    let (_, hit_index) = ctx.host.store_and_hit_index_mut();
    paint_section_header(&head, rect, scene, text_system, theme);
    hit_index.register(id, rect);
    (!collapsed, y + h + Spacing::Sm.px())
}

/// Um grupo segmentado sem rótulo (a lista de ferramentas).
#[allow(clippy::too_many_arguments)]
fn seg(
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
fn labelled_seg(
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
fn row_of_two(
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
fn toggle(
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
        ButtonState::Pressed
    } else {
        ctx.host
            .store()
            .button_state(id)
            .unwrap_or(ButtonState::Normal)
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
        &Button::new(id, label).kind(kind).state(state),
        rect,
        scene,
        text_system,
        theme,
    );
    hit_index.register(id, rect);
    y + ROW_H_PX
}

/// Um botão de ação.
fn command(ctx: &mut PaintCtx, id: ph2d_a11y::NodeId, label: &str, x: f32, w: f32, y: f32) -> f32 {
    let theme = ctx.host.theme();
    let rect = Rect::new(x, y, w, ROW_H_PX);
    let state = ctx
        .host
        .store()
        .button_state(id)
        .unwrap_or(ButtonState::Normal);
    let scene = &mut *ctx.scene;
    let text_system = &mut *ctx.text_system;
    let (_, hit_index) = ctx.host.store_and_hit_index_mut();
    paint_button(
        &Button::new(id, label).state(state),
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
fn readout(ctx: &mut PaintCtx, text: &str, x: f32, w: f32, y: f32) -> f32 {
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
