//! **C3 na grade FINA** (W4b) — os dois passes depois das cascatas: a visibilidade das lâmpadas
//! diretas nas sondas da c0 e a grade fina (a c0 interpolada + a luz direta), com os grupos deles.
//! Módulo-filho do [`super`] (vê os auxiliares dele sem os abrir).

use super::*;
use crate::config::Refino;
use crate::passlist::{FLUENCIA, VISIBILIDADE};

/// O espelho de `Fina` do `c3_direta.wgsl` (48 B).
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct UniformeFina {
    tamanho: [u32; 2],
    inicio: [u32; 2],
    fim: [u32; 2],
    inicio_grosso: [u32; 2],
    fim_grosso: [u32; 2],
    fina: u32,
    limiar: f32,
}

/// O que os passes C3 pegam do solver ao nascer.
pub(super) struct Entradas<'a> {
    pub gpu: &'a GpuContext,
    pub cfg: &'a Config,
    pub lista: &'a PassList,
    pub vistas: &'a [wgpu::TextureView],
    pub modulo: &'a wgpu::ShaderModule,
    pub lampadas_layout: &'a wgpu::BindGroupLayout,
    pub uni_cascatas: &'a wgpu::Buffer,
    pub uni_c3: &'a wgpu::Buffer,
    pub sementes_finais: usize,
    pub vizinhos: &'a wgpu::Sampler,
}

pub(super) struct C3 {
    /// O grupo 0: a cena capturada e o uniforme da c0 (a marcha da c0, sem as saídas das cascatas).
    cena: wgpu::BindGroup,
    visibilidade: Option<(wgpu::ComputePipeline, wgpu::BindGroup)>,
    fina: wgpu::ComputePipeline,
    grupo_fina: wgpu::BindGroup,
    uniforme: wgpu::Buffer,
    /// O uniforme deste quadro: a grade fina inteira por omissão, a região da janela com
    /// [`Self::regiao`].
    u: UniformeFina,
}

fn layout(
    device: &wgpu::Device,
    rotulo: &str,
    entradas: &[wgpu::BindGroupLayoutEntry],
) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some(rotulo),
        entries: entradas,
    })
}

fn vista(binding: u32, v: &wgpu::TextureView) -> wgpu::BindGroupEntry<'_> {
    wgpu::BindGroupEntry {
        binding,
        resource: wgpu::BindingResource::TextureView(v),
    }
}

