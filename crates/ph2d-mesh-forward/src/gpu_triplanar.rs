//! ⭐⭐ **As TEXTURAS TRIPLANARES no desenhista** ([`ph2d_triplanar`]): dois `texture_2d_array`
//! (cor `Rgba8UnormSrgb` e `nrh` `Rgba8Unorm`, `1024²` com mips, uma camada por textura) e um
//! amostrador trilinear com repetição — o que o WebGL2 filtra. Subir ou trocar uma camada não compila
//! nada; crescer refaz as duas matrizes e copia as camadas na placa.

use crate::TexturaMaterial;

/// As ligações `12` (cor), `13` (`nrh`) e `14` (o amostrador) do `g0`.
pub(crate) fn entradas() -> [wgpu::BindGroupLayoutEntry; 3] {
    let tex = |binding| wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
            view_dimension: wgpu::TextureViewDimension::D2Array,
            multisampled: false,
        },
        count: None,
    };
    [
        tex(12),
        tex(13),
        wgpu::BindGroupLayoutEntry {
            binding: 14,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            count: None,
        },
    ]
}

/// Quantos `vec4` a textura de um material acrescenta à tabela dos materiais.
pub const TEXTURA_V4: u32 = 5;

pub(crate) struct TexturasGpu {
    capacidade: u32,
    lado: u32,
    cor: wgpu::Texture,
    nrh: wgpu::Texture,
    pub vista_cor: wgpu::TextureView,
    pub vista_nrh: wgpu::TextureView,
    pub amostrador: wgpu::Sampler,
}

fn matriz(device: &wgpu::Device, lado: u32, camadas: u32, srgb: bool) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("ph2d-mesh-forward triplanar"),
        size: wgpu::Extent3d {
            width: lado,
            height: lado,
            depth_or_array_layers: camadas,
        },
        mip_level_count: 32 - lado.leading_zeros(),
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: if srgb {
            wgpu::TextureFormat::Rgba8UnormSrgb
        } else {
            wgpu::TextureFormat::Rgba8Unorm
        },
        usage: wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_DST
            | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    })
}

fn vista(t: &wgpu::Texture) -> wgpu::TextureView {
    t.create_view(&wgpu::TextureViewDescriptor {
        dimension: Some(wgpu::TextureViewDimension::D2Array),
        ..Default::default()
    })
}

impl TexturasGpu {
    /// Vazias: `1 × 1`, uma camada — o que a ligação lê enquanto nenhuma textura subiu.
    pub fn vazias(device: &wgpu::Device) -> Self {
        let cor = matriz(device, 1, 1, true);
        let nrh = matriz(device, 1, 1, false);
        Self {
            capacidade: 0,
            lado: 1,
            vista_cor: vista(&cor),
            vista_nrh: vista(&nrh),
            cor,
            nrh,
            amostrador: device.create_sampler(&wgpu::SamplerDescriptor {
                label: Some("ph2d-mesh-forward triplanar"),
                address_mode_u: wgpu::AddressMode::Repeat,
                address_mode_v: wgpu::AddressMode::Repeat,
                address_mode_w: wgpu::AddressMode::Repeat,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                mipmap_filter: wgpu::MipmapFilterMode::Linear,
                ..Default::default()
            }),
        }
    }

    /// Garante a camada `camada` (lado `lado`): cresce para a potência de dois seguinte e copia as
    /// camadas que já lá estavam.
    fn garante(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, camada: u32, lado: u32) {
        if camada < self.capacidade && lado == self.lado {
            return;
        }
        let cap = (camada + 1).next_power_of_two();
        let (cor, nrh) = (
            matriz(device, lado, cap, true),
            matriz(device, lado, cap, false),
        );
        if self.capacidade > 0 && lado == self.lado {
            let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("ph2d-mesh-forward triplanar cresce"),
            });
            for (de, para) in [(&self.cor, &cor), (&self.nrh, &nrh)] {
                for nivel in 0..de.mip_level_count() {
                    let l = (lado >> nivel).max(1);
                    enc.copy_texture_to_texture(
                        wgpu::TexelCopyTextureInfo {
                            texture: de,
                            mip_level: nivel,
                            origin: wgpu::Origin3d::ZERO,
                            aspect: wgpu::TextureAspect::All,
                        },
                        wgpu::TexelCopyTextureInfo {
                            texture: para,
                            mip_level: nivel,
                            origin: wgpu::Origin3d::ZERO,
                            aspect: wgpu::TextureAspect::All,
                        },
                        wgpu::Extent3d {
                            width: l,
                            height: l,
                            depth_or_array_layers: self.capacidade,
                        },
                    );
                }
            }
            queue.submit([enc.finish()]);
        }
        self.vista_cor = vista(&cor);
        self.vista_nrh = vista(&nrh);
        self.cor = cor;
        self.nrh = nrh;
        self.capacidade = cap;
        self.lado = lado;
    }

    /// ⭐ Sobe os [`ph2d_triplanar::Mapas`] na `camada`.
    ///
    /// # Panics
    /// Se a cor e o `nrh` não tiverem o mesmo lado.
    pub fn sobe(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        camada: u32,
        m: &ph2d_triplanar::Mapas,
    ) {
        let lado = m.cor.lado();
        assert_eq!(m.nrh.lado(), lado, "cor e nrh do mesmo lado");
        self.garante(device, queue, camada, lado);
        for (t, mip) in [(&self.cor, &m.cor), (&self.nrh, &m.nrh)] {
            for k in 0..mip.contagem() {
                let l = (lado >> k).max(1);
                queue.write_texture(
                    wgpu::TexelCopyTextureInfo {
                        texture: t,
                        mip_level: k,
                        origin: wgpu::Origin3d {
                            x: 0,
                            y: 0,
                            z: camada,
                        },
                        aspect: wgpu::TextureAspect::All,
                    },
                    bytemuck::cast_slice(mip.nivel(k)),
                    wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(l * 4),
                        rows_per_image: Some(l),
                    },
                    wgpu::Extent3d {
                        width: l,
                        height: l,
                        depth_or_array_layers: 1,
                    },
                );
            }
        }
    }

    /// O lado das camadas (`1` sem textura nenhuma).
    pub fn lado(&self) -> u32 {
        self.lado
    }
}

/// ⭐ As colunas da textura de um material na tabela (a arrumação que o `forward.wgsl` lê):
/// `(camada ou −1, 1/tamanho, 1/(tamanho·aspecto), blend)` · `(relevo, normal?, rugosidade?, lado)` ·
/// as três linhas da matriz mundo → folha.
#[must_use]
pub fn colunas(t: Option<&TexturaMaterial>, lado: u32) -> [f32; (TEXTURA_V4 * 4) as usize] {
    let mut o = [0.0f32; (TEXTURA_V4 * 4) as usize];
    let Some(t) = t else {
        o[0] = -1.0;
        return o;
    };
    let (eu, ev) = t.triplanar.escalas();
    o[..4].copy_from_slice(&[t.camada as f32, eu, ev, t.triplanar.blend]);
    o[4..8].copy_from_slice(&[
        t.triplanar.relevo,
        f32::from(u8::from(t.tem_normal)),
        f32::from(u8::from(t.tem_rugosidade)),
        lado as f32,
    ]);
    for (i, linha) in t.mundo_para_folha.iter().enumerate() {
        o[8 + 4 * i..12 + 4 * i].copy_from_slice(linha);
    }
    o
}
