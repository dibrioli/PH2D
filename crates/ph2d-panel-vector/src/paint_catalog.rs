//! O **catálogo de formas** do painel Vector: a CATEGORIA numa linha, os TIPOS numa
//! grade de thumbnails.
//!
//! ## O problema que este módulo resolve
//!
//! Antes, o seletor de família (7 botões) e o de forma (48 botões) eram pintados com o
//! **mesmo widget, na mesma altura e na mesma largura** (`paint_segmented_button` numa
//! grade de 3 colunas), um logo abaixo do outro — dez fileiras de botões idênticos numa
//! coluna de 304 px. Categoria e tipo ficavam tipograficamente indistinguíveis ("está
//! tudo junto e confuso", Enio). A separação agora é de **widget**, não de espaçamento:
//!
//! - **Categoria** = um *dropdown chip* (uma linha, com chevron) — outro widget, outra
//!   forma, óbvio à primeira vista que é outro nível.
//! - **Tipo** = uma grade de **thumbnails de verdade**: cada célula pinta a forma cozida
//!   pela MESMA porta que o canvas usa (`vec_scene::cook` → `vec_render::build_bezpath`),
//!   ajustada à célula por um `Affine`. Um grid de 48 NOMES ("Manual in", "Prepare",
//!   "Off-page") é ilegível — nenhum app de desenho faz isso.
//!
//! Módulo irmão de `paint_modes` (teto de 600 LOC por arquivo de painel).
//!
//! ## Gotchas
//!
//! - O popover da categoria **tem** de ser pintado no passe DIFERIDO de `paint.rs` (como
//!   o de fonte): dentro do corpo, o `push_clip` do scroll o cortaria.
//! - Os 48 `BezPath` são cacheados num `thread_local`. `cook` usa transcendentais; rodar
//!   48 formas por frame seria desperdício puro.

use crate::paint_sections::BodyCtx;
use crate::state;
use crate::{ids, state::group_i18n_key};
use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::paint::{paint_text, paint_text_centered, resolve};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::panel_chrome::paint_segmented_button;
use ph2d_editor_core::widget::{
    ButtonState, DROPDOWN_SCROLLBAR_ID, Dropdown, DropdownOption, paint_dropdown_chip,
    paint_dropdown_popover_scrolled, scrollbar_is_needed, scrollbar_track_rect,
};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_tokens::{ColorToken, ICON_BTN_SIZE_PX, Spacing, StrokeToken, Theme, TypeToken};
use ph2d_tool_vector::VectorStyleSnapshot;
use ph2d_tool_vector::params::DrawMode;
use ph2d_tool_vector::shapes::{self, ShapeGroup};
use ph2d_vec_render::build_bezpath;
use ph2d_vec_scene::cook;
use ph2d_vector::{Affine, BezPath, Brush, Color, Fill, Shape, Stroke, VectorScene};
use std::cell::RefCell;

/// A caixa em que cada forma é cozida para o thumbnail: 2×2 centrada na origem. É a mesma
/// ordem de grandeza de um traço real (a viewport toda tem ~10 unidades de mundo), então
/// os defaults que vêm em MUNDO — `DEFAULT_CORNER_RADIUS = 0.12` — saem na proporção que
/// o usuário verá ao desenhar, e não como um canto absurdo ou invisível.
const THUMB_BOX_MIN: [f64; 2] = [-1.0, -1.0];
const THUMB_BOX_MAX: [f64; 2] = [1.0, 1.0];

/// Colunas da grade de tipos. Três cabem em 252 px com rótulo legível ("Subroutine", o
/// pior caso, cabe em 80 px).
const GRID_COLS: usize = 3;

thread_local! {
    /// Os `BezPath` do catálogo, cozidos UMA vez por thread (o `cook` chama seno/cosseno;
    /// 48 formas por frame seria puro desperdício). Indexado pelo índice do catálogo.
    static SHAPE_THUMBS: RefCell<Vec<BezPath>> = const { RefCell::new(Vec::new()) };
}

/// Roda `f` com o `BezPath` da forma `index`, construindo o cache na primeira chamada.
fn with_thumb<R>(index: usize, f: impl FnOnce(&BezPath) -> R) -> Option<R> {
    SHAPE_THUMBS.with(|c| {
        let mut cache = c.borrow_mut();
        if cache.is_empty() {
            cache.extend(shapes::SHAPES.iter().map(|d| {
                build_bezpath(&cook(
                    d.kind,
                    THUMB_BOX_MIN,
                    THUMB_BOX_MAX,
                    &d.kind.defaults(),
                ))
            }));
        }
        cache.get(index).map(f)
    })
}

