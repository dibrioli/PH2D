//! **O SENSOR DE PÉ** (`PH2D_PHYSICS_SMOKE=71`, W-PartSensor).
//!
//! A cena 70 mostra que uma PEÇA é editável. Esta mostra a coisa que a edição
//! passou a alcançar e que não chegava a lugar nenhum: marcar a peça como
//! **Sensor**.
//!
//! O caso é o mais comum que existe num módulo 2D — o *isGrounded* de Box2D e
//! Unity: um personagem com o tronco SÓLIDO e uma peça-sensor embaixo, que
//! responde *"estou no chão?"*. O chip sempre chegou ao solver (a peça de fato
//! atravessa); o que não chegava era o **canal de trigger**, indexado por CORPO
//! desde uma época em que um corpo tinha exatamente uma forma.
//!
//! ⚠️ **O oráculo é a COR do contorno** (tecla `B`): magenta APAGADO = nada
//! dentro, magenta VIVO = disparado. O overlay é o único consumidor deste canal
//! neste build, então antes da wave o pé ficava apagado para sempre.
//!
//! ⚠️ **A rotação é travada nos três personagens** (`LockRotation`): sem ela o
//! tronco tomba, o pé sai de baixo, e a cena passaria a medir a queda em vez do
//! contato.
//!
//! Os números da mensagem saem da sonda `probe_smoke_71`.

use ph2d_core::Vec2;
use ph2d_ecs::{ChildOf, Entity, Name, Transform, World};
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, GravityScale, LockRotation, RigidBody};
use ph2d_render::{Sprite, WHITE_TILE_KEY};

/// Onde cada personagem está, em `x`.
pub(crate) const LANES: [f32; 3] = [-3.0, 0.0, 3.0];
/// Os nomes das três faixas, na ordem de `LANES`.
pub(crate) const LANE_NAMES: [&str; 3] = ["Grounded", "Hovering", "Falling"];

/// O tronco (o CORPO).
const TORSO_HALF: [f32; 2] = [0.35, 0.9];
/// O pé (a PEÇA) — largo e chato, pendurado logo abaixo do tronco.
const FOOT_HALF: [f32; 2] = [0.5, 0.16];

/// O chão: topo em `y = 0`.
const GROUND_HALF_Y: f32 = 0.5;

/// Onde cada personagem NASCE. O que decide a cena inteira:
/// - **Grounded** já assentado, o pé mergulhado no chão;
/// - **Hovering** parado no ar (`GravityScale 0`) — o **CONTROLE**;
/// - **Falling** alto, para o pé ACENDER na aterrissagem.
const SPAWN_Y: [f32; 3] = [0.86, 2.6, 5.5];

const TORSO_RGBA: [f32; 4] = [0.70, 0.72, 0.80, 1.0];
const FOOT_RGBA: [f32; 4] = [0.90, 0.55, 0.88, 1.0];
const GROUND_RGBA: [f32; 4] = [0.34, 0.36, 0.42, 1.0];

const CAMERA_CENTRE: [f32; 2] = [0.0, 2.2];
const CAMERA_HEIGHT: f32 = 10.0;

/// **MEDIDO** pela sonda (`probe_smoke_71`): quais pés estão disparados depois
/// de 5 s. O que aterrissa acende e fica; o que paira nunca acende.
///
/// ⚠️ `#[cfg(test)]` porque é a forma **legível por máquina** do que a mensagem
/// diz em prosa — o irmão da cena 70 interpola os números dele no texto, e um
/// booleano não interpola em português. Fora dos testes ele não tem leitor, e um
/// `pub(crate)` sem leitor é a segunda resposta esperando alguém chamá-la.
#[cfg(test)]
pub(crate) const MEASURED_LIT: [bool; 3] = [true, false, true];

/// Um personagem: tronco (corpo) + pé (peça SENSORA).
fn character(world: &mut World, i: usize) -> Entity {
    let name = LANE_NAMES[i];
    let mut torso = world.spawn((
        Name::new(format!("{name} Torso")),
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: TORSO_HALF[0],
                half_y: TORSO_HALF[1],
            },
            density: 1.0,
            ..Collider::default()
        },
        // Sem isto o tronco tomba e o pé sai de baixo: a cena mediria a queda.
        LockRotation,
        Sprite::atlas(
            WHITE_TILE_KEY,
            [TORSO_HALF[0] * 2.0, TORSO_HALF[1] * 2.0],
            TORSO_RGBA,
        ),
        Transform::from_translation(Vec2::new(LANES[i], SPAWN_Y[i])),
    ));
    // O CONTROLE paira: sem peso, ele nunca desce até o chão.
    if i == 1 {
        torso.insert(GravityScale(0.0));
    }
    let torso = torso.id();
    world.spawn((
        Name::new(format!("{name} Foot")),
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: FOOT_HALF[0],
                half_y: FOOT_HALF[1],
            },
            // ⚠️ A linha inteira da wave. Sólido, o pé ESCORA o tronco; sensor,
            // ele atravessa e REPORTA.
            is_sensor: true,
            ..Collider::default()
        },
        Sprite::atlas(
            WHITE_TILE_KEY,
            [FOOT_HALF[0] * 2.0, FOOT_HALF[1] * 2.0],
            FOOT_RGBA,
        ),
        Transform::from_translation(Vec2::new(0.0, -(TORSO_HALF[1] + FOOT_HALF[1]))),
        ChildOf(torso),
    ));
    torso
}

