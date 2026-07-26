//! **A cena do TRILHO** (`PH2D_PHYSICS_SMOKE=47`, W-J5).
//!
//! O 5º tipo de joint é o espelho do Pin: um Pin deixa girar e proíbe transladar,
//! um **Slider** deixa transladar por UMA direção e proíbe todo o resto. O
//! elevador, a porta de correr, o pistão.
//!
//! A pergunta de projeto que ele traz é *onde mora o eixo?*, e a resposta é a que
//! Godot e Unreal dão e a que este componente já implicava: **na rotação da
//! entidade-joint**. O `Transform` de um joint é onde a *colocação* dele vive (a
//! translação é a âncora), então a direção de uma colocação vive na rotação — e o
//! eixo fica autorável no dia um, pelo campo **Rotation** do §0, com zero widget
//! novo.
//!
//! Os números abaixo saíram de uma sonda headless sobre esta mesma armação,
//! rodada ANTES desta mensagem ser escrita.

use crate::physics_smoke::spawn_floor;
use ph2d_core::Vec2;
use ph2d_ecs::{Name, Transform, stable_name_id};
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, JointKind, PhysicsJoint, RigidBody};
use ph2d_render::{Sprite, WHITE_TILE_KEY};

impl crate::App {
    /// **Cena 47 (W-J5).** Três trilhos + um par para autorar. PAUSADA.
    pub(crate) fn physics_smoke_joint_slider(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        spawn_floor(gfx.sim.world_mut());
        let world = gfx.sim.world_mut();

        {
            let mut body = |name: &str,
                            kind: BodyKind,
                            shape: ColliderShape,
                            size: [f32; 2],
                            rgba: [f32; 4],
                            at: [f32; 2]| {
                world.spawn((
                    Transform::from_translation(Vec2::new(at[0], at[1])),
                    Sprite::atlas(WHITE_TILE_KEY, size, rgba),
                    Name::new(name.to_string()),
                    RigidBody { kind },
                    Collider {
                        shape,
                        ..Collider::default()
                    },
                ));
            };
            let grey = [0.75, 0.75, 0.8, 1.0];
            let hot = [0.95, 0.6, 0.2, 1.0];
            let cool = [0.4, 0.8, 0.95, 1.0];

            // ── O ELEVADOR: trilho VERTICAL, curso 0,6 m.
            body(
                "Shaft",
                BodyKind::Static,
                ColliderShape::Cuboid {
                    half_x: 0.08,
                    half_y: 1.2,
                },
                [0.16, 2.4],
                grey,
                [-4.0, 6.0],
            );
            body(
                "Cabin",
                BodyKind::Dynamic,
                ColliderShape::Cuboid {
                    half_x: 0.45,
                    half_y: 0.45,
                },
                [0.9, 0.9],
                hot,
                [-4.0, 6.0],
            );
            // ── A CALHA: trilho a 45 graus, curso 1,0 m.
            body(
                "Post",
                BodyKind::Static,
                ColliderShape::Ball { radius: 0.12 },
                [0.24, 0.24],
                grey,
                [0.0, 7.0],
            );
            body(
                "Slug",
                BodyKind::Dynamic,
                ColliderShape::Ball { radius: 0.3 },
                [0.6, 0.6],
                cool,
                [0.0, 7.0],
            );
            // ── A VIGA: trilho HORIZONTAL, o CONTROLE — a gravidade e perpendicular.
            body(
                "Beam",
                BodyKind::Static,
                ColliderShape::Cuboid {
                    half_x: 1.5,
                    half_y: 0.08,
                },
                [3.0, 0.16],
                grey,
                [4.0, 6.0],
            );
            body(
                "Trolley",
                BodyKind::Dynamic,
                ColliderShape::Cuboid {
                    half_x: 0.3,
                    half_y: 0.3,
                },
                [0.6, 0.6],
                hot,
                [4.0, 6.0],
            );

            // ── E um par PELADO, para autorar do zero (spawnado ANTES dos joints para
            // que a closure `body` morra antes de a `rail` pegar o mundo — dois
            // closures não podem ter o `&mut World` ao mesmo tempo).
            body(
                "Frame",
                BodyKind::Static,
                ColliderShape::Cuboid {
                    half_x: 0.1,
                    half_y: 0.1,
                },
                [0.2, 0.2],
                grey,
                [8.0, 7.0],
            );
            body(
                "Door",
                BodyKind::Dynamic,
                ColliderShape::Cuboid {
                    half_x: 0.5,
                    half_y: 0.7,
                },
                [1.0, 1.4],
                cool,
                [8.0, 7.0],
            );
        }

        let mut rail = |name: &str, a: &str, b: &str, at: [f32; 2], rot: f32, stroke: f32| {
            let mut t = Transform::from_translation(Vec2::new(at[0], at[1]));
            // ⚠️ **O eixo É esta rotação** — nenhum campo novo o guarda.
            t.rotation = rot;
            world.spawn((
                Name::new(name.to_string()),
                PhysicsJoint {
                    body_a: stable_name_id(a),
                    body_b: stable_name_id(b),
                    kind: JointKind::Slider,
                    limits_enabled: true,
                    limit_min: -stroke,
                    limit_max: stroke,
                    ..PhysicsJoint::default()
                },
                t,
            ));
        };
        rail(
            "Shaft Rail",
            "Shaft",
            "Cabin",
            [-4.0, 6.0],
            -std::f32::consts::FRAC_PI_2,
            0.6,
        );
        rail(
            "Chute",
            "Post",
            "Slug",
            [0.0, 7.0],
            -std::f32::consts::FRAC_PI_4,
            1.0,
        );
        rail("Beam Rail", "Beam", "Trolley", [4.0, 6.0], 0.0, 1.2);

        eprintln!(
            "[physics-smoke 47] O TRILHO -- o 5o tipo de joint (W-J5).\n\
             PAUSADA. Tres trilhos prontos + um par para autorar.\n  \
               1. Aperte B para ver os contornos. Cada Slider desenha um TRILHO:\n     \
                  uma reta pelo eixo, com tracinhos perpendiculares nos fins de\n     \
                  curso. Sem curso nao ha tracinhos -- eles dizem onde o movimento\n     \
                  PARA, e um trilho ilimitado nao para em lugar nenhum.\n  \
               2. Play. Medido:\n       \
                  - CABINE (laranja, trilho vertical): cai EXATAMENTE 0,60 m e\n         \
                    para, em (-4,000, 5,400). O curso e em METROS.\n       \
                  - PILULA (ciano, trilho a 45 graus): corre a diagonal inteira,\n         \
                    (0,707, 6,293) -- dx = dy = 0,707, que e 1,0 m ao longo do\n         \
                    eixo. O eixo NAO e um eixo do mundo: e a rotacao do joint.\n       \
                  - CARRINHO (laranja, trilho horizontal): fica em (4,000, 6,000).\n         \
                    E o CONTROLE: a gravidade e perpendicular ao trilho, entao ele\n         \
                    nao tem por onde cair. Se ele cair, o eixo esta sendo ignorado.\n  \
               3. Rebobine. Selecione 'Chute' na Hierarquia: a secao Physics Joint\n     \
                  mostra Kind = Slider e as rows Min/Max dizendo **(m)**, nao (graus)\n     \
                  -- o mesmo par de campos carrega a unidade do TIPO. Mude o Max para\n     \
                  0.3 e de Play: a pilula para na terca parte do caminho.\n  \
               4. O EIXO e autorado no §0: com 'Chute' selecionada, mude **Rotation**\n     \
                  para 0 e de Play -- a calha vira horizontal e a pilula para de\n     \
                  descer. Nenhum widget novo foi preciso para isso.\n  \
               5. Trocar o Kind RE-SEMEIA o alcance, e e de proposito: ponha 'Chute'\n     \
                  em Pin e as rows voltam a dizer (graus) com +-45; volte para Slider\n     \
                  e elas dizem (m) com +-0,5. Sem isso os +-45 graus (0,785 rad)\n     \
                  viravam +-0,785 METROS de curso -- um numero que ninguem digitou.\n  \
               6. **DESENHE UM TRILHO** (a estrela): selecione 'Door' (a porta ciano\n     \
                  da direita), ponha 'Join As' em **Slider**, aperte 'Draw Joint on\n     \
                  Canvas' e arraste do 'Frame' cinza ate a porta **na direcao que\n     \
                  voce quer o trilho** (tente a diagonal). O rumo do arrasto E o\n     \
                  eixo -- o gesto desenha o trilho, em vez de te mandar digitar o\n     \
                  angulo depois.\n\
             O Slider nao tem MOTOR nesta wave, de proposito: um motor LINEAR (o\n\
             guincho) chega no W-J6 junto com os modos Position|Velocity, e oferece-lo\n\
             aqui seria pintar dois knobs que o solver ignora."
        );
    }
}
