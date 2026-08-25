//! **A CORDA E A PEÇA** (`PH2D_PHYSICS_SMOKE=74`, W-LeadDrag).
//!
//! Quatro rigs que respondem à mesma pergunta — *o que acontece quando eu
//! arrasto o corpo da ponta A?* — e dão quatro respostas diferentes, cada uma
//! porque a CENA é diferente, nunca porque a ferramenta tem um caso especial.
//!
//! ⚠️ Os dois primeiros eram gestos **MORTOS** antes desta wave: pegar a cabeça
//! de uma cadeia solta era recusado pelo plano de IK (`root == tip`), e uma peça
//! soldada não abria sessão de FK nenhuma.

use ph2d_core::Vec2;
use ph2d_ecs::{Name, Transform, World, stable_name_id};
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, JointKind, PhysicsJoint, RigidBody};
use ph2d_render::{Sprite, WHITE_TILE_KEY};

const ROPE_RGBA: [f32; 4] = [0.45, 0.80, 0.95, 1.0];
const PIECE_RGBA: [f32; 4] = [0.98, 0.75, 0.25, 1.0];
const ARM_RGBA: [f32; 4] = [0.55, 0.85, 0.55, 1.0];
const WALL_RGBA: [f32; 4] = [0.40, 0.72, 0.45, 1.0];
const DEAD_RGBA: [f32; 4] = [0.55, 0.58, 0.62, 1.0];

const CAMERA_CENTRE: [f32; 2] = [0.0, 2.6];
const CAMERA_HEIGHT: f32 = 11.0;

/// Meio-comprimento de um elo. Os centros ficam a `2 × HALF` um do outro e a
/// âncora cai exatamente na fronteira entre dois elos.
const HALF: f32 = 0.5;

fn link(world: &mut World, name: &str, x: f32, y: f32, kind: BodyKind, rgba: [f32; 4]) {
    world.spawn((
        Name::new(name),
        RigidBody { kind },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: HALF,
                half_y: 0.14,
            },
            density: 1.0,
            ..Collider::default()
        },
        Sprite::atlas(WHITE_TILE_KEY, [HALF * 2.0, 0.28], rgba),
        Transform::from_translation(Vec2::new(x, y)),
    ));
}

fn joint(world: &mut World, name: &str, a: &str, b: &str, kind: JointKind, at: [f32; 2]) {
    world.spawn((
        Name::new(name),
        PhysicsJoint {
            body_a: stable_name_id(a),
            body_b: stable_name_id(b),
            kind,
            ..PhysicsJoint::of_kind(kind)
        },
        Transform::from_translation(Vec2::new(at[0], at[1])),
    ));
    ph2d_physics_ecs::resolve_body_names(world);
}

/// Uma fileira de elos ligados ponta a ponta, autorada da ESQUERDA para a
/// direita: `<prefix> 1` é sempre a cabeça (o lado A do primeiro joint).
fn row(
    world: &mut World,
    prefix: &str,
    x0: f32,
    y: f32,
    n: usize,
    kind: JointKind,
    rgba: [f32; 4],
) {
    for i in 0..n {
        link(
            world,
            &format!("{prefix} {}", i + 1),
            x0 + (i as f32) * HALF * 2.0,
            y,
            BodyKind::Dynamic,
            rgba,
        );
    }
    for i in 0..n - 1 {
        joint(
            world,
            &format!("J.{prefix}{}", i + 1),
            &format!("{prefix} {}", i + 1),
            &format!("{prefix} {}", i + 2),
            kind,
            [x0 + (i as f32) * HALF * 2.0 + HALF, y],
        );
    }
}

pub(crate) fn build_lead_scene(world: &mut World) {
    // 1. A CORDA — cadeia SOLTA de dobradiças. Pegar a cabeça arrasta o resto.
    row(world, "Rope", -6.5, 5.0, 4, JointKind::Pin, ROPE_RGBA);

    // 2. A PEÇA — cadeia SOLTA toda SOLDADA. Não dobra em lugar nenhum, então a
    //    única coisa que ela pode fazer é ir inteira.
    row(world, "Piece", -6.5, 3.0, 3, JointKind::Weld, PIECE_RGBA);

    // 3. O CONTROLE — a MESMA cadeia de dobradiças, presa a uma PAREDE. Nada
    //    aqui mudou, e é isso que a cena tem de mostrar.
    link(world, "Post", 2.0, 5.0, BodyKind::Static, WALL_RGBA);
    row(world, "Arm", 3.0, 5.0, 3, JointKind::Pin, ARM_RGBA);
    joint(world, "J.Post", "Post", "Arm 1", JointKind::Pin, [2.5, 5.0]);

    // 4. O SUPORTE — soldado a uma parede. Sem grau de liberdade nenhum: nem
    //    dobra, nem viaja, e o gesto RECUSA em vez de arrastar a parede.
    link(world, "Base", 2.0, 3.0, BodyKind::Static, WALL_RGBA);
    row(world, "Bracket", 3.0, 3.0, 2, JointKind::Weld, DEAD_RGBA);
    joint(
        world,
        "J.Base",
        "Base",
        "Bracket 1",
        JointKind::Weld,
        [2.5, 3.0],
    );
}

