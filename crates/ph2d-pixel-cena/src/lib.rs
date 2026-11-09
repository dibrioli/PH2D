//! **O Pixel Sprite na cena** ([P14 §4](../../../docs/PixelArt/P14_plano.md)): o documento Pixel é a
//! verdade; o motor lê DERIVADOS que nunca se gravam (os clips, a forma por quadro). Funções puras;
//! a família `ph2d-app-pixel` compõe-nas.

use ph2d_ecs::scene::registry::ComponentRegistry;
use ph2d_ecs::{
    AnimTagError, Entity, FrameRef, PixelSprite, SpriteAnimations, SpriteAnimator, SpriteGrid,
    World, tag_do_aseprite,
};
use ph2d_pixel_core::{FrameId, Sprite};

mod deriva;
#[doc(hidden)]
pub mod fixture;

pub use deriva::{CLIP_SEM_TAGS, DerivadoDe, animador_de_nascenca, deriva, grelha};

/// As tags do documento como clips do motor, pela porta única [`tag_do_aseprite`] (a mesma do
/// import de `.ase`). Sem tags ⇒ uma que cobre todos os quadros, chamada `nome_sem_tags`.
#[must_use]
pub fn clips_das_tags(sprite: &Sprite, nome_sem_tags: &str) -> SpriteAnimations {
    clips_e_recusas(sprite, nome_sem_tags).0
}

/// [`clips_das_tags`] mais as tags que a §11 recusou (nome repetido, vazio, lista cheia), cada uma
/// com o motivo — quem mostra ao artista lê esta.
#[must_use]
pub fn clips_e_recusas(
    sprite: &Sprite,
    nome_sem_tags: &str,
) -> (SpriteAnimations, Vec<(String, AnimTagError)>) {
    let duracoes: Vec<u32> = sprite.frames().iter().map(|f| f.duration_ms).collect();
    let mut lib = SpriteAnimations::new();
    let mut recusas = Vec::new();
    if sprite.tags().is_empty() {
        let last = duracoes.len().saturating_sub(1) as u32;
        let _ = lib.insert(tag_do_aseprite(nome_sem_tags, 0, last, 0, 0, &duracoes));
        return (lib, recusas);
    }
    for t in sprite.tags() {
        let tag = tag_do_aseprite(
            &t.name,
            t.from,
            t.to,
            t.direction.code(),
            t.repeat,
            &duracoes,
        );
        if let Err(e) = lib.insert(tag) {
            recusas.push((t.name.clone(), e));
        }
    }
    (lib, recusas)
}

/// **Que quadro este Pixel Sprite mostra agora** — a porta única de desenho, luz, collider e pivô.
///
/// Com um [`SpriteAnimator`] a tocar (ou que acabou a tag, e fica no último quadro dela) o quadro é
/// a célula viva da [`SpriteGrid`] — a que o tique escreve; senão o `quadro_do_editor`.
#[must_use]
pub fn quadro_mostrado(
    world: &World,
    e: Entity,
    sprite: &Sprite,
    quadro_do_editor: FrameId,
) -> FrameId {
    let Ok(er) = world.get_entity(e) else {
        return quadro_do_editor;
    };
    let (Some(anim), Some(grid)) = (er.get::<SpriteAnimator>(), er.get::<SpriteGrid>()) else {
        return quadro_do_editor;
    };
    let acabou = || {
        er.get::<SpriteAnimations>()
            .and_then(|l| l.get(&anim.current))
            .is_some_and(|t| anim.is_finished(t))
    };
    if !anim.playing && !acabou() {
        return quadro_do_editor;
    }
    match grid.shown() {
        FrameRef::Cell { sheet: 0, cell } => sprite
            .frames()
            .get(cell as usize)
            .map_or(quadro_do_editor, |f| f.id),
        _ => quadro_do_editor,
    }
}

/// **O quadro que o desenho do Pixel Sprite pinta**: em Draw (`em_edicao`) o do editor, que o
/// artista está a pintar; fora dele, [`quadro_mostrado`] (o do animador em Play).
#[must_use]
pub fn quadro_a_desenhar(
    world: &World,
    e: Entity,
    sprite: &Sprite,
    quadro_do_editor: FrameId,
    em_edicao: bool,
) -> FrameId {
    if em_edicao {
        quadro_do_editor
    } else {
        quadro_mostrado(world, e, sprite, quadro_do_editor)
    }
}

/// O registo sabe que, no Pixel Sprite, os clips e a grelha são derivados das tags: não se gravam.
pub fn registar(reg: &mut ComponentRegistry) {
    reg.derivado_quando::<SpriteAnimations, PixelSprite>();
    reg.derivado_quando::<SpriteGrid, PixelSprite>();
}
