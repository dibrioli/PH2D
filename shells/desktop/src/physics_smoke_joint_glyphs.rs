//! **A cena do VOCABULÁRIO visual dos joints** (`PH2D_PHYSICS_SMOKE=43`, W-J1).
//!
//! Irmã de [`crate::physics_smoke_authoring`] (que demonstra um GESTO de
//! autoria) e de [`crate::physics_smoke_rigs`] (que demonstra o SOLVER): esta
//! demonstra o **desenho** — o que um joint diz de si mesmo no canvas. Até esta
//! wave os quatro tipos desenhavam a mesma figura, e tudo que o artista autorou
//! (tipo, alcance, comprimento, folga) era número cego no §12.
//!
//! Os números abaixo saíram de uma sonda headless sobre esta mesma armação,
//! rodada ANTES desta mensagem ser escrita.

use crate::physics_smoke::spawn_floor;
use ph2d_core::Vec2;
use ph2d_ecs::{Name, Transform, stable_name_id};
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, JointKind, PhysicsJoint, RigidBody};
use ph2d_render::{Sprite, WHITE_TILE_KEY};

impl crate::App {
    /// **Cena 43 (W-J1).** Os quatro tipos lado a lado, PAUSADA — cada um com o
    /// que o artista autorou visível como geometria.
    ///
    /// Pausada de propósito: as figuras se leem paradas, e três dos quatro
    /// fatos (a mola ESTICADA, a corda FROUXA, o weld TORTO) são estados
    /// autorados que a simulação corrige em menos de um segundo. O passo 2 da
    /// mensagem manda tocar justamente para ver a correção acontecer.
    pub(crate) fn physics_smoke_joint_glyphs(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        spawn_floor(gfx.sim.world_mut());
        let world = gfx.sim.world_mut();

        let mut anchor = |name: &str, x: f32, y: f32| {
            world.spawn((
                Transform::from_translation(Vec2::new(x, y)),
                Sprite::atlas(WHITE_TILE_KEY, [0.2, 0.2], [0.75, 0.75, 0.8, 1.0]),
                Name::new(name.to_string()),
                RigidBody {
                    kind: BodyKind::Static,
                },
                Collider {
                    shape: ColliderShape::Ball { radius: 0.1 },
                    ..Collider::default()
                },
            ));
        };
        anchor("PostA", -5.5, 4.5);
        anchor("HookB", -1.8, 5.4);
        anchor("HookC", 1.8, 5.4);
        anchor("PostD", 5.2, 4.5);

        // 1. PIN — a dobradiça limitada e motorizada. O arco desenha as duas
        // paredes e a AGULHA aponta o ângulo vivo: com o motor ligado ela
        // caminha até a parede e para lá (medido: 0,0deg -> 40,1deg, e fica).
        world.spawn((
            Transform::from_translation(Vec2::new(-4.7, 4.5)),
            Sprite::atlas(WHITE_TILE_KEY, [1.6, 0.2], [0.95, 0.6, 0.2, 1.0]),
            Name::new("Wheel".to_string()),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: 0.8,
                    half_y: 0.1,
                },
                ..Collider::default()
            },
        ));
        world.spawn((
            Transform::from_translation(Vec2::new(-5.5, 4.5)),
            Name::new("HingeJoint".to_string()),
            PhysicsJoint {
                body_a: stable_name_id("PostA"),
                body_b: stable_name_id("Wheel"),
                kind: JointKind::Pin,
                limits_enabled: true,
                limit_min: -0.7,
                limit_max: 0.7,
                motor_enabled: true,
                motor_speed: 1.2,
                motor_max_force: 30.0,
                ..PhysicsJoint::default()
            },
        ));

        // 2. SPRING — a bola nasce a 1,60 m de um repouso de 1,20: o anel do
        // repouso passa DENTRO da bola, e o zigue-zague chega esticado. Tocando,
        // ela sobe e assenta em 1,240 m (o peso próprio estica 4 cm).
        world.spawn((
            Transform::from_translation(Vec2::new(-1.8, 3.8)),
            Sprite::atlas(WHITE_TILE_KEY, [0.56, 0.56], [0.4, 0.85, 0.55, 1.0]),
            Name::new("SpringBob".to_string()),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Ball { radius: 0.28 },
                ..Collider::default()
            },
        ));
        world.spawn((
            Transform::from_translation(Vec2::new(-1.8, 5.4)),
            Name::new("SpringJoint".to_string()),
            PhysicsJoint {
                body_a: stable_name_id("HookB"),
                body_b: stable_name_id("SpringBob"),
                kind: JointKind::Spring,
                rest_length: 1.2,
                stiffness: 60.0,
                damping: 2.0,
                ..PhysicsJoint::default()
            },
        ));

        // 3. ROPE — a bola descansa numa borda a 0,82 m do gancho, com 1,60 m de
        // corda: quase o dobro de fio que de vão, então ela PENDURA (a barriga
        // cai para onde a gravidade aponta) e o anel do máximo fica bem fora da
        // bola. A corda nunca fica tesa nesta cena — é o ponto.
        world.spawn((
            Transform::from_translation(Vec2::new(1.8, 4.0)),
            Sprite::atlas(WHITE_TILE_KEY, [1.0, 0.2], [0.55, 0.55, 0.6, 1.0]),
            Name::new("Ledge".to_string()),
            RigidBody {
                kind: BodyKind::Static,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: 0.5,
                    half_y: 0.1,
                },
                ..Collider::default()
            },
        ));
        world.spawn((
            Transform::from_translation(Vec2::new(1.8, 4.4)),
            Sprite::atlas(WHITE_TILE_KEY, [0.56, 0.56], [0.4, 0.7, 0.95, 1.0]),
            Name::new("RopeBob".to_string()),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Ball { radius: 0.28 },
                ..Collider::default()
            },
        ));
        world.spawn((
            Transform::from_translation(Vec2::new(1.8, 5.4)),
            Name::new("RopeJoint".to_string()),
            PhysicsJoint {
                body_a: stable_name_id("HookC"),
                body_b: stable_name_id("RopeBob"),
                kind: JointKind::Rope,
                max_length: 1.6,
                ..PhysicsJoint::default()
            },
        ));

        // 4. WELD — o quadrado veste a rotação do corpo A, então o POSTE é que
        // nasce girado (0,45 rad = 26deg). A barra nasce fora desse ângulo e o
        // weld a alinha no 1º passo: medido, o ângulo RELATIVO vai de -0,400 rad
        // a 0,000 e fica — que é o que "travado" significa.
        let post = {
            let mut q = world.query::<(ph2d_ecs::Entity, &Name)>();
            q.iter(world)
                .find(|(_, n)| n.as_str() == "PostD")
                .map(|(e, _)| e)
                .expect("PostD")
        };
        world.get_mut::<Transform>(post).expect("t").rotation = 0.45;
        world.spawn((
            Transform::from_translation(Vec2::new(5.9, 4.2)),
            Sprite::atlas(WHITE_TILE_KEY, [1.6, 0.18], [0.85, 0.4, 0.75, 1.0]),
            Name::new("WeldBar".to_string()),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: 0.8,
                    half_y: 0.09,
                },
                ..Collider::default()
            },
        ));
        world.spawn((
            Transform::from_translation(Vec2::new(5.2, 4.5)),
            Name::new("WeldJoint".to_string()),
            PhysicsJoint {
                body_a: stable_name_id("PostD"),
                body_b: stable_name_id("WeldBar"),
                kind: JointKind::Weld,
                ..PhysicsJoint::default()
            },
        ));

        eprintln!(
            "[physics-smoke 43] O VOCABULARIO dos joints, quatro tipos lado a lado.\n\
             PAUSADA. Aperte B se o contorno nao estiver ligado.\n  \
               1. OLHE: os quatro desenham figuras DIFERENTES, e cada uma diz o que\n     \
                  o artista autorou --\n     \
                  PIN (esq.):    anel + ARCO com duas paredes e a agulha no angulo vivo\n     \
                  SPRING:        zigue-zague + ANEL do repouso (1,20 m) -- a bola nasce\n                    \
                                 a 1,60, entao o anel passa DENTRO dela: esticada\n     \
                  ROPE:          fio PENDURADO + anel do maximo (1,60 m) bem fora da\n                    \
                                 bola, que descansa a 0,82: quase o dobro de fio\n     \
                  WELD (dir.):   QUADRADO, girado com o poste (26 graus)\n  \
               2. De Play. O que muda, e por que:\n     \
                  - a agulha do PIN caminha ate a parede e PARA la (0,0 -> 40,1 graus)\n     \
                  - o motor desenha o mesmo glifo de giro da zona de torque, em ambar\n     \
                  - a mola encolhe ate assentar em 1,240 m: o anel do repouso quase\n       \
                    encosta na bola (o peso proprio estica 4 cm)\n     \
                  - a corda NAO fica tesa: a bola esta apoiada, entao a barriga fica\n  \
               3. Cada ponta diz de quem e: linha SOLIDA ate o corpo A, TRACEJADA ate\n     \
                  o B (a paleta ja usa ciano para collider dinamico, entao a diferenca\n     \
                  e de forma, nao de cor).\n\
             O VERMELHO (restricao nao imposta) nao aparece aqui, e isso foi MEDIDO:\n\
             joint do rapier e rigido -- um pino com 500x a massa e outro levando um\n\
             martelo de 400x abriram 0,00000 m. Quem abre o vao e a arquitetura: um\n\
             joint nao move corpo KINEMATIC, entao dois corpos curva-dirigidos que a\n\
             animacao afasta ficam soltos com o pino desenhado por cima (medido: 1,50 m\n\
             = 150 px). E o estado em que um rig ASSADO fica."
        );
    }
}
