//! **A MONTAGEM da pilha** — a quota, o `+` e o `x` (ordem do dono, 2026-09-21).
//!
//! *«A seção nasce sem nenhuma camada. Teremos um botão + para criar camadas … o máximo que pode
//! ser criado é 3 de Brush, 2 de erase, 1 de Blur, 1 de Smear. As opções vão sumindo do dropdown à
//! medida que vão sendo usadas … cada camada passa a ter um x para ser retirada.»*
//!
//! ⭐ **O motor ficou INTOCADO por esta wave**, e a linha que o consegue é uma só: a cauda das
//! posições que não existem fica a `strength = 0`, que é como o laço da pilha já pulava uma
//! camada. *Quantas camadas existem* é uma pergunta do ARTISTA, não do carimbo.
//!
//! ⚠️ Corte por RESPONSABILIDADE do [`super::composite`] (que bateu `763` linhas contra o tecto de
//! `700`): lá mora *o que uma camada É*, aqui *como a pilha se monta*.

use super::composite::{CompositeLayer, CompositeOp, MAX_CAMADAS, N_OPERACOES, quota_da_operacao};
use crate::tool::PainterTool;

impl PainterTool {
    /// **Quantas camadas a pilha TEM** (`0..=MAX_CAMADAS`). ⚠️ Não confundir com [`MAX_CAMADAS`],
    /// que é quantas ela pode ter.
    pub fn composite_len(&self) -> usize {
        self.paint.composite_len
    }

    /// Quantas camadas desta operação já existem.
    fn quota_gasta(&self, op: CompositeOp) -> usize {
        self.paint.composite[..self.paint.composite_len]
            .iter()
            .filter(|l| l.op == op)
            .count()
    }

    /// **Que operações o menu do `+` ainda oferece** — a porta ÚNICA da quota.
    ///
    /// ⚠️ Ela viaja DERIVADA no instantâneo (`BrushSettings::composite_add_available`): o painel
    /// não recalcula a quota, ele pinta a resposta. *Uma segunda aritmética da quota no painel
    /// divergiria no dia em que uma quota mudasse.*
    pub(crate) fn ops_com_quota_livre(&self) -> [bool; N_OPERACOES] {
        std::array::from_fn(|i| {
            let op = CompositeOp::from_u8(i as u8);
            self.quota_gasta(op) < quota_da_operacao(op)
        })
    }

    /// Ainda cabe alguma camada? (o estado do `+` e do menu ao lado dele)
    pub fn pode_acrescentar_camada(&self) -> bool {
        self.ops_com_quota_livre().iter().any(|&x| x)
    }

    /// A operação que o menu do `+` tem escolhida.
    pub fn composite_add_op(&self) -> u8 {
        self.paint.composite_add_op.to_u8()
    }

    /// Escolher a operação do menu do `+`. ⚠️ Uma escolha sem quota é **recusada**, senão o `+`
    /// ficaria armado sobre uma operação que ele não pode criar.
    pub fn set_composite_add_op(&mut self, op: u8) {
        let op = CompositeOp::from_u8(op);
        if self.ops_com_quota_livre()[usize::from(op.to_u8())] {
            self.paint.composite_add_op = op;
        }
    }

    /// ⭐ **Criar uma camada** da operação escolhida, no FUNDO da pilha.
    ///
    /// ⚠️ **No fundo e não no topo:** a pilha corre de baixo para cima, logo uma camada nova
    /// entra por baixo do que já lá está e não tapa o trabalho anterior — e as setas ↑/↓ levam-na
    /// a qualquer lado. *O contrário faria toda camada nova esconder a pilha inteira.*
    ///
    /// ⚠️ Depois de criar, a escolha do menu SALTA para a primeira operação que ainda tem quota:
    /// sem isso o `+` ficaria armado sobre uma operação esgotada.
    pub fn acrescenta_camada(&mut self, op: u8) {
        let op = CompositeOp::from_u8(op);
        if self.paint.composite_len >= MAX_CAMADAS
            || !self.ops_com_quota_livre()[usize::from(op.to_u8())]
        {
            return;
        }
        let pos = self.paint.composite_len;
        self.paint.composite[pos] = CompositeLayer::nova(op);
        self.paint.composite_len += 1;
        self.reaponta_o_menu();
    }

    /// ⭐ **Retirar a camada `pos`** — as de baixo sobem e a cauda volta ao `default()`.
    ///
    /// ⛔ **A cauda tem de voltar a `strength = 0`**: o motor pula uma camada pelo Strength, e uma
    /// cauda com os bytes da camada retirada continuaria a pintar. *É a mesma linha que mantém o
    /// motor inteiro alheio a esta wave.*
    pub fn retira_camada(&mut self, pos: usize) {
        if pos >= self.paint.composite_len {
            return;
        }
        for i in pos..self.paint.composite_len - 1 {
            self.paint.composite[i] = self.paint.composite[i + 1];
        }
        self.paint.composite_len -= 1;
        self.paint.composite[self.paint.composite_len] = CompositeLayer::default();
        self.reaponta_o_menu();
    }

    /// Pôr o menu do `+` numa operação que ainda tenha quota (ou deixá-lo como está se nenhuma
    /// tiver — aí o `+` está desligado e a escolha não é alcançável).
    fn reaponta_o_menu(&mut self) {
        let livres = self.ops_com_quota_livre();
        if livres[usize::from(self.paint.composite_add_op.to_u8())] {
            return;
        }
        if let Some(i) = livres.iter().position(|&x| x) {
            self.paint.composite_add_op = CompositeOp::from_u8(i as u8);
        }
    }
}
