//! **O MASTRO TELESCÓPICO** (`PH2D_PHYSICS_SMOKE=77`, W-RailRope).
//!
//! O W-LeadDrag ensinou o rig a ser arrastado como uma corda, e deixou UM tipo
//! de elo de fora com o motivo escrito: *"a lei é ANGULAR e só o Pin oferece o
//! ângulo que ela escolhe; um Slider desliza ao longo de um EIXO, que é outra
//! coordenada e pediria outra lei"*.
//!
//! ⚠️ **A lei não era outra — era a MESMA na outra coordenada.** O princípio é
//! *use a sua única liberdade para atrapalhar o menos possível*, e ele tem uma
//! resposta em cada grau de liberdade: a dobradiça escolhe o **ângulo**, o trilho
//! escolhe o **deslize**.
//!
//! Esta cena mostra as duas metades da lei lado a lado, e a terceira pista é o
//! CONTROLE sem o qual nenhuma das duas diz nada.

use ph2d_core::Vec2;
use ph2d_ecs::{Name, Transform, World, stable_name_id};
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, JointKind, PhysicsJoint, RigidBody};
use ph2d_render::{Sprite, WHITE_TILE_KEY};

const RAIL_RGBA: [f32; 4] = [0.45, 0.80, 0.95, 1.0];
const CROSS_RGBA: [f32; 4] = [0.98, 0.75, 0.25, 1.0];
const WELD_RGBA: [f32; 4] = [0.55, 0.58, 0.62, 1.0];

const CAMERA_CENTRE: [f32; 2] = [1.5, 0.0];
const CAMERA_HEIGHT: f32 = 9.0;

/// O curso de cada trilho, metros — o número que o perfil `[2,0 · 1,5 · 1,0 ·
/// 0,5]` da medição sai. **É o default do tipo** (`PhysicsJoint::DEFAULT_STROKE`
/// dá `±0,5`), então a cena não inventa número nenhum: ela só LIGA os batentes,
/// que nascem desligados.
const STROKE: f32 = 0.5;

/// A altura de cada pista.
pub(crate) const LANE_Y: [f32; 3] = [2.4, 0.0, -2.4];
pub(crate) const LANE_NAMES: [&str; 3] = ["Rail", "Cross", "Weld"];

fn link(world: &mut World, name: &str, x: f32, y: f32, rgba: [f32; 4]) {
    world.spawn((
        Name::new(name),
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 0.45,
                half_y: 0.12,
            },
            ..Collider::default()
        },
        Sprite::atlas(WHITE_TILE_KEY, [0.9, 0.24], rgba),
        Transform::from_translation(Vec2::new(x, y)),
    ));
}

fn tie(world: &mut World, a: &str, b: &str, kind: JointKind, at: [f32; 2], limited: bool) {
    world.spawn((
        Name::new(format!("J-{a}-{b}")),
        PhysicsJoint {
            body_a: stable_name_id(a),
            body_b: stable_name_id(b),
            kind,
            // ⚠️ Os batentes nascem DESLIGADOS (`limits_enabled: false`), e é
            // ligá-los que faz cada trilho comer o próprio curso em vez de o
            // primeiro absorver a puxada inteira.
            limits_enabled: limited,
            limit_min: -STROKE,
            limit_max: STROKE,
            ..PhysicsJoint::of_kind(kind)
        },
        Transform::from_translation(Vec2::new(at[0], at[1])),
    ));
}

/// Uma corrente de quatro elos de 1 m em +X, presa na altura `y`.
fn chain(world: &mut World, lane: usize, kind: JointKind, rgba: [f32; 4], limited: bool) {
    let y = LANE_Y[lane];
    let p = LANE_NAMES[lane];
    for i in 0..4 {
        link(world, &format!("{p}{i}"), i as f32, y, rgba);
    }
    for i in 0..3 {
        tie(
            world,
            &format!("{p}{i}"),
            &format!("{p}{}", i + 1),
            kind,
            [i as f32 + 0.5, y],
            limited,
        );
    }
}

