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

/// Os uniformes do shader; o layout casa com `struct U` do WGSL (24 × 4 B).
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
    /// Gravidade do passo (`Sim::gravity`) — o `momentum`.
    pub gx: f32,
    pub gy: f32,
    pub max_v: f32,
    pub _p2: f32,
    pub _p3: f32,
    pub _p4: f32,
    /// O retângulo na grade de FLUXO (`flow::flow_bbox` do retângulo fino).
    pub fx0: i32,
    pub fy0: i32,
    pub fx1: i32,
    pub fy1: i32,
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
        let (fx0, fy0, fx1, fy1) = ph2d_wet_paint::flow::flow_bbox((r.x0, r.y0, r.x1, r.y1), f.rf);
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
            gx: 0.0,
            gy: 0.0,
            max_v: max_velocity as f32,
            _p2: 0.0,
            _p3: 0.0,
            _p4: 0.0,
            fx0,
            fy0,
            fx1,
            fy1,
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

/// As passadas na grade de FLUXO, depois das quatro: `smooth` (= `smooth_velocity`, escreve
/// `flow`) e `momentum` (= `momentum_rows`, escreve `vel`). Índices `4` e `5` em `pipes`.
pub const PASSADAS_FLUXO: [&str; 2] = ["suaviza", "momentum"];

/// Um passo de «quadro de suavização» sem secagem: smooth → momentum → advecção.
pub const PLANO_SUAVE: [usize; 6] = [4, 5, 0, 1, 2, 3];

/// As passadas do CAMPO (`build_flow_field_jacobi`): `campo` (= `flow_rows`) e `carimbo`
/// (= `wet_stamp_rows`). Índices `6` e `7` em `pipes`.
pub const PASSADAS_CAMPO: [&str; 2] = ["campo", "carimbo"];

/// Um «quadro de campo» sem secagem (`n % 4 == 0`): campo → carimbo → momentum → advecção.
pub const PLANO_CAMPO: [usize; 7] = [6, 7, 5, 0, 1, 2, 3];

/// As passadas do resto do passo sem secagem (índices `8..=19` em `pipes`): `bordas_*`
/// (`apply_boundaries`), `proj_*` (`project`), `sedimenta`, `re_*` (`rebuild_active_region`).
pub const PASSADAS_PASSO: [&str; 12] = [
    "bordas_massa",
    "bordas_vel",
    "proj_zera",
    "proj_div",
    "proj_prs",
    "proj_corr",
    "sedimenta",
    "re_zera",
    "re_limpa",
    "re_passe1",
    "re_saia",
    "re_caixa",
];

/// `apply_boundaries(g, false)`.
pub const PLANO_BORDAS: [usize; 2] = [8, 9];
/// `project` + `apply_boundaries(g, true)`.
pub const PLANO_PROJETA: [usize; 5] = [10, 11, 12, 13, 9];
/// `sedimenta` + `commit` (o `dst` leva o film inalterado).
pub const PLANO_SEDIMENTA: [usize; 2] = [14, 3];
/// `rebuild_active_region`: a máscara e a caixa nova (lida por [`AdvectGpu::readback_caixa`]).
pub const PLANO_REBUILD: [usize; 5] = [15, 16, 17, 18, 19];
/// O `vmax` da advecção reduzido na placa (`caixa[15]`, ver [`AdvectGpu::zera_vmax`]/[`AdvectGpu::readback_vmax`]).
pub const PASSADA_VMAX: [&str; 1] = ["vmax_red"];
/// O índice de [`PASSADA_VMAX`] em `pipes`.
pub const PLANO_VMAX: [usize; 1] = [20];
/// As passadas da lei `Pigmento` em `pipes`, depois de [`AdvectGpu::liga_tinta`] (`crate::tinta`).
pub const T_TRANSPORTE: usize = 21;
pub const T_COMMIT: usize = 22;
pub const T_SEDIMENTA: usize = 23;
pub const T_BORDAS: usize = 24;
pub const CAMPO_OLEO: usize = 25;
pub const T_CURA: usize = 26;

/// O passo SEM SECAGEM do quadro `n` (1-based, o `Sim::frame`): estágios 0, 2, 3, 4, 5 do
/// `sim_step_stage` — rebuild (`n % 2`), campo (`n % 4`) ou suave, advecção, sedimenta (se
/// `sed`), bordas, projeção (`n % 3`).
#[must_use]
pub fn plano_passo(n: u64, sed: bool) -> Vec<usize> {
    let mut v = Vec::with_capacity(24);
    if n.is_multiple_of(2) {
        v.extend(PLANO_REBUILD);
    }
    if n.is_multiple_of(4) {
        v.extend(PLANO_CAMPO);
    } else {
        v.extend(PLANO_SUAVE);
    }
    if sed {
        v.extend(PLANO_SEDIMENTA);
    }
    v.extend(PLANO_BORDAS);
    if n.is_multiple_of(3) {
        v.extend(PLANO_PROJETA);
    }
    v
}

