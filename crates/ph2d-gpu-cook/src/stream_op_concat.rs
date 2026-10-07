//! **A concatenação e a junção** (ADR-0136 · ciclo 7) — `motion.combine` e a junção de um `Carry`:
//! cópias e enchimentos por região de coluna, sem leitura de volta. Irmão do [`super`] pelo tecto
//! de LOC; a lei é a mesma.

use super::{FillRegion, StreamOpPipes};
use crate::{GpuColumn, GpuCook, GpuStream, stream};
use ph2d_gpu::GpuContext;
use ph2d_nodegraph::gpu::ConcatFill;
use ph2d_nodegraph::port::Dim;
use std::collections::BTreeMap;

impl GpuCook {
    /// `motion.combine` (ADR-0136): the listed ports laid end to end — column
    /// union, first-seen dim as the prototype, zeros where an input lacks the
    /// column **or carries it at another dim** (the CPU's variant-match rule).
    /// Pure copies and clears; the count is a host-side sum.
    pub(crate) fn encode_concat(
        &mut self,
        gpu: &GpuContext,
        encoder: &mut wgpu::CommandEncoder,
        ports: &[usize],
        inputs: &[GpuStream],
    ) -> GpuStream {
        let fontes: Vec<&GpuStream> = ports.iter().filter_map(|p| inputs.get(*p)).collect();
        self.encode_join(gpu, encoder, &fontes, &[])
    }

    /// **A junção** — as fontes, pela ordem, ponta a ponta: a união das colunas, a dimensão
    /// da PRIMEIRA fonte não-vazia que a tem como protótipo, e onde uma fonte não a tem (ou a
    /// tem noutra dimensão) a IDENTIDADE que as `fills` dão — zeros quando nenhuma a nomeia
    /// (o `motion.combine`), o `default_for` da CPU num `Carry` (ciclo 7: um `size`/`tint` a
    /// zero apagava o eco). Fontes vazias saltam-se (a regra de snapshot da CPU).
    pub(crate) fn encode_join(
        &mut self,
        gpu: &GpuContext,
        encoder: &mut wgpu::CommandEncoder,
        fontes: &[&GpuStream],
        fills: &[ConcatFill],
    ) -> GpuStream {
        let live: Vec<&GpuStream> = fontes.iter().copied().filter(|s| s.count > 0).collect();
        let total64: u64 = live.iter().map(|s| u64::from(s.count)).sum();
        let total = total64.min(u64::from(u32::MAX)) as u32;
        if total == 0 {
            return GpuStream::default();
        }
        // Ordered column union: the prototype dim is the FIRST live input
        // carrying the name (the CPU's `find_map` over snaps).
        let mut protos: Vec<(String, Dim)> = Vec::new();
        for s in &live {
            for (name, col) in &s.cols {
                if !protos.iter().any(|(n, _)| n == name) {
                    protos.push((name.clone(), col.dim));
                }
            }
        }
        let mut out = GpuStream {
            count: total,
            cols: BTreeMap::new(),
        };
        let mut hold: Vec<wgpu::Buffer> = Vec::new();
        for (name, dim) in protos {
            let stride = stream::element_stride(dim);
            let dst = self.pool.acquire(gpu, u64::from(total) * stride);
            let identidade = ConcatFill::of(fills, &name, dim);
            let mut off: u64 = 0;
            for s in &live {
                let bytes = u64::from(s.count) * stride;
                match s.cols.get(&name) {
                    Some(c) if c.dim == dim => {
                        encoder.copy_buffer_to_buffer(&c.buffer, 0, &dst, off, bytes);
                    }
                    _ if identidade == [0.0; 4] => encoder.clear_buffer(&dst, off, Some(bytes)),
                    _ => {
                        let pipes = self
                            .stream_op_pipes
                            .get_or_insert_with(|| StreamOpPipes::new(gpu));
                        pipes.fill(
                            gpu,
                            encoder,
                            FillRegion {
                                dst: &dst,
                                n: s.count,
                                words: (stride / 4) as u32,
                                first: (off / stride) as u32,
                                value: identidade,
                            },
                            &mut hold,
                        );
                    }
                }
                off += bytes;
            }
            out.cols.insert(name, GpuColumn { buffer: dst, dim });
        }
        self.stream_op_hold.append(&mut hold);
        out
    }
}