/// Encaixa o `BezPath` na célula (bbox → escala uniforme + centro) e o pinta.
///
/// Formas **abertas** (Line / Arc / Spiral / Brace / Note) são traçadas, não preenchidas:
/// elas não têm interior, e preenchê-las desenharia o fecho implícito — uma mancha que
/// não existe no canvas. A largura do traço é dividida pela escala porque o `Affine` a
/// multiplicaria de volta: assim o contorno fica constante em PIXELS de tela, não em
/// unidades da forma (senão uma espiral fina e um retângulo gordo teriam traços diferentes).
fn draw_thumb(scene: &mut VectorScene, path: &BezPath, cell: Rect, color: Color, closed: bool) {
    let bbox = path.bounding_box();
    let (bw, bh) = (bbox.width(), bbox.height());
    let (tw, th) = (f64::from(cell.w), f64::from(cell.h));
    // Uma forma degenerada num eixo (linha horizontal: altura zero) escala só pelo outro.
    let scale = match (bw > f64::EPSILON, bh > f64::EPSILON) {
        (true, true) => (tw / bw).min(th / bh),
        (true, false) => tw / bw,
        (false, true) => th / bh,
        (false, false) => 1.0,
    };
    if !scale.is_finite() || scale <= 0.0 {
        return;
    }
    let cx = f64::from(cell.x) + tw * 0.5;
    let cy = f64::from(cell.y) + th * 0.5;
    let bcx = (bbox.x0 + bbox.x1) * 0.5;
    let bcy = (bbox.y0 + bbox.y1) * 0.5;
    let affine = Affine::translate((cx - scale * bcx, cy - scale * bcy)) * Affine::scale(scale);
    let brush = Brush::Solid(color);
    if closed {
        scene
            .inner_mut()
            .fill(Fill::NonZero, affine, &brush, None, path);
    } else {
        let w = f64::from(StrokeToken::Default.px()) / scale;
        scene
            .inner_mut()
            .stroke(&Stroke::new(w), affine, &brush, None, path);
    }
}

impl BodyCtx<'_> {
    /// Seção **SHAPE** — a categoria (dropdown) + a grade de tipos (thumbnails).
    pub(crate) fn shape_catalog_section(&mut self, snap: &VectorStyleSnapshot, y: f32) -> f32 {
        let (mut y, collapsed) = self.section_header(
            ids::VECTOR_SECTION_SHAPE,
            tr("panel.vector.section.shape"),
            y,
        );
        if collapsed {
            return y;
        }
        let group = state::active_shape_group(snap.shape);
        y = self.category_row(group, y);
        self.shape_grid(snap, group, y)
    }

    /// A linha da CATEGORIA: rótulo + chip de dropdown. O widget é deliberadamente OUTRO
    /// (chip com chevron, não botão segmentado) — é essa diferença que separa "categoria"
    /// de "tipo" sem gastar altura nenhuma.
    fn category_row(&mut self, group: ShapeGroup, y: f32) -> f32 {
        let gap = Spacing::Sm.px();
        paint_text(
            self.text_system,
            self.scene,
            tr("panel.vector.category"),
            self.inner_x,
            y + (self.row_h - TypeToken::Sm.px()) * 0.5,
            TypeToken::Sm.px(),
            crate::paint_sections::LABEL_COL_W,
            resolve(ColorToken::Text2, self.theme),
        );
        let chip_x = self.inner_x + crate::paint_sections::LABEL_COL_W + gap;
        let chip_w = (self.inner_w - crate::paint_sections::LABEL_COL_W - gap).max(1.0);
        let chip = Rect::new(chip_x, y, chip_w, self.row_h);
        let open = matches!(
            self.store.get(ids::VECTOR_SHAPE_GROUP_DD),
            Some(InteractiveState::Dropdown { open: true, .. })
        );
        let dd = Dropdown::new(
            ids::VECTOR_SHAPE_GROUP_DD,
            "",
            vec![DropdownOption::new(
                ids::VECTOR_SHAPE_GROUP_DD,
                (),
                tr(group_i18n_key(group)),
            )],
        )
        .selected(())
        .open(open);
        paint_dropdown_chip(&dd, chip, self.scene, self.text_system, self.theme);
        self.hit_index.register(ids::VECTOR_SHAPE_GROUP_DD, chip);
        if open {
            // O popover pinta no passe DIFERIDO (fora do clip do scroll) — `paint.rs`.
            state::set_pending_group_dd(Some(chip));
        }
        y + self.row_h + self.row_gap
    }

    /// A grade de TIPOS da família aberta: plate segmentado (a chrome canônica de
    /// selecionado / hover / press) + o thumbnail da forma + o nome dela.
    fn shape_grid(&mut self, snap: &VectorStyleSnapshot, group: ShapeGroup, y: f32) -> f32 {
        let in_group: Vec<(usize, &shapes::ShapeDesc)> = shapes::SHAPES
            .iter()
            .enumerate()
            .filter(|(_, d)| d.group == group)
            .collect();
        if in_group.is_empty() {
            return y;
        }
        let gap = Spacing::Sm.px();
        let pad = Spacing::Xs.px();
        let thumb_h = ICON_BTN_SIZE_PX + Spacing::Md.px();
        let name_h = TypeToken::Xs.px();
        let cell_h = pad + thumb_h + Spacing::Xxs.px() + name_h + pad;
        let cell_w = ((self.inner_w - gap * (GRID_COLS as f32 - 1.0)) / GRID_COLS as f32).max(1.0);

        for (slot, (cat_index, d)) in in_group.iter().enumerate() {
            let cx = self.inner_x + (slot % GRID_COLS) as f32 * (cell_w + gap);
            let cy = y + (slot / GRID_COLS) as f32 * (cell_h + gap);
            let cell = Rect::new(cx, cy, cell_w, cell_h);
            let id = ids::vector_shape_id(*cat_index);
            // Ativa = a forma DESTA célula é a do gesto armado. Fora do modo Shape nenhuma
            // acende (o usuário não está desenhando forma nenhuma).
            let active = snap.shape == d.kind && snap.mode == DrawMode::Shape;
            let st = self.store.button_state(id).unwrap_or(ButtonState::Normal);
            // Rótulo VAZIO: reaproveitamos a chrome canônica do botão segmentado (borda
            // Accent quando selecionado, BgElev no hover, AccentSoft no press) e desenhamos
            // o conteúdo — thumbnail + nome — por cima. Zero cor inventada.
            paint_segmented_button(
                cell,
                "",
                active,
                st,
                self.scene,
                self.text_system,
                self.theme,
            );
            let ink = resolve(
                if active {
                    ColorToken::Text1
                } else {
                    ColorToken::Text2
                },
                self.theme,
            );
            let thumb = Rect::new(
                cell.x + pad,
                cell.y + pad,
                (cell.w - pad * 2.0).max(1.0),
                thumb_h,
            );
            let closed = d.kind.is_closed();
            with_thumb(*cat_index, |bp| {
                draw_thumb(self.scene, bp, thumb, ink, closed);
            });
            let name = Rect::new(
                cell.x,
                cell.y + pad + thumb_h + Spacing::Xxs.px(),
                cell.w,
                name_h,
            );
            paint_text_centered(self.text_system, self.scene, d.label, name, name_h, ink);
            self.hit_index.register(id, cell);
        }
        let rows = in_group.len().div_ceil(GRID_COLS) as f32;
        y + rows * cell_h + (rows - 1.0) * gap + self.row_gap
    }
}