/// Os knobs do `campo`, do `project` e da `sedimenta`; o layout casa com `struct K` do WGSL
/// (16 × 4 B).
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct Knobs {
    pub level_k: f32,
    pub level_clamp: f32,
    pub cap_k: f32,
    pub cap_gate: f32,
    /// O menor f32 `t` com `t as f64 > viscosity` — a CPU compara `film as f64 > viscosity`.
    pub visc_t: f32,
    pub brake_bias: f32,
    pub brake_reach: f32,
    pub dif: f32,
    pub projection: f32,
    pub sed_ax: f32,
    pub sed_ay: f32,
    pub _k0: f32,
    pub sed_dx: i32,
    pub sed_dy: i32,
    /// `1` = o freio do B13.10 na grade fina ([`ph2d_wet_paint::sim::Params::freio_na_grade_fina`]); `0` = o de antes.
    pub freio_fino: i32,
    pub _k2: i32,
}

/// O menor f32 cujo valor f64 é `> x` — o limiar `>=` em f32 que casa com `> x` em f64.
#[must_use]
pub fn f32_acima(x: f64) -> f32 {
    let t = x as f32;
    if f64::from(t) > x {
        t
    } else {
        f32::from_bits(t.to_bits() + 1)
    }
}

impl Knobs {
    /// Os knobs de `p` na grade de fluxo de razão `rf`.
    #[must_use]
    pub fn from_params(p: &ph2d_wet_paint::sim::Params, rf: usize) -> Self {
        use ph2d_wet_paint::tuning::Knob;
        Knobs {
            level_k: p.k(Knob::Leveling) as f32,
            level_clamp: p.k(Knob::LevelClamp) as f32,
            cap_k: p.k(Knob::Capillary) as f32,
            cap_gate: p.k(Knob::CapillaryGate) as f32,
            visc_t: f32_acima(p.k(Knob::Viscosity)),
            brake_bias: p.k(Knob::Brake) as f32,
            brake_reach: p.k(Knob::BrakeReach) as f32,
            dif: ph2d_wet_paint::flow::diff_scale(rf) as f32,
            projection: p.k(Knob::Projection) as f32,
            freio_fino: i32::from(p.freio_na_grade_fina),
            ..Zeroable::zeroed()
        }
    }

    /// A geometria da `sedimenta` de um passo — a de `solver::sedimenta_rows` com
    /// `a = p.sedimenta·|g|`.
    #[must_use]
    pub fn com_sedimenta(mut self, sedimenta: f64, gx: f64, gy: f64) -> Self {
        let n = (gx * gx + gy * gy).sqrt();
        let a = (sedimenta * n).min(ph2d_wet_paint::solver::SEDIMENTA_MAX);
        let sentido = |v: f64| {
            if v > 1e-12 * n {
                1
            } else if v < -1e-12 * n {
                -1
            } else {
                0
            }
        };
        if a > 0.0 && n > 0.0 {
            self.sed_ax = (a * (gx / n).abs()) as f32;
            self.sed_ay = (a * (gy / n).abs()) as f32;
            self.sed_dx = sentido(gx);
            self.sed_dy = sentido(gy);
        }
        self
    }
}

