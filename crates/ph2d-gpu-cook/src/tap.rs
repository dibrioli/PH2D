//! **The bounded tap** — a fixed, small subsample of a GPU-resident frame,
//! cheap enough to take every frame.
//!
//! ## Why this exists, and why it is not `debug_read`
//!
//! A GPU cook does not feed the CPU memo, so the graph panel's readouts, change
//! digest, postage stamps and probe all go blank exactly when the numbers get
//! interesting — which is what kept `PH2D_GPU_COOK` opt-in. The obvious answer,
//! "read the results back", was ruled out by measurement: `readback_tap_cost_probe`
//! reports **297 ms** to pull 4,19 M instances, worse than cooking the whole
//! thing on the CPU.
//!
//! That measurement is right and its conclusion was over-applied. It is a
//! statement about a SIZE. The panel does not want the buffer — it wants **48
//! samples**, because the CPU path it mirrors already subsamples to 48
//! (`DIGEST_SAMPLES`, `PREVIEW_POINTS`). `bounded_readback_cost_probe` measures
//! the difference: 48 elements costs **0,022-0,023 ms flat at every window size**,
//! and **+0,075 ms taken in flight** on top of a 6,989 ms cook of 4,19 M — 1% of
//! it, 0,5% of a 60 fps frame. The cost is bandwidth, not the map+poll stall
//! ([[feedback_the_ceiling_is_the_hardwares_never_the_fallbacks]]).
//!
//! So: `debug_read` pulls WHOLE columns and needs `retain_streams_for_debug`,
//! which pins buffers past [`crate::BufferPool::reclaim`] — gates only, forever.
//! This module pulls a bounded sample, pins nothing, and belongs on the frame
//! path.
//!
//! ## Strided, never a prefix
//!
//! The samples are taken by **stride**, not off the front. The panel's stamp
//! shows the SHAPE of what a node emits, and the first 48 points of a 5 000-point
//! spiral are an arc — the CPU-side docstring says exactly this and its sampler
//! strides for exactly this reason. A prefix copy would be one
//! `copy_buffer_to_buffer` and no shader; it would also be the wrong picture, so
//! this is a gather pass.
//!
//! ## One submit, one map
//!
//! Every requested column is gathered by one dispatch into a **single** shared
//! output buffer, all in one encoder — so the tap costs one submit and one map
//! however many nodes the panel is looking at. Tapping per node would be N device
//! syncs, and the sync is the part that does not amortise.
//!
//! ## What the tap does NOT answer
//!
//! **How many elements a node carried.** That comes from [`crate::shape::CookShape`],
//! exactly, because it is what the host SIZED the dispatch with. Counting the
//! tapped samples instead would report `48 inst` for a stream of four million —
//! a subsample is not a census, and the one number here that must be exact is the
//! one the tap cannot give.

use crate::{GpuCook, GpuStream};
use ph2d_gpu::GpuContext;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::graph::NodeId;
use ph2d_nodegraph::port::Dim;
use std::collections::BTreeMap;

/// How many elements a tap takes from each column.
///
/// It mirrors the CPU readout path's `DIGEST_SAMPLES` / `PREVIEW_POINTS` (both
/// 48) — the tap exists to feed those, and a different number here would mean the
/// two paths subsample differently and their digests would disagree on a document
/// that had not changed.
pub const TAP_SAMPLES: u32 = 48;

/// The gather kernel: `out[i] = src[i * stride_elems + first]`, clamped.
///
/// `lanes` is the column's element width in f32s (from
/// [`crate::stream::element_stride`] — never re-derived here, or a `Vec3`'s
/// 16-byte padding would be re-guessed as 12 and every sample past the first
/// would misindex).
const TAP_WGSL: &str = r#"
struct TapParams {
    src_count: u32,
    lanes: u32,
    samples: u32,
    dst_offset: u32,
};
@group(0) @binding(0) var<uniform> p: TapParams;
@group(0) @binding(1) var<storage, read> src: array<f32>;
@group(0) @binding(2) var<storage, read_write> dst: array<f32>;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    if (i >= p.samples) { return; }
    // Stride so the SHAPE survives: sample i of n maps onto element
    // `i * count / samples`, which walks the whole stream instead of its front.
    var e = i;
    if (p.samples > 0u) {
        e = (i * p.src_count) / p.samples;
    }
    if (e >= p.src_count) { e = p.src_count - 1u; }
    for (var l = 0u; l < p.lanes; l = l + 1u) {
        dst[p.dst_offset + i * p.lanes + l] = src[e * p.lanes + l];
    }
}
"#;