/// O popover da CATEGORIA — pintado no passe DIFERIDO de `paint.rs`, POR CIMA de todas as
/// seções (dentro do corpo, o `push_clip` do scroll o cortaria).
///
/// As opções reusam os ids que já existiam (`vector_shape_group_id(i)`), já registrados
/// como Button no `populate` e já resolvidos no `event.rs` — o dropdown só trocou a
/// APRESENTAÇÃO da escolha, não o caminho dela.
pub(crate) fn paint_group_popover(ctx: &mut PaintCtx, chip: Rect, theme: Theme) {
    let id = ids::VECTOR_SHAPE_GROUP_DD;
    let groups = shapes::ALL_GROUPS;
    let active = state::active_shape_group(state::current_snapshot().shape);
    let sel = groups.iter().position(|g| *g == active).unwrap_or(0);
    let options: Vec<DropdownOption<usize>> = groups
        .iter()
        .enumerate()
        .map(|(i, g)| DropdownOption::new(ids::vector_shape_group_id(i), i, tr(group_i18n_key(*g))))
        .collect();
    let dd = Dropdown::new(id, "", options).selected(sel).open(true);

    let panel = dd.popover_rect_clamped(chip, ctx.viewport);
    let content_h = dd.content_height(chip.h);
    let visible_h = panel.h;
    let max_scroll = (content_h - visible_h).max(0.0);
    {
        let store = ctx.host.store_mut();
        store.set_dropdown_popover(id, panel);
        store.set_panel_content_h(id, content_h);
        store.set_panel_visible_h(id, visible_h);
        if store.panel_scroll(id) > max_scroll {
            store.set_panel_scroll(id, max_scroll);
        }
    }
    let scroll = ctx.host.store().panel_scroll(id).clamp(0.0, max_scroll); // CLAMP-OK: 0.0 literal; max_scroll is a non-negative px extent
    let scrollbar_active = matches!(ctx.host.store().scrollbar_drag(), Some(d) if d.panel == id);
    paint_dropdown_popover_scrolled(
        &dd,
        chip,
        panel,
        scroll,
        scrollbar_active,
        ctx.scene,
        ctx.text_system,
        theme,
    );

    // Hit-register só a parte VISÍVEL de cada linha (a barra de rolagem é o alvo do drag).
    let hit_index = ctx.host.hit_index_mut();
    for i in 0..groups.len() {
        let r = dd.option_rect_in_scrolled(chip, panel, i, scroll);
        let top = r.y.max(panel.y);
        let bot = (r.y + r.h).min(panel.y + panel.h);
        if bot - top >= 1.0 {
            hit_index.register(
                ids::vector_shape_group_id(i),
                Rect::new(r.x, top, r.w, bot - top),
            );
        }
    }
    if scrollbar_is_needed(content_h, visible_h) {
        ctx.host
            .hit_index_mut()
            .register(DROPDOWN_SCROLLBAR_ID, scrollbar_track_rect(panel));
    }
}
