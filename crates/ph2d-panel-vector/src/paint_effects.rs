//! A seção **EFFECTS** (ADR-0132) — a pilha de Live Path Effects do caminho selecionado.
//!
//! Módulo irmão do [`super`] pelo teto de 600 LOC, par do `populate_effects` (que REGISTRA os
//! widgets — sem isso ficam pintados e mortos).
//!
//! # Este arquivo não conhece nenhum efeito, e é esse o ponto
//!
//! Ele desenha **linhas** vindas do snapshot: um rótulo, os botões de ordem e remoção, e um
//! controle por parâmetro DESCRITO (nome, faixa, é-caixinha). Acrescentar um efeito ao motor
//! não toca aqui — que é a diferença entre a promessa do ADR-0132 e o que ela de facto
//! entregava antes deste refactor: o 1º efeito custou uma rodada dos 8 sites de costura, e o
//! 2º ia custar outra.
//!
//! A **ordem** ganha botões porque a ordem MUDA a geometria (há gate com dois tipos: ondular e
//! depois cortar não é o mesmo que cortar e depois ondular). Sem eles, mudar de ideia obrigaria
//! a remover e re-adicionar.

use super::*;

/// Quantas casas o chip mostra. Os parâmetros vão de frações a contagens, e duas casas cobrem
/// as duas sem transformar "8 cristas" em algo com cauda decimal.
const DECIMALS: usize = 2;

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
            y = self.effect_row(row, fx, stack.len(), y);
        }
        // Os "Add": um por tipo PUBLICADO. A tabela vem do motor, então um efeito novo aparece
        // aqui sem este arquivo saber que ele existe.
        if stack.len() < ids::MAX_FX_ROWS {
            for (kind, name) in state::kinds().iter().enumerate().take(ids::MAX_FX_KINDS) {
                y = self.action_button(ids::vector_fx_add_id(kind), &format!("Add {name}"), y);
            }
        }
        y
    }

    /// Uma linha da pilha: o rótulo, os controles de ordem/remoção e os parâmetros.
    fn effect_row(&mut self, row: usize, fx: &state::FxRowView, total: usize, y: f32) -> f32 {
        let mut y = self.action_button(ids::vector_fx_remove_id(row), fx.label, y);
        // Subir na primeira linha e descer na última não fazem NADA — então não são oferecidos.
        // Um botão inerte ensina o artista a desconfiar dos que funcionam.
        let mut live: Vec<(ph2d_a11y::NodeId, &str)> = Vec::new();
        if row > 0 {
            live.push((ids::vector_fx_up_id(row), "Up"));
        }
        if row + 1 < total {
            live.push((ids::vector_fx_down_id(row), "Down"));
        }
        for pair in live.chunks(2) {
            y = match pair {
                [a, b] => self.row2(self.inner_w, Spacing::Xs.px(), [*a, *b], y),
                [a] => self.action_button(a.0, a.1, y),
                _ => y,
            };
        }
        for (param, p) in fx.params.iter().enumerate().take(ids::MAX_FX_ROW_PARAMS) {
            y = self.fx_param(row, param, p, y);
        }
        y
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
                slider,
                &format!("{}: {}", p.name, if on { "On" } else { "Off" }),
                y,
            );
        }
        // O track é a posição NORMALIZADA na faixa; o número mostrado é o do documento. A
        // conversão de volta mora no `event.rs`, que é a fronteira onde a unidade é decidida.
        let span = p.max - p.min;
        #[allow(clippy::cast_possible_truncation)]
        let track = if span > 0.0 {
            ((p.value - p.min) / span) as f32
        } else {
            0.0
        };
        self.slider_row(
            p.name,
            slider,
            chip,
            track,
            p.value,
            &format!("{:.DECIMALS$}", p.value),
            y,
        )
    }
}
