//! **A medida de uma linha de tinta** — as colunas, quando ela parte em duas filas e o Weight que ela
//! MOSTRA (a fonte única que a linha pinta e que a lista mede). Filho do [`super`] (tecto de 600 LOC
//! por ficheiro de painel, integ 02/11).

use super::*;
use ph2d_card_stack::metrics as m;
use ph2d_editor_core::widget::card_ink::card_body;

/// Os três botões (⚙ 👁 −) têm a largura [`BodyCtx::botao_w`]; a cor é um quadrado `h`.
const QUADRADOS: u8 = 3;
/// Losangos há quatro (cor, Weight, %, olho).
const LOSANGOS: u8 = 4;

/// ⭐ **O Weight que uma linha MOSTRA** — a fonte única que a linha pinta e que a lista mede para a
/// altura dela ([`BodyCtx::peso_base`], [`BodyCtx::peso_camada`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::paint_sections) struct PesoValor {
    pub(in crate::paint_sections) id: NodeId,
    pub(in crate::paint_sections) px: f64,
    pub(in crate::paint_sections) mixed: bool,
}

/// ⭐ **As colunas de uma linha**, da direita para a esquerda — as MESMAS em toda linha de Fill e
/// de Stroke: `[cor]◇ [Weight]◇ [%]◇ [⚙] [👁]◇ [−]`; a cor é sempre um quadrado, e num
/// preenchimento a coluna do Weight fica vazia.
#[derive(Clone, Copy)]
pub(in crate::paint_sections) struct Colunas {
    pub(in crate::paint_sections) caixa: Rect,
    pub(in crate::paint_sections) caixa_key: Rect,
    pub(in crate::paint_sections) weight: Rect,
    pub(in crate::paint_sections) weight_key: Rect,
    pub(in crate::paint_sections) pct: Rect,
    pub(in crate::paint_sections) pct_key: Rect,
    pub(in crate::paint_sections) settings: Rect,
    pub(in crate::paint_sections) eye: Rect,
    pub(in crate::paint_sections) eye_key: Rect,
    pub(in crate::paint_sections) minus: Rect,
}

