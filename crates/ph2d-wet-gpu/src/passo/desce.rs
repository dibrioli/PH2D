//! As descidas por faixa de linhas do [`PassoPlaca`].

use super::PassoPlaca;
use super::hashes::fluxo_de;
use ph2d_gpu::GpuContext;
use ph2d_wet_paint::grid::Grid;

impl PassoPlaca {
    /// As faixas de linhas finas que o `desce_suja` desce inteiras: as duas de borda de cada lado e a `suja`.
    pub(super) fn faixas_suja(&self, g: &Grid) -> Vec<(usize, usize)> {
        let ult = g.rows - 1;
        let mut faixas = vec![(0, 1), (ult - 1, ult)];
        if let Some((a, b)) = self.suja {
            faixas.push(((a.max(0) as usize).min(ult), (b.max(0) as usize).min(ult)));
        }
        faixas
    }

    /// B44.9: as linhas finas que o próximo `desce_suja` pode escrever na CPU (as faixas, e as curadas na lei
    /// `Pigmento`); sem espelho, a grade inteira. Quem guarda a história marca-as ANTES de descer.
    #[must_use]
    pub fn linhas_da_descida(&self, g: &Grid) -> Vec<(usize, usize)> {
        if self.hash_fina.len() != g.rows {
            return vec![(0, g.rows - 1)];
        }
        let mut faixas = self.faixas_suja(g);
        if let (Some((a, b)), Some(_)) = (self.curada, &self.tinta) {
            faixas.push((a.min(g.rows - 1), b.min(g.rows - 1)));
        }
        faixas
    }

    /// B44.9: o próximo [`Self::passo`] cura (e pode fechar: `desce` + `cura` + `sobe` da grade INTEIRA na CPU).
    #[must_use]
    pub fn cura_neste_passo(&self, sim: &ph2d_wet_paint::sim::Sim) -> bool {
        self.tinta.is_some()
            && (sim.frame + 1).is_multiple_of(ph2d_wet_paint::drying::PASSOS_DA_CURA)
    }

    /// Desce a abertura das linhas curadas que `faixas` não trouxe, sem isto o `sobe_sujo` seguinte devolvia à placa o `o` velho.
    /// A água não tem meio e não cura: não paga nada. Devolve essas linhas.
    pub(super) fn desce_o_curado(
        &mut self,
        gpu: &GpuContext,
        g: &mut Grid,
        faixas: &[(usize, usize)],
    ) -> Vec<usize> {
        let (Some((a, b)), Some(t)) = (self.curada.take(), &self.tinta) else {
            return Vec::new();
        };
        let fora: Vec<usize> = (a..=b.min(g.rows - 1))
            .filter(|y| !faixas.iter().any(|&(p, q)| (p..=q).contains(y)))
            .collect();
        for run in fora.chunk_by(|p, q| p + 1 == *q) {
            t.desce_abertura(gpu, g, (run[0], run[run.len() - 1]));
        }
        fora
    }

    /// Desce as linhas finas de cada faixa e as de fluxo que as cobrem: as 10 cópias de cada faixa num
    /// codificador, num buffer, numa espera (antes: uma espera por cópia, 33 por descida suja).
    pub(super) fn desce_faixas(&self, gpu: &GpuContext, g: &mut Grid, faixas: &[(usize, usize)]) {
        let a = &self.adv;
        let fino: [(&wgpu::Buffer, u64); 6] = [
            (&a.st, 8),
            (&a.rgb, 12),
            (&a.sett, 4),
            (&self.seca.trgb, 12),
            (&a.active, 4),
            (&a.wet, 4),
        ];
        let fluxo = [&a.vel_x, &a.vel_y, &a.flow_x, &a.flow_y];
        // (origem, offset na origem, bytes, offset no stage)
        let mut copias: Vec<(&wgpu::Buffer, u64, u64, u64)> = Vec::new();
        let mut tam = 0u64;
        for &(y0, y1) in faixas {
            let (ca, cb) = ((y0 * g.s) as u64, ((y1 + 1) * g.s) as u64);
            for &(b, k) in &fino {
                copias.push((b, ca * k, (cb - ca) * k, tam));
                tam += (cb - ca) * k;
            }
            let (f0, f1) = fluxo_de(g, y0, y1);
            let (fa, fb) = ((f0 * g.flow.s) as u64, ((f1 + 1) * g.flow.s) as u64);
            for b in fluxo {
                copias.push((b, fa * 4, (fb - fa) * 4, tam));
                tam += (fb - fa) * 4;
            }
        }
        if tam == 0 {
            return;
        }
        let stage = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("wet-gpu desce"),
            size: tam,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut enc = gpu.device.create_command_encoder(&Default::default());
        for &(b, o, n, d) in &copias {
            enc.copy_buffer_to_buffer(b, o, &stage, d, n);
        }
        gpu.queue.submit([enc.finish()]);
        let sl = stage.slice(..);
        sl.map_async(wgpu::MapMode::Read, |r| r.expect("map"));
        let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
        {
            let d = sl.get_mapped_range().expect("mapped range");
            let mut it = copias
                .iter()
                .map(|&(_, _, n, o)| &d[o as usize..(o + n) as usize]);
            let mut prox = || it.next().expect("uma cópia por destino");
            for &(y0, y1) in faixas {
                let (ca, cb) = (y0 * g.s, (y1 + 1) * g.s);
                let st: &[[f32; 2]] = bytemuck::cast_slice(prox());
                for (k, v) in st.iter().enumerate() {
                    g.film[ca + k] = v[0];
                    g.susp[ca + k] = v[1];
                }
                g.susp_rgb[ca..cb].copy_from_slice(bytemuck::cast_slice(prox()));
                g.sett[ca..cb].copy_from_slice(bytemuck::cast_slice(prox()));
                g.sett_rgb[ca..cb].copy_from_slice(bytemuck::cast_slice(prox()));
                let act: &[u32] = bytemuck::cast_slice(prox());
                for (o, &v) in g.active[ca..cb].iter_mut().zip(act) {
                    *o = v as u8;
                }
                let wet: &[u32] = bytemuck::cast_slice(prox());
                for (o, &v) in g.wet[ca..cb].iter_mut().zip(wet) {
                    *o = v as u8;
                }
                let (f0, f1) = fluxo_de(g, y0, y1);
                let (fa, fb) = (f0 * g.flow.s, (f1 + 1) * g.flow.s);
                g.vel_x[fa..fb].copy_from_slice(bytemuck::cast_slice(prox()));
                g.vel_y[fa..fb].copy_from_slice(bytemuck::cast_slice(prox()));
                g.flow_x[fa..fb].copy_from_slice(bytemuck::cast_slice(prox()));
                g.flow_y[fa..fb].copy_from_slice(bytemuck::cast_slice(prox()));
            }
        }
        stage.unmap();
    }
}
