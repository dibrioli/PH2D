//! **O COMPLEMENTO de um `Compact`** (doc 110 §14.1 (1)) — as linhas que o predicado TIROU, da
//! MESMA varredura que achou as que ficam: a removida `i` vai para `i − scan[i]` (quantas
//! removidas a precedem — a ordem original, a da CPU), e a contagem é `n − total`, da MESMA
//! leitura de 8 bytes. Zero leituras a mais; um passe, e o *gather* por coluna.

use super::{FillRegion, StreamOpPipes};
use crate::{GpuColumn, GpuCook, GpuStream, stream};
use ph2d_gpu::GpuContext;
use ph2d_nodegraph::gpu::{Complement, ComplementPort};
use ph2d_nodegraph::port::Dim;
use std::collections::BTreeMap;

/// `if flags[i] < 0.5 { rows[i − scan[i]] = i }` — o *scatter* das removidas.
pub(super) fn rows_module() -> String {
    super::simple_module(
        "@group(0) @binding(1) var<storage, read> flags: array<f32>;\n\
         @group(0) @binding(2) var<storage, read> scan_data: array<u32>;\n\
         @group(0) @binding(3) var<storage, read_write> rows: array<u32>;",
        "\x20   if (flags[i] < 0.5) { rows[i - scan_data[i]] = i; }",
    )
}

/// Uma porta do complemento e as colunas (já alocadas) que ela vai levar.
type Plano = (u16, Complement, Vec<(String, GpuColumn)>);

/// O que a compactação deixa ao complemento: a fonte filtrada, as bandeiras, a varredura
/// exclusiva e quantas ficaram.
pub(super) struct Varredura<'a> {
    pub src: &'a GpuStream,
    pub flags: &'a wgpu::Buffer,
    pub scan: &'a wgpu::Buffer,
    pub total: u32,
}

impl GpuCook {
    /// Uma corrente por [`ComplementPort`] — `Rows`: todas as colunas da fonte nas linhas
    /// removidas; `Event(c)`: só `c`, a `1.0`. Nenhuma removida ⇒ nenhuma corrente (o vazio, como
    /// o `Compact` devolve o vazio quando nada fica).
    pub(super) fn encode_complement(
        &mut self,
        gpu: &GpuContext,
        encoder: &mut wgpu::CommandEncoder,
        v: Varredura<'_>,
        complement: &[ComplementPort],
    ) -> Vec<(u16, GpuStream)> {
        let n = v.src.count;
        let k = n.saturating_sub(v.total);
        if complement.is_empty() || k == 0 {
            return Vec::new();
        }
        let crow = self.pool.acquire(gpu, u64::from(k) * 4);
        let mut hold: Vec<wgpu::Buffer> = Vec::new();
        // Os buffers de saída primeiro: o `pool` e as tubagens são campos distintos.
        let mut planos: Vec<Plano> = Vec::new();
        for c in complement {
            let cols = match c.carries {
                Complement::Rows => v
                    .src
                    .cols
                    .iter()
                    .map(|(name, col)| {
                        let bytes = u64::from(k) * stream::element_stride(col.dim);
                        let buffer = self.pool.acquire(gpu, bytes);
                        (
                            name.clone(),
                            GpuColumn {
                                buffer,
                                dim: col.dim,
                            },
                        )
                    })
                    .collect(),
                Complement::Event(col) => vec![(
                    col.to_string(),
                    GpuColumn {
                        buffer: self.pool.acquire(gpu, u64::from(k) * 4),
                        dim: Dim::Scalar,
                    },
                )],
            };
            planos.push((c.port, c.carries, cols));
        }
        let pipes: &StreamOpPipes = self
            .stream_op_pipes
            .as_ref()
            .expect("built by the compaction");
        pipes.pass(
            gpu,
            encoder,
            &pipes.complement_rows,
            n,
            1,
            0,
            &[v.flags, v.scan, &crow],
            &mut hold,
        );
        let mut out = Vec::with_capacity(planos.len());
        for (port, carries, cols) in planos {
            for (name, dst) in &cols {
                match carries {
                    Complement::Rows => {
                        let src = &v.src.cols[name];
                        let words = (stream::element_stride(src.dim) / 4) as u32;
                        pipes.pass(
                            gpu,
                            encoder,
                            &pipes.gather_u32,
                            k,
                            words,
                            0,
                            &[&crow, &src.buffer, &dst.buffer],
                            &mut hold,
                        );
                    }
                    Complement::Event(_) => pipes.fill(
                        gpu,
                        encoder,
                        FillRegion {
                            dst: &dst.buffer,
                            n: k,
                            words: 1,
                            first: 0,
                            value: [1.0, 0.0, 0.0, 0.0],
                        },
                        &mut hold,
                    ),
                }
            }
            let cols: BTreeMap<String, GpuColumn> = cols.into_iter().collect();
            out.push((port, GpuStream { count: k, cols }));
        }
        self.stream_op_hold.append(&mut hold);
        self.stream_op_hold_bufs.push(crow);
        out
    }
}
