//! **A cena da HIGIENE DO PAR** (`PH2D_PHYSICS_SMOKE=50`, W-J8).
//!
//! Três coisas que um joint tem e que até agora não eram do artista: *ele está
//! em vigor?*, *os dois corpos que ele une ainda se batem?* e *qual das duas
//! pontas é a A?*. Mais uma quarta que se vê sem clicar em nada — o joint novo
//! nasce chamado **"Post : Plank"** em vez de "Joint (3)".
//!
//! Cada trio tem um par IDÊNTICO ao lado com o interruptor no outro lado, porque
//! um interruptor sozinho não se lê: *"a caixa está caindo"* só quer dizer
//! alguma coisa ao lado de uma que não está.
//!
//! ⚠️ **O Swap não move nada, e isso é o produto.** Ele é preservador de
//! comportamento por construção (medido: motor, servo, limites e curso todos
//! reproduzem o autorado ao 4º decimal), então o que ele muda é *qual ponta se
//! chama A* — as duas linhas do §12, o dono do ponto âmbar, e a linha sólida ×
//! tracejada do overlay. Numa CORDA as duas âncoras ficam separadas, e é por isso
//! que o rig do Swap é uma corda: o ponto âmbar SALTA de uma ponta para a outra.
//!
//! Os números abaixo saíram de uma sonda headless sobre estas mesmas armações,
//! rodada ANTES desta mensagem ser escrita.

use crate::physics_smoke::spawn_floor;
use ph2d_core::Vec2;
use ph2d_ecs::{Name, Transform, stable_name_id};
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, JointKind, PhysicsJoint, RigidBody};
use ph2d_render::{Sprite, WHITE_TILE_KEY};

