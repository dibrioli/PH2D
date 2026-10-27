//! **O nascimento de uma textura de CONTEÚDO sob o escopo de memória** (F36 §1,
//! `docs/UI_New_and_Simple/F36_plano.md`). Uma textura criada com a placa cheia nasce inválida para
//! sempre; quem a possui só sabe disso se o perguntar NO NASCIMENTO. Os donos (o átlas, a loja de
//! texturas individuais) guardam a fonte da que nasceu recusada e refazem-na no fim do episódio.

/// Cria a textura sob [`ph2d_gpu::memoria::sob_escopo_de_memoria`]; `true` = a placa recusou.
pub(crate) fn nasce_textura(
    device: &wgpu::Device,
    cria: impl FnOnce() -> wgpu::Texture,
) -> (wgpu::Texture, bool) {
    let (texture, erro) = ph2d_gpu::memoria::sob_escopo_de_memoria(device, cria);
    #[cfg(test)]
    if testes::consome_recusa_forcada() {
        // A recusa de verdade: o conteúdo que vier a seguir perde-se.
        texture.destroy();
        return (texture, true);
    }
    (texture, erro.is_some())
}

#[cfg(test)]
pub(crate) mod testes {
    use std::cell::Cell;

    thread_local! {
        static FORCADAS: Cell<u32> = const { Cell::new(0) };
    }

    /// As próximas `n` texturas de conteúdo deste thread nascem recusadas (destruídas).
    pub(crate) fn forca_recusas(n: u32) {
        FORCADAS.with(|c| c.set(n));
    }

    pub(super) fn consome_recusa_forcada() -> bool {
        FORCADAS.with(|c| {
            let n = c.get();
            c.set(n.saturating_sub(1));
            n > 0
        })
    }
}
