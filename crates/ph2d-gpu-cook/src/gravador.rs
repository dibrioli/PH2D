//! **O codificador do cozimento com o passe de computação ABERTO** (doc 120 §8.3.2, a cura 2 do
//! custo fixo da rota híbrida).
//!
//! Cada sítio pede o passe por [`Gravador::passe`] em vez de abrir o seu. Com `um_passe` desligado
//! abre-se um passe novo a cada pedido — os mesmos comandos, pela mesma ordem, que um
//! `begin_compute_pass` por sítio. Ligado, os despachos consecutivos partilham UM passe: o wgpu põe
//! a barreira entre dois despachos do mesmo passe (cada despacho é um âmbito de uso).
//!
//! ⚠️ Um comando que não é de computação (cópia, limpeza, `finish`) chega ao codificador pelo
//! [`std::ops::DerefMut`], que FECHA o passe primeiro: o wgpu tranca o codificador enquanto há um
//! passe vivo, e uma cópia dentro dele seria um erro de validação.

/// O codificador do cozimento e o passe aberto.
pub(crate) struct Gravador {
    /// Antes do `encoder`: o passe morre primeiro, e só então o codificador destranca.
    aberto: Option<wgpu::ComputePass<'static>>,
    encoder: wgpu::CommandEncoder,
    um_passe: bool,
    /// Passes de computação abertos por este gravador.
    pub(crate) passes: u32,
}

impl Gravador {
    pub(crate) fn new(encoder: wgpu::CommandEncoder, um_passe: bool) -> Self {
        Self {
            aberto: None,
            encoder,
            um_passe,
            passes: 0,
        }
    }

    /// O passe onde o próximo despacho entra.
    pub(crate) fn passe(
        &mut self,
        desc: &wgpu::ComputePassDescriptor<'_>,
    ) -> &mut wgpu::ComputePass<'static> {
        if !self.um_passe || self.aberto.is_none() {
            self.aberto = None;
            self.passes += 1;
            self.aberto = Some(self.encoder.begin_compute_pass(desc).forget_lifetime());
        }
        self.aberto.as_mut().expect("aberto acima")
    }

    pub(crate) fn finish(mut self) -> wgpu::CommandBuffer {
        self.aberto = None;
        self.encoder.finish()
    }
}

impl std::ops::Deref for Gravador {
    type Target = wgpu::CommandEncoder;
    fn deref(&self) -> &wgpu::CommandEncoder {
        &self.encoder
    }
}

impl std::ops::DerefMut for Gravador {
    fn deref_mut(&mut self) -> &mut wgpu::CommandEncoder {
        self.aberto = None;
        &mut self.encoder
    }
}
