//! ⭐ **A luz 2D RECEBIDA** — o `@group(3)` do passe de sprite e da colagem do vetor (plano
//! `docs/Luz2D/01_plano.md` §2.4). A luz em si (o solver) vive em `ph2d-light2d-gpu`; esta crate só
//! sabe AMOSTRÁ-la: uma textura de fluência ancorada no mundo e a transformação até ela.
//!
//! ⛔ Sem luz o grupo ligado é o NEUTRO (`ligada = 0`): o ramo do `receber.wgsl` não é tomado e a
//! cor sai intacta — o quadro de sempre, byte a byte (gate G6, em `ph2d-app-light2d`).

use std::collections::BTreeSet;

use bytemuck::{Pod, Zeroable};
use ph2d_gpu::GpuContext;

/// O texto de `receber.wgsl` — o mesmo que o sprite e a colagem concatenam (censo no gate).
pub const RECEBER_WGSL: &str = include_str!("shaders/receber.wgsl");

/// O espelho de `LuzUniforme` do `receber.wgsl` (64 B).
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct LuzUniforme {
    pub origem: [f32; 2],
    pub escala: [f32; 2],
    pub tela_x: [f32; 4],
    pub tela_y: [f32; 4],
    pub ligada: u32,
    pub _p: [u32; 3],
}

/// A luz deste quadro, como o receptor a vê.
pub struct LuzRecebida<'a> {
    /// A fluência por sonda (`Rgba16Float` ou `Rgba32Float`), texel `(i, j)` centrado em
    /// `origem_m + (i + ½, j + ½)·texel_m`.
    pub fluencia: &'a wgpu::TextureView,
    pub origem_m: [f64; 2],
    pub texel_m: f64,
    pub tamanho: [u32; 2],
    /// As entidades da SIMULAÇÃO com `NaoRecebeLuz` (casadas pelo `SimRef` na coleta).
    pub sem_luz: &'a BTreeSet<ph2d_ecs::Entity>,
}

impl LuzRecebida<'_> {
    /// O uniforme aceso. `tela_para_mundo` = as duas linhas do afim píxel → mundo (só quem não tem a
    /// posição do mundo o lê: a colagem do vetor); `None` = identidade.
    #[must_use]
    pub fn uniforme(&self, tela_para_mundo: Option<[[f64; 3]; 2]>) -> LuzUniforme {
        let [lx, ly] = [
            self.texel_m * f64::from(self.tamanho[0]),
            self.texel_m * f64::from(self.tamanho[1]),
        ];
        let [tx, ty] = tela_para_mundo.unwrap_or([[1.0, 0.0, 0.0], [0.0, 1.0, 0.0]]);
        LuzUniforme {
            origem: [self.origem_m[0] as f32, self.origem_m[1] as f32],
            escala: [(1.0 / lx) as f32, (1.0 / ly) as f32],
            tela_x: [tx[0] as f32, tx[1] as f32, tx[2] as f32, 0.0],
            tela_y: [ty[0] as f32, ty[1] as f32, ty[2] as f32, 0.0],
            ligada: 1,
            _p: [0; 3],
        }
    }
}

/// As três entradas do `@group(3)` que o `receber.wgsl` declara (bindings 0–2).
pub(crate) fn entradas_do_layout() -> [wgpu::BindGroupLayoutEntry; 3] {
    let frag = wgpu::ShaderStages::FRAGMENT;
    [
        wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: frag,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        wgpu::BindGroupLayoutEntry {
            binding: 1,
            visibility: frag,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        },
        wgpu::BindGroupLayoutEntry {
            binding: 2,
            visibility: frag,
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            count: None,
        },
    ]
}

/// O `@group(3)` de um receptor: o grupo NEUTRO, construído uma vez, e o ACESO do quadro.
pub(crate) struct GrupoDeLuz {
    uniforme: wgpu::Buffer,
    amostra: wgpu::Sampler,
    neutro: wgpu::BindGroup,
    aceso: Option<wgpu::BindGroup>,
}

impl GrupoDeLuz {
    /// `extra` = entradas a mais do layout (a colagem acrescenta o LUT do tonemap), já no `bgl`.
    pub(crate) fn novo(
        gpu: &GpuContext,
        bgl: &wgpu::BindGroupLayout,
        extra: &[wgpu::BindGroupEntry<'_>],
    ) -> Self {
        use wgpu::util::DeviceExt as _;
        let device = &gpu.device;
        let uniforme = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ph2d-render luz recebida ubo"),
            size: size_of::<LuzUniforme>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let desligada = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("ph2d-render luz desligada ubo"),
            contents: bytemuck::bytes_of(&LuzUniforme::zeroed()),
            usage: wgpu::BufferUsages::UNIFORM,
        });
        let amostra = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("ph2d-render luz recebida sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let vazia = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("ph2d-render luz neutra"),
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let vista = vazia.create_view(&wgpu::TextureViewDescriptor::default());
        let neutro = grupo(device, bgl, &desligada, &vista, &amostra, extra);
        Self {
            uniforme,
            amostra,
            neutro,
            aceso: None,
        }
    }

    /// Acende (`Some`) ou apaga (`None`) o grupo para os próximos desenhos.
    pub(crate) fn definir(
        &mut self,
        gpu: &GpuContext,
        bgl: &wgpu::BindGroupLayout,
        luz: Option<(&wgpu::TextureView, LuzUniforme)>,
        extra: &[wgpu::BindGroupEntry<'_>],
    ) {
        self.aceso = luz.map(|(vista, u)| {
            gpu.queue
                .write_buffer(&self.uniforme, 0, bytemuck::bytes_of(&u));
            grupo(
                &gpu.device,
                bgl,
                &self.uniforme,
                vista,
                &self.amostra,
                extra,
            )
        });
    }

    pub(crate) fn aceso(&self) -> bool {
        self.aceso.is_some()
    }

    /// O grupo a ligar: o aceso se `com_luz` e houver luz; senão o neutro.
    pub(crate) fn ligar(&self, com_luz: bool) -> &wgpu::BindGroup {
        match &self.aceso {
            Some(g) if com_luz => g,
            _ => &self.neutro,
        }
    }
}

fn grupo(
    device: &wgpu::Device,
    bgl: &wgpu::BindGroupLayout,
    uniforme: &wgpu::Buffer,
    vista: &wgpu::TextureView,
    amostra: &wgpu::Sampler,
    extra: &[wgpu::BindGroupEntry<'_>],
) -> wgpu::BindGroup {
    let mut entradas = vec![
        wgpu::BindGroupEntry {
            binding: 0,
            resource: uniforme.as_entire_binding(),
        },
        wgpu::BindGroupEntry {
            binding: 1,
            resource: wgpu::BindingResource::TextureView(vista),
        },
        wgpu::BindGroupEntry {
            binding: 2,
            resource: wgpu::BindingResource::Sampler(amostra),
        },
    ];
    entradas.extend_from_slice(extra);
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("ph2d-render luz recebida bg"),
        layout: bgl,
        entries: &entradas,
    })
}
