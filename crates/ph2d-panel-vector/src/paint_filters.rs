//! A seção **FILTERS** do painel Vector — a PILHA de FX raster do caminho selecionado (plano 24).
//!
//! Módulo irmão do [`super`] (teto de 600 LOC), par do `populate_filters` (que REGISTRA os
//! widgets — sem isso ficam pintados e mortos). É a única porta do produto para o
//! [`ph2d_ecs::VecFilter`]: sem ela o produtor GPU existiria, gateado e smokado, e a feature não
//! existiria para o artista.
//!
//! ⚠️ **Distinta da seção Effects** (`VECTOR_SECTION_EFFECTS`, ADR-0132), que é a pilha de
//! deformadores VETORIAIS. Um filtro produz PIXELS; um efeito produz geometria.
//!
//! # Cada degrau é um CARD, e a ORDEM é a feature
//!
//! O mesmo idioma da pilha de Effects (Enio, 2026-07-18) — cabeçalho de ícones (↑ ↓ 👁 ✕) e os
//! parâmetros dentro. Aqui o motivo de reordenar é ainda mais direto: `Drop Shadow → Blur` borra a
//! sombra junto com a forma; `Blur → Drop Shadow` projeta a sombra da forma JÁ borrada. São dois
//! desenhos, e a lista é como se escolhe entre eles.
//!
//! Cada linha oferece só os controles que o TIPO dela usa — e **quem responde é a TABELA publicada
//! pelo motor** (`FilterKindView`), nunca aritmética de código espalhada por aqui: só as sombras
//! têm Offset, o Blur não tem cor (reusa os pixels que recebeu), o Color Overlay não tem raio
//! (é pontual) e o Outline chama o raio de **Width** (a borda dele para exatamente ali). Um knob
//! morto ensina o artista a desconfiar dos vivos.

use super::*;
use crate::state::filters as fst;
use crate::state::filters::{FILTER_OFFSET_MAX, FILTER_RADIUS_MAX};
use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::{
    Card, DROPDOWN_SCROLLBAR_ID, Dropdown, DropdownOption, IconButtonStyle, IconGlyph, SliderState,
    paint_card, paint_dropdown_chip, paint_dropdown_popover_scrolled, paint_icon_button,
    scrollbar_is_needed, scrollbar_track_rect,
};

/// O lado de um botão de ícone do cabeçalho.
const ICON_PX: f32 = 22.0; // LITERAL-PX-OK: lado do glifo, espelha o do card de Effects

/// A posição do trilho a pintar: **o valor de arrasto do store SÓ enquanto o slider está sendo
/// arrastado**, senão o track derivado do estado PUBLICADO (que reflete o componente via o
/// publish do frame). É o que faz o slider mostrar o filtro da forma no instante em que ela é
/// selecionada — sem um "mirror" que re-semeie o store — e ainda seguir o dedo durante o arrasto:
/// no release, o drain do mesmo frame já atualizou o componente, o publish o leu, e o estado
/// vence de novo.
fn live_track(
    store: &ph2d_editor_core::interaction::WidgetStore,
    id: ph2d_a11y::NodeId,
    from_state: f32,
) -> f32 {
    match store.slider(id) {
        Some((SliderState::Dragging, v)) => v,
        _ => from_state,
    }
}

