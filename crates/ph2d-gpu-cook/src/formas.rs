//! ⭐⭐⭐ **AS CÓPIAS DE FORMA do cozimento** (doc 121 do Motion, W3) — o buffer que o passe de
//! formas (`ph2d_shape_gpu::ShapePass::draw`, `Copias { buffer, count }`) lê sem o trazer de volta.
//!
//! Irmão do [`crate::instances`] pela mesma razão: o sequenciador pergunta *«o que este nó
//! computa?»*, isto pergunta *«o que o passe de formas precisa?»*.
//!
//! ⚠️ **A contagem é do HOST**, como a das sprites: cada saída que carrega a coluna `geometry_id`
//! escreve `count` cópias (uma por linha; as que não são forma levam a geometria que o passe não
//! acha, ver [`crate::lower_forma::SEM_GEOMETRIA`]). Nada de desenho indirecto, nada de compactação.
//!
//! ⛔ **Uma saída com forma e com a coluna `blend` RECUSA o quadro** ([`GpuCookError::FormaComMistura`]):
//! a mistura POR LINHA pede uma camada fora do alvo, que só a cena vectorial sabe (a mesma recusa da
//! rota da CPU, `motion_shape_gen::mistura::precisa_do_vello`). ⚠️ A pergunta aqui é a PRESENÇA
//! da coluna e na CPU é o VALOR — ler o valor custaria uma descarga por quadro, e a divergência
//! cai para o lado conservador (uma coluna toda a zero manda o quadro à CPU, que o desenha certo).

use crate::stream::GpuStream;
use crate::{CachedPipeline, GpuCook, GpuCookError, codegen, create_pipeline, lower_forma};
use ph2d_gpu::GpuContext;

/// O buffer de cópias de forma do último cozimento: `[ShapeInstance; len]`, com uso `STORAGE`
/// (o passe liga-o inteiro) e `COPY_SRC` (os gates de paridade lêem-no de volta, fora do caminho).
pub struct GpuFormas {
    pub(crate) buffer: wgpu::Buffer,
    pub(crate) len: u32,
    pub(crate) capacity: u32,
}

impl GpuFormas {
    /// O buffer — o `Copias::buffer` do passe de formas.
    #[must_use]
    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }
    /// Quantas cópias o último cozimento escreveu — o `Copias::count`.
    #[must_use]
    pub fn len(&self) -> u32 {
        self.len
    }
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

/// **Esta corrente leva formas?** — a coluna `geometry_id` escalar. Uma porta só, lida pela
/// reserva e pela escrita (duas redacções dela discordariam numa coluna do tipo errado).
pub(crate) fn leva_formas(stream: &GpuStream) -> bool {
    stream
        .cols
        .get("geometry_id")
        .is_some_and(|c| c.dim == ph2d_nodegraph::port::Dim::Scalar)
}

/// **As cópias de forma cabem na ligação de armazenamento?** — o mesmo tecto que o
/// [`GpuCookError::BindingTooLarge`] mede para as sprites, a `64 B` por cópia.
pub(crate) fn cabe(gpu: &GpuContext, total: u64) -> Result<u32, GpuCookError> {
    let bytes = total * u64::from(lower_forma::FORMA_WORDS) * 4;
    let limit = gpu.device.limits().max_storage_buffer_binding_size;
    if bytes > limit {
        return Err(GpuCookError::BindingTooLarge { bytes, limit });
    }
    u32::try_from(total).map_err(|_| GpuCookError::BindingTooLarge { bytes, limit })
}

impl GpuCook {
    /// As cópias de forma do último cozimento — `None` quando nenhuma saída levou formas (ou antes
    /// do primeiro). ⚠️ Um quadro sem formas deixa-as a `None`, e não com `len = 0` sobre um buffer
    /// velho: quem desenha pergunta **uma** coisa.
    #[must_use]
    pub fn formas(&self) -> Option<&GpuFormas> {
        self.formas.as_ref().filter(|f| f.len > 0)
    }