pub(crate) fn build_characters(world: &mut World) {
    world.spawn((
        Name::new("Ground"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 8.0,
                half_y: GROUND_HALF_Y,
            },
            ..Collider::default()
        },
        Sprite::atlas(WHITE_TILE_KEY, [16.0, GROUND_HALF_Y * 2.0], GROUND_RGBA),
        Transform::from_translation(Vec2::new(0.0, -GROUND_HALF_Y)),
    ));
    for i in 0..LANES.len() {
        character(world, i);
    }
}

#[cfg(test)]
#[path = "physics_smoke_foot_tests.rs"]
mod tests;

impl crate::App {
    /// **Cena 71 (W-PartSensor).** O sensor de pé responde *"estou no chão?"*.
    pub(crate) fn physics_smoke_foot(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        build_characters(gfx.sim.world_mut());
        gfx.camera.center = CAMERA_CENTRE;
        gfx.camera.height_world = CAMERA_HEIGHT;
        // ⚠️ O contorno NÃO é armado aqui: `show_colliders` já nasce `true` no
        // `App`, e uma linha que reafirma um default é uma linha que ninguém
        // consegue distinguir de uma que faz algo.
        if let Some(hero) = gfx.hero_screen.as_mut() {
            hero.panel_visibility.insert("physics", true);
        }

        eprintln!(
            "[physics-smoke 71] O SENSOR DE PE -- ser sensor e' propriedade da FORMA,\n  \
               nunca do corpo. Tres personagens IDENTICOS: tronco SOLIDO (o corpo) e um\n  \
               pe' marcado **Sensor** (uma PECA, um filho com `Collider` e sem\n  \
               `RigidBody`). E' o isGrounded de Box2D e Unity.\n\n  \
               (!) O CONTORNO E' O ORACULO (ele ja' vem ligado; tecla B alterna).\n      \
                   Magenta APAGADO = nada dentro. Magenta VIVO = disparado.\n\n  \
               1. Toque Play e olhe os pes:\n     \
                  - ESQUERDA ('{n0}') ja' esta' no chao -- o pe' acende e FICA.\n     \
                  - MEIO ('{n1}') paira (GravityScale 0) -- o pe' NUNCA acende. E' o\n       \
                    CONTROLE: sem ele a cena nao distinguiria \"o sensor dispara\" de\n       \
                    \"todo pe' fica aceso\".\n     \
                  - DIREITA ('{n2}') cai de {drop:.1} m -- o pe' acende NO INSTANTE em\n       \
                    que encosta. E' a transicao, e ela e' o que um jogo consome.\n\n  \
               2. O TRONCO NAO ACENDE, e isso e' a wave. Marcar uma peca como sensor nao\n     \
                  pode transformar o corpo inteiro num gatilho; quem acende e' a FORMA\n     \
                  que o artista marcou.\n\n  \
               3. O PE ATRAVESSA. Repare que os tres troncos assentam na MESMA altura,\n     \
                  apoiados no proprio collider: um pe' sensor nao escora nada. Selecione\n     \
                  '{n0} Foot' e troque o chip para **Solid** -- na hora ele vira apoio e\n     \
                  o tronco sobe {lift:.2} m.\n\n  \
               === O que estava quebrado ===\n  \
               O chip SEMPRE chegou ao solver: medido, o tronco assenta em 1,6990 com o\n  \
               pe' solido e em 1,4990 com o pe' sensor -- a peca de fato atravessava. O\n  \
               que nao chegava era o CANAL: o par reportado pelo solver era\n  \
               (tronco, chao), e o codigo perguntava se o collider PROPRIO do tronco era\n  \
               sensor -- nao e'. A sobreposicao era descartada, `triggered_sensors()`\n  \
               voltava VAZIO, e como o contorno e' o unico consumidor deste canal o pe'\n  \
               ficava apagado PARA SEMPRE.\n\n  \
               A premissa estava escrita no proprio doc do wrapper: \"each body owns one\n  \
               collider, so the pair is reported by body handle\". Era verdade ate a\n  \
               W-Compound, e ninguem reconferiu a nota.\n",
            n0 = LANE_NAMES[0],
            n1 = LANE_NAMES[1],
            n2 = LANE_NAMES[2],
            drop = SPAWN_Y[2],
            lift = (FOOT_HALF[1] * 2.0),
        );
    }
}
