//! **A cena de POSAR em vez de digitar** (`PH2D_PHYSICS_SMOKE=45`, W-J3).
//!
//! A cena 43 mostrou o que um joint DIZ de si; a 44, o que se pode FAZER com as
//! âncoras dele. Esta fecha o par: os NÚMEROS do §12 que descrevem geometria —
//! até onde a dobradiça vai, que comprimento a mola/corda nomeia — passam a ter
//! lugar no canvas, e o lugar se arrasta.
//!
//! Duas pistas, porque as duas grandezas se posam de formas diferentes:
//!
//! - **esquerda (Pin com limites):** as duas paredes do arco. Arrastar uma escreve
//!   SÓ ela (o alcance é assimétrico por construção — o que o cone do Unreal não
//!   expressa) e, enquanto se arrasta, o **FANTASMA** do corpo B mostra onde ele
//!   vai parar. É o *'L'* do RUBE sem modo: arrastar JÁ posa.
//! - **direita (Spring):** o anel de comprimento. Ele é construído em MUNDO, então
//!   arrastá-lo é dizer *"a mola descansa a esta distância"* na régua da cena.
//!
//! ⚠️ **O motor NÃO tem alça, e a ausência é a decisão** — ver
//! `render_loop::point_gizmo::joint_param_handles`: um limite é um ÂNGULO e um
//! comprimento é uma DISTÂNCIA, e cada um já tem lugar; velocidade é uma TAXA, e
//! nenhum lugar da tela é 120 °/s.
//!
//! Os números abaixo saíram de uma sonda headless sobre esta mesma armação,
//! rodada ANTES desta mensagem ser escrita.

use crate::physics_smoke::spawn_floor;
use ph2d_core::Vec2;
use ph2d_ecs::{Name, Transform, stable_name_id};
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, JointKind, PhysicsJoint, RigidBody};
use ph2d_render::{Sprite, WHITE_TILE_KEY};

