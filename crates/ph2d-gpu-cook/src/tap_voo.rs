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
//! 2. **regista o PEDIDO**: o cozimento seguinte codifica a leitura no SEU encoder e ela vai no
//!    MESMO `submit` que ele (doc 122 §5 — um `submit` próprio por quadro custava `~0,02 ms` de CPU
//!    na NVIDIA). Só o ARRANQUE — um cozimento que aconteceu antes de haver pedido — encomenda
//!    pelo próprio `submit` ([`GpuCook::encomenda_tap`]);
//! 3. devolve a **última leitura completa**.
//!
//! ⇒ os cartões ficam **um quadro mais atrás** (já eram um — ver o `stamp` da ponte ⇒ **dois no total**). ⚠️ E a
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
    /// A última chamada RECOLHEU uma leitura nova? — ver [`GpuCook::tap_fresca`].
    fresca: bool,
    /// As amostras que a última chamada pediu — `Some` ⇒ o cozimento codifica a leitura.
    pedido: Option<u32>,
    /// O último cozimento já tinha o pedido? `false` ⇒ ele não codificou leitura nenhuma, e a
    /// chamada seguinte encomenda-a ela (o arranque).
    cozimento_viu_pedido: bool,
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
        self.tap_voo.fresca = false;
        if let Some(voo) = &self.tap_voo.em_voo {
            match voo.pronto.try_recv() {
                Ok(Ok(())) => {
                    let voo = self.tap_voo.em_voo.take().expect("acabou de ser lido");
                    self.tap_voo.ultimo = Some(le_tap(&voo.staging, &voo.slots));
                    self.tap_voo.fresca = true;
                }
                // A placa recusou o mapeamento (ou o pedido morreu): esquece-o e encomenda outro.
                Ok(Err(_)) | Err(TryRecvError::Disconnected) => {
                    self.tap_voo.em_voo = None;
                    self.solta_a_leitura_guardada();
                }
                // Ainda na placa: fica em voo, e não se encomenda um segundo.
                Err(TryRecvError::Empty) => {}
            }
        }
        self.tap_voo.pedido = Some(samples);
        if self.tap_voo.em_voo.is_none() && !self.tap_voo.cozimento_viu_pedido {
            // O ARRANQUE: o cozimento que temos aconteceu sem pedido — encomenda-se aqui, uma vez.
            self.tap_voo.cozimento_viu_pedido = true;
            let Some((staging, slots)) = self.encomenda_tap(gpu, samples, true) else {
                // ⛔ Nada a amostrar (todo stream vazio — um emissor rebobinado ao zero): a leitura
                // antiga NÃO fica — o `tap` síncrono também devolve nada aqui, e um cartão com os
                // números de antes do rebobinar mente (auditoria do fecho, 2026-09-24).
                self.tap_voo.ultimo = None;
                return None;
            };
            self.arma_o_tap(staging, slots);
        }
        self.tap_voo.ultimo.clone()
    }

    /// Põe a leitura em voo: o `map_async` da leitura já submetida (nunca espera por ele).
    fn arma_o_tap(&mut self, staging: wgpu::Buffer, slots: Vec<Slot>) {
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

    /// ⭐ **A leitura no MESMO `submit` do cozimento** (doc 122 §5) — chamada pelo cozimento antes
    /// de submeter: com um pedido e nada em voo, codifica a leitura dos streams DESTE cozimento no
    /// encoder dele. Devolve o que o [`Self::tap_depois_do_submit`] arma depois do `submit`.
    pub(crate) fn tap_no_cozimento(
        &mut self,
        gpu: &GpuContext,
        encoder: &mut wgpu::CommandEncoder,
    ) -> Option<Option<(wgpu::Buffer, Vec<Slot>)>> {
        self.tap_voo.cozimento_viu_pedido = self.tap_voo.pedido.is_some();
        let samples = self.tap_voo.pedido?;
        if self.tap_voo.em_voo.is_some() {
            return None;
        }
        Some(self.codifica_tap(gpu, encoder, samples, true))
    }

    /// A metade de depois do `submit` — `Some(None)`: havia pedido e nada a amostrar, e a leitura
    /// antiga NÃO fica (a mesma lei do arranque, acima).
    pub(crate) fn tap_depois_do_submit(&mut self, t: Option<Option<(wgpu::Buffer, Vec<Slot>)>>) {
        match t {
            None => {}
            Some(None) => self.tap_voo.ultimo = None,
            Some(Some((staging, slots))) => self.arma_o_tap(staging, slots),
        }
    }

    /// **Esquece o pedido em voo e a última leitura** — para quem deixa de conduzir pela placa.
    pub fn descarta_tap_em_voo(&mut self) {
        if self.tap_voo.em_voo.is_some() {
            self.solta_a_leitura_guardada();
        }
        self.tap_voo = TapVoo::default();
    }

    /// A leitura em voo morreu sem ser lida — ver [`crate::tap::TapPipeline::solta_a_leitura`].
    fn solta_a_leitura_guardada(&mut self) {
        if let Some(t) = &mut self.tap_pipeline {
            t.solta_a_leitura();
        }
    }

    /// **A última chamada de [`Self::tap_sem_espera`] trouxe uma leitura NOVA?** Quando a placa
    /// ainda não acabou, a chamada devolve a leitura anterior — e quem compara leituras entre
    /// quadros (os fios que «marcham» quando o valor MUDA; a sonda que junta uma amostra por
    /// leitura) tem de saber que aquela é a MESMA (auditoria do fecho, 2026-09-24).
    #[must_use]
    pub fn tap_fresca(&self) -> bool {
        self.tap_voo.fresca
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
