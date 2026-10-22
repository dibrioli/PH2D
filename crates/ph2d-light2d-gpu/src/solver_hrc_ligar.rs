//! As ligações e os grupos da holográfica: o berço dos buffers (W5x), as entradas dos grupos de ligação e os
//! grupos de trabalho dobrados em 2D. Módulo-filho de `solver_hrc.rs`.

use super::*;

/// ⭐ W5x — o BERÇO do buffer `nome`: uma textura crua (nunca usada) com pelo menos os bytes dele; `None`
/// se não cabe numa textura do dispositivo (o buffer nasce então onde calhar).
pub(super) fn berco(e: &Entradas<'_>, nome: &'static str) -> Option<wgpu::Texture> {
    const TEXEL: u64 = 16;
    let bytes = e.lista.buffers.iter().find(|b| b.nome == nome)?.bytes;
    let lado = e.device.limits().max_texture_dimension_2d;
    let altura = u32::try_from(bytes.div_ceil(TEXEL * u64::from(lado))).ok()?;
    (altura <= lado).then(|| {
        e.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("luz2d/tex/berco_hrc"),
            size: wgpu::Extent3d {
                width: lado,
                height: altura.max(1),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba32Uint,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        })
    })
}

pub(super) fn armazenado(binding: u32, leitura: bool) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only: leitura },
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

pub(super) fn vista(binding: u32, v: &wgpu::TextureView) -> wgpu::BindGroupEntry<'_> {
    wgpu::BindGroupEntry {
        binding,
        resource: wgpu::BindingResource::TextureView(v),
    }
}

pub(super) fn buf(binding: u32, x: &wgpu::Buffer) -> wgpu::BindGroupEntry<'_> {
    wgpu::BindGroupEntry {
        binding,
        resource: x.as_entire_binding(),
    }
}

/// Grupos de trabalho de 64 para `total` entradas, dobrados em 2D (65 535 por eixo) por IGUAL nas filas
/// (⭐ W5s: `[65 535, 2]` deixava até metade da 2.ª fila de grupos vazios, e lançá-los custava — a sobe
/// n10–n12 e a desce n10–n11 −0,63 ms no High; handoff W5s §3).
pub(super) fn grupos_1d(total: u64) -> [u32; 3] {
    dobrar(total.div_ceil(64))
}

/// `g` grupos dobrados em 2D por igual nas filas (o [`grupos_1d`]; ⭐ §44 o despacho junto).
pub(super) fn dobrar(g: u64) -> [u32; 3] {
    let g = g.max(1);
    let y = g.div_ceil(65_535);
    [g.div_ceil(y) as u32, y as u32, 1]
}

/// As constantes de pipeline da holográfica (os `override` do `c2_hrc.wgsl`).
pub(super) fn constantes(e: &Entradas<'_>, cones: u32, pares: u32) -> [(&'static str, f64); 13] {
    let b = |x: bool| f64::from(u8::from(x));
    [
        ("PASSO_HRC", f64::from(e.escolhas.passo)),
        ("BORRAO_HRC", b(e.escolhas.borrao)),
        ("MAIS_UM_HRC", b(e.escolhas.mais_um)),
        ("MASCARA_C0", b(e.cfg.preenchimento.is_some())),
        ("COLUNAS_HRC", b(e.cfg.hrc_por_colunas)),
        ("ARCOS_TABELA", b(e.cfg.hrc_arcos_em_tabela)),
        ("SALTA_O_VAZIO_HRC", b(e.cfg.hrc_salta_o_vazio)),
        ("SONDA_HRC", b(e.cfg.hrc_fio_por_sonda)),
        ("GRADE3D_HRC", b(e.cfg.hrc_grade_3d)),
        ("CONES_HRC", f64::from(cones)),
        ("PARES_SOBE_HRC", f64::from(pares)),
        ("JUNTOS_HRC", b(e.cfg.hrc_juntos())),
        ("T_NULO_HRC", b(e.cfg.hrc_salta_o_t_nulo)),
    ]
}
