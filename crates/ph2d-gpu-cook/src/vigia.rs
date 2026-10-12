//! ⭐⭐ **A VIGIA DAS IMAGENS** (B6, família B do Motion) — quantas linhas de IMAGEM caíram numa
//! saída que mistura EM GRUPO, sem esperar pela placa.
//!
//! Uma saída com mistura em grupo (`Add`/`Multiply`/`Screen` entre as cópias) só vai ao dispositivo
//! se as linhas dela forem todas FORMAS: as formas misturam por cópia na placa de formas, e uma
//! imagem iria ao passe de sprites, que não sabe o alcance e mistura em luz linear. O `geometry_id`
//! de cada linha é um VALOR que só a placa vê ⇒ o chamador decide com a memória da CPU antes de
//! cozer, e a vigia confirma depois:
//! - uma saída vigiada SEM a coluna `geometry_id` é toda imagens — sabe-se no host, e o cozimento
//!   recusa já ([`crate::GpuCookError::ImagemEmGrupo`]);
//! - com a coluna, o baixamento das formas soma as linhas que não são forma num contador
//!   ([`crate::lower_forma::forma_module`], `conta`); a cópia dele vai no MESMO `submit` e é lida
//!   num quadro seguinte ([`GpuCook::toma_imagens_vigiadas`], `poll(Poll)` e nunca `wait`).
//!
//! ⚠️ Um pedido em voo de cada vez (a lei do [`crate::tap_voo`]). O preço da espera é a janela:
//! um a dois quadros com a imagem no passe de sprites, quando o artista a liga a uma saída que
//! até ali só tinha formas — depois a rota volta à CPU e diz porquê.

use crate::GpuCook;
use ph2d_gpu::GpuContext;
use std::sync::mpsc::{Receiver, TryRecvError};

/// O estado da vigia: as saídas vigiadas, o contador na placa e a leitura em voo.
#[derive(Default)]
pub(crate) struct Vigia {
    /// Uma por [`crate::GpuPlan::sinks`], na mesma ordem. Vazio = nenhuma (o caminho de sempre).
    saidas: Vec<bool>,
    contador: Option<wgpu::Buffer>,
    em_voo: Option<(wgpu::Buffer, Receiver<Result<(), wgpu::BufferAsyncError>>)>,
    /// A última leitura completa, até ser tomada.
    lida: Option<u32>,
}

impl Vigia {
    pub(crate) fn vigia(&self, k: usize) -> bool {
        self.saidas.get(k).copied().unwrap_or(false)
    }

    fn ativa(&self) -> bool {
        self.saidas.iter().any(|&v| v)
    }

    /// O contador existe sempre que um baixamento de formas o liga (a ligação é fixa no módulo).
    pub(crate) fn garante_o_contador(&mut self, gpu: &GpuContext) {
        if self.contador.is_none() {
            self.contador = Some(gpu.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("ph2d-gpu-cook vigia das imagens"),
                size: 4,
                usage: wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_SRC
                    | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }));
        }
    }

    pub(crate) fn contador(&self) -> Option<&wgpu::Buffer> {
        self.contador.as_ref()
    }
}

impl GpuCook {
    /// **As saídas a vigiar**, uma por [`crate::GpuPlan::sinks`] — `true` = mistura em grupo, e
    /// uma linha que não seja forma tem de se saber. Vazio (o default) = nenhuma.
    pub fn set_vigia_de_imagens(&mut self, saidas: Vec<bool>) {
        self.vigia.saidas = saidas;
    }

    /// Zera o contador antes dos baixamentos deste cozimento (só com uma saída vigiada).
    pub(crate) fn vigia_no_inicio(&mut self, gpu: &GpuContext, encoder: &mut wgpu::CommandEncoder) {
        if !self.vigia.ativa() {
            return;
        }
        self.vigia.garante_o_contador(gpu);
        if let Some(c) = &self.vigia.contador {
            encoder.clear_buffer(c, 0, None);
        }
    }

    /// A cópia do contador no encoder do cozimento, se há saída vigiada e nada em voo.
    pub(crate) fn vigia_no_cozimento(
        &mut self,
        gpu: &GpuContext,
        encoder: &mut wgpu::CommandEncoder,
    ) -> Option<wgpu::Buffer> {
        if !self.vigia.ativa() || self.vigia.em_voo.is_some() {
            return None;
        }
        let contador = self.vigia.contador.as_ref()?;
        let staging = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ph2d-gpu-cook vigia (leitura)"),
            size: 4,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        encoder.copy_buffer_to_buffer(contador, 0, &staging, 0, 4);
        Some(staging)
    }

    /// Depois do `submit`: põe a leitura em voo (nunca espera por ela).
    pub(crate) fn vigia_depois_do_submit(&mut self, staging: Option<wgpu::Buffer>) {
        let Some(staging) = staging else {
            return;
        };
        let (tx, rx) = std::sync::mpsc::channel();
        staging.slice(..).map_async(wgpu::MapMode::Read, move |r| {
            let _ = tx.send(r);
        });
        self.vigia.em_voo = Some((staging, rx));
    }

    /// ⭐ **Quantas linhas de imagem a placa contou** nas saídas vigiadas, na última leitura
    /// completa — e esquece-a (cada leitura decide UMA vez). `0` enquanto nada voltou.
    pub fn toma_imagens_vigiadas(&mut self, gpu: &GpuContext) -> u32 {
        // ⚠️ Sem leitura em voo não há `poll`: ele acordaria as leituras dos OUTROS (os cartões, o
        // relatório do passe) num ponto do quadro onde elas nunca acordavam.
        if self.vigia.em_voo.is_none() {
            return self.vigia.lida.take().unwrap_or(0);
        }
        // ⛔ `Poll` e NUNCA `wait` — ver o cabeçalho.
        let _ = gpu.device.poll(wgpu::PollType::Poll);
        if let Some((staging, pronto)) = &self.vigia.em_voo {
            match pronto.try_recv() {
                Ok(Ok(())) => {
                    let n = {
                        let bytes = staging.slice(..).get_mapped_range();
                        u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
                    };
                    self.vigia.lida = Some(n);
                    self.vigia.em_voo = None;
                }
                Ok(Err(_)) | Err(TryRecvError::Disconnected) => self.vigia.em_voo = None,
                Err(TryRecvError::Empty) => {}
            }
        }
        self.vigia.lida.take().unwrap_or(0)
    }

    /// **Esquece a leitura em voo e a última lida** — para quem deixa de cozer na placa: uma
    /// leitura de há muitos quadros não pode decidir o regresso.
    pub fn descarta_a_vigia(&mut self) {
        self.vigia.em_voo = None;
        self.vigia.lida = None;
    }
}
