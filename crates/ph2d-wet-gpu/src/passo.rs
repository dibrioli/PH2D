//! O passo INTEIRO na placa para o produto (degraus A+B, docs/Painter/49_realshader/LIGACAO_PLACA.md):
//! o laço de `sim_step` com as passadas da placa. O estado vive na placa entre [`PassoPlaca::sobe`] e
//! [`PassoPlaca::desce`]; quem lê ou escreve a grade na CPU desce antes e sobe depois (grade inteira).

use crate::seca::{SecaGpu, SecaU};
use crate::{
    AdvectGpu, Knobs, PLANO_BORDAS, PLANO_CAMPO, PLANO_PROJETA, PLANO_REBUILD, PLANO_SEDIMENTA,
    PLANO_SUAVE, PLANO_VMAX, Rect, Uniforms, grupos, read, read_at,
};
use ph2d_gpu::GpuContext;
use ph2d_wet_paint::assenta::Ecra;
use ph2d_wet_paint::grid::Grid;
use ph2d_wet_paint::sim::Sim;
use ph2d_wet_paint::tuning::{Knob, Tuning};

/// A cena só usa o que a placa sabe: lei Cobre, sem K–M, sem gota/óleo/ladeira, sem
/// fingering/backrun/difusão, passo Jacobi (degraus 3–5). As paredes entre traços a placa sabe (`traco`).
#[must_use]
pub fn placa_sabe(sim: &Sim, g: &Grid, tuning: &Tuning) -> bool {
    let p = sim.gather_params(tuning);
    let ext = sim.ext_bypass
        || (p.k(Knob::ExtFingering) == 0.0
            && p.k(Knob::ExtBackrun) == 0.0
            && p.k(Knob::ExtDiffusion) == 0.0);
    sim.order_invariant
        && !sim.step_pending()
        && matches!(p.ecra, Ecra::Cobre { .. })
        && !p.km_mixing
        && p.oleo == 0.0
        && sim.drip_head == 0.0
        && g.oleo.is_empty()
        && ext
}

/// O `st` entrelaçado (film, susp) do advect ⇄ os `film`/`susp` da secagem, NA placa (defeito 1).
const ENTRELACA: &str = r"
@group(0) @binding(0) var<storage, read_write> st: array<vec2<f32>>;
@group(0) @binding(1) var<storage, read_write> film: array<f32>;
@group(0) @binding(2) var<storage, read_write> susp: array<f32>;
fn idx(g: vec3<u32>, n: vec3<u32>) -> u32 { return g.x + g.y * n.x * 256u; }
@compute @workgroup_size(256)
fn separa(@builtin(global_invocation_id) g: vec3<u32>, @builtin(num_workgroups) n: vec3<u32>) {
    let i = idx(g, n);
    if i >= arrayLength(&film) { return; }
    film[i] = st[i].x;
    susp[i] = st[i].y;
}
@compute @workgroup_size(256)
fn junta(@builtin(global_invocation_id) g: vec3<u32>, @builtin(num_workgroups) n: vec3<u32>) {
    let i = idx(g, n);
    if i >= arrayLength(&film) { return; }
    st[i] = vec2(film[i], susp[i]);
}
";

/// A porta do `has_fluid` NA placa (item 3 de LIGACAO_PLACA.md): depois do rebuild, os grupos de cada passada
/// do resto do passo = os da CPU se `caixa[14]` (has_fluid), senão 0 — o passo inteiro num submit e numa espera.
const PORTA: &str = r"
@group(0) @binding(0) var<storage, read> caixa: array<i32>;
@group(0) @binding(1) var<storage, read> base: array<u32>;
@group(0) @binding(2) var<storage, read_write> args: array<u32>;
@compute @workgroup_size(64)
fn porta(@builtin(global_invocation_id) g: vec3<u32>) {
    if g.x >= arrayLength(&base) { return; }
    args[g.x] = select(0u, base[g.x], caixa[14] != 0);
}
";
/// Passadas no máximo num passo depois do rebuild (campo 7 + sedimenta 2 + bordas 2 + projecção 5 + vmax 1).
const MAX_PLANO: usize = 24;

