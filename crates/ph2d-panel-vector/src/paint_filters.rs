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
//! # Cada degrau é uma LINHA, e a ORDEM é a feature
//!
//! F43 (dono 01/11, layout do Figma): o «+» do cabeçalho abre a lista dos tipos, cada degrau é uma
//! linha ([`super::fx_list`]) e clicar nela abre os controles num painel à ESQUERDA do Inspector
//! ([`crate::fx_side`]), com ↑ ↓ no cabeçalho dele. A ordem continua a ser a feature:
//! `Drop Shadow → Blur` borra a sombra junto com a forma; `Blur → Drop Shadow` projeta a sombra da
//! forma JÁ borrada.
//!
//! Cada linha oferece só os controles que o TIPO dela usa — e **quem responde é a TABELA publicada
//! pelo motor** (`FilterKindView`), nunca aritmética de código espalhada por aqui: só as sombras
//! têm Offset, o Blur não tem cor (reusa os pixels que recebeu), o Color Overlay não tem raio
//! (é pontual) e o Outline chama o raio de **Width** (a borda dele para exatamente ali). Um knob
//! morto ensina o artista a desconfiar dos vivos.

use super::fx_list::Pilha;
use super::*;
use crate::state::filters as fst;
use crate::state::filters::{
    FILTER_ADJUST_MAX, FILTER_DETAIL_MAX, FILTER_GROW_MAX, FILTER_HUE_MAX, FILTER_OFFSET_MAX,
    FILTER_RADIUS_MAX, FILTER_SCALE_MAX, FILTER_SEED_MAX,
};
use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::widget::{IconButtonStyle, IconGlyph, paint_icon_button};
use ph2d_i18n::tr;

/// O lado de um botão de ícone do cabeçalho.
const ICON_PX: f32 = 22.0; // LITERAL-PX-OK: lado do glifo, espelha o do card de Effects

