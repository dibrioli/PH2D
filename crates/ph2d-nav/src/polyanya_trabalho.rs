//! ⭐ (W14) **O TRABALHO de uma procura** — a unidade do orçamento da fila do replaneio
//! ([`crate::refresh`]), em NÓS UNIFORMES: o que custa expandir um nó da procura sem áreas.
//!
//! Um nó da procura PONDERADA custa mais, e não um factor fixo: quanto mais frentes na mesma aresta (a
//! dominância percorre-as) e quanto mais raízes de fronteira se materializam, mais caro (plano 30
//! §22.1). O trabalho soma as três contagens com pesos MEDIDOS; sem áreas, é o número de nós ao bit.
//! ⚠️ Só contagens: nada lê um relógio, e o replay serve a mesma fila.

use super::Stats;

/// Os pesos, em oitavos de um nó uniforme. Medidos (`sonda_custo_por_no_w14`, plano 30 §22.1):
/// mínimos quadrados do tempo de cada consulta sobre as três contagens — nó expandido `≈127 ns`, raiz
/// materializada `≈590`, frente comparada `≈18`.
const OITAVOS_POR_PENDENTE: u64 = 37;
const OITAVOS_POR_FRENTE: u64 = 1;

impl Stats {
    /// O trabalho acumulado nestas contagens, em nós uniformes (a diferença entre duas leituras é o
    /// de uma procura).
    #[must_use]
    pub fn work(&self) -> u64 {
        self.expanded
            + (OITAVOS_POR_PENDENTE * self.pending + OITAVOS_POR_FRENTE * self.compared) / 8
    }
}
