//! ⭐⭐⭐ **A DOBRA DO RELEVO** — *«que relevo tem esta amostra?»*, UMA resposta para o Painter 2D
//! e para a peça 3D (`docs/3D/30` §2, a W4).
//!
//! A pilha decide QUEM entra ([`LayerStack::relief_layers_bottom_up`]: visível com todos os grupos
//! acima, de baixo para cima) e COMO cada camada se junta ao que está por baixo
//! ([`fold_relief_step`]: `own · depth`, [`ReliefComposite::Add`] soma, [`ReliefComposite::Level`]
//! enterra na proporção da cobertura da PRÓPRIA camada). O 2D (`ReliefFields::height_at`) e a peça
//! (`PilhaDaPeca`) chamam as duas; cada um lê os seus planos.
//!
//! ⚠️ O tecto de vidro (`impasto_ceiling::soft_ceiling`) NÃO mora aqui: é a aparência da tinta numa
//! tela em píxeis, aplicada por quem a desenha, depois da dobra.

use super::{LayerId, LayerStack, ReliefComposite};

/// ⭐⭐⭐ **Um passo da dobra**: a pilha `h` por baixo, a camada por cima com o relevo `own`, a sua
/// profundidade e o seu modo. `cover` (a cobertura da camada, `0..=1`) só é lida por `Level`.
#[inline]
#[must_use]
pub fn fold_relief_step(
    h: f32,
    own: f32,
    depth: f32,
    composite: ReliefComposite,
    cover: impl FnOnce() -> f32,
) -> f32 {
    let own = own * depth;
    match composite {
        ReliefComposite::Add => h + own,
        // Solid paint IS the surface; bare paint shows the pile below untouched — otherwise an empty
        // region of a `Level` layer would flatten the whole painting.
        ReliefComposite::Level => {
            let c = cover();
            h * (1.0 - c) + own * c
        }
    }
}

impl LayerStack {
    /// Whether `id` is visible *and every group above it is too* — hiding a GROUP puts out the relief
    /// of everything inside it.
    #[must_use]
    pub fn effectively_visible(&self, id: LayerId) -> bool {
        let mut cur = Some(id);
        while let Some(c) = cur {
            match self.get(c) {
                Some(l) if l.visible => cur = self.parent_of(c),
                _ => return false,
            }
        }
        true
    }

    /// ⭐⭐ **As camadas que entram na dobra do relevo, de baixo para cima** — o z-order com as
    /// escondidas (ou dentro de um grupo escondido) de fora. Quem chama fica só com as que TÊM relevo.
    #[must_use]
    pub fn relief_layers_bottom_up(&self) -> Vec<LayerId> {
        let mut ids = self.z_order_bottom_up();
        ids.retain(|&id| self.effectively_visible(id));
        ids
    }
}

#[cfg(test)]
#[path = "relief_fold_tests.rs"]
mod tests;
