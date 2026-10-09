//! **O vetor que bloqueia a luz** (W3) — os caminhos com `BloqueiaLuz`, desenhados pelo Vello pela
//! MESMA porta do tile de uma forma (`draw_path_standalone`: o que se vê é o que bloqueia) num alvo
//! alinhado à GRADE de luz, `Degrau::vetor_px_por_texel` píxeis por texel. Cada camada (uma por opacidade) entra na C0 como
//! um sprite que cobre a grade inteira — a captura por texel já provada para os sprites.
//!
//! ⚠️ O Vello só escreve `Rgba8Unorm` com alfa directa: o alfa É a cobertura, a cor não importa (o
//! vetor não emite — a emissão é do `SpriteEmissive`, que é de sprite).

use std::collections::BTreeMap;

use kurbo::Affine;
use ph2d_gpu::GpuContext;
use ph2d_light2d_ecs::CenaDeLuz;
use ph2d_light2d_gpu::{Grade, SpriteNaC0};
use ph2d_render::VelloPass;
use ph2d_vec_scene::{VecScene, VecXforms, xform_of};
use ph2d_vector::{Color as VelloColor, VectorScene};

struct Camada {
    opacidade: f32,
    textura: wgpu::Texture,
    vista: wgpu::TextureView,
}

#[derive(Default)]
pub(crate) struct CamadaVetorial {
    vello: Option<VelloPass>,
    camadas: Vec<Camada>,
    /// Quantas camadas o último quadro desenhou (as primeiras `n` de `camadas`).
    n: usize,
    grade: Option<Grade>,
}

impl CamadaVetorial {
    /// Desenha os caminhos que bloqueiam (`cena.vetores`) para a grade `g`. Sem nenhum, não faz nada.
    pub(crate) fn desenhar(
        &mut self,
        gpu: &GpuContext,
        (g, fator): (Grade, u32),
        cena: &CenaDeLuz,
        doc: &VecScene,
        xforms: &VecXforms,
    ) {
        self.n = 0;
        self.grade = Some(g);
        if cena.vetores.is_empty() {
            return;
        }
        // Uma camada por opacidade (quase sempre uma: opaco).
        let mut por_opacidade: BTreeMap<u32, Vec<u64>> = BTreeMap::new();
        for v in &cena.vetores {
            por_opacidade
                .entry(v.opacidade.to_bits())
                .or_default()
                .push(v.caminho);
        }
        let fator = fator.max(1);
        let tamanho = (g.largura * fator, g.altura * fator);
        let k = f64::from(fator) / g.texel_m;
        // Mundo → píxel do alvo: x para a direita, y para BAIXO a partir do topo da grade.
        let mundo_para_alvo = Affine::new([
            k,
            0.0,
            0.0,
            -k,
            -g.origem_m[0] * k,
            (g.origem_m[1] + f64::from(g.altura) * g.texel_m) * k,
        ]);
        let Some(vello) = vello_para(&mut self.vello, gpu, tamanho) else {
            return;
        };
        let mut cena_vello = VectorScene::new();
        let mut n = 0;
        for (bits, caminhos) in por_opacidade {
            cena_vello.reset();
            for id in caminhos {
                if let Some(path) = doc.path(id) {
                    let xf = mundo_para_alvo * Affine::new(xform_of(xforms, id).0);
                    ph2d_vec_render::draw_path_standalone(path, xf, &mut cena_vello);
                }
            }
            if let Err(e) = vello.render_to_intermediate(
                gpu,
                cena_vello.inner(),
                tamanho,
                VelloColor::TRANSPARENT,
            ) {
                eprintln!("[luz2d] a camada vetorial falhou: {e}");
                continue;
            }
            copiar(
                gpu,
                &mut self.camadas,
                n,
                f32::from_bits(bits),
                vello.intermediate_texture(),
                tamanho,
            );
            n += 1;
        }
        self.n = n;
    }

    /// As camadas do último quadro como sprites da C0: `(vista, sprite)` — cada uma cobre a grade.
    pub(crate) fn sprites(&self) -> impl Iterator<Item = (&wgpu::TextureView, SpriteNaC0)> + '_ {
        let g = self.grade;
        self.camadas[..self.n].iter().filter_map(move |c| {
            let g = g?;
            let (lx, ly) = (
                f64::from(g.largura) * g.texel_m,
                f64::from(g.altura) * g.texel_m,
            );
            Some((
                &c.vista,
                SpriteNaC0 {
                    origem: [g.origem_m[0], g.origem_m[1] + ly],
                    eixo_s: [lx, 0.0],
                    eixo_t: [0.0, -ly],
                    uv: [0.0, 0.0, 1.0, 1.0],
                    emissao: [0.0; 3],
                    opacidade: c.opacidade,
                    premultiplicada: false,
                    textura: 0,
                },
            ))
        })
    }
}

fn vello_para<'a>(
    vello: &'a mut Option<VelloPass>,
    gpu: &GpuContext,
    tamanho: (u32, u32),
) -> Option<&'a mut VelloPass> {
    if vello.is_none() {
        match VelloPass::new(gpu, wgpu::TextureFormat::Bgra8UnormSrgb, tamanho) {
            Ok(v) => *vello = Some(v),
            Err(e) => {
                eprintln!("[luz2d] sem Vello para a camada vetorial: {e}");
                return None;
            }
        }
    }
    vello.as_mut()
}

/// O intermediário do Vello é UM só: cada camada copia-se para uma textura sua (placa → placa).
fn copiar(
    gpu: &GpuContext,
    camadas: &mut Vec<Camada>,
    i: usize,
    opacidade: f32,
    origem: &wgpu::Texture,
    tamanho: (u32, u32),
) {
    let serve = camadas
        .get(i)
        .is_some_and(|c| (c.textura.width(), c.textura.height()) == tamanho);
    if !serve {
        let textura = gpu.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("luz2d/tex/camada_vetorial"),
            size: wgpu::Extent3d {
                width: tamanho.0,
                height: tamanho.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: origem.format(),
            usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let vista = textura.create_view(&wgpu::TextureViewDescriptor::default());
        let c = Camada {
            opacidade,
            textura,
            vista,
        };
        if i < camadas.len() {
            camadas[i] = c;
        } else {
            camadas.push(c);
        }
    }
    camadas[i].opacidade = opacidade;
    let mut enc = gpu.device.create_command_encoder(&Default::default());
    enc.copy_texture_to_texture(
        origem.as_image_copy(),
        camadas[i].textura.as_image_copy(),
        wgpu::Extent3d {
            width: tamanho.0,
            height: tamanho.1,
            depth_or_array_layers: 1,
        },
    );
    gpu.queue.submit([enc.finish()]);
}
