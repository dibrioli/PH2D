//! ⭐ **P14 — o Sprite do motor com normal map** (`docs/PixelArt/P14_plano.md` §4 «L»): uma SEGUNDA
//! textura por textura individual da cor, com o MESMO tamanho — logo o mesmo `atlas_uv` lê a mesma
//! célula da folha ou o mesmo quadro do Animated Sprite (o idioma da Unity: a textura secundária
//! partilha o recorte). O `fs_main_n` do `sprite.wgsl` lê-a, roda-a pela base da instância e entrega-a
//! a `luz_receber_n`.
//!
//! ⛔ Sem normal registada NADA é alocado: nem layout, nem pipelines, nem textura — o desenho segue
//! pela pipeline de sempre, byte a byte (gate `sem_normal_o_sprite_e_o_de_hoje_byte_a_byte`).
//! A normal é `Rgba8Unorm` LINEAR, sem mips (uma média de normais não é uma normal).

use std::collections::BTreeMap;

use ph2d_gpu::GpuContext;

use crate::pipeline::{BLEND_PIPELINE_COUNT, SpritePipeline};

/// Uma normal: a textura, a cor contra a qual o grupo foi feito, e os grupos por amostragem.
struct Entrada {
    vista: wgpu::TextureView,
    /// A textura da COR quando o grupo nasceu: um id reciclado (outra cor) invalida-o.
    cor: wgpu::Texture,
    grupos: BTreeMap<u32, wgpu::BindGroup>,
}

/// Layout de material com 3 entradas e uma pipeline `fs_main_n` por mistura — feitos no 1.º registo.
struct Placa {
    bgl: wgpu::BindGroupLayout,
    pipelines: [wgpu::RenderPipeline; BLEND_PIPELINE_COUNT],
}

/// A normal do ÁTLAS: uma textura do tamanho do átlas da cor, cada célula com normal no MESMO
/// retângulo da célula da cor (o resto alfa 0 = sem normal ⇒ `n = 0`, a luz de hoje).
struct NormalDoAtlas {
    vista: wgpu::TextureView,
    /// A textura do átlas da cor quando esta nasceu: crescer o átlas refaz esta a partir da CPU.
    cor: wgpu::Texture,
    grupos: BTreeMap<u32, wgpu::BindGroup>,
}

/// As normais dos sprites, por `texture_id` da COR (individuais) e por chave de célula (átlas).
pub(crate) struct NormaisDosSprites {
    color_format: wgpu::TextureFormat,
    placa: Option<Placa>,
    mapa: BTreeMap<u32, Entrada>,
    /// As normais das células do átlas, guardadas na CPU para refazer a textura quando o átlas cresce.
    celulas: BTreeMap<u32, (u32, u32, Vec<u8>)>,
    atlas: Option<NormalDoAtlas>,
}

impl NormaisDosSprites {
    pub(crate) fn nova(color_format: wgpu::TextureFormat) -> Self {
        Self {
            color_format,
            placa: None,
            mapa: BTreeMap::new(),
            celulas: BTreeMap::new(),
            atlas: None,
        }
    }

    /// Texturas individuais com normal + células do átlas com normal.
    pub(crate) fn len(&self) -> usize {
        self.mapa.len() + self.celulas.len()
    }

    pub(crate) fn tem(&self, texture_id: u32) -> bool {
        self.mapa.contains_key(&texture_id)
    }

    pub(crate) fn bytes_da_celula(&self, key: u32) -> Option<&[u8]> {
        self.celulas.get(&key).map(|(_, _, b)| b.as_slice())
    }

    pub(crate) fn tem_celula(&self, key: u32) -> bool {
        self.celulas.contains_key(&key)
    }

    /// Regista (`Some`) ou tira (`None`) a normal da célula `key` do átlas; tamanho = o da célula.
    pub(crate) fn definir_celula(
        &mut self,
        gpu: &GpuContext,
        p: &SpritePipeline,
        atlas: &crate::TextureAtlas,
        key: u32,
        normal: Option<(u32, u32, &[u8])>,
    ) -> Result<(), String> {
        let Some((w, h, rgba)) = normal else {
            self.celulas.remove(&key);
            self.atlas = None;
            return Ok(());
        };
        let Some(dims) = atlas.region_px(key) else {
            return Err(format!("célula {key} desconhecida no átlas"));
        };
        if dims != (w, h) || rgba.len() != (w * h * 4) as usize {
            return Err(format!(
                "a normal {w}×{h} não tem o tamanho da célula {}×{} (o recorte seria outro)",
                dims.0, dims.1
            ));
        }
        self.placa(gpu, p);
        self.celulas.insert(key, (w, h, rgba.to_vec()));
        // A textura refaz-se no próximo `garantir_atlas` com todas as células.
        self.atlas = None;
        Ok(())
    }