    /// Cresce (nunca encolhe) o buffer para `count` cópias e zera a contagem. ⚠️ Chamado UMA vez,
    /// ANTES da primeira escrita, com o total — crescer substitui o buffer e perde o que lá estava.
    pub(crate) fn reservar_formas(&mut self, gpu: &GpuContext, count: u32) {
        let cabe = self.formas.as_ref().is_some_and(|f| f.capacity >= count);
        if !cabe {
            let mut capacity = self.formas.as_ref().map_or(1, |f| f.capacity.max(1));
            while capacity < count {
                capacity = capacity.saturating_mul(2);
            }
            let buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("ph2d-gpu-cook formas"),
                size: u64::from(capacity) * u64::from(lower_forma::FORMA_WORDS) * 4,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
                mapped_at_creation: false,
            });
            self.formas = Some(GpuFormas {
                buffer,
                len: 0,
                capacity,
            });
        }
        if let Some(f) = self.formas.as_mut() {
            f.len = 0;
        }
    }

    /// Escreve as `stream.count` cópias desta corrente a partir da cópia `primeiro`. O buffer já
    /// foi reservado para o total ([`Self::reservar_formas`]).
    pub(crate) fn encode_formas(
        &mut self,
        gpu: &GpuContext,
        encoder: &mut wgpu::CommandEncoder,
        uniform_slot: usize,
        stream: &GpuStream,
        pivot: [f32; 2],
    ) {
        let count = stream.count;
        let Some(formas) = self.formas.as_mut() else {
            return;
        };
        let primeiro = formas.len;
        formas.len = primeiro.saturating_add(count).min(formas.capacity);
        if count == 0 {
            return;
        }
        let present: [bool; 5] = std::array::from_fn(|i| {
            let dim = match i {
                0 | 1 => ph2d_nodegraph::port::Dim::Vec2,
                3 => ph2d_nodegraph::port::Dim::Vec4,
                _ => ph2d_nodegraph::port::Dim::Scalar,
            };
            stream
                .cols
                .get(lower_forma::FORMA_COLUMNS[i])
                .is_some_and(|c| c.dim == dim)
        });
        let sig = lower_forma::forma_signature(present, pivot);
        self.forma_pipelines.entry(sig).or_insert_with(|| {
            let src = lower_forma::forma_module(present, pivot);
            CachedPipeline {
                pipeline: create_pipeline(gpu, &src, "ph2d-gpu-cook formas"),
            }
        });
        // Uniform: count, primeiro (o resto do slot fica por usar).
        let mut uni = [0u8; 8];
        uni[0..4].copy_from_slice(&count.to_le_bytes());
        uni[4..8].copy_from_slice(&primeiro.to_le_bytes());
        let uniform = self.uniform_slot(gpu, uniform_slot);
        gpu.queue.write_buffer(uniform, 0, &uni);
        let uniform = &self.uniforms[uniform_slot];
        let formas = self.formas.as_ref().expect("reservado acima");
        let mut entries = vec![
            wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: formas.buffer.as_entire_binding(),
            },
        ];
        let mut slot = 2u32;
        for (i, name) in lower_forma::FORMA_COLUMNS.iter().enumerate() {
            if present[i] {
                let col = stream.cols.get(*name).expect("presença verificada");
                entries.push(wgpu::BindGroupEntry {
                    binding: slot,
                    resource: col.buffer.as_entire_binding(),
                });
                slot += 1;
            }
        }
        let pipeline = &self
            .forma_pipelines
            .get(&sig)
            .expect("inserido acima")
            .pipeline;
        let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ph2d-gpu-cook formas"),
            layout: &pipeline.get_bind_group_layout(0),
            entries: &entries,
        });
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("ph2d-gpu-cook formas"),
            timestamp_writes: None,
        });
        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.dispatch_workgroups(count.div_ceil(codegen::WORKGROUP_SIZE), 1, 1);
    }
}
