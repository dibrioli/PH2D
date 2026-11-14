//! A secagem na placa (degrau 5): `drying_pass_jacobi_rows` da lei Cobre em f32 (`seca.wgsl`).
//! A CPU continua canónica; desenho e tolerâncias em docs/Painter/49_realshader/PLACA_degrau5.md.

use bytemuck::{Pod, Zeroable};
use ph2d_gpu::GpuContext;
use ph2d_wet_paint::assenta::Ecra;
use ph2d_wet_paint::grid::{Grid, aparencia, oleo_celula, span_x_of};
use ph2d_wet_paint::sim::Params;
use ph2d_wet_paint::tuning::Knob;

/// O `struct S` do WGSL (20 × 4 B).
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct SecaU {
    pub retention: f32,
    pub edge_dark: f32,
    pub base_evap: f32,
    pub ext_gran: f32,
    pub ext_stain: f32,
    pub evap_base: f32,
    pub rewet_base: f32,
    pub bypass: u32,
    pub filme: f32,
    pub esconde: f32,
    pub vela: f32,
    pub oleo: f32,
    pub s: u32,
    pub by0: u32,
    pub by1: u32,
    pub bx0: u32,
    pub bx1: u32,
    /// `1` = `entre_perfis` pelo produto vectorial (Lagrange); `0` = a conta literal.
    pub cross: u32,
    /// O par do tile calmo (B13.4, o `modo` 1); o `evap_base`/`rewet_base` é o do chamador (o `modo` 2).
    pub evap_calmo: f32,
    pub rewet_calmo: f32,
}

impl SecaU {
    /// Os mesmos números do `DryConst::new` da CPU. Só a lei `Cobre` está portada.
    #[must_use]
    pub fn new(g: &Grid, p: &Params, evap_base: f64, rewet_base: f64, bypass: bool) -> Self {
        let Ecra::Cobre {
            filme,
            esconde,
            vela,
            oleo,
            ..
        } = p.ecra
        else {
            panic!(
                "a secagem na placa só tem a lei Cobre (ecra = {:?})",
                p.ecra
            );
        };
        SecaU {
            // A fuga escalada pela Absorbency da tela, a mesma da CPU (`DryConst::new`, E10d2).
            retention: p.retencao() as f32,
            // O fundo da grade; a borda própria de cada célula vai no `aux.w`.
            edge_dark: if g.borda_fundo < 0.0 {
                p.k(Knob::EdgeDarkening) as f32
            } else {
                g.borda_fundo as f32
            },
            base_evap: ph2d_wet_paint::drying::EVAPORACAO_BASE as f32,
            ext_gran: p.k(Knob::ExtGranulation) as f32,
            ext_stain: p.k(Knob::ExtStaining) as f32,
            evap_base: evap_base as f32,
            rewet_base: rewet_base as f32,
            bypass: u32::from(bypass),
            filme: filme as f32,
            esconde: esconde as f32,
            vela: vela as f32,
            oleo: oleo as f32,
            s: g.s as u32,
            by0: g.by0.max(0) as u32,
            by1: g.by1.max(0) as u32,
            bx0: g.bx0.max(0) as u32,
            bx1: g.bx1.max(0) as u32,
            cross: 1,
            evap_calmo: evap_base as f32,
            rewet_calmo: rewet_base as f32,
        }
    }
}

/// Uma passada da secagem: o pipeline e o seu grupo.
pub(crate) type Passada = (wgpu::ComputePipeline, wgpu::BindGroup);

pub struct SecaGpu {
    pub ub: wgpu::Buffer,
    pub film: wgpu::Buffer,
    pub susp: wgpu::Buffer,
    pub sett: wgpu::Buffer,
    pub srgb: wgpu::Buffer,
    pub trgb: wgpu::Buffer,
    paper: wgpu::Buffer,
    aux: wgpu::Buffer,
    edge: wgpu::Buffer,
    alfa: wgpu::Buffer,
    faixa: wgpu::Buffer,
    /// A cadência por tile 64×64 (`seca.wgsl` binding 15): 0 salta · 1 calmo · 2 o par do chamador.
    modo: wgpu::Buffer,
    modo_atual: Vec<u32>,
    pipes: [(wgpu::ComputePipeline, wgpu::BindGroup); 2],
    u: SecaU,
    cells: usize,
}