#[cfg(test)]
#[path = "physics_smoke_lead_tests.rs"]
mod tests;

impl crate::App {
    /// **Cena 74 (W-LeadDrag).** Quatro rigs, PAUSADA, painel aberto.
    pub(crate) fn physics_smoke_lead(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        build_lead_scene(gfx.sim.world_mut());
        gfx.camera.center = CAMERA_CENTRE;
        gfx.camera.height_world = CAMERA_HEIGHT;
        if let Some(hero) = gfx.hero_screen.as_mut() {
            hero.panel_visibility.insert("physics", true);
        }
        eprintln!(
            "[physics-smoke 74] A CORDA E A PECA -- o que acontece ao arrastar o corpo\n  \
               da ponta A. A cena esta PAUSADA e o painel PHYSICS esta aberto (tecla W);\n  \
               aperte B para ver colliders e joints.\n\n  \
               Dois destes gestos eram MORTOS: pegar a cabeca de uma cadeia solta era\n  \
               recusado, e uma peca soldada nao abria gesto nenhum -- o artista\n  \
               arrastava e nada acontecia, sem aviso.\n\n  \
               1. Abra a secao JOINTS e escolha FK. Arraste 'Rope 1', o elo mais a\n     \
                  ESQUERDA da fileira azul (a cabeca da cadeia).\n     \
                  -> A CORRENTE INTEIRA vai junto, RIGIDA, sem dobrar em lugar nenhum.\n     \
                  (medido: 4 corpos, todos com o mesmo deslocamento)\n\n  \
               2. Agora escolha IK e arraste a MESMA 'Rope 1'.\n     \
                  -> A corrente e' ARRASTADA: os elos de tras seguem com atraso e o\n        \
                     ULTIMO chega por ultimo, como puxar uma corda pela ponta.\n     \
                  (medido, nos primeiros 20 cm: 0,200 / 0,102 / 0,020 / 0,020 -- a\n      \
                   cauda mal se mexe; aos 2 m ja' e' 2,000 / 1,562 / 1,102 / 1,030,\n      \
                   porque puxada longa o bastante tudo acaba andando)\n\n  \
               3. FAZ CURVA: em IK, arraste 'Rope 1' em ARCO em vez de reto. A corda\n     \
                  guarda o CAMINHO -- ela nao volta a' forma anterior quando voce\n     \
                  retorna ao ponto de partida, e e' isso que a torna uma corda em vez\n     \
                  de um bloco articulado.\n\n  \
               4. A PECA (fileira ambar): em FK, arraste QUALQUER um dos tres elos.\n     \
                  Eles sao SOLDADOS, entao nao ha' onde dobrar e a peca vai inteira.\n     \
                  (medido: abre gesto, move 3 corpos, e nenhum deles GIRA)\n\n  \
               5. Pegue tambem 'Rope 3', do MEIO da corrente azul. A cabeca continua\n     \
                  sendo a mesma -- o rig tem UM 'para cima', e nao um por gesto.\n     \
                  (antes desta wave a raiz dependia de qual elo a mao pegou: pegar a\n      \
                   cabeca enraizava na cauda e vice-versa)\n\n  \
               6. O CONTROLE (fileira verde, presa ao poste): em FK arraste 'Arm 1'.\n     \
                  Ela GIRA em torno do poste, como sempre girou -- ha' uma dobradica\n     \
                  acima dela, e o ramo novo nao pode roubar esse caso.\n\n  \
               (!) O SUPORTE cinza esta SOLDADO a' parede. Arraste-o em FK: nada\n      \
                   acontece, e e' o certo -- sem dobradica e sem poder viajar, o gesto\n      \
                   RECUSA. Se a parede verde sair do lugar, e' bug (foi um defeito real\n      \
                   desta wave, achado por medicao no meio dela).\n\n  \
               (!) Alt+arrastar continua significando 'leve o rig inteiro' em qualquer\n      \
                   modo, inclusive nestes dois.\n",
        );
    }
}