/// Os buffers e as pipelines de uma grade.
pub struct AdvectGpu {
    pub cells: usize,
    pub ub: wgpu::Buffer,
    pub flow_x: wgpu::Buffer,
    pub flow_y: wgpu::Buffer,
    pub active: wgpu::Buffer,
    pub st: wgpu::Buffer,
    pub rgb: wgpu::Buffer,
    pub vel_x: wgpu::Buffer,
    pub vel_y: wgpu::Buffer,
    pub paper: wgpu::Buffer,
    pub wet: wgpu::Buffer,
    pub sett: wgpu::Buffer,
    pub kb: wgpu::Buffer,
    /// A caixa do rebuild (16 × i32, ver o WGSL) e, depois, o leque do `vmax` por tile 64×64 (B13.4).
    pub caixa: wgpu::Buffer,
    /// Os tiles do leque (`caixa[16..16 + leque]`, [`ph2d_wet_paint::drying::por_tile::tiles_da_grade`]).
    pub leque: usize,
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

/// O conjunto de timestamps e o índice de cada (passo, passada).
pub type Carimbos<'a> = (&'a wgpu::QuerySet, &'a dyn Fn(u32, u32) -> u32);

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
        let vel_x = storage(gpu, "wet-gpu vel_x", fnc * 4);
        let vel_y = storage(gpu, "wet-gpu vel_y", fnc * 4);
        let paper = storage(gpu, "wet-gpu paper", n * 4);
        let wet = storage(gpu, "wet-gpu wet", n * 4);
        let sett = storage(gpu, "wet-gpu sett", n * 4);
        let kb = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("wet-gpu K"),
            size: std::mem::size_of::<Knobs>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let (tx, ty) = ph2d_wet_paint::drying::por_tile::tiles_da_grade(g);
        let leque = tx * ty;
        let caixa = storage(gpu, "wet-gpu caixa", 64 + 4 * leque as u64);
        let module = gpu
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("wet-gpu advect"),
                source: wgpu::ShaderSource::Wgsl(WGSL.into()),
            });
        let all: [(u32, &wgpu::Buffer); 18] = [
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
            (10, &vel_x),
            (11, &vel_y),
            (12, &paper),
            (13, &wet),
            (14, &sett),
            (15, &kb),
            (16, &active),
            (17, &caixa),
        ];
        // Os bindings que cada passada usa (o layout automático só inclui esses).
        let usa: [&[u32]; 21] = [
            &[0, 1, 2, 4],
            &[0, 3, 4, 5],
            &[0, 3, 4, 5, 6, 7, 8, 9],
            &[0, 6, 7, 8, 9],
            &[0, 1, 2, 3, 6, 10, 11],
            &[0, 1, 2, 3, 6, 10, 11],
            &[0, 1, 2, 3, 6, 10, 11, 12, 13, 14, 15],
            &[0, 3, 6, 12, 13],
            &[0, 6, 13],
            &[0, 1, 2, 10, 11],
            &[0, 1, 2],
            &[0, 1, 3, 10, 11, 15],
            &[0, 1, 2, 3],
            &[0, 2, 3, 10, 11, 15],
            &[0, 6, 7, 8, 9, 15],
            &[0, 17],
            &[0, 16],
            &[0, 6, 16, 17],
            &[0, 16, 17],
            &[0, 17],
            &[0, 3, 4, 17],
        ];
        let mut pipes = Vec::new();
        let mut groups = Vec::new();
        for (k, entry) in PASSADAS
            .iter()
            .chain(&PASSADAS_FLUXO)
            .chain(&PASSADAS_CAMPO)
            .chain(&PASSADAS_PASSO)
            .chain(&PASSADA_VMAX)
            .enumerate()
        {
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
            vel_x,
            vel_y,
            paper,
            wet,
            sett,
            kb,
            caixa,
            leque,
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
        q.write_buffer(&self.vel_x, 0, bytemuck::cast_slice(&g.vel_x));
        q.write_buffer(&self.vel_y, 0, bytemuck::cast_slice(&g.vel_y));
        q.write_buffer(&self.paper, 0, bytemuck::cast_slice(&g.paper));
        q.write_buffer(&self.wet, 0, bytemuck::cast_slice(&pack_active(&g.wet)));
        q.write_buffer(&self.sett, 0, bytemuck::cast_slice(&g.sett));
    }

    /// Os knobs do `campo`.
    pub fn set_knobs(&self, gpu: &GpuContext, k: &Knobs) {
        gpu.queue.write_buffer(&self.kb, 0, bytemuck::bytes_of(k));
    }

    /// Lê o `wet` de volta (`u32` por célula, o byte da CPU).
    #[must_use]
    pub fn readback_wet(&self, gpu: &GpuContext) -> Vec<u32> {
        read(gpu, &self.wet, self.cells as u64 * 4)
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
        ts: Option<Carimbos<'_>>,
    ) {
        let (gx, gy) = (r.cols().div_ceil(16), r.rows().div_ceil(16));
        for step in 0..n {
            for (k, (pipe, bg)) in self.pipes.iter().zip(&self.groups).take(4).enumerate() {
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

    /// `n` passos do `plano` (índices de `pipes`) sobre `r`, sem timestamps; as passadas de
    /// fluxo (`≥ 4`) despacham sobre o retângulo de fluxo `u.fx0..fx1`.
    pub fn encode_plano(
        &self,
        enc: &mut wgpu::CommandEncoder,
        u: &Uniforms,
        n: u32,
        plano: &[usize],
        ts: Option<&wgpu::QuerySet>,
    ) {
        for step in 0..n {
            for (j, &k) in plano.iter().enumerate() {
                let (c, r) = grupos(u, k);
                // Com `ts`: início/fim de cada passada em `2·(step·len + j)` e `+1`.
                let q = 2 * (step * plano.len() as u32 + j as u32);
                let tw = ts.map(|qs| wgpu::ComputePassTimestampWrites {
                    query_set: qs,
                    beginning_of_pass_write_index: Some(q),
                    end_of_pass_write_index: Some(q + 1),
                });
                let mut p = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: None,
                    timestamp_writes: tw,
                });
                p.set_pipeline(&self.pipes[k]);
                p.set_bind_group(0, &self.groups[k], &[]);
                p.dispatch_workgroups(c, r, 1);
            }
        }
    }

    /// A caixa publicada pelo último rebuild: `(bx0, bx1, by0, by1, has_fluid)`.
    /// O plano com `dispatch_workgroups_indirect`: a passada `j` lê os seus grupos em `args[12·j..]` (a porta do
    /// `has_fluid` na placa, [`crate::passo`]).
    pub fn encode_plano_indireto(
        &self,
        enc: &mut wgpu::CommandEncoder,
        plano: &[usize],
        args: &wgpu::Buffer,
    ) {
        for (j, &k) in plano.iter().enumerate() {
            let mut p = enc.begin_compute_pass(&Default::default());
            p.set_pipeline(&self.pipes[k]);
            p.set_bind_group(0, &self.groups[k], &[]);
            p.dispatch_workgroups_indirect(args, 12 * j as u64);
        }
    }

    #[must_use]
    pub fn readback_caixa(&self, gpu: &GpuContext) -> (i32, i32, i32, i32, bool) {
        let c: Vec<i32> = read(gpu, &self.caixa, 64);
        (c[10], c[11], c[12], c[13], c[14] != 0)
    }

    /// Zera o `vmax` da placa (`caixa[15]`) e o leque por tile antes de um plano que acaba em [`PLANO_VMAX`].
    pub fn zera_vmax(&self, gpu: &GpuContext) {
        let z = vec![0u8; 4 * (1 + self.leque)];
        gpu.queue.write_buffer(&self.caixa, 60, &z);
    }

    /// O `vmax` reduzido por [`PLANO_VMAX`]: máx |u| por componente nas células ativas da caixa (4 B lidos).
    #[must_use]
    pub fn readback_vmax(&self, gpu: &GpuContext) -> f32 {
        let c: Vec<i32> = read(gpu, &self.caixa, 64);
        f32::from_bits(c[15] as u32)
    }

    /// A máscara `active` de volta (`u32` por célula).
    #[must_use]
    pub fn readback_active(&self, gpu: &GpuContext) -> Vec<u32> {
        read(gpu, &self.active, self.cells as u64 * 4)
    }

    /// `n` passos do `plano`, esperando a placa.
    pub fn step_plano(&self, gpu: &GpuContext, u: &Uniforms, n: u32, plano: &[usize]) {
        let mut enc = gpu.device.create_command_encoder(&Default::default());
        self.encode_plano(&mut enc, u, n, plano, None);
        gpu.queue.submit([enc.finish()]);
        let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
    }

    /// Lê `vel` e `flow` (a grade de fluxo) de volta: `(vel_x, vel_y, flow_x, flow_y)`.
    #[must_use]
    pub fn readback_fluxo(&self, gpu: &GpuContext, fcells: usize) -> [Vec<f32>; 4] {
        let b = fcells as u64 * 4;
        [
            read(gpu, &self.vel_x, b),
            read(gpu, &self.vel_y, b),
            read(gpu, &self.flow_x, b),
            read(gpu, &self.flow_y, b),
        ]
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

/// Os workgroups da passada `k`: o retângulo fino (`0..=3`, `14`), o de fluxo (`4..=7`,
/// `11..=13`), o de fluxo com margem 1 (`10`), a folha fina inteira com o anel (`8`, `16`, `17`),
/// a de fluxo inteira (`9`), uma thread por coluna (`18`, workgroup 64) ou uma thread só.
pub(crate) fn grupos(u: &Uniforms, k: usize) -> (u32, u32) {
    let d = |a: i32, b: i32| (b - a + 1).max(0) as u32;
    let (c, r) = match k {
        0..=3 | 14 | 20 | T_TRANSPORTE | T_COMMIT | T_SEDIMENTA => (d(u.x0, u.x1), d(u.y0, u.y1)),
        4..=7 | 11..=13 | CAMPO_OLEO => (d(u.fx0, u.fx1), d(u.fy0, u.fy1)),
        10 => (d(u.fx0, u.fx1) + 2, d(u.fy0, u.fy1) + 2),
        8 | 16 | 17 | T_BORDAS | T_CURA => ((u.w + 2) as u32, (u.h + 2) as u32),
        9 => ((u.fw + 2) as u32, (u.fh + 2) as u32),
        18 => return (((u.w + 2) as u32).div_ceil(64), 1),
        _ => return (1, 1),
    };
    (c.div_ceil(16), r.div_ceil(16))
}

pub use transfere::{read, read_at};

pub mod cena;
pub mod passo;
pub mod seca;
mod tinta;
mod transfere;
