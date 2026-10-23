//! Onde fica cada controlo das barras do quadro: a barra curta, a grelha das formas e a barra de
//! estilo por cima da selecção (módulo filho de [`super`]).

use super::{Item, Selected, TOOLBAR, TOOLBAR_TAIL, style_groups};
use crate::zones::Rect;
use ph2d_board_model::ShapeType;
use ph2d_tokens::Spacing;

// ⭐ A GEOMETRIA e o BOTÃO são do widget genérico ([`super::super::float_bar`]) — o Vetor pinta a
// coluna e a barra da seleção dele pela MESMA conta. Aqui fica só a tabela do Quadro.
pub(super) use super::super::float_bar::{button_px as btn_px, gap};

/// O rectângulo de cada atalho da barra curta, de cima para baixo, encostado à esquerda de `area`.
#[must_use]
///
/// Numa janela baixa a coluna parte-se em colunas lado a lado, de alturas iguais (W23 A6): nenhum
/// botão sai da área. Com lugar, é a coluna do [`super::super::float_bar::column_rects`].
pub fn toolbar_rects(area: Rect) -> Vec<(Item, Rect)> {
    let n = TOOLBAR.len() + TOOLBAR_TAIL.len();
    let (s, g, m) = (btn_px(), gap(), Spacing::Md.px());
    let fits = (((area.h - 2.0 * m + g) / (s + g)).floor().max(1.0)) as usize;
    let rows = n.div_ceil(n.div_ceil(fits));
    TOOLBAR
        .iter()
        .chain(&TOOLBAR_TAIL)
        .copied()
        .enumerate()
        .map(|(i, it)| {
            let (c, r) = ((i / rows) as f32, (i % rows) as f32);
            (
                it,
                Rect::new(area.x + m + c * (s + g), area.y + m + r * (s + g), s, s),
            )
        })
        .collect()
}

/// O botão `item` da barra visto pelos painéis que abrem À DIREITA dele: na última coluna da barra
/// (um painel aberto de um botão da 1.ª coluna não tapa a 2.ª).
pub(super) fn anchor(area: Rect, item: Item) -> Option<Rect> {
    let rects = toolbar_rects(area);
    let right = rects.iter().map(|(_, r)| r.x).fold(f32::MIN, f32::max);
    rects
        .into_iter()
        .find(|(it, _)| *it == item)
        .map(|(_, r)| Rect::new(right, r.y, r.w, r.h))
}

/// A grelha de todas as formas, à direita do botão «mais formas» (três colunas).
#[must_use]
pub fn shapes_rects(area: Rect) -> Vec<(Item, Rect)> {
    let more = anchor(area, Item::MoreShapes).expect("a barra tem o «mais formas»");
    let grid = super::super::float_bar::grid_rects(area, more, ShapeType::ALL.len(), 3);
    ShapeType::ALL
        .iter()
        .map(|t| Item::Pick(*t))
        .zip(grid)
        .collect()
}

/// A barra de estilo por cima de `sel` (o rectângulo da selecção no ecrã), presa dentro de `area`;
/// por baixo da selecção quando não cabe por cima.
#[must_use]
pub fn style_rects(area: Rect, sel: Rect, what: Selected) -> Vec<(Item, Rect)> {
    style_rects_of(area, sel, style_groups(what), &[])
}

/// O mesmo, para os grupos que a selecção de AGORA pinta (sem os controlos que não teriam efeito).
/// Fica à direita da barra curta, fora de todo o chrome de `keep` (gavetas, painéis, avisos — W24) e,
/// sem lugar numa linha, parte-se em linhas (a da forma, com a letra e o Organizar, passava da área a
/// 1366 — foto da cena 28, 22/10).
#[must_use]
pub fn style_rects_of(
    area: Rect,
    sel: Rect,
    groups: Vec<Vec<Item>>,
    keep: &[Rect],
) -> Vec<(Item, Rect)> {
    if groups.is_empty() {
        return Vec::new();
    }
    let left = toolbar_rects(area)
        .iter()
        .map(|(_, r)| r.x + r.w)
        .fold(area.x, f32::max)
        + Spacing::Md.px();
    let room = Rect::new(left, area.y, (area.x + area.w - left).max(0.0), area.h);
    let sizes: Vec<usize> = groups.iter().map(Vec::len).collect();
    groups
        .into_iter()
        .flatten()
        .zip(super::super::float_bar::strip_rects_clear_of(
            room,
            sel,
            &sizes,
            super::super::float_bar::strip_button_px(),
            Spacing::Lg.px(),
            keep,
        ))
        .collect()
}