impl crate::App {
    /// **Cena 45 (W-J3).** Posar limite e comprimento no canvas, PAUSADA.
    pub(crate) fn physics_smoke_joint_pose(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        spawn_floor(gfx.sim.world_mut());
        let world = gfx.sim.world_mut();

        let mut post = |name: &str, x: f32, y: f32| {
            world.spawn((
                Transform::from_translation(Vec2::new(x, y)),
                Sprite::atlas(WHITE_TILE_KEY, [0.3, 0.3], [0.75, 0.75, 0.8, 1.0]),
                Name::new(name.to_string()),
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
            ));
        };
        post("HingePost", -4.0, 6.0);
        post("SpringPost", 0.0, 6.0);

        // ── ESQUERDA: a dobradiça com alcance ────────────────────────────────
        // A barra nasce HORIZONTAL, dentro do alcance: assim a agulha viva do
        // arco (onde o corpo está) e as paredes (até onde ele pode ir) são três
        // marcas distintas desde o primeiro frame.
        world.spawn((
            Transform::from_translation(Vec2::new(-3.2, 6.0)),
            Sprite::atlas(WHITE_TILE_KEY, [1.6, 0.24], [0.95, 0.6, 0.2, 1.0]),
            Name::new("HingeArm".to_string()),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: 0.8,
                    half_y: 0.12,
                },
                ..Collider::default()
            },
        ));
        world.spawn((
            Transform::from_translation(Vec2::new(-4.0, 6.0)),
            Name::new("Hinge".to_string()),
            PhysicsJoint {
                body_a: stable_name_id("HingePost"),
                body_b: stable_name_id("HingeArm"),
                kind: JointKind::Pin,
                limits_enabled: true,
                limit_min: (-45.0_f32).to_radians(),
                limit_max: 45.0_f32.to_radians(),
                ..PhysicsJoint::default()
            },
        ));

        // ── DIREITA: a mola e o anel de repouso ──────────────────────────────
        world.spawn((
            Transform::from_translation(Vec2::new(0.0, 5.0)),
            Sprite::atlas(WHITE_TILE_KEY, [0.5, 0.5], [0.4, 0.8, 0.95, 1.0]),
            Name::new("SpringBob".to_string()),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Ball { radius: 0.25 },
                ..Collider::default()
            },
        ));
        world.spawn((
            Transform::from_translation(Vec2::new(0.0, 6.0)),
            Name::new("Spring".to_string()),
            PhysicsJoint {
                body_a: stable_name_id("SpringPost"),
                body_b: stable_name_id("SpringBob"),
                kind: JointKind::Spring,
                rest_length: 1.0,
                stiffness: 30.0,
                damping: 0.5,
                ..PhysicsJoint::default()
            },
        ));

        eprintln!(
            "[physics-smoke 45] POSE, NAO DIGITE: o limite e o comprimento no canvas (W-J3).\n\
             PAUSADA, nada selecionado, contorno LIGADO (aperte B se nao estiver).\n  \
               1. Olhe a dobradica da esquerda: o ARCO tem duas PAREDES (as marcas\n     \
                  radiais, em -45 e +45 graus) e uma AGULHA viva do centro ate onde a\n     \
                  barra esta agora. Sobre cada parede ha um GRIP ambar pequeno --\n     \
                  menor que os discos de ancora, porque ele agarra uma linha que ja\n     \
                  estava desenhada em vez de ser uma marca nova.\n  \
               2. Arraste a parede de BAIXO. Enquanto arrasta, aparece o FANTASMA da\n     \
                  barra: a silhueta dela na pose que aquela parede permite. Solte em\n     \
                  ~-20 graus e de Play -- medido: com a parede em -45 a barra assenta\n     \
                  em rot -45,0 graus, posicao (-3,434, 5,434); com a parede em -20 ela\n     \
                  assenta em rot -20,0 graus, (-3,248, 5,726). Ou seja: a barra para\n     \
                  EXATAMENTE onde voce posou a parede.\n  \
               3. Rebobine e arraste a parede de baixo para CIMA, alem da de cima. Ela\n     \
                  PARA na irma -- nao troca de lugar com ela. (`clamped()` TROCA um par\n     \
                  invertido, o que e certo para um numero digitado e errado para um\n     \
                  gesto: a troca poria a OUTRA parede na sua mao no meio do arrasto.)\n  \
               4. Abra o Inspector com a joint selecionada: os campos Min/Max seguem o\n     \
                  arrasto em GRAUS, e digitar neles move a parede. Sao a mesma edicao\n     \
                  por dois caminhos -- um funil so (`joint_with_edit`).\n  \
               5. A mola da direita: arraste o GRIP do ANEL de comprimento (ele fica\n     \
                  sobre o anel, na direcao do corpo B). Puxe de 1,0 para ~2,0 m e de\n     \
                  Play -- medido: com repouso 1,00 o peso pendura a 1,065 m do poste;\n     \
                  com 2,00, a 2,063 m (a sobra e o proprio peso esticando a mola).\n     \
                  O anel e construido em MUNDO: dar zoom faz ele crescer junto com a\n     \
                  cena, porque um comprimento e um comprimento.\n  \
               6. Ctrl+Z desfaz um arrasto inteiro num passo so.\n\
             O QUE NAO TEM ALCA, E POR QUE: o MOTOR. Um limite e um angulo e um\n\
             comprimento e uma distancia -- cada um ja tem lugar na tela, e arrastar\n\
             ate ele nao converte nada. Velocidade e uma TAXA: nenhum lugar da tela\n\
             e 120 graus/s, entao qualquer alca precisaria de uma constante px-por-\n\
             grau/s, e a row do §12 que ela espelharia nao tem faixa de onde tirar\n\
             uma. As duas alternativas sem constante falham sozinhas (o arco SATURA\n\
             em 270 graus; uma posicao angular DA A VOLTA e 400 graus/s desenha o\n\
             mesmo ponto que 40). Fica para uma decisao sua sobre a lei de controle."
        );
    }
}
