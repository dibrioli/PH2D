//! **A cena dos CINCO MODOS DE JOINT** (`PH2D_PHYSICS_SMOKE=55`, W-FK +
//! W-JointTools).
//!
//! A cena 54 demonstra UM gesto (a IK). Esta demonstra a **escolha**: a seção
//! Joints do painel tem cinco modos, três deles sobre *quanto da cadeia um
//! arrasto carrega* e dois que são gestos de pose inteiramente diferentes. Só
//! comparando lado a lado é que a lista faz sentido.
//!
//! Duas estações, cada uma para o que a outra não mostra:
//!
//! - **O BRAÇO** (esquerda): ombro ESTÁTICO + três elos. É onde `Rig` e `Links`
//!   diferem de forma visível — o `Rig` leva o ombro junto (você está mudando a
//!   parede de lugar), o `Links` o deixa pregado.
//! - **A PERNA** (direita): quadril estático + coxa + canela, com o **joelho
//!   limitado**. É a estação da FK: girar a coxa leva a canela junto, e o joelho
//!   nunca passa da faixa autorada.
//!
//! Os números da mensagem saem da sonda `probe_smoke_55`, rodada sobre ESTAS
//! peças antes de a mensagem ser escrita — a regra que esta linha adotou depois
//! de duas cenas afirmarem coisas que a medição desmentiu.

use ph2d_core::Vec2;
use ph2d_ecs::{Name, Transform, World, stable_name_id};
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, JointKind, PhysicsJoint, RigidBody};
use ph2d_render::{Sprite, WHITE_TILE_KEY};

const BONE: [f32; 4] = [0.85, 0.82, 0.72, 1.0];
const LIMB: [f32; 4] = [0.55, 0.9, 0.35, 1.0];
const KNEE: [f32; 4] = [0.95, 0.6, 0.2, 1.0];

fn link(world: &mut World, name: &str, x: f32, y: f32, len: f32, kind: BodyKind, tint: [f32; 4]) {
    world.spawn((
        Transform::from_translation(Vec2::new(x, y)),
        Sprite::atlas(WHITE_TILE_KEY, [len, 0.2], tint),
        Name::new(name),
        RigidBody { kind },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: len * 0.5,
                half_y: 0.1,
            },
            ..Collider::default()
        },
    ));
}

fn pin(world: &mut World, name: &str, a: &str, b: &str, at: [f32; 2], limits: Option<[f32; 2]>) {
    let mut j = PhysicsJoint {
        body_a: stable_name_id(a),
        body_b: stable_name_id(b),
        kind: JointKind::Pin,
        ..PhysicsJoint::of_kind(JointKind::Pin)
    };
    if let Some([min, max]) = limits {
        j.limits_enabled = true;
        j.limit_min = min;
        j.limit_max = max;
    }
    world.spawn((
        Transform::from_translation(Vec2::new(at[0], at[1])),
        Name::new(name),
        j,
    ));
}

/// As duas estações. A MESMA construção que a sonda headless mede.
pub(crate) fn spawn_props(world: &mut World) {
    // ── O BRAÇO: ombro ESTÁTICO + três elos de 1 m.
    link(world, "Shoulder", -6.0, 2.0, 0.4, BodyKind::Static, BONE);
    for (i, name) in ["UpperArm", "Forearm", "Hand"].iter().enumerate() {
        link(
            world,
            name,
            -5.5 + i as f32,
            2.0,
            1.0,
            BodyKind::Dynamic,
            LIMB,
        );
    }
    pin(
        world,
        "J.Shoulder",
        "Shoulder",
        "UpperArm",
        [-6.0, 2.0],
        None,
    );
    pin(world, "J.Elbow", "UpperArm", "Forearm", [-5.0, 2.0], None);
    pin(world, "J.Wrist", "Forearm", "Hand", [-4.0, 2.0], None);

    // ── A PERNA: quadril estático + coxa + canela, joelho limitado a [0, 2] rad.
    link(world, "Hip", 2.0, 2.0, 0.4, BodyKind::Static, BONE);
    link(world, "Thigh", 2.5, 2.0, 1.0, BodyKind::Dynamic, LIMB);
    link(world, "Shin", 3.5, 2.0, 1.0, BodyKind::Dynamic, KNEE);
    pin(world, "J.Hip", "Hip", "Thigh", [2.0, 2.0], None);
    pin(
        world,
        "J.Knee",
        "Thigh",
        "Shin",
        [3.0, 2.0],
        Some([0.0, 2.0]),
    );
}

