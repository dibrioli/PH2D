//! ⭐⭐ **AS LIGAÇÕES DO COZIMENTO, GUARDADAS ENTRE QUADROS** (doc 122 §3 do Motion).
//!
//! Medido pela porta do produto (`motion_ciclo6_relogio_probe`): o custo de CPU de um quadro na
//! placa estava em RECRIAR, a cada quadro, o que não muda — um `BindGroup` por passe, um uniform de
//! 16 bytes por passe auxiliar, os níveis de uma redução — e em o `submit` rastrear esses recursos
//! novos. Um condutor `value.attribute → value.reduce` custava `+0,07 ms` sobre o constante, metade na
//! etapa e metade no `submit`.
//!
//! Duas memórias, ambas por IDADE (em viragens, uma por cozimento):
//!
//! - **uniformes por CONTEÚDO** — um uniform só de leitura cujo conteúdo é o mesmo é o MESMO buffer,
//!   escrito uma vez. Dois passes do mesmo `submit` com o mesmo conteúdo partilham-no (leitura pura).
//! - **ligações por (pipeline, buffers)** — um `BindGroup` só liga buffers inteiros; com os mesmos
//!   buffers nas mesmas posições é o mesmo objecto. O conjunto de buffers alterna entre quadros (o
//!   pool devolve-os por ordem inversa), e a idade guarda as duas fases.
//!
//! ⚠️ Uma entrada segura os buffers dela (o `BindGroup` do wgpu referencia-os): por isso a idade.
//! Um buffer que o pool continua a dar é visto em cada viragem; o que saiu de uso morre ao fim de
//! [`IDADE_MAX`] viragens.

use std::collections::BTreeMap;

use ph2d_gpu::GpuContext;

/// Viragens sem uso ao fim das quais uma entrada morre. Um quadro de simulação vira uma vez por
/// tique cozido, e os buffers alternam com período `2`: `8` cobre os dois com folga.
pub(crate) const IDADE_MAX: u64 = 8;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
struct ChaveGrupo {
    pipeline: wgpu::ComputePipeline,
    entradas: Vec<(u32, wgpu::Buffer, u64, Option<std::num::NonZeroU64>)>,
}

/// Uma entrada de ligação que é uma FATIA de um buffer — `(binding, buffer, offset, size)`.
pub(crate) type Fatia<'a> = (u32, &'a wgpu::Buffer, u64, Option<std::num::NonZeroU64>);

#[derive(Default)]
pub(crate) struct Ligacoes {
    uniformes: BTreeMap<Vec<u8>, (wgpu::Buffer, u64)>,
    grupos: BTreeMap<ChaveGrupo, (wgpu::BindGroup, u64)>,
    viragem: u64,
}

impl Ligacoes {
    /// O uniform com estes bytes — o mesmo buffer enquanto o conteúdo for o mesmo.
    pub(crate) fn uniforme(&mut self, gpu: &GpuContext, bytes: &[u8]) -> wgpu::Buffer {
        let agora = self.viragem;
        if let Some((b, uso)) = self.uniformes.get_mut(bytes) {
            *uso = agora;
            return b.clone();
        }
        let tamanho = (bytes.len() as u64).max(16).next_multiple_of(16);
        let b = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ph2d-gpu-cook uniforme"),
            size: tamanho,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        gpu.queue.write_buffer(&b, 0, bytes);
        self.uniformes.insert(bytes.to_vec(), (b.clone(), agora));
        b
    }

    /// A ligação do grupo `0` de `pipeline` com cada buffer INTEIRO na sua posição.
    pub(crate) fn grupo(
        &mut self,
        gpu: &GpuContext,
        pipeline: &wgpu::ComputePipeline,
        entradas: &[(u32, &wgpu::Buffer)],
    ) -> wgpu::BindGroup {
        let fatias: Vec<Fatia<'_>> = entradas.iter().map(|(i, b)| (*i, *b, 0, None)).collect();
        self.grupo_de_fatias(gpu, pipeline, None, &fatias)
    }

