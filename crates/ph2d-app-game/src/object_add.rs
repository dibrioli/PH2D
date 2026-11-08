//! ⭐ **Os objectos do jogo de plataforma no menu Add** (`docs/GameEngineFuncional/05_N1_plano.md` §1).
//!
//! Cada um é um vazio com um conjunto FIXO de componentes que já existem. O que tem `Default` entra pela
//! porta do `+` do Inspector (`ph2d_app_components::component_attach::attach_by_name`: cascata e semente);
//! a `Sprite` (sem `Default`) e os valores próprios do papel entram antes, para as sementes os lerem.
//!
//! ⚠️ As duas portas — o vazio na raiz e o `+` — são de OUTRA família e chegam por [`Portas`], entregues
//! pela composição (ADR-0075: uma família não chama outra; o precedente é o `spawn_empty_root` que o Motion e
//! as Macros recebem).

use ph2d_ecs::{
    Entity, RepeatMode, ScrollFactor, ScrollRepeat, SimWorld, SpriteAnimations, SpriteAnimator,
    TextureRepeat, UvTransform, ZIndexOverride,
};
use ph2d_editor_core::object_add::AddEntry;
use ph2d_physics_ecs::{BodyKind, CharacterAnimations, ColliderShape, RigidBody};
use ph2d_render::{FactoryArt, HERO_SHEET_SOLE_BELOW_CENTRE, HILLS_TILE, Sprite, WHITE_TILE_KEY};

/// As entradas e o ensino do menu Add moram na folha de dados (`ph2d-game-dicas`), pela catraca de LOC.
pub use ph2d_game_dicas::add::{
    ANIMATED_SPRITE, BACKGROUND_LAYER, CHARACTER, COLLISION_SHAPE, ENTRIES, SPRITE, STATIC_BODY,
    TEACH,
};

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
    art.native_m()
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
        UvTransform {
            scale: [ladrilhos, 1.0],
            offset: [0.0, 0.0],
        },
        TextureRepeat(RepeatMode::Enabled),
        ScrollFactor { k: [k, k] },
        ScrollRepeat { tile: [lw, 0.0] },
    ));
}

type Nasceu = Option<Result<u64, String>>;

/// As portas de OUTRA família que o menu usa, entregues pela composição.
#[derive(Clone, Copy)]
pub struct Portas<'a> {
    /// O vazio na raiz com o nome dado (tornado único) — `ph2d_app_components::object_add::spawn_empty_root`.
    pub raiz: fn(&mut SimWorld, &str) -> u64,
    /// O `+` do Inspector: anexa pelo nome canónico, com a cascata e a semente da família dona
    /// (`ph2d_app_components::component_attach::attach_by_name` sobre o registo e as sementes do app).
    pub anexa: &'a dyn Fn(&mut SimWorld, u64, &str) -> Result<(), String>,
}

/// ⭐ **Cria o objecto** desta entrada (`None` se não é desta família; se um componente não entra, o objecto
/// sai). `pai` = o seleccionado: a Collision Shape segue [`ph2d_physics_ecs::pendura_forma`] (sem selecção nasce
/// um «Static Body» com ela por filha) e o devolvido é a FORMA. Os corpos nascem com a forma em filha.
pub fn add(entry: AddEntry, sim: &mut SimWorld, portas: Portas<'_>, pai: Option<u64>) -> Nasceu {
    if !ENTRIES.contains(&entry) {
        return None;
    }
    let solta = entry == COLLISION_SHAPE && pai.is_none();
    let bits = (portas.raiz)(sim, if solta { STATIC_BODY } else { entry }.key.tr());
    let e = Entity::from_bits(bits);
    let attach = |sim: &mut SimWorld, name: &str| (portas.anexa)(sim, bits, name);
    let r = (|| -> Result<u64, String> {
        let world = sim.world_mut();
        if entry == SPRITE {
            world
                .entity_mut(e)
                .insert(Sprite::atlas(WHITE_TILE_KEY, [1.0, 1.0], BRANCO));
        } else if entry == ANIMATED_SPRITE {
            // Nasce vazio: o cartão Image pede a folha e oferece o herói (ficha 02, decisão 3).
            let mut animator = SpriteAnimator::new("");
            animator.autoplay = true;
            world.entity_mut(e).insert((
                Sprite::atlas(WHITE_TILE_KEY, [1.0, 1.0], BRANCO),
                ph2d_ecs::SpriteGrid::SINGLE,
                SpriteAnimations::new(),
                animator,
            ));
        } else if entry == STATIC_BODY {
            world.entity_mut(e).insert((
                Sprite::atlas(WHITE_TILE_KEY, BLOCO, PEDRA),
                RigidBody {
                    kind: BodyKind::Static,
                },
            ));
            let [w, h] = BLOCO;
            let caixa = ColliderShape::Cuboid {
                half_x: w / 2.0,
                half_y: h / 2.0,
            };
            ph2d_physics_ecs::nasce_com_forma(world, e, caixa);
        } else if entry == BACKGROUND_LAYER {
            veste_camada(world, e, HILLS_TILE, CAMADA_LADRILHOS, CAMADA_K);
            world.entity_mut(e).insert(ZIndexOverride(-1));
        } else if entry == CHARACTER {
            ph2d_render::veste_o_heroi(world, e)?;
            world.entity_mut(e).insert(CharacterAnimations::default());
            let forma = ph2d_physics_ecs::nasce_com_forma(world, e, CAPSULA);
            let sem_atrito = ph2d_physics_ecs::MaterialCombine::sem_atrito(CAPSULA);
            world.entity_mut(forma).insert(sem_atrito);
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
            let forma = ph2d_physics_ecs::pendura_forma(world, e, pai.map(Entity::from_bits));
            return Ok(forma.to_bits());
        }
        Ok(bits)
    })();
    Some(r.inspect_err(|_| {
        sim.world_mut().despawn(e);
    }))
}
