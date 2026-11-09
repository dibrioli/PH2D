//! ⭐ P14 «L» — o primeiro momento `M = [Mx, My]` na placa (`docs/PixelArt/P14_plano.md` §4; o WGSL em
//! `c3_momento.wgsl`). Só existe com receptor de normal à vista (`CenaDeLuz::com_normal`): sem ele nada
//! se aloca nem se despacha, e o quadro é o de sempre. Dois passes depois da C3: a c0 de `M` (nível 0
//! das cascatas, lendo o nível 1 que a c0 leu) e a fina de `M` (o `luz_do_texel` com `MOMENTO`).
//! Fora daqui: a holográfica, o rebote e o miolo (o `M` deles é 0 — a lacuna está no plano §11).
//! Módulo-filho do [`super`] (o tecto de 700 LOC do `solver.rs`).

use super::*;
use crate::passlist::Opcionais;

const FORMATO: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;
const NOMES: [&str; 4] = [
    "luz2d/tex/momento_c0_x",
    "luz2d/tex/momento_c0_y",
    "luz2d/tex/momento_x",
    "luz2d/tex/momento_y",
];

/// O módulo das cascatas (C2, C3 e o momento): a fábrica devolve o MESMO de cada descrição.
pub(super) fn modulo_rc(device: &crate::Placa<'_>, cfg: &Config) -> wgpu::ShaderModule {
    device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("luz2d c2_rc.wgsl"),
        source: wgpu::ShaderSource::Wgsl(
            cfg.modulo_rc
                .unwrap_or_else(|| {
                    shaders::rc(
                        cfg.aritmetica,
                        cfg.armazenamento,
                        cfg.hrc_niveis_em_textura(),
                    )
                })
                .into(),
        ),
    })
}

pub(super) struct Momento {
    c0: wgpu::ComputePipeline,
    c0_grupo: wgpu::BindGroup,
    /// A fina sem e com os distantes.
    fina: [Option<wgpu::ComputePipeline>; 2],
    fina_grupo: wgpu::BindGroup,
    texturas: [wgpu::Texture; 4],
    vistas: [wgpu::TextureView; 4],
}

fn textura(device: &wgpu::Device, nome: &'static str, [w, h]: [u32; 2]) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some(nome),
        size: wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: FORMATO,
        usage: wgpu::TextureUsages::STORAGE_BINDING
            | wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    })
}

fn vista(binding: u32, v: &wgpu::TextureView) -> wgpu::BindGroupEntry<'_> {
    wgpu::BindGroupEntry {
        binding,
        resource: wgpu::BindingResource::TextureView(v),
    }
}

