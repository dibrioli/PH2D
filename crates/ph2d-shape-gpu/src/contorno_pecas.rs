//! doc 121 §9.18 (E) — **a emissão tracejada por PEÇA** (`contorno_pecas.wgsl`): as linhas dos troços, o
//! estado por cópia e os três passes dela, que correm DENTRO do passe da escrita (o relógio `escreve`).

use super::*;

/// Onde estão, no buffer do despacho, os argumentos de um fio por PEÇA (`POR_PECA_ARGS` no WGSL).
const POR_PECA_ARGS: u64 = 68;
/// Bytes por cópia em `porcopia_rw` (`PORCOPIA` palavras no WGSL).
const PORCOPIA: u64 = 8 * 4;
/// Bytes de uma LINHA da tabela dos troços (dois `vec4<u32>`); uma por bloco de arestas reservado.
const LINHA: u64 = 32;

pub(super) struct Pecas {
    /// O `POR_PECA` das constantes do passe; `0` ⇒ nenhum destes passes corre.
    pub(super) modo: u32,
    pub(super) soma: wgpu::ComputePipeline,
    pub(super) pecas: wgpu::ComputePipeline,
    pub(super) fecha: wgpu::ComputePipeline,
    trocos: wgpu::Buffer,
    porcopia: wgpu::Buffer,
}

/// O `POR_PECA` pedido nas constantes `override` (a omissão do WGSL: `0`).
pub(super) fn modo(constantes: &[(&str, f64)]) -> u32 {
    constantes
        .iter()
        .find(|(k, _)| *k == "POR_PECA")
        .map_or(0, |(_, v)| *v as u32)
}

impl Pecas {
    pub(super) fn new(
        gpu: &GpuContext,
        modo: u32,
        [soma, pecas, fecha]: [wgpu::ComputePipeline; 3],
    ) -> Self {
        let armazens = wgpu::BufferUsages::STORAGE;
        Self {
            modo,
            soma,
            pecas,
            fecha,
            trocos: buffer(gpu, "ph2d-shape-gpu linhas dos trocos", 16, armazens),
            porcopia: buffer(gpu, "ph2d-shape-gpu pecas por copia", 16, armazens),
        }
    }

    /// As ligações `9` e `10` dos grupos `2`.
    pub(super) fn entradas(&self) -> [wgpu::BindGroupEntry<'_>; 2] {
        [
            wgpu::BindGroupEntry {
                binding: 9,
                resource: self.trocos.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 10,
                resource: self.porcopia.as_entire_binding(),
            },
        ]
    }

    /// Os buffers para `copias` cópias e `arestas` arestas reservadas (só com o modo ligado).
    pub(super) fn garante(&mut self, gpu: &GpuContext, copias: Option<u64>, arestas: Option<u64>) {
        if self.modo == 0 {
            return;
        }
        let armazens = wgpu::BufferUsages::STORAGE;
        if let Some(c) = copias {
            self.porcopia = buffer(gpu, "ph2d-shape-gpu pecas por copia", c * PORCOPIA, armazens);
        }
        if let Some(a) = arestas {
            self.trocos = buffer(
                gpu,
                "ph2d-shape-gpu linhas dos trocos",
                a / BLOCO * LINHA,
                armazens,
            );
        }
    }

    /// Os passes por peça, depois do `cs_escreve` e antes do `cs_soma_escritas`. O `cs_pecas` é indirecto
    /// e lê o `despacho` como argumento: corre com o grupo das células (sem a ligação `7`).
    pub(super) fn escreve(
        &self,
        pass: &mut wgpu::ComputePass<'static>,
        [escrita, celulas]: [&wgpu::BindGroup; 2],
        despacho: &wgpu::Buffer,
        (x, y): (u32, u32),
    ) {
        if self.modo == 0 {
            return;
        }
        pass.set_pipeline(&self.soma);
        pass.dispatch_workgroups(1, 1, 1);
        pass.set_bind_group(2, celulas, &[]);
        pass.set_pipeline(&self.pecas);
        pass.dispatch_workgroups_indirect(despacho, POR_PECA_ARGS);
        pass.set_bind_group(2, escrita, &[]);
        if matches!(self.modo, 2 | 3) {
            pass.set_pipeline(&self.fecha);
            pass.dispatch_workgroups(x, y, 1);
        }
    }
}
