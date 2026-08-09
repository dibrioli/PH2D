//! **Cena 102 — O QUE ESTÁ AO LADO TAMBÉM** (`PH2D_PHYSICS_SMOKE=102`,
//! W-KinPush).
//!
//! Duas pistas empilhadas, e cada uma tem um personagem e o MESMO trio de
//! obstáculos: um caixote LEVE, um caixote PESADO e uma parede estática.
//!
//! - **em cima, LARANJA** — o modo Snap (cinemático). É o sujeito.
//! - **em baixo, CIANO** — o modo Spring (dinâmico). É o **CONTROLE**, e ele não
//!   é decoração: sob Spring o solver já empurra o que o personagem esbarra, e
//!   sem uma pista que empurra um caixote parado em cima não distingue *"a lei
//!   não funciona"* de *"o caixote é pesado demais"*.
//!
//! ⚠️ **Um dedo só para os dois** (`hand_input_to_players`), então tudo o que
//! difere entre as pistas é o modo.
//!
//! Os números da mensagem saem do irmão `physics_smoke_kin_push_tests.rs`, que
//! dirige esta MESMA cena headless — a política do plano 07 §6.

use ph2d_core::Vec2;
use ph2d_ecs::{Name, Transform, World};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, LockRotation, PlatformPlayer, PlayerMode, RigidBody,
};
use ph2d_render::{Sprite, WHITE_TILE_KEY};

use crate::physics_smoke_player::slab;

/// A altura de flutuação — a mesma das outras cenas de player.
const FLOAT: f32 = 0.9;
/// **A altura de repouso do modo Snap, MEDIDA** — a cápsula pousa e o
/// controlador guarda a própria pele. O número é o da cena 101.
const SNAP_REST: f32 = 0.5566;

/// O `y` do TOPO do chão de cada pista.
pub(crate) const LANE_TOP: [f32; 2] = [0.0, -6.0];
/// Onde cada obstáculo nasce, medido do começo da pista.
pub(crate) const LIGHT_X: f32 = 3.0;
pub(crate) const HEAVY_X: f32 = 7.0;
pub(crate) const WALL_X: f32 = 12.0;
/// ⚠️ **A razão de densidade é 16, e é ela que torna a coluna da MASSA legível:**
/// com 2× as duas colunas dão quase o mesmo número e a tabela não decide nada.
pub(crate) const HEAVY_DENSITY: f32 = 16.0;

const CRATE_HALF: f32 = 0.3;

fn crate_box(world: &mut World, name: &str, at: Vec2, density: f32, tint: [f32; 4]) {
    world.spawn((
        Name::new(name.to_string()),
        Transform::from_translation(at),
        Sprite::atlas(WHITE_TILE_KEY, [CRATE_HALF * 2.0, CRATE_HALF * 2.0], tint),
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: CRATE_HALF,
                half_y: CRATE_HALF,
            },
            density,
            ..Collider::default()
        },
        // ⚠️ Sem isto os caixotes TOMBAM ao serem empurrados e a cena passa a
        // medir tombo, não empurrão — o mesmo cuidado que a sonda desta wave
        // teve de tomar na jangada.
        LockRotation,
    ));
}

fn player(world: &mut World, name: &str, at: Vec2, tint: [f32; 4], kinematic: bool) {
    let mut e = world.spawn((
        Name::new(name.to_string()),
        Transform::from_translation(at),
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
    if kinematic {
        e.insert(PlayerMode::Kinematic);
    }
}

/// Monta a cena — extraída para o irmão headless poder dirigir a MESMA coisa.
pub(crate) fn build(world: &mut World) {
    for (lane, top) in LANE_TOP.iter().copied().enumerate() {
        let kinematic = lane == 0;
        let tag = if kinematic { "Snap" } else { "Spring" };
        slab(
            world,
            &format!("Floor {tag}"),
            Vec2::new(6.0, top - 0.5),
            [16.0, 0.5],
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
        crate_box(
            world,
            &format!("Light {tag}"),
            Vec2::new(LIGHT_X, top + CRATE_HALF),
            1.0,
            [0.85, 0.78, 0.45, 1.0],
        );
        crate_box(
            world,
            &format!("Heavy {tag}"),
            Vec2::new(HEAVY_X, top + CRATE_HALF),
            HEAVY_DENSITY,
            [0.45, 0.42, 0.38, 1.0],
        );
        player(
            world,
            tag,
            Vec2::new(0.0, top + if kinematic { SNAP_REST } else { FLOAT }),
            if kinematic {
                [1.0, 0.65, 0.25, 1.0]
            } else {
                [0.25, 0.85, 1.0, 1.0]
            },
            kinematic,
        );
    }
}

impl crate::App {
    pub(crate) fn physics_smoke_kin_push(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        build(gfx.sim.world_mut());

        eprintln!(
            "[physics-smoke 102] O QUE ESTA AO LADO TAMBEM (W-KinPush). Duas pistas\n\
             com o MESMO trio de obstaculos: LARANJA em cima = Snap (cinematico),\n\
             CIANO em baixo = Spring (dinamico, o CONTROLE). Um dedo so' para os\n\
             dois.\n\
             \n\
             ⚠️ Se a linha acima nao aparecer, pare: a cena nao montou.\n\
             \n\
             1) ANDE PARA A DIREITA (seta ->). Os DOIS tem de empurrar o caixote\n\
                claro a' frente deles. Medido NESTA cena em 3 s: ciano 6,39 m e\n\
                laranja 4,95 -- antes desta wave o laranja empurrava 0,0000 e\n\
                apenas PARAVA nele. (Os dois numeros diferem porque a parede\n\
                limita a viagem; em pista LIVRE eles medem 16,55 e 16,54.)\n\
             \n\
             2) O CAIXOTE ESCURO (x = 7) e' 16x mais denso. Ele anda bem MENOS\n\
                que o claro nas DUAS pistas -- medido, 1,55 contra 4,95 no\n\
                laranja e 2,99 contra 6,39 no ciano. A massa manda, e e' o sinal\n\
                de que isto e' um impulso e nao um teleporte.\n\
             \n\
             3) A PAREDE (x = 12) e' estatica: ela nao pode ceder um milimetro,\n\
                e os dois personagens tem de PARAR nela. Massa infinita absorve\n\
                o empurrao inteiro, e nao ha caso especial nenhum para isso.\n\
             \n\
             4) ENCOSTE E FIQUE. Com o personagem pressionado contra o caixote\n\
                preso na parede, nada pode VIBRAR: medida, a folga em regime e'\n\
                constante. Era este o risco da wave -- empurra, o caixote foge,\n\
                o slide segue, empurra outra vez.\n\
             \n\
             5) O KNOB -- faca este passo, e' a metade que prova o canal:\n\
                selecione o LARANJA, abra 'Platform Player' > REACTION e baixe\n\
                'Push on Bodies' de 1,00 para 0. Ande de novo: o laranja volta a\n\
                ser um fantasma de lado (ele PARA no caixote sem o mover) e o\n\
                ciano continua a empurrar. Suba de volta para 1,00 e ele volta.\n\
                ⚠️ No CIANO esse mesmo knob e' INERTE, e o rotulo o diz: um\n\
                corpo dinamico ja' empurra pelo solver."
        );
    }
}

#[cfg(test)]
#[path = "physics_smoke_kin_push_tests.rs"]
mod tests;