impl crate::App {
    /// **Cena 55 (W-FK + W-JointTools).** Dois rigs, PAUSADA, painel aberto.
    pub(crate) fn physics_smoke_fk(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        spawn_props(gfx.sim.world_mut());
        if let Some(hero) = gfx.hero_screen.as_mut() {
            hero.panel_visibility.insert("physics", true);
        }
        // O modo NÃO é armado em código: o passo 1 é escolher no painel, e um
        // smoke que arma o estado por baixo pula exatamente a costura que ele
        // deveria provar.

        eprintln!(
            "[physics-smoke 55] A cena esta PAUSADA e o painel PHYSICS esta aberto (tecla W).\n  \
               A secao JOINTS tem CINCO modos de arrastar uma cadeia. Compare-os na\n  \
               mesma cena -- e o unico jeito de a lista fazer sentido.\n\n  \
               1. Abra a secao JOINTS. O modo inicial e 'Body', que e o que o editor\n     \
                  sempre fez. Aperte B para ver colliders e joints.\n  \
               2. BODY: arraste a MAO (o elo da ponta do braco). So ela anda; o joint\n     \
                  fica esticado, e voce VE isso pelo segmento ambar.\n  \
               3. RIG: arraste a mesma mao. A cadeia INTEIRA acompanha -- inclusive o\n     \
                  OMBRO claro, que e estatico. E o certo: voce esta mudando a parede\n     \
                  de lugar junto com o braco.\n     \
                  (medido: o conjunto carregado e UpperArm+Forearm+Hand+SHOULDER)\n  \
               4. LINKS: arraste a mao de novo. Os elos moveis acompanham e o OMBRO\n     \
                  FICA. E o modo de posar um braco sem arrancar o ombro da parede.\n     \
                  (medido: o conjunto carregado e UpperArm+Forearm+Hand, sem o ombro)\n  \
               5. Em QUALQUER modo, segure ALT e arraste: e sempre o rig inteiro. O\n     \
                  atalho nao muda de significado com o modo -- inclusive em IK/FK, onde\n     \
                  ele SUPRIME a pose (Alt quer dizer 'leve tudo', que e um arrasto).\n  \
               6. IK: arraste a mao. O cotovelo e o ombro dobram ATRAS dela -- e a\n     \
                  cinematica inversa (cena 54 tem a demonstracao completa).\n  \
               7. FK: arraste a COXA da perna (o elo verde da direita). Ela gira em\n     \
                  torno do QUADRIL e a canela laranja vai junto, rigidamente. Depois\n     \
                  arraste a CANELA: ela gira em torno do JOELHO e a coxa nao se mexe.\n     \
                  E isso que separa FK de IK: aqui voce autora UM angulo.\n     \
                  (medido: girando a coxa 90 graus a canela viaja 2,12 m e a coxa\n      \
                   0,71 m, com a distancia entre elas em 1,000 antes e depois -- a\n      \
                   peca e rigida; girando a CANELA o conjunto movido e so ela)\n  \
               8. FK + LIMITE: arraste a canela em volta do joelho, para todos os\n     \
                  lados. A junta esta limitada a [0, 2] rad e NUNCA passa disso --\n     \
                  uma pose que o Play desfaz no primeiro tick nao e uma pose.\n     \
                  E ao voltar para dentro da faixa o elo volta AO CURSOR na hora.\n  \
               9. Ctrl+Z: UM passo desfaz o arrasto INTEIRO, com todos os elos juntos.\n  \
              10. Marque 'Physics' no transporte e de Play: os rigs caem a partir da\n     \
                  pose que voce autorou. Posar nao simula -- ele prepara o que a\n     \
                  simulacao usa."
        );
    }
}

#[cfg(test)]
#[path = "physics_smoke_fk_tests.rs"]
mod tests;