pub(crate) fn build_rail_rope_scene(world: &mut World) {
    // 1. TRILHOS com curso: puxe a cabeça PELO eixo (para os lados) e cada elo
    //    come meio metro antes de arrastar o vizinho.
    chain(world, 0, JointKind::Slider, RAIL_RGBA, true);
    // 2. Os MESMOS trilhos: puxe a cabeça DE TRAVÉS (para cima ou para baixo) e a
    //    corrente vai inteira — um rail não tem liberdade na perpendicular.
    chain(world, 1, JointKind::Slider, CROSS_RGBA, true);
    // 3. O CONTROLE: uma corrente SOLDADA. Ela vai inteira em QUALQUER direção, e
    //    sem ela a cena não distingue *o trilho desliza* de *tudo é rígido*.
    chain(world, 2, JointKind::Weld, WELD_RGBA, false);
}

#[cfg(test)]
#[path = "physics_smoke_rail_rope_tests.rs"]
mod tests;

impl crate::App {
    /// **Cena 77 (W-RailRope).** O trilho como elo de corda.
    pub(crate) fn physics_smoke_rail_rope(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        build_rail_rope_scene(gfx.sim.world_mut());
        gfx.camera.center = CAMERA_CENTRE;
        gfx.camera.height_world = CAMERA_HEIGHT;
        if let Some(hero) = gfx.hero_screen.as_mut() {
            hero.panel_visibility.insert("physics", true);
        }

        eprintln!(
            "[physics-smoke 77] O MASTRO TELESCOPICO -- o trilho como elo de corda.\n\n  \
               O W-LeadDrag ensinou o rig a ser arrastado como uma CORDA e deixou o\n  \
               Slider de fora, com a nota dizendo que a lei e' ANGULAR e que um trilho\n  \
               'pediria outra lei'. A lei nao era outra: e' a MESMA na outra coordenada.\n  \
               A dobradica escolhe o ANGULO que mantem o ponto apontado; o trilho\n  \
               escolhe o DESLIZE.\n\n  \
               Tres correntes de quatro elos, cada uma com o curso de {stroke:.1} m por\n  \
               junta. Segure **Alt** e arraste a cabeca (o elo mais a ESQUERDA):\n     \
                  - EM CIMA '{n0}' (azul) -- arraste PARA OS LADOS, ao longo do trilho.\n       \
                    Cada elo come meio metro de curso e so' entao arrasta o vizinho: um\n       \
                    MASTRO TELESCOPICO. Medido numa puxada de 2 m: os quatro andam\n       \
                    2,0 / 1,5 / 1,0 / 0,5 -- o perfil decai da mao para a cauda, que e'\n       \
                    o que arrastar uma corda parece.\n     \
                  - MEIO '{n1}' (ambar) -- os MESMOS trilhos, arrastados PARA CIMA ou\n       \
                    PARA BAIXO. A corrente vai INTEIRA (2,0 / 2,0 / 2,0 / 2,0): um rail\n       \
                    nao tem liberdade na perpendicular, e essa metade e' o que impede a\n       \
                    lei de virar 'deslize sempre'.\n     \
                  - EMBAIXO '{n2}' (cinza) -- o CONTROLE, uma corrente SOLDADA. Ela vai\n       \
                    inteira em QUALQUER direcao. Sem ela a cena nao distingue *o trilho\n       \
                    desliza* de *tudo e' rigido*.\n\n  \
               (!) O CURSO e' load-bearing, e da' para ver: selecione uma junta da pista\n      \
                   de cima e DESMARQUE 'Limits' na secao Physics Joint. Agora o primeiro\n      \
                   trilho absorve a puxada INTEIRA e nada atras dele se mexe -- que e' o\n      \
                   que um rail sem batente de fato faz.\n\n  \
               (!) O arrasto tem MEMORIA (a corda e' funcao do CAMINHO): vai ate' o fim\n      \
                   do curso, volta, e vai de novo -- o carrinho fica onde a SOMA dos\n      \
                   movimentos o pos, nao onde a posicao final do cursor sugere.\n",
            stroke = STROKE,
            n0 = LANE_NAMES[0],
            n1 = LANE_NAMES[1],
            n2 = LANE_NAMES[2],
        );
    }
}
