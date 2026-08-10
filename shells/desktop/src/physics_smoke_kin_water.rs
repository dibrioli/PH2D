//! **A ÁGUA E O MODO CINEMÁTICO** (`PH2D_PHYSICS_SMOKE=104`) — a `W-KinFluid`.
//!
//! Pergunta do Enio (2026-08-09): *"testou o kinemático na água? ou ele não
//! funciona lá?"*. Não funcionava, e a cena existe para o veredito ser de OLHO:
//! medido antes da cura, numa poça funda de 4 s, o cinemático **afundava
//! 139,67 m** enquanto a cápsula solta e o player dinâmico boiavam.
//!
//! # ⚠️ Três sujeitos, e é a cena inteira
//!
//! O oráculo desta wave não é uma linha d'água — é a **PARIDADE ENTRE MODOS**, e
//! um número não se lê num screenshot. Então a poça carrega três corpos de forma
//! e densidade IDÊNTICAS, lado a lado:
//!
//! - **verde** — a cápsula solta, sem `PlatformPlayer` nenhum: o que a água faz a
//!   um corpo qualquer;
//! - **âmbar** — o player DINÂMICO, o que já funcionava;
//! - **azul** — o player CINEMÁTICO, o que não existia para a água.
//!
//! Os três largados da MESMA altura, no MESMO instante. *Se o azul não
//! acompanhar os outros dois, pare.*
//!
//! ⚠️ **E o que se julga é o CONJUNTO, não uma altura:** nesta poça o player
//! bobeia ~1,44 m de amplitude nos **dois** modos (a cápsula solta faz 0,81) —
//! isso é anterior a esta wave e é o que os dois players fazem. *Uma amostra
//! única de um sistema que oscila não é um repouso*, e foi exatamente esse erro
//! que a primeira medição desta cura cometeu.
//!
//! ⚠️ **A poça é FUNDA de propósito.** Com fundo ao alcance da perna o sensor de
//! chão responde, o personagem fica de pé e a água deixa de ser a única coisa a
//! agi-lo — a cena mediria outra pergunta.
//!
//! Os números da mensagem saem da sonda `probe_smoke_104`.

use ph2d_core::Vec2;
use ph2d_ecs::{Name, Transform, World};
use ph2d_physics_ecs::{
    AreaBuoyancy, AreaDrag, BodyKind, Collider, ColliderShape, LockRotation, PlatformPlayer,
    PlayerMode, RigidBody,
};
use ph2d_render::{Sprite, WHITE_TILE_KEY};

/// A poça: superfície em `y = 0`, funda o bastante para nada tocar o fundo.
const POOL_HALF: [f32; 2] = [12.0, 6.0];
/// ⚠️ **O arrasto não é enfeite** — empuxo sem resistência é uma mola sem
/// amortecimento. Medido nesta poça: com `AreaDrag 0` o cinemático oscila
/// **2,90 m** de amplitude entre o 3.º e o 6.º segundo, contra **1,44** aqui.
const POOL_DRAG: f32 = 0.6;
/// Quatro vezes a densidade dos corpos — o mesmo par das fixtures irmãs.
const FLUID_DENSITY: f32 = 4.0;

/// O cais sólido à esquerda, de onde se anda para dentro da água.
const DOCK_HALF: [f32; 2] = [4.0, 0.5];

const CAP_HALF_H: f32 = 0.3;
const CAP_RADIUS: f32 = 0.2;
const FLOAT: f32 = 0.9;
/// De onde os três são largados — acima da superfície, para a ENTRADA na água
/// fazer parte do que se vê.
const DROP_Y: f32 = 1.5;

