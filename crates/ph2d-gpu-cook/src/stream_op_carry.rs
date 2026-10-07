//! **O `Carry`** (ciclo 7, W1d — o `motion.trail`): `filtrar(estado) ++ vivo`, e só depois o
//! kernel do nó corre sobre a junção. Irmão do [`super`] pelo tecto de LOC do `lib.rs`; a lei e a
//! ordem são as de sempre (ver [`StreamOp::Carry`]).

use super::PredicateExtras;
use crate::plan::resolve_param;
use crate::{GpuCook, GpuCookError, GpuStream};
use ph2d_gpu::GpuContext;
use ph2d_nodegraph::gpu::{CountLawCtx, GpuKernel, KernelResolver, StreamOp};
use ph2d_nodegraph::graph::{Graph, NodeId};
use ph2d_nodegraph::node::{NodeManifest, NodeTypeId};

/// O que um `Carry` devolve ao sequenciador.
pub(crate) enum Carregado<'k> {
    /// A CPU devolve a entrada viva: corre este kernel sobre ela (a base é a porta 0, a viva).
    Identidade(&'k GpuKernel),
    /// A junção — a nova base, sobre a qual corre o kernel do nó.
    Junto(GpuStream),
}

impl GpuCook {
    /// 1. as reduções do predicado, sobre o estado CRU; 2. o filtro; 3. a junção.
    #[allow(clippy::too_many_arguments)] // a costura do sequenciador, como `encode_kernel_stage`
    pub(crate) fn encode_carry<'k>(
        &mut self,
        gpu: &GpuContext,
        encoder: &mut wgpu::CommandEncoder,
        slot: usize,
        op: &'k StreamOp,
        kernels: &dyn KernelResolver,
        (graph, node, ty, manifest): (&Graph, NodeId, NodeTypeId, &'static NodeManifest),
        (playhead, dt): (f64, f64),
        (inputs, raw_counts): (&mut [GpuStream], &[u32]),
    ) -> Result<Carregado<'k>, GpuCookError> {
        let StreamOp::Carry {
            state_port,
            live_port,
            predicate,
            predicate_reduces,
            fills,
            identity,
            identity_kernel,
        } = op
        else {
            unreachable!("encode_carry só recebe um Carry");
        };
        let derived = kernels.derived_uniforms(ty);
        let param = |name: &str| resolve_param(graph, node, manifest, name, &self.driven);
        let ctx = CountLawCtx {
            inputs: raw_counts,
            param: &param,
            playhead,
            dt,
        };
        if identity(&ctx) {
            return Ok(Carregado::Identidade(identity_kernel));
        }
        let shared = kernels.wgsl_shared(ty);
        let pred_red = self.run_reduces(
            gpu,
            encoder,
            predicate_reduces,
            inputs,
            graph,
            node,
            manifest,
            shared,
        );
        let (carried, _) = self.encode_compact(
            gpu,
            encoder,
            slot,
            predicate,
            graph,
            node,
            manifest,
            playhead,
            dt,
            inputs,
            *state_port,
            PredicateExtras {
                reduces: (predicate_reduces, &pred_red.buffers),
                shared,
                derived: (derived, raw_counts),
            },
            &[],
        )?;
        self.reduce_results_hold.push(pred_red);
        let live = inputs.get(*live_port).cloned().unwrap_or_default();
        let joined = self.encode_join(gpu, encoder, &[&carried, &live], fills);
        if *state_port < inputs.len() {
            inputs[*state_port] = carried;
        }
        if *live_port < inputs.len() {
            inputs[*live_port] = joined.clone();
        }
        Ok(Carregado::Junto(joined))
    }
}