pub struct PassoPlaca {
    /// `porta`: os grupos de base (CPU) → os indirectos (`args`, INDIRECT) pela `caixa[14]`.
    porta: wgpu::ComputePipeline,
    porta_bg: wgpu::BindGroup,
    base: wgpu::Buffer,
    args: wgpu::Buffer,
    /// 64 B mapeáveis: a caixa e o `vmax` lidos numa só cópia no fim do passo.
    stage: wgpu::Buffer,
    adv: AdvectGpu,
    seca: SecaGpu,
    /// `[separa, junta]` com o mesmo grupo (st, film, susp).
    entrelaca: [wgpu::ComputePipeline; 2],
    entrelaca_bg: [wgpu::BindGroup; 2],
    w: usize,
    h: usize,
    rf: usize,
    /// O ESPELHO: o hash de cada linha fina e de fluxo da CPU quando placa e CPU eram iguais (depois de subir ou
    /// descer). Vazio = não há espelho (sobe tudo). O `sobe_sujo` sobe só as linhas cujo hash mudou.
    hash_fina: Vec<u64>,
    hash_fluxo: Vec<u64>,
    /// As linhas finas que a placa pode ter mudado desde a subida (a união das caixas): o `desce_suja` desce só elas.
    suja: Option<(i32, i32)>,
}

/// Hash de uma linha: multiplica-e-roda sobre palavras de 8 B (não é cripto; a colisão de 64 bits é desprezável).
fn mistura(mut h: u64, d: &[u8]) -> u64 {
    let (palavras, resto) = d.as_chunks::<8>();
    for c in palavras {
        let w = u64::from_le_bytes(*c);
        h = (h ^ w).wrapping_mul(0x9E37_79B9_7F4A_7C15).rotate_left(31);
    }
    for &b in resto {
        h = (h ^ u64::from(b))
            .wrapping_mul(0x9E37_79B9_7F4A_7C15)
            .rotate_left(31);
    }
    h
}

/// Os hashes das linhas `0..n` por `f(linha)`, em paralelo (fatias de linhas por núcleo).
fn hashes(n: usize, f: &(dyn Fn(usize) -> u64 + Sync)) -> Vec<u64> {
    let mut v = vec![0u64; n];
    let nt = std::thread::available_parallelism()
        .map_or(4, |k| k.get())
        .min(16);
    let por = n.div_ceil(nt).max(1);
    std::thread::scope(|sc| {
        for (k, ch) in v.chunks_mut(por).enumerate() {
            sc.spawn(move || {
                for (j, h) in ch.iter_mut().enumerate() {
                    *h = f(k * por + j);
                }
            });
        }
    });
    v
}

/// O hash de uma linha FINA de tudo o que a placa espelha.
fn hash_fina(g: &Grid, y: usize) -> u64 {
    let (a, b) = (y * g.s, (y + 1) * g.s);
    let c = bytemuck::cast_slice::<f32, u8>;
    let mut h = mistura(y as u64, c(&g.film[a..b]));
    h = mistura(h, c(&g.susp[a..b]));
    h = mistura(h, bytemuck::cast_slice(&g.susp_rgb[a..b]));
    h = mistura(h, c(&g.sett[a..b]));
    h = mistura(h, bytemuck::cast_slice(&g.sett_rgb[a..b]));
    h = mistura(h, c(&g.paper[a..b]));
    h = mistura(h, &g.active[a..b]);
    h = mistura(h, &g.wet[a..b]);
    if !g.traco.is_empty() {
        h = mistura(h, bytemuck::cast_slice(&g.traco[a..b]));
    }
    if !g.vela.is_empty() {
        h = mistura(h, c(&g.vela[a..b]));
    }
    if !g.aparencia.is_empty() {
        let v: Vec<f32> = (a..b)
            .map(|i| ph2d_wet_paint::grid::aparencia::vidrado(&g.aparencia, i))
            .collect();
        h = mistura(h, c(&v));
    }
    h
}

/// O hash de uma linha da grade de FLUXO (velocidade e fluxo).
fn hash_fluxo(g: &Grid, y: usize) -> u64 {
    let (a, b) = (y * g.flow.s, (y + 1) * g.flow.s);
    let c = bytemuck::cast_slice::<f32, u8>;
    let mut h = mistura(y as u64, c(&g.vel_x[a..b]));
    h = mistura(h, c(&g.vel_y[a..b]));
    h = mistura(h, c(&g.flow_x[a..b]));
    mistura(h, c(&g.flow_y[a..b]))
}

/// As linhas de fluxo que cobrem as finas `y0..=y1` (com uma de folga de cada lado).
fn fluxo_de(g: &Grid, y0: usize, y1: usize) -> (usize, usize) {
    let rf = g.flow.rf;
    (
        y0.saturating_sub(1) / rf,
        (y1 / rf + 1).min(g.flow.rows - 1),
    )
}

