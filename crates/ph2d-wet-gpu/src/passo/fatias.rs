//! As submissões de um passo cortadas pelo orçamento, com carimbos por passada.

use ph2d_gpu::GpuContext;

/// Carimbos por passo: 2 por passada (rebuild 5 + porta 1 + secagem 4 + plano 24 = 34 ⇒ 68).
pub(super) const MAX_CARIMBOS: u32 = 96;

pub(super) enum Despacho<'a> {
    Direto(u32, u32),
    Indireto(&'a wgpu::Buffer, u64),
}

/// As submissões de um passo, cortadas pelo orçamento; cada passada leva carimbos de início e fim.
pub(super) struct Fatias<'g> {
    pub(super) gpu: &'g GpuContext,
    pub(super) enc: wgpu::CommandEncoder,
    custo: u64,
    orcamento: u64,
    qs: Option<wgpu::QuerySet>,
    pub(super) q: u32,
    ini: u32,
    /// `(1.º carimbo, último)` de cada submissão já cortada.
    pub(super) cortes: Vec<(u32, u32)>,
}

impl<'g> Fatias<'g> {
    pub(super) fn new(gpu: &'g GpuContext, orcamento: u64, qs: Option<wgpu::QuerySet>) -> Self {
        Fatias {
            gpu,
            enc: gpu.device.create_command_encoder(&Default::default()),
            custo: 0,
            orcamento,
            qs,
            q: 0,
            ini: 0,
            cortes: Vec::new(),
        }
    }

    pub(super) fn passada(
        &mut self,
        pipe: &wgpu::ComputePipeline,
        bg: &wgpu::BindGroup,
        d: Despacho<'_>,
        custo: u64,
    ) {
        if self.custo > 0 && self.custo + custo > self.orcamento {
            self.corta();
        }
        let tw = self
            .qs
            .as_ref()
            .filter(|_| self.q + 2 <= MAX_CARIMBOS)
            .map(|qs| wgpu::ComputePassTimestampWrites {
                query_set: qs,
                beginning_of_pass_write_index: Some(self.q),
                end_of_pass_write_index: Some(self.q + 1),
            });
        if tw.is_some() {
            self.q += 2;
        }
        let mut p = self.enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: None,
            timestamp_writes: tw,
        });
        p.set_pipeline(pipe);
        p.set_bind_group(0, bg, &[]);
        match d {
            Despacho::Direto(c, r) => p.dispatch_workgroups(c, r, 1),
            Despacho::Indireto(args, o) => p.dispatch_workgroups_indirect(args, o),
        }
        drop(p);
        self.custo += custo;
    }

    /// Fecha a submissão corrente (se tem passadas) e abre outra.
    pub(super) fn corta(&mut self) {
        let enc = std::mem::replace(
            &mut self.enc,
            self.gpu.device.create_command_encoder(&Default::default()),
        );
        self.gpu.queue.submit([enc.finish()]);
        if self.q > self.ini {
            self.cortes.push((self.ini, self.q - 1));
        }
        self.ini = self.q;
        self.custo = 0;
    }
}