impl C3 {
    pub(super) fn novo(e: &Entradas<'_>) -> Self {
        let device = &e.gpu.device;
        let cfg = e.cfg;
        let cena_layout = layout(
            device,
            "luz2d c3 cena",
            &[
                entrada_uniforme(0, wgpu::ShaderStages::COMPUTE),
                entrada_textura(1, FLOAT),
                entrada_textura(2, wgpu::TextureSampleType::Uint),
                entrada_textura(5, FLOAT),
                entrada_textura(6, FLOAT),
                wgpu::BindGroupLayoutEntry {
                    binding: 7,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                    count: None,
                },
            ],
        );
        let cena = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("luz2d c3 cena"),
            layout: &cena_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: fatia_uniforme(
                        e.uni_cascatas,
                        0,
                        size_of::<UniformeCascata>() as u64,
                    ),
                },
                vista(1, &e.vistas[EMISSAO]),
                vista(2, &e.vistas[e.sementes_finais]),
                vista(5, &e.vistas[COBERTURA]),
                vista(6, &e.vistas[TAU_DENTRO]),
                wgpu::BindGroupEntry {
                    binding: 7,
                    resource: wgpu::BindingResource::Sampler(e.vizinhos),
                },
            ],
        });
        let fino = |passo_fino: bool| {
            f64::from(cfg.passo_perto) / if passo_fino { f64::from(cfg.fina) } else { 1.0 }
        };
        let bordas = match cfg.refino {
            Refino::Bordas(b) => Some(b),
            Refino::Tudo => None,
        };
        let pipeline = |rotulo: &str,
                        g2: &wgpu::BindGroupLayout,
                        entrada: &str,
                        constantes: &[(&str, f64)]| {
            let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some(rotulo),
                bind_group_layouts: &[Some(&cena_layout), Some(e.lampadas_layout), Some(g2)],
                immediate_size: 0,
            });
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some(rotulo),
                layout: Some(&pl),
                module: e.modulo,
                entry_point: Some(entrada),
                compilation_options: wgpu::PipelineCompilationOptions {
                    constants: constantes,
                    zero_initialize_workgroup_memory: true,
                },
                cache: None,
            })
        };

        let visibilidade = bordas.filter(|_| cfg.usa_visibilidade()).map(|b| {
            let g2 = layout(
                device,
                "luz2d c3 visibilidade",
                &[
                    entrada_uniforme(0, wgpu::ShaderStages::COMPUTE),
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::StorageTexture {
                            access: wgpu::StorageTextureAccess::WriteOnly,
                            format: wgpu::TextureFormat::Rg32Uint,
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        count: None,
                    },
                ],
            );
            let grupo = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("luz2d c3 visibilidade"),
                layout: &g2,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: e.uni_c3.as_entire_binding(),
                    },
                    vista(3, &e.vistas[VISIBILIDADE]),
                ],
            });
            let constantes = [
                ("PASSO", fino(b.visibilidade_no_passo_fino)),
                ("CERTIFICAR", f64::from(u8::from(b.certificar))),
                ("RAIOS_DIRETOS", f64::from(b.raios)),
                ("BISSECOES", f64::from(b.bissecoes)),
            ];
            (
                pipeline("luz2d c3 visibilidade", &g2, "c3_visibilidade", &constantes),
                grupo,
            )
        });

        let c0 = PassList::indice_da_c0(cfg);
        let formato = e.lista.texturas[FLUENCIA].formato;
        let mut entradas = vec![
            entrada_uniforme(0, wgpu::ShaderStages::COMPUTE),
            entrada_textura(1, FLOAT),
            entrada_escrita(4, formato),
        ];
        let mut grupo = vec![
            wgpu::BindGroupEntry {
                binding: 0,
                resource: e.uni_c3.as_entire_binding(),
            },
            vista(1, &e.vistas[c0]),
            vista(4, &e.vistas[FLUENCIA]),
        ];
        if visibilidade.is_some() {
            entradas.push(entrada_textura(2, wgpu::TextureSampleType::Uint));
            grupo.push(vista(2, &e.vistas[VISIBILIDADE]));
        }
        let g2 = layout(device, "luz2d c3 fina", &entradas);
        let grupo_fina = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("luz2d c3 fina"),
            layout: &g2,
            entries: &grupo,
        });
        // ⭐ Os raios da luz direta marcham no passo da grade FINA: com o das cascatas as amostras
        // entram e saem do canto do oclusor conforme o texel fino anda, e a borda ondula (medido na
        // W4b, Low: 0,53 → 0,06 px no contorno de 50 %).
        let fina = match bordas.filter(|_| visibilidade.is_some()) {
            Some(b) => pipeline(
                "luz2d c3 fina",
                &g2,
                "c3_fina",
                &[
                    ("PASSO", fino(true)),
                    ("RAIOS_DIRETOS", f64::from(b.raios)),
                    ("BISSECOES", f64::from(b.bissecoes)),
                    ("JOELHO", f64::from(u8::from(b.joelho))),
                ],
            ),
            None => pipeline(
                "luz2d c3 fina",
                &g2,
                "c3_fina_exata",
                &[("PASSO", fino(true))],
            ),
        };

        let gf = cfg.grade_da_fluencia();
        let mut c3 = Self {
            cena,
            visibilidade,
            fina,
            grupo_fina,
            uniforme: e.uni_c3.clone(),
            u: UniformeFina {
                tamanho: [gf.largura, gf.altura],
                inicio: [0; 2],
                fim: [0; 2],
                inicio_grosso: [0; 2],
                fim_grosso: [0; 2],
                fina: cfg.fina,
                limiar: bordas.map_or(0.0, |b| b.limiar),
            },
        };
        c3.regiao(cfg, [0; 2], [gf.largura, gf.altura]);
        c3
    }

    /// ⭐ Só a REGIÃO `[inicio, fim)` da grade fina se calcula (a da janela; o resto da grade
    /// existe para as cascatas verem o que está fora dela), e as sondas da c0 que a cobrem.
    pub(super) fn regiao(&mut self, cfg: &Config, inicio: [u32; 2], fim: [u32; 2]) {
        let u = &mut self.u;
        let grossa = [cfg.grade.largura, cfg.grade.altura];
        for k in 0..2 {
            u.fim[k] = fim[k].min(u.tamanho[k]);
            u.inicio[k] = inicio[k].min(u.fim[k]);
            u.inicio_grosso[k] = (u.inicio[k] / cfg.fina).saturating_sub(1);
            u.fim_grosso[k] = (u.fim[k].div_ceil(cfg.fina) + 1).min(grossa[k]);
        }
    }

    /// Grava os dois passes (o da visibilidade fica vazio sem ela: os carimbos dele valem 0).
    pub(super) fn gravar(
        &self,
        gpu: &GpuContext,
        enc: &mut wgpu::CommandEncoder,
        lampadas: &wgpu::BindGroup,
        tempos: Option<&Tempos>,
    ) {
        gpu.queue
            .write_buffer(&self.uniforme, 0, bytemuck::bytes_of(&self.u));
        let grupos =
            |a: [u32; 2], b: [u32; 2]| [(b[0] - a[0]).div_ceil(8), (b[1] - a[1]).div_ceil(8)];
        let grossa = grupos(self.u.inicio_grosso, self.u.fim_grosso);
        let fina = grupos(self.u.inicio, self.u.fim);
        {
            let mut p = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("luz2d C3 visibilidade"),
                timestamp_writes: tempos.map(|t| t.escrita_compute(tempos::C3_VISIBILIDADE)),
            });
            if let Some((pipe, g2)) = &self.visibilidade {
                p.set_pipeline(pipe);
                p.set_bind_group(0, &self.cena, &[]);
                p.set_bind_group(1, lampadas, &[]);
                p.set_bind_group(2, g2, &[]);
                p.dispatch_workgroups(grossa[0], grossa[1], 1);
            }
        }
        let mut p = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("luz2d C3 fina"),
            timestamp_writes: tempos.map(|t| t.escrita_compute(tempos::C3_FINA)),
        });
        p.set_pipeline(&self.fina);
        p.set_bind_group(0, &self.cena, &[]);
        p.set_bind_group(1, lampadas, &[]);
        p.set_bind_group(2, &self.grupo_fina, &[]);
        p.dispatch_workgroups(fina[0], fina[1], 1);
    }
}
