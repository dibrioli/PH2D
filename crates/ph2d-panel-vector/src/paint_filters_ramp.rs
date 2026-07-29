//! **O TRILHO da rampa** do card de Gradient Map (plano 24 W11) — irmão do `paint_filters` pelo
//! teto de 600 LOC do painel.
//!
//! O corte é por RESPONSABILIDADE: aqui vive *como um trilho de rampa se desenha e se torna
//! agarrável* (a barra amostrada, um punho por stop, `+`/`−`, a swatch do stop em foco), enquanto o
//! irmão fica com *as rows de um degrau de filtro*. É o mesmo corte que separou o `physics_rows` do
//! painel de física.
//!
//! ⚠️ **O gesto é o primitivo FOUNDATIONAL, os pixels são locais** — cada punho é um
//! `InteractiveState::CurvePoint`, o MESMO dispatch de 2D que o editor de falloff do Painter e a
//! curva do motion-params usam. E a drenagem dele **pergunta de quem é o gesto**
//! (`take_curve_point_drag_if`, em `event_filters`): o stash é um canal GLOBAL, e um painel que
//! drenava sem perguntar foi o que impediu estes punhos de se moverem (medido 2026-07-29).

use super::*;
use crate::ids;
use crate::ids::MAX_FILTER_STOPS;
use crate::state::filters as fst;
use crate::state::filters::RAMP_PREVIEW_N;
use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::paint::{fill_circle, fill_rounded_rect, stroke_rounded_rect};

/// Altura da barra de preview da rampa.
pub(crate) const RAMP_BAR_H: f32 = 14.0; // LITERAL-PX-OK: espelha o GRAD_BAR_H do editor do Painter
/// Raio do punho DESENHADO.
pub(crate) const RAMP_HANDLE_R: f32 = 5.0; // LITERAL-PX-OK: espelha o GRAD_HANDLE_R do Painter
/// Meia-largura da caixa de AGARRE (maior que o punho desenhado — é a mira do dedo).
const RAMP_GRAB_R: f32 = 8.0; // LITERAL-PX-OK: espelha o CURVE_GRAB_R do primitivo compartilhado
/// Largura dos botões `+` / `−` do trilho.
const RAMP_BTN_W: f32 = 24.0; // LITERAL-PX-OK: espelha o CURVE_BTN_W do editor do Painter
/// Arredondamento da barra.
const RAMP_BAR_RADIUS: f32 = 4.0; // LITERAL-PX-OK: espelha o Radius::Sm do design system

/// A altura que o trilho consome. ⚠️ **UMA régua, dois consumidores** — a conta da altura do card e
/// o paint perguntam a ESTA função; medir por uma régua e preencher por outra é exactamente o que a
/// lei do `segmented_row_counts` proíbe.
pub(crate) fn ramp_extra_h(row_gap: f32) -> f32 {
    RAMP_BAR_H + RAMP_HANDLE_R + row_gap
}

impl BodyCtx<'_> {
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
    pub(crate) fn filter_ramp_rows(&mut self, row: usize, fx: &fst::FilterRowView, y: f32) -> f32 {
        // ⚠️ **O MESMO `row_gap` que a ALTURA do card reserva.** Eu usei `Spacing::Xs` aqui e
        // `self.row_gap` (que é `Sm`) na conta da altura — container medido por uma régua e
        // preenchido por outra, exactamente o que a lei do `segmented_row_counts` proíbe. Sobrava
        // folga (Sm > Xs), então não transbordava; mas a próxima linha a mexer no trilho herdaria a
        // discordância na direção que transborda.
        let gap = self.row_gap;
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
            // A rampa amostrada é DADO, não um token de tema (a shell a amostra da MESMA
            // função que o dispositivo honra — paridade medida em 1 nível de byte).
            let col = ph2d_vector::Color::from_rgba8(rgb[0], rgb[1], rgb[2], 255); // LITERAL-COLOR-OK: a rampa amostrada e DADO do documento, nao cor de tema
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
            let scol = ph2d_vector::Color::from_rgba8(c[0], c[1], c[2], 255); // LITERAL-COLOR-OK: a cor autorada do stop e DADO, nao cor de tema
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
        // ⚠️ **Diagnóstico atrás de env** — o report *"não posso mover os pontos de cor"* não
        // reproduz headless (o seam dirige o gesto real e chega ao barramento), então o que falta
        // medir é o que só o app vivo tem. Ele responde as DUAS perguntas de uma vez: os punhos
        // existem (e onde), e o store sabe que eles são arrastáveis.
        if std::env::var_os("PH2D_FX_RAMP_DIAG").is_some() {
            let armed = matches!(
                self.store.get(ids::filter_stop_id(row, 0)),
                Some(InteractiveState::CurvePoint { .. })
            );
            eprintln!(
                "[ramp] linha {row}: barra x={:.0} w={:.0} y={:.0} · {n} punho(s) · store armado: {armed}",
                bar.x, bar.w, bar.y
            );
        }
        // ── E a cor do stop escolhido, pela MESMA swatch/picker das duas pontas do Duotone ──
        let colour = fx
            .stop_colors
            .get(selected)
            .copied()
            .unwrap_or([0, 0, 0, 255]);
        self.filter_color_swatch(ids::filter_stop_color_id(row), colour, "Stop", py)
    }
}
