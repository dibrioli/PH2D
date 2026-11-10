//! A lei `Pigmento` na placa (E11, F4_2 §6): as somas do filme e do aberto, a abertura, o meio e o corpo da célula. Só
//! existe com a lei nova: a velha não paga um byte nem muda uma passada. `r_g`, `r_g0` e `altura_fechada` ficam na CPU (só o
//! fecho os escreve, e o fecho é dela); a composição também é da CPU, logo a aparência (`suave` incluído) não sobe.

use ph2d_gpu::GpuContext;
use ph2d_paint_tinta::Soma;
use ph2d_wet_paint::drying::PASSOS_DA_CURA;
use ph2d_wet_paint::drying::tinta::{O_FECHA, TAU_OLEO_S};
use ph2d_wet_paint::grid::Grid;
use ph2d_wet_paint::grid::oleo_celula::da_celula;
use ph2d_wet_paint::grid::tinta::{CelulaTinta, garante, migra};

/// As constantes das leis nos shaders, geradas da CPU: uma resposta só.
#[must_use]
pub(crate) fn cabecalho() -> String {
    use ph2d_wet_paint::meio::{ENDURECE, ETA_OLEO};
    use ph2d_wet_paint::painter::receita::K_W;
    use ph2d_wet_paint::solver::{PHI_C, TAU_CORPO, TAU0};
    format!(
        "const K_W: f32 = {:?};\nconst ETA_OLEO: f32 = {:?};\nconst ENDURECE: f32 = {:?};\nconst TAU0: f32 = {:?};\n\
         const PHI_C: f32 = {:?};\nconst TAU_CORPO: f32 = {:?};\nconst TAU_OLEO_S: f32 = {:?};\nconst O_FECHA: f32 = {:?};\n\
         const DT_CURA: f32 = {:?};\n",
        K_W as f32,
        ETA_OLEO as f32,
        ENDURECE as f32,
        TAU0 as f32,
        PHI_C as f32,
        TAU_CORPO as f32,
        TAU_OLEO_S,
        O_FECHA,
        PASSOS_DA_CURA as f32 / 40.0,
    )
}

/// `advect.wgsl` + a lei nova.
#[must_use]
pub(crate) fn wgsl_advect() -> String {
    format!(
        "{}{}{}",
        cabecalho(),
        crate::WGSL,
        include_str!("advect_tinta.wgsl")
    )
}

/// `seca.wgsl` + a lei nova.
#[must_use]
pub(crate) fn wgsl_seca() -> String {
    format!(
        "{}{}{}",
        cabecalho(),
        include_str!("seca.wgsl"),
        include_str!("seca_tinta.wgsl")
    )
}

/// Os buffers da lei nova, por célula.
pub(crate) struct TintaGpu {
    pub filme: wgpu::Buffer,
    pub dfilme: wgpu::Buffer,
    pub aberto: wgpu::Buffer,
    pub abertura: wgpu::Buffer,
    pub meio: wgpu::Buffer,
    pub dmeio: wgpu::Buffer,
    pub corpo: wgpu::Buffer,
    /// `[células que pediram o fecho, 1.ª linha curada, última linha curada]` da última cura; vazio = `[0, MAX, 0]`.
    pub fecham: wgpu::Buffer,
}

/// O valor de partida de [`TintaGpu::fecham`] antes de cada cura.
pub(crate) const FECHAM_ZERO: [u32; 3] = [0, u32::MAX, 0];

fn seis(s: &Soma) -> [f32; 6] {
    [s.v, s.lab[0], s.lab[1], s.lab[2], s.t, s.m]
}

fn soma(v: &[f32]) -> Soma {
    Soma {
        v: v[0],
        lab: [v[1], v[2], v[3]],
        t: v[4],
        m: v[5],
    }
}