impl BodyCtx<'_> {
    /// ⭐ **A linha cabe numa fila?** (dono 02/11: *«parte só quando precisa»*). As partes fixas (a cor,
    /// os quatro losangos, ⚙ 👁 −) mais o `%` e o Weight MEDIDOS — o Weight pelo número que ele mostra
    /// AGORA. No degrau estreito (`inner_w` 168, integ 02/11) o Weight ficava com 1 px e o número
    /// pintava em nada; na largura de fábrica o valor padrão cabe e a linha fica numa fila (F44c), e
    /// só um valor que não cabe (`100`) a parte. Um preenchimento (sem Weight) nunca parte.
    pub(in crate::paint_sections) fn linha_dupla(&mut self, peso: Option<PesoValor>) -> bool {
        let Some(peso) = peso else {
            return false;
        };
        let (h, b, mw) = (self.row_h, self.botao_w(), self.decorator_w());
        let fixo = h + f32::from(QUADRADOS) * b + f32::from(LOSANGOS) * mw;
        let pct_w = self.pct_w();
        let weight_w = self.weight_min_w(peso);
        self.inner_w < fixo + pct_w + weight_w
    }

    /// O Weight do traço de BASE: o número em edição, senão a ficha.
    pub(in crate::paint_sections) fn peso_base(&self, snap: &VectorStyleSnapshot) -> PesoValor {
        let id = ph2d_tool_vector::ids::VECTOR_WIDTH_NUM;
        PesoValor {
            id,
            px: self.store.number_value(id).unwrap_or(snap.stroke_width_px),
            mixed: !snap.agrees(StyleFields::WIDTH),
        }
    }

    /// O Weight da camada `i` (índice do DOCUMENTO).
    pub(in crate::paint_sections) fn peso_camada(&self, i: usize, row: &PaintRow) -> PesoValor {
        let id = ids::vector_paint_row_width_id(i);
        PesoValor {
            id,
            px: self.live_number(id, row.width),
            mixed: row.mixed.width,
        }
    }

    /// A altura de UMA linha de tinta — uma fila, ou duas quando o Weight não cabe. ⚠️ A lista (passo
    /// do arrasto, vão, linha que voa) lê ESTA com o MESMO [`PesoValor`] que a linha pinta.
    pub(in crate::paint_sections) fn linha_h(&mut self, peso: Option<PesoValor>) -> f32 {
        if self.linha_dupla(peso) {
            self.row_h * 2.0 + self.row_gap
        } else {
            self.row_h
        }
    }

    /// A largura de um botão da linha: a célula do cartão, ou o quadrado da linha de antes.
    fn botao_w(&self) -> f32 {
        if card_body().is_some() {
            m::ICON_CELL
        } else {
            self.row_h
        }
    }

    fn decorator_w(&self) -> f32 {
        if card_body().is_some() {
            m::KEY_MARK_BOX
        } else if ph2d_editor_core::paint::ui_is_redesign() {
            ph2d_editor_core::widget::DECORATOR_W
        } else {
            0.0
        }
    }

    /// As colunas da linha em `y`. O losango reserva a largura dele mesmo sem canal (alinhamento).
    /// Na linha dupla a 2.ª fila (sob a cor) leva `[Weight]◇ [%]◇`.
    pub(in crate::paint_sections) fn colunas(
        &mut self,
        y: f32,
        peso: Option<PesoValor>,
    ) -> Colunas {
        let h = self.row_h;
        let b = self.botao_w();
        let mw = self.decorator_w();
        let dupla = self.linha_dupla(peso);
        let right = self.inner_x + self.inner_w;
        // Os três botões encostados (o glifo já tem a folga dele): a caixa do hex fica com a largura.
        let minus = Rect::new(right - b, y, b, h);
        let eye_key = Rect::new(minus.x - mw, y, mw, h);
        let eye = Rect::new(eye_key.x - b, y, b, h);
        let settings = Rect::new(eye.x - b, y, b, h);
        let pct_w = self.pct_w();
        let (y2, fim) = if dupla {
            (y + h + self.row_gap, right)
        } else {
            (y, settings.x)
        };
        let pct_key = Rect::new(fim - mw, y2, mw, h);
        let pct = Rect::new(pct_key.x - pct_w, y2, pct_w, h);
        let weight_key = Rect::new(pct.x - mw, y2, mw, h);
        let caixa = Rect::new(self.inner_x, y, h, h);
        let caixa_key = Rect::new(caixa.x + caixa.w, y, mw, h);
        let wx = if dupla {
            self.inner_x
        } else {
            caixa_key.x + caixa_key.w
        };
        let weight = Rect::new(wx, y2, (weight_key.x - wx).max(1.0), h);
        Colunas {
            caixa,
            caixa_key,
            weight,
            weight_key,
            pct,
            pct_key,
            settings,
            eye,
            eye_key,
            minus,
        }
    }

    /// ⭐ **A largura do `%`, MEDIDA** — o maior valor («100 %») com a folga e o stepper do campo, nunca
    /// acima do piso genérico dos números. É o que dá à caixa do hex a largura dos seis dígitos no
    /// Inspector estreito (o piso de `72 px` é do maior número da casa, e aqui o maior é `100`).
    fn pct_w(&mut self) -> f32 {
        if card_body().is_some() {
            return self.campo_do_cartao_w("100");
        }
        self.campo_w(&format!("100{}", Unit::Percent.suffix()))
    }

    /// O mínimo do Weight, MEDIDO sobre o número que ele mostra agora — sem o «px» (a unidade vive no
    /// rótulo, nunca na caixa, regra do dono). Em repouso o pintor perde CASAS, nunca dígitos
    /// (`numero_que_cabe`), logo o que não pode faltar é o inteiro arredondado; a editar, o buffer
    /// inteiro; misto, nada.
    fn weight_min_w(&mut self, peso: PesoValor) -> f32 {
        if card_body().is_some() {
            return self.weight_do_cartao_w(peso);
        }
        let editando = self.store.focus_id() == Some(peso.id);
        let maior = if editando {
            let (_, _, buffer, _, _) =
                ph2d_editor_core::widget::showcase::read_number_input(self.store, peso.id);
            buffer.to_string()
        } else if peso.mixed {
            String::new()
        } else {
            ph2d_editor_core::widget::format_number(peso.px.round())
        };
        // A conta do PINTOR do número (`paint_number_input_with_buffer`): o texto em `Sm`, a folga
        // da esquerda e o stepper — à direita só as setas ocluem o número.
        let host = Rect::new(0.0, 0.0, self.chip_w, self.row_h);
        self.text_system.prefix_width_weighted(
            &maior,
            ph2d_tokens::TypeToken::Sm.px(),
            ph2d_text::FontWeight::MEDIUM,
        ) + ph2d_editor_core::widget::field_pad_x()
            + ph2d_editor_core::widget::stepper_width(host)
    }

    /// O campo do cartão que mostra `texto` inteiro: a pega, o texto e a folga da direita.
    fn campo_do_cartao_w(&mut self, texto: &str) -> f32 {
        m::ICON_CELL + self.text_system.prefix_width(texto, m::VALUE_PX).ceil() + m::FIELD_PAD_R
    }

    /// O campo do Weight no cartão: pelo que ele MOSTRA agora (a casa decimal conta: nunca se corta).
    fn weight_do_cartao_w(&mut self, peso: PesoValor) -> f32 {
        let texto = if self.store.focus_id() == Some(peso.id) {
            let (_, _, buffer, _, _) =
                ph2d_editor_core::widget::showcase::read_number_input(self.store, peso.id);
            buffer.to_string()
        } else if peso.mixed {
            tr("card.field.mixed").to_string()
        } else {
            ph2d_editor_core::widget::format_number(peso.px)
        };
        self.campo_do_cartao_w(&texto)
    }

    /// Um campo numérico que mostra `texto` inteiro: o texto, a folga dos dois bordos e o stepper.
    fn campo_w(&mut self, texto: &str) -> f32 {
        let medido =
            self.text_system
                .prefix_width_weighted(texto, self.font, ph2d_text::FontWeight::MEDIUM);
        let host = Rect::new(0.0, 0.0, self.chip_w, self.row_h);
        let w = medido
            + ph2d_editor_core::widget::field_pad_x() * 2.0
            + ph2d_editor_core::widget::stepper_width(host);
        w.min(self.chip_w)
    }
}
