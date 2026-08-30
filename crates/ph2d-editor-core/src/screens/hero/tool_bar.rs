//! ⭐⭐ **A FILA DE FERRAMENTAS** — os chips do trilho, na horizontal, por cima da área de desenho.
//!
//! Enio, 2026-08-30: *«ainda temos os botões da lateral»* — o trilho vertical saiu, e é aqui que
//! eles voltam, no modelo do Godot: uma fila por cima do canvas, não uma coluna a comer largura.
//!
//! # ⭐ A fila é uma REGIÃO da área, e essa é a diferença que importa
//!
//! Ela sai de [`HeroLayout::tool_bar`], que é cortado da **área de desenho** — entre as colunas —,
//! não da janela. A régua começa por baixo dela, e nenhuma das duas pode tapar a outra porque não
//! partilham coordenada (D5). ⛔ O trilho antigo ancorava em `x = 0` e tapava **86,8 %** da régua
//! da esquerda; uma fila que atravessasse o ecrã faria o mesmo às colunas.
//!
//! # ⚠️ A LISTA é a mesma do trilho, e tem de ser
//!
//! [`super::left_rail::rail_entries`] é a fonte das duas disposições. Uma segunda lista aqui seria
//! a tabela paralela: um verbo novo apareceria numa e não na outra, conforme quem se lembrasse —
//! e o gate anti-botão-morto (`every_painted_rail_button_is_dispatched`) percorre aquela.
//!
//! # ⚠️ E a GEOMETRIA é a mesma porta
//!
//! [`crate::widget::entry_rects`] responde *«onde cai cada entrada?»* nos dois eixos, e é ela que
//! o pintor e o registo de hit perguntam. Enquanto ela não existia, a mesma aritmética estava
//! escrita **três** vezes — e um pintor horizontal com um hit vertical compilaria e passaria a
//! suíte inteira.

use super::HeroLayout;
use super::ids;
use super::left_rail::{PAINTER_MASK_SUBS, PAINTER_SHAPES, rail_entries, tool_entry};
use crate::interaction::{HitIndex, WidgetStore};
use crate::paint::{fill_rounded_rect, resolve};
use crate::widget::{
    LABEL_TO_CHIP_GAP_PX, LABEL_VISUAL_EXTENT_PX, RailAxis, RailButtonSize, ToolRail, entry_rects,
    paint_tool_rail_axis,
};
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, Theme};
use ph2d_vector::VectorScene;

/// **A altura da fila** — o rótulo, a folga, o chip, e o respiro em cima e em baixo.
///
/// ⚠️ **Derivada, não escolhida**: cada parcela é a mesma constante que a coluna usa para a mesma
/// coisa. Um número aqui faria a fila e a coluna terem chips de tamanhos diferentes no dia em que
/// alguém mexesse no preset.
#[must_use]
pub fn tool_bar_h(size: RailButtonSize) -> f32 {
    Spacing::Xxs.px() * 2.0 + LABEL_VISUAL_EXTENT_PX + LABEL_TO_CHIP_GAP_PX + size.chip_px()
}

/// **O rectângulo em que os chips de facto correm** — a faixa menos o respiro.
///
/// ⚠️ **Uma função, e não uma conta inline no pintor.** Ela é a origem que a porta
/// ([`crate::widget::entry_rects`]) recebe, logo quem quiser saber onde um chip caiu — um gate, um
/// flyout, a próxima wave — tem de fazer a MESMA conta. Uma segunda cópia dela seria o espelho que
/// esta wave inteira existiu para apagar.
#[must_use]
pub fn content_rect(bar: Rect) -> Rect {
    Rect::new(
        bar.x + Spacing::Xs.px(),
        bar.y + Spacing::Xxs.px(),
        (bar.w - Spacing::Xs.px() * 2.0).max(0.0),
        (bar.h - Spacing::Xxs.px() * 2.0).max(0.0),
    )
}

/// O rail da fila — a MESMA lista que a coluna pinta.
fn bar_rail(store: &WidgetStore, painter_active: bool) -> ToolRail {
    ToolRail::new(
        NodeId(203),
        "Editor tools",
        rail_entries(store, painter_active),
    )
}

