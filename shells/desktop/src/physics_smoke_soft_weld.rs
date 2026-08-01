//! **A cena da SOLDA QUE CEDE** (`PH2D_PHYSICS_SMOKE=68`, W-SoftWeld).
//!
//! Quatro vigas em balanço, todas idênticas — mesmo braço, mesma parede, mesma
//! gravidade. Só a SOLDA difere, e é isso que faz da cena um A/B em vez de uma
//! demonstração.
//!
//! ⚠️ **A viga é HORIZONTAL de propósito.** Um poste vertical soldado pela base
//! não verga sob o próprio peso — a carga corre ao longo dele e os eixos lineares
//! estão travados —, então ele mostraria a solda mole como se ela não fizesse
//! nada. O peso de um braço em balanço faz TORQUE na solda, que é exatamente a
//! grandeza que a mola nova governa.
//!
//! Os números da mensagem saem da sonda `probe_smoke_68`, rodada sobre ESTAS
//! constantes.

use ph2d_core::Vec2;
use ph2d_ecs::{Name, Transform, World, stable_name_id};
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, JointKind, PhysicsJoint, RigidBody};
use ph2d_render::{Sprite, WHITE_TILE_KEY};

const ARM_HALF: [f32; 2] = [0.9, 0.12];
const WALL_HALF: [f32; 2] = [0.18, 0.5];
const ARM_Y: f32 = 4.0;

/// Onde cada parede fica. O braço nasce à direita dela, soldado pela ponta
/// esquerda.
const LANES: [f32; 4] = [-6.6, -2.2, 2.2, 6.6];

/// A rigidez de cada faixa. A primeira é ignorada (a solda é RÍGIDA); a terceira
/// é o extremo MOLE do knob, e a quarta repete o default para o teste do impacto.
pub(crate) const LANE_STIFFNESS: [f32; 4] = [0.0, 30.0, 3.0, 30.0];

const WALL: [f32; 4] = [0.30, 0.32, 0.36, 1.0];
const RIGID: [f32; 4] = [0.62, 0.64, 0.68, 1.0];
const SOFT: [f32; 4] = [0.35, 0.75, 0.45, 1.0];
const FLOPPY: [f32; 4] = [0.90, 0.60, 0.25, 1.0];
const IMPACT: [f32; 4] = [0.40, 0.60, 0.90, 1.0];
const BALL: [f32; 4] = [0.85, 0.30, 0.35, 1.0];

const BALL_RADIUS: f32 = 0.45;
/// De onde a bola larga. Alta o bastante para o braço já ter assentado quando ela
/// chega — senão o smoke mostraria o impacto e o repouso misturados.
const BALL_Y: f32 = 9.0;

const CAMERA_CENTRE: [f32; 2] = [0.0, 4.0];
const CAMERA_HEIGHT: f32 = 12.0;

/// **MEDIDO** pela sonda `probe_smoke_68`: quanto cada braço pendeu, em graus,
/// depois de 6 s. O primeiro é o CONTROLE e tem de ser zero.
///
/// ⚠️ **Eles NÃO são os da fixture do wrapper**, e a sonda os corrigiu: lá o
/// braço é 1,0 × 0,2 m e o default pende 5,34°; aqui ele é 1,8 × 0,24 m e pende
/// 3,03°. O pendor é função da GEOMETRIA, então um número emprestado da fixture
/// vizinha é um número sobre outra cena.
pub(crate) const MEASURED_DROOP_DEG: [f32; 4] = [0.00, 3.03, 26.83, 3.01];
/// **MEDIDO**: quanto a ponta soldada se afastou da parede, em metros. Numa solda
/// isto é zero — é a metade que separa VERGAR de SOLTAR, e a variante *tudo mole*
/// (nada travado, três molas) mediu 0,92 m aqui antes de ser descartada.
pub(crate) const MEASURED_SEPARATION_M: f32 = 0.0000;
/// **MEDIDO**: o pico de que a viga da 4ª faixa verga sob a bola, em graus, e o
/// ângulo em que ela volta a assentar depois de a bola rolar fora.
pub(crate) const MEASURED_IMPACT_PEAK_DEG: f32 = 62.42;
pub(crate) const MEASURED_IMPACT_REST_DEG: f32 = 3.01;

/// Uma faixa: parede estática + braço em balanço + a solda entre eles.
fn lane(world: &mut World, i: usize, name: &str, soft: bool, tint: [f32; 4]) {
    let x = LANES[i];
    let wall = format!("{name} Wall");
    world.spawn((
        Name::new(wall.clone()),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: WALL_HALF[0],
                half_y: WALL_HALF[1],
            },
            ..Collider::default()
        },
        Sprite::atlas(
            WHITE_TILE_KEY,
            [WALL_HALF[0] * 2.0, WALL_HALF[1] * 2.0],
            WALL,
        ),
        Transform::from_translation(Vec2::new(x, ARM_Y)),
    ));
    let arm = format!("{name} Arm");
    world.spawn((
        Name::new(arm.clone()),
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: ARM_HALF[0],
                half_y: ARM_HALF[1],
            },
            ..Collider::default()
        },
        Sprite::atlas(WHITE_TILE_KEY, [ARM_HALF[0] * 2.0, ARM_HALF[1] * 2.0], tint),
        // O centro do braço fica meio comprimento à direita da parede, então a
        // ponta ESQUERDA dele encosta nela — e é ali que a solda vai.
        Transform::from_translation(Vec2::new(x + ARM_HALF[0], ARM_Y)),
    ));
    world.spawn((
        Name::new(format!("{name} Weld")),
        PhysicsJoint {
            body_a: stable_name_id(&wall),
            body_b: stable_name_id(&arm),
            kind: JointKind::Weld,
            soft,
            stiffness: LANE_STIFFNESS[i],
            ..PhysicsJoint::default()
        },
        Transform::from_translation(Vec2::new(x, ARM_Y)),
    ));
}