/// One column's slot in the packed output buffer.
pub(crate) struct Slot {
    node: NodeId,
    column: String,
    dim: Dim,
    /// Elements actually sampled — `min(TAP_SAMPLES, count)`, so a stream of 3
    /// yields 3 and not 48 copies of the same row.
    samples: u32,
    /// Offset into the shared `f32` output buffer.
    offset: u32,
    /// Uma TOMADA pedida por nome ([`Pedido::tomadas`]) e não a amostra dos cartões.
    tomada: bool,
}

/// **O que uma leitura encomenda**: a amostra dos cartões (todo nó encenado, `cartoes` linhas) e
/// as TOMADAS (os nós que o host pediu por nome, cada um com o seu tecto de linhas) — as dos
/// gizmos de um quadro feito inteiro na placa (Bug #20), num gather só.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Pedido {
    pub(crate) cartoes: Option<u32>,
    pub(crate) tomadas: Vec<(NodeId, u32)>,
}

impl Pedido {
    pub(crate) fn vazio(&self) -> bool {
        self.cartoes.is_none() && self.tomadas.is_empty()
    }
}

/// Uma leitura recolhida: os cartões e as tomadas, cada uma no seu mapa (um nó pode estar nos
/// dois, com contagens diferentes).
#[derive(Clone, Debug, Default)]
pub(crate) struct Leitura {
    pub(crate) cartoes: BTreeMap<NodeId, Stream>,
    pub(crate) tomadas: BTreeMap<NodeId, Stream>,
}

/// The tap's persistent GPU resources — created once, reused every frame.
pub(crate) struct TapPipeline {
    pipeline: wgpu::ComputePipeline,
    layout: wgpu::BindGroupLayout,
    /// ⭐ **Os parâmetros de TODAS as colunas num buffer só, persistente e só a crescer** (ciclo 12,
    /// doc 120 §8.6). Um buffer `mapped_at_creation` por coluna e por quadro custava `0,34 ms` na
    /// RTX a encomendar a leitura da escada — mais do que a leitura inteira. Um `write_buffer` por
    /// quadro, e cada coluna lê a sua fatia alinhada.
    params: Option<wgpu::Buffer>,
    /// ⭐ O destino, a leitura (`MAP_READ`) e as ligações por coluna, GUARDADOS entre quadros
    /// (doc 122 §5): recriados só quando o tamanho muda. A leitura persistente é só a da leitura
    /// SEM ESPERA — uma em voo de cada vez, e desmapeada antes do pedido seguinte.
    dst: Option<wgpu::Buffer>,
    staging: Option<wgpu::Buffer>,
    ligacoes: crate::ligacoes::Ligacoes,
}

/// Um buffer persistente do tamanho EXACTO `bytes` — o mesmo enquanto o tamanho não mudar.
fn do_tamanho(
    gpu: &GpuContext,
    slot: &mut Option<wgpu::Buffer>,
    bytes: u64,
    usage: wgpu::BufferUsages,
    label: &'static str,
) -> wgpu::Buffer {
    if slot.as_ref().is_none_or(|b| b.size() != bytes) {
        *slot = Some(gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size: bytes,
            usage,
            mapped_at_creation: false,
        }));
    }
    slot.clone().expect("acabou de ser garantido")
}

impl TapPipeline {
    /// ⛔ **Larga a leitura `MAP_READ` guardada** — chamada quando uma leitura em voo morre SEM ser
    /// lida (descartada, ou o mapeamento recusado): ela continua mapeada ou por mapear, e o pedido
    /// seguinte que a reusasse pediria um segundo `map_async` sobre ela (`Buffer is already mapped`,
    /// o pânico do wgpu que o gate `um_quadro_da_cpu_descarta_a_leitura_da_placa` apanhou).
    pub(crate) fn solta_a_leitura(&mut self) {
        self.staging = None;
    }

    /// Esquece o destino, a leitura e as ligações guardados — a variante «a frio» da sonda.
    pub(crate) fn esquece(&mut self) {
        self.dst = None;
        self.staging = None;
        self.ligacoes.esquece();
    }

    /// Built through the crate's own [`crate::create_pipeline`] with an AUTO
    /// layout, like every other pipeline here — a hand-written
    /// `BindGroupLayout` would be a second description of a binding set the
    /// shader already declares, and the two would drift.
    fn new(gpu: &GpuContext) -> Self {
        let pipeline = crate::create_pipeline(gpu, TAP_WGSL, "ph2d-gpu-cook tap");
        let layout = pipeline.get_bind_group_layout(0);
        Self {
            pipeline,
            layout,
            params: None,
            dst: None,
            staging: None,
            ligacoes: crate::ligacoes::Ligacoes::default(),
        }
    }
}

