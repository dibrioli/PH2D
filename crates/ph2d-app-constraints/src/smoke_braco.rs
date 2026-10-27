//! Cena 8 — **Braço articulado** (`docs/BlenderConstraints/05_W5_diferenciais.md` §5): o quadrilátero do oráculo
//! (chão com cadeado, manivela, biela e balancim presos por quatro pinos) e uma MÃO soldada (Rígido) na ponta do
//! balancim. Arraste a manivela: o balancim segue; o balancim: a manivela segue; a mão: tudo segue. Abre com a
//! manivela escolhida.

use bevy_ecs::entity::Entity;
use bevy_ecs::world::World;
use ph2d_constraints::Pose2;
use ph2d_constraints_cola::Cola;
use ph2d_constraints_ecs::Colas;
use ph2d_ecs::{Locked, Transform, assign_missing_stable_ids, stable_id_of};
use ph2d_render::{DISC_TILE_KEY, WHITE_TILE_KEY};

use crate::smoke::peca;

/// As medidas do oráculo (`fixtures/freecad_1_1_4_assembly_cola.json`): chão 4, manivela 1, biela 3,5, balancim 3.
pub const CHAO: f64 = 4.0;
pub const MANIVELA: f64 = 1.0;
pub const BIELA: f64 = 3.5;
pub const BALANCIM: f64 = 3.0;
/// A manivela abre a 60° (a 0° deitava-se sobre o chão).
const MANIVELA_GRAUS: f64 = 60.0;
const GROSSURA: f32 = 0.18;
/// O canto de baixo-esquerda do chão, para o mecanismo ficar ao centro da vista.
const ORIGEM: [f64; 2] = [-2.0, -1.2];

fn pose(e: Entity, world: &World) -> Pose2 {
    let t = world
        .get::<Transform>(e)
        .copied()
        .unwrap_or(Transform::IDENTITY);
    Pose2 {
        x: f64::from(t.translation.x),
        y: f64::from(t.translation.y),
        rot: f64::from(t.rotation),
        sx: f64::from(t.scale.x),
        sy: f64::from(t.scale.y),
    }
}

/// Uma barra entre dois pontos do mundo (o centro dela é a origem do objecto).
#[expect(
    clippy::cast_possible_truncation,
    reason = "o `Transform` da casa é f32"
)]
fn barra(world: &mut World, nome: &str, p: [f64; 2], q: [f64; 2], cor: [f32; 4]) -> Entity {
    let centro = [((p[0] + q[0]) / 2.0) as f32, ((p[1] + q[1]) / 2.0) as f32];
    let graus = (q[1] - p[1]).atan2(q[0] - p[0]).to_degrees() as f32;
    let comprimento = (q[0] - p[0]).hypot(q[1] - p[1]) as f32;
    peca(
        world,
        nome,
        WHITE_TILE_KEY,
        centro,
        graus,
        [comprimento + GROSSURA, GROSSURA],
        cor,
    )
}

/// ⭐ Monta a cena 8. Devolve a manivela (o objecto escolhido à abertura).
pub(crate) fn braco_articulado(world: &mut World) -> Entity {
    let em = |x: f64, y: f64| [ORIGEM[0] + x, ORIGEM[1] + y];
    let (o2, o4) = (em(0.0, 0.0), em(CHAO, 0.0));
    let ang = MANIVELA_GRAUS.to_radians();
    let a = em(MANIVELA * ang.cos(), MANIVELA * ang.sin());
    // O ponto da biela com o balancim: a interseção dos dois círculos, o ramo de cima.
    let d = (o4[0] - a[0]).hypot(o4[1] - a[1]);
    let l = (BIELA * BIELA - BALANCIM * BALANCIM + d * d) / (2.0 * d);
    let h = (BIELA * BIELA - l * l).sqrt();
    let u = [(o4[0] - a[0]) / d, (o4[1] - a[1]) / d];
    let b = [a[0] + l * u[0] - h * u[1], a[1] + l * u[1] + h * u[0]];
    // A mão continua o balancim para lá da ponta.
    let dir = [(b[0] - o4[0]) / BALANCIM, (b[1] - o4[1]) / BALANCIM];
    let ponta = [b[0] + 0.8 * dir[0], b[1] + 0.8 * dir[1]];

    let chao = barra(world, "Chão", o2, o4, [0.35, 0.35, 0.38, 1.0]);
    world.entity_mut(chao).insert(Locked);
    let manivela = barra(world, "Manivela", o2, a, [0.9, 0.45, 0.1, 1.0]);
    let biela = barra(world, "Biela", a, b, [0.2, 0.55, 0.9, 1.0]);
    let balancim = barra(world, "Balancim", o4, b, [0.3, 0.75, 0.35, 1.0]);
    #[expect(
        clippy::cast_possible_truncation,
        reason = "o `Transform` da casa é f32"
    )]
    let mao = peca(
        world,
        "Mão",
        DISC_TILE_KEY,
        [ponta[0] as f32, ponta[1] as f32],
        0.0,
        [0.6, 0.6],
        [0.95, 0.8, 0.6, 1.0],
    );

    assign_missing_stable_ids(world);
    let sid = |e: Entity| stable_id_of(world, e).map_or(0, |s| s.0);
    let p = |e: Entity| pose(e, world);
    let pinos = [
        (chao, manivela, o2),
        (manivela, biela, a),
        (biela, balancim, b),
        (chao, balancim, o4),
    ];
    let mut colas = Colas::default();
    for (x, y, onde) in pinos {
        let (sx, sy, px, py) = (sid(x), sid(y), p(x), p(y));
        colas.push(|id| Cola::pino(id, sx, sy, px, py, onde));
    }
    let (sb, sm, pb, pm) = (sid(balancim), sid(mao), p(balancim), p(mao));
    colas.push(|id| Cola::rigida(id, sb, sm, pb, pm));
    world.entity_mut(chao).insert(colas);
    manivela
}