/// ⭐ Pinta a barra curta do quadro activo e as gavetas abertas — a 1.ª metade das barras (W25): o aviso do
/// topo e a barra do vídeo pintam-se a seguir, por cima delas e fora delas, e a barra de estilo por último
/// ([`super::paint`]). `area` é a área do quadro no ecrã.
pub fn paint_tools(
    hero: &mut super::HeroScreen,
    area: Rect,
    scene: &mut ph2d_vector::VectorScene,
    text_system: &mut ph2d_text::TextSystem,
) {
    use super::{fac, mind, notes, paint_item, pen, shapes};
    use ph2d_board_edit::Tool;
    #[cfg(test)]
    super::PAINTED_ON.with(|v| v.borrow_mut().clear());
    let theme = hero.theme;
    let shapes_open = hero.documents.live.shapes_open;
    let templates_open = mind::sync_gallery(&mut hero.documents.live);
    let fac_now = fac::Now::of(hero);
    let Some((board, live)) = hero.documents.active_parts() else {
        return;
    };
    let ed = super::super::board_view::editor(&mut live.editor, theme);
    let tool = ed.tool;
    let now = notes::Now::of(ed, &board.doc);
    let tools = toolbar_rects(area);
    super::paint_panel(scene, &tools, theme);
    let note_tool = Item::Tool(Tool::Shape(ShapeType::Sticky));
    let sketch_on = super::look::sketch_on(ed, &board.doc, board.sketch);
    let (hit, store) = (&mut hero.hit_index, &hero.store);
    for (it, r) in tools {
        let on = matches!(it, Item::Tool(t) if t == tool)
            || (it == Item::MoreShapes && shapes_open)
            || (it == note_tool && notes::flyout_open(tool))
            || (it == pen::PEN_TOOL && pen::flyout_open(tool))
            || (it == Item::Sketch && sketch_on)
            || (it == super::organize::FRAME_TOOL && tool == Tool::Frame)
            || mind::is_on(it, ed, &board.doc, templates_open) == Some(true)
            || fac_now.is_on(it) == Some(true);
        paint_item(scene, text_system, theme, hit, store, it, r, on);
    }
    if shapes_open {
        shapes::paint_tool_grid(scene, text_system, theme, hit, store, area, tool);
    }
    if notes::flyout_open(tool) {
        notes::paint_flyout(scene, text_system, theme, hit, store, area, tool, &now);
    }
    if pen::flyout_open(tool) {
        pen::paint_flyout(scene, text_system, theme, hit, store, area, ed);
    }
    if templates_open {
        mind::paint_gallery(scene, text_system, theme, hit, store, area);
    }
    float_drawers(&mut live.nav, area, tool, [shapes_open, templates_open]);
}

/// ⭐ W25 — onde abre um menu da barra de estilo (o «Forma», as pontas, os carimbos, o agrupar): a área, a
/// barra pintada e o chrome que ele não tapa.
pub(super) struct MenuRoom<'a> {
    pub(super) area: Rect,
    pub(super) bar: &'a [(Item, Rect)],
    pub(super) keep: &'a [Rect],
}

impl MenuRoom<'_> {
    /// O canto de cima de um menu `w×h` aberto pelo botão `button`: por baixo da barra INTEIRA (aberto por
    /// baixo do botão tapava as outras linhas dela), senão por cima, a começar no botão e fora do chrome;
    /// sem lugar livre, rente ao botão como antes.
    pub(super) fn origin(&self, button: Rect, (w, h): (f32, f32)) -> (f32, f32) {
        let fb = super::super::float_bar::panel_of(self.bar.iter().map(|(_, r)| *r));
        let bar = fb.unwrap_or(button);
        let pad = Spacing::Sm.px();
        let ys = [bar.y + bar.h + pad, bar.y - pad - h];
        let clear = super::super::float_bar::rect_clear_of(
            self.area,
            self.keep,
            (w, h),
            &ys,
            button.x + w / 2.0,
        );
        if let Some(r) = clear {
            return (r.x, r.y);
        }
        let below = button.y + button.h + pad;
        let y = if below + h <= self.area.y + self.area.h {
            below
        } else {
            (button.y - pad - h).max(self.area.y)
        };
        let x = button.x.min(self.area.x + self.area.w - w).max(self.area.x);
        (x, y)
    }
}

/// W24 — as gavetas abertas (formas, notas, caneta, modelos) entram no chrome que a barra de estilo, a
/// última a pintar, não tapa.
pub(super) fn float_drawers(
    nav: &mut crate::documents::nav::Nav,
    area: Rect,
    tool: ph2d_board_edit::Tool,
    [shapes_open, templates_open]: [bool; 2],
) {
    let drawers = [
        (shapes_open, shapes_rects(area)),
        (
            super::notes::flyout_open(tool),
            super::notes::flyout_rects(area),
        ),
        (
            super::pen::flyout_open(tool),
            super::pen::flyout_rects(area),
        ),
        (templates_open, super::mind::gallery_rects(area)),
    ];
    for (_, rects) in drawers.into_iter().filter(|(open, _)| *open) {
        let panel = super::super::float_bar::panel_of(rects.into_iter().map(|(_, r)| r));
        panel
            .into_iter()
            .for_each(|r| super::super::board_nav::fit::float(nav, r));
    }
}
