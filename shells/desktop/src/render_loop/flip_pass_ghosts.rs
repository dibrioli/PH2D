//! **Ghost Frames** (ADR-0114 W3.T3.3) — QUEM aparece como fantasma.
//!
//! O *quem* sai da função pura `ph2d_flip::ghosts` (port do `get_frame_id`, testada
//! headless nos 3 modos). Este módulo só aplica os **gates** e traduz o resultado
//! para o que o compositor precisa.
//!
//! O *como* é do [`super::flip_pass`]: cada fantasma vira uma **fatia da pilha de
//! camadas**, inserida logo ABAIXO da camada a que pertence. Isso importa — a
//! primeira versão desenhava todos os fantasmas num passe por baixo de TUDO, e
//! bastava uma camada de fundo opaca (o retângulo amarelo do demo) para engolir o
//! fantasma da camada de cima. O fantasma pertence à sua camada; o z dele é o dela.
//!
//! Gates (todos do GP, `02_referencia §8`):
//! - **some no play** — durante a reprodução o fantasma é ruído puro;
//! - `onion.enabled` por objeto e `use_onion` por camada (o fundo não vira ghost);
//! - camada invisível não gera fantasma (não se vê o que não está lá).

use ph2d_core::Playhead;
use ph2d_flip::{FlipDrawing, FlipLayer, FlipObject, Frame};

/// Um fantasma pronto para virar fatia: a arte, sua chave de cache e como pintá-la.
pub(super) struct GhostRef<'a> {
    /// A arte do desenho vizinho (a mesma cobertura; só a cor muda).
    pub(super) drawing: &'a FlipDrawing,
    /// `DrawingId.0` — a chave do cache de tesselação (um fantasma de um desenho já
    /// visitado não re-empacota nada).
    pub(super) drawing_id: u32,
    /// Distância à corrente (negativo = passado). Entra na chave do compositor.
    pub(super) delta: i32,
    /// Cor da silhueta (verde = passado, azul = futuro).
    pub(super) tint: [f32; 3],
    /// Opacidade final (fade `1/|Δ|` × opacidade do onion × opacidade da camada).
    pub(super) alpha: f32,
}

/// Os fantasmas da camada `layer` no quadro `frame`, ou vazio se algum gate barra.
/// `selected` são as chaves marcadas na tira (só o modo `Selected` as lê).
pub(super) fn collect<'a>(
    obj: &'a FlipObject,
    layer: &FlipLayer,
    frame: Frame,
    playhead: &Playhead,
    selected: &[Frame],
) -> Vec<GhostRef<'a>> {
    if playhead.is_playing() || !obj.onion.enabled || !layer.use_onion || !layer.visible {
        return Vec::new();
    }
    ph2d_flip::ghosts(layer, frame, &obj.onion, selected)
        .into_iter()
        .filter_map(|g| {
            let art = obj.drawing(g.drawing)?;
            if art.strokes.is_empty() {
                return None; // um desenho vazio não tem silhueta
            }
            Some(GhostRef {
                drawing: art,
                drawing_id: g.drawing.0,
                delta: g.delta,
                tint: [g.tint.r(), g.tint.g(), g.tint.b()],
                // A opacidade da CAMADA entra aqui (e não pelo compositor): a fatia
                // do fantasma compõe em Normal/1.0, senão o blend da camada (um
                // Multiply, por exemplo) tingiria o fantasma com a arte de baixo.
                alpha: g.alpha * layer.opacity,
            })
        })
        .collect()
}
