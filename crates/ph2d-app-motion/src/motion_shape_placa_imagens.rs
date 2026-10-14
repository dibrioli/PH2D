//! ⭐⭐ **AS IMAGENS DE UM GRUPO NA PLACA DE FORMAS** (P4, doc 132 §4 do Motion) — o segundo
//! desenhador das corridas ([`ph2d_shape_gpu::ShapePass::draw_em_corridas_com`]): as linhas de imagem
//! que uma saída em grupo entregou à placa ([`ph2d_gpu_cook::GpuImagensDoGrupo`]) pintadas na MESMA
//! camada, na MESMA corrida e com a MESMA mistura que as formas, pela conta da cena vectorial da CPU
//! (o WGSL irmão diz qual, linha a linha).
//!
//! ⚠️ **Numa corrida de Soma, Tela ou Multiplica as imagens desenham-se depois das formas** — a ordem
//! das linhas não importa, as misturas são comutativas e associativas. A corrida `Normal` com as duas
//! médias (o `Scene`, [`Corrida::ordenada`]) desenha-as NUM desenho, na ordem das linhas: este WGSL é
//! o SEGUNDO MEIO do passe de formas ([`ph2d_shape_gpu::ShapePass::com_segundo_meio`], doc 132 §4.1 (3)).

use std::ops::Range;

use ph2d_gpu::GpuContext;
use ph2d_shape_gpu::{Corrida, Mistura, ShapeView};

/// O uniforme do passe: a vista da camada e a tabela das tags de filtro que ampliam por PONTO.
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vista {
    lin: [f32; 4],
    t: [f32; 2],
    alvo: [f32; 2],
    ponto: [u32; 8],
    mudo: u32,
    _p: [u32; 3],
}

/// **A tabela do filtro**: o bit `tag` = a tag amplia por ponto — a MESMA pergunta que a rota da CPU
/// faz a cada quad ([`crate::motion_shape_gen::mistura::qualidade_da_imagem`]), com a tag `0` a herdar
/// o filtro do projecto.
#[must_use]
pub(crate) fn tabela_do_ponto(projeto: ph2d_render::ImageFilterMode) -> [u32; 8] {
    let mut bits = [0u32; 8];
    for tag in 0u32..256 {
        if crate::motion_shape_gen::mistura::qualidade_da_imagem(tag, projeto)
            == ph2d_vector::ImageQuality::Low
        {
            bits[(tag / 32) as usize] |= 1 << (tag % 32);
        }
    }
    bits
}

/// O MEIO das imagens com o que ele pede: a `codifica`, as palavras do [`ph2d_render::RenderInstance`]
/// (contadas pelo `offset_of!`, para o WGSL nunca ler um campo que mudou de sítio) e os bindings no
/// `grupo` (`0` no pipeline das imagens sozinhas, `2` no desenho ordenado do passe de formas).
fn meio(grupo: u32) -> String {
    use ph2d_render::RenderInstance as R;
    use std::mem::{offset_of, size_of};
    let w = |o: usize| o / 4;
    format!(
        "{codifica}\nconst PALAVRAS: u32 = {n}u;\nconst W_P: u32 = {p}u;\nconst W_SIZE: u32 = {s}u;\n\
         const W_UV: u32 = {uv}u;\nconst W_TINT: u32 = {tint}u;\nconst W_BASIS: u32 = {b}u;\n\
         const W_PREMUL: u32 = {pm}u;\nconst W_ANCHOR: u32 = {a}u;\nconst W_UV_XFORM: u32 = {x}u;\n\
         const W_SAMPLING: u32 = {smp}u;\n\
         @group({grupo}) @binding(0) var<uniform> vista: Vista;\n\
         @group({grupo}) @binding(1) var<storage, read> linhas_img: array<u32>;\n\
         @group({grupo}) @binding(2) var arte: texture_2d<f32>;\n{corpo}",
        corpo = include_str!("motion_shape_placa_imagens_meio.wgsl"),
        codifica = ph2d_render::CODIFICA_WGSL,
        n = size_of::<R>() / 4,
        p = w(offset_of!(R, world_pos)),
        s = w(offset_of!(R, size)),
        uv = w(offset_of!(R, atlas_uv)),
        tint = w(offset_of!(R, tint)),
        b = w(offset_of!(R, basis)),
        pm = w(offset_of!(R, premultiplied)),
        a = w(offset_of!(R, anchor)),
        x = w(offset_of!(R, uv_xform)),
        smp = w(offset_of!(R, sampling)),
    )
}

