//! O passo INTEIRO na placa para o produto (degraus A+B, docs/Painter/49_realshader/LIGACAO_PLACA.md):
//! o laço de `sim_step` com as passadas da placa. O estado vive na placa entre [`PassoPlaca::sobe`] e
//! [`PassoPlaca::desce`]; quem lê ou escreve a grade na CPU desce antes e sobe depois (grade inteira).

use crate::seca::{SecaGpu, SecaU};
use crate::{
    AdvectGpu, Knobs, PLANO_BORDAS, PLANO_CAMPO, PLANO_PROJETA, PLANO_REBUILD, PLANO_SEDIMENTA,
    PLANO_SUAVE, PLANO_VMAX, Rect, Uniforms, grupos, read,
};
use ph2d_gpu::GpuContext;
use ph2d_wet_paint::assenta::Ecra;
use ph2d_wet_paint::grid::Grid;
use ph2d_wet_paint::sim::{LeiDaTinta, Sim};
use ph2d_wet_paint::tuning::{Knob, Tuning};

/// A cena só usa o que a placa sabe: lei Cobre, sem K–M, sem gota/óleo/ladeira, sem fingering/backrun/difusão, passo
/// Jacobi (degraus 3–5). A tinta OPACA (`Knob::Opaque`, `Ecra::com_opaca`) e a
/// lei `Pigmento` (até à E11) a placa NÃO sabe: o pincel opaco ou uma célula com ela fica na CPU — nunca diverge.
#[must_use]
pub fn placa_sabe(sim: &Sim, g: &Grid, tuning: &Tuning) -> bool {
    let p = sim.gather_params(tuning);
    let ext = sim.ext_bypass
        || (p.k(Knob::ExtFingering) == 0.0
            && p.k(Knob::ExtBackrun) == 0.0
            && p.k(Knob::ExtDiffusion) == 0.0);
    sim.order_invariant
        && sim.tinta == LeiDaTinta::Velha
        && !sim.step_pending()
        && matches!(p.ecra, Ecra::Cobre { .. })
        && !p.km_mixing
        && p.oleo == 0.0
        && sim.drip_head == 0.0
        && g.oleo.is_empty()
        && p.k(Knob::Opaque) == 0.0
        && g.aparencia.iter().all(|a| a.opaca == 0.0)
        && ext
}

mod fatias;
mod hashes;
mod wgsl;

use fatias::{Despacho, Fatias, MAX_CARIMBOS};
use hashes::{finas_de, fluxo_de, hash_fina, hash_fluxo, hashes};
use wgsl::{ENTRELACA, PORTA};