    /// Garante a textura normal do átlas e o grupo de `sampling` antes do passe.
    pub(crate) fn garantir_atlas(
        &mut self,
        gpu: &GpuContext,
        atlas: &crate::TextureAtlas,
        sampling: u32,
    ) {
        if self.celulas.is_empty() {
            self.atlas = None;
            return;
        }
        let Some(placa) = self.placa.as_ref() else {
            return;
        };
        if self.atlas.as_ref().is_some_and(|a| a.cor != atlas.texture) {
            self.atlas = None;
        }
        let na = self.atlas.get_or_insert_with(|| {
            let lado = atlas.size_px;
            let t = gpu.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("ph2d-render sprite normal do atlas"),
                size: wgpu::Extent3d {
                    width: lado,
                    height: lado,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });
            for (key, (w, h, rgba)) in &self.celulas {
                let Some(r) = atlas.region(*key).filter(|r| (r.w, r.h) == (*w, *h)) else {
                    continue;
                };
                gpu.queue.write_texture(
                    wgpu::TexelCopyTextureInfo {
                        texture: &t,
                        mip_level: 0,
                        origin: wgpu::Origin3d {
                            x: r.x,
                            y: r.y,
                            z: 0,
                        },
                        aspect: wgpu::TextureAspect::All,
                    },
                    rgba,
                    wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(w * 4),
                        rows_per_image: Some(*h),
                    },
                    wgpu::Extent3d {
                        width: *w,
                        height: *h,
                        depth_or_array_layers: 1,
                    },
                );
            }
            NormalDoAtlas {
                vista: t.create_view(&Default::default()),
                cor: atlas.texture.clone(),
                grupos: BTreeMap::new(),
            }
        });
        if na.grupos.contains_key(&sampling) {
            return;
        }
        let amostra = (sampling != 0).then(|| {
            let (filter, repeat) = crate::RenderInstance::unpack_sampling(sampling);
            crate::image_filter::sampler_from_tags(&gpu.device, filter, repeat)
        });
        let bg = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ph2d-render sprite material com normal (atlas)"),
            layout: &placa.bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&atlas.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(
                        amostra.as_ref().unwrap_or(&atlas.sampler),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&na.vista),
                },
            ],
        });
        na.grupos.insert(sampling, bg);
    }

    fn placa(&mut self, gpu: &GpuContext, p: &SpritePipeline) -> &Placa {
        let fmt = self.color_format;
        self.placa.get_or_insert_with(|| {
            let mut entradas = crate::pipeline::entradas_do_material().to_vec();
            entradas.push(wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            });
            let bgl = gpu
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("ph2d-render sprite material com normal bgl"),
                    entries: &entradas,
                });
            let pipelines = p.pipelines_com_normal(gpu, &bgl, fmt);
            Placa { bgl, pipelines }
        })
    }

    /// Regista (`Some`) ou tira (`None`) a normal de `cor_id`. A normal tem de ter o tamanho da cor.
    pub(crate) fn definir(
        &mut self,
        gpu: &GpuContext,
        p: &SpritePipeline,
        cor: Option<(&wgpu::Texture, (u32, u32))>,
        cor_id: u32,
        normal: Option<(u32, u32, &[u8])>,
    ) -> Result<(), String> {
        let Some((w, h, rgba)) = normal else {
            self.mapa.remove(&cor_id);
            return Ok(());
        };
        let Some((tex_cor, dims)) = cor else {
            return Err(format!("textura {cor_id} desconhecida"));
        };
        if dims != (w, h) || rgba.len() != (w * h * 4) as usize {
            return Err(format!(
                "a normal {w}×{h} não tem o tamanho da cor {}×{} (o recorte seria outro)",
                dims.0, dims.1
            ));
        }
        self.placa(gpu, p);
        let t = gpu.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("ph2d-render sprite normal"),
            size: wgpu::Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        gpu.queue.write_texture(
            t.as_image_copy(),
            rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(w * 4),
                rows_per_image: Some(h),
            },
            t.size(),
        );
        self.mapa.insert(
            cor_id,
            Entrada {
                vista: t.create_view(&Default::default()),
                cor: tex_cor.clone(),
                grupos: BTreeMap::new(),
            },
        );
        Ok(())
    }

    /// Garante o grupo `(cor, amostragem, normal)` antes do passe; tira a entrada cuja cor morreu.
    pub(crate) fn garantir(
        &mut self,
        gpu: &GpuContext,
        individual: &crate::individual::IndividualTextureStore,
        cor_id: u32,
        sampling: u32,
    ) {
        let Some(e) = self.mapa.get(&cor_id) else {
            return;
        };
        let Some(viva) = individual.entries.get(&cor_id) else {
            self.mapa.remove(&cor_id);
            return;
        };
        if viva.texture != e.cor {
            self.mapa.remove(&cor_id);
            return;
        }
        if e.grupos.contains_key(&sampling) {
            return;
        }
        let Some(placa) = self.placa.as_ref() else {
            return;
        };
        let amostra = if sampling == 0 {
            None
        } else {
            let (filter, repeat) = crate::RenderInstance::unpack_sampling(sampling);
            Some(crate::image_filter::sampler_from_tags(
                &gpu.device,
                filter,
                repeat,
            ))
        };
        let sampler = amostra.as_ref().unwrap_or(&individual.sampler);
        let bg = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ph2d-render sprite material com normal"),
            layout: &placa.bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&viva.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&e.vista),
                },
            ],
        });
        if let Some(e) = self.mapa.get_mut(&cor_id) {
            e.grupos.insert(sampling, bg);
        }
    }

    /// A pipeline e o grupo de um run COM normal, ou `None` (o run segue pela pipeline de sempre).
    pub(crate) fn do_run(
        &self,
        cor_id: u32,
        sampling: u32,
        blend: u8,
    ) -> Option<(&wgpu::RenderPipeline, &wgpu::BindGroup)> {
        let placa = self.placa.as_ref()?;
        let bg = if cor_id == crate::RenderInstance::ATLAS_TEXTURE_ID {
            self.atlas.as_ref()?.grupos.get(&sampling)?
        } else {
            self.mapa.get(&cor_id)?.grupos.get(&sampling)?
        };
        let pl = placa
            .pipelines
            .get(blend as usize)
            .unwrap_or(&placa.pipelines[0]);
        Some((pl, bg))
    }
}