impl Momento {
    /// `None` onde o momento ainda não corre (a holográfica e o entorno `so_a_c0`).
    pub(super) fn novo(s: &Solver, gpu: &GpuContext) -> Option<Self> {
        let cfg = &s.cfg;
        if s.hrc.is_some() || cfg.so_a_c0 {
            return None;
        }
        let device = crate::Placa {
            device: &gpu.device,
            fabrica: &s.fabrica,
            conta: None,
        };
        let modulo = modulo_rc(&device, cfg);
        let grossa = [cfg.grade.largura, cfg.grade.altura];
        let gf = cfg.grade_da_fluencia();
        let fina = [gf.largura, gf.altura];
        let texturas: [wgpu::Texture; 4] = std::array::from_fn(|k| {
            textura(&gpu.device, NOMES[k], if k < 2 { grossa } else { fina })
        });
        let vistas = texturas
            .each_ref()
            .map(|t| t.create_view(&wgpu::TextureViewDescriptor::default()));

        // ── a c0 de M ──
        let g2_c0 = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("luz2d momento c0"),
            entries: &[entrada_escrita(34, FORMATO), entrada_escrita(35, FORMATO)],
        });
        let c0_grupo = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("luz2d momento c0"),
            layout: &g2_c0,
            entries: &[vista(34, &vistas[0]), vista(35, &vistas[1])],
        });
        let rc_layout = s.rc.get_bind_group_layout(0);
        let pipeline = |rotulo: &str,
                        grupos: &[Option<&wgpu::BindGroupLayout>],
                        entrada: &str,
                        constantes: &[(&str, f64)]| {
            let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some(rotulo),
                bind_group_layouts: grupos,
                immediate_size: 0,
            });
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some(rotulo),
                layout: Some(&pl),
                module: &modulo,
                entry_point: Some(entrada),
                compilation_options: wgpu::PipelineCompilationOptions {
                    constants: constantes,
                    zero_initialize_workgroup_memory: true,
                },
                cache: None,
            })
        };
        let c0 = pipeline(
            "luz2d momento c0",
            &[Some(&rc_layout), Some(&s.lampadas_layout), Some(&g2_c0)],
            "cascata_momento",
            &c2::constantes(cfg),
        );

        // ── a fina de M: as leituras da fina de hoje (0, 1, 2, 5) e o momento (30–33) ──
        let op = Opcionais::de(cfg);
        let sementes = SEMENTES[cfg.saltos_jfa().len() % 2];
        let visibilidade = op
            .visibilidade
            .filter(|_| c3::constantes::com_visibilidade(cfg))
            .unwrap_or(sementes);
        let preenche = op.preenche_final(cfg).unwrap_or(sementes);
        let ler = |b: usize| s.texturas[b].create_view(&wgpu::TextureViewDescriptor::default());
        let (c0v, visv, prev) = (
            ler(PassList::indice_da_c0(cfg)),
            ler(visibilidade),
            ler(preenche),
        );
        let g2_fina = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("luz2d momento fina"),
            entries: &[
                entrada_uniforme(0, wgpu::ShaderStages::COMPUTE),
                entrada_textura(1, FLOAT),
                entrada_textura(2, wgpu::TextureSampleType::Uint),
                entrada_textura(5, wgpu::TextureSampleType::Uint),
                entrada_textura(30, FLOAT),
                entrada_textura(31, FLOAT),
                entrada_escrita(32, FORMATO),
                entrada_escrita(33, FORMATO),
            ],
        });
        let fina_grupo = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("luz2d momento fina"),
            layout: &g2_fina,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: s.c3.uniforme.as_entire_binding(),
                },
                vista(1, &c0v),
                vista(2, &visv),
                vista(5, &prev),
                vista(30, &vistas[0]),
                vista(31, &vistas[1]),
                vista(32, &vistas[2]),
                vista(33, &vistas[3]),
            ],
        });
        let (entrada, proprias) = c3::constantes::da_fina(cfg);
        let exata = f64::from(u8::from(entrada == "c3_fina_exata"));
        let grupos = [
            Some(&s.c3.cena_layout),
            Some(&s.lampadas_layout),
            Some(&g2_fina),
        ];
        let fina_com = |distantes: f64| {
            let c: Vec<(&str, f64)> = proprias
                .iter()
                .copied()
                .chain(
                    c3::constantes::comuns(cfg)
                        .into_iter()
                        .filter(|(k, _)| proprias.iter().all(|(o, _)| o != k)),
                )
                .chain([
                    ("DISTANTES", distantes),
                    ("MOMENTO", 1.0),
                    ("MOMENTO_EXATA", exata),
                ])
                .collect();
            pipeline("luz2d momento fina", &grupos, "c3_momento", &c)
        };
        let fina_p = [Some(fina_com(0.0)), cfg.distantes.then(|| fina_com(1.0))];
        Some(Self {
            c0,
            c0_grupo,
            fina: fina_p,
            fina_grupo,
            texturas,
            vistas,
        })
    }

    /// Os dois passes, depois da C3 (a c0 de `M` lê o nível 1 das cascatas, intacto até o quadro seguinte).
    pub(super) fn gravar(&self, s: &Solver, enc: &mut wgpu::CommandEncoder, distantes: bool) {
        let mut p = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("luz2d momento"),
            timestamp_writes: None,
        });
        let [w, h] = [s.cfg.grade.largura, s.cfg.grade.altura];
        p.set_pipeline(&self.c0);
        p.set_bind_group(0, &s.rc_grupos[0], &[]);
        p.set_bind_group(1, &s.lampadas_grupo, &[]);
        p.set_bind_group(2, &self.c0_grupo, &[]);
        p.dispatch_workgroups(w.div_ceil(8), h.div_ceil(8), 1);
        let fina = if distantes { &self.fina[1] } else { &None };
        let fina = fina.as_ref().or(self.fina[0].as_ref()).expect("a fina");
        let gf = s.cfg.grade_da_fluencia();
        p.set_pipeline(fina);
        p.set_bind_group(0, &s.c3.cena, &[]);
        p.set_bind_group(2, &self.fina_grupo, &[]);
        p.dispatch_workgroups(gf.largura.div_ceil(8), gf.altura.div_ceil(8), 1);
    }

    /// `[Mx, My]` na grade da fluência.
    pub(super) fn vistas(&self) -> [&wgpu::TextureView; 2] {
        [&self.vistas[2], &self.vistas[3]]
    }

    /// `[Mx, My]` (as texturas, para quem as lê de volta).
    pub(super) fn texturas(&self) -> [&wgpu::Texture; 2] {
        [&self.texturas[2], &self.texturas[3]]
    }

    /// Os bytes das quatro texturas (a conta da memória).
    pub(super) fn bytes(&self) -> u64 {
        self.texturas
            .iter()
            .map(|t| u64::from(t.width()) * u64::from(t.height()) * 8)
            .sum()
    }
}

impl Solver {
    /// ⭐ P14 «L» — `[Mx, My]` do último quadro, na grade da [`Self::fluencia`]; `None` sem normal.
    #[must_use]
    pub fn momento(&self) -> Option<[&wgpu::TextureView; 2]> {
        self.momento.as_ref().map(Momento::vistas)
    }

    /// [`Self::momento`] como texturas (para lê-las de volta).
    #[must_use]
    pub fn momento_texturas(&self) -> Option<[&wgpu::Texture; 2]> {
        self.momento.as_ref().map(Momento::texturas)
    }

    /// Os bytes do momento residente (0 sem normal à vista).
    #[must_use]
    pub fn bytes_do_momento(&self) -> u64 {
        self.momento.as_ref().map_or(0, Momento::bytes)
    }

    /// Liga o momento quando a cena tem normal à vista e solta-o quando não tem.
    pub(super) fn acertar_o_momento(&mut self, gpu: &GpuContext, com_normal: bool) {
        if !com_normal {
            self.momento = None;
        } else if self.momento.is_none() {
            self.momento = Momento::novo(self, gpu);
        }
    }
}
