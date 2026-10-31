//! As transferências CPU ⇄ placa do [`AdvectGpu`]: a leitura de um buffer (inteiro ou por faixa) e a subida de uma
//! faixa de linhas (o espelho de [`crate::passo`]).

use crate::{AdvectGpu, pack_active, pack_state};
use bytemuck::Pod;
use ph2d_gpu::GpuContext;
use ph2d_wet_paint::grid::Grid;

impl AdvectGpu {
    /// Sobe só as linhas finas `y0..=y1` e as de fluxo `f0..=f1` (o que a CPU mudou, [`crate::passo`]).
    pub fn upload_linhas(
        &self,
        gpu: &GpuContext,
        g: &Grid,
        (y0, y1): (usize, usize),
        (f0, f1): (usize, usize),
    ) {
        let q = &gpu.queue;
        let (a, b) = (y0 * g.s, (y1 + 1) * g.s);
        let w = |buf: &wgpu::Buffer, d: &[u8], k: usize| q.write_buffer(buf, (a * k) as u64, d);
        w(
            &self.active,
            bytemuck::cast_slice(&pack_active(&g.active[a..b])),
            4,
        );
        w(
            &self.st,
            bytemuck::cast_slice(&pack_state(&g.film[a..b], &g.susp[a..b])),
            8,
        );
        w(&self.rgb, bytemuck::cast_slice(&g.susp_rgb[a..b]), 12);
        w(&self.paper, bytemuck::cast_slice(&g.paper[a..b]), 4);
        w(
            &self.wet,
            bytemuck::cast_slice(&pack_active(&g.wet[a..b])),
            4,
        );
        w(&self.sett, bytemuck::cast_slice(&g.sett[a..b]), 4);
        if !g.traco.is_empty() {
            w(&self.traco, bytemuck::cast_slice(&g.traco[a..b]), 4);
        }
        let (fa, fb) = (f0 * g.flow.s, (f1 + 1) * g.flow.s);
        let o = (fa * 4) as u64;
        q.write_buffer(&self.flow_x, o, bytemuck::cast_slice(&g.flow_x[fa..fb]));
        q.write_buffer(&self.flow_y, o, bytemuck::cast_slice(&g.flow_y[fa..fb]));
        q.write_buffer(&self.vel_x, o, bytemuck::cast_slice(&g.vel_x[fa..fb]));
        q.write_buffer(&self.vel_y, o, bytemuck::cast_slice(&g.vel_y[fa..fb]));
    }
}

/// Copia `bytes` de `src` para um buffer mapeável e devolve-o tipado.
#[must_use]
pub fn read<T: Pod>(gpu: &GpuContext, src: &wgpu::Buffer, bytes: u64) -> Vec<T> {
    read_at(gpu, src, 0, bytes)
}

/// [`read`] de `bytes` a partir de `offset` (múltiplos de 4).
#[must_use]
pub fn read_at<T: Pod>(gpu: &GpuContext, src: &wgpu::Buffer, offset: u64, bytes: u64) -> Vec<T> {
    let stage = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("wet-gpu readback"),
        size: bytes,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut enc = gpu.device.create_command_encoder(&Default::default());
    enc.copy_buffer_to_buffer(src, offset, &stage, 0, bytes);
    gpu.queue.submit([enc.finish()]);
    stage
        .slice(..)
        .map_async(wgpu::MapMode::Read, |r| r.expect("map"));
    let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
    let v = {
        let d = stage.slice(..).get_mapped_range().expect("mapped range");
        bytemuck::cast_slice::<u8, T>(&d).to_vec()
    };
    stage.unmap();
    v
}
