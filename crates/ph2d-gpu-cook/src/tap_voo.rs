//! ⭐⭐⭐ **A leitura dos cartões SEM ESPERAR pela placa** — o [`GpuCook::tap`] partido em
//! «encomendar» e «recolher» (ciclo 12, doc 120 §8.6).
//!
//! ## O defeito, medido
//!
//! O [`GpuCook::tap`] é síncrono: submete o gather, mapeia e faz `poll(wait_indefinitely)`. O
//! cabeçalho do [`crate::tap`] mediu-o a `+0,075 ms` num arnês sem janela — e ali **não há mais
//! nada na fila**. No app a fila tem o quadro anterior inteiro (o desenho, o cozimento), e o `poll`
//! espera por TUDO: na escada dos tectos (RTX, `32 768` imagens) ele custava **`0,68 ms` por
//! quadro**, `60 %` do Motion que sobrava depois do carimbo ir para a placa. *O custo não era a
//! largura de banda; era a CPU parada à espera da placa, que é serializar os dois processadores.*
//!
//! ## A lei
//!
//! Um pedido em voo de cada vez. Cada chamada de [`GpuCook::tap_sem_espera`]:
//!
//! 1. **recolhe** o pedido anterior se a placa já o acabou (`poll(Poll)` não bloqueia) — senão
//!    deixa-o em voo e **não** encomenda outro (a fila não cresce com uma placa lenta);
//! 2. **encomenda** o seguinte sobre os streams DESTE cozimento;
//! 3. devolve a **última leitura completa**.
//!
//! ⇒ os cartões ficam **um quadro mais atrás** (já eram um — ver o `stamp` da ponte). ⚠️ E a
//! leitura é DESTE caminho: quem deixa de conduzir pela placa tem de chamar
//! [`GpuCook::descarta_tap_em_voo`], senão um regresso à placa mostraria números de há minutos.
//!
//! ⛔ O [`GpuCook::tap`] síncrono **fica**, e não por preguiça: os gates e as sondas querem a
//! leitura DESTE cozimento, agora — é a pergunta deles —, e as duas metades partilhadas
//! ([`GpuCook::encomenda_tap`] · [`crate::tap::le_tap`]) garantem que as duas rotas leem as MESMAS
//! amostras.

use crate::GpuCook;
use crate::tap::{Slot, le_tap};
use ph2d_gpu::GpuContext;
use ph2d_nodegraph::attr::Stream;
use ph2d_nodegraph::graph::NodeId;
use std::collections::BTreeMap;
use std::sync::mpsc::{Receiver, TryRecvError};

/// O estado da leitura sem espera: o pedido em voo e a última leitura completa.
#[derive(Default)]
pub(crate) struct TapVoo {
    em_voo: Option<EmVoo>,
    ultimo: Option<BTreeMap<NodeId, Stream>>,
}

struct EmVoo {
    staging: wgpu::Buffer,
    slots: Vec<Slot>,
    pronto: Receiver<Result<(), wgpu::BufferAsyncError>>,
}

impl GpuCook {
    /// **A leitura dos cartões sem esperar pela placa** — ver o cabeçalho do módulo.
    ///
    /// `None` até à primeira leitura completa (o primeiro quadro na placa não tem números nos
    /// cartões; o seguinte tem).
    pub fn tap_sem_espera(
        &mut self,
        gpu: &GpuContext,
        samples: u32,
    ) -> Option<BTreeMap<NodeId, Stream>> {
        // ⛔ `Poll` e NUNCA `wait`: esperar aqui é o defeito que este módulo existe para curar.
        let _ = gpu.device.poll(wgpu::PollType::Poll);
        if let Some(voo) = &self.tap_voo.em_voo {
            match voo.pronto.try_recv() {
                Ok(Ok(())) => {
                    let voo = self.tap_voo.em_voo.take().expect("acabou de ser lido");
                    self.tap_voo.ultimo = Some(le_tap(&voo.staging, &voo.slots));
                }
                // A placa recusou o mapeamento (ou o pedido morreu): esquece-o e encomenda outro.
                Ok(Err(_)) | Err(TryRecvError::Disconnected) => self.tap_voo.em_voo = None,
                // Ainda na placa: fica em voo, e não se encomenda um segundo.
                Err(TryRecvError::Empty) => {}
            }
        }
        if self.tap_voo.em_voo.is_none()
            && let Some((staging, slots)) = self.encomenda_tap(gpu, samples)
        {
            let (tx, rx) = std::sync::mpsc::channel();
            staging.slice(..).map_async(wgpu::MapMode::Read, move |r| {
                let _ = tx.send(r);
            });
            self.tap_voo.em_voo = Some(EmVoo {
                staging,
                slots,
                pronto: rx,
            });
        }
        self.tap_voo.ultimo.clone()
    }

    /// **Esquece o pedido em voo e a última leitura** — para quem deixa de conduzir pela placa.
    pub fn descarta_tap_em_voo(&mut self) {
        self.tap_voo = TapVoo::default();
    }

    /// Há um pedido na placa por recolher? — para o gate afirmar «um de cada vez».
    #[must_use]
    pub fn tap_em_voo(&self) -> bool {
        self.tap_voo.em_voo.is_some()
    }
}

#[cfg(test)]
#[path = "tap_voo_tests.rs"]
mod tests;