    /// A [`Self::grupo`] com FATIAS de buffers e, quando o pipeline a declara à parte, a planta
    /// explícita (`layout`) — a leitura dos cartões liga uma fatia de parâmetros por coluna.
    pub(crate) fn grupo_de_fatias(
        &mut self,
        gpu: &GpuContext,
        pipeline: &wgpu::ComputePipeline,
        layout: Option<&wgpu::BindGroupLayout>,
        entradas: &[Fatia<'_>],
    ) -> wgpu::BindGroup {
        let agora = self.viragem;
        let chave = ChaveGrupo {
            pipeline: pipeline.clone(),
            entradas: entradas
                .iter()
                .map(|(i, b, o, t)| (*i, (*b).clone(), *o, *t))
                .collect(),
        };
        if let Some((g, uso)) = self.grupos.get_mut(&chave) {
            *uso = agora;
            return g.clone();
        }
        let entries: Vec<wgpu::BindGroupEntry> = entradas
            .iter()
            .map(|(i, b, o, t)| wgpu::BindGroupEntry {
                binding: *i,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: b,
                    offset: *o,
                    size: *t,
                }),
            })
            .collect();
        let implicita;
        let layout = match layout {
            Some(l) => l,
            None => {
                implicita = pipeline.get_bind_group_layout(0);
                &implicita
            }
        };
        let g = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ph2d-gpu-cook ligacao"),
            layout,
            entries: &entries,
        });
        self.grupos.insert(chave, (g.clone(), agora));
        g
    }

    /// Uma viragem (um cozimento): o que ficou [`IDADE_MAX`] viragens sem uso morre.
    pub(crate) fn vira(&mut self) {
        self.viragem += 1;
        let agora = self.viragem;
        self.uniformes
            .retain(|_, (_, uso)| agora - *uso <= IDADE_MAX);
        self.grupos.retain(|_, (_, uso)| agora - *uso <= IDADE_MAX);
    }

    /// Esquece tudo — a variante «a frio» da sonda intercalada (o custo antigo, pelo MESMO código).
    pub(crate) fn esquece(&mut self) {
        self.uniformes.clear();
        self.grupos.clear();
    }

    /// `(uniformes, ligações)` vivos — a régua de que a memória não cresce sem tecto.
    pub(crate) fn tamanho(&self) -> (usize, usize) {
        (self.uniformes.len(), self.grupos.len())
    }
}

/// O buffer de trabalho `STORAGE` de pelo menos `bytes`, por classe de potência de dois — PERSISTENTE.
///
/// ⚠️ Partilhado entre usos SEQUENCIAIS no mesmo `submit` (os níveis de duas reduções): os passes de
/// um encoder correm por ordem e o wgpu põe as barreiras, logo reutilizar um rascunho depois de o
/// passe anterior o ter consumido é seguro. Dois usos VIVOS ao mesmo tempo pedem classes diferentes,
/// e é o chamador que as separa (`nivel`).
#[derive(Default)]
pub(crate) struct Rascunhos {
    por_classe: BTreeMap<(u32, u64), wgpu::Buffer>,
}

impl Rascunhos {
    pub(crate) fn de(&mut self, gpu: &GpuContext, nivel: u32, bytes: u64) -> wgpu::Buffer {
        let classe = bytes.max(16).next_power_of_two();
        self.por_classe
            .entry((nivel, classe))
            .or_insert_with(|| {
                gpu.device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("ph2d-gpu-cook rascunho"),
                    size: classe,
                    usage: wgpu::BufferUsages::STORAGE,
                    mapped_at_creation: false,
                })
            })
            .clone()
    }

    pub(crate) fn esquece(&mut self) {
        self.por_classe.clear();
    }
}

impl crate::GpuCook {
    /// Uma viragem de TODAS as memórias de ligações do cozimento (uma por `cook`).
    pub(crate) fn vira_as_ligacoes(&mut self) {
        self.ligacoes.vira();
        if let Some(r) = &self.reduce {
            r.vira();
        }
        if let Some(p) = &self.stream_op_pipes {
            p.vira();
        }
        if let Some(g) = &self.grid {
            g.vira();
        }
        self.passe.vira();
    }

    /// **Esquece todas as ligações guardadas** — o próximo cozimento recria-as como antes do doc
    /// 122 §3. Só a sonda intercalada o chama (a variante «a frio», pelo MESMO código).
    #[doc(hidden)]
    pub fn esquece_as_ligacoes(&mut self) {
        self.ligacoes.esquece();
        if let Some(r) = &self.reduce {
            r.esquece();
        }
        if let Some(p) = &self.stream_op_pipes {
            p.esquece();
        }
        if let Some(g) = &self.grid {
            g.esquece();
        }
        // ⚠️ A leitura em voo segura a SUA `staging` — esquecer a guardada não a desmapeia.
        if let Some(t) = &mut self.tap_pipeline {
            t.esquece();
        }
    }

    /// `(uniformes, ligações)` vivos na memória das etapas — a régua de que ela não cresce.
    #[doc(hidden)]
    #[must_use]
    pub fn ligacoes_vivas(&self) -> (usize, usize) {
        self.ligacoes.tamanho()
    }
}