/// **MEDIDO** (`probe_smoke_104`): quanto cada um AFUNDA desde a largada, em
/// quatro segundos — cápsula solta, player dinâmico, player cinemático.
///
/// ⚠️ **É a mesma grandeza dos `139,67 m` que o cinemático afundava antes da
/// cura**, e é por isso que a comparação é direta: os três descem ~1,1 m da
/// largada e **param logo acima da superfície**; o quebrado não parava.
///
/// ⚠️ **A primeira versão destes números veio da fixture IRMÃ** (a poça do
/// `measure_player_in_water`, que tem outra geometria) e o gate os reprovou —
/// que é exactamente para isso que ele existe. *Uma cena cuja mensagem cita
/// números tem de medir os DELA.*
pub(crate) const SANK: [f32; 3] = [1.1070, 1.0855, 1.0860];
/// A amplitude da oscilação entre o 3.º e o 6.º segundo — dinâmico e cinemático.
///
/// ⚠️ **Os dois números são o gate desta cena em prosa:** eles concordam na
/// quarta decimal, e é isso que *"a espécie do corpo não é uma pergunta que a
/// água faça"* quer dizer.
pub(crate) const BOB: [f32; 2] = [1.4357, 1.4394];

const DOCK_RGBA: [f32; 4] = [0.55, 0.50, 0.44, 1.0];
const WATER_RGBA: [f32; 4] = [0.20, 0.34, 0.46, 1.0];
const CONTROL_RGBA: [f32; 4] = [0.45, 0.78, 0.62, 1.0];
const DYNAMIC_RGBA: [f32; 4] = [0.90, 0.72, 0.32, 1.0];
const KINEMATIC_RGBA: [f32; 4] = [0.42, 0.66, 0.94, 1.0];

const CAMERA_CENTRE: [f32; 2] = [2.0, -1.0];
const CAMERA_HEIGHT: f32 = 12.0;

/// Um dos três sujeitos. `player` liga a lei; `kinematic` escolhe o modo.
///
/// ⚠️ **Os DOIS campos, como o chip da §14 os escreve** — o `PlayerMode` decide a
/// lei e quem escreve a pose, o `RigidBody.kind` decide o que o corpo é no
/// rapier. Escrever só um seria a cena a montar um estado que o gesto do artista
/// não produz.
fn subject(world: &mut World, name: &str, x: f32, tint: [f32; 4], player: bool, kinematic: bool) {
    let mut e = world.spawn((
        Name::new(name.to_string()),
        RigidBody {
            kind: if kinematic {
                BodyKind::Kinematic
            } else {
                BodyKind::Dynamic
            },
        },
        Collider {
            shape: ColliderShape::Capsule {
                half_height: CAP_HALF_H,
                radius: CAP_RADIUS,
            },
            density: 1.0,
            ..Collider::default()
        },
        LockRotation,
        Sprite::atlas(
            WHITE_TILE_KEY,
            [CAP_RADIUS * 2.0, (CAP_HALF_H + CAP_RADIUS) * 2.0],
            tint,
        ),
        Transform::from_translation(Vec2::new(x, DROP_Y)),
    ));
    if player {
        e.insert(PlatformPlayer {
            float_height: FLOAT,
            ..PlatformPlayer::default()
        });
    }
    if kinematic {
        e.insert(PlayerMode::Kinematic);
    }
}

pub(crate) fn build_kin_water_scene(world: &mut World) {
    // O CAIS — sólido, topo em y = 0, à esquerda da poça.
    world.spawn((
        Name::new("Dock"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: DOCK_HALF[0],
                half_y: DOCK_HALF[1],
            },
            density: 1.0,
            ..Collider::default()
        },
        Sprite::atlas(
            WHITE_TILE_KEY,
            [DOCK_HALF[0] * 2.0, DOCK_HALF[1] * 2.0],
            DOCK_RGBA,
        ),
        Transform::from_translation(Vec2::new(-10.0, -DOCK_HALF[1])),
    ));

    // A POÇA — um SENSOR, e é o par dele com um corpo CINEMÁTICO que não existia.
    world.spawn((
        Name::new("Pool"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            is_sensor: true,
            shape: ColliderShape::Cuboid {
                half_x: POOL_HALF[0],
                half_y: POOL_HALF[1],
            },
            density: 1.0,
            ..Collider::default()
        },
        AreaBuoyancy(FLUID_DENSITY),
        AreaDrag(POOL_DRAG),
        Sprite::atlas(
            WHITE_TILE_KEY,
            [POOL_HALF[0] * 2.0, POOL_HALF[1] * 2.0],
            WATER_RGBA,
        ),
        Transform::from_translation(Vec2::new(0.0, -POOL_HALF[1])),
    ));

    // Os TRÊS, da mesma altura, no mesmo instante — e a ordem na tela é a ordem
    // da história: o que a água sempre fez, o que já funcionava, o que não
    // existia.
    subject(world, "Control Capsule", -2.0, CONTROL_RGBA, false, false);
    subject(world, "Dynamic Player", 0.0, DYNAMIC_RGBA, true, false);
    subject(world, "Kinematic Player", 2.0, KINEMATIC_RGBA, true, true);
}