impl BodyCtx<'_> {
    /// Seção **FILTERS**.
    pub(crate) fn filters_section(&mut self, y: f32) -> f32 {
        // Só quando há forma para filtrar (ou uma pilha viva). Fora disso o cabeçalho nem sobe.
        let stack = fst::stack();
        if stack.is_empty() && !fst::can_add() {
            return y;
        }
        let (mut y, collapsed) = self.section_header(
            ids::VECTOR_SECTION_FILTERS,
            tr("panel.vector.section.filters"),
            y,
        );
        if collapsed {
            return y;
        }
        for (row, fx) in stack.iter().enumerate().take(ids::MAX_FILTER_ROWS) {
            // Um `kind` sem spec publicada não é desenhável (a shell publica a tabela inteira; um
            // buraco aqui seria um card sem nome e sem controles).
            let Some(spec) = fst::kind_spec(fx.kind) else {
                continue;
            };
            y = self.filter_card(row, fx, &spec, stack.len(), y);
        }
        // Os "Add": um por tipo PUBLICADO pelo motor. Um tipo novo aparece aqui sem este arquivo
        // saber que ele existe.
        if stack.len() < ids::MAX_FILTER_ROWS {
            for (kind, spec) in fst::kinds().iter().enumerate().take(ids::MAX_FILTER_KINDS) {
                let label = format!("Add {}", spec.name);
                y = self.action_button(ids::filter_add_id(kind), &label, y);
            }
        }
        y
    }

    /// Um degrau: o card, o cabeçalho de ícones e os parâmetros dentro dele.
    ///
    /// **Que rows existem é resposta da TABELA**, nunca do código do tipo — é isto que faz um tipo
    /// novo nascer com os controles certos sem editar este arquivo.
    fn filter_card(
        &mut self,
        row: usize,
        fx: &fst::FilterRowView,
        spec: &fst::FilterKindView,
        total: usize,
        y: f32,
    ) -> f32 {
        let pad = Spacing::Sm.px();
        let head_h = self.row_h.max(ICON_PX);
        // Opacity sempre; Radius, Offset X/Y, Color e a fileira de MODO conforme o tipo. A row de
        // modo é mais alta (rótulo + chips), como as `segmented` do resto do painel.
        let rows = 1
            + usize::from(spec.radius_label.is_some())
            + usize::from(spec.offset_labels.is_some()) * 2
            + usize::from(spec.color_label.is_some())
            + usize::from(spec.takes_blend);
        #[allow(clippy::cast_precision_loss)]
        let body_h = rows as f32 * (self.row_h + self.row_gap);
        let mode_h = if spec.modes.is_empty() {
            0.0
        } else {
            TypeToken::Sm.px() + Spacing::Xs.px() + self.row_h + self.row_gap
        };
        let card_h = pad + head_h + body_h + mode_h + pad;
        let card_rect = Rect::new(self.inner_x, y, self.inner_w, card_h);

        let card = Card::new(ids::filter_card_id(row));
        paint_card(&card, card_rect, self.scene, self.text_system, self.theme);

        let inner_x = self.inner_x + pad;
        let inner_w = self.inner_w - pad * 2.0;
        self.filter_header(
            row,
            fx,
            spec,
            total,
            Rect::new(inner_x, y + pad, inner_w, head_h),
        );

        // Os parâmetros, indentados dentro do card. Guardo e restauro a coluna: o `slider_row`
        // desenha no `inner_x`/`inner_w` do CONTEXTO.
        let (keep_x, keep_w) = (self.inner_x, self.inner_w);
        self.inner_x = inner_x;
        self.inner_w = inner_w;
        let mut py = y + pad + head_h;
        // O MODO vem PRIMEIRO: ele escolhe a LEI do degrau, e os números abaixo são lidos por ela.
        if !spec.modes.is_empty() {
            let chips: Vec<(ph2d_a11y::NodeId, &str, bool)> = spec
                .modes
                .iter()
                .enumerate()
                .take(ids::MAX_FILTER_MODES)
                .map(|(m, name)| {
                    (
                        ids::filter_mode_id(row, m),
                        *name,
                        u8::try_from(m) == Ok(fx.mode),
                    )
                })
                .collect();
            py = self.segmented("Mode", &chips, py);
        }
        if let Some(label) = spec.radius_label {
            py = self.filter_radius_row(row, fx, label, py);
        }
        if let Some((lx, ly)) = spec.offset_labels {
            py = self.filter_offset_row(
                lx,
                ids::filter_offx_id(row),
                ids::filter_offx_num_id(row),
                fx.offx,
                py,
            );
            py = self.filter_offset_row(
                ly,
                ids::filter_offy_id(row),
                ids::filter_offy_num_id(row),
                fx.offy,
                py,
            );
        }
        if let Some(label) = spec.color_label {
            py = self.filter_color_swatch(row, fx, label, py);
        }
        // A LEI DE MISTURA vem logo depois da COR que ela qualifica: as duas respondem à mesma
        // pergunta em dois tempos — *que cor* e *como ela encosta na que já está ali*.
        if spec.takes_blend {
            py = self.filter_blend_row(row, fx, py);
        }
        self.filter_opacity_row(row, fx, py);
        self.inner_x = keep_x;
        self.inner_w = keep_w;

        y + card_h + Spacing::Sm.px()
    }

    /// O cabeçalho do card: o nome à esquerda, os ícones à direita.
    ///
    /// **Subir na primeira linha e descer na última não fazem nada — então não são desenhados**
    /// (a posição de cada ícone é contada da DIREITA, então a ausência de uma seta nas bordas não
    /// desloca os outros). Um ícone inerte ensina o artista a desconfiar dos que funcionam.
    fn filter_header(
        &mut self,
        row: usize,
        fx: &fst::FilterRowView,
        spec: &fst::FilterKindView,
        total: usize,
        at: Rect,
    ) {
        let (x, w, y, h) = (at.x, at.w, at.y, at.h);
        let dim = if fx.enabled {
            ColorToken::Text1
        } else {
            ColorToken::TextDisabled
        };
        paint_text(
            self.text_system,
            self.scene,
            spec.name,
            x,
            y + (h - self.font) * 0.5,
            self.font,
            w,
            resolve(dim, self.theme),
        );
        let slot = |i: usize| x + w - ICON_PX * (i as f32 + 1.0) - Spacing::Xs.px() * i as f32;
        self.filter_icon(ids::filter_remove_id(row), IconId::Close, slot(0), y, h);
        self.filter_icon(
            ids::filter_hide_id(row),
            if fx.enabled {
                IconId::Eye
            } else {
                IconId::EyeClosed
            },
            slot(1),
            y,
            h,
        );
        if row + 1 < total {
            self.filter_icon(ids::filter_down_id(row), IconId::ChevronDown, slot(2), y, h);
        }
        if row > 0 {
            self.filter_icon(ids::filter_up_id(row), IconId::ChevronUp, slot(3), y, h);
        }
    }

    /// Um botão de ícone do cabeçalho: pinta e REGISTRA o hit-rect (sem ele não há Click).
    fn filter_icon(&mut self, id: ph2d_a11y::NodeId, glyph: IconId, x: f32, y: f32, h: f32) {
        let rect = Rect::new(x, y + (h - ICON_PX) * 0.5, ICON_PX, ICON_PX);
        let state = self
            .store
            .button_state(id)
            .unwrap_or(ph2d_editor_core::widget::ButtonState::Normal);
        paint_icon_button(
            rect,
            IconGlyph::Builtin(glyph),
            IconButtonStyle::Compact,
            state,
            self.scene,
            self.theme,
        );
        self.hit_index.register(id, rect);
    }

    /// **Radius** — o `stdDev` do borrão (mundo). O RÓTULO vem da tabela: no Outline ele é a
    /// LARGURA do contorno, e chamá-lo de "Radius" ali prometeria outra coisa.
    fn filter_radius_row(
        &mut self,
        row: usize,
        fx: &fst::FilterRowView,
        label: &str,
        y: f32,
    ) -> f32 {
        let (slider, chip) = (ids::filter_radius_id(row), ids::filter_radius_num_id(row));
        let track = live_track(self.store, slider, (fx.radius / FILTER_RADIUS_MAX) as f32);
        self.slider_row(
            label,
            slider,
            chip,
            track,
            fx.radius,
            &format!("{:.2}", fx.radius),
            y,
        )
    }

    /// **Opacity** — a intensidade do degrau, presente em todo degrau.
    fn filter_opacity_row(&mut self, row: usize, fx: &fst::FilterRowView, y: f32) -> f32 {
        let (slider, chip) = (ids::filter_opacity_id(row), ids::filter_opacity_num_id(row));
        let track = live_track(self.store, slider, fx.opacity as f32);
        self.slider_row(
            "Opacity",
            slider,
            chip,
            track,
            fx.opacity,
            &format!("{:.2}", fx.opacity),
            y,
        )
    }

    /// Uma fileira de offset BIPOLAR (mundo): o track `0..1` mapeia `−MAX..MAX`, `0.5` = zero — o
    /// mesmo mapa que o `populate` dá ao chip e o `event` desfaz na fronteira.
    fn filter_offset_row(
        &mut self,
        label: &str,
        slider: ph2d_a11y::NodeId,
        chip: ph2d_a11y::NodeId,
        stored: f64,
        y: f32,
    ) -> f32 {
        let from_state = ((stored + FILTER_OFFSET_MAX) / (2.0 * FILTER_OFFSET_MAX)) as f32;
        let track = live_track(self.store, slider, from_state);
        self.slider_row(
            label,
            slider,
            chip,
            track,
            stored,
            &format!("{stored:.2}"),
            y,
        )
    }

    /// A fileira da **LEI DE MISTURA**: rótulo + chip de dropdown com o nome da lei armada.
    ///
    /// Mesmo desenho da linha de PONTA do traço (rótulo na coluna de rótulos, chip ocupando o
    /// resto) — a mesma estética para a mesma pergunta *"qual destes?"*. A lista em si é pintada
    /// no passe DIFERIDO: são vinte leis, e o card mora dentro do scroll da seção.
    fn filter_blend_row(&mut self, row: usize, fx: &fst::FilterRowView, y: f32) -> f32 {
        let gap = Spacing::Sm.px();
        let id = ids::filter_blend_id(row);
        paint_text(
            self.text_system,
            self.scene,
            "Blend",
            self.inner_x,
            y + (self.row_h - self.font) * 0.5,
            self.font,
            LABEL_COL_W,
            resolve(ColorToken::Text1, self.theme),
        );
        let chip = Rect::new(
            self.inner_x + LABEL_COL_W + gap,
            y,
            (self.inner_w - LABEL_COL_W - gap).max(1.0),
            self.row_h,
        );
        let open = matches!(
            self.store.get(id),
            Some(InteractiveState::Dropdown { open: true, .. })
        );
        let dd = Dropdown::new(
            id,
            "",
            vec![DropdownOption::new(id, (), fst::blend_name(fx.blend))],
        )
        .selected(())
        .open(open);
        paint_dropdown_chip(&dd, chip, self.scene, self.text_system, self.theme);
        self.hit_index.register(id, chip);
        if open {
            state::set_pending_blend_dd(Some((row, chip)));
        }
        y + self.row_h + self.row_gap
    }

    /// A fileira da cor do halo: rótulo + swatch que abre o picker OKLCH partilhado (espelho da
    /// swatch de Fill / Contour — a mesma estética para a mesma pergunta *"que cor?"*).
    fn filter_color_swatch(
        &mut self,
        row: usize,
        fx: &fst::FilterRowView,
        label: &str,
        y: f32,
    ) -> f32 {
        let id = ids::filter_color_id(row);
        let swatch_w = SwatchSize::Md.px();
        paint_text(
            self.text_system,
            self.scene,
            label,
            self.inner_x,
            y + (self.row_h - self.font) * 0.5,
            self.font,
            LABEL_COL_W,
            resolve(ColorToken::Text1, self.theme),
        );
        let rect = Rect::new(
            self.inner_x + self.inner_w - swatch_w,
            y,
            swatch_w,
            self.row_h,
        );
        let swatch = ColorSwatch::new(id, "Filter effect color", fx.color).size(SwatchSize::Md);
        paint_color_swatch(&swatch, rect, self.scene, self.theme);
        self.hit_index.register(id, rect);
        y + self.row_h + self.row_gap
    }
}

