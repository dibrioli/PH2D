//! The **lowering**: the last compute pass of a cook, which gathers the stream's
//! columns straight into a buffer laid out as [`ph2d_render::RenderInstance`] —
//! the sprite renderer's instance vertex buffer.
//!
//! This is where the GPU cook stops being a stream engine and becomes a frame:
//! it is the only stage that is not a node, the only one that applies the
//! caller's `default_uv_rect`/`default_size`, and the only one that writes a
//! PERSISTENT (grow-only) buffer rather than a pooled column. Kept beside the
//! sequencer rather than in it because it answers a different question — the
//! sequencer asks "what does this node compute?", this asks "what does the
//! renderer need?".

use crate::stream::GpuStream;
use crate::{CachedPipeline, GpuCook, UNIFORM_BYTES, codegen, create_pipeline, lower};
use ph2d_gpu::GpuContext;

/// ⭐⭐⭐ **O VEREDITO DA LEI DO DONO, DO LADO DO DISPOSITIVO — e ele é PURO de propósito.**
///
/// A [`GpuCook::encode_lowering`] pede um `GpuContext` e um `CommandEncoder`, logo um gate sobre
/// ela nasceria `#[ignore]` e **o CI nunca o correria**. ⛔⛔ E esta é a rota de OMISSÃO deste
/// módulo (o cozimento é GPU-resident por default): até 2026-09-19 a lei **não existia aqui**, o
/// report do dono — *«o grid continua desenhando quadrados»* — reproduzia-se com a rota da CPU
/// inteiramente gateada, e **duas mutações sobreviveram** a apagá-la porque nenhum teste desta
/// crate a nomeava. *Quando um gate precisa de um device para medir uma decisão que não tem pixel
/// nenhum, a lei está no sítio errado.*
///
/// Devolve `(desenha, estilo)`:
/// - `desenha = false` ⇒ a corrente não produz instância nenhuma (o despacho nem corre).
/// - o `estilo` devolvido é o que o `lower_signature`/`lower_module` assam, e pode ter a lei
///   **desligada** pelo braço da geometria viva.
///
/// ⚠️ **A pergunta aqui é a metade do LADRILHO** e na CPU é a corrente inteira
/// (`uv_rect` **ou** `geometry_id > 0`): a outra metade é por VALOR, e lê-la aqui custaria uma
/// descarga do buffer por quadro. A divergência é **NOMEADA** e cai para o lado conservador —
/// assim que a coluna da geometria existe, a lei desliga-se e desenha-se como sempre.
#[must_use]
pub fn veredito_do_dispositivo(
    style: ph2d_render::SinkStyle,
    tem_geometria: bool,
    tem_ladrilho: bool,
) -> (bool, ph2d_render::SinkStyle) {
    let style = if style.so_com_forma && tem_geometria {
        ph2d_render::SinkStyle {
            so_com_forma: false,
            ..style
        }
    } else {
        style
    };
    (!(style.so_com_forma && !tem_ladrilho), style)
}

/// The GPU-resident instance output of a cook: a buffer laid out as
/// `[RenderInstance; len]`, usable directly as the sprite renderer's instance
/// vertex buffer (usage VERTEX) and mappable for the parity gates (COPY_SRC).
pub struct GpuInstances {
    pub(crate) buffer: wgpu::Buffer,
    pub(crate) len: u32,
    pub(crate) capacity: u32,
}

impl GpuInstances {
    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }
    pub fn len(&self) -> u32 {
        self.len
    }
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

