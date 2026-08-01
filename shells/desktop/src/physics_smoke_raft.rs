//! **A JANGADA COMPOSTA** (`PH2D_PHYSICS_SMOKE=72`, W-CompoundZone).
//!
//! Três jangadas na MESMA poça, todas de mesma silhueta e mesma massa. A única
//! diferença é **de que elas são feitas**:
//!
//! - **Single** — uma caixa larga. O **CONTROLE**.
//! - **Compound** — duas metades, a segunda como PEÇA.
//! - **Sensor Cargo** — a caixa larga com uma peça-sensor EMPILHADA em cima.
//!
//! ⚠️ **O oráculo é a INCLINAÇÃO**, e é isso que torna a cena decisiva: antes da
//! wave a composta **CAPOTAVA** (−90°), porque o empuxo saía de uma forma só e
//! portanto nascia descentrado. Uma força no lugar errado é um torque.
//!
//! Os números da mensagem saem da sonda `probe_smoke_72`.

use ph2d_core::Vec2;
use ph2d_ecs::{ChildOf, Entity, Name, Transform, World};
use ph2d_physics_ecs::{AreaBuoyancy, AreaDrag, BodyKind, Collider, ColliderShape, RigidBody};
use ph2d_render::{Sprite, WHITE_TILE_KEY};

/// Meia-largura de cada metade; a silhueta inteira mede `4 × HALF_X`.
const HALF_X: f32 = 0.6;
const HALF_Y: f32 = 0.25;

/// Onde cada jangada nasce, em `x` — e os nomes, na mesma ordem.
pub(crate) const LANES: [f32; 3] = [-4.0, 0.0, 4.0];
pub(crate) const LANE_NAMES: [&str; 3] = ["Single", "Compound", "Sensor Cargo"];

/// A poça: `4×` a densidade dos corpos, e um meio que RESISTE.
///
/// ⚠️ O arrasto não é enfeite — empuxo sem resistência é uma **mola sem
/// amortecimento**, e as jangadas balançariam para sempre em vez de assentarem
/// numa linha d'água que o artista possa ler.
const FLUID_DENSITY: f32 = 4.0;
const POOL_HALF: [f32; 2] = [7.0, 3.0];

/// **MEDIDO** (`probe_smoke_72`): a altura de repouso do centro de cada jangada.
///
/// Arquimedes prevê `0,125` para quem desloca a silhueta inteira, e as duas
/// primeiras batem. A terceira carrega o peso da caixa-sensor e desloca a MESMA
/// água, então afunda — nivelada.
pub(crate) const MEASURED_Y: [f32; 3] = [0.125, 0.125, 0.063];

const SINGLE_RGBA: [f32; 4] = [0.72, 0.74, 0.80, 1.0];
const HALF_A_RGBA: [f32; 4] = [0.55, 0.75, 0.60, 1.0];
const HALF_B_RGBA: [f32; 4] = [0.40, 0.62, 0.85, 1.0];
const SENSOR_RGBA: [f32; 4] = [0.90, 0.55, 0.88, 1.0];
const WATER_RGBA: [f32; 4] = [0.20, 0.34, 0.46, 1.0];

const CAMERA_CENTRE: [f32; 2] = [0.0, -0.4];
const CAMERA_HEIGHT: f32 = 7.0;

fn cuboid(half_x: f32, half_y: f32) -> Collider {
    Collider {
        shape: ColliderShape::Cuboid { half_x, half_y },
        density: 1.0,
        ..Collider::default()
    }
}

/// Uma jangada.
///
/// - `0` **Single** — uma caixa larga, o CONTROLE.
/// - `1` **Compound** — a MESMA silhueta partida em duas, a segunda como PEÇA.
/// - `2` **Sensor Cargo** — a caixa larga com uma peça-sensor EMPILHADA em cima.
///
/// ⚠️ **A carga do lane 2 fica CENTRADA, e a escolha é deliberada.** A primeira
/// versão punha o sensor ao LADO (o convés da composta), e a jangada **capotava**
/// — fisicamente correto (metade dela tem peso e não desloca água, logo o empuxo
/// é descentrado) e um desastre como demonstração: um barco de pé é exatamente a
/// imagem do BUG que esta cena existe para mostrar consertado. Centrada, ela
/// mostra a MESMA lei sem ambiguidade: **afunda, nivelada.**
fn raft(world: &mut World, i: usize) -> Entity {
    let name = LANE_NAMES[i];
    let split = i == 1;
    // A composta ocupa `x ∈ [−h, 3h]` a partir da origem do corpo, então as
    // outras são centradas em `+h` para as três silhuetas serem a MESMA.
    let x = LANES[i] + if split { 0.0 } else { HALF_X };
    let hull_half_x = if split { HALF_X } else { HALF_X * 2.0 };
    let body = world
        .spawn((
            Name::new(name),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            cuboid(hull_half_x, HALF_Y),
            Sprite::atlas(
                WHITE_TILE_KEY,
                [hull_half_x * 2.0, HALF_Y * 2.0],
                if split { HALF_A_RGBA } else { SINGLE_RGBA },
            ),
            Transform::from_translation(Vec2::new(x, 2.2)),
        ))
        .id();
    if split {
        world.spawn((
            Name::new(format!("{name} Deck")),
            cuboid(HALF_X, HALF_Y),
            Sprite::atlas(WHITE_TILE_KEY, [HALF_X * 2.0, HALF_Y * 2.0], HALF_B_RGBA),
            Transform::from_translation(Vec2::new(HALF_X * 2.0, 0.0)),
            ChildOf(body),
        ));
    }
    if i == 2 {
        world.spawn((
            Name::new(format!("{name} Crate")),
            Collider {
                is_sensor: true,
                ..cuboid(HALF_X, HALF_Y)
            },
            Sprite::atlas(WHITE_TILE_KEY, [HALF_X * 2.0, HALF_Y * 2.0], SENSOR_RGBA),
            Transform::from_translation(Vec2::new(0.0, HALF_Y * 2.0)),
            ChildOf(body),
        ));
    }
    body
}