/// O shader inteiro (o prefixo e o corpo), para os pipelines e para o gate que o valida.
#[must_use]
pub(crate) fn modulo(multiplica: bool) -> String {
    let mut s = String::new();
    if multiplica {
        s.push_str("enable dual_source_blending;\n");
    }
    s.push_str(ph2d_shape_gpu::AREA_DO_VELLO);
    s.push_str(&meio(0));
    s.push_str(include_str!("motion_shape_placa_imagens.wgsl"));
    if multiplica {
        s.push_str(
            "struct DuasSaidas {\n    @location(0) @blend_src(0) cor: vec4<f32>,\n    \
             @location(0) @blend_src(1) factor: vec4<f32>,\n}\n\
             @fragment\nfn fs_imagem_multiplica(in: VsOut) -> DuasSaidas {\n    \
             let c = cor_de_mistura(in);\n    \
             return DuasSaidas(c, vec4<f32>(vec3<f32>(1.0 - c.a) + c.rgb, 1.0));\n}\n",
        );
    }
    s
}

/// Os pipelines das imagens, um por mistura, para alvos do formato da camada.
pub(crate) struct ImagensDaPlaca {
    layout: wgpu::BindGroupLayout,
    normal: wgpu::RenderPipeline,
    soma: wgpu::RenderPipeline,
    tela: wgpu::RenderPipeline,
    /// `None` ⇒ o dispositivo não tem `DUAL_SOURCE_BLENDING` (a ponte não manda o Multiplica cá).
    multiplica: Option<wgpu::RenderPipeline>,
    /// A textura da ligação MUDA: as linhas de uma corrida ordenada fora de todo troço conhecido.
    vazia: wgpu::TextureView,
}

/// As fontes das imagens de UM quadro: o buffer das linhas, a textura de cada troço e o filtro.
pub struct FontesDasImagens {
    pub buffer: wgpu::Buffer,
    /// Quantas linhas o buffer tem.
    pub linhas: u32,
    pub texturas: Vec<(Range<u32>, wgpu::TextureView)>,
    pub projeto: ph2d_render::ImageFilterMode,
}

/// Os grupos de ligação de um quadro: um por troço de textura, e o MUDO (a imagem não desenha) para
/// as linhas de uma corrida ordenada que nenhum troço cobre.
pub(crate) struct Ligacoes {
    trocos: Vec<(Range<u32>, wgpu::BindGroup)>,
    muda: wgpu::BindGroup,
}

