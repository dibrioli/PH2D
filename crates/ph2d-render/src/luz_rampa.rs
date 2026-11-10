//! P13 L — a luz na rampa na placa (`docs/PixelArt/P13_plano.md` §4.L): a escolha do degrau em
//! WGSL, a mesma conta do gabarito `ph2d_pixel_rampa::luz` (o `pc_to_oklab` determinista da
//! `ph2d-color`). Gate: `a_placa_e_o_gabarito_concordam_na_rampa` (`shells/pixel-scenes`).

/// `luz_receber_rampa(lut, rampa, pos, luz) -> vec4` com as bibliotecas `pe_*`/`pc_*` à frente.
pub const LUZ_RAMPA_WGSL: &str = concat!(
    ph2d_ease::wgsl_lib!(),
    ph2d_color::wgsl_perceptual!(),
    include_str!("shaders/luz_rampa.wgsl")
);

/// O formato da LUT (`ph2d_pixel_rampa::luz::LutDaLuz::texeis`): `(L, r, g, b)` em `f32`.
pub const FORMATO_DA_LUT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba32Float;

/// Uma textura 1×1 vazia (as bandas neutras da colagem sem normal ou sem rampa).
pub(crate) fn neutra_1x1(device: &wgpu::Device, format: wgpu::TextureFormat) -> wgpu::TextureView {
    device
        .create_texture(&wgpu::TextureDescriptor {
            label: Some("ph2d-render band blit neutra"),
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        })
        .create_view(&wgpu::TextureViewDescriptor::default())
}