pub(crate) fn build_rafts(world: &mut World) {
    world.spawn((
        Name::new("Pool"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            is_sensor: true,
            ..cuboid(POOL_HALF[0], POOL_HALF[1])
        },
        AreaBuoyancy(FLUID_DENSITY),
        AreaDrag(0.6),
        Sprite::atlas(
            WHITE_TILE_KEY,
            [POOL_HALF[0] * 2.0, POOL_HALF[1] * 2.0],
            WATER_RGBA,
        ),
        // Topo da poça (a superfície) em `y = 0`.
        Transform::from_translation(Vec2::new(0.0, -POOL_HALF[1])),
    ));
    for i in 0..LANES.len() {
        raft(world, i);
    }
}

#[cfg(test)]
#[path = "physics_smoke_raft_tests.rs"]
mod tests;

impl crate::App {
    /// **Cena 72 (W-CompoundZone).** A jangada composta boia NIVELADA.
    pub(crate) fn physics_smoke_raft(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        build_rafts(gfx.sim.world_mut());
        gfx.camera.center = CAMERA_CENTRE;
        gfx.camera.height_world = CAMERA_HEIGHT;
        if let Some(hero) = gfx.hero_screen.as_mut() {
            hero.panel_visibility.insert("physics", true);
        }

        eprintln!(
            "[physics-smoke 72] A JANGADA COMPOSTA -- uma ZONA tem de ver o corpo\n  \
               inteiro, nao a primeira forma dele. Tres jangadas de MESMA silhueta\n  \
               (2,40 x 0,50) e MESMA massa (1,200000, medida), so' mudando de que\n  \
               elas sao FEITAS.\n\n  \
               1. Toque Play e olhe as tres. As TRES tem de ficar NIVELADAS:\n     \
                  - ESQUERDA '{n0}' -- uma caixa larga. E' o CONTROLE: sem ele nada\n       \
                    nesta cena e' atribuivel. Centro em {y0:.3}.\n     \
                  - MEIO '{n1}' -- a MESMA silhueta partida em duas (verde = o\n       \
                    CORPO, azul = a PECA). Mesma linha d'agua: {y1:.3}.\n     \
                  - DIREITA '{n2}' -- a caixa larga com uma peca-SENSOR empilhada\n       \
                    (magenta). Ela desloca a MESMA agua e carrega peso a mais,\n       \
                    entao afunda: {y2:.3}. Um sensor e' marcador, nao materia.\n\n  \
               2. O QUE ESTAVA QUEBRADO. Antes desta wave a do meio CAPOTAVA: medido,\n     \
                  -90,007 graus, de pe' na agua. O empuxo saia de UMA forma so', entao\n     \
                  nascia DESCENTRADO -- e uma forca no lugar errado e' um torque.\n\n  \
               (!) A intuicao erra o sintoma: meia-forca faria esperar \"afunda o\n      \
                   dobro\". Meia-forca e forca-no-lugar-errado sao defeitos\n      \
                   diferentes, e este era o segundo.\n\n  \
               3. E O DEFEITO ERA INVISIVEL POR COMPENSACAO. A zona aplicava o empuxo\n     \
                  uma vez por PAR de colliders, entao a composta levava\n     \
                  \"2 x meia-forca\" = a forca certa. A LINHA D'AGUA parecia certa; so'\n     \
                  a INCLINACAO denunciava. Consertar so' uma das metades a faria boiar\n     \
                  com metade da submersao.\n\n  \
               4. Selecione '{n1} Deck' na Hierarquia: e' uma PECA, com a face de\n     \
                  Physics Body que a wave anterior deu a ela. Troque o chip para\n     \
                  **Sensor** e a jangada do meio CAPOTA -- e esta' certo: metade dela\n     \
                  passa a ter peso sem deslocar agua, o que e' um barco desequilibrado.\n     \
                  Volte para **Solid** e ela se endireita.\n\n  \
               (!) Toque B para o contorno. O collider da poca e' magenta (sensor); as\n      \
                   pecas sao desenhadas na cor do DONO, porque e' o corpo dele que as\n      \
                   governa.\n",
            n0 = LANE_NAMES[0],
            n1 = LANE_NAMES[1],
            n2 = LANE_NAMES[2],
            y0 = MEASURED_Y[0],
            y1 = MEASURED_Y[1],
            y2 = MEASURED_Y[2],
        );
    }
}
