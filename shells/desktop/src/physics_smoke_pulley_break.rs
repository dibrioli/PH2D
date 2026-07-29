//! **A cena da RUPTURA** (`PH2D_PHYSICS_SMOKE=60`, W-Pulley W2).
//!
//! Irmã de `physics_smoke_pulley.rs` pelo cap de 600 LOC, e o corte é por
//! assunto: lá moram o elevador (58) e o guincho (59) — o que uma corda FAZ —,
//! aqui o que acontece quando ela, ou um eixo, **deixa de aguentar**.
//!
//! Os números da mensagem saem da sonda `probe_smoke_60`, rodada sobre ESTAS
//! constantes.

use super::physics_smoke_pulley::{BOOM_Y, GROUND, HOOK_Y, POST, R, ball};
use ph2d_core::Vec2;
use ph2d_ecs::{Name, Transform, World, stable_name_id};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, JointKind, PhysicsJoint, PulleyWheel, RigidBody, WrapSide,
};
use ph2d_render::{Sprite, WHITE_TILE_KEY};

const SNAPS: [f32; 4] = [0.95, 0.45, 0.45, 1.0];
const AXLE: [f32; 4] = [0.95, 0.80, 0.35, 1.0];
const HOLDS: [f32; 4] = [0.45, 0.90, 0.55, 1.0];

/// **O que a corda dos dois primeiros rigs aguenta**, newtons — a MESMA nos
/// dois. 3 kg pesam 29,4 N e 5 kg pesam 49,1: 40 fica no meio, que é o que faz
/// os dois rigs diferirem por uma caixa a mais e não por dois números.
const ROPE_LIMIT: f32 = 40.0;
/// **O que o eixo do terceiro aguenta**, newtons. A resultante de um enlace de
/// ~90° é `√2` da tensão, então 3 kg fazem ~41 N no eixo — bem acima de 20.
const AXLE_LIMIT: f32 = 20.0;
/// **A que velocidade a carga do rig do meio já está descendo** quando a corda a
/// pega, m/s — o número que a sonda escolheu (ver a cena).
const DROP_SPEED: f32 = 6.0;

/// Uma corda com uma ponta amarrada num poste e uma carga na outra, passando por
/// UMA roldana no alto — e, por escolha, uma corda fraca ou um eixo fraco.
#[allow(clippy::too_many_arguments)]
fn breaker(
    world: &mut World,
    tag: &str,
    x: f32,
    mass: f32,
    drop_speed: f32,
    rope_break: Option<f32>,
    axle_break: Option<f32>,
    rgba: [f32; 4],
) {
    let post = format!("{tag} Post");
    let load = format!("{tag} Load");
    let rope = format!("{tag} Rope");
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
        Transform::from_translation(Vec2::new(x - 2.4, BOOM_Y)),
    ));
    ball(world, &load, x, mass, rgba, R);
    {
        let mut q = world.query::<(&Name, &mut Transform)>();
        for (n, mut t) in q.iter_mut(world) {
            if n.as_str() == load {
                t.translation.y = HOOK_Y;
            }
        }
    }
    if drop_speed != 0.0 {
        let e = {
            let mut q = world.query::<(ph2d_ecs::Entity, &Name)>();
            q.iter(world)
                .find(|(_, n)| n.as_str() == load)
                .map(|(e, _)| e)
                .expect("a carga acabou de nascer")
        };
        world
            .entity_mut(e)
            .insert(ph2d_physics_ecs::InitialVelocity {
                linvel: [0.0, -drop_speed],
                angvel: 0.0,
            });
    }
    // ⚠️ **O TRANCO é a diferença entre os dois primeiros rigs**, e ele entra
    // pela velocidade inicial (W9): a carga já está descendo quando a corda a
    // pega. Um limiar abaixo do PESO parado partiria no primeiro quadro e o
    // artista nunca veria a corda segurar — e o que uma ruptura ensina é
    // justamente o instante em que ela deixa de segurar.
    world.spawn((
        Name::new(rope.clone()),
        PhysicsJoint {
            body_a: stable_name_id(&post),
            body_b: stable_name_id(&load),
            kind: JointKind::Pulley,
            break_enabled: rope_break.is_some(),
            break_force: rope_break.unwrap_or(PhysicsJoint::DEFAULT_BREAK_FORCE),
            ..PhysicsJoint::of_kind(JointKind::Pulley)
        },
        Transform::from_translation(Vec2::new(x - 2.4, BOOM_Y)),
    ));
    world.spawn((
        Name::new(format!("{rope} Wheel 1")),
        PulleyWheel {
            rope: stable_name_id(&rope),
            order: 0,
            radius: 0.45,
            wrap: WrapSide::Auto,
            motor_speed: 0.0,
            break_enabled: axle_break.is_some(),
            break_force: axle_break.unwrap_or(PulleyWheel::DEFAULT_BREAK_FORCE),
        },
        Transform::from_translation(Vec2::new(x, BOOM_Y)),
    ));
}

/// **Quanto a carga do rig VERDE anda em 2 s**, metros — ela fica onde está,
/// porque 3 kg pesam 29,4 N e a corda aguenta 40.
pub(crate) const MEASURED_HOLDS_DRIFT: f32 = 0.01;
/// **O TRANCO**, newtons — a carga do meio já chega descendo a 6 m/s, e uma
/// corda inextensível que a para num sub-passo aplica uma força que **não tem
/// relação com o peso**: 177× ele.
pub(crate) const MEASURED_JERK: f32 = 5202.0;
/// **O que o EIXO do terceiro rig carregava** quando cedeu, newtons — a
/// resultante de um enlace de ~90° sobre uma corda que segura 29,4 N.
pub(crate) const MEASURED_AXLE_LOAD: f32 = 52.4;

