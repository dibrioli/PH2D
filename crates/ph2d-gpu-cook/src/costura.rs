//! **AS COSTURAS QUE ATRAVESSAM** — o passo CPU→GPU de um quadro (ciclo 8, W1 —
//! [doc 113 §6](../../../docs/Motion%20Nodes/113_ciclo_8_fontes_e_dados.md)).
//!
//! ⚠️ Irmão do `lib.rs` pelo tecto de LOC (HR-18) e por ASSUNTO: ali está a ORDEM do quadro (o
//! `cook`), aqui *o que atravessa a fronteira e a que preço*. O corte foi forçado pelo tecto e é
//! melhor por isso — a reutilização e o descarte de uma costura são uma pergunta só, e agora ela
//! tem um sítio.

use crate::{GpuCook, GpuStream, stream};
use ph2d_gpu::GpuContext;
use ph2d_nodegraph::attr::Stream;
use ph2d_nodegraph::graph::NodeId;
use std::collections::BTreeMap;

impl GpuCook {
    /// Sobe as costuras deste quadro, reaproveitando as que não mudaram.
    pub(crate) fn enviar_costuras(
        &mut self,
        gpu: &GpuContext,
        boundary_streams: &[(NodeId, &Stream)],
    ) -> BTreeMap<NodeId, GpuStream> {
        // The CPU→GPU crossings: one upload per boundary node, before anything
        // is encoded (a node consumed twice uploads once).
        //
        // ⭐⭐⭐ **E a costura que NÃO MUDOU não volta a atravessar** (ciclo 8, W1 — doc 113 §6): uma
        // fonte parada (uma tabela, um texto, uma forma) partilha as alocações com o que já foi
        // enviado, e o envio do quadro anterior serve. Medido no produto: `1 000 000` de linhas
        // paradas custavam o envio inteiro, a cada quadro.
        //
        // ⚠️ **Reutilizar é seguro porque nenhum estágio escreve no que recebe** — cada kernel
        // escreve buffers FRESCOS (o ping-pong implícito deste módulo), e há gate a prová-lo com
        // dois quadros seguidos sobre a mesma costura.
        let mut uploaded: BTreeMap<NodeId, GpuStream> = BTreeMap::new();
        for (node, s) in boundary_streams {
            let reaproveita = self
                .sent_boundaries
                .get(node)
                .filter(|(antes, _)| antes.shares_storage_with(s))
                .map(|(_, g)| g.clone());
            let g = match reaproveita {
                Some(g) => {
                    self.boundary_reuses += 1;
                    g
                }
                None => {
                    let g = stream::upload_stream(gpu, &mut self.pool, s);
                    self.boundary_uploads += 1;
                    self.sent_boundaries
                        .insert(*node, ((*s).clone(), g.clone()));
                    g
                }
            };
            uploaded.insert(*node, g);
        }
        // ⚠️ **E o que deixou de ser costura larga-se**: um plano novo (o artista reescreveu o
        // grafo) não pode manter vivos os buffers de uma fronteira que já não existe.
        self.sent_boundaries
            .retain(|n, _| boundary_streams.iter().any(|(m, _)| m == n));
        uploaded
    }
}