impl ImagensDaPlaca {
    pub(crate) fn new(gpu: &GpuContext, formato: wgpu::TextureFormat) -> Self {
        let device = &gpu.device;
        let entrada = |binding, ty| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
            ty,
            count: None,
        };
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("motion imagens da placa"),
            entries: &[
                entrada(
                    0,
                    wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                ),
                entrada(
                    1,
                    wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                ),
                entrada(
                    2,
                    wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                ),
            ],
        });
        let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("motion imagens da placa"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let modulo_de = |m: bool| {
            device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("motion imagens da placa"),
                source: wgpu::ShaderSource::Wgsl(modulo(m).into()),
            })
        };
        let base = modulo_de(false);
        let pipeline = |module: &wgpu::ShaderModule, fs: &str, m: Mistura| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("motion imagens da placa"),
                layout: Some(&pl),
                vertex: wgpu::VertexState {
                    module,
                    entry_point: Some("vs_imagem"),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    buffers: &[],
                },
                fragment: Some(wgpu::FragmentState {
                    module,
                    entry_point: Some(fs),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: formato,
                        blend: Some(m.estado()),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview_mask: None,
                cache: None,
            })
        };
        let multiplica = ph2d_shape_gpu::multiplica_no_dispositivo(device).then(|| {
            pipeline(
                &modulo_de(true),
                "fs_imagem_multiplica",
                Mistura::Multiplica,
            )
        });
        let vazia = device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("motion imagens da placa (muda)"),
                size: wgpu::Extent3d {
                    width: 1,
                    height: 1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            })
            .create_view(&wgpu::TextureViewDescriptor::default());
        Self {
            normal: pipeline(&base, "fs_imagem", Mistura::Normal),
            soma: pipeline(&base, "fs_imagem_mistura", Mistura::Soma),
            tela: pipeline(&base, "fs_imagem_mistura", Mistura::Tela),
            multiplica,
            layout,
            vazia,
        }
    }

    /// ⭐ doc 132 §4.1 (3) — **as imagens como SEGUNDO MEIO do passe de formas**: o desenho ordenado das
    /// corridas `Normal` com as duas médias ([`Corrida::ordenada`]) liga estes grupos no `2`.
    pub(crate) fn regista_no(&self, gpu: &GpuContext, passe: &mut ph2d_shape_gpu::ShapePass) {
        if !passe.tem_segundo_meio() {
            passe.com_segundo_meio(gpu, &meio(2), &self.layout);
        }
    }

    /// A vista e a tabela do filtro, e um grupo de ligação por troço de textura. ⚠️ Um uniforme POR
    /// ligação: a camada e as páginas das células (`ph2d_fx_cells`) desenham com vistas diferentes no
    /// mesmo quadro.
    pub(crate) fn liga(
        &self,
        gpu: &GpuContext,
        vista: ShapeView,
        fontes: &FontesDasImagens,
    ) -> Ligacoes {
        let uniforme = |mudo: u32| {
            let v = Vista {
                lin: vista.lin,
                t: vista.t,
                alvo: vista.alvo,
                ponto: tabela_do_ponto(fontes.projeto),
                mudo,
                _p: [0; 3],
            };
            let b = gpu.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("motion imagens da placa (vista)"),
                size: std::mem::size_of::<Vista>() as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            gpu.queue.write_buffer(&b, 0, bytemuck::bytes_of(&v));
            b
        };
        let grupo = |u: &wgpu::Buffer, textura: &wgpu::TextureView| {
            gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("motion imagens da placa"),
                layout: &self.layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: u.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: fontes.buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::TextureView(textura),
                    },
                ],
            })
        };
        let vivo = uniforme(0);
        Ligacoes {
            trocos: fontes
                .texturas
                .iter()
                .map(|(linhas, textura)| (linhas.clone(), grupo(&vivo, textura)))
                .collect(),
            muda: grupo(&uniforme(1), &self.vazia),
        }
    }

    /// **As imagens de uma corrida**, no passe dela: o pipeline da mistura da corrida e, por troço de
    /// textura, a interseção com [`Corrida::imagens`].
    pub(crate) fn desenha(
        &self,
        pass: &mut wgpu::RenderPass<'_>,
        ligacoes: &Ligacoes,
        c: &Corrida,
    ) {
        if c.ordenada() {
            Self::desenha_ordenada(pass, ligacoes, c);
            return;
        }
        let pipeline = match c.mistura {
            Mistura::Normal => Some(&self.normal),
            Mistura::Soma => Some(&self.soma),
            Mistura::Tela => Some(&self.tela),
            Mistura::Multiplica => self.multiplica.as_ref(),
        };
        let Some(pipeline) = pipeline else {
            eprintln!(
                "[formas] Multiplica pedido num dispositivo sem DUAL_SOURCE_BLENDING: as imagens da corrida nao desenham"
            );
            return;
        };
        pass.set_pipeline(pipeline);
        for (linhas, bg) in &ligacoes.trocos {
            let (a, b) = (
                linhas.start.max(c.imagens.start),
                linhas.end.min(c.imagens.end),
            );
            if a < b {
                pass.set_bind_group(0, bg, &[]);
                pass.draw(0..6, a..b);
            }
        }
    }

    /// ⭐⭐ doc 132 §4.1 (3) — **a corrida ordenada**: o passe de formas já ligou o pipeline ordenado e
    /// os grupos `0`/`1`; aqui o grupo `2` de cada troço e UM desenho por troço sobre as linhas das
    /// FORMAS (a instância é a linha nelas; a das imagens viaja no `first_vertex`). As linhas sem troço
    /// desenham com a ligação muda (só a forma, se houver).
    fn desenha_ordenada(pass: &mut wgpu::RenderPass<'_>, ligacoes: &Ligacoes, c: &Corrida) {
        let n = c.imagens.end - c.imagens.start;
        let n_formas = c.copias.end - c.copias.start;
        debug_assert_eq!(
            n_formas, n,
            "uma corrida ordenada é a MESMA faixa do sink nos dois buffers"
        );
        let Some(v0) = ph2d_shape_gpu::primeiro_vertice_ordenado(c.copias.start, c.imagens.start)
        else {
            eprintln!("[formas] a corrida ordenada passa de 2^28 linhas: as imagens nao desenham");
            return;
        };
        let forma = |i: u32| c.copias.start + (i - c.imagens.start);
        let mut desenha = |a: u32, b: u32, bg: &wgpu::BindGroup| {
            if a < b {
                pass.set_bind_group(2, bg, &[]);
                pass.draw(v0..v0 + 6, forma(a)..forma(b));
            }
        };
        let mut i = c.imagens.start;
        let fim = c.imagens.start + n.min(n_formas);
        let mut trocos: Vec<&(Range<u32>, wgpu::BindGroup)> = ligacoes
            .trocos
            .iter()
            .filter(|(r, _)| r.start < fim && r.end > i)
            .collect();
        trocos.sort_by_key(|(r, _)| r.start);
        for (r, bg) in trocos {
            desenha(i, r.start.clamp(i, fim), &ligacoes.muda);
            let b = r.end.min(fim);
            desenha(r.start.max(i), b, bg);
            i = i.max(b);
        }
        desenha(i, fim, &ligacoes.muda);
    }
}

