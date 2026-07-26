//! A seção **FILTERS** do painel Vector — módulo irmão do [`super`] (teto de 600 LOC).
//!
//! O FX raster por-forma (plano 24): Blur / Glow / Drop Shadow. É a única porta do produto para o
//! [`ph2d_ecs::VecFilter`] — sem ela o produtor GPU existiria, gateado e smokado, e a feature não
//! existiria para o artista.
//!
//! ⚠️ **Distinta da seção Effects** (`VECTOR_SECTION_EFFECTS`, ADR-0132), que é a pilha de
//! deformadores VETORIAIS. Um filtro produz PIXELS; um efeito produz geometria. O seletor de tipo
//! é a porta de armar/desarmar (o *None* remove o componente) — não há botão *Add* separado como
//! no Contour, porque um filtro TEM tipo, e escolher o tipo É armá-lo.

use super::*;
use crate::state::filters as fst;
use crate::state::filters::{FILTER_OFFSET_MAX, FILTER_RADIUS_MAX};
use ph2d_editor_core::widget::SliderState;

/// A posição do trilho a pintar: **o valor de arrasto do store SÓ enquanto o slider está sendo
/// arrastado**, senão o track derivado do estado PUBLICADO (que reflete o componente via o
/// publish do frame). É o que faz o slider mostrar o filtro da forma no instante em que ela é
/// selecionada — sem um "mirror" que re-semeie o store — e ainda seguir o dedo durante o arrasto:
/// no release, o drain do mesmo frame já atualizou o componente, o publish o leu, e o estado
/// vence de novo.
fn live_track(store: &ph2d_editor_core::interaction::WidgetStore, id: ph2d_a11y::NodeId, from_state: f32) -> f32 {
    match store.slider(id) {
        Some((SliderState::Dragging, v)) => v,
        _ => from_state,
    }
}

impl BodyCtx<'_> {
    /// Seção **FILTERS**.
    pub(crate) fn filters_section(&mut self, y: f32) -> f32 {
        // Só quando há forma para filtrar (ou um filtro vivo). Fora disso o cabeçalho nem sobe.
        if !fst::present() && !fst::can_add() {
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
        let present = fst::present();
        let kind = fst::kind();
        // O seletor de TIPO é a porta de armar/desarmar, sempre visível: None remove, os outros
        // três armam. O ativo é derivado do estado publicado.
        y = self.segmented(
            "Filter",
            &[
                (ids::VECTOR_FILTER_KIND_NONE, "None", !present),
                (ids::VECTOR_FILTER_KIND_BLUR, "Blur", present && kind == 0),
                (ids::VECTOR_FILTER_KIND_GLOW, "Glow", present && kind == 1),
                (ids::VECTOR_FILTER_KIND_SHADOW, "Shadow", present && kind == 2),
            ],
            y,
        );
        if !present {
            return y;
        }
        // Radius — o stdDev do borrão (mundo), presente em todo filtro.
        let radius = fst::radius();
        let radius_track = live_track(
            self.store,
            ids::VECTOR_FILTER_RADIUS,
            (radius / FILTER_RADIUS_MAX) as f32,
        );
        y = self.slider_row(
            "Radius",
            ids::VECTOR_FILTER_RADIUS,
            ids::VECTOR_FILTER_RADIUS_NUM,
            radius_track,
            radius,
            &format!("{radius:.2}"),
            y,
        );
        // Offset X/Y — só a Drop Shadow tem deslocamento; para Blur/Glow seria controle morto.
        if kind == 2 {
            y = self.filter_offset_row("Offset X", ids::VECTOR_FILTER_OFFX, ids::VECTOR_FILTER_OFFX_NUM, fst::offx(), y);
            y = self.filter_offset_row("Offset Y", ids::VECTOR_FILTER_OFFY, ids::VECTOR_FILTER_OFFY_NUM, fst::offy(), y);
        }
        // Color — Glow/Drop Shadow tingem; o Blur ignora a cor (a swatch seria knob morto nele).
        if kind == 1 || kind == 2 {
            y = self.filter_color_swatch(y);
        }
        // Opacity — todo filtro.
        let opacity = fst::opacity();
        let op_track = live_track(self.store, ids::VECTOR_FILTER_OPACITY, opacity as f32);
        self.slider_row(
            "Opacity",
            ids::VECTOR_FILTER_OPACITY,
            ids::VECTOR_FILTER_OPACITY_NUM,
            op_track,
            opacity,
            &format!("{opacity:.2}"),
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
        self.slider_row(label, slider, chip, track, stored, &format!("{stored:.2}"), y)
    }

    /// A fileira da cor do efeito: rótulo + swatch que abre o picker OKLCH partilhado (espelho da
    /// swatch de Fill / Contour — a mesma estética para a mesma pergunta *"que cor?"*).
    fn filter_color_swatch(&mut self, y: f32) -> f32 {
        let swatch_w = SwatchSize::Md.px();
        paint_text(
            self.text_system,
            self.scene,
            "Color",
            self.inner_x,
            y + (self.row_h - self.font) * 0.5,
            self.font,
            LABEL_COL_W,
            resolve(ColorToken::Text1, self.theme),
        );
        let rect = Rect::new(self.inner_x + self.inner_w - swatch_w, y, swatch_w, self.row_h);
        let swatch = ColorSwatch::new(ids::VECTOR_FILTER_COLOR, "Filter effect color", fst::color())
            .size(SwatchSize::Md);
        paint_color_swatch(&swatch, rect, self.scene, self.theme);
        self.hit_index.register(ids::VECTOR_FILTER_COLOR, rect);
        y + self.row_h + self.row_gap
    }
}
