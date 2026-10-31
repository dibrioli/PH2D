//! **A advecção do Wet Paint na placa** — 1.º degrau da água na GPU.
//!
//! A mesma lei de [`ph2d_wet_paint::solver::advect_jacobi`] (ADR-0147: a forma Jacobi/gather,
//! prepare → outflow → transport → commit) em quatro compute shaders WGSL, f32. A CPU é a
//! CANÓNICA (ADR-0146): isto é desempenho, reconciliado por ε medido **por placa**
//! (`tests/paridade.rs`). Desenho e números: `docs/Painter/49_realshader/PLACA_degrau1.md`.
//!
//! Fora deste degrau: o `momentum_rows` (escreve `vel`, que a advecção não lê), a mistura K–M
//! (`km_mixing`; aqui só a média linear) e as faixas vivas por linha (aqui um retângulo).

use bytemuck::{Pod, Zeroable};
use ph2d_gpu::GpuContext;
use ph2d_wet_paint::grid::Grid;

/// O shader — público para o exemplo de medida e para a validação naga dos testes.
pub const WGSL: &str = include_str!("advect.wgsl");

/// Os uniformes do shader; o layout casa com `struct U` do WGSL (16 × 4 B).
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct Uniforms {
    pub w: i32,
    pub h: i32,
    pub s: i32,
    pub reach: i32,
    pub x0: i32,
    pub y0: i32,
    pub x1: i32,
    pub y1: i32,
    pub fs: i32,
    pub fw: i32,
    pub fh: i32,
    pub rf: i32,
    pub half: i32,
    pub inv_rf: f32,
    pub _p0: f32,
    pub _p1: f32,
}

/// Retângulo inclusivo de células (`1..=w`, `1..=h`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rect {
    pub x0: i32,
    pub y0: i32,
    pub x1: i32,
    pub y1: i32,
}

impl Rect {
    #[must_use]
    pub fn full(g: &Grid) -> Self {
        Rect {
            x0: 1,
            y0: 1,
            x1: g.w as i32,
            y1: g.h as i32,
        }
    }
    #[must_use]
    pub fn cols(&self) -> u32 {
        (self.x1 - self.x0 + 1).max(0) as u32
    }
    #[must_use]
    pub fn rows(&self) -> u32 {
        (self.y1 - self.y0 + 1).max(0) as u32
    }
}

impl Uniforms {
    /// Os uniformes de `g` para `max_velocity` (o knob) sobre o retângulo `r`.
    #[must_use]
    pub fn from_grid(g: &Grid, max_velocity: f64, r: Rect) -> Self {
        let f = &g.flow;
        Uniforms {
            w: g.w as i32,
            h: g.h as i32,
            s: g.s as i32,
            reach: ph2d_wet_paint::solver::destination_reach(max_velocity),
            x0: r.x0,
            y0: r.y0,
            x1: r.x1,
            y1: r.y1,
            fs: f.s as i32,
            fw: f.w as i32,
            fh: f.h as i32,
            rf: f.rf as i32,
            half: ((f.rf - 1) / 2) as i32,
            inv_rf: f.inv_rf as f32,
            _p0: 0.0,
            _p1: 0.0,
        }
    }
}

/// `film`/`susp` entrelaçados — o `vec2<f32>` do shader.
#[must_use]
pub fn pack_state(film: &[f32], susp: &[f32]) -> Vec<[f32; 2]> {
    film.iter().zip(susp).map(|(&f, &m)| [f, m]).collect()
}

/// O inverso de [`pack_state`].
#[must_use]
pub fn unpack_state(st: &[[f32; 2]]) -> (Vec<f32>, Vec<f32>) {
    st.iter().map(|p| (p[0], p[1])).unzip()
}

/// A máscara `u8` como `u32` (o WGSL não tem `u8` em storage).
#[must_use]
pub fn pack_active(a: &[u8]) -> Vec<u32> {
    a.iter().map(|&v| u32::from(v)).collect()
}

/// O estado que a advecção escreve, lido de volta.
#[derive(Clone, Debug, PartialEq)]
pub struct Estado {
    pub film: Vec<f32>,
    pub susp: Vec<f32>,
    pub rgb: Vec<[f32; 3]>,
}

/// As quatro passadas, na ordem. O índice é o da pipeline em [`AdvectGpu::pipes`].
pub const PASSADAS: [&str; 4] = ["prepare", "outflow_pass", "transport", "commit"];

/// Os buffers e as pipelines de uma grade.
pub struct AdvectGpu {
    pub cells: usize,
    pub ub: wgpu::Buffer,
    pub flow_x: wgpu::Buffer,
    pub flow_y: wgpu::Buffer,
    pub active: wgpu::Buffer,
    pub st: wgpu::Buffer,
    pub rgb: wgpu::Buffer,
    pub pipes: Vec<wgpu::ComputePipeline>,
    pub groups: Vec<wgpu::BindGroup>,
    _scratch: Vec<wgpu::Buffer>,
}

