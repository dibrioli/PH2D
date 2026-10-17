//! ⭐ W5z — os T e os R da holográfica (ligações 5–9 do `c2_hrc.wgsl`) em TEXTURA ou em buffer
//! ([`crate::Config::hrc_niveis_em_textura`]): o layout, o recurso de cada ligação e de onde vem cada
//! nível. Módulo-filho de `solver_hrc.rs`.

use super::*;

/// Um T, um R ou um vazio: o buffer, ou a vista da textura da `PassList` com o mesmo rótulo.
pub(super) enum Nivel {
    Buffer(wgpu::Buffer),
    Textura(wgpu::TextureView),
}

impl Nivel {
    /// O nível `nome`: a vista da textura declarada com ele, ou um buffer novo.
    pub(super) fn de(e: &Entradas<'_>, em_textura: bool, nome: &'static str) -> Self {
        if !em_textura {
            return Self::Buffer((e.buffer)(nome));
        }
        let k = e
            .lista
            .texturas
            .iter()
            .position(|t| t.nome == nome)
            .expect("nível em textura declarado na PassList");
        Self::Textura(e.vistas[k].clone())
    }

    pub(super) fn ligar(&self, binding: u32) -> wgpu::BindGroupEntry<'_> {
        match self {
            Self::Buffer(b) => buf(binding, b),
            Self::Textura(v) => vista(binding, v),
        }
    }

    /// O buffer (as réguas do regime medem só os buffers).
    pub(super) fn buffer(self) -> Option<wgpu::Buffer> {
        match self {
            Self::Buffer(b) => Some(b),
            Self::Textura(_) => None,
        }
    }
}

/// As ligações 5–9: `t_a`, `t_b` (ler), `t_saida` (escrever), `r_cima` (ler), `r_saida` (escrever).
pub(super) fn layout(em_textura: bool) -> [wgpu::BindGroupLayoutEntry; 5] {
    let ler = |b| {
        if em_textura {
            entrada_textura(b, wgpu::TextureSampleType::Uint)
        } else {
            armazenado(b, true)
        }
    };
    let escrever = |b| {
        if em_textura {
            entrada_escrita(b, wgpu::TextureFormat::Rg32Uint)
        } else {
            armazenado(b, false)
        }
    };
    [ler(5), ler(6), escrever(7), ler(8), escrever(9)]
}
