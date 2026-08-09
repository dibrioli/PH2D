//! **Cena 103 — OS TRÊS MODOS** (`PH2D_PHYSICS_SMOKE=103`, W-KinPure).
//!
//! Três pistas empilhadas, o MESMO percurso em todas, e um dedo só para os três
//! personagens (`hand_input_to_players`) — então tudo o que difere entre elas é
//! o modo.
//!
//! - **em cima, VERDE** — `Pure`, o puro sangue. É o sujeito.
//! - **no meio, LARANJA** — `Kinematic` (Snap).
//! - **em baixo, CIANO** — `Dynamic` (Spring).
//!
//! ⚠️ **As duas de baixo são o CONTROLE, e não são decoração:** um zero medido
//! num personagem que não empurra é indistinguível de um zero medido numa cena
//! que não tem o que empurrar.
//!
//! O percurso responde às quatro perguntas da wave de uma passada: o CAIXOTE (o
//! empurrão lateral), a PLATAFORMA PENDURADA (o peso), a PAREDE (ele continua
//! sólido) e o próprio andar/pular (ele continua o mesmo controlador).
//!
//! ⚠️ **E a plataforma é a cena que faltava à W-KinWeight:** ela fechou sem cena
//! nenhuma, contra a política do plano 07 §6, e o passo 5 daqui é o dela — a
//! massa autorada muda QUANTO a plataforma afunda.
//!
//! Os números da mensagem saem do irmão `physics_smoke_kin_pure_tests.rs`, que
//! dirige esta MESMA cena headless.

use ph2d_core::Vec2;
use ph2d_ecs::{Name, Transform, World, stable_name_id};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, JointKind, LockRotation, PhysicsJoint, PlatformPlayer,
    PlayerMode, RigidBody,
};
use ph2d_render::{Sprite, WHITE_TILE_KEY};

use crate::physics_smoke_player::slab;

/// A altura de flutuação — a mesma das outras cenas de player.
const FLOAT: f32 = 0.9;
/// **A altura de repouso do modo cinemático, MEDIDA** — a cápsula pousa e o
/// controlador guarda a própria pele. O número é o da cena 101.
const SNAP_REST: f32 = 0.5566;

/// O `y` do TOPO do chão de cada pista, de cima para baixo.
pub(crate) const LANE_TOP: [f32; 3] = [6.0, 0.0, -6.0];
/// Onde cada obstáculo nasce.
pub(crate) const CRATE_X: f32 = 3.0;
/// A metade PENDURADA do chão — o artista anda para cima dela.
pub(crate) const SLING_X: f32 = 9.0;
pub(crate) const SLING_HALF: f32 = 3.0;
/// A que altura o poste fica acima do centro da prancha — e, por construção, o
/// comprimento de repouso da mola.
const SLING_RISE: f32 = 3.25;
pub(crate) const WALL_X: f32 = 14.0;

const CRATE_HALF: f32 = 0.3;

/// Os três modos, na ordem em que a cena os empilha.
const LANES: [(&str, PlayerMode, [f32; 4]); 3] = [
    ("Pure", PlayerMode::Pure, [0.35, 0.9, 0.4, 1.0]),
    ("Snap", PlayerMode::Kinematic, [1.0, 0.65, 0.25, 1.0]),
    ("Spring", PlayerMode::Dynamic, [0.25, 0.85, 1.0, 1.0]),
];

/// Uma prancha pendurada por uma MOLA num poste estático acima dela.
///
/// ⚠️ **Ela é a segunda metade do CHÃO da pista**, e não um brinquedo ao lado:
/// o personagem tem de ANDAR para cima dela, senão o peso dele nunca a alcança.
/// A `LockRotation` mantém-na plana — sem isso o gate mediria tombo, não peso.
fn sling(world: &mut World, tag: &str, top: f32) {
    let anchor = format!("Sling Post {tag}");
    let plank = format!("Sling {tag}");
    slab(
        world,
        &anchor,
        Vec2::new(SLING_X, top - 0.25 + SLING_RISE),
        [0.15, 0.15],
        0.0,
        [0.4, 0.4, 0.45, 1.0],
    );
    world.spawn((
        Name::new(plank.clone()),
        Transform::from_translation(Vec2::new(SLING_X, top - 0.25)),
        Sprite::atlas(
            WHITE_TILE_KEY,
            [SLING_HALF * 2.0, 0.5],
            [0.55, 0.5, 0.62, 1.0],
        ),
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: SLING_HALF,
                half_y: 0.25,
            },
            ..Collider::default()
        },
        LockRotation,
        // ⚠️ **Sem peso PRÓPRIO, e é o que faz dela um oráculo:** todo milímetro
        // que ela desce é do personagem. Com peso próprio a mola sagaria sozinha
        // e o comprimento de repouso teria de ser corrigido por uma aritmética
        // que ninguém consegue reler — a mesma razão do `GravityScale(0)` da
        // jangada nas fixtures desta wave.
        ph2d_physics_ecs::GravityScale(0.0),
    ));
    world.spawn((
        Name::new(format!("Sling Rope {tag}")),
        Transform::from_translation(Vec2::new(SLING_X, top - 0.25 + SLING_RISE)),
        PhysicsJoint {
            body_a: stable_name_id(&anchor),
            body_b: stable_name_id(&plank),
            kind: JointKind::Spring,
            // O repouso É a altura do poste, então a prancha nasce nivelada com
            // o chão firme em vez de subir para o lugar dela na frente do
            // artista.
            rest_length: SLING_RISE,
            ..PhysicsJoint::of_kind(JointKind::Spring)
        },
    ));
}

