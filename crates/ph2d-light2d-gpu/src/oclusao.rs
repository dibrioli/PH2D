//! ⭐ **A camada de oclusão de outra família** (W5e) — o traço do Flip e as formas do Motion não
//! estão no `World` como geometria nem no documento vetorial: a família dona é que sabe desenhá-los.
//! A luz pede-lhe, por grupo de opacidade, uma camada alinhada à grade; a C0 lê-a como um sprite
//! (pelo meio: a cobertura é a área do alfa ≥ ½ — a lei da borda exata, `docs/Luz2D/BUGS_luz2d.md`
//! §2). A família não depende da luz além deste contrato, e a luz não depende da família.

use ph2d_gpu::GpuContext;

/// O formato do alvo de uma [`CamadaDeOclusao`]: premultiplicado e linear, o do `game_rt`.
pub const FORMATO_DA_OCLUSAO: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

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
