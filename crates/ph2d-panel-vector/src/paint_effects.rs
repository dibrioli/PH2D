//! A seção **EFFECTS** (ADR-0132) — a pilha de Live Path Effects do caminho selecionado.
//!
//! Módulo irmão do [`super`] pelo teto de 600 LOC, par do `populate_effects` (que REGISTRA os
//! widgets — sem isso ficam pintados e mortos).
//!
//! # Cada efeito é um CARD (Enio, 2026-07-18)
//!
//! A 1ª versão empilhava botões de largura inteira e o resultado era ilegível: nada dizia onde
//! um efeito acabava e o seguinte começava, e os parâmetros pareciam soltos. Agora cada
//! entrada é um **card** — o idioma que o painel do Painter já usa —, com um **cabeçalho de
//! ícones** e os parâmetros dentro:
//!
//! ```text
//! ┌──────────────────────────────────┐
//! │ Zig Zag        ↑  ↓  👁  ✕       │
//! │ Size    ────●──────────  42.00   │
//! │ Ridges  ──●────────────   8.00   │
//! └──────────────────────────────────┘
//! ```
//!
//! # Este arquivo não conhece nenhum efeito, e é esse o ponto
//!
//! Ele desenha **linhas** vindas do snapshot: um rótulo, os ícones e um controle por parâmetro
//! DESCRITO (nome, faixa, é-caixinha). Acrescentar um efeito ao motor não toca aqui.
//!
//! O **olho** desarma sem apagar, e os parâmetros de um efeito desarmado continuam editáveis:
//! desarmar não pode custar números que o artista teria de lembrar.

use super::*;
use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::widget::{Card, IconButtonStyle, IconGlyph, paint_card, paint_icon_button};

/// Quantas casas o chip mostra.
const DECIMALS: usize = 2;

/// O lado de um botão de ícone do cabeçalho.
const ICON_PX: f32 = 22.0; // LITERAL-PX-OK: lado do glifo, espelha o `BTN_W` do transporte