fn storage(gpu: &GpuContext, label: &str, bytes: u64) -> wgpu::Buffer {
    use wgpu::BufferUsages as B;
    gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size: bytes.max(16),
        usage: B::STORAGE | B::COPY_DST | B::COPY_SRC,
        mapped_at_creation: false,
    })
}

impl AdvectGpu {
    /// Aloca os buffers à medida de `g` e compila as quatro pipelines.
    #[must_use]
    pub fn new(gpu: &GpuContext, g: &Grid) -> Self {
        let n = g.cells as u64;
        let fnc = g.flow.cells as u64;
        let ub = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("wet-gpu U"),
            size: std::mem::size_of::<Uniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let flow_x = storage(gpu, "wet-gpu flow_x", fnc * 4);
        let flow_y = storage(gpu, "wet-gpu flow_y", fnc * 4);
        let active = storage(gpu, "wet-gpu active", n * 4);
        let uv = storage(gpu, "wet-gpu uv", n * 8);
        let outflow = storage(gpu, "wet-gpu outflow", n * 4);
        let st = storage(gpu, "wet-gpu st", n * 8);
        let rgb = storage(gpu, "wet-gpu rgb", n * 12);
        let dst = storage(gpu, "wet-gpu dst", n * 8);
        let drgb = storage(gpu, "wet-gpu drgb", n * 12);
        let module = gpu
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("wet-gpu advect"),
                source: wgpu::ShaderSource::Wgsl(WGSL.into()),
            });
        let all: [(u32, &wgpu::Buffer); 10] = [
            (0, &ub),
            (1, &flow_x),
            (2, &flow_y),
            (3, &active),
            (4, &uv),
            (5, &outflow),
            (6, &st),
            (7, &rgb),
            (8, &dst),
            (9, &drgb),
        ];
        // Os bindings que cada passada usa (o layout automático só inclui esses).
        let usa: [&[u32]; 4] = [
            &[0, 1, 2, 4],
            &[0, 3, 4, 5],
            &[0, 3, 4, 5, 6, 7, 8, 9],
            &[0, 6, 7, 8, 9],
        ];
        let mut pipes = Vec::new();
        let mut groups = Vec::new();
        for (k, entry) in PASSADAS.iter().enumerate() {
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
                .filter(|(b, _)| usa[k].contains(b))
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
            pipes.push(pipe);
            groups.push(bg);
        }
        AdvectGpu {
            cells: g.cells,
            ub,
            flow_x,
            flow_y,
            active,
            st,
            rgb,
            pipes,
            groups,
            _scratch: vec![uv, outflow, dst, drgb],
        }
    }

    /// Sobe a grade inteira: os planos que a advecção lê e os uniformes.
    pub fn upload(&self, gpu: &GpuContext, g: &Grid, u: &Uniforms) {
        let q = &gpu.queue;
        q.write_buffer(&self.ub, 0, bytemuck::bytes_of(u));
        q.write_buffer(&self.flow_x, 0, bytemuck::cast_slice(&g.flow_x));
        q.write_buffer(&self.flow_y, 0, bytemuck::cast_slice(&g.flow_y));
        q.write_buffer(
            &self.active,
            0,
            bytemuck::cast_slice(&pack_active(&g.active)),
        );
        q.write_buffer(
            &self.st,
            0,
            bytemuck::cast_slice(&pack_state(&g.film, &g.susp)),
        );
        q.write_buffer(&self.rgb, 0, bytemuck::cast_slice(&g.susp_rgb));
    }

    /// Só os uniformes (troca de retângulo).
    pub fn set_uniforms(&self, gpu: &GpuContext, u: &Uniforms) {
        gpu.queue.write_buffer(&self.ub, 0, bytemuck::bytes_of(u));
    }

    /// Grava `n` passos no `enc`, um compute pass por passada; `ts(passo, passada)` dá os
    /// índices de timestamp (ou `None`).
    pub fn encode(
        &self,
        enc: &mut wgpu::CommandEncoder,
        r: Rect,
        n: u32,
        ts: Option<(&wgpu::QuerySet, &dyn Fn(u32, u32) -> u32)>,
    ) {
        let (gx, gy) = (r.cols().div_ceil(16), r.rows().div_ceil(16));
        for step in 0..n {
            for (k, (pipe, bg)) in self.pipes.iter().zip(&self.groups).enumerate() {
                let tw = ts.map(|(qs, f)| {
                    let i = f(step, k as u32);
                    wgpu::ComputePassTimestampWrites {
                        query_set: qs,
                        beginning_of_pass_write_index: Some(i),
                        end_of_pass_write_index: Some(i + 1),
                    }
                });
                let mut p = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some(PASSADAS[k]),
                    timestamp_writes: tw,
                });
                p.set_pipeline(pipe);
                p.set_bind_group(0, bg, &[]);
                p.dispatch_workgroups(gx, gy, 1);
            }
        }
    }

    /// `n` passos sobre `r`, esperando a placa.
    pub fn step(&self, gpu: &GpuContext, r: Rect, n: u32) {
        let mut enc = gpu.device.create_command_encoder(&Default::default());
        self.encode(&mut enc, r, n, None);
        gpu.queue.submit([enc.finish()]);
        let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
    }

    /// Lê `film`, `susp` e `susp_rgb` de volta (a grade inteira).
    #[must_use]
    pub fn readback(&self, gpu: &GpuContext) -> Estado {
        let n = self.cells as u64;
        let st: Vec<[f32; 2]> = read(gpu, &self.st, n * 8);
        let rgb: Vec<[f32; 3]> = read(gpu, &self.rgb, n * 12);
        let (film, susp) = unpack_state(&st);
        Estado { film, susp, rgb }
    }
}

