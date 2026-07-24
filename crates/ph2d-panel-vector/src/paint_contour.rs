//! A seção **CONTOUR** do painel Vector — módulo irmão do [`super`] (teto de 600 LOC).
//!
//! N anéis concêntricos com uma progressão de cor (pesquisa `20_*` #9): o efeito que a Corel
//! publica como não tendo equivalente no Illustrator, e que a pesquisa resume em *"não é o nosso
//! offset — é offset × N + cor"*. Os anéis são DESENHO derivado (`contour_live`) — a curva que o
//! modo Node edita nunca é tocada.
//!
//! **Esta seção é a única porta do produto para o efeito.** Sem ela o motor existiria, gateado e
//! smokado, e a feature não existiria para o artista.
//!
//! # As duas caras da seção, e por que a swatch só existe na segunda
//!
//! Sem contour, a seção mostra **um** botão: *Add Contour*. Com contour, mostra os controles + os
//! dois comandos de saída. A razão é a swatch de cor-alvo: um controle de cor sem alvo é o knob
//! morto na sua forma mais cara — abre um picker inteiro para depois descartar a escolha. O botão
//! explícito faz *"existe onde escrever"* ser verdade antes de a swatch ser oferecida.
//!
//! É a mesma forma da seção Pattern on Path (botão de prender × controles + Detach), e a mesma
//! lei do `Join Selected Bodies`: cada porta é oferecida só na seleção em que funciona.

use super::*;
use crate::contour_params::{accel_to_track, d_to_track, steps_to_track};
/// O módulo INTEIRO, e não os getters um a um: são sete nomes genéricos (`steps`, `accel`,
/// `join`, `side`, `to`…) que colidiriam com os das outras seções se subissem soltos. A
/// qualificação (`cst::join()`) diz de que seção o número é.
use crate::state::contour as cst;

/// Quantos por cento é uma unidade de fração — o fator do readout, não uma medida de desenho.
const PERCENT: f64 = 100.0; // LITERAL-PX-OK: conversão de unidade (fração -> percentual)