impl crate::App {
    /// **Cena 50 (W-J8).** Dois braços (um desarmado), duas prateleiras (uma que
    /// deixa a caixa atravessar), e uma corda para trocar as pontas.
    pub(crate) fn physics_smoke_joint_pair(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        spawn_floor(gfx.sim.world_mut());
        let world = gfx.sim.world_mut();

        let grey = [0.75, 0.75, 0.8, 1.0];
        let hot = [0.95, 0.6, 0.2, 1.0];
        let cool = [0.4, 0.8, 0.95, 1.0];
        let peg = ColliderShape::Ball { radius: 0.08 };

        let hook = |world: &mut ph2d_ecs::World, name: &str, at: [f32; 2]| {
            world.spawn((
                Transform::from_translation(Vec2::new(at[0], at[1])),
                Sprite::atlas(WHITE_TILE_KEY, [0.16, 0.16], grey),
                Name::new(name.to_string()),
                RigidBody {
                    kind: BodyKind::Static,
                },
                Collider {
                    shape: peg,
                    ..Collider::default()
                },
            ));
        };
        for (n, x) in [("Arm Hook On", -7.5), ("Arm Hook Off", -4.5)] {
            hook(world, n, [x, 8.0]);
        }
        hook(world, "Rope Hook", [6.5, 8.0]);

        // --- Rig A: os dois braços. Mesmo peso, mesma corda, um desarmado. ---
        let plank = |world: &mut ph2d_ecs::World, name: &str, at: [f32; 2], rgba: [f32; 4]| {
            world.spawn((
                Transform::from_translation(Vec2::new(at[0], at[1])),
                Sprite::atlas(WHITE_TILE_KEY, [1.4, 0.3], rgba),
                Name::new(name.to_string()),
                RigidBody {
                    kind: BodyKind::Dynamic,
                },
                Collider {
                    shape: ColliderShape::Cuboid {
                        half_x: 0.7,
                        half_y: 0.15,
                    },
                    ..Collider::default()
                },
            ));
        };
        plank(world, "Arm On", [-6.8, 8.0], cool);
        plank(world, "Arm Off", [-3.8, 8.0], hot);

        // --- Rig B: as duas prateleiras. Mesma corda, mesma caixa. ---
        let shelf = |world: &mut ph2d_ecs::World, name: &str, at: [f32; 2]| {
            world.spawn((
                Transform::from_translation(Vec2::new(at[0], at[1])),
                Sprite::atlas(WHITE_TILE_KEY, [2.0, 0.5], grey),
                Name::new(name.to_string()),
                RigidBody {
                    kind: BodyKind::Static,
                },
                Collider {
                    shape: ColliderShape::Cuboid {
                        half_x: 1.0,
                        half_y: 0.25,
                    },
                    ..Collider::default()
                },
            ));
        };
        shelf(world, "Shelf Through", [-1.0, 5.0]);
        shelf(world, "Shelf Rest", [2.5, 5.0]);

        let crate_box = |world: &mut ph2d_ecs::World, name: &str, at: [f32; 2], rgba: [f32; 4]| {
            world.spawn((
                Transform::from_translation(Vec2::new(at[0], at[1])),
                Sprite::atlas(WHITE_TILE_KEY, [0.8, 0.8], rgba),
                Name::new(name.to_string()),
                RigidBody {
                    kind: BodyKind::Dynamic,
                },
                Collider {
                    shape: ColliderShape::Cuboid {
                        half_x: 0.4,
                        half_y: 0.4,
                    },
                    ..Collider::default()
                },
            ));
        };
        crate_box(world, "Crate Through", [-1.0, 7.5], hot);
        crate_box(world, "Crate Rest", [2.5, 7.5], cool);

        // --- Rig C: a corda cujas pontas se trocam. ---
        crate_box(world, "Rope Load", [6.5, 6.0], cool);

        // --- Os joints ---
        let joint = |world: &mut ph2d_ecs::World, name: &str, j: PhysicsJoint, at: [f32; 2]| {
            world.spawn((
                Name::new(name.to_string()),
                j,
                Transform::from_translation(Vec2::new(at[0], at[1])),
            ));
        };
        let pin = |a: &str, b: &str, active: bool| PhysicsJoint {
            body_a: stable_name_id(a),
            body_b: stable_name_id(b),
            kind: JointKind::Pin,
            active,
            ..PhysicsJoint::default()
        };
        joint(
            world,
            "Arm Hook On : Arm On",
            pin("Arm Hook On", "Arm On", true),
            [-7.5, 8.0],
        );
        joint(
            world,
            "Arm Hook Off : Arm Off",
            pin("Arm Hook Off", "Arm Off", false),
            [-4.5, 8.0],
        );

        let rope = |a: &str, b: &str, collide: bool| PhysicsJoint {
            body_a: stable_name_id(a),
            body_b: stable_name_id(b),
            kind: JointKind::Rope,
            max_length: 4.0,
            collide_connected: collide,
            ..PhysicsJoint::default()
        };
        joint(
            world,
            "Shelf Through : Crate Through",
            rope("Shelf Through", "Crate Through", false),
            [-1.0, 5.0],
        );
        joint(
            world,
            "Shelf Rest : Crate Rest",
            rope("Shelf Rest", "Crate Rest", true),
            [2.5, 5.0],
        );
        joint(
            world,
            "Rope Hook : Rope Load",
            PhysicsJoint {
                body_a: stable_name_id("Rope Hook"),
                body_b: stable_name_id("Rope Load"),
                kind: JointKind::Rope,
                max_length: 2.0,
                ..PhysicsJoint::default()
            },
            [6.5, 8.0],
        );

        eprintln!(
            "\n=== SMOKE 50: a HIGIENE DO PAR (W-J8) ===\n\
             Tres interruptores que um joint sempre teve e que nunca foram do artista.\n\
             Aperte `B` para ver os joints. O relogio JA esta tocando.\n\n\
             1) ACTIVE -- os dois bracos a ESQUERDA sao identicos: mesmo peso, mesma\n\
                dobradica. O ciano (`Arm On`) esta pendurado; o laranja (`Arm Off`) caiu\n\
                no chao, e o joint dele esta desenhado APAGADO no gancho -- presente, com\n\
                todos os parametros, so nao em vigor. Medido: y = 7.46 contra 0.15.\n\
                Selecione `Arm Hook Off : Arm Off` na Hierarquia e ponha Active = On:\n\
                a dobradica acende e volta a segurar. Ponha o outro em Off e veja o\n\
                inverso. ⚠️ Um joint desarmado NAO fica vermelho e NAO ganha estouro --\n\
                isso e ruptura, e ele nao rompeu.\n\n\
             2) COLLIDE -- as duas prateleiras no MEIO tem a mesma corda de 4 m e a\n\
                mesma caixa. A laranja (`Crate Through`) ATRAVESSA a prateleira a que\n\
                esta amarrada e fica pendurada embaixo; a ciano (`Crate Rest`) POUSA em\n\
                cima dela. Medido: y = 1.00 contra 5.65. A unica diferenca e o Collide\n\
                no §12. ⚠️ O default e OFF porque o caso comum e um elo de corrente, que\n\
                se sobrepoe ao vizinho por construcao -- ligado ali, um motor preso\n\
                dentro da propria carga cai de 4 rad/s para ZERO (medido).\n\n\
             3) SWAP -- a corda a DIREITA. ⚠️ **PAUSE primeiro** (barra de espaco): o\n\
                ponto ambar agarravel so e desenhado com o relogio parado (tocando, o\n\
                overlay desenha as ancoras VIVAS do solver). Selecione\n\
                `Rope Hook : Rope Load` e olhe o §12: Body A = Rope Hook, Body B = Rope\n\
                Load, e o ponto esta no GANCHO (y = 8.0). Aperte `Swap A / B`: as duas\n\
                linhas trocam e o ponto SALTA para a carga (y = 6.0) -- e a carga nao se\n\
                mexe um milimetro (6.0000 antes, 6.0000 depois). E isso que o botao\n\
                promete: ele troca qual ponta se chama A, nao o que o joint faz.\n\
                (Num Pin as duas ancoras coincidem, entao o ponto nao teria para onde\n\
                saltar -- por isso o rig do Swap e uma corda.)\n\n\
             4) O NOME -- olhe a Hierarquia sem clicar em nada: os cinco joints se\n\
                chamam `Arm Hook On : Arm On`, `Shelf Rest : Crate Rest`, e assim por\n\
                diante. Um joint criado pelo botao Join nasce assim.\n"
        );
    }
}