impl BodyCtx<'_> {
    /// Seção **FILTERS**.
    pub(crate) fn filters_section(&mut self, y: f32) -> f32 {
        // Só quando há forma para filtrar (ou uma pilha viva). Fora disso o cabeçalho nem sobe.
        let stack = fst::stack();
        if stack.is_empty() && !fst::can_add() {
            return y;
        }
        let live = fst::can_add() && stack.len() < crate::ids::MAX_FILTER_ROWS;
        let (mut y, collapsed) = self.fx_list_header(
            Pilha::Filtros,
            ids::VECTOR_SECTION_FILTERS,
            tr("panel.vector.section.filters"),
            live,
            y,
        );
        if collapsed {
            return y;
        }
        for (row, fx) in stack.iter().enumerate().take(crate::ids::MAX_FILTER_ROWS) {
            // Um `kind` sem spec publicada não é desenhável (a shell publica a tabela inteira; um
            // buraco aqui seria uma linha sem nome e sem controles).
            let Some(spec) = fst::kind_spec(fx.kind) else {
                continue;
            };
            y = self.fx_list_row(
                Pilha::Filtros,
                row,
                fst::kind_icon(usize::from(fx.kind)),
                spec.name,
                fx.enabled,
                y,
            );
        }
        y
    }

    /// **Os controles do degrau `row`** — o corpo do painel à esquerda ([`crate::fx_side`]).
    /// Devolve o `y` final.
    ///
    /// **Que rows existem é resposta da TABELA**, nunca do código do tipo — é isto que faz um tipo
    /// novo nascer com os controles certos sem editar este arquivo.
    pub(crate) fn filter_props(
        &mut self,
        row: usize,
        fx: &fst::FilterRowView,
        spec: &fst::FilterKindView,
        y: f32,
    ) -> f32 {
        let mut py = y;
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
                        crate::nomes_do_motor::modo(name),
                        u8::try_from(m) == Ok(fx.mode),
                    )
                })
                .collect();
            py = self.segmented(tr("panel.vector.filter.mode"), &chips, py);
        }
        if let Some(label) = spec.radius_label {
            py = self.filter_radius_row(row, fx, label, py);
        }
        // Os três knobs do RUÍDO vêm logo depois do Amount que eles qualificam: *quanto anda* e
        // depois *segundo que campo*.
        if let Some((size, detail, seed)) = spec.noise_labels {
            py = self.filter_noise_rows(row, fx, (size, detail, seed), py);
        }
        // O **Amount** do Grow / Shrink: o único número do tipo, logo no topo do corpo.
        if let Some(label) = spec.grow_label {
            py = self.filter_grow_row(row, fx, label, py);
        }
        // Os três do **Color Adjust** — juntos, e nessa ordem, porque é a ficha que Photoshop / AE
        // / Krita / Blender desenharam e é a ordem em que um artista a lê.
        if let Some((h, s, b)) = spec.adjust_labels {
            py = self.filter_adjust_rows(row, fx, (h, s, b), py);
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
            py = self.filter_color_swatch(ids::filter_color_id(row), fx.color, label, py);
        }
        // A SEGUNDA ponta da rampa, logo abaixo da primeira: as duas descrevem o mesmo objeto (o
        // degradê) e leem-se de cima para baixo, escuro → claro.
        if let Some(label) = spec.color_b_label {
            py = self.filter_color_swatch(ids::filter_color_b_id(row), fx.color_b, label, py);
        }
        // O TRILHO ocupa o lugar das swatches nos tipos que têm rampa: ele É a cor deste degrau, e
        // vem antes da lei de mistura pela MESMA razão que as swatches vêm (a lei qualifica a cor).
        if spec.takes_ramp {
            py = self.filter_ramp_rows(row, fx, py);
        }
        // A LEI DE MISTURA vem logo depois da COR que ela qualifica: as duas respondem à mesma
        // pergunta em dois tempos — *que cor* e *como ela encosta na que já está ali*.
        if spec.takes_blend {
            py = self.filter_blend_row(row, fx, py);
        }
        self.filter_opacity_row(row, fx, py)
    }

    /// Um botão de ícone do cabeçalho: pinta e REGISTRA o hit-rect (sem ele não há Click).
    pub(crate) fn filter_icon(
        &mut self,
        id: ph2d_a11y::NodeId,
        glyph: IconId,
        x: f32,
        y: f32,
        h: f32,
    ) {
        let rect = Rect::new(x, y + (h - ICON_PX) * 0.5, ICON_PX, ICON_PX);
        let state = self.store.button_visual(id);
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
        let track = self.live_track(slider, (fx.radius / FILTER_RADIUS_MAX) as f32);
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

    /// **Os três knobs do RUÍDO** — Size (mundo), Detail (oitavas) e Seed (a realização).
    ///
    /// Uma função para os três porque eles são UMA pergunta em três tempos (*qual ruído?*), e a
    /// tabela os oferece ou não em bloco: um Size sem Detail descreve metade de um campo.
    fn filter_noise_rows(
        &mut self,
        row: usize,
        fx: &fst::FilterRowView,
        labels: (&str, &str, &str),
        y: f32,
    ) -> f32 {
        let (size, detail, seed) = labels;
        let s_ids = (ids::filter_scale_id(row), ids::filter_scale_num_id(row));
        let track = self.live_track(s_ids.0, (fx.scale / FILTER_SCALE_MAX) as f32);
        let mut py = self.slider_row(
            size,
            s_ids.0,
            s_ids.1,
            track,
            fx.scale,
            &format!("{:.2}", fx.scale),
            y,
        );
        // ⚠️ Detail e Seed são CONTAGENS: o chip mostra inteiro, e o `populate` os liga por
        // `slider_chip_int` — sem isso, digitar "3,5" deixa o campo em 3,5 sob um painel que
        // desenha "4".
        let d_ids = (ids::filter_detail_id(row), ids::filter_detail_num_id(row));
        let d = f64::from(fx.detail).max(1.0);
        let d_track = self.live_track(d_ids.0, ((d - 1.0) / (FILTER_DETAIL_MAX - 1.0)) as f32);
        py = self.slider_row(detail, d_ids.0, d_ids.1, d_track, d, &format!("{d:.0}"), py);
        let k_ids = (ids::filter_seed_id(row), ids::filter_seed_num_id(row));
        let k = f64::from(fx.seed);
        let k_track = self.live_track(k_ids.0, (k / FILTER_SEED_MAX) as f32);
        self.slider_row(seed, k_ids.0, k_ids.1, k_track, k, &format!("{k:.0}"), py)
    }

    /// **Amount** do Grow / Shrink — BIPOLAR, e é o que o distingue do raio.
    ///
    /// ⚠️ O readout traz o SINAL explícito (`+0,06` / `−0,06`): num slider cujo neutro é o meio do
    /// curso, um número sem sinal deixa as duas metades a ler igual.
    fn filter_grow_row(&mut self, row: usize, fx: &fst::FilterRowView, label: &str, y: f32) -> f32 {
        let (slider, chip) = (ids::filter_grow_id(row), ids::filter_grow_num_id(row));
        let t = ((fx.grow + FILTER_GROW_MAX) / (2.0 * FILTER_GROW_MAX)) as f32;
        let track = self.live_track(slider, t);
        self.slider_row(
            label,
            slider,
            chip,
            track,
            fx.grow,
            &format!("{:+.2}", fx.grow),
            y,
        )
    }

    /// **Os três knobs do Color Adjust** — matiz (graus), saturação e brilho, todos BIPOLARES.
    ///
    /// ⚠️ Os readouts trazem o SINAL explícito pela mesma razão do Amount: num slider cujo neutro
    /// é o meio do curso, um número sem sinal deixa as duas metades a ler igual. A matiz traz o
    /// GRAU (`+90°`) porque voltas não é a unidade em que ninguém pensa uma cor.
    fn filter_adjust_rows(
        &mut self,
        row: usize,
        fx: &fst::FilterRowView,
        labels: (&str, &str, &str),
        y: f32,
    ) -> f32 {
        let (hue_l, sat_l, bright_l) = labels;
        let mut py = y;
        let h_ids = (ids::filter_hue_id(row), ids::filter_hue_num_id(row));
        let h_t = ((fx.hue + FILTER_HUE_MAX) / (2.0 * FILTER_HUE_MAX)) as f32;
        let h_track = self.live_track(h_ids.0, h_t);
        py = self.slider_row(
            hue_l,
            h_ids.0,
            h_ids.1,
            h_track,
            fx.hue,
            &format!("{:+.0}", fx.hue),
            py,
        );
        for (label, value, ids2) in [
            (
                sat_l,
                fx.sat,
                (ids::filter_sat_id(row), ids::filter_sat_num_id(row)),
            ),
            (
                bright_l,
                fx.bright,
                (ids::filter_bright_id(row), ids::filter_bright_num_id(row)),
            ),
        ] {
            let t = ((value + FILTER_ADJUST_MAX) / (2.0 * FILTER_ADJUST_MAX)) as f32;
            let track = self.live_track(ids2.0, t);
            py = self.slider_row(
                label,
                ids2.0,
                ids2.1,
                track,
                value,
                &format!("{value:+.2}"),
                py,
            );
        }
        py
    }

    /// **Opacity** — a intensidade do degrau, presente em todo degrau.
    fn filter_opacity_row(&mut self, row: usize, fx: &fst::FilterRowView, y: f32) -> f32 {
        let (slider, chip) = (ids::filter_opacity_id(row), ids::filter_opacity_num_id(row));
        let track = self.live_track(slider, fx.opacity as f32);
        self.slider_row(
            tr("panel.vector.filter.opacity"),
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
        let track = self.live_track(slider, from_state);
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
        let id = ids::filter_blend_id(row);
        ph2d_editor_core::widget::paint_property_label(
            self.text_system,
            self.scene,
            tr("panel.vector.filter.blend"),
            self.inner_x,
            y + (self.row_h - self.font) * 0.5,
            self.font,
            label_col_w(self.inner_x, self.inner_w),
            resolve(ColorToken::Text1, self.theme),
        );
        let chip = ph2d_editor_core::panel::value_col(self.inner_x, self.inner_w, y, self.row_h);
        if let Some(chip) = self.dropdown_row(id, "", fst::blend_name(fx.blend), chip) {
            state::set_pending_blend_dd(Some((row, chip)));
        }
        y + self.row_h + self.row_gap
    }

    /// ⚠️ **Delega** à porta única [`BodyCtx::colour_swatch_row`] — a W5b deu-lhe um segundo
    /// consumidor (a cor de uma peça de instância), e duas cópias da mesma linha divergiriam no
    /// primeiro ajuste de layout.
    pub(crate) fn filter_color_swatch(
        &mut self,
        id: ph2d_a11y::NodeId,
        colour: [u8; 4],
        label: &str,
        y: f32,
    ) -> f32 {
        // ⚠️ O rótulo de a11y deixou de ser um argumento: a porta da casa usa o NOME da fileira
        //    para os dois papéis, e duas frases para a mesma linha divergem.
        self.colour_swatch_row(id, colour, label, y)
    }
}
