//! **A projecção por NOME** (`value.attribute`, ADR-0136) — cortada do `stream_op.rs` pelo
//! tecto de LOC (ciclo 7, W1d: a junção do rastro entrou lá), movida verbatim. Filho do
//! `stream_op` para ver os pipelines privados dele.

use super::StreamOpPipes;
use crate::plan::resolve_param;
use crate::{GpuColumn, GpuCook, GpuStream, stream};
use ph2d_gpu::GpuContext;
use ph2d_nodegraph::graph::{Graph, NodeId};
use ph2d_nodegraph::node::NodeManifest;
use ph2d_nodegraph::port::Dim;
use std::collections::BTreeMap;
use std::sync::Arc;

impl GpuCook {
    /// `value.attribute` (ADR-0136): project the column NAMED BY A TEXT PARAM as
    /// the value field `v`. The CPU ladder, exactly: a scalar column in scalar
    /// mode is a copy; a vec2 column in length mode is the magnitude kernel;
    /// anything else — missing, mistyped, wrong dim for the mode — is zeros at
    /// full length, never an error and never an empty broadcast.
    #[allow(clippy::too_many_arguments)] // private seam of `cook`
    pub(crate) fn encode_project(
        &mut self,
        gpu: &GpuContext,
        encoder: &mut wgpu::CommandEncoder,
        graph: &Graph,
        node: NodeId,
        manifest: &'static NodeManifest,
        text_param: &str,
        mode_param: &str,
        inputs: &[GpuStream],
    ) -> GpuStream {
        // `value.attribute`'s own constants. ⚠️ They are DUPLICATED here on purpose and the
        // duplication is structural, not laziness: the node crate reaches this one only as a
        // `[dev-dependencies]` (machete-safe — the cook engine must not depend on the nodes it
        // cooks). What keeps them from drifting is a gate in the parity suite, which CAN see
        // both: `the_projection_modes_agree_across_the_dev_dependency_fence`.
        const MODE_LENGTH: i32 = 1;
        const MODE_COMPONENT_BASE: i32 = 2;
        const MODE_ANGLE: i32 = -1;
        let src_stream = inputs.first().cloned().unwrap_or_default();
        let n = src_stream.count;
        if n == 0 {
            return GpuStream::default();
        }
        let name = graph
            .node_text_param_overrides(node)
            .and_then(|m| m.get(text_param))
            .map(String::as_str)
            .unwrap_or("");
        let mode = resolve_param(graph, node, manifest, mode_param, &self.driven).round() as i32;
        let dst: Arc<wgpu::Buffer> = self.pool.acquire(gpu, u64::from(n) * 4);
        let mut hold: Vec<wgpu::Buffer> = Vec::new();
        match (src_stream.cols.get(name), mode) {
            (Some(c), m) if c.dim == Dim::Scalar && m != MODE_LENGTH && m != MODE_ANGLE => {
                encoder.copy_buffer_to_buffer(&c.buffer, 0, &dst, 0, u64::from(n) * 4);
            }
            (Some(c), MODE_LENGTH) if c.dim == Dim::Vec2 => {
                let pipes = self
                    .stream_op_pipes
                    .get_or_insert_with(|| StreamOpPipes::new(gpu));
                pipes.pass(
                    gpu,
                    encoder,
                    &pipes.length,
                    n,
                    1,
                    0,
                    &[&c.buffer, &dst],
                    &mut hold,
                );
            }
            // A DIRECAO — o irmao do length, e o unico braco transcendental da escada.
            (Some(c), MODE_ANGLE) if c.dim == Dim::Vec2 => {
                let pipes = self
                    .stream_op_pipes
                    .get_or_insert_with(|| StreamOpPipes::new(gpu));
                pipes.pass(
                    gpu,
                    encoder,
                    &pipes.angle,
                    n,
                    1,
                    0,
                    &[&c.buffer, &dst],
                    &mut hold,
                );
            }
            // The COMPONENT rung — the CPU's `component()`, lane for lane. The width
            // rides `stride`, so ONE pipeline serves Vec2/Vec3/Vec4 exactly as one CPU
            // arm does; a lane the column does not have falls through to the zeros
            // below, which is the CPU's ordinary miss and not a special case here.
            (Some(c), m)
                if m >= MODE_COMPONENT_BASE
                    && ((m - MODE_COMPONENT_BASE) as u32)
                        < (stream::element_stride(c.dim) / 4) as u32 =>
            {
                let pipes = self
                    .stream_op_pipes
                    .get_or_insert_with(|| StreamOpPipes::new(gpu));
                pipes.pass(
                    gpu,
                    encoder,
                    &pipes.component,
                    n,
                    (stream::element_stride(c.dim) / 4) as u32,
                    (m - MODE_COMPONENT_BASE) as u32,
                    &[&c.buffer, &dst],
                    &mut hold,
                );
            }
            _ => encoder.clear_buffer(&dst, 0, Some(u64::from(n) * 4)),
        }
        self.stream_op_hold.append(&mut hold);
        let mut out = GpuStream {
            count: n,
            cols: BTreeMap::new(),
        };
        out.cols.insert(
            "v".to_string(),
            GpuColumn {
                buffer: dst,
                dim: Dim::Scalar,
            },
        );
        out
    }
}