/// **O popover das vinte leis de mistura** de um degrau — pintado no passe DIFERIDO de `paint.rs`,
/// POR CIMA de todas as seções. Espelho exato do `paint_markers::paint_marker_popover`.
///
/// ⚠️ As opções saem da tabela PUBLICADA (`blend_names`), nunca de uma lista escrita aqui: o painel
/// não alcança o `BlendMode`, e uma cópia derivaria dele na primeira lei nova — com o rótulo a
/// nomear outra coisa.
pub(crate) fn paint_blend_popover(ctx: &mut PaintCtx, row: usize, chip: Rect, theme: Theme) {
    let id = ids::filter_blend_id(row);
    let names = fst::blend_names();
    if names.is_empty() {
        return;
    }
    let sel = fst::stack()
        .get(row)
        .map_or(0, |fx| usize::from(fx.blend))
        .min(names.len() - 1);
    let options: Vec<DropdownOption<usize>> = names
        .iter()
        .enumerate()
        .map(|(i, n)| DropdownOption::new(ids::filter_blend_option_id(row, i), i, *n))
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
    for i in 0..names.len() {
        let r = dd.option_rect_in_scrolled(chip, panel, i, scroll);
        let top = r.y.max(panel.y);
        let bot = (r.y + r.h).min(panel.y + panel.h);
        if bot - top >= 1.0 {
            hit_index.register(
                ids::filter_blend_option_id(row, i),
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
