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
use crate::ids::MAX_FILTER_STOPS;
use crate::state::filters as fst;
use crate::state::filters::{
    FILTER_ADJUST_MAX, FILTER_DETAIL_MAX, FILTER_GROW_MAX, FILTER_HUE_MAX, FILTER_OFFSET_MAX,
    FILTER_RADIUS_MAX, FILTER_SCALE_MAX, FILTER_SEED_MAX, RAMP_PREVIEW_N,
};
use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::paint::{fill_circle, fill_rounded_rect, stroke_rounded_rect};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::{
    Card, DROPDOWN_SCROLLBAR_ID, Dropdown, DropdownOption, IconButtonStyle, IconGlyph, SliderState,
    paint_card, paint_dropdown_chip, paint_dropdown_popover_scrolled, paint_icon_button,
    scrollbar_is_needed, scrollbar_track_rect,
};

/// O lado de um botão de ícone do cabeçalho.
const ICON_PX: f32 = 22.0; // LITERAL-PX-OK: lado do glifo, espelha o do card de Effects

/// A altura da barra de preview da rampa.
const RAMP_BAR_H: f32 = 14.0; // LITERAL-PX-OK: espelha o GRAD_BAR_H do editor do Painter
/// O raio do punho de um stop — o que se VÊ.
const RAMP_HANDLE_R: f32 = 5.0; // LITERAL-PX-OK: espelha o GRAD_HANDLE_R do Painter
/// O raio da CAIXA DE AGARRE — o que o ponteiro alcança, e **o recurso de que o teto de stops é**.
///
/// ⚠️ Oito punhos numa barra de ~200 px ficam a ~28 px um do outro, folgado sobre 16 px de agarre;
/// é isto que o gate do teto mede, e não a memória do uniform.
const RAMP_GRAB_R: f32 = 8.0; // LITERAL-PX-OK: espelha o CURVE_GRAB_R do primitivo compartilhado
/// A largura de um botão `+` / `−` do trilho.
const RAMP_BTN_W: f32 = 24.0; // LITERAL-PX-OK: espelha o CURVE_BTN_W do editor do Painter
/// O arredondamento do contorno da barra — o `Radius::Sm` que o resto do card usa.
const RAMP_BAR_RADIUS: f32 = 4.0; // LITERAL-PX-OK: espelha o Radius::Sm do design system

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
            + usize::from(spec.color_b_label.is_some())
            + usize::from(spec.takes_blend)
            + usize::from(spec.noise_labels.is_some()) * 3
            + usize::from(spec.grow_label.is_some())
            + usize::from(spec.adjust_labels.is_some()) * 3
            // O trilho custa DUAS rows: a fileira dos botões `+`/`−` e a swatch do stop escolhido.
            // A barra e os punhos vão no `ramp_h`, que não é múltiplo de `row_h`.
            + usize::from(spec.takes_ramp) * 2;
        #[allow(clippy::cast_precision_loss)]
        let body_h = rows as f32 * (self.row_h + self.row_gap);
        let mode_h = if spec.modes.is_empty() {
            0.0
        } else {
            TypeToken::Sm.px() + Spacing::Xs.px() + self.row_h + self.row_gap
        };
        // ⚠️ A barra + os punhos têm altura PRÓPRIA (não é uma row): medir por `row_h` deixaria a
        // próxima seção pintar por cima das alças, que é a falha que a lei do `segmented_row_counts`
        // já nomeia — container medido por uma régua e preenchido por outra.
        let ramp_h = if spec.takes_ramp {
            RAMP_BAR_H + RAMP_HANDLE_R + self.row_gap
        } else {
            0.0
        };
        let card_h = pad + head_h + body_h + mode_h + ramp_h + pad;
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
        let track = live_track(self.store, s_ids.0, (fx.scale / FILTER_SCALE_MAX) as f32);
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
        let d_track = live_track(
            self.store,
            d_ids.0,
            ((d - 1.0) / (FILTER_DETAIL_MAX - 1.0)) as f32,
        );
        py = self.slider_row(detail, d_ids.0, d_ids.1, d_track, d, &format!("{d:.0}"), py);
        let k_ids = (ids::filter_seed_id(row), ids::filter_seed_num_id(row));
        let k = f64::from(fx.seed);
        let k_track = live_track(self.store, k_ids.0, (k / FILTER_SEED_MAX) as f32);
        self.slider_row(seed, k_ids.0, k_ids.1, k_track, k, &format!("{k:.0}"), py)
    }

    /// **Amount** do Grow / Shrink — BIPOLAR, e é o que o distingue do raio.
    ///
    /// ⚠️ O readout traz o SINAL explícito (`+0,06` / `−0,06`): num slider cujo neutro é o meio do
    /// curso, um número sem sinal deixa as duas metades a ler igual.
    fn filter_grow_row(&mut self, row: usize, fx: &fst::FilterRowView, label: &str, y: f32) -> f32 {
        let (slider, chip) = (ids::filter_grow_id(row), ids::filter_grow_num_id(row));
        let t = ((fx.grow + FILTER_GROW_MAX) / (2.0 * FILTER_GROW_MAX)) as f32;
        let track = live_track(self.store, slider, t);
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
        let h_track = live_track(self.store, h_ids.0, h_t);
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
            let track = live_track(self.store, ids2.0, t);
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
    ///
    /// ⚠️ **Recebe o id e a cor, em vez de os derivar da `row`** — é o que faz a SEGUNDA ponta do
    /// Duotone ser esta mesma função outra vez, em vez de um caminho de desenho paralelo que
    /// divergiria na primeira mudança de estética.
    /// **O TRILHO da rampa** — a fileira `+`/`−`, a barra de preview com os punhos arrastáveis, e a
    /// swatch do stop SELECIONADO. Devolve o `y` seguinte.
    ///
    /// ⚠️ **O gesto é o primitivo FOUNDATIONAL, os pixels são locais** — e é o precedente deste
    /// repo, não um atalho: cada punho é um `InteractiveState::CurvePoint` (o mesmo dispatch de 2D
    /// que o editor de falloff do Painter e a curva do motion-params usam), e é ele que carrega o
    /// retângulo da barra para o dispatch converter x → posição. O DESENHO é do painel, como em
    /// todos os outros; extrair o pintor exigiria parametrizar dois `PaintCtx` diferentes, e o custo
    /// está NOMEADO no handoff em vez de escondido.
    ///
    /// ⚠️ **O `register` do `CurvePoint` é reescrito a CADA frame**, de propósito: ele carrega o
    /// `canvas`, e um painel redimensionado deixaria o gesto a converter contra uma barra que já não
    /// está ali.
    fn filter_ramp_rows(&mut self, row: usize, fx: &fst::FilterRowView, y: f32) -> f32 {
        let gap = Spacing::Xs.px();
        let mut py = y;
        // ── A fileira `+` / `−`, alinhada à direita ──
        let n = usize::from(fx.stop_count).min(MAX_FILTER_STOPS);
        for (i, (id, glyph)) in [
            (ids::filter_stop_add_id(row), IconId::Plus),
            (ids::filter_stop_remove_id(row), IconId::Minus),
        ]
        .into_iter()
        .enumerate()
        {
            #[allow(clippy::cast_precision_loss)]
            let bx = self.inner_x + self.inner_w - RAMP_BTN_W * (2.0 - i as f32);
            // O MESMO helper do cabeçalho: um segundo estilo de botão de ícone no mesmo card faria
            // dois vocabulários para o mesmo gesto.
            self.filter_icon(id, glyph, bx, py, self.row_h);
        }
        py += self.row_h + gap;
        // ── A barra: as fatias que a SHELL amostrou ──
        let bar = Rect::new(self.inner_x, py, self.inner_w.max(1.0), RAMP_BAR_H);
        #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
        let slice_w = bar.w / RAMP_PREVIEW_N as f32;
        for (i, rgb) in fx.ramp_preview.iter().enumerate() {
            #[allow(clippy::cast_precision_loss)]
            let sx = bar.x + i as f32 * slice_w;
            // LITERAL-COLOR-OK: a rampa amostrada é DADO, não um token de tema.
            let col = ph2d_vector::Color::from_rgba8(rgb[0], rgb[1], rgb[2], 255);
            // `+1` na largura para não deixar costura entre fatias no arredondamento.
            fill_rounded_rect(
                self.scene,
                Rect::new(sx, bar.y, slice_w + 1.0, bar.h),
                0.0,
                col,
            );
        }
        stroke_rounded_rect(
            self.scene,
            bar,
            RAMP_BAR_RADIUS,
            1.0, // LITERAL-PX-OK: fio de 1 px, o mesmo do editor do Painter
            resolve(ColorToken::Border, self.theme),
        );
        // ── Os punhos, na borda de baixo da barra ──
        //
        // Publica ONDE a barra está: quem registra o `CurvePoint` é o passe de sementes (o `store`
        // é imutável aqui), e ele tem de converter contra ESTE retângulo, não contra um que ele
        // re-derive.
        fst::set_ramp_bar(row, [bar.x, bar.y, bar.w, bar.h]);
        let selected = usize::from(fst::selected_stop(row)).min(n.saturating_sub(1));
        let cy = bar.y + bar.h;
        for stop in 0..n {
            let id = ids::filter_stop_id(row, stop);
            let cx = bar.x + fx.stop_pos[stop].clamp(0.0, 1.0) * bar.w;
            self.hit_index.register(
                id,
                Rect::new(
                    cx - RAMP_GRAB_R,
                    cy - RAMP_GRAB_R,
                    RAMP_GRAB_R * 2.0,
                    RAMP_GRAB_R * 2.0,
                ),
            );
            let c = fx.stop_colors[stop];
            // LITERAL-COLOR-OK: a cor do próprio stop é DADO.
            let scol = ph2d_vector::Color::from_rgba8(c[0], c[1], c[2], 255);
            // O anel diz QUAL está em foco — é o que liga o punho à swatch abaixo.
            let ring = if stop == selected {
                ColorToken::Accent
            } else {
                ColorToken::Border
            };
            fill_circle(
                self.scene,
                cx,
                cy,
                RAMP_HANDLE_R + 1.5, // LITERAL-PX-OK: o fio do anel
                resolve(ring, self.theme),
            );
            fill_circle(self.scene, cx, cy, RAMP_HANDLE_R, scol);
        }
        py += RAMP_BAR_H + RAMP_HANDLE_R + gap;
        // ── E a cor do stop escolhido, pela MESMA swatch/picker das duas pontas do Duotone ──
        let colour = fx
            .stop_colors
            .get(selected)
            .copied()
            .unwrap_or([0, 0, 0, 255]);
        self.filter_color_swatch(ids::filter_stop_color_id(row), colour, "Stop", py)
    }

    fn filter_color_swatch(
        &mut self,
        id: ph2d_a11y::NodeId,
        colour: [u8; 4],
        label: &str,
        y: f32,
    ) -> f32 {
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
        let swatch = ColorSwatch::new(id, "Filter effect color", colour).size(SwatchSize::Md);
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
