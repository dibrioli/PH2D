//! ⭐ W5h — **o miolo dos opacos pela DIFUSÃO** na placa (`c3_miolo.wgsl`): depois da `c3_fina`, a
//! pirâmide (classifica → puxa até ao nível mais grosso → relaxa e desce até à fina), num passe de
//! cômputo com o carimbo `C3_MIOLO`. Módulo-filho do [`super`].

use super::*;
use crate::config::Miolo;
use crate::passlist::{FLUENCIA, Opcionais};

/// O espelho de `Nivel` do `c3_miolo.wgsl` (48 B).
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct UniformeNivel {
    inicio: [u32; 2],
    fim: [u32; 2],
    outro_inicio: [u32; 2],
    outro_fim: [u32; 2],
    kh2: f32,
    suavidade: f32,
    _p: [f32; 2],
}

#[derive(Clone, Copy)]
enum Tipo {
    Classifica,
    Puxa,
    Inicia,
    Relaxa,
    Desce,
    Final,
}

struct Passo {
    tipo: Tipo,
    nivel: u32,
    grupo: wgpu::BindGroup,
}

pub(super) struct DoMiolo {
    pipelines: [wgpu::ComputePipeline; 6],
    passos: Vec<Passo>,
    uniforme: wgpu::Buffer,
    niveis: u32,
    miolo: Miolo,
    /// O tamanho da grade fina e a região dela neste quadro.
    tamanho: [u32; 2],
    regiao: ([u32; 2], [u32; 2]),
}

pub(super) struct Entradas<'a> {
    pub placa: crate::Placa<'a>,
    pub cfg: &'a Config,
    pub texturas: &'a [wgpu::Texture],
    pub vistas: &'a [wgpu::TextureView],
    pub uniforme: wgpu::Buffer,
}

fn mip(t: &wgpu::Texture, nivel: u32) -> wgpu::TextureView {
    t.create_view(&wgpu::TextureViewDescriptor {
        base_mip_level: nivel,
        mip_level_count: Some(1),
        ..Default::default()
    })
}

