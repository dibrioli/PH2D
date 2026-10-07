//! **O nascimento nas linhas que DISPARAM** (doc 110 §14.1 (7)) — o `pulse` do `sim.spawn` na
//! placa. Um estágio = a CONCATENAÇÃO de dois nascimentos ([`FiredBirth`]):
//! 1. a taxa, pela variante `rate_kernel`, como qualquer `SourceRows`;
//! 2. o pulso: a porta lateral ganha a coluna [`FIRED_ROW_COL`] (`iota`), é COMPACTADA pelo
//!    predicado — a mesma varredura e a mesma leitura de 8 bytes do `Compact` —, e o `kernel`
//!    corre na janela da lei DELE, que vê nessa porta a LARGURA `F` das linhas que dispararam.

use super::PredicateExtras;
use crate::count;
use crate::{GpuColumn, GpuCook, GpuCookError, GpuStream};
use ph2d_gpu::GpuContext;
use ph2d_nodegraph::gpu::{FIRED_ROW_COL, FiredBirth, GpuKernel};
use ph2d_nodegraph::graph::{Graph, NodeId};
use ph2d_nodegraph::node::NodeManifest;
use ph2d_nodegraph::port::Dim;

/// `dst[i] = f32(i)` — exacto abaixo de `2²⁴`, que é o tecto do modelo de ids.
pub(super) fn iota_module() -> String {
    super::simple_module(
        "@group(0) @binding(1) var<storage, read_write> dst: array<f32>;",
        "\x20   dst[i] = f32(i);",
    )
}

impl GpuCook {
    /// Os dois nascimentos, concatenados (taxa primeiro — a ordem do `eval`). `slots` são três
    /// slots de uniform distintos: a taxa, o predicado e o pulso.
    #[allow(clippy::too_many_arguments)] // a costura do sequenciador, como `encode_kernel_stage`
    pub(crate) fn encode_disparo(
        &mut self,
        gpu: &GpuContext,
        encoder: &mut wgpu::CommandEncoder,
        slots: [usize; 3],
        f: &FiredBirth,
        (template, nao_herda): (usize, &[&str]),
        (graph, node, manifest): (&Graph, NodeId, &'static NodeManifest),
        (playhead, dt): (f64, f64),
        (inputs, raw_counts): (&[GpuStream], &[u32]),
    ) -> Result<GpuStream, GpuCookError> {
        let no = (graph, node, manifest);
        let taxa = self.nascer(
            gpu,
            encoder,
            slots[0],
            &f.rate_kernel,
            no,
            (playhead, dt),
            (inputs, template, nao_herda),
            (&[], raw_counts),
        );

        // As linhas que dispararam, com a linha ORIGINAL de cada uma.
        let mut lateral = inputs.to_vec();
        let pulso = match lateral.get_mut(f.port) {
            Some(s) if s.count > 0 => {
                let n = s.count;
                let dst = self.pool.acquire(gpu, u64::from(n) * 4);
                let mut hold: Vec<wgpu::Buffer> = Vec::new();
                let pipes = self
                    .stream_op_pipes
                    .get_or_insert_with(|| super::StreamOpPipes::new(gpu));
                pipes.pass(gpu, encoder, &pipes.iota, n, 1, 0, &[&dst], &mut hold);
                self.stream_op_hold.append(&mut hold);
                s.cols.insert(
                    FIRED_ROW_COL.to_string(),
                    GpuColumn {
                        buffer: dst,
                        dim: Dim::Scalar,
                    },
                );
                let (disparos, _) = self.encode_compact(
                    gpu,
                    encoder,
                    slots[1],
                    &f.predicate,
                    graph,
                    node,
                    manifest,
                    playhead,
                    dt,
                    &lateral,
                    f.port,
                    PredicateExtras {
                        reduces: (&[], &[]),
                        shared: "",
                        derived: (f.derived, raw_counts),
                    },
                    &[],
                )?;
                lateral[f.port] = disparos;
                let largura: Vec<u32> = lateral.iter().map(|s| s.count).collect();
                self.nascer(
                    gpu,
                    encoder,
                    slots[2],
                    &f.kernel,
                    no,
                    (playhead, dt),
                    (&lateral, template, nao_herda),
                    (f.derived, &largura),
                )
            }
            _ => GpuStream::default(),
        };
        Ok(self.encode_concat(gpu, encoder, &[0, 1], &[taxa, pulso]))
    }

    /// Um nascimento `SourceRows`: a janela da lei do `kernel`, o despacho sobre uma base NOVA e o
    /// *gather* do modelo nas linhas que ele escreveu.
    #[allow(clippy::too_many_arguments)]
    fn nascer(
        &mut self,
        gpu: &GpuContext,
        encoder: &mut wgpu::CommandEncoder,
        slot: usize,
        kernel: &GpuKernel,
        (graph, node, manifest): (&Graph, NodeId, &'static NodeManifest),
        (playhead, dt): (f64, f64),
        (inputs, template, nao_herda): (&[GpuStream], usize, &[&str]),
        derived: (&'static [ph2d_nodegraph::gpu::DerivedUniform], &[u32]),
    ) -> GpuStream {
        let driven = self.driven.clone();
        let no = count::No {
            graph,
            node,
            manifest,
            driven: &driven,
        };
        let window = count::stage_window(kernel, no, inputs, playhead, dt);
        let n = window.count.min(u32::MAX as usize) as u32;
        if n == 0 {
            return GpuStream::default();
        }
        let out = self.encode_kernel_stage(
            gpu,
            encoder,
            slot,
            0,
            kernel,
            graph,
            node,
            manifest,
            window,
            playhead,
            dt,
            inputs,
            GpuStream::default(),
            None,
            (&[], &[]),
            (&[], &[]),
            "",
            derived,
        );
        self.encode_source_gather(gpu, encoder, out, inputs.get(template), n, nao_herda)
    }
}
