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

/// ⭐ **Onde a dobra começa: `-0,0`** — a identidade da soma em IEEE (`-0 + x = x` para todo `x`,
/// o `-0` incluído; `+0 + -0 = +0`), logo uma camada só, neutra, devolve o relevo dela AO BIT: um
/// `-0,0` gravado atravessa o ficheiro da peça (gate `o_relevo_atravessa_o_ficheiro_ao_bit`).
pub const RELIEF_FOLD_SEED: f32 = -0.0;

/// ⭐⭐⭐ **Um passo da dobra**: a pilha `h` por baixo, a camada por cima com o relevo `own`, a sua
/// profundidade, o seu modo, a sua cobertura `cover` e a MAIOR cobertura naquele ponto `cover_max`
/// (`0..=1`, a que a luz lê — *relevo sem tinta não acende*).
///
/// ⭐ A camada conta o seu relevo na proporção da SUA tinta face à mais forte ali
/// ([`relief_share`]): numa camada só é `1`, ao bit; a encosta que o alisamento espalha para fora da
/// tinta de uma camada de cima não acende sobre a tinta da de baixo (report do dono, 04/10 — o anel
/// de 01/10 a voltar pelas camadas, `docs/3D/30` §18).
#[inline]
#[must_use]
pub fn fold_relief_step(
    h: f32,
    own: f32,
    depth: f32,
    composite: ReliefComposite,
    cover: f32,
    cover_max: f32,
) -> f32 {
    let own = own * depth * relief_share(cover, cover_max);
    match composite {
        ReliefComposite::Add => h + own,
        // Solid paint IS the surface; bare paint shows the pile below untouched — otherwise an empty
        // region of a `Level` layer would flatten the whole painting.
        ReliefComposite::Level => h * (1.0 - cover) + own * cover,
    }
}

/// A parte do relevo de uma camada que conta: `min(1, corpo(cover) / corpo(min(cover_max,
/// W_SOLID)))`, com `corpo` a [`body_profile`](ph2d_painter_brush::height_film::body_profile) do
/// pincel — nenhum corpo sobre a mancha (`≤ W_TAIL`), a parede até `W_SOLID`, planalto acima.
///
/// `1` exacto quando ela É a maior (uma camada só não muda, ao bit) e `1` sem corpo nenhum ali (o
/// relevo cru, como sempre). Sobre tinta SÓLIDA de outra camada é o corpo da própria tinta: a regra
/// do 2D — *a parede sobe DENTRO da parte pigmentada* — que impede a luz de sombrear o que se vê
/// através de uma tinta fina (a orla cinzenta das fotos de 04/10, `docs/3D/30` §18). ⛔ A razão crua
/// `cover / cover_max` desenhava uma moldura onde a de baixo começa; a linear `cover / W_SOLID`
/// deixava a parede inteira sobre a mancha (medido: a orla na 3.ª foto do dono).
#[inline]
#[must_use]
pub fn relief_share(cover: f32, cover_max: f32) -> f32 {
    use ph2d_painter_brush::height_film::{W_SOLID, body_profile};
    let ref_ = body_profile(cover_max.min(W_SOLID));
    if ref_ <= 0.0 {
        1.0
    } else {
        (body_profile(cover) / ref_).min(1.0)
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

    /// The Depth slider of a row: its bare `0..1` track maps to the `-1..1` domain, `0.5` being the
    /// zero (the two halves mean opposite things). One law for the 2D tool and the 3D piece's panel.
    pub fn set_impasto_depth_norm(&mut self, id: LayerId, norm: f32) {
        let depth = norm.clamp(0.0, 1.0).mul_add(2.0, -1.0); // CLAMP-OK: 0..1 track → -1..1 domain
        self.set_impasto_depth(id, depth);
    }

    /// The `Add`/`Level` chip of a row flips the mode. No-op if `id` unknown.
    pub fn toggle_impasto_composite(&mut self, id: LayerId) {
        let next = match self.get(id).map(|l| l.impasto_composite) {
            Some(ReliefComposite::Add) => ReliefComposite::Level,
            Some(ReliefComposite::Level) => ReliefComposite::Add,
            None => return,
        };
        self.set_impasto_composite(id, next);
    }
}

#[cfg(test)]
#[path = "relief_fold_tests.rs"]
mod tests;
