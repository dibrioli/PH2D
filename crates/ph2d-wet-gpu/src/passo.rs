//! O passo INTEIRO na placa para o produto (degraus A+B, docs/Painter/49_realshader/LIGACAO_PLACA.md):
//! o laço de `sim_step` com as passadas da placa. O estado vive na placa entre [`PassoPlaca::sobe`] e
//! [`PassoPlaca::desce`]; quem lê ou escreve a grade na CPU desce antes e sobe depois (grade inteira).

use crate::seca::{SecaGpu, SecaU};
use crate::{
    AdvectGpu, Knobs, PLANO_BORDAS, PLANO_CAMPO, PLANO_PROJETA, PLANO_REBUILD, PLANO_SEDIMENTA,
    PLANO_SUAVE, PLANO_VMAX, Rect, Uniforms, pack_state, read, unpack_state,
};
use ph2d_gpu::GpuContext;
use ph2d_wet_paint::assenta::Ecra;
use ph2d_wet_paint::grid::Grid;
use ph2d_wet_paint::sim::Sim;
use ph2d_wet_paint::tuning::{Knob, Tuning};

/// A cena só usa o que a placa sabe: lei Cobre, sem K–M, sem paredes/gota/óleo/ladeira, sem
/// fingering/backrun/difusão, passo Jacobi (degraus 3–5).
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
        && !p.paredes
        && p.oleo == 0.0
        && sim.drip_head == 0.0
        && g.oleo.is_empty()
        && ext
}

pub struct PassoPlaca {
    adv: AdvectGpu,
    seca: SecaGpu,
    w: usize,
    h: usize,
    rf: usize,
}

impl PassoPlaca {
    #[must_use]
    pub fn new(gpu: &GpuContext, g: &Grid, sim: &Sim, tuning: &Tuning) -> Self {
        let p = sim.gather_params(tuning);
        let u = SecaU::new(g, &p, sim.evap_scale, sim.rewet_base, sim.ext_bypass);
        let s = PassoPlaca {
            adv: AdvectGpu::new(gpu, g),
            seca: SecaGpu::new(gpu, g, u),
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
        self.adv.set_knobs(
            gpu,
            &Knobs::from_params(&p, g.flow.rf).com_sedimenta(p.sedimenta, grav[0], grav[1]),
        );
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
            self.seca_ida_e_volta(gpu, g, SecaU::new(g, &p, evap, rewet, sim.ext_bypass));
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

    /// A secagem: o estado `st` (entrelaçado) passa pela CPU; `rgb` e `sett` copiam-se na placa.
    /// As faixas = a caixa inteira (a placa não tem as faixas vivas; a caixa as contém).
    fn seca_ida_e_volta(&mut self, gpu: &GpuContext, g: &Grid, u: SecaU) {
        let n = g.cells as u64;
        let (film, susp) = unpack_state(&read::<[f32; 2]>(gpu, &self.adv.st, n * 8));
        let q = &gpu.queue;
        self.seca.set_u(gpu, u);
        self.seca.faixas_da_caixa(gpu, g);
        q.write_buffer(&self.seca.film, 0, bytemuck::cast_slice(&film));
        q.write_buffer(&self.seca.susp, 0, bytemuck::cast_slice(&susp));
        let mut enc = gpu.device.create_command_encoder(&Default::default());
        enc.copy_buffer_to_buffer(&self.adv.rgb, 0, &self.seca.srgb, 0, n * 12);
        enc.copy_buffer_to_buffer(&self.adv.sett, 0, &self.seca.sett, 0, n * 4);
        self.seca.encode(&mut enc, 1);
        enc.copy_buffer_to_buffer(&self.seca.srgb, 0, &self.adv.rgb, 0, n * 12);
        enc.copy_buffer_to_buffer(&self.seca.sett, 0, &self.adv.sett, 0, n * 4);
        q.submit([enc.finish()]);
        let film: Vec<f32> = read(gpu, &self.seca.film, n * 4);
        let susp: Vec<f32> = read(gpu, &self.seca.susp, n * 4);
        q.write_buffer(
            &self.adv.st,
            0,
            bytemuck::cast_slice(&pack_state(&film, &susp)),
        );
    }
}