impl GpuCook {
    /// Encode the final lowering pass into the persistent instance buffer.
    // The lowering seam: device + encoder + slot + stream + the three host-side
    // lowering decisions (uv rect, size, style). Bundling them into a struct would
    // buy a name and cost a second place to keep in step with `cook`'s signature.
    /// ⭐⭐⭐ **`primeiro` é o deslocamento em INSTÂNCIAS**: esta corrente escreve em
    /// `[primeiro, primeiro + count)` do buffer partilhado. `0` é o caminho de sempre, byte a
    /// byte.
    ///
    /// ⛔⛔⛔ **CRESCER O BUFFER SUBSTITUI-O E PERDE O CONTEÚDO** — ver
    /// [`Self::ensure_instance_capacity`], que cria um `wgpu::Buffer` NOVO. Num acrescento isso
    /// apagaria, em silêncio, tudo o que os sinks anteriores já escreveram: *o modo de falha seria
    /// uma cena a desenhar só o último sink, sem erro nenhum.*
    ///
    /// ⇒ com `primeiro > 0` este método **NUNCA cresce**: ele devolve `false` e quem chama recua.
    /// A reserva do TOTAL é do chamador, feita **uma vez, antes da primeira escrita** — que é a
    /// única ordem em que a soma é conhecida e nada foi ainda escrito.
    ///
    /// ⚠️ **Hoje o único chamador passa `0`**, logo este método é INERTE: o `primeiro` existe para
    /// que a caminhada de N sinks (a wave da cerca do multi-sink) não tenha de reabrir o lowering,
    /// e a inércia dele é o que os gates de paridade de GPU desta crate afirmam.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn encode_lowering(
        &mut self,
        gpu: &GpuContext,
        encoder: &mut wgpu::CommandEncoder,
        uniform_slot: usize,
        stream: &GpuStream,
        primeiro: u32,
        default_uv_rect: [f32; 4],
        default_size: [f32; 2],
        style: ph2d_render::SinkStyle,
    ) -> bool {
        let count = stream.count;
        let fim = primeiro.saturating_add(count);
        if primeiro == 0 {
            self.ensure_instance_capacity(gpu, fim.max(1));
        } else if self.instances.as_ref().is_none_or(|gi| gi.capacity < fim) {
            // O chamador não reservou o total. Recusar é a única saída honesta: crescer aqui
            // apagaria os sinks já escritos, e escrever fora da capacidade é validação do wgpu.
            return false;
        }
        let instances = self
            .instances
            .as_mut()
            .expect("reservado acima ou pelo chamador");
        instances.len = fim;
        if count == 0 {
            return true;
        }

        let present: [bool; 8] = std::array::from_fn(|i| {
            stream
                .cols
                .get(lower::LOWER_COLUMNS[i])
                .is_some_and(|c| c.dim == expected_lower_dim(i))
        });
        // ⭐⭐⭐ **A OUTRA METADE DA LEI DO DONO, e ela é do HOST** — ver o `marca` do
        // [`lower::lower_module`]. A rota da CPU pergunta `uv_rect` **ou** `geometry_id > 0`; o
        // shader só consegue a primeira, porque a segunda é por VALOR e lê-la aqui custaria uma
        // descarga do buffer por quadro.
        //
        // ⚠️ **A divergência é NOMEADA e cai para o lado conservador:** assim que a coluna da
        // geometria existe, este caminho desliga a lei e desenha como sempre desenhou. Uma
        // corrente com a coluna toda a ZERO (uma `source.shape` sem forma escolhida) recebe
        // marcas na CPU e quads aqui — e chegar aqui exige que o planeador a tenha deixado ir à
        // placa, que hoje não acontece com nenhuma fonte de geometria.
        let (desenha, style) =
            veredito_do_dispositivo(style, stream.cols.contains_key("geometry_id"), present[4]);
        // ⭐⭐⭐ **A LEI DO DONO, DO LADO DO DISPOSITIVO** — e até 2026-09-19 ela **não existia
        // aqui**: o `lower_module` nunca leu o `so_com_forma` e o `lower_signature` nem sequer o
        // comia, logo uma cena que fosse à placa (o caminho de OMISSÃO deste módulo) desenhava os
        // quads de sempre com a lei ligada. *Uma lei escrita só na CPU é uma lei que o produto
        // não tem.*
        //
        // ⚠️ **A pergunta aqui é a metade do LADRILHO** (`present[4]`, a coluna `uv_rect`) e na
        // CPU é a corrente inteira (`uv_rect` **ou** `geometry_id > 0`). A outra metade é por
        // VALOR, e lê-la aqui custaria uma descarga do buffer por quadro — quem a cobre é o
        // braço da geometria logo acima, que **desliga a lei** e desenha como sempre desenhou
        // assim que a coluna existe. ⚠️ A divergência cai para o lado conservador, e é NOMEADA.
        //
        // ⛔ E o despacho não corre: a corrente não produz instância nenhuma.
        if !desenha {
            // ⚠️ **`primeiro` e NAO zero**: a lei do dono cala ESTA corrente, e zerar o `len`
            // apagaria do desenho os sinks que ja escreveram antes dela. Com `primeiro == 0` isto
            // e' `0`, que e' o caminho de sempre.
            instances.len = primeiro;
            return true;
        }
        let sig = lower::lower_signature(present, style);
        self.lower_pipelines.entry(sig).or_insert_with(|| {
            let src = lower::lower_module(present, style);
            CachedPipeline {
                pipeline: create_pipeline(gpu, &src, "ph2d-gpu-cook lowering"),
            }
        });

        // Uniform: count, primeiro, default_size (vec2 @ 8), default_uv (vec4 @ 16).
        let mut uni = [0u8; 32];
        uni[0..4].copy_from_slice(&count.to_le_bytes());
        uni[4..8].copy_from_slice(&primeiro.to_le_bytes());
        uni[8..12].copy_from_slice(&default_size[0].to_le_bytes());
        uni[12..16].copy_from_slice(&default_size[1].to_le_bytes());
        for (k, v) in default_uv_rect.iter().enumerate() {
            uni[16 + k * 4..20 + k * 4].copy_from_slice(&v.to_le_bytes());
        }
        let uniform = self.uniform_slot(gpu, uniform_slot);
        gpu.queue.write_buffer(uniform, 0, &uni);
        let uniform = &self.uniforms[uniform_slot];

        let instances = self.instances.as_ref().expect("ensured above");
        let mut entries = vec![
            wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: instances.buffer.as_entire_binding(),
            },
        ];
        let mut slot = 2u32;
        for (i, name) in lower::LOWER_COLUMNS.iter().enumerate() {
            if present[i] {
                let col = stream.cols.get(*name).expect("presence checked");
                entries.push(wgpu::BindGroupEntry {
                    binding: slot,
                    resource: col.buffer.as_entire_binding(),
                });
                slot += 1;
            }
        }
        let pipeline = &self
            .lower_pipelines
            .get(&sig)
            .expect("inserted above")
            .pipeline;
        let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ph2d-gpu-cook lowering"),
            layout: &pipeline.get_bind_group_layout(0),
            entries: &entries,
        });
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("ph2d-gpu-cook lowering"),
            timestamp_writes: None,
        });
        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.dispatch_workgroups(count.div_ceil(codegen::WORKGROUP_SIZE), 1, 1);
        true
    }

    /// The persistent uniform buffer for stage slot `idx` (created on demand).
    pub(crate) fn uniform_slot(&mut self, gpu: &GpuContext, idx: usize) -> &wgpu::Buffer {
        while self.uniforms.len() <= idx {
            self.uniforms
                .push(gpu.device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("ph2d-gpu-cook uniforms"),
                    size: UNIFORM_BYTES,
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                }));
        }
        &self.uniforms[idx]
    }

    /// Grow (never shrink) the instance output to hold `count` instances —
    /// `InstanceBuffer`'s policy, plus STORAGE (the lowering writes it) and
    /// COPY_SRC (the parity gates read it back, deliberately off-path).
    pub(crate) fn ensure_instance_capacity(&mut self, gpu: &GpuContext, count: u32) {
        let needs_grow = match &self.instances {
            Some(gi) => gi.capacity < count,
            None => true,
        };
        if !needs_grow {
            return;
        }
        let mut capacity = self
            .instances
            .as_ref()
            .map(|gi| gi.capacity.max(1))
            .unwrap_or(1);
        while capacity < count {
            capacity = capacity.saturating_mul(2);
        }
        let buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ph2d-gpu-cook instances"),
            size: u64::from(capacity) * u64::from(lower::INSTANCE_WORDS) * 4,
            usage: wgpu::BufferUsages::VERTEX
                | wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        self.instances = Some(GpuInstances {
            buffer,
            len: 0,
            capacity,
        });
    }
}

/// The dim each lowering column must have to be gathered (a column with the
/// wrong type is ignored, exactly like the CPU's typed `*_at` accessors).
fn expected_lower_dim(i: usize) -> ph2d_nodegraph::port::Dim {
    use ph2d_nodegraph::port::Dim;
    match i {
        0 | 1 => Dim::Vec2,       // P, size
        2 | 5 | 6 => Dim::Scalar, // rot, texture_id, blend
        _ => Dim::Vec4,           // tint, uv_rect, uv_cell
    }
}