impl BodyCtx<'_> {
    /// A seção Effects. Devolve o `y` intocado quando não há caminho único selecionado — é o
    /// que faz o `step` não emitir separador órfão.
    pub(crate) fn effects_section(&mut self, y: f32) -> f32 {
        let (mut y, collapsed) = self.section_header(
            ids::VECTOR_SECTION_EFFECTS,
            tr("panel.vector.section.effects"),
            y,
        );
        if collapsed || !state::has_target() {
            return y;
        }
        let stack = state::stack();
        for (row, fx) in stack.iter().enumerate().take(ids::MAX_FX_ROWS) {
            y = self.effect_card(row, fx, stack.len(), y);
        }
        // Os "Add": um por tipo PUBLICADO. A tabela vem do motor, então um efeito novo aparece
        // aqui sem este arquivo saber que ele existe.
        if stack.len() < ids::MAX_FX_ROWS {
            for (kind, name) in state::kinds().iter().enumerate().take(ids::MAX_FX_KINDS) {
                y = self.action_button(ids::vector_fx_add_id(kind), &format!("Add {name}"), y);
            }
        }
        // **Apply**, no FIM da seção e em ACCENT: é a ação de *commit* (assa a pilha no cozido e a
        // esvazia — o *Expand Appearance*), então destaca-se das edições comuns e fecha a fila.
        // Só aparece quando há o que assar — um botão que esvazia uma pilha vazia seria inerte, e
        // um botão inerte ensina o artista a desconfiar dos que funcionam.
        if !stack.is_empty() {
            y = self.action_button_kind(
                ids::VECTOR_FX_APPLY,
                "Apply Effects",
                ButtonKind::Accent,
                y,
            );
        }
        y
    }

    /// Um efeito: o card, o cabeçalho de ícones e os parâmetros dentro dele.
    fn effect_card(&mut self, row: usize, fx: &state::FxRowView, total: usize, y: f32) -> f32 {
        let pad = Spacing::Sm.px();
        let head_h = self.row_h.max(ICON_PX);
        let params = fx.params.len().min(ids::MAX_FX_ROW_PARAMS);
        #[allow(clippy::cast_precision_loss)]
        let body_h = params as f32 * (self.row_h + self.row_gap);
        // Um Falloff ganha uma LINHA de dica ("modula abaixo" / "adicione um deformador"): ele não
        // deforma nada sozinho, e sem a dica um Falloff sem deformador abaixo pareceria quebrado.
        let note_h = if fx.falloff_role == state::FalloffRole::NotFalloff {
            0.0
        } else {
            self.row_h
        };
        let card_h = pad + head_h + note_h + body_h + pad;
        let card_rect = Rect::new(self.inner_x, y, self.inner_w, card_h);

        // O card em si (moldura + fundo) — o mesmo primitivo que o painel do Painter usa, para
        // não haver duas respostas a "como é um card neste app".
        let card = Card::new(ids::vector_fx_card_id(row));
        paint_card(&card, card_rect, self.scene, self.text_system, self.theme);

        let inner_x = self.inner_x + pad;
        let inner_w = self.inner_w - pad * 2.0;
        self.effect_header(row, fx, total, Rect::new(inner_x, y + pad, inner_w, head_h));

        // A linha de dica do Falloff, logo abaixo do cabeçalho, apagada (é meta-informação, não um
        // parâmetro). O texto sai de i18n (HR-15), nunca hardcoded.
        if note_h > 0.0 {
            let note = match fx.falloff_role {
                state::FalloffRole::ModulatesBelow => tr("panel.vector.fx.falloff.modulates"),
                state::FalloffRole::Inert => tr("panel.vector.fx.falloff.inert"),
                state::FalloffRole::NotFalloff => "",
            };
            paint_text(
                self.text_system,
                self.scene,
                note,
                inner_x,
                y + pad + head_h + (note_h - self.font) * 0.5,
                self.font,
                inner_w,
                resolve(ColorToken::Text3, self.theme),
            );
        }

        // Os parâmetros, indentados dentro do card. Guardo e restauro a coluna: o `slider_row`
        // desenha no `inner_x`/`inner_w` do CONTEXTO, e a alternativa seria duplicá-lo.
        let (keep_x, keep_w) = (self.inner_x, self.inner_w);
        self.inner_x = inner_x;
        self.inner_w = inner_w;
        let mut py = y + pad + head_h + note_h;
        for (param, p) in fx.params.iter().enumerate().take(ids::MAX_FX_ROW_PARAMS) {
            py = self.fx_param(row, param, p, py);
        }
        self.inner_x = keep_x;
        self.inner_w = keep_w;

        y + card_h + Spacing::Sm.px()
    }

    /// O cabeçalho do card: o nome à esquerda, os ícones à direita.
    ///
    /// **Subir na primeira linha e descer na última não fazem nada — então não são desenhados**
    /// (o espaço fica vazio, e o ícone seguinte não salta de lugar, porque a posição de cada um
    /// é contada da DIREITA). Um ícone inerte ensina o artista a desconfiar dos que funcionam.
    fn effect_header(&mut self, row: usize, fx: &state::FxRowView, total: usize, at: Rect) {
        let (x, w, y, h) = (at.x, at.w, at.y, at.h);
        let dim = if fx.enabled {
            ColorToken::Text1
        } else {
            ColorToken::TextDisabled
        };
        paint_text(
            self.text_system,
            self.scene,
            fx.label,
            x,
            y + (h - self.font) * 0.5,
            self.font,
            w,
            resolve(dim, self.theme),
        );

        // Da DIREITA para a esquerda: ✕ · olho · ↓ · ↑. Assim o ✕ fica sempre no mesmo sítio,
        // e a ausência de uma seta nas bordas não desloca os outros.
        let slot = |i: usize| x + w - ICON_PX * (i as f32 + 1.0) - Spacing::Xs.px() * i as f32;
        self.icon(ids::vector_fx_remove_id(row), IconId::Close, slot(0), y, h);
        self.icon(
            ids::vector_fx_hide_id(row),
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
            self.icon(
                ids::vector_fx_down_id(row),
                IconId::ChevronDown,
                slot(2),
                y,
                h,
            );
        }
        if row > 0 {
            self.icon(ids::vector_fx_up_id(row), IconId::ChevronUp, slot(3), y, h);
        }
    }

    /// Um botão de ícone do cabeçalho: pinta e REGISTRA o hit-rect (sem ele não há Click).
    fn icon(&mut self, id: ph2d_a11y::NodeId, glyph: IconId, x: f32, y: f32, h: f32) {
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

    /// Um parâmetro: slider ou caixinha, conforme o efeito o DESCREVEU.
    fn fx_param(&mut self, row: usize, param: usize, p: &state::FxParamView, y: f32) -> f32 {
        let (slider, chip) = (
            ids::vector_fx_param_id(row, param),
            ids::vector_fx_param_num_id(row, param),
        );
        if p.toggle {
            // A caixinha é um botão cujo rótulo diz o ESTADO: um slider de dois valores seria
            // um controle contínuo a mentir sobre um fato binário.
            let on = p.value >= 0.5;
            return self.action_button(
                ids::vector_fx_toggle_id(row, param),
                &format!("{}: {}", p.name, if on { "On" } else { "Off" }),
                y,
            );
        }
        // O track é a posição NORMALIZADA na faixa; o número mostrado é o do documento. A
        // projeção entre os dois está registada no store (`seed_effect_ranges`), então o chip
        // mostra o valor do documento TAMBÉM durante o arrasto.
        let span = p.max - p.min;
        #[allow(clippy::cast_possible_truncation)]
        let track = if span > 0.0 {
            ((p.value - p.min) / span) as f32
        } else {
            0.0
        };
        // Uma CONTAGEM não tem casas decimais — o motor guarda o inteiro, e mostrar `8,00`
        // sugeriria que há meia crista algures.
        let display = if p.integer {
            format!("{:.0}", p.value)
        } else {
            format!("{:.DECIMALS$}", p.value)
        };
        self.slider_row(p.name, slider, chip, track, p.value, &display, y)
    }
}
