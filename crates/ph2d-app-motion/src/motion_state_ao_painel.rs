//! **O que as portas partilhadas pedem ao PAINEL do editor** (ADR-0186 §Emenda W3, «A fronteira»):
//! guardado na instância e publicado só pela porta do editor (`motion_bridge::publica_no_painel`). O
//! palco de uma macro corre as mesmas portas e não tem painel — os pedidos dele ficam nele.

use crate::MotionParamIntent;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PedidosAoPainel {
    /// A selecção que uma edição entrega ao painel (as cópias de um Ctrl+D, o nó inserido); a última
    /// ganha, como no canal do painel.
    pub selecao: Option<Vec<u32>>,
    /// As edições de param que um lote do grafo encaminha ao dreno de params, pela ordem.
    pub params: Vec<MotionParamIntent>,
}

impl super::MotionState {
    pub(crate) fn pede_selecao(&mut self, nos: Vec<u32>) {
        self.ao_painel.selecao = Some(nos);
    }

    pub(crate) fn encaminha_param(&mut self, intent: MotionParamIntent) {
        self.ao_painel.params.push(intent);
    }
}