/// Copia `bytes` de `src` para um buffer mapeável e devolve-o tipado.
#[must_use]
pub fn read<T: Pod>(gpu: &GpuContext, src: &wgpu::Buffer, bytes: u64) -> Vec<T> {
    let stage = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("wet-gpu readback"),
        size: bytes,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut enc = gpu.device.create_command_encoder(&Default::default());
    enc.copy_buffer_to_buffer(src, 0, &stage, 0, bytes);
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

/// Cenas de teste e de medida — partilhadas pelo gate de paridade e pelo `mede_advect`.
pub mod cena {
    use ph2d_wet_paint::grid::Grid;

    /// Uma poça com três manchas e um redemoinho + cisalhamento no fluxo, cobrindo a folha
    /// inteira (`active = 1`, bbox = folha, faixas desligadas). `|uv| ≤ amp`.
    #[must_use]
    pub fn poca(w: usize, h: usize, rf: usize, amp: f32) -> Grid {
        let mut g = Grid::with_flow_ratio(w, h, rf);
        let (wf, hf) = (w as f32, h as f32);
        let manchas = [
            (0.3f32, 0.4f32, 0.18f32),
            (0.62, 0.55, 0.22),
            (0.45, 0.75, 0.12),
        ];
        for y in 1..=h {
            for x in 1..=w {
                let i = x + y * g.s;
                let (nx, ny) = (x as f32 / wf, y as f32 / hf);
                let mut f = 0.05f32;
                for (cx, cy, r) in manchas {
                    let d2 = ((nx - cx).powi(2) + (ny - cy).powi(2)) / (r * r);
                    f += (-d2).exp();
                }
                g.film[i] = f;
                g.susp[i] = 0.3 * f * (0.5 + 0.5 * (7.0 * nx).sin().abs());
                g.susp_rgb[i] = [nx, ny, 1.0 - 0.5 * (nx + ny)];
                g.active[i] = 1;
            }
        }
        let fl = g.flow;
        for cy in 1..=fl.h {
            for cx in 1..=fl.w {
                let (nx, ny) = (cx as f32 / fl.w as f32, cy as f32 / fl.h as f32);
                let (dx, dy) = (nx - 0.5, ny - 0.5);
                let i = cx + cy * fl.s;
                g.flow_x[i] = amp * (-dy * 1.6 + 0.35 * (9.0 * ny).sin()).clamp(-1.0, 1.0);
                g.flow_y[i] = amp * (dx * 1.6 + 0.35 * (11.0 * nx).cos()).clamp(-1.0, 1.0);
            }
        }
        g.bx0 = 1;
        g.by0 = 1;
        g.bx1 = w as i32;
        g.by1 = h as i32;
        g.has_fluid = true;
        g.spans_enabled = false;
        g
    }

    /// A grade espelhada em x: célula `x → w+1−x`, fluxo `ux → −ux` (exige `rf = 1`).
    #[must_use]
    pub fn espelha_x(g: &Grid) -> Grid {
        assert_eq!(
            g.flow.rf, 1,
            "o espelho exato da amostragem do fluxo exige rf = 1"
        );
        let mut m = g.clone();
        for y in 0..g.rows {
            for x in 1..=g.w {
                let (i, j) = (x + y * g.s, (g.w + 1 - x) + y * g.s);
                m.film[j] = g.film[i];
                m.susp[j] = g.susp[i];
                m.susp_rgb[j] = g.susp_rgb[i];
                m.active[j] = g.active[i];
                m.flow_x[j] = -g.flow_x[i];
                m.flow_y[j] = g.flow_y[i];
            }
        }
        m
    }
}
