//! **Quem reescreve o derivado** ([P14 §4 A](../../../docs/PixelArt/P14_plano.md)): os clips e a
//! grelha do Pixel Sprite saem das tags do documento, e nunca se gravam (`registar`). O Stop e o
//! undo fazem respawn sem eles; esta passagem devolve-os no quadro seguinte.

use ph2d_ecs::{
    Component, Entity, PixelSprite, PresentComponent, SpriteAnimations, SpriteAnimator, SpriteGrid,
    World,
};
use ph2d_pixel_core::Sprite;

/// O clip de um documento sem tags: o nome do Godot para o `SpriteFrames` sem animações.
pub const CLIP_SEM_TAGS: &str = "default";

/// O `stamp` do documento de que saiu o derivado desta entidade. NÃO registado: some no respawn
/// (Stop, undo), e a ausência força a reescrita.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct DerivadoDe(pub u64);

impl PresentComponent for DerivadoDe {}

/// A grelha de `n` quadros numa linha, com a célula viva de `antes` (a do tique) quando cabe.
#[must_use]
pub fn grelha(n: u32, antes: Option<SpriteGrid>) -> SpriteGrid {
    let mut g = antes.unwrap_or_default();
    g.hframes = n.max(1);
    g.vframes = 1;
    if g.frame >= g.hframes {
        g.frame = 0;
    }
    g
}

/// Para cada Pixel Sprite cujo documento mudou (`stamp`) ou a quem falta o derivado: escreve
/// [`SpriteAnimations`] (de [`crate::clips_das_tags`]) e a [`SpriteGrid`] `n × 1`. `doc_de(chave)`
/// dá o documento e o `stamp` dele. O [`SpriteAnimator`] é do artista e não se toca.
pub fn deriva<'a>(world: &mut World, doc_de: impl Fn(u32) -> Option<(&'a Sprite, u64)>) {
    let mut q = world.query::<(
        Entity,
        &PixelSprite,
        Option<&DerivadoDe>,
        Option<&SpriteGrid>,
        Option<&SpriteAnimations>,
    )>();
    let mut fazer = Vec::new();
    for (e, ps, de, grid, clips) in q.iter(world) {
        let Some((sprite, stamp)) = doc_de(ps.0) else {
            continue;
        };
        if de.is_some_and(|d| d.0 == stamp) && grid.is_some() && clips.is_some() {
            continue;
        }
        fazer.push((e, sprite, stamp, grid.copied()));
    }
    for (e, sprite, stamp, antes) in fazer {
        let n = u32::try_from(sprite.frames().len()).unwrap_or(u32::MAX);
        let g = grelha(n, antes);
        let mut ent = world.entity_mut(e);
        ent.insert((
            crate::clips_das_tags(sprite, CLIP_SEM_TAGS),
            DerivadoDe(stamp),
        ));
        if antes != Some(g) {
            ent.insert(g);
        }
    }
}

/// O animador com que um Pixel Sprite NASCE: o do Animated Sprite do Game Engine (Autoplay ligado,
/// `current` vazio). Vazio começa no 1.º clip ([`SpriteAnimator::start_or_current`]), e renomear
/// uma tag não o deixa a apontar para nada.
#[must_use]
pub fn animador_de_nascenca() -> SpriteAnimator {
    let mut a = SpriteAnimator::new("");
    a.autoplay = true;
    a
}
