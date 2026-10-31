//! O passo INTEIRO na placa para o produto (degraus A+B, docs/Painter/49_realshader/LIGACAO_PLACA.md):
//! o laço de `sim_step` com as passadas da placa. O estado vive na placa entre [`PassoPlaca::sobe`] e
//! [`PassoPlaca::desce`]; quem lê ou escreve a grade na CPU desce antes e sobe depois (grade inteira).

use crate::seca::{SecaGpu, SecaU};
use crate::{
    AdvectGpu, Knobs, PLANO_BORDAS, PLANO_CAMPO, PLANO_PROJETA, PLANO_REBUILD, PLANO_SEDIMENTA,
    PLANO_SUAVE, PLANO_VMAX, Rect, Uniforms, read,
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

pub struct PassoPlaca {
    adv: AdvectGpu,
    seca: SecaGpu,
    /// `[separa, junta]` com o mesmo grupo (st, film, susp).
    entrelaca: [wgpu::ComputePipeline; 2],
    entrelaca_bg: [wgpu::BindGroup; 2],
    w: usize,
    h: usize,
    rf: usize,
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
        let s = PassoPlaca {
            adv,
            seca,
            entrelaca: [p0, p1],
            entrelaca_bg: [b0, b1],
            w: g.w,
            h: g.h,
            rf: g.flow.rf,
        };
        s.sobe(gpu, g);
        s
    }

    /// A grade tem a forma desta placa (senão recria-se).
    #[must_use]
    pub fn serve(&self, g: &Grid) -> bool {
        (self.w, self.h, self.rf) == (g.w, g.h, g.flow.rf)
    }

    /// Sobe a grade INTEIRA (o estado da CPU passa a ser o da placa).
    pub fn sobe(&self, gpu: &GpuContext, g: &Grid) {
        self.adv
            .upload(gpu, g, &Uniforms::from_grid(g, 0.0, Rect::full(g)));
        self.seca.upload(gpu, g);
    }

    /// Desce a grade INTEIRA para a CPU (film, susp, cores, sett, vel, flow, active, wet).
    pub fn desce(&self, gpu: &GpuContext, g: &mut Grid) {
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
        if n.is_multiple_of(2) {
            self.adv.step_plano(gpu, &u, 1, &PLANO_REBUILD);
            let (bx0, bx1, by0, by1, hf) = self.adv.readback_caixa(gpu);
            (g.bx0, g.bx1, g.by0, g.by1, g.has_fluid) = (bx0, bx1, by0, by1, hf);
            if !hf {
                return false;
            }
        }
        if n.is_multiple_of(sim.dry_every) {
            let evap = sim.evap_scale * p.k(Knob::Evaporation);
            let rewet = sim.rewet_base * p.k(Knob::Rewet);
            self.seca_na_placa(gpu, g, SecaU::new(g, &p, evap, rewet, sim.ext_bypass));
        }
        let mut plano = Vec::with_capacity(20);
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
        self.adv.step_plano(gpu, &u, 1, &plano);
        if self.adv.readback_vmax(gpu) < 0.5 {
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