/// Desenha a fila e regista os alvos.
#[allow(clippy::too_many_arguments)] // o relógio é o 8º, como no irmão vertical
pub fn paint_tool_bar(
    layout: &HeroLayout,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    painter_active: bool,
    motion: &crate::motion::UiMotion,
) {
    let bar = layout.tool_bar;
    if bar.w <= 0.0 || bar.h <= 0.0 {
        return;
    }
    let rail = bar_rail(store, painter_active);
    let size = store.rail_button_size();
    // A faixa inteira leva o fundo do trilho — é o mesmo chrome, deitado.
    scene.fill_rect(
        crate::paint::rect_to_vello(bar),
        resolve(ColorToken::RailBg, theme),
    );
    let content = content_rect(bar);
    // ⚠️ **A fila é CORTADA pela própria faixa, no desenho E no hit.** Numa janela estreita (ou
    // com as duas colunas abertas) os chips passam do fim da área — e sem a blindagem eles
    // pintariam por cima da coluna da direita e continuariam **clicáveis lá**. A tinta e o dedo
    // recebem a mesma banda: cortar só um deixaria um alvo invisível, que é pior que um chip
    // cortado. (O `HitIndex::push_clip` já existia; era o painel de nós que o pedia.)
    let clip = ph2d_vector::Rect::new(
        bar.x as f64,
        bar.y as f64,
        (bar.x + bar.w) as f64,
        (bar.y + bar.h) as f64,
    );
    scene.push_clip(&clip);
    hit_index.push_clip(bar);
    paint_tool_rail_axis(
        &rail,
        content,
        scene,
        text_system,
        theme,
        store,
        &|id| Some(motion.get(id).unwrap_or(0.0)),
        motion.travels(),
        RailAxis::Horizontal,
    );
    let mut shapes_chip: Option<Rect> = None;
    let mut mask_group_chip: Option<Rect> = None;
    for slot in entry_rects(&rail, content, size, RailAxis::Horizontal) {
        let Some(id) = slot.id else {
            continue; // o divisor não se clica
        };
        hit_index.register(id, slot.rect);
        if id == ids::PAINTER_RAIL_SHAPES {
            shapes_chip = Some(slot.rect);
        } else if id == ids::PAINTER_RAIL_MASK_GROUP {
            mask_group_chip = Some(slot.rect);
        }
    }
    scene.pop_layer();
    hit_index.pop_clip();
    // Os dois flyouts de grupo (só em modo Painter). ⚠️ **Fora da blindagem, de propósito**: eles
    // caem POR BAIXO da faixa, e cortá-los pela faixa apagava-os inteiros. ⚠️ **Eles caem PARA BAIXO**, não para o lado:
    // numa fila horizontal o vizinho da direita é outro verbo, e um flyout lateral cobri-lo-ia.
    if painter_active
        && store.painter_shapes_flyout_open()
        && let Some(anchor) = shapes_chip
    {
        paint_flyout_below(
            anchor,
            scene,
            text_system,
            theme,
            hit_index,
            store,
            &PAINTER_SHAPES,
            NodeId(204),
            "Shape options",
            motion,
        );
    }
    if painter_active
        && store.painter_mask_flyout_open()
        && let Some(anchor) = mask_group_chip
    {
        paint_flyout_below(
            anchor,
            scene,
            text_system,
            theme,
            hit_index,
            store,
            &PAINTER_MASK_SUBS,
            NodeId(205),
            "Mask options",
            motion,
        );
    }
}

/// O flyout de um chip de grupo, **por baixo** dele — uma mini-coluna, com a geometria da mesma
/// porta.
#[allow(clippy::too_many_arguments)]
fn paint_flyout_below(
    anchor: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    subs: &[(NodeId, &str, crate::icons::IconId, &str)],
    rail_id: NodeId,
    a11y: &str,
    motion: &crate::motion::UiMotion,
) {
    let entries = subs.iter().map(|t| tool_entry(store, *t)).collect();
    let rail = ToolRail::new(rail_id, a11y, entries);
    let size = store.rail_button_size();
    let flyout = Rect::new(
        // ⚠️ O `CHIP_X_OFFSET_PX` que a coluna reserva para o rótulo rodado desloca o chip para a
        // direita; recuá-lo aqui mantém os chips do flyout alinhados com o chip que os abriu.
        anchor.x - crate::widget::CHIP_X_OFFSET_PX,
        anchor.y + anchor.h + Spacing::Xs.px(),
        size.rail_width_px(),
        rail.preferred_height(size),
    );
    let bg = Rect::new(
        flyout.x,
        flyout.y - Spacing::Sm.px(),
        flyout.w,
        flyout.h + Spacing::Sm.px() * 2.0,
    );
    fill_rounded_rect(
        scene,
        bg,
        Radius::Md.px(),
        resolve(ColorToken::RailBg, theme),
    );
    paint_tool_rail_axis(
        &rail,
        flyout,
        scene,
        text_system,
        theme,
        store,
        &|id| Some(motion.get(id).unwrap_or(0.0)),
        motion.travels(),
        RailAxis::Vertical,
    );
    for slot in entry_rects(&rail, flyout, size, RailAxis::Vertical) {
        if let Some(id) = slot.id {
            hit_index.register(id, slot.rect);
        }
    }
}
