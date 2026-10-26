//! ⭐ **Os objectos do jogo de plataforma no menu Add** (`docs/GameEngineFuncional/05_N1_plano.md` §1).
//!
//! Cada um é um vazio com um conjunto FIXO de componentes que já existem. O que tem `Default` entra pela
//! porta do `+` do Inspector ([`ph2d_app_components::component_attach::attach_by_name`]: cascata e semente);
//! a `Sprite` (sem `Default`) e os valores próprios do papel entram antes, para as sementes os lerem.

use ph2d_app_components::component_seed::SeedTable;
use ph2d_ecs::scene::ComponentRegistry;
use ph2d_ecs::{
    Entity, RepeatMode, ScrollFactor, ScrollRepeat, SimWorld, SpriteAnimations, SpriteAnimator,
    SpriteGrid, TextureRepeat, UvTransform, ZIndexOverride,
};
use ph2d_editor_core::object_add::{AddEntry, AddGroup};
use ph2d_physics_ecs::{BodyKind, CharacterAnimations, Collider, ColliderShape, RigidBody};
use ph2d_render::{
    BACKGROUND_TEXELS_PER_M, FactoryArt, HERO_SHEET_COLS, HERO_SHEET_KEY, HERO_SHEET_ROWS,
    HERO_SHEET_SOLE_BELOW_CENTRE, HERO_SHEET_TAGS, HILLS_TILE, Sprite, WHITE_TILE_KEY,
};

/// Uma imagem.
pub const SPRITE: AddEntry = AddEntry::new("object_add.game.sprite", AddGroup::Game)
    .with_tip("object_add.game.sprite.tip", crate::dicas::SPRITE);
/// Uma imagem que se anima por intervalos da folha.
pub const ANIMATED_SPRITE: AddEntry =
    AddEntry::new("object_add.game.animated_sprite", AddGroup::Game).with_tip(
        "object_add.game.animated_sprite.tip",
        crate::dicas::ANIMATED_SPRITE,
    );
/// Um bloco fixo em que as coisas batem (chão, plataforma, parede).
pub const STATIC_BODY: AddEntry = AddEntry::new("object_add.game.static_body", AddGroup::Game)
    .with_tip("object_add.game.static_body.tip", crate::dicas::STATIC_BODY);
/// O personagem que o jogador controla.
pub const CHARACTER: AddEntry = AddEntry::new("object_add.game.character", AddGroup::Game)
    .with_tip("object_add.game.character.tip", crate::dicas::CHARACTER);
/// Uma forma de colisão invisível — dentro de um corpo, é mais uma forma dele.
pub const COLLISION_SHAPE: AddEntry =
    AddEntry::new("object_add.game.collision_shape", AddGroup::Game).with_tip(
        "object_add.game.collision_shape.tip",
        crate::dicas::COLLISION_SHAPE,
    );
/// Uma camada de fundo que fica para trás quando a câmara anda (parallax) — plano 11.
pub const BACKGROUND_LAYER: AddEntry =
    AddEntry::new("object_add.game.background_layer", AddGroup::Game).with_tip(
        "object_add.game.background_layer.tip",
        crate::dicas::BACKGROUND_LAYER,
    );
/// O que esta família põe no menu, na ordem do modal.
pub const ENTRIES: &[AddEntry] = &[
    SPRITE,
    ANIMATED_SPRITE,
    STATIC_BODY,
    CHARACTER,
    COLLISION_SHAPE,
    BACKGROUND_LAYER,
];

const BRANCO: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
const PEDRA: [f32; 4] = [0.45, 0.47, 0.52, 1.0];
/// O bloco do *Static Body* nasce como uma plataforma (m).
pub const BLOCO: [f32; 2] = [3.0, 0.5];
/// A cápsula do personagem vai do corpo ao topo do capacete do boneco de fábrica (0,36 acima do centro); um
/// pouco mais estreita que ele (±0,2 contra ±0,27) porque o raio também tem de pairar numa rampa de 45° à
/// altura dos pés (`fitted_float`). As pernas desenhadas chegam ao chão.
pub const CAPSULA: ColliderShape = ColliderShape::Capsule {
    half_height: CAPSULA_MEIA,
    radius: 0.2,
};
const CAPSULA_MEIA: f32 = 0.15;
/// A altura agachada do personagem (m): a cápsula encolhe o troço recto inteiro (0,30) e o topo desce
/// de 0,80 para 0,50, com os pés onde a folha os desenha (plano 09).
pub const AGACHADO: f32 = HERO_SHEET_SOLE_BELOW_CENTRE - 2.0 * CAPSULA_MEIA;

/// A camada de fundo nasce com os montes ao longe: guarda 0,4 do movimento da câmara e desenha-se atrás do mundo.
pub const CAMADA_K: f32 = 0.4;
/// Quantos ladrilhos a sprite da camada desenha lado a lado — a vista mais larga (21:9 a 8 m ≈ 19 m) mais
/// um ladrilho de 8 m, para a repetição nunca deixar a borda à vista.
pub const CAMADA_LADRILHOS: f32 = 4.0;

/// O lado (m) de um ladrilho do fundo.
#[must_use]
pub fn ladrilho_m(art: FactoryArt) -> [f32; 2] {
    art.cell_px()
        .map(|v| v as f32 / BACKGROUND_TEXELS_PER_M as f32)
}