fn buf(gpu: &GpuContext, label: &str, bytes: u64) -> wgpu::Buffer {
    use wgpu::BufferUsages as B;
    gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size: bytes.max(16),
        usage: B::STORAGE | B::COPY_DST | B::COPY_SRC,
        mapped_at_creation: false,
    })
}

impl SecaGpu {
    #[must_use]
    pub fn new(gpu: &GpuContext, g: &Grid, u: SecaU) -> Self {
        let n = g.cells as u64;
        let ub = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("seca U"),
            size: std::mem::size_of::<SecaU>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let film = buf(gpu, "seca film", n * 4);
        let susp = buf(gpu, "seca susp", n * 4);
        let sett = buf(gpu, "seca sett", n * 4);
        let srgb = buf(gpu, "seca srgb", n * 12);
        let trgb = buf(gpu, "seca trgb", n * 12);
        let paper = buf(gpu, "seca paper", n * 4);
        let aux = buf(gpu, "seca aux", n * 16);
        let edge = buf(gpu, "seca edge", n * 4);
        let alfa = buf(gpu, "seca alfa", 3001 * 4);
        let faixa = buf(gpu, "seca faixa", g.rows as u64 * 8);
        let (tx, ty) = ph2d_wet_paint::drying::por_tile::tiles_da_grade(g);
        let modo = buf(gpu, "seca modo", (tx * ty) as u64 * 4);
        let modo_atual = vec![2u32; tx * ty];
        gpu.queue
            .write_buffer(&modo, 0, bytemuck::cast_slice(&modo_atual));
        let tab: Vec<f32> = (0..=3000)
            .map(|i| ph2d_wet_paint::opacity::table_at(i) as f32)
            .collect();
        gpu.queue.write_buffer(&alfa, 0, bytemuck::cast_slice(&tab));
        let module = gpu
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("seca.wgsl"),
                source: wgpu::ShaderSource::Wgsl(include_str!("seca.wgsl").into()),
            });
        let all: [(u32, &wgpu::Buffer); 12] = [
            (0, &ub),
            (1, &film),
            (2, &susp),
            (3, &sett),
            (4, &srgb),
            (5, &trgb),
            (6, &paper),
            (7, &aux),
            (8, &edge),
            (9, &alfa),
            (10, &faixa),
            (15, &modo),
        ];
        let mk = |entry: &str, usa: &[u32]| {
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
            let entries: Vec<wgpu::BindGroupEntry> = all
                .iter()
                .filter(|(b, _)| usa.contains(b))
                .map(|(b, bf)| wgpu::BindGroupEntry {
                    binding: *b,
                    resource: bf.as_entire_binding(),
                })
                .collect();
            let bg = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(entry),
                layout: &pipe.get_bind_group_layout(0),
                entries: &entries,
            });
            (pipe, bg)
        };
        let pipes = [
            mk("borda", &[0, 2, 8, 10]),
            mk("seca", &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 15]),
        ];
        let s = SecaGpu {
            ub,
            film,
            susp,
            sett,
            srgb,
            trgb,
            paper,
            aux,
            edge,
            alfa,
            faixa,
            modo,
            modo_atual,
            pipes,
            u,
            cells: g.cells,
        };
        s.upload(gpu, g);
        s
    }

    /// A passada de secagem passa a ser a da lei `Pigmento` (`seca_tinta.wgsl`); a `borda` é a mesma.
    pub(crate) fn liga_tinta(&mut self, gpu: &GpuContext, t: &crate::tinta::TintaGpu) {
        let module = gpu
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("seca.wgsl + tinta"),
                source: wgpu::ShaderSource::Wgsl(crate::tinta::wgsl_seca().into()),
            });
        let pipe = gpu
            .device
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("seca_tinta"),
                layout: None,
                module: &module,
                entry_point: Some("seca_tinta"),
                compilation_options: Default::default(),
                cache: None,
            });
        let buffers: [(u32, &wgpu::Buffer); 16] = [
            (0, &self.ub),
            (1, &self.film),
            (2, &self.susp),
            (3, &self.sett),
            (4, &self.srgb),
            (5, &self.trgb),
            (6, &self.paper),
            (7, &self.aux),
            (8, &self.edge),
            (9, &self.alfa),
            (10, &self.faixa),
            (11, &t.filme),
            (12, &t.aberto),
            (13, &t.abertura),
            (14, &t.meio),
            (15, &self.modo),
        ];
        let entries: Vec<wgpu::BindGroupEntry> = buffers
            .iter()
            .map(|(b, bf)| wgpu::BindGroupEntry {
                binding: *b,
                resource: bf.as_entire_binding(),
            })
            .collect();
        let bg = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("seca_tinta"),
            layout: &pipe.get_bind_group_layout(0),
            entries: &entries,
        });
        self.pipes[1] = (pipe, bg);
    }

    /// Sobe o estado, o papel, a vela/vidrado/óleo por célula, as faixas e os uniformes.
    pub fn upload(&self, gpu: &GpuContext, g: &Grid) {
        let q = &gpu.queue;
        q.write_buffer(&self.ub, 0, bytemuck::bytes_of(&self.u));
        q.write_buffer(&self.film, 0, bytemuck::cast_slice(&g.film));
        q.write_buffer(&self.susp, 0, bytemuck::cast_slice(&g.susp));
        q.write_buffer(&self.sett, 0, bytemuck::cast_slice(&g.sett));
        q.write_buffer(&self.srgb, 0, bytemuck::cast_slice(&g.susp_rgb));
        q.write_buffer(&self.trgb, 0, bytemuck::cast_slice(&g.sett_rgb));
        q.write_buffer(&self.paper, 0, bytemuck::cast_slice(&g.paper));
        let aux: Vec<[f32; 4]> = (0..g.cells)
            .map(|i| {
                [
                    g.vela.get(i).copied().unwrap_or(0.0),
                    aparencia::vidrado(&g.aparencia, i),
                    oleo_celula::da_celula(&g.oleo, i).oleo,
                    aparencia::borda(&g.aparencia, i),
                ]
            })
            .collect();
        q.write_buffer(&self.aux, 0, bytemuck::cast_slice(&aux));
        let fx: Vec<[i32; 2]> = (0..g.rows as i32)
            .map(|y| {
                let (lo, hi) = span_x_of(&g.row_lo, &g.row_hi, g.spans_enabled, g.bx0, g.bx1, y);
                [lo, hi]
            })
            .collect();
        q.write_buffer(&self.faixa, 0, bytemuck::cast_slice(&fx));
    }

    /// Sobe só as linhas finas `y0..=y1` (o que a CPU mudou desde a última descida, [`crate::passo`]).
    pub fn upload_linhas(&self, gpu: &GpuContext, g: &Grid, y0: usize, y1: usize) {
        let q = &gpu.queue;
        let (a, b) = (y0 * g.s, (y1 + 1) * g.s);
        let w = |buf: &wgpu::Buffer, d: &[u8], k: usize| q.write_buffer(buf, (a * k) as u64, d);
        w(&self.film, bytemuck::cast_slice(&g.film[a..b]), 4);
        w(&self.susp, bytemuck::cast_slice(&g.susp[a..b]), 4);
        w(&self.sett, bytemuck::cast_slice(&g.sett[a..b]), 4);
        w(&self.srgb, bytemuck::cast_slice(&g.susp_rgb[a..b]), 12);
        w(&self.trgb, bytemuck::cast_slice(&g.sett_rgb[a..b]), 12);
        w(&self.paper, bytemuck::cast_slice(&g.paper[a..b]), 4);
        let aux: Vec<[f32; 4]> = (a..b)
            .map(|i| {
                [
                    g.vela.get(i).copied().unwrap_or(0.0),
                    aparencia::vidrado(&g.aparencia, i),
                    oleo_celula::da_celula(&g.oleo, i).oleo,
                    aparencia::borda(&g.aparencia, i),
                ]
            })
            .collect();
        w(&self.aux, bytemuck::cast_slice(&aux), 16);
    }

    /// A cadência por tile do passe (B13.4): `None` = todo tile com o par do chamador (o de antes, ao bit); senão o
    /// tile a correr usa o do chamador, o calmo o seu par quando `calmo_seca`, e salta nos outros passes.
    /// Devolve a `u` com o par calmo; só sobe o mapa quando muda.
    pub fn set_modo(
        &mut self,
        gpu: &GpuContext,
        mut u: SecaU,
        t: Option<&ph2d_wet_paint::drying::SecaPorTile<'_>>,
    ) -> SecaU {
        let novo: Vec<u32> = match t {
            Some(t) if t.corre.len() == self.modo_atual.len() => {
                u.evap_calmo = t.evap_calmo as f32;
                u.rewet_calmo = t.rewet_calmo as f32;
                let calmo = u32::from(t.calmo_seca);
                t.corre.iter().map(|&c| if c { 2 } else { calmo }).collect()
            }
            _ => vec![2; self.modo_atual.len()],
        };
        if novo != self.modo_atual {
            gpu.queue
                .write_buffer(&self.modo, 0, bytemuck::cast_slice(&novo));
            self.modo_atual = novo;
        }
        u
    }

    pub fn set_u(&mut self, gpu: &GpuContext, u: SecaU) {
        self.u = u;
        gpu.queue.write_buffer(&self.ub, 0, bytemuck::bytes_of(&u));
    }

    /// As faixas = a caixa inteira em cada linha (o `span_x_of` com faixas desligadas).
    pub fn faixas_da_caixa(&self, gpu: &GpuContext, g: &Grid) {
        let fx: Vec<[i32; 2]> = (0..g.rows as i32)
            .map(|y| {
                let (lo, hi) = span_x_of(&g.row_lo, &g.row_hi, false, g.bx0, g.bx1, y);
                [lo, hi]
            })
            .collect();
        gpu.queue
            .write_buffer(&self.faixa, 0, bytemuck::cast_slice(&fx));
    }

    /// Grava `n` secagens (borda → seca) num codificador.
    pub fn encode(&self, enc: &mut wgpu::CommandEncoder, n: u32) {
        if self.u.by0 > self.u.by1 || self.u.bx0 > self.u.bx1 {
            return;
        }
        let gx = (self.u.bx1 - self.u.bx0 + 1).div_ceil(16);
        let gy = (self.u.by1 - self.u.by0 + 1).div_ceil(16);
        for _ in 0..n {
            for (p, bg) in &self.pipes {
                let mut cp = enc.begin_compute_pass(&Default::default());
                cp.set_pipeline(p);
                cp.set_bind_group(0, bg, &[]);
                cp.dispatch_workgroups(gx, gy, 1);
            }
        }
    }

    /// As duas passadas (borda → seca) e os grupos sobre a caixa, para quem fatia as submissões
    /// ([`crate::passo`]); `None` = caixa vazia (o [`Self::encode`] também não grava nada).
    pub(crate) fn passadas(&self) -> Option<(&[Passada; 2], u32, u32)> {
        if self.u.by0 > self.u.by1 || self.u.bx0 > self.u.bx1 {
            return None;
        }
        let gx = (self.u.bx1 - self.u.bx0 + 1).div_ceil(16);
        let gy = (self.u.by1 - self.u.by0 + 1).div_ceil(16);
        Some((&self.pipes, gx, gy))
    }

    pub fn step(&self, gpu: &GpuContext, n: u32) {
        let mut enc = gpu.device.create_command_encoder(&Default::default());
        self.encode(&mut enc, n);
        gpu.queue.submit([enc.finish()]);
    }

    /// Lê film, susp, sett, susp_rgb, sett_rgb de volta para `g`.
    pub fn readback(&self, gpu: &GpuContext, g: &mut Grid) {
        let n = self.cells as u64;
        g.film = crate::read(gpu, &self.film, n * 4);
        g.susp = crate::read(gpu, &self.susp, n * 4);
        g.sett = crate::read(gpu, &self.sett, n * 4);
        g.susp_rgb = crate::read(gpu, &self.srgb, n * 12);
        g.sett_rgb = crate::read(gpu, &self.trgb, n * 12);
    }
}