#[cfg(test)]
#[path = "physics_smoke_kin_water_tests.rs"]
mod tests;

impl crate::App {
    pub(crate) fn physics_smoke_kin_water(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        build_kin_water_scene(gfx.sim.world_mut());
        gfx.camera.center = CAMERA_CENTRE;
        gfx.camera.height_world = CAMERA_HEIGHT;
        if let Some(hero) = gfx.hero_screen.as_mut() {
            hero.panel_visibility.insert("physics", true);
        }

        eprintln!(
            "[physics-smoke 104] A AGUA E O MODO CINEMATICO (W-KinFluid).\n  \
               Uma poca FUNDA e tres capsulas IDENTICAS largadas da mesma altura:\n    \
                 VERDE  = capsula solta, sem lei de player nenhuma (o CONTROLE)\n    \
                 AMBAR  = player DINAMICO (o que ja' funcionava)\n    \
                 AZUL   = player CINEMATICO (o que nao existia para a agua)\n\n  \
               1. OS TRES TEM DE PARAR NA AGUA. Deixe correr. Largados de 1,5 m eles\n     \
                  afundam {c:.3} / {d:.3} / {k:.3} m e assentam LOGO ACIMA da\n     \
                  superficie.\n     \
                  O QUE ESTAVA QUEBRADO: o AZUL afundava 139,67 m -- queda livre com\n     \
                  o multiplicador de queda por cima -- e sumia de quadro. Se ele\n     \
                  atravessar o fundo da poca enquanto os outros boiam, PARE.\n\n  \
               2. A CAUSA NAO ERA A MASSA INFINITA. O `ActiveCollisionTypes` default\n     \
                  do rapier so' liga pares que comecam em DYNAMIC, entao uma poca\n     \
                  ESTATICA e um corpo CINEMATICO nunca existiam um para o outro no\n     \
                  grafo de intersecao. Nao era o impulso a nao alcancar massa\n     \
                  infinita: era o PAR que nao existia.\n\n  \
               3. ELES TEM DE BOBEAR JUNTOS. Isto NAO e' um repouso: o player\n     \
                  oscila ~{ba:.2} m (ambar) e ~{bk:.2} m (azul) entre o 3o e o 6o\n     \
                  segundo -- concordam na quarta decimal. A capsula VERDE oscila\n     \
                  menos (0,81), e essa diferenca e' da lei de player, anterior a\n     \
                  esta wave. O que se julga aqui e' o AMBAR e o AZUL fazerem A\n     \
                  MESMA COISA, nao uma altura.\n\n  \
               4. ANDE PARA DENTRO (A/D com o azul selecionado no Inspector, §14).\n     \
                  Ele tem de MOLHAR O PE' e ser freado pela agua nos dois eixos --\n     \
                  o meio resiste, e resiste igual na horizontal.\n\n  \
               (!) ABLACAO: no painel de fisica (tecla W nao; use o Inspector da\n      \
                   POCA) zere o Fluid Drag. A oscilacao sobe de {bk:.2} para 2,90 m\n      \
                   e NAO decai -- empuxo sem resistencia e' uma mola sem\n      \
                   amortecimento, e o arrasto e' a metade que fecha a lei.\n\n  \
               (!) A FORCA da zona (uma correnteza) NAO estava nesta wave, e a\n      \
                   W-ZoneForce a fechou: ela chega hoje aos tres modos, pela MESMA\n      \
                   porta que o solver usa (nao uma re-derivacao). Cena =106.\n\n  \
               (!) Toque B para o contorno: a poca fica magenta (sensor) e acende\n      \
                   quando ha' corpo dentro.\n",
            c = SANK[0],
            d = SANK[1],
            k = SANK[2],
            ba = BOB[0],
            bk = BOB[1],
        );
    }
}