pub(crate) fn build_soft_weld(world: &mut World) {
    lane(world, 0, "Rigid", false, RIGID);
    lane(world, 1, "Soft", true, SOFT);
    lane(world, 2, "Floppy", true, FLOPPY);
    lane(world, 3, "Impact", true, IMPACT);

    // A bola que cai sobre a ponta da 4ª viga. Ela verga muito, a bola escorrega
    // pela rampa que a própria vergadura fez, e o braço VOLTA — que é a metade
    // que um pendor parado não consegue mostrar.
    world.spawn((
        Name::new("Weight"),
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        Collider {
            shape: ColliderShape::Ball {
                radius: BALL_RADIUS,
            },
            density: 6.0,
            ..Collider::default()
        },
        Sprite::atlas(WHITE_TILE_KEY, [BALL_RADIUS * 2.0, BALL_RADIUS * 2.0], BALL),
        Transform::from_translation(Vec2::new(LANES[3] + ARM_HALF[0] * 1.5, BALL_Y)),
    ));
}

#[cfg(test)]
#[path = "physics_smoke_soft_weld_tests.rs"]
mod tests;

impl crate::App {
    /// **Cena 68 (W-SoftWeld).** Quatro vigas iguais, quatro soldas diferentes.
    pub(crate) fn physics_smoke_soft_weld(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        build_soft_weld(gfx.sim.world_mut());
        gfx.camera.center = CAMERA_CENTRE;
        gfx.camera.height_world = CAMERA_HEIGHT;
        if let Some(hero) = gfx.hero_screen.as_mut() {
            hero.panel_visibility.insert("physics", true);
        }

        eprintln!(
            "[physics-smoke 68] A SOLDA QUE CEDE -- ate aqui este conjunto so sabia\n  \
               segurar um angulo ABSOLUTAMENTE (Weld, Slider) ou deixa-lo\n  \
               INTEIRAMENTE LIVRE (Spring, Rope, Rod, o giro do Wheel). Nao havia\n  \
               nada no meio: um poste que balanca e volta, um pescoco que resiste\n  \
               mas cede, uma placa que treme -- nenhum era exprimivel.\n\n  \
               As quatro vigas sao IDENTICAS (mesmo braco de {arm:.1} m, mesma parede,\n  \
               mesma gravidade). So a SOLDA difere.\n\n  \
               1. CINZA   -- solda RIGIDA, a de sempre. E o CONTROLE: {d0:.2} graus.\n  \
               2. VERDE   -- solda MOLE, dureza {k1:.0} (o default): pende {d1:.2} graus e PARA.\n  \
               3. LARANJA -- solda MOLE, dureza {k2:.0}: pende {d2:.1} graus. E o mesmo knob.\n  \
               4. AZUL    -- solda MOLE + uma bola pesada caindo na ponta.\n\n  \
               A 4a faixa e a que mostra a palavra inteira: a viga verga ate\n  \
               {peak:.1} graus sob a bola, a bola escorrega pela rampa que a propria\n  \
               vergadura fez, e o braco VOLTA para {rest:.2} graus. Uma solda rigida\n  \
               nao teria se mexido; uma dobradica nao teria voltado.\n\n  \
               (!) E AS PECAS CONTINUAM UMA. A ponta soldada nao se afasta da parede\n     \
               ({sep:.4} m em todas as faixas). Foi a medicao que escolheu o desenho:\n     \
               com os TRES eixos moles o braco derivava 0,92 m para longe da parede e\n     \
               balancava 104 graus sem nunca assentar -- as pecas vinham APART, que se\n     \
               le como a solda FALHANDO, nao vergando. Hoje so o ANGULO cede.\n\n  \
               AUTORE VOCE MESMO: selecione qualquer '... Weld' na Hierarquia. Na\n  \
               secao Joint aparece a chave [Rigid | Soft].\n  \
               - clique 'Soft' na CINZA: ela passa a pender como a verde.\n  \
               - clique 'Rigid' na LARANJA: ela endireita na hora.\n  \
               - com 'Soft' marcado, Stiffness e Damping aparecem LOGO ABAIXO. Sao os\n     \
                 MESMOS dois campos que uma Spring usa -- e com 'Rigid' eles somem, em\n     \
                 vez de ficar na tela sem alcancar o solver.\n  \
               - baixe a Stiffness da verde ate 1: ela vira borracha (65 graus). Suba\n     \
                 para 1000: ela endurece (0,16 grau). A faixa inteira ASSENTA.\n\n  \
               (!) SCRUB: toque Play, deixe as vigas assentarem e arraste a regua PARA\n     \
               TRAS ate o zero. Elas tem de voltar a ficar RETAS e cair de novo igual.\n     \
               Se a verde voltar rigida, o `soft` nao sobreviveu ao rebuild.\n",
            arm = ARM_HALF[0] * 2.0,
            d0 = MEASURED_DROOP_DEG[0],
            k1 = LANE_STIFFNESS[1],
            d1 = MEASURED_DROOP_DEG[1],
            k2 = LANE_STIFFNESS[2],
            d2 = MEASURED_DROOP_DEG[2],
            peak = MEASURED_IMPACT_PEAK_DEG,
            rest = MEASURED_IMPACT_REST_DEG,
            sep = MEASURED_SEPARATION_M,
        );
    }
}