/// O que a placa espelha de uma linha, para o hash ([`crate::passo`]).
pub(crate) fn plano_do_hash(g: &Grid, a: usize, b: usize) -> Vec<f32> {
    let mut v = Vec::new();
    if !g.tinta.is_empty() {
        v.reserve((b - a) * 13);
        for c in &g.tinta[a..b] {
            v.extend(seis(&c.filme));
            v.extend(seis(&c.aberto));
            v.push(c.o);
        }
    }
    if !g.oleo.is_empty() {
        v.extend((a..b).flat_map(|i| {
            let c = g.oleo[i];
            [c.meio, c.corpo]
        }));
    }
    v
}

impl TintaGpu {
    #[must_use]
    pub(crate) fn new(gpu: &GpuContext, g: &Grid) -> Self {
        use wgpu::BufferUsages as B;
        let n = g.cells as u64;
        let buf = |label: &str, bytes: u64| {
            gpu.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                size: bytes.max(16),
                usage: B::STORAGE | B::COPY_DST | B::COPY_SRC,
                mapped_at_creation: false,
            })
        };
        TintaGpu {
            filme: buf("tinta filme", n * 24),
            dfilme: buf("tinta dfilme", n * 24),
            aberto: buf("tinta aberto", n * 24),
            abertura: buf("tinta abertura", n * 4),
            meio: buf("tinta meio", n * 4),
            dmeio: buf("tinta dmeio", n * 4),
            corpo: buf("tinta corpo", n * 4),
            fecham: buf("tinta fecham", 16),
        }
    }

    /// Sobe as células `a..b`; a grade ainda sem o campo sobe migrada, como o início do passo da CPU a migraria.
    pub(crate) fn sobe(&self, gpu: &GpuContext, g: &Grid, a: usize, b: usize) {
        let n = b - a;
        let (mut fi, mut ab, mut op) = (
            Vec::with_capacity(n * 6),
            Vec::with_capacity(n * 6),
            Vec::with_capacity(n),
        );
        let (mut me, mut co) = (Vec::with_capacity(n), Vec::with_capacity(n));
        for i in a..b {
            let c: CelulaTinta = g.tinta.get(i).copied().unwrap_or_else(|| migra(g, i));
            fi.extend(seis(&c.filme));
            ab.extend(seis(&c.aberto));
            op.push(c.o);
            let o = da_celula(&g.oleo, i);
            me.push(o.meio);
            co.push(o.corpo);
        }
        let q = &gpu.queue;
        let w = |buf: &wgpu::Buffer, d: &[f32], k: usize| {
            q.write_buffer(buf, (a * k) as u64, bytemuck::cast_slice(d));
        };
        w(&self.filme, &fi, 24);
        w(&self.aberto, &ab, 24);
        w(&self.abertura, &op, 4);
        w(&self.meio, &me, 4);
        w(&self.corpo, &co, 4);
    }

    /// Desce SÓ a abertura das linhas `y0..=y1`: a cura escreve-a fora da caixa, onde mais nada mudou.
    pub(crate) fn desce_abertura(&self, gpu: &GpuContext, g: &mut Grid, (y0, y1): (usize, usize)) {
        let (a, n) = (y0 * g.s, (y1 + 1 - y0) * g.s);
        let o: Vec<f32> = crate::read_at(gpu, &self.abertura, a as u64 * 4, n as u64 * 4);
        garante(g);
        for (c, o) in g.tinta[a..a + n].iter_mut().zip(o) {
            c.o = o;
        }
    }

    /// Desce as células de cada faixa de linhas, numa cópia e numa espera. O corpo e o resto da célula a placa não muda.
    pub(crate) fn desce_faixas(&self, gpu: &GpuContext, g: &mut Grid, faixas: &[(usize, usize)]) {
        let fontes: [(&wgpu::Buffer, u64); 4] = [
            (&self.filme, 24),
            (&self.aberto, 24),
            (&self.abertura, 4),
            (&self.meio, 4),
        ];
        let mut copias = Vec::new();
        let mut tam = 0u64;
        for &(y0, y1) in faixas {
            let (ca, cb) = ((y0 * g.s) as u64, ((y1 + 1) * g.s) as u64);
            for &(b, k) in &fontes {
                copias.push((b, ca * k, (cb - ca) * k, tam));
                tam += (cb - ca) * k;
            }
        }
        if tam == 0 {
            return;
        }
        let stage = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("tinta desce"),
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
        garante(g);
        {
            let d = sl.get_mapped_range().expect("mapped range");
            let mut it = copias.iter().map(|&(_, _, n, o)| -> &[f32] {
                bytemuck::cast_slice(&d[o as usize..(o + n) as usize])
            });
            let mut prox = || it.next().expect("uma cópia por destino");
            for &(y0, y1) in faixas {
                let (ca, cb) = (y0 * g.s, (y1 + 1) * g.s);
                let (fi, ab, op, me) = (prox(), prox(), prox(), prox());
                for i in ca..cb {
                    let k = i - ca;
                    let c = &mut g.tinta[i];
                    c.filme = soma(&fi[k * 6..k * 6 + 6]);
                    c.aberto = soma(&ab[k * 6..k * 6 + 6]);
                    c.o = op[k];
                    // Só o que mudou ao bit: o pedaço do chão não acorda (B12.3).
                    if g.oleo
                        .get(i)
                        .is_some_and(|o| o.meio.to_bits() != me[k].to_bits())
                    {
                        g.oleo[i].meio = me[k];
                    }
                }
            }
        }
        stage.unmap();
    }
}