/// As linhas finas que uma linha de fluxo cobre (com folga).
fn finas_de(g: &Grid, f: usize) -> (usize, usize) {
    let rf = g.flow.rf;
    (f.saturating_sub(1) * rf, ((f + 1) * rf).min(g.rows - 1))
}

impl PassoPlaca {
    #[must_use]
    pub fn new(gpu: &GpuContext, g: &Grid, sim: &Sim, tuning: &Tuning) -> Self {
        let p = sim.gather_params(tuning);
        let u = SecaU::new(g, &p, sim.evap_scale, sim.rewet_base, sim.ext_bypass);
        let adv = AdvectGpu::new(gpu, g);
        let seca = SecaGpu::new(gpu, g, u);
        let module = gpu
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("wet-gpu entrelaca"),
                source: wgpu::ShaderSource::Wgsl(ENTRELACA.into()),
            });
        let mk = |entry: &str| {
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
            let bg = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(entry),
                layout: &pipe.get_bind_group_layout(0),
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: adv.st.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: seca.film.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: seca.susp.as_entire_binding(),
                    },
                ],
            });
            (pipe, bg)
        };
        let (p0, b0) = mk("separa");
        let (p1, b1) = mk("junta");
        use wgpu::BufferUsages as B;
        let buf = |label: &str, size: u64, usage: B| {
            gpu.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                size,
                usage,
                mapped_at_creation: false,
            })
        };
        let n_args = (MAX_PLANO * 12) as u64;
        let base = buf("wet-gpu porta base", n_args, B::STORAGE | B::COPY_DST);
        let args = buf("wet-gpu porta args", n_args, B::STORAGE | B::INDIRECT);
        let stage = buf("wet-gpu caixa stage", 64, B::COPY_DST | B::MAP_READ);
        let pm = gpu
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("wet-gpu porta"),
                source: wgpu::ShaderSource::Wgsl(PORTA.into()),
            });
        let porta = gpu
            .device
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("porta"),
                layout: None,
                module: &pm,
                entry_point: Some("porta"),
                compilation_options: Default::default(),
                cache: None,
            });
        let porta_bg = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("porta"),
            layout: &porta.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: adv.caixa.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: base.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: args.as_entire_binding(),
                },
            ],
        });
        let s = PassoPlaca {
            porta,
            porta_bg,
            base,
            args,
            stage,
            adv,
            seca,
            entrelaca: [p0, p1],
            entrelaca_bg: [b0, b1],
            w: g.w,
            h: g.h,
            rf: g.flow.rf,
            hash_fina: Vec::new(),
            hash_fluxo: Vec::new(),
            suja: None,
        };
        let mut s = s;
        s.sobe(gpu, g);
        s
    }

    /// A grade tem a forma desta placa (senão recria-se).
    #[must_use]
    pub fn serve(&self, g: &Grid) -> bool {
        (self.w, self.h, self.rf) == (g.w, g.h, g.flow.rf)
    }

    /// Sobe a grade INTEIRA (o estado da CPU passa a ser o da placa) e grava o espelho.
    pub fn sobe(&mut self, gpu: &GpuContext, g: &Grid) {
        self.adv
            .upload(gpu, g, &Uniforms::from_grid(g, 0.0, Rect::full(g)));
        self.seca.upload(gpu, g);
        self.hash_fina = hashes(g.rows, &|y| hash_fina(g, y));
        self.hash_fluxo = hashes(g.flow.rows, &|y| hash_fluxo(g, y));
        self.suja = g.has_fluid.then_some((g.by0, g.by1));
    }

    /// A placa deixa de espelhar a CPU (a CPU vai andar o passo): a próxima subida é inteira.
    pub fn esquece(&mut self) {
        self.hash_fina.clear();
        self.hash_fluxo.clear();
    }

    /// Sobe só a FAIXA de linhas que a CPU mudou desde a última subida/descida (hash por linha); sem espelho, tudo.
    /// Devolve as linhas finas subidas (`None` = nada mudou).
    pub fn sobe_sujo(&mut self, gpu: &GpuContext, g: &Grid) -> Option<(usize, usize)> {
        if self.hash_fina.len() != g.rows || self.hash_fluxo.len() != g.flow.rows {
            self.sobe(gpu, g);
            return Some((0, g.rows - 1));
        }
        let nf = hashes(g.rows, &|y| hash_fina(g, y));
        let nv = hashes(g.flow.rows, &|y| hash_fluxo(g, y));
        let mut faixa: Option<(usize, usize)> = None;
        let mut junta = |a: usize, b: usize| {
            faixa = Some(faixa.map_or((a, b), |(x, y)| (x.min(a), y.max(b))));
        };
        for y in (0..g.rows).filter(|&y| nf[y] != self.hash_fina[y]) {
            junta(y, y);
        }
        for f in (0..g.flow.rows).filter(|&f| nv[f] != self.hash_fluxo[f]) {
            let (a, b) = finas_de(g, f);
            junta(a, b);
        }
        self.hash_fina = nf;
        self.hash_fluxo = nv;
        self.suja = g.has_fluid.then_some((g.by0, g.by1));
        let (y0, y1) = faixa?;
        self.adv
            .upload_linhas(gpu, g, (y0, y1), fluxo_de(g, y0, y1));
        self.seca.upload_linhas(gpu, g, y0, y1);
        Some((y0, y1))
    }

    /// Desce só as linhas que a placa pode ter mudado (a união das caixas desde a subida, e as duas de borda de
    /// cada lado — o dreno das `bordas`) e actualiza o espelho delas.
    pub fn desce_suja(&mut self, gpu: &GpuContext, g: &mut Grid) {
        if self.hash_fina.len() != g.rows {
            self.desce(gpu, g);
            return;
        }
        let ult = g.rows - 1;
        let mut faixas = vec![(0, 1), (ult - 1, ult)];
        if let Some((a, b)) = self.suja {
            faixas.push(((a.max(0) as usize).min(ult), (b.max(0) as usize).min(ult)));
        }
        for (y0, y1) in faixas {
            self.desce_linhas(gpu, g, y0, y1);
            let (f0, f1) = fluxo_de(g, y0, y1);
            for y in y0..=y1 {
                self.hash_fina[y] = hash_fina(g, y);
            }
            for f in f0..=f1 {
                self.hash_fluxo[f] = hash_fluxo(g, f);
            }
        }
        let full = g.s as i32 - 1;
        g.row_lo.fill(0);
        g.row_hi.fill(full);
        self.suja = g.has_fluid.then_some((g.by0, g.by1));
    }

    /// Desce as linhas finas `y0..=y1` e as de fluxo que as cobrem.
    fn desce_linhas(&self, gpu: &GpuContext, g: &mut Grid, y0: usize, y1: usize) {
        let (a, b) = (y0 * g.s, (y1 + 1) * g.s);
        let n = (b - a) as u64;
        let st: Vec<[f32; 2]> = read_at(gpu, &self.adv.st, a as u64 * 8, n * 8);
        for (k, v) in st.iter().enumerate() {
            g.film[a + k] = v[0];
            g.susp[a + k] = v[1];
        }
        let rgb: Vec<[f32; 3]> = read_at(gpu, &self.adv.rgb, a as u64 * 12, n * 12);
        g.susp_rgb[a..b].copy_from_slice(&rgb);
        let sett: Vec<f32> = read_at(gpu, &self.adv.sett, a as u64 * 4, n * 4);
        g.sett[a..b].copy_from_slice(&sett);
        let trgb: Vec<[f32; 3]> = read_at(gpu, &self.seca.trgb, a as u64 * 12, n * 12);
        g.sett_rgb[a..b].copy_from_slice(&trgb);
        let act: Vec<u32> = read_at(gpu, &self.adv.active, a as u64 * 4, n * 4);
        for (d, &v) in g.active[a..b].iter_mut().zip(&act) {
            *d = v as u8;
        }
        let wet: Vec<u32> = read_at(gpu, &self.adv.wet, a as u64 * 4, n * 4);
        for (d, &v) in g.wet[a..b].iter_mut().zip(&wet) {
            *d = v as u8;
        }
        let (f0, f1) = fluxo_de(g, y0, y1);
        let (fa, fb) = (f0 * g.flow.s, (f1 + 1) * g.flow.s);
        let rf = |buf: &wgpu::Buffer| -> Vec<f32> {
            read_at(gpu, buf, fa as u64 * 4, (fb - fa) as u64 * 4)
        };
        g.vel_x[fa..fb].copy_from_slice(&rf(&self.adv.vel_x));
        g.vel_y[fa..fb].copy_from_slice(&rf(&self.adv.vel_y));
        g.flow_x[fa..fb].copy_from_slice(&rf(&self.adv.flow_x));
        g.flow_y[fa..fb].copy_from_slice(&rf(&self.adv.flow_y));
    }

    /// Desce a grade INTEIRA para a CPU (film, susp, cores, sett, vel, flow, active, wet).
    pub fn desce(&mut self, gpu: &GpuContext, g: &mut Grid) {
        let e = self.adv.readback(gpu);
        g.film = e.film;
        g.susp = e.susp;
        g.susp_rgb = e.rgb;
        let n = g.cells as u64;
        g.sett = read(gpu, &self.adv.sett, n * 4);
        g.sett_rgb = read(gpu, &self.seca.trgb, n * 12);
        let [vx, vy, fx, fy] = self.adv.readback_fluxo(gpu, g.flow.cells);
        (g.vel_x, g.vel_y, g.flow_x, g.flow_y) = (vx, vy, fx, fy);
        let a = self.adv.readback_active(gpu);
        g.active = a.iter().map(|&v| v as u8).collect();
        let w = self.adv.readback_wet(gpu);
        g.wet = w.iter().map(|&v| v as u8).collect();
        // A placa não tem faixas vivas: a linha inteira é o superconjunto declarado (`active ⊆ faixa`);
        // o próximo rebuild da CPU reaperta-as.
        let full = g.s as i32 - 1;
        g.row_lo.fill(0);
        g.row_hi.fill(full);
        self.hash_fina = hashes(g.rows, &|y| hash_fina(g, y));
        self.hash_fluxo = hashes(g.flow.rows, &|y| hash_fluxo(g, y));
        self.suja = g.has_fluid.then_some((g.by0, g.by1));
    }

    /// A caixa entra na faixa suja (o que a placa pode mudar).
    fn suja_junta(&mut self, g: &Grid) {
        if g.has_fluid {
            let (a, b) = (g.by0, g.by1);
            self.suja = Some(self.suja.map_or((a, b), |(x, y)| (x.min(a), y.max(b))));
        }
    }

    /// UM passo (o `sim_step` da CPU). `g` só guarda a caixa e o `has_fluid` (o resto está na placa).
    pub fn passo(
        &mut self,
        gpu: &GpuContext,
        sim: &mut Sim,
        g: &mut Grid,
        tuning: &Tuning,
    ) -> bool {
        if !g.has_fluid {
            return false;
        }
        self.suja_junta(g);
        let p = sim.gather_params(tuning);
        let grav = sim.gravity(tuning);
        sim.frame += 1;
        let n = sim.frame;
        let mut u = Uniforms::from_grid(g, p.k(Knob::MaxVelocity), Rect::full(g));
        u.gx = grav[0] as f32;
        u.gy = grav[1] as f32;
        self.adv.set_uniforms(gpu, &u);
        let mut k = Knobs::from_params(&p, g.flow.rf).com_sedimenta(p.sedimenta, grav[0], grav[1]);
        k.isola = i32::from(p.paredes && !g.traco.is_empty());
        self.adv.set_knobs(gpu, &k);
        let par = n.is_multiple_of(2);
        let seca = n.is_multiple_of(sim.dry_every);
        let mut enc = gpu.device.create_command_encoder(&Default::default());
        if par {
            self.adv.encode_plano(&mut enc, &u, 1, &PLANO_REBUILD, None);
        }
        // Passo de rebuild COM secagem: a secagem precisa da caixa nova na CPU (faixas e grupos) — a 2.ª espera.
        if par && seca {
            enc.copy_buffer_to_buffer(&self.adv.caixa, 0, &self.stage, 0, 64);
            gpu.queue.submit([enc.finish()]);
            let c = self.le_caixa(gpu);
            (g.bx0, g.bx1, g.by0, g.by1, g.has_fluid) = (c[10], c[11], c[12], c[13], c[14] != 0);
            self.suja_junta(g);
            if !g.has_fluid {
                return false;
            }
            enc = gpu.device.create_command_encoder(&Default::default());
        }
        if seca {
            let evap = sim.evap_scale * p.k(Knob::Evaporation);
            let rewet = sim.rewet_base * p.k(Knob::Rewet);
            self.seca_na_placa(gpu, g, SecaU::new(g, &p, evap, rewet, sim.ext_bypass));
        }
        let mut plano = Vec::with_capacity(MAX_PLANO);
        plano.extend_from_slice(if n.is_multiple_of(4) {
            &PLANO_CAMPO[..]
        } else {
            &PLANO_SUAVE[..]
        });
        if p.sedimenta > 0.0 {
            plano.extend(PLANO_SEDIMENTA);
        }
        plano.extend(PLANO_BORDAS);
        if n.is_multiple_of(3) {
            plano.extend(PLANO_PROJETA);
        }
        plano.extend(PLANO_VMAX);
        self.adv.zera_vmax(gpu);
        // Rebuild sem secagem no mesmo submit: o resto do passo passa pela porta do `has_fluid` (indirecto).
        let porta = par && !seca;
        if porta {
            let base: Vec<[u32; 3]> = plano
                .iter()
                .map(|&k| {
                    let (c, r) = grupos(&u, k);
                    [c, r, 1]
                })
                .collect();
            gpu.queue
                .write_buffer(&self.base, 0, bytemuck::cast_slice(&base));
            let mut cp = enc.begin_compute_pass(&Default::default());
            cp.set_pipeline(&self.porta);
            cp.set_bind_group(0, &self.porta_bg, &[]);
            cp.dispatch_workgroups(1, 1, 1);
            drop(cp);
            self.adv.encode_plano_indireto(&mut enc, &plano, &self.args);
        } else {
            self.adv.encode_plano(&mut enc, &u, 1, &plano, None);
        }
        enc.copy_buffer_to_buffer(&self.adv.caixa, 0, &self.stage, 0, 64);
        gpu.queue.submit([enc.finish()]);
        let c = self.le_caixa(gpu);
        if porta {
            (g.bx0, g.bx1, g.by0, g.by1, g.has_fluid) = (c[10], c[11], c[12], c[13], c[14] != 0);
            self.suja_junta(g);
            if !g.has_fluid {
                return false;
            }
        }
        if f32::from_bits(c[15] as u32) < 0.5 {
            sim.dry_every = 6;
            sim.evap_scale = 0.001;
            sim.rewet_base = 0.0001;
        } else {
            sim.dry_every = 3;
            sim.evap_scale = 0.00025;
            sim.rewet_base = 0.000025;
        }
        true
    }

    /// A ÚNICA espera: mapeia os 64 B que o submit copiou para `stage` (caixa 10..14, `vmax` 15).
    fn le_caixa(&self, gpu: &GpuContext) -> [i32; 16] {
        let sl = self.stage.slice(..);
        sl.map_async(wgpu::MapMode::Read, |r| r.expect("map"));
        let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
        let c = {
            let d = sl.get_mapped_range().expect("mapped range");
            let mut c = [0i32; 16];
            c.copy_from_slice(bytemuck::cast_slice(&d));
            c
        };
        self.stage.unmap();
        c
    }

    /// A secagem toda na placa: `st` → film/susp (separa), `rgb`/`sett` copiados, as secagens, e de volta.
    /// As faixas = a caixa inteira (a placa não tem as faixas vivas; a caixa as contém).
    fn seca_na_placa(&mut self, gpu: &GpuContext, g: &Grid, u: SecaU) {
        let n = g.cells as u64;
        self.seca.set_u(gpu, u);
        self.seca.faixas_da_caixa(gpu, g);
        let grupos = (g.cells as u32).div_ceil(256);
        let (gx, gy) = (grupos.min(65_535), grupos.div_ceil(65_535));
        let mut enc = gpu.device.create_command_encoder(&Default::default());
        let entrelaca = |enc: &mut wgpu::CommandEncoder, k: usize| {
            let mut cp = enc.begin_compute_pass(&Default::default());
            cp.set_pipeline(&self.entrelaca[k]);
            cp.set_bind_group(0, &self.entrelaca_bg[k], &[]);
            cp.dispatch_workgroups(gx, gy, 1);
        };
        entrelaca(&mut enc, 0);
        enc.copy_buffer_to_buffer(&self.adv.rgb, 0, &self.seca.srgb, 0, n * 12);
        enc.copy_buffer_to_buffer(&self.adv.sett, 0, &self.seca.sett, 0, n * 4);
        self.seca.encode(&mut enc, 1);
        enc.copy_buffer_to_buffer(&self.seca.srgb, 0, &self.adv.rgb, 0, n * 12);
        enc.copy_buffer_to_buffer(&self.seca.sett, 0, &self.adv.sett, 0, n * 4);
        entrelaca(&mut enc, 1);
        gpu.queue.submit([enc.finish()]);
    }
}