impl BodyCtx<'_> {
    /// Seção **CONTOUR**.
    pub(crate) fn contour_section(&mut self, y: f32) -> f32 {
        // Só quando há o que dizer: um contour vivo, ou a seleção que permite criar um. Fora
        // disso o cabeçalho nem sobe.
        if !cst::present() && !cst::can_add() {
            return y;
        }
        let (mut y, collapsed) = self.section_header(
            ids::VECTOR_SECTION_CONTOUR,
            tr("panel.vector.section.contour"),
            y,
        );
        if collapsed {
            return y;
        }
        if !cst::present() {
            return self.action_button(ids::VECTOR_CONTOUR_ADD, "Add Contour", y);
        }
        // Steps — quantos anéis. O controle-assinatura: é ele que distingue este efeito do
        // Offset da seção Expand, que é o caso `steps = 1` sem rampa de cor.
        let steps = self
            .store
            .number_value(ids::VECTOR_CONTOUR_STEPS_NUM)
            .unwrap_or_else(cst::steps);
        let steps_track = self
            .store
            .slider(ids::VECTOR_CONTOUR_STEPS)
            .map_or_else(|| steps_to_track(cst::steps()), |(_, v)| v);
        y = self.slider_row(
            "Steps",
            ids::VECTOR_CONTOUR_STEPS,
            ids::VECTOR_CONTOUR_STEPS_NUM,
            steps_track,
            steps,
            &format!("{}", steps.round() as i64),
            y,
        );
        // Offset — a distância POR PASSO, bipolar, em PERCENTUAL do tamanho da forma. Percentual
        // porque o mapa do store é estático: um rótulo em unidades de mundo mentiria a cada troca
        // de seleção (a razão que a seção Expand já paga no mesmo painel).
        let d_pct = self
            .store
            .number_value(ids::VECTOR_CONTOUR_OFFSET_NUM)
            .unwrap_or_else(|| cst::d_frac() * PERCENT);
        let d_track = self
            .store
            .slider(ids::VECTOR_CONTOUR_OFFSET)
            .map_or_else(|| d_to_track(cst::d_frac()), |(_, v)| v);
        y = self.slider_row(
            "Offset",
            ids::VECTOR_CONTOUR_OFFSET,
            ids::VECTOR_CONTOUR_OFFSET_NUM,
            d_track,
            d_pct,
            &format!("{d_pct:.1}"),
            y,
        );
        // Accel — a progressão: `1` linear, `>1` espalha, `<1` amontoa. O trilho é GEOMÉTRICO
        // (o neutro cai no centro), então o campo numérico NÃO está ligado ao slider pelo mapa
        // afim do store — quem os casa é o `event.rs`. Ver `populate_contour`.
        let accel = self
            .store
            .number_value(ids::VECTOR_CONTOUR_ACCEL_NUM)
            .unwrap_or_else(cst::accel);
        let accel_track = self
            .store
            .slider(ids::VECTOR_CONTOUR_ACCEL)
            .map_or_else(|| accel_to_track(cst::accel()), |(_, v)| v);
        y = self.slider_row(
            "Accel",
            ids::VECTOR_CONTOUR_ACCEL,
            ids::VECTOR_CONTOUR_ACCEL_NUM,
            accel_track,
            accel,
            &format!("{accel:.2}"),
            y,
        );
        y = self.contour_to_swatch(y);
        // ⚠️ O rótulo é **"Corner"**, e não "Join": a seção Stroke, no MESMO painel, tem uma
        // fileira "Join · Miter/Round/Bevel" idêntica (a quina do TRAÇO). Duas fileiras gêmeas
        // para perguntas diferentes é como o clique do artista cai na errada — a mesma decisão
        // que a seção Expand tomou, e pela mesma razão.
        let join = cst::join();
        y = self.segmented3(
            "Corner",
            [
                (ids::VECTOR_CONTOUR_JOIN_MITER, "Miter", join == 0),
                (ids::VECTOR_CONTOUR_JOIN_ROUND, "Round", join == 1),
                (ids::VECTOR_CONTOUR_JOIN_BEVEL, "Bevel", join == 2),
            ],
            y,
        );
        let side = cst::side();
        y = self.segmented3(
            "Side",
            [
                (ids::VECTOR_CONTOUR_SIDE_OUTER, "Outer", side == 0),
                (ids::VECTOR_CONTOUR_SIDE_INNER, "Inner", side == 1),
                (ids::VECTOR_CONTOUR_SIDE_BOTH, "Both", side == 2),
            ],
            y,
        );
        // As duas saídas, e elas fazem coisas OPOSTAS: uma entrega os anéis como formas de
        // verdade, a outra os apaga. Ficam nesta ordem porque Expand é a que se procura (é o
        // que se faz com um contour pronto) e Remove é a que se lamenta ter clicado.
        y = self.action_button(ids::VECTOR_CONTOUR_EXPAND, "Expand Contour", y);
        self.action_button(ids::VECTOR_CONTOUR_REMOVE, "Remove Contour", y)
    }

    /// A fileira da cor-alvo: rótulo + swatch que abre o picker OKLCH partilhado.
    ///
    /// Espelha a fileira de Fill (mesmo `ColorSwatch`, mesmo tamanho, mesma coluna à direita) —
    /// uma segunda estética para a mesma pergunta *"que cor?"* é como o artista deixa de
    /// reconhecer o controle.
    fn contour_to_swatch(&mut self, y: f32) -> f32 {
        let swatch_w = SwatchSize::Md.px();
        paint_text(
            self.text_system,
            self.scene,
            "To",
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
        let swatch = ColorSwatch::new(ids::VECTOR_CONTOUR_TO, "Contour target color", cst::to())
            .size(SwatchSize::Md);
        paint_color_swatch(&swatch, rect, self.scene, self.theme);
        self.hit_index.register(ids::VECTOR_CONTOUR_TO, rect);
        y + self.row_h + self.row_gap
    }
}