impl GpuCook {
    /// **Sample every staged node's output** into small CPU-side streams — the
    /// frame-path tap.
    ///
    /// Returns one [`Stream`] per node whose `count` is the number of SAMPLES
    /// taken, never the node's element count. Ask [`GpuCook::shape`] for the
    /// count; see the module docs for why the tap must not be asked.
    ///
    /// `None` when there is nothing to sample (no streams retained for this
    /// frame, or every node empty) — distinct from an empty map, which would say
    /// "I sampled and found nothing".
    ///
    /// Requires the cook's streams to still be alive, which is why the shell
    /// calls this inside the same frame as the cook and before the next
    /// [`crate::BufferPool::reclaim`].
    pub fn tap(&mut self, gpu: &GpuContext, samples: u32) -> Option<BTreeMap<NodeId, Stream>> {
        let pedido = Pedido {
            cartoes: Some(samples),
            tomadas: Vec::new(),
        };
        let (staging, slots) = self.encomenda_tap(gpu, &pedido, false)?;
        let slice = staging.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |r| {
            let _ = tx.send(r);
        });
        let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
        rx.recv().ok()?.ok()?;
        Some(le_tap(&staging, &slots).cartoes)
    }

    /// **A metade que ENCOMENDA** — codifica o gather num encoder SEU e submete-o. Partilhada pelo
    /// [`Self::tap`] síncrono e pelo arranque da leitura sem espera ([`crate::tap_voo`]); no
    /// regime, a leitura vai no submit do PRÓPRIO cozimento ([`Self::codifica_tap`], doc 122 §5).
    /// `persistente`: a leitura `MAP_READ` guardada (só a sem espera).
    pub(crate) fn encomenda_tap(
        &mut self,
        gpu: &GpuContext,
        pedido: &Pedido,
        persistente: bool,
    ) -> Option<(wgpu::Buffer, Vec<Slot>)> {
        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("ph2d-gpu-cook tap"),
            });
        let feito = self.codifica_tap(gpu, &mut encoder, pedido, persistente)?;
        gpu.queue.submit(Some(encoder.finish()));
        Some(feito)
    }

    /// **A metade que CODIFICA** — o gather de todas as colunas e a cópia para a leitura, no
    /// `encoder` dado; devolve a leitura ainda por mapear e o mapa das colunas. As duas portas
    /// acima e o cozimento leem exactamente as mesmas amostras por aqui.
    pub(crate) fn codifica_tap(
        &mut self,
        gpu: &GpuContext,
        encoder: &mut wgpu::CommandEncoder,
        pedido: &Pedido,
        persistente: bool,
    ) -> Option<(wgpu::Buffer, Vec<Slot>)> {
        let streams: Vec<(NodeId, &GpuStream)> =
            self.tap_streams.iter().map(|(n, s)| (*n, s)).collect();
        if streams.is_empty() {
            return None;
        }
        // Lay out the packed destination: every (node, column) gets a contiguous
        // run of `samples * lanes` floats.
        let mut slots: Vec<Slot> = Vec::new();
        let mut total: u32 = 0;
        let cartoes = pedido
            .cartoes
            .into_iter()
            .flat_map(|n| streams.iter().map(move |(node, s)| (*node, *s, n, false)));
        let tomadas = pedido.tomadas.iter().filter_map(|&(node, tecto)| {
            streams
                .iter()
                .find(|(n, _)| *n == node)
                .map(|(_, s)| (node, *s, tecto, true))
        });
        for (node, stream, samples, tomada) in cartoes.chain(tomadas) {
            if stream.count == 0 {
                continue;
            }
            let take = samples.min(stream.count);
            for (name, col) in &stream.cols {
                if !cabe_numa_coluna(col.dim) {
                    continue;
                }
                let lanes = crate::stream::element_stride(col.dim) as u32 / 4;
                slots.push(Slot {
                    node,
                    column: name.clone(),
                    dim: col.dim,
                    samples: take,
                    offset: total,
                    tomada,
                });
                total += take * lanes;
            }
        }
        if total == 0 {
            return None;
        }

        let pipe = self
            .tap_pipeline
            .get_or_insert_with(|| TapPipeline::new(gpu));
        // Os parâmetros de cada fatia, num buffer só: uma fatia por coluna, no alinhamento que o
        // dispositivo exige a um deslocamento de uniform.
        let passo = u64::from(gpu.device.limits().min_uniform_buffer_offset_alignment).max(16);
        let precisa = passo * slots.len() as u64;
        if pipe.params.as_ref().is_none_or(|b| b.size() < precisa) {
            pipe.params = Some(gpu.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("ph2d-gpu-cook tap params"),
                size: precisa.next_power_of_two().max(passo),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }));
        }
        let params_buf = pipe.params.as_ref().expect("acabou de ser garantido");
        let mut empacotado = vec![0u8; precisa as usize];
        for (k, slot) in slots.iter().enumerate() {
            let stream = streams
                .iter()
                .find(|(n, _)| *n == slot.node)
                .map(|(_, s)| *s)
                .expect("slot came from this list");
            let col = stream
                .get(&slot.column)
                .expect("slot came from this stream");
            let lanes = crate::stream::element_stride(col.dim) as u32 / 4;
            let params = [stream.count, lanes, slot.samples, slot.offset];
            let ini = k * passo as usize;
            empacotado[ini..ini + 16].copy_from_slice(bytemuck::cast_slice(&params));
        }
        gpu.queue.write_buffer(params_buf, 0, &empacotado);
        let bytes = u64::from(total) * 4;
        let dst = do_tamanho(
            gpu,
            &mut pipe.dst,
            bytes,
            wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            "ph2d-gpu-cook tap dst",
        );
        // One pass, one dispatch per column — all inside a single encoder, so the
        // whole tap is one submit no matter how many nodes are being watched.
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("ph2d-gpu-cook tap pass"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&pipe.pipeline);
            for (k, slot) in slots.iter().enumerate() {
                let stream = streams
                    .iter()
                    .find(|(n, _)| *n == slot.node)
                    .map(|(_, s)| *s)
                    .expect("slot came from this list");
                let col = stream
                    .get(&slot.column)
                    .expect("slot came from this stream");
                let bg = pipe.ligacoes.grupo_de_fatias(
                    gpu,
                    &pipe.pipeline,
                    Some(&pipe.layout),
                    &[
                        (
                            0,
                            params_buf,
                            k as u64 * passo,
                            std::num::NonZeroU64::new(16),
                        ),
                        (1, &col.buffer, 0, None),
                        (2, &dst, 0, None),
                    ],
                );
                pass.set_bind_group(0, &bg, &[]);
                pass.dispatch_workgroups(slot.samples.div_ceil(64), 1, 1);
            }
        }
        let uso = wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ;
        let staging = if persistente {
            do_tamanho(
                gpu,
                &mut pipe.staging,
                bytes,
                uso,
                "ph2d-gpu-cook tap staging",
            )
        } else {
            gpu.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("ph2d-gpu-cook tap staging"),
                size: bytes,
                usage: uso,
                mapped_at_creation: false,
            })
        };
        encoder.copy_buffer_to_buffer(&dst, 0, &staging, 0, bytes);
        pipe.ligacoes.vira();
        Some((staging, slots))
    }
}