/// Monta a cena — extraída para o irmão headless poder dirigir a MESMA coisa.
pub(crate) fn build(world: &mut World) {
    for (tag, mode, tint) in LANES {
        let top = LANE_TOP[LANES.iter().position(|l| l.0 == tag).unwrap()];
        // ⚠️ **O chão é firme, PENDURADO, firme** — nessa ordem, e a prancha
        // fica no MEIO do percurso de propósito: o artista tem de andar para
        // cima dela, e a parede do fim precisa de chão sob si (uma parede a
        // flutuar seria um obstáculo que a cena não explica).
        slab(
            world,
            &format!("Floor {tag}"),
            Vec2::new(2.5, top - 0.5),
            [3.5, 0.5],
            0.0,
            [0.35, 0.35, 0.4, 1.0],
        );
        slab(
            world,
            &format!("Landing {tag}"),
            Vec2::new(14.0, top - 0.5),
            [2.0, 0.5],
            0.0,
            [0.35, 0.35, 0.4, 1.0],
        );
        slab(
            world,
            &format!("Wall {tag}"),
            Vec2::new(WALL_X, top + 1.0),
            [0.25, 1.0],
            0.0,
            [0.5, 0.32, 0.32, 1.0],
        );
        world.spawn((
            Name::new(format!("Crate {tag}")),
            Transform::from_translation(Vec2::new(CRATE_X, top + CRATE_HALF)),
            Sprite::atlas(
                WHITE_TILE_KEY,
                [CRATE_HALF * 2.0, CRATE_HALF * 2.0],
                [0.85, 0.78, 0.45, 1.0],
            ),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: CRATE_HALF,
                    half_y: CRATE_HALF,
                },
                ..Collider::default()
            },
            // ⚠️ Sem isto os caixotes TOMBAM ao serem empurrados e a cena passa a
            // medir tombo, não empurrão.
            LockRotation,
        ));
        sling(world, tag, top);

        let kinematic = mode.drives_itself();
        let mut e = world.spawn((
            Name::new(tag.to_string()),
            Transform::from_translation(Vec2::new(
                0.0,
                top + if kinematic { SNAP_REST } else { FLOAT },
            )),
            Sprite::atlas(WHITE_TILE_KEY, [0.4, 1.0], tint),
            RigidBody {
                kind: if kinematic {
                    BodyKind::Kinematic
                } else {
                    BodyKind::Dynamic
                },
            },
            Collider {
                shape: ColliderShape::Capsule {
                    half_height: 0.3,
                    radius: 0.2,
                },
                ..Collider::default()
            },
            LockRotation,
            PlatformPlayer {
                float_height: FLOAT,
                ..PlatformPlayer::default()
            },
        ));
        // ⚠️ **Os DOIS campos, como o chip da §14 os escreve** — escrever só um
        // montaria um estado que o gesto do artista não produz.
        if mode != PlayerMode::default() {
            e.insert(mode);
        }
    }
}

impl crate::App {
    pub(crate) fn physics_smoke_kin_pure(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        build(gfx.sim.world_mut());

        eprintln!(
            "[physics-smoke 103] OS TRES MODOS (W-KinPure). Tres pistas com o\n\
             MESMO percurso: VERDE em cima = Pure (o puro sangue), LARANJA no\n\
             meio = Kinematic, CIANO em baixo = Dynamic. Um dedo so' para os\n\
             tres.\n\
             \n\
             ⚠️ Se a linha acima nao aparecer, pare: a cena nao montou.\n\
             \n\
             1) ANDE PARA A DIREITA (seta ->). No CAIXOTE (x = 3) as duas pistas\n\
                de baixo empurram e a de CIMA nao: o verde PARA nele e o\n\
                caixote fica onde estava. Medido em 3 s: ciano 8,70 m,\n\
                laranja 8,70 m, VERDE 0,0000.\n\
             \n\
             2) A PLATAFORMA PENDURADA (x = 9) e' a segunda metade do chao. Suba\n\
                nela: ela AFUNDA sob o ciano e o laranja e NAO se move sob o\n\
                verde. Medido: ciano -0,4036 m, laranja -0,3684, VERDE 0,0000.\n\
                E' o PESO -- a outra metade da 3a lei, a que um caixote nao\n\
                mostra. (A prancha nao tem peso proprio, entao todo milimetro\n\
                que ela desce e' do personagem.)\n\
             \n\
             3) A PAREDE (x = 13) para os TRES. Cenario nao quer dizer fantasma:\n\
                o puro sangue e' solido, o mundo so' nao lhe obedece.\n\
             \n\
             4) PULE (espaco) nas tres pistas. O verde pula IGUAL ao laranja --\n\
                e' o mesmo controlador com dois canais calados, nao um segundo\n\
                personagem. (Medido bit a bit num gate, nao no olho.)\n\
             \n\
             5) O CARD QUE SOME -- faca este passo, e' a metade que prova o\n\
                canal: selecione o LARANJA e veja 'Platform Player' > REACTION\n\
                (tres sliders). Agora selecione o VERDE: o card NAO existe. Nada\n\
                ali seria lido, e um slider inerte ensina a desconfiar dos\n\
                outros. Troque o chip 'Body' do verde para Kinematic e o card\n\
                volta COM OS NUMEROS QUE LA' ESTAVAM -- o modo cala, nao apaga.\n\
             \n\
             6) A MASSA (esta e' a cena que faltava a' W-KinWeight): com o\n\
                LARANJA em cima da plataforma, va' a 'Physics Body' > Mass:\n\
                Manual e suba a massa. A plataforma tem de afundar MAIS. No\n\
                VERDE esse par Auto/Manual nem e' oferecido -- sob o puro sangue\n\
                ninguem le' a massa."
        );
    }
}

#[cfg(test)]
#[path = "physics_smoke_kin_pure_tests.rs"]
mod tests;