/// ⭐ Veste `e` como camada de fundo com o azulejo `art` repetido `ladrilhos` vezes na horizontal e a fracção
/// `k` da câmara (o que o artista faz no Inspector: a imagem, *Repeat*, a escala do UV e a secção Parallax).
pub fn veste_camada(
    world: &mut ph2d_ecs::World,
    e: Entity,
    art: FactoryArt,
    ladrilhos: f32,
    k: f32,
) {
    let [lw, lh] = ladrilho_m(art);
    world.entity_mut(e).insert((
        Sprite::atlas(art.key, [lw * ladrilhos, lh], BRANCO),
        SpriteGrid {
            hframes: art.grid[0],
            vframes: art.grid[1],
            frame: art.middle_frame(),
        },
        UvTransform {
            scale: [ladrilhos, 1.0],
            offset: [0.0, 0.0],
        },
        TextureRepeat(RepeatMode::Enabled),
        ScrollFactor { k: [k, k] },
        ScrollRepeat { tile: [lw, 0.0] },
    ));
}

/// A folha do boneco de fábrica, as suas animações e o animador a tocar `idle`.
fn anima(world: &mut ph2d_ecs::World, e: Entity) -> Result<(), String> {
    let mut tags = SpriteAnimations::new();
    for t in HERO_SHEET_TAGS {
        let mut tag = ph2d_ecs::AnimationTag::new(t.name, t.from, t.to);
        tag.frame_ms = t.per_frame_ms[0];
        // Só o que difere do ritmo é tempo próprio — senão o Frames/s não move a animação.
        for (i, ms) in t.per_frame_ms.iter().enumerate() {
            tag.set_own_ms(i, *ms);
        }
        tag.repeat = t.once.then_some(1);
        tags.insert(tag).map_err(|e| format!("{e:?}"))?;
    }
    let mut animator = SpriteAnimator::new(HERO_SHEET_TAGS[0].name);
    animator.autoplay = true;
    world.entity_mut(e).insert((
        Sprite::atlas(HERO_SHEET_KEY, [1.0, 1.0], BRANCO),
        SpriteGrid {
            hframes: HERO_SHEET_COLS,
            vframes: HERO_SHEET_ROWS,
            frame: 0,
        },
        tags,
        animator,
    ));
    Ok(())
}

/// ⭐ **Cria o objecto** desta entrada — `None` se a entrada não é desta família. Se um componente não
/// entra, o objecto sai (um *«Character»* sem personagem seria um nome que mente).
pub fn add(
    entry: AddEntry,
    sim: &mut SimWorld,
    registry: &ComponentRegistry,
    seeds: SeedTable<'_>,
) -> Option<Result<u64, String>> {
    if !ENTRIES.contains(&entry) {
        return None;
    }
    let bits = ph2d_app_components::object_add::spawn_empty_root(sim, entry.key.tr());
    let e = Entity::from_bits(bits);
    let attach = |sim: &mut SimWorld, name: &str| {
        ph2d_app_components::component_attach::attach_by_name(sim, registry, seeds, bits, name)
    };
    let r = (|| -> Result<(), String> {
        let world = sim.world_mut();
        if entry == SPRITE {
            world
                .entity_mut(e)
                .insert(Sprite::atlas(WHITE_TILE_KEY, [1.0, 1.0], BRANCO));
        } else if entry == ANIMATED_SPRITE {
            anima(world, e)?;
        } else if entry == STATIC_BODY {
            world.entity_mut(e).insert((
                Sprite::atlas(WHITE_TILE_KEY, BLOCO, PEDRA),
                RigidBody {
                    kind: BodyKind::Static,
                },
            ));
            attach(sim, "ph2d::physics::Collider")?;
        } else if entry == BACKGROUND_LAYER {
            veste_camada(world, e, HILLS_TILE, CAMADA_LADRILHOS, CAMADA_K);
            world.entity_mut(e).insert(ZIndexOverride(-1));
        } else if entry == CHARACTER {
            anima(world, e)?;
            world.entity_mut(e).insert((
                Collider {
                    shape: CAPSULA,
                    ..Collider::default()
                },
                CharacterAnimations::default(),
            ));
            attach(sim, "ph2d::physics::LockRotation")?;
            attach(sim, "ph2d::physics::PlatformPlayer")?;
            // Paira à altura dos pés desenhados (a semente parte de 0,5 m e só sobe).
            if let Some(mut p) = sim
                .world_mut()
                .get_mut::<ph2d_physics_ecs::PlatformPlayer>(e)
            {
                p.float_height = HERO_SHEET_SOLE_BELOW_CENTRE;
                p.crouch_height = AGACHADO;
            }
        } else {
            world.entity_mut(e).insert(Collider {
                shape: ColliderShape::Cuboid {
                    half_x: 0.5,
                    half_y: 0.5,
                },
                ..Collider::default()
            });
        }
        Ok(())
    })();
    Some(match r {
        Ok(()) => Ok(bits),
        Err(msg) => {
            sim.world_mut().despawn(e);
            Err(msg)
        }
    })
}

#[cfg(test)]
#[path = "object_add_tests.rs"]
mod tests;