impl crate::AdvectGpu {
    /// Acrescenta as passadas da lei `Pigmento`, nos índices [`T_TRANSPORTE`]..=[`T_CURA`].
    pub(crate) fn liga_tinta(&mut self, gpu: &GpuContext, t: &TintaGpu) {
        let module = gpu
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("wet-gpu advect + tinta"),
                source: wgpu::ShaderSource::Wgsl(wgsl_advect().into()),
            });
        let (uv, outflow) = (&self._scratch[0], &self._scratch[1]);
        let buffers: [(u32, &wgpu::Buffer); 21] = [
            (0, &self.ub),
            (1, &self.flow_x),
            (2, &self.flow_y),
            (3, &self.active),
            (4, uv),
            (5, outflow),
            (6, &self.st),
            (10, &self.vel_x),
            (11, &self.vel_y),
            (12, &self.paper),
            (13, &self.wet),
            (14, &self.sett),
            (15, &self.kb),
            (18, &t.filme),
            (19, &t.dfilme),
            (20, &t.meio),
            (21, &t.dmeio),
            (22, &t.corpo),
            (23, &t.aberto),
            (24, &t.abertura),
            (25, &t.fecham),
        ];
        let passadas: [(&str, &[u32]); 6] = [
            ("t_transporte", &[0, 3, 4, 5, 6, 18, 19, 20, 21]),
            ("t_commit", &[0, 18, 19, 20, 21]),
            ("t_sedimenta", &[0, 6, 15, 18, 19]),
            ("t_bordas", &[0, 18]),
            (
                "campo_oleo",
                &[0, 1, 2, 3, 6, 10, 11, 12, 13, 14, 15, 20, 22],
            ),
            ("t_cura", &[0, 6, 18, 23, 24, 25]),
        ];
        for (entry, usa) in passadas {
            let pipe = gpu
                .device
                .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                    label: Some(entry),
                    layout: None,
                    module: &module,
                    entry_point: Some(entry),
                    compilation_options: Default::default(),
                    cache: None,
                });
            let entries: Vec<wgpu::BindGroupEntry> = buffers
                .iter()
                .filter(|(b, _)| usa.contains(b))
                .map(|(b, buf)| wgpu::BindGroupEntry {
                    binding: *b,
                    resource: buf.as_entire_binding(),
                })
                .collect();
            let bg = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(entry),
                layout: &pipe.get_bind_group_layout(0),
                entries: &entries,
            });
            self.pipes.push(pipe);
            self.groups.push(bg);
        }
    }
}