impl ImagensDaPlaca {
    /// ⭐ **As imagens SOZINHAS** (a página das células do `Each Shape`, `ph2d_fx_cells`): cada linha
    /// em `Normal` sobre `alvo` (que já tem as formas da página), com a vista da página.
    pub(crate) fn pinta_isoladas(
        &self,
        gpu: &GpuContext,
        enc: &mut wgpu::CommandEncoder,
        alvo: &wgpu::TextureView,
        vista: ShapeView,
        fontes: &FontesDasImagens,
    ) {
        let ligacoes = self.liga(gpu, vista, fontes);
        let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("motion imagens da placa (celulas)"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: alvo,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(&self.normal);
        for (linhas, bg) in &ligacoes.trocos {
            pass.set_bind_group(0, bg, &[]);
            pass.draw(0..6, linhas.clone());
        }
    }
}

/// **As texturas das linhas RELOCADAS** — a linha `j` é a original `indices[j]`: a textura de cada
/// troço da relocação pela partição original, vizinhos da mesma textura juntos.
#[must_use]
pub(crate) fn texturas_relocadas(
    texturas: &[(Range<u32>, wgpu::TextureView)],
    indices: &[u32],
) -> Vec<(Range<u32>, wgpu::TextureView)> {
    let mut out: Vec<(Range<u32>, usize)> = Vec::new();
    for (j, i) in (0u32..).zip(indices) {
        let Some(k) = texturas.iter().position(|(r, _)| r.contains(i)) else {
            continue;
        };
        match out.last_mut() {
            Some((r, kk)) if *kk == k && r.end == j => r.end = j + 1,
            _ => out.push((j..j + 1, k)),
        }
    }
    out.into_iter()
        .map(|(r, k)| (r, texturas[k].1.clone()))
        .collect()
}

/// ⭐ **As fontes das imagens do quadro** — as linhas (as que o cozimento separou na rota do
/// dispositivo, as do [`super::PlacaDeFormas`] na da CPU) e, por troço, a textura que o renderer liga
/// àquele `texture_id` (`SpriteRenderer::texture_view_and_dims`, a porta única das três lojas: átlas,
/// individuais e ASSADAS). Um troço cuja textura nenhuma loja conhece não desenha.
#[must_use]
pub fn fontes_do_quadro(
    placa: &super::PlacaDeFormas,
    gpu: &GpuContext,
    motion: &crate::motion_state::MotionState,
    renderer: &ph2d_render::SpriteRenderer,
    projeto: ph2d_render::ImageFilterMode,
) -> Option<FontesDasImagens> {
    let vista = |id: u32| {
        renderer
            .texture_view_and_dims(id)
            .map(|(v, _, _)| v.clone())
    };
    if let Some(imagens) = motion
        .gpu_cook
        .imagens_do_grupo()
        .filter(|_| motion.gpu_live)
    {
        let texturas = imagens
            .texturas()
            .iter()
            .filter_map(|(linhas, id)| Some((linhas.clone(), vista(*id)?)))
            .collect();
        return Some(FontesDasImagens {
            buffer: imagens.buffer().clone(),
            linhas: imagens.len(),
            texturas,
            projeto,
        });
    }
    let linhas = placa.imagens_da_cpu();
    if linhas.is_empty() {
        return None;
    }
    let mut troços: Vec<(Range<u32>, u32)> = Vec::new();
    for (j, l) in (0u32..).zip(linhas) {
        match troços.last_mut() {
            Some((r, id)) if *id == l.texture_id => r.end = j + 1,
            _ => troços.push((j..j + 1, l.texture_id)),
        }
    }
    let bytes: &[u8] = bytemuck::cast_slice(linhas);
    let buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("motion imagens da placa (cpu)"),
        size: bytes.len() as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    gpu.queue.write_buffer(&buffer, 0, bytes);
    Some(FontesDasImagens {
        buffer,
        linhas: u32::try_from(linhas.len()).unwrap_or(u32::MAX),
        texturas: troços
            .into_iter()
            .filter_map(|(r, id)| Some((r, vista(id)?)))
            .collect(),
        projeto,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// O módulo VALIDA no naga (as duas variantes), e a tabela do filtro é a da CPU: a tag `0` herda
    /// o projecto, as tags de ponto ampliam por ponto, e o projecto de fábrica é bilinear.
    #[test]
    fn o_modulo_das_imagens_valida_e_a_tabela_do_filtro_e_a_da_cpu() {
        // O pipeline sozinho (com e sem Multiplica) e o desenho ORDENADO do passe de formas com as
        // imagens como segundo meio (doc 132 §4.1 (3)).
        let ordenado = ph2d_shape_gpu::ShapePass::modulo_ordenado(&meio(2));
        for src in [modulo(false), modulo(true), ordenado] {
            let module = naga::front::wgsl::parse_str(&src).unwrap_or_else(|e| {
                panic!("o WGSL das imagens nao compila: {}", e.emit_to_string(&src))
            });
            let mut v = naga::valid::Validator::new(
                naga::valid::ValidationFlags::all(),
                naga::valid::Capabilities::all(),
            );
            v.validate(&module).expect("o WGSL das imagens valida");
        }
        let bit = |t: &[u32; 8], tag: u32| (t[(tag / 32) as usize] >> (tag % 32)) & 1 == 1;
        let liso = tabela_do_ponto(ph2d_render::ImageFilterMode::Smooth);
        let pixel = tabela_do_ponto(ph2d_render::ImageFilterMode::PixelArt);
        assert!(!bit(&liso, 0) && bit(&pixel, 0), "a tag 0 herda o projecto");
        for tag in 1..=6 {
            assert_eq!(
                bit(&liso, tag),
                ph2d_render::filter_tag_magnifies_by_point(tag as u8),
                "tag {tag}"
            );
            assert_eq!(bit(&liso, tag), bit(&pixel, tag), "tag {tag} não herda");
        }
    }
}