/// **A leitura só leva as colunas que um [`Column`] exprime** (`Scalar`…`Vec4`). Uma `Mat*` é
/// estado privado de um kernel (o anel `ss_ring*` do `Slit Scan`, `Mat4`, 16 números por
/// elemento) que nenhum cartão mostra; lida como `Vec4` ela dava `count × 4` elementos e o
/// `Stream::set` pânicava (`144` contra `36`, a cena `=118` ao 12.º quadro do app).
fn cabe_numa_coluna(dim: Dim) -> bool {
    matches!(dim, Dim::Scalar | Dim::Vec2 | Dim::Vec3 | Dim::Vec4)
}

/// **A metade que LÊ** — um `staging` já MAPEADO de volta em streams, e desmapeia-o.
pub(crate) fn le_tap(staging: &wgpu::Buffer, slots: &[Slot]) -> Leitura {
    let flat: Vec<f32> = bytemuck::cast_slice(&staging.slice(..).get_mapped_range()).to_vec();
    staging.unmap();
    let mut leitura = Leitura::default();
    for slot in slots {
        let lanes = crate::stream::element_stride(slot.dim) as usize / 4;
        let n = slot.samples as usize;
        let base = slot.offset as usize;
        let run = &flat[base..base + n * lanes];
        let col = match slot.dim {
            Dim::Scalar => Column::Scalar(run.to_vec()),
            Dim::Vec2 => Column::Vec2(
                run.as_chunks::<2>()
                    .0
                    .iter()
                    .map(|c| [c[0], c[1]])
                    .collect(),
            ),
            // Vec3 pads to 16 bytes (4 lanes) — take the first three.
            Dim::Vec3 => Column::Vec3(
                run.chunks_exact(lanes)
                    .map(|c| [c[0], c[1], c[2]])
                    .collect(),
            ),
            Dim::Vec4 => Column::Vec4(
                run.as_chunks::<4>()
                    .0
                    .iter()
                    .map(|c| [c[0], c[1], c[2], c[3]])
                    .collect(),
            ),
            // O [`GpuCook::codifica_tap`] nunca as encomenda — ver [`cabe_numa_coluna`].
            Dim::Mat2 | Dim::Mat3 | Dim::Mat4 => continue,
        };
        let out = if slot.tomada {
            &mut leitura.tomadas
        } else {
            &mut leitura.cartoes
        };
        out.entry(slot.node)
            .or_insert_with(|| Stream::new(n))
            .set(slot.column.clone(), col);
    }
    leitura
}
