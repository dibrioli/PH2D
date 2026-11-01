//! A seção **EFFECTS** (ADR-0132) — a pilha de Live Path Effects do caminho selecionado.
//!
//! Módulo irmão do [`super`] pelo teto de 600 LOC, par do `populate_effects` (que REGISTRA os
//! widgets — sem isso ficam pintados e mortos).
//!
//! # Cada efeito é uma LINHA, e os parâmetros abrem à esquerda (F43, dono 01/11)
//!
//! Era um card por efeito com os parâmetros dentro e um botão «Add» por tipo à mostra (21). Agora
//! é o layout do Figma: o «+» do cabeçalho abre a lista dos tipos, cada efeito aplicado é uma linha
//! ([`super::fx_list`]) e clicar nela abre os parâmetros num painel à ESQUERDA do Inspector
//! ([`crate::fx_side`]), que chama [`BodyCtx::effect_props`].
//!
//! # Este arquivo não conhece nenhum efeito, e é esse o ponto
//!
//! Ele desenha **linhas** vindas do snapshot: um rótulo, os ícones e um controle por parâmetro
//! DESCRITO (nome, faixa, é-caixinha). Acrescentar um efeito ao motor não toca aqui.
//!
//! O **olho** desarma sem apagar, e os parâmetros de um efeito desarmado continuam editáveis:
//! desarmar não pode custar números que o artista teria de lembrar.

use super::fx_list::Pilha;
use super::*;
use ph2d_i18n::tr;

/// Quantas casas o chip mostra.
const DECIMALS: usize = 2;

impl BodyCtx<'_> {
    /// A seção Effects. Devolve o `y` intocado quando não há caminho único selecionado — é o
    /// que faz o `step` não emitir separador órfão.
    ///
    /// ⚠️⚠️ **E a guarda estava DEPOIS do cabeçalho, o que fazia este doc-comment mentir**: o
    /// `section_header` já tinha avançado o `y`, então com a seleção vazia o painel pintava o
    /// título *EFFECTS*, o separador de rodapé — e corpo nenhum. Foi a única das 39 a fazê-lo, e a
    /// nota que a descreve estava certa sobre a intenção e errada sobre o código
    /// (report do Enio, 2026-08-31: *"deixar no painel apenas o que é útil"*).
    /// *Um cabeçalho que promete um corpo que não existe é pior que uma seção ausente.*
    pub(crate) fn effects_section(&mut self, y: f32) -> f32 {
        if !state::has_target() {
            return y;
        }
        let stack = state::stack();
        let live = !state::bound() && stack.len() < ph2d_tool_vector::ids::MAX_FX_ROWS;
        let (mut y, collapsed) = self.fx_list_header(
            Pilha::Efeitos,
            ph2d_tool_vector::ids::VECTOR_SECTION_EFFECTS,
            tr("panel.vector.section.effects"),
            live,
            y,
        );
        if collapsed {
            return y;
        }
        // ⭐⭐ Uma forma PRESA a ossos não recebe efeitos — os que tinha foram cozidos no desenho ao
        // prender (ordem do dono, 2026-10-03). A secção diz porquê em vez de sumir.
        if state::bound() {
            paint_text(
                self.text_system,
                self.scene,
                tr("panel.vector.fx.bound"),
                self.inner_x,
                y + (self.row_h - self.font) * 0.5,
                self.font,
                self.inner_w,
                resolve(ColorToken::Text3, self.theme),
            );
            return y + self.row_h + self.row_gap;
        }
        for (row, fx) in stack
            .iter()
            .enumerate()
            .take(ph2d_tool_vector::ids::MAX_FX_ROWS)
        {
            y = self.fx_list_row(
                Pilha::Efeitos,
                row,
                state::row_icon(row),
                fx.label,
                fx.enabled,
                y,
            );
        }
        // **Apply**, no FIM da seção e em ACCENT: é a ação de *commit* (assa a pilha no cozido e a
        // esvazia — o *Expand Appearance*), então destaca-se das edições comuns e fecha a fila.
        // Só aparece quando há o que assar — um botão que esvazia uma pilha vazia seria inerte, e
        // um botão inerte ensina o artista a desconfiar dos que funcionam.
        if !stack.is_empty() {
            y = self.action_button_kind(
                ph2d_tool_vector::ids::VECTOR_FX_APPLY,
                tr("panel.vector.fx.apply_effects"),
                ButtonKind::Accent,
                y,
            );
        }
        y
    }

    /// **Os parâmetros do efeito da linha `row`** — o corpo do painel à esquerda
    /// ([`crate::fx_side`]). Devolve o `y` final.
    pub(crate) fn effect_props(&mut self, row: usize, fx: &state::FxRowView, y: f32) -> f32 {
        let mut y = y;
        // Um Falloff ganha uma LINHA de dica ("modula abaixo" / "adicione um deformador"): ele não
        // deforma nada sozinho, e sem a dica um Falloff sem deformador abaixo pareceria quebrado.
        let note = match fx.falloff_role {
            state::FalloffRole::ModulatesBelow => Some(tr("panel.vector.fx.falloff.modulates")),
            state::FalloffRole::Inert => Some(tr("panel.vector.fx.falloff.inert")),
            state::FalloffRole::NotFalloff => None,
        };
        if let Some(note) = note {
            paint_text(
                self.text_system,
                self.scene,
                note,
                self.inner_x,
                y + (self.row_h - self.font) * 0.5,
                self.font,
                self.inner_w,
                resolve(ColorToken::Text3, self.theme),
            );
            y += self.row_h;
        }
        for (param, p) in fx
            .params
            .iter()
            .enumerate()
            .take(ph2d_tool_vector::ids::MAX_FX_ROW_PARAMS)
        {
            y = self.fx_param(row, param, p, y);
        }
        y
    }

    /// Um parâmetro: slider ou caixinha, conforme o efeito o DESCREVEU.
    fn fx_param(&mut self, row: usize, param: usize, p: &state::FxParamView, y: f32) -> f32 {
        let (slider, chip) = (
            ph2d_tool_vector::ids::vector_fx_param_id(row, param),
            ph2d_tool_vector::ids::vector_fx_param_num_id(row, param),
        );
        if p.toggle {
            // A caixinha é um botão cujo rótulo diz o ESTADO: um slider de dois valores seria
            // um controle contínuo a mentir sobre um fato binário.
            let on = p.value >= 0.5;
            let tid = ph2d_tool_vector::ids::vector_fx_toggle_id(row, param);
            let (w, dot) = ph2d_editor_core::widget::form_row_columns(
                self.inner_x,
                self.inner_w,
                y,
                self.row_h,
            );
            self.decorator(dot, &[tid]);
            // O botão na coluna do controlo (o idioma do card: guardar e repor a coluna).
            let keep = std::mem::replace(&mut self.inner_w, w);
            let ny = self.action_button(
                tid,
                &format!(
                    "{}: {}",
                    p.name,
                    if on {
                        tr("panel.vector.fx.on")
                    } else {
                        tr("panel.vector.fx.off")
                    }
                ),
                y,
            );
            self.inner_w = keep;
            return ny;
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
