//! **Cena 75 — O LIMITADOR** (W-RopeStop): a carga para antes da roldana, e a
//! roldana é selecionável clicando nela.
//!
//! Dois guinchos idênticos, lado a lado, com a MESMA taxa de recolhimento e a
//! mesma roldana. O da esquerda não tem limitador; o da direita tem. É o par que
//! torna o defeito e a cura visíveis na mesma tela — sem ele o artista veria uma
//! carga parando e não teria como saber do que foi salvo.

use bevy_ecs::world::World;
use ph2d_core::Vec2;
use ph2d_ecs::{Name, Transform, stable_name_id};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, JointKind, PhysicsJoint, PulleyWheel, RigidBody, RopeStops,
    WrapSide,
};
use ph2d_render::{Sprite, WHITE_TILE_KEY};

use crate::physics_smoke_pulley::ball;

/// A altura do eixo — a mesma dos dois guinchos.
const BOOM_Y: f32 = 6.0;
/// O raio da roldana. **Grande de propósito:** é ele que faz a diferença entre
/// *distância ao centro* e *folga de tangente* ser visível a olho.
const WHEEL_R: f32 = 0.7;
/// Onde a carga nasce.
const LOAD_Y: f32 = 0.4;
/// O limitador da corda da DIREITA, metros de corda.
const STOP_M: f32 = 1.6;

const FREE: [f32; 4] = [0.85, 0.35, 0.30, 1.0];
const HELD: [f32; 4] = [0.35, 0.72, 0.55, 1.0];
const POST: [f32; 4] = [0.45, 0.47, 0.55, 1.0];

/// Um guincho: poste morto, roldana com tambor, carga pendurada.
fn winch(world: &mut World, tag: &str, x: f32, stop: f32, rgba: [f32; 4]) {
    let (post, load, rope) = (
        format!("{tag} Post"),
        format!("{tag} Load"),
        format!("{tag} Rope"),
    );
    world.spawn((
        Name::new(post.clone()),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 0.15,
                half_y: 0.15,
            },
            ..Collider::default()
        },
        Sprite::atlas(WHITE_TILE_KEY, [0.3, 0.3], POST),
        Transform::from_translation(Vec2::new(x + 2.0, BOOM_Y)),
    ));
    ball(world, &load, x, 2.0, rgba, 0.3);
    {
        let mut q = world.query::<(&Name, &mut Transform)>();
        for (n, mut t) in q.iter_mut(world) {
            if n.as_str() == load {
                t.translation.y = LOAD_Y;
            }
        }
    }
    let mut e = world.spawn((
        Name::new(rope.clone()),
        PhysicsJoint {
            body_a: stable_name_id(&load),
            body_b: stable_name_id(&post),
            kind: JointKind::Pulley,
            ..PhysicsJoint::of_kind(JointKind::Pulley)
        },
        Transform::from_translation(Vec2::new(x, LOAD_Y)),
    ));
    // ⚠️ **O componente só é anexado quando há limitador**, para que o guincho da
    // esquerda seja literalmente o mundo de antes desta wave: ausente == zero ==
    // a trava no próprio aro.
    if stop > 0.0 {
        e.insert(RopeStops { a: stop, b: 0.0 });
    }
    world.spawn((
        Name::new(format!("{rope} Wheel 1")),
        PulleyWheel {
            rope: stable_name_id(&rope),
            order: 0,
            radius: WHEEL_R,
            wrap: WrapSide::Auto,
            motor_speed: 45.0f32.to_radians(),
            ..Default::default()
        },
        Transform::from_translation(Vec2::new(x, BOOM_Y)),
    ));
}

/// Monta a cena.
pub(crate) fn build(world: &mut World) {
    world.spawn((
        Name::new("Floor"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 16.0,
                half_y: 0.3,
            },
            ..Collider::default()
        },
        Sprite::atlas(WHITE_TILE_KEY, [32.0, 0.6], [0.40, 0.42, 0.48, 1.0]),
        Transform::from_translation(Vec2::new(0.0, -0.6)),
    ));
    winch(world, "Free", -4.0, 0.0, FREE);
    winch(world, "Held", 4.0, STOP_M, HELD);
}

#[cfg(test)]
#[path = "physics_smoke_stop_tests.rs"]
mod tests;

impl crate::App {
    /// **Cena 75 (W-RopeStop).** Dois guinchos, um com limitador e um sem.
    pub(crate) fn physics_smoke_stop(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        build(gfx.sim.world_mut());
        gfx.camera.center = [0.0, 3.5];
        gfx.camera.height_world = 12.0;
        if let Some(hero) = gfx.hero_screen.as_mut() {
            hero.panel_visibility.insert("physics", true);
        }
        eprintln!(
            "[physics-smoke 75] O LIMITADOR -- a carga para ANTES da roldana.\n  \
               Dois guinchos IDENTICOS: o VERMELHO (esquerda) sem limitador, o VERDE\n  \
               (direita) com um de {STOP_M} m. A corda, as roldanas e as marcas ja'\n  \
               estao na tela (o contorno nasce LIGADO; B o desliga).\n\n  \
               1. De PLAY e olhe as duas cargas subirem.\n     \
                  -> A VERMELHA entra na roldana. Passado esse ponto a rota fica\n        \
                     DEGENERADA e a corda DEIXA DE SEGURAR: a carga cai de volta, sem\n        \
                     erro e sem aviso. (medido: folga de tangente 0,0000 m, e depois a\n        \
                     carga voltando a descer: folga 1,18 m aos 10 s, 2,41 aos 12,5,\n        \
                     3,40 aos 15 -- ela foi DEVOLVIDA)\n     \
                  -> A VERDE PARA e fica PARADA: 1,5865 m, imovel do momento em que\n        \
                     encosta na marca ate' o fim (o limitador pedia 1,6 -- 0,8% de\n        \
                     esticamento da corda, o mesmo que o PULLEY_BIAS ja' documenta).\n\n  \
               2. PAUSE e olhe as marcas: um CIRCULO COM UM X, em cima da corda, em\n     \
                  cada ponta de cada polia. Na vermelha as duas estao coladas na\n     \
                  roldana -- e' assim que 'desligado' se parece: a trava esta' no\n     \
                  proprio aro, que e' onde a corda ja' podia chegar.\n\n  \
               3. ARRASTE a marca do guincho vermelho para BAIXO, pela corda. Ela anda\n     \
                  SOBRE o traço (nao sai dele nem com o cursor longe). De PLAY: agora\n     \
                  aquela carga tambem para, exatamente onde voce deixou a marca.\n\n  \
               4. CLIQUE NUMA RODA -- no ANEL dela ou no cubo do meio. Ela e'\n     \
                  SELECIONADA: as alcas de centro e de aro aparecem, e a secao da\n     \
                  roldana abre no Inspector. Antes desta wave a unica porta de entrada\n     \
                  era achar o nome na Hierarquia.\n\n  \
               (!) Clique no MIOLO do disco, longe do anel: NADA e' selecionado, e e' o\n      \
                   certo -- uma roldana grande emoldura a arte que passa por dentro\n      \
                   dela, e reclamar o disco faria a polia engolir o clique de tudo o\n      \
                   que ela emoldura.\n\n  \
               (!) A marca so' anda ate' a amarracao e ate' a roldana: ela nao vai para\n      \
                   tras do corpo nem para o outro lado da roda.\n",
        );
    }
}