impl DoMiolo {
    pub(super) fn novo(e: &Entradas<'_>) -> Option<Self> {
        let cfg = e.cfg;
        let miolo = cfg.miolo?;
        let niveis = cfg.niveis_do_miolo()?;
        let [i0, ik, ia, ib] = Opcionais::de(cfg).miolo?;
        let device = e.placa;
        // Um módulo e um layout por formato da `saida` (o WGSL fixa-o na declaração).
        let por_formato = |formato: wgpu::TextureFormat| {
            let nome = match formato {
                wgpu::TextureFormat::Rgba32Float => "rgba32float",
                _ => "rgba16float",
            };
            let modulo = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("luz2d c3 miolo"),
                source: wgpu::ShaderSource::Wgsl(shaders::C3_MIOLO.replace("FORMATO", nome).into()),
            });
            let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("luz2d c3 miolo"),
                entries: &[
                    entrada_uniforme(0, wgpu::ShaderStages::COMPUTE),
                    entrada_textura(1, FLOAT),
                    entrada_textura(2, FLOAT),
                    entrada_textura(3, FLOAT),
                    entrada_escrita(4, formato),
                ],
            });
            let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("luz2d c3 miolo"),
                bind_group_layouts: &[Some(&layout)],
                immediate_size: 0,
            });
            (modulo, layout, pl)
        };
        let fluencia = e.texturas[FLUENCIA].format();
        let formatos = [
            fluencia,
            wgpu::TextureFormat::Rgba16Float,
            wgpu::TextureFormat::Rgba32Float,
        ];
        let mut feitos: Vec<(wgpu::TextureFormat, _)> = Vec::new();
        for f in formatos {
            if !feitos.iter().any(|(g, _)| *g == f) {
                feitos.push((f, por_formato(f)));
            }
        }
        let de = |f: wgpu::TextureFormat| &feitos.iter().find(|(g, _)| *g == f).expect("formato").1;
        let pipeline = |entrada: &str, f: wgpu::TextureFormat| {
            let (modulo, _, pl) = de(f);
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some(entrada),
                layout: Some(pl),
                module: modulo,
                entry_point: Some(entrada),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                cache: None,
            })
        };
        let (f16, f32) = (wgpu::TextureFormat::Rgba16Float, wgpu::TextureFormat::Rgba32Float);
        let pipelines = [
            pipeline("miolo_classifica", fluencia),
            pipeline("miolo_puxa", f16),
            pipeline("miolo_inicia", f32),
            pipeline("miolo_relaxa", f32),
            pipeline("miolo_desce", f32),
            pipeline("miolo_final", fluencia),
        ];
        let saida_de = |t: Tipo| match t {
            Tipo::Classifica | Tipo::Final => fluencia,
            Tipo::Puxa => f16,
            _ => f32,
        };
        // As pirâmides começam no nível 1: o mip `l − 1` é o nível `l`.
        let piramide = |t: usize| -> Vec<wgpu::TextureView> {
            (1..=niveis).map(|l| mip(&e.texturas[t], l - 1)).collect()
        };
        let (kv, av, bv) = (piramide(ik), piramide(ia), piramide(ib));
        let k0 = &e.vistas[i0];
        let fl = &e.vistas[FLUENCIA];
        // O conhecido do nível `l` (0 = o da fina).
        let k = |l: u32| if l == 0 { k0 } else { &kv[l as usize - 1] };
        let a = |l: u32| &av[l as usize - 1];
        let b = |l: u32| &bv[l as usize - 1];
        let tam = size_of::<UniformeNivel>() as u64;
        // As fatias: `2l` lê a região do nível de BAIXO no par «outro» (puxa); `2l + 1`, a de cima.
        let grupo = |tipo: Tipo, fatia: u64, v: [&wgpu::TextureView; 4]| {
            let (_, layout, _) = de(saida_de(tipo));
            let entrada = |binding, vista| wgpu::BindGroupEntry {
                binding,
                resource: wgpu::BindingResource::TextureView(vista),
            };
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("luz2d c3 miolo"),
                layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: fatia_uniforme(&e.uniforme, fatia, tam),
                    },
                    entrada(1, v[0]),
                    entrada(2, v[1]),
                    entrada(3, v[2]),
                    entrada(4, v[3]),
                ],
            })
        };
        let resto = |l: u32| 2 * u64::from(l) + 1;
        let mut passos = Vec::new();
        let mut por = |tipo: Tipo, nivel: u32, v: [&wgpu::TextureView; 4]| {
            let fatia = if matches!(tipo, Tipo::Puxa) {
                2 * u64::from(nivel)
            } else {
                resto(nivel)
            };
            passos.push(Passo {
                tipo,
                nivel,
                grupo: grupo(tipo, fatia, v),
            });
        };
        por(Tipo::Classifica, 0, [fl, a(1), a(1), k0]);
        for l in 1..=niveis {
            por(Tipo::Puxa, l, [k(l - 1), a(1), a(1), k(l)]);
        }
        // Um par de relaxações no nível `l` (a → b → a): o resultado fica na `a`.
        let no_nivel = |l: u32| {
            let n = miolo
                .relaxa
                .saturating_mul(1 << (l - 1).min(16))
                .min(miolo.relaxa_max.max(miolo.relaxa));
            if l == niveis { 2 * n } else { n }
        };
        let pares = |l: u32| no_nivel(l).div_ceil(2);
        por(Tipo::Inicia, niveis, [k(niveis - 1), k(niveis), k(niveis), a(niveis)]);
        for l in (1..=niveis).rev() {
            if l < niveis {
                por(Tipo::Desce, l, [a(l + 1), k(l), k(l + 1), a(l)]);
            }
            for _ in 0..pares(l) {
                por(Tipo::Relaxa, l, [a(l), k(l), k(l), b(l)]);
                por(Tipo::Relaxa, l, [b(l), k(l), k(l), a(l)]);
            }
        }
        por(Tipo::Final, 0, [a(1), k0, k(1), fl]);
        let gf = cfg.grade_da_fluencia();
        Some(Self {
            pipelines,
            passos,
            uniforme: e.uniforme.clone(),
            niveis,
            miolo,
            tamanho: [gf.largura, gf.altura],
            regiao: ([0; 2], [gf.largura, gf.altura]),
        })
    }

    pub(super) fn regiao(&mut self, inicio: [u32; 2], fim: [u32; 2]) {
        let fim = [0, 1].map(|k| fim[k].min(self.tamanho[k]));
        self.regiao = ([0, 1].map(|k| inicio[k].min(fim[k])), fim);
    }

    /// A região do nível `l`: a da fina reduzida, arredondada para fora.
    fn regiao_em(&self, l: u32) -> ([u32; 2], [u32; 2]) {
        let (i, f) = self.regiao;
        (i.map(|x| x >> l), f.map(|x| x.div_ceil(1 << l)))
    }

    /// `texel_m`: o lado de um texel da grade fina neste quadro (o `reposicionar` muda-o).
    pub(super) fn gravar(
        &self,
        gpu: &GpuContext,
        enc: &mut wgpu::CommandEncoder,
        texel_m: f64,
        tempos: Option<&Tempos>,
    ) {
        let mut uni = vec![0u8; (PASSO_UNIFORME * 2 * u64::from(self.niveis + 1)) as usize];
        let pen = f64::from(self.miolo.penetracao_m);
        for l in 0..=self.niveis {
            let (inicio, fim) = self.regiao_em(l);
            let h = texel_m * f64::from(1u32 << l);
            let kh2 = if pen.is_finite() && pen > 0.0 {
                (h / pen).powi(2) as f32
            } else {
                0.0
            };
            // A fatia `2l` lê o nível de baixo (puxa); a `2l + 1`, o de cima (desce).
            for (fatia, outro) in [(2 * l, l.saturating_sub(1)), (2 * l + 1, l + 1)] {
                let (outro_inicio, outro_fim) = self.regiao_em(outro);
                let u = UniformeNivel {
                    inicio,
                    fim,
                    outro_inicio,
                    outro_fim,
                    kh2,
                    suavidade: self.miolo.suavidade.clamp(0.0, 1.0),
                    _p: [0.0; 2],
                };
                let o = fatia as usize * PASSO_UNIFORME as usize;
                uni[o..o + size_of::<UniformeNivel>()].copy_from_slice(bytemuck::bytes_of(&u));
            }
        }
        gpu.queue.write_buffer(&self.uniforme, 0, &uni);
        let mut p = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("luz2d C3 miolo"),
            timestamp_writes: tempos.map(|t| t.escrita_compute(tempos::C3_MIOLO)),
        });
        for passo in &self.passos {
            let (inicio, fim) = self.regiao_em(passo.nivel);
            let w = fim[0].saturating_sub(inicio[0]);
            let h = fim[1].saturating_sub(inicio[1]);
            if w == 0 || h == 0 {
                continue;
            }
            let i = match passo.tipo {
                Tipo::Classifica => 0,
                Tipo::Puxa => 1,
                Tipo::Inicia => 2,
                Tipo::Relaxa => 3,
                Tipo::Desce => 4,
                Tipo::Final => 5,
            };
            p.set_pipeline(&self.pipelines[i]);
            p.set_bind_group(0, &passo.grupo, &[]);
            p.dispatch_workgroups(w.div_ceil(8), h.div_ceil(8), 1);
        }
    }
}
