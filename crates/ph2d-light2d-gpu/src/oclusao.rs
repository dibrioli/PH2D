//! ⭐ **A camada de oclusão de outra família** (W5e) — o traço do Flip e as formas do Motion não
//! estão no `World` como geometria nem no documento vetorial: a família dona é que sabe desenhá-los.
//! A luz pede-lhe, por grupo de opacidade, uma camada alinhada à grade na resolução da CAPTURA fina,
//! e a C0 lê-a EXATA (o píxel já é a cobertura de área) — a lei da camada do vetor (W3/W5a). Lida pelo
//! meio na grade fina (1 px), a variação nos anéis da cena 7 saía +13 · +18 % (discos de área exata)
//! contra +3,4 · −1,8 % assim (handoff W5e). A família não depende da luz além deste contrato, e a luz
//! não depende da família.

use ph2d_gpu::GpuContext;

/// O formato do alvo de uma [`CamadaDeOclusao`]: premultiplicado, guardado em sRGB de 8 bits (a placa
/// mistura e lê em linear) — a memória da camada do vetor (63 MB por grupo a 1080p, na captura), com os
/// escuros do albedo sem degraus.
pub const FORMATO_DA_OCLUSAO: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;

/// Uma família que desenha, para a luz, os seus objetos que bloqueiam.
pub trait CamadaDeOclusao {
    /// Desenha em `alvo` ([`FORMATO_DA_OCLUSAO`], `tamanho` px, já limpo e transparente) os objetos
    /// cujas entidades (`Entity::to_bits`) estão em `quem`, como a janela do mundo
    /// `mundo = [x0, y0, x1, y1]` os mostra — a linha 0 do alvo é `y1` (y do mundo para cima). O
    /// alfa é a COBERTURA de área (premultiplicado); a cor, a do objeto (o albedo do rebote).
    /// Devolve se desenhou algum.
    fn desenhar(
        &mut self,
        gpu: &GpuContext,
        alvo: &wgpu::TextureView,
        tamanho: (u32, u32),
        mundo: [f64; 4],
        quem: &[u64],
    ) -> bool;
}
