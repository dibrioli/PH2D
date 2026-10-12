//! As caixas no ECRÃ de um quadro (sprites ++ formas): dois passes e UMA leitura de volta, à
//! espera da própria submissão — a única sincronização de uma corrida (doc 131 §7 C.10: 0,12 ms da
//! placa a 4 096 formas, uma vez por quadro e não por página).

use ph2d_gpu::GpuContext;
use ph2d_shape_gpu::Copias;

use super::{Celulas, Formas};
use crate::passes::{buffer, liga, vazio};
use crate::plano::Caixa;

impl Celulas {
    /// As caixas no ecrã, sprites ++ formas — dois passes e UMA leitura (à espera da própria
    /// submissão).
    pub(super) fn caixas(
        &self,
        gpu: &GpuContext,
        s_buf: &wgpu::Buffer,
        n_s: u32,
        afim: [f64; 6],
        formas: Option<Formas<'_>>,
    ) -> Vec<Caixa> {
        let n_f = formas.map_or(0, |f| f.n);
        let n = u64::from(n_s + n_f);
        if n == 0 {
            return Vec::new();
        }
        let tam = std::mem::size_of::<Caixa>() as u64;
        let uso = wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC;
        let cs = vazio(gpu, "fx_cells caixas sprites", u64::from(n_s) * tam, uso);
        let cf = vazio(gpu, "fx_cells caixas formas", u64::from(n_f) * tam, uso);
        let leitura = vazio(
            gpu,
            "fx_cells caixas leitura",
            n * tam,
            wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        );
        let mut enc = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        if n_s > 0 {
            #[expect(clippy::cast_possible_truncation, reason = "a placa lê f32")]
            let p: [f32; 8] = [
                afim[0] as f32,
                afim[1] as f32,
                afim[2] as f32,
                afim[3] as f32,
                afim[4] as f32,
                afim[5] as f32,
                f32::from_bits(n_s),
                0.0,
            ];
            let u = buffer(
                gpu,
                "fx_cells limites p",
                bytemuck::cast_slice(&p),
                wgpu::BufferUsages::UNIFORM,
            );
            let bg = liga(
                gpu,
                &self.limites.get_bind_group_layout(0),
                &[
                    u.as_entire_binding(),
                    s_buf.as_entire_binding(),
                    cs.as_entire_binding(),
                ],
            );
            let mut pass = enc.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
            pass.set_pipeline(&self.limites);
            pass.set_bind_group(0, &bg, &[]);
            pass.dispatch_workgroups(n_s.div_ceil(64), 1, 1);
        }
        if let Some(f) = formas.filter(|f| f.n > 0) {
            self.formas.caixas(
                gpu,
                &mut enc,
                f.vista,
                Copias {
                    buffer: f.buffer,
                    count: f.n,
                },
                &cf,
            );
        }
        enc.copy_buffer_to_buffer(&cs, 0, &leitura, 0, u64::from(n_s) * tam);
        enc.copy_buffer_to_buffer(&cf, 0, &leitura, u64::from(n_s) * tam, u64::from(n_f) * tam);
        let sub = gpu.queue.submit([enc.finish()]);
        let fatia = leitura.slice(..n * tam);
        fatia.map_async(wgpu::MapMode::Read, |_| {});
        let _ = gpu.device.poll(wgpu::PollType::Wait {
            submission_index: Some(sub),
            timeout: None,
        });
        let caixas = bytemuck::cast_slice::<u8, Caixa>(&fatia.get_mapped_range()).to_vec();
        leitura.unmap();
        caixas
    }
}
