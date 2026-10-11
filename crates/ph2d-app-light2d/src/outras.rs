//! ⭐ **Os outros que bloqueiam** (W5e) — o traço do Flip, as formas do Motion: a família dona
//! desenha-os ([`CamadaDeOclusao`]) numa camada por grupo de opacidade, na resolução da CAPTURA fina, e
//! a C0 lê cada camada EXATA como um sprite que cobre a grade — a lei da camada do vetor.

use std::collections::BTreeMap;

use ph2d_gpu::GpuContext;
use ph2d_light2d_ecs::CenaDeLuz;
use ph2d_light2d_gpu::{CamadaDeOclusao, FORMATO_DA_OCLUSAO, Grade, SpriteNaC0};

struct Camada {
    vista: wgpu::TextureView,
    tamanho: (u32, u32),
    opacidade: f32,
    reflete: bool,
}

/// As camadas do último quadro.
#[derive(Default)]
pub(crate) struct CamadaDeOutros {
    camadas: Vec<Camada>,
    n: usize,
    grade: Option<Grade>,
}

impl CamadaDeOutros {
    /// Pede às `familias` os `cena.outros`, um grupo de opacidade por camada, na grade `g` com
    /// `fator` píxeis por texel. Sem nenhum, não faz nada.
    pub(crate) fn desenhar(
        &mut self,
        gpu: &GpuContext,
        (g, fator): (Grade, u32),
        cena: &CenaDeLuz,
        familias: &mut [&mut dyn CamadaDeOclusao],
    ) {
        self.n = 0;
        self.grade = Some(g);
        if cena.outros.is_empty() || familias.is_empty() {
            return;
        }
        let mut grupos: BTreeMap<(u32, bool), Vec<u64>> = BTreeMap::new();
        for o in &cena.outros {
            grupos
                .entry((o.opacidade.to_bits(), o.reflete))
                .or_default()
                .push(o.entidade.to_bits());
        }
        let fator = fator.max(1);
        let tamanho = (g.largura * fator, g.altura * fator);
        let mundo = [
            g.origem_m[0],
            g.origem_m[1],
            g.origem_m[0] + f64::from(g.largura) * g.texel_m,
            g.origem_m[1] + f64::from(g.altura) * g.texel_m,
        ];
        for ((bits, reflete), quem) in grupos {
            let n = self.n;
            if self.camadas.get(n).is_none_or(|c| c.tamanho != tamanho) {
                let c = nova(gpu, tamanho);
                if n < self.camadas.len() {
                    self.camadas[n] = c;
                } else {
                    self.camadas.push(c);
                }
            }
            let c = &mut self.camadas[n];
            limpar(gpu, &c.vista);
            let mut desenhou = false;
            for f in familias.iter_mut() {
                desenhou |= f.desenhar(gpu, &c.vista, tamanho, mundo, &quem);
            }
            if desenhou {
                c.opacidade = f32::from_bits(bits);
                c.reflete = reflete;
                self.n += 1;
            }
        }
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
                    premultiplicada: true,
                    textura: 0,
                    albedo: if c.reflete { [1.0; 3] } else { [0.0; 3] },
                    cor_em_srgb: false,
                },
            ))
        })
    }
}

fn nova(gpu: &GpuContext, tamanho: (u32, u32)) -> Camada {
    let textura = gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("luz2d/outros"),
        size: wgpu::Extent3d {
            width: tamanho.0.max(1),
            height: tamanho.1.max(1),
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: FORMATO_DA_OCLUSAO,
        // `COPY_DST`: uma família pode também ESCREVER a camada (os testes, com a área exata).
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    Camada {
        vista: textura.create_view(&wgpu::TextureViewDescriptor::default()),
        tamanho,
        opacidade: 0.0,
        reflete: false,
    }
}

fn limpar(gpu: &GpuContext, vista: &wgpu::TextureView) {
    let mut enc = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("luz2d/outros/limpar"),
        });
    drop(enc.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("luz2d/outros/limpar"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: vista,
            depth_slice: None,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                store: wgpu::StoreOp::Store,
            },
        })],
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
        multiview_mask: None,
    }));
    gpu.queue.submit([enc.finish()]);
}