/// Passadas no máximo num passo depois do rebuild (campo 7 + sedimenta 2 + bordas 2 + projecção 5 + vmax 1).
const MAX_PLANO: usize = 24;
/// O orçamento de UMA submissão, em workgroups de 256 fios: corta-se antes da passada que o passaria
/// (uma passada maior que ele vai sozinha). 4K = 41 k grupos por passada de folha ⇒ cada uma vai só.
/// Medido (PLACA_degrau6.md, Radeon 610M, 4096×2560): maior submissão 93,3 → 22,6 ms (= a maior passada,
/// `outflow_pass`) ⇒ ~2,8 ms na RTX 5060 Ti (8,0× no passo inteiro, 74 × 9,24 ms), ao bit.
pub const ORCAMENTO_GRUPOS: u64 = 40_000;
/// A guarda: uma submissão acima disto (carimbos da placa) ⇒ [`PassoPlaca::estourou`] e o worker volta à CPU.
/// ~9× a maior submissão extrapolada para a RTX (2,8 ms); na Radeon 610M a maior passada já é 22,6 ms — lá
/// a guarda pode disparar sob carga, e a CPU é ~10× mais rápida nessa placa (LIGACAO_PLACA.md ronda 3).
pub const TETO_MS: f64 = 25.0;

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
    /// Carimbos por passada (`None` = a placa não tem `TIMESTAMP_QUERY`: sem guarda) e o destino resolvido.
    qs: Option<(wgpu::QuerySet, wgpu::Buffer)>,
    orcamento: u64,
    teto_ms: f64,
    /// ms de placa de cada submissão do último passo, e de cada passada (pelos carimbos).
    ms_sub: Vec<f64>,
    ms_passada: Vec<f64>,
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
        let n_ts = u64::from(MAX_CARIMBOS) * 8;
        let stage = buf("wet-gpu caixa stage", 64 + n_ts, B::COPY_DST | B::MAP_READ);
        let qs = gpu
            .device
            .features()
            .contains(wgpu::Features::TIMESTAMP_QUERY)
            .then(|| {
                let q = gpu.device.create_query_set(&wgpu::QuerySetDescriptor {
                    label: Some("wet-gpu carimbos"),
                    ty: wgpu::QueryType::Timestamp,
                    count: MAX_CARIMBOS,
                });
                (
                    q,
                    buf("wet-gpu carimbos", n_ts, B::QUERY_RESOLVE | B::COPY_SRC),
                )
            });
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
            qs,
            orcamento: ORCAMENTO_GRUPOS,
            teto_ms: TETO_MS,
            ms_sub: Vec::new(),
            ms_passada: Vec::new(),
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
        self.desce_faixas(gpu, g, &faixas);
        for (y0, y1) in faixas {
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

    /// Desce as linhas finas de cada faixa e as de fluxo que as cobrem: as 10 cópias de cada faixa num
    /// codificador, num buffer, numa espera (antes: uma espera por cópia, 33 por descida suja).
    fn desce_faixas(&self, gpu: &GpuContext, g: &mut Grid, faixas: &[(usize, usize)]) {
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
        let k = Knobs::from_params(&p, g.flow.rf).com_sedimenta(p.sedimenta, grav[0], grav[1]);
        self.adv.set_knobs(gpu, &k);
        let par = n.is_multiple_of(2);
        let seca = n.is_multiple_of(sim.dry_every);
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
        // As escritas da fila aplicam-se antes da PRÓXIMA submissão: todas antes da primeira (como num submit só).
        self.adv.zera_vmax(gpu);
        // Rebuild sem secagem no mesmo passo: o resto passa pela porta do `has_fluid` (indirecto).
        let porta = par && !seca;
        let base: Vec<[u32; 3]> = plano
            .iter()
            .map(|&k| {
                let (c, r) = grupos(&u, k);
                [c, r, 1]
            })
            .collect();
        if porta {
            gpu.queue
                .write_buffer(&self.base, 0, bytemuck::cast_slice(&base));
        }
        let mut f = Fatias::new(gpu, self.orcamento, self.qs.as_ref().map(|q| q.0.clone()));
        let custo = |c: u32, r: u32| u64::from(c) * u64::from(r);
        if par {
            for &k in &PLANO_REBUILD {
                let (c, r) = grupos(&u, k);
                let (pipe, bg) = (&self.adv.pipes[k], &self.adv.groups[k]);
                f.passada(pipe, bg, Despacho::Direto(c, r), custo(c, r));
            }
        }
        // Passo de rebuild COM secagem: a secagem precisa da caixa nova na CPU (faixas e grupos) — a 2.ª espera.
        if par && seca {
            f.enc
                .copy_buffer_to_buffer(&self.adv.caixa, 0, &self.stage, 0, 64);
            f.corta();
            let (c, _) = self.le_stage(gpu, 0);
            (g.bx0, g.bx1, g.by0, g.by1, g.has_fluid) = (c[10], c[11], c[12], c[13], c[14] != 0);
            self.suja_junta(g);
            if !g.has_fluid {
                return false;
            }
        }
        if seca {
            let evap = sim.evap_scale * p.k(Knob::Evaporation);
            let rewet = sim.rewet_base * p.k(Knob::Rewet);
            self.seca_na_placa(&mut f, g, SecaU::new(g, &p, evap, rewet, sim.ext_bypass));
        }
        if porta {
            f.passada(&self.porta, &self.porta_bg, Despacho::Direto(1, 1), 1);
            for (j, &k) in plano.iter().enumerate() {
                let (pipe, bg) = (&self.adv.pipes[k], &self.adv.groups[k]);
                let d = Despacho::Indireto(&self.args, 12 * j as u64);
                f.passada(pipe, bg, d, custo(base[j][0], base[j][1]));
            }
        } else {
            for (j, &k) in plano.iter().enumerate() {
                let (pipe, bg) = (&self.adv.pipes[k], &self.adv.groups[k]);
                let (c, r) = (base[j][0], base[j][1]);
                f.passada(pipe, bg, Despacho::Direto(c, r), custo(c, r));
            }
        }
        f.enc
            .copy_buffer_to_buffer(&self.adv.caixa, 0, &self.stage, 0, 64);
        let nq = f.q;
        if let Some((qs, res)) = &self.qs
            && nq > 0
        {
            f.enc.resolve_query_set(qs, 0..nq, res, 0);
            f.enc
                .copy_buffer_to_buffer(res, 0, &self.stage, 64, u64::from(nq) * 8);
        }
        f.corta();
        let cortes = std::mem::take(&mut f.cortes);
        drop(f);
        let (c, ts) = self.le_stage(gpu, nq);
        self.carimba(&ts, &cortes, gpu);
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

    /// ms de placa por submissão e por passada do último passo, a partir dos carimbos.
    fn carimba(&mut self, ts: &[u64], cortes: &[(u32, u32)], gpu: &GpuContext) {
        self.ms_sub.clear();
        self.ms_passada.clear();
        if ts.is_empty() {
            return;
        }
        let per = f64::from(gpu.queue.get_timestamp_period()) * 1e-6;
        let ms = |a: u32, b: u32| ts[b as usize].saturating_sub(ts[a as usize]) as f64 * per;
        self.ms_sub = cortes.iter().map(|&(a, b)| ms(a, b)).collect();
        self.ms_passada = (0..ts.len() as u32 / 2)
            .map(|j| ms(2 * j, 2 * j + 1))
            .collect();
    }

    /// ms de placa de cada submissão do último passo (vazio sem `TIMESTAMP_QUERY`).
    #[must_use]
    pub fn ms_por_submissao(&self) -> &[f64] {
        &self.ms_sub
    }

    /// ms de placa de cada passada do último passo, na ordem de gravação.
    #[must_use]
    pub fn ms_por_passada(&self) -> &[f64] {
        &self.ms_passada
    }

    /// A guarda: `Some(ms)` = a maior submissão do último passo passou do tecto ([`TETO_MS`]).
    #[must_use]
    pub fn estourou(&self) -> Option<f64> {
        let m = self.ms_sub.iter().copied().fold(0.0, f64::max);
        (m > self.teto_ms).then_some(m)
    }

    /// O orçamento por submissão em workgroups (`u64::MAX` = um submit por passo, o de antes) e o tecto da guarda.
    pub fn afina(&mut self, orcamento: u64, teto_ms: f64) {
        self.orcamento = orcamento.max(1);
        self.teto_ms = teto_ms;
    }

    /// A espera: mapeia os 64 B que o submit copiou para `stage` (caixa 10..14, `vmax` 15) e os `nq` carimbos.
    fn le_stage(&self, gpu: &GpuContext, nq: u32) -> ([i32; 16], Vec<u64>) {
        let sl = self.stage.slice(..);
        sl.map_async(wgpu::MapMode::Read, |r| r.expect("map"));
        let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
        let r = {
            let d = sl.get_mapped_range().expect("mapped range");
            let mut c = [0i32; 16];
            c.copy_from_slice(bytemuck::cast_slice(&d[..64]));
            let ts: Vec<u64> = bytemuck::cast_slice(&d[64..64 + nq as usize * 8]).to_vec();
            (c, ts)
        };
        self.stage.unmap();
        r
    }

    /// A secagem toda na placa: `st` → film/susp (separa), `rgb`/`sett` copiados, as secagens, e de volta.
    /// As faixas = a caixa inteira (a placa não tem as faixas vivas; a caixa as contém).
    fn seca_na_placa(&mut self, f: &mut Fatias<'_>, g: &Grid, u: SecaU) {
        let n = g.cells as u64;
        self.seca.set_u(f.gpu, u);
        self.seca.faixas_da_caixa(f.gpu, g);
        let grupos = (g.cells as u32).div_ceil(256);
        let (gx, gy) = (grupos.min(65_535), grupos.div_ceil(65_535));
        let ent = u64::from(grupos);
        let (p0, b0) = (&self.entrelaca[0], &self.entrelaca_bg[0]);
        f.passada(p0, b0, Despacho::Direto(gx, gy), ent);
        f.enc
            .copy_buffer_to_buffer(&self.adv.rgb, 0, &self.seca.srgb, 0, n * 12);
        f.enc
            .copy_buffer_to_buffer(&self.adv.sett, 0, &self.seca.sett, 0, n * 4);
        if let Some((pipes, sx, sy)) = self.seca.passadas() {
            for (pipe, bg) in pipes {
                f.passada(
                    pipe,
                    bg,
                    Despacho::Direto(sx, sy),
                    u64::from(sx) * u64::from(sy),
                );
            }
        }
        f.enc
            .copy_buffer_to_buffer(&self.seca.srgb, 0, &self.adv.rgb, 0, n * 12);
        f.enc
            .copy_buffer_to_buffer(&self.seca.sett, 0, &self.adv.sett, 0, n * 4);
        let (p1, b1) = (&self.entrelaca[1], &self.entrelaca_bg[1]);
        f.passada(p1, b1, Despacho::Direto(gx, gy), ent);
    }
}