/// Monta a cena 60.
pub(crate) fn build_break(world: &mut World) {
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
        Sprite::atlas(WHITE_TILE_KEY, [32.0, 0.6], GROUND),
        Transform::from_translation(Vec2::new(0.0, -0.6)),
    ));
    // ⚠️ **Cada rig difere do vizinho por UM número**, e a corda é a MESMA nos
    // dois primeiros (40 N): o que muda é o peso pendurado nela. É assim que
    // "break force" significa alguma coisa — *uma caixa a mais do que ela
    // aguenta* —, e não "um limiar abaixo do próprio peso", que parte no
    // primeiro quadro e não ensina nada.
    breaker(
        world,
        "Holds",
        -7.0,
        3.0,
        0.0,
        Some(ROPE_LIMIT),
        None,
        HOLDS,
    );
    breaker(
        world,
        "Snap",
        0.0,
        3.0,
        DROP_SPEED,
        Some(ROPE_LIMIT),
        None,
        SNAPS,
    );
    // E o terceiro tem a corda INTEIRA e o EIXO fraco: a resultante do enlace é
    // ~1,4x a tensão, então o eixo cede antes de a corda sentir qualquer coisa.
    breaker(world, "Axle", 7.0, 3.0, 0.0, None, Some(AXLE_LIMIT), AXLE);
}

impl crate::App {
    /// **Cena 60 (W-Pulley W2).** Três cordas: uma que segura, uma que parte, e
    /// uma cujo EIXO cede antes de a corda sentir qualquer coisa.
    ///
    /// Os números saem da sonda `probe_smoke_60`, rodada sobre ESTA cena.
    pub(crate) fn physics_smoke_break(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        build_break(gfx.sim.world_mut());
        if let Some(hero) = gfx.hero_screen.as_mut() {
            hero.panel_visibility.insert("physics", true);
        }

        eprintln!(
            "[physics-smoke 60] A RUPTURA -- a corda parte, e o eixo tambem.\n  \
               Aperte B para ver os vinculos, e depois PLAY (o toggle Physics ja esta armado).\n\n  \
               1. VERDE (esquerda) -- 3 kg numa corda de {limit:.0} N. O peso e 29,4 N, entao ela\n     \
                  SEGURA: a carga fica onde esta ({holds:.2} m em 2 s). E o controle.\n  \
               2. VERMELHO (meio) -- a MESMA corda de {limit:.0} N e o MESMO peso de 3 kg. A\n     \
                  unica diferenca e que esta carga ja chega DESCENDO a 6 m/s -- e ai a\n     \
                  corda parte na hora, carregando {jerk:.0} N.\n     \
                  ⚠️ Esse numero e o coracao da cena: uma corda INEXTENSIVEL que para uma\n     \
                  massa em movimento aplica uma forca que nao tem relacao com o peso --\n     \
                  177x ele, aqui. 'Break force' e um limiar de CARGA, e um tranco e uma\n     \
                  carga enorme.\n  \
               3. AMARELO (direita) -- a corda e INDESTRUTIVEL e o EIXO da roldana aguenta\n     \
                  so {axle:.0} N. A corda segura 29,4 N, mas o eixo carrega a RESULTANTE do\n     \
                  desvio (~1,4x num enlace de 90 graus, 2x num de 180): {axle_load:.1} N. O eixo\n     \
                  cede, a roldana SAI DA ROTA, e a carga cai -- sem tranco nenhum, porque\n     \
                  a rota sem ela e mais CURTA, entao a corda fica frouxa por construcao.\n  \
               4. OLHE OS READOUTS. Com B ligado, cada corda mostra 'carga / limiar' ao lado\n     \
                  dela; quando ela parte, o numero CONGELA na carga que cruzou. E o mesmo\n     \
                  readout dos outros joints -- ate esta wave a polia era o unico tipo que\n     \
                  nao podia partir, e mostrava um '0 / 0 N' permanente.\n  \
               5. SELECIONE a corda ('Holds Rope') na Hierarquia. No Inspector, na secao\n     \
                  Physics Joint, ha 'Breakable' e 'Break Force (N)'. Baixe o limiar COM O\n     \
                  RELOGIO ANDANDO e ela parte na hora.\n  \
               6. SELECIONE uma roldana ('Holds Rope Wheel 1'). Na secao 'Pulley Wheel' ha\n     \
                  'Axle Breaks' e 'Break Force (N)' -- o limiar do EIXO, que e outro numero\n     \
                  e outra grandeza. Uma corda tem tensao uniforme (um limiar so, nas duas\n     \
                  pontas); cada eixo carrega a resultante do enlace DELE.\n  \
               7. Reset devolve tudo: uma ruptura e um fato da CORRIDA, nunca uma edicao\n     \
                  que o artista tenha de desfazer. Scrub para tras tambem -- a corda volta\n     \
                  inteira e rompe de novo no mesmo tique.",
            limit = ROPE_LIMIT,
            holds = MEASURED_HOLDS_DRIFT,
            jerk = MEASURED_JERK,
            axle = AXLE_LIMIT,
            axle_load = MEASURED_AXLE_LOAD,
        );
    }
}

#[cfg(test)]
#[path = "physics_smoke_pulley_break_tests.rs"]
mod tests;
