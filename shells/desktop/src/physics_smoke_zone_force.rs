//! **A CORRENTEZA LEVA OS TRÊS MODOS** (`PH2D_PHYSICS_SMOKE=106`) — a `W-ZoneForce`.
//!
//! O item do Enio, e o último buraco da família das zonas: a FORÇA de uma zona movia o
//! modo Dynamic e **não movia** o Snap/Push/Pure. As duas metades estavam corretas
//! sozinhas — o `effector::apply` recusa corpo não-dinâmico antes de o tocar (um corpo
//! cinemático tem massa infinita e o solver ignoraria o impulso) e a lei cinemática
//! integrava um `Fluid` sem força nenhuma — e a soma delas era um personagem que a
//! correnteza empurra num modo e não empurra nos outros dois. Medido antes da cura:
//! **`0,0000 m` em qualquer força**, contra os 21,83 m de um caixote solto.
//!
//! # ⚠️ O oráculo é o MODO DINÂMICO, e é por isso que a cena tem quatro corpos
//!
//! Não há altura nem distância "certa" a citar: o que a correnteza consegue mover contra
//! um personagem depende do FREIO da caminhada dele, que é um knob do artista. Então a
//! cena põe os quatro lado a lado, na MESMA correnteza, com o MESMO freio:
//!
//! - **verde** — o caixote solto, sem lei de player nenhuma: o que a zona faz a um corpo
//!   qualquer, e o teto de quanto ela pode fazer;
//! - **âmbar** — o player DINÂMICO, o que já funcionava;
//! - **azul** — o player CINEMÁTICO (Snap/Push);
//! - **roxo** — o player PURO, em que o mundo físico é cenário.
//!
//! *Se o azul e o roxo ficarem parados enquanto o âmbar viaja, pare.*
//!
//! ⚠️ **E o que se julga é a CONCORDÂNCIA, não uma marca no chão** — os três players têm
//! de andar aproximadamente o mesmo, e todos MENOS que o caixote (a caminhada resiste à
//! correnteza, que é o certo: pode-se andar contra a corrente).
//!
//! ⚠️ **Sem gravidade, e a poça não é uma poça.** A zona aqui não tem empuxo nenhum: é
//! uma correnteza pura, para *onde eles pararam* ser *o que ela fez*. Com gravidade os
//! quatro cairiam e o chão passaria a ser a segunda coisa a agi-los.
//!
//! Os números da mensagem saem da sonda `probe_smoke_106`.

use ph2d_core::Vec2;
use ph2d_ecs::{Name, Transform, World};
use ph2d_physics_ecs::{
    AreaEffector, BodyKind, Collider, ColliderShape, LockRotation, PlatformPlayer, PlayerMode,
    RigidBody,
};
use ph2d_render::{Sprite, WHITE_TILE_KEY};

/// A correnteza — larga o bastante para ninguém sair dela na janela medida.
///
/// ⚠️ **O número é do CAIXOTE, e ele é MEDIDO:** sem lei de player nenhuma a resistir,
/// ele acelera livremente e cobre **87,3 m** em dois segundos. A primeira versão desta
/// cena tinha meia-extensão 40 e o gate `nobody_leaves_the_current_during_the_run` a
/// reprovou — *"andou menos"* teria passado a significar *"saiu"* justamente no corpo que
/// serve de teto à comparação, e o número publicado (78,4) era o de um caixote que já
/// tinha saído.
const ZONE_HALF: [f32; 2] = [120.0, 8.0];
/// ⚠️ **MEDIDO, não escolhido:** com o freio de fábrica (60) a caminhada satura contra
/// qualquer correnteza razoável e os quatro mal se movem — a cena mostraria a mesma coisa
/// antes e depois da wave. A `16 N` a correnteza VENCE o freio e a pergunta fica
/// observável (varrido: 1 · 4 · 16 · 64 · 256 N).
const FORCE: f32 = 16.0;

const CAP_HALF_H: f32 = 0.3;
const CAP_RADIUS: f32 = 0.2;

/// A separação vertical entre as raias — cada sujeito na sua, para os rastros não se
/// cruzarem e a comparação ser de OLHO.
const LANE: f32 = 1.6;

/// **MEDIDO** (`probe_smoke_106`): quanto cada um anda em 2 s — caixote, dinâmico,
/// cinemático, puro.
///
/// ⚠️ **É a mesma grandeza do `0,0000` que os dois últimos andavam antes da cura**, e é
/// por isso que a comparação é direta.
pub(crate) const CARRIED: [f32; 4] = [87.330, 20.590, 20.999, 20.999];

const CRATE_RGBA: [f32; 4] = [0.45, 0.78, 0.62, 1.0];
const DYNAMIC_RGBA: [f32; 4] = [0.90, 0.72, 0.32, 1.0];
const KINEMATIC_RGBA: [f32; 4] = [0.42, 0.66, 0.94, 1.0];
const PURE_RGBA: [f32; 4] = [0.72, 0.52, 0.90, 1.0];
const ZONE_RGBA: [f32; 4] = [0.18, 0.30, 0.36, 1.0];

const CAMERA_CENTRE: [f32; 2] = [10.0, 0.0];
const CAMERA_HEIGHT: f32 = 16.0;

/// Um dos quatro. `mode` é `None` para o caixote solto.
///
/// ⚠️ **Os DOIS campos, como o chip da §14 os escreve** — o `PlayerMode` decide a lei e
/// quem escreve a pose, o `RigidBody.kind` decide o que o corpo é no rapier.
fn subject(world: &mut World, name: &str, lane: f32, tint: [f32; 4], mode: Option<PlayerMode>) {
    let mut e = world.spawn((
        Name::new(name.to_string()),
        RigidBody {
            kind: if mode.is_some_and(PlayerMode::drives_itself) {
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
        Transform::from_translation(Vec2::new(0.0, lane)),
    ));
    if let Some(m) = mode {
        // ⚠️ **O pincel de FÁBRICA, e é decisão da cena.** O freio da caminhada é o que
        // decide quanto a correnteza consegue mover um personagem, e afrouxá-lo mostraria
        // um personagem que ninguém autora: medido, com o freio em `1` os três andam
        // ~83,6 m e o caixote 87,3 — o contraste *"a caminhada resiste"* desaparece. No
        // default eles andam ~21 e o caixote 87, que é a leitura que a cena quer.
        e.insert(PlatformPlayer::default());
        e.insert(m);
    }
}

pub(crate) fn build_zone_force_scene(world: &mut World) {
    // A CORRENTEZA — um SENSOR carregando uma força em `+X`. É o par dela com um corpo
    // CINEMÁTICO que não fazia nada.
    world.spawn((
        Name::new("Current"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            is_sensor: true,
            shape: ColliderShape::Cuboid {
                half_x: ZONE_HALF[0],
                half_y: ZONE_HALF[1],
            },
            density: 1.0,
            ..Collider::default()
        },
        AreaEffector {
            force: [FORCE, 0.0],
        },
        Sprite::atlas(
            WHITE_TILE_KEY,
            [ZONE_HALF[0] * 2.0, ZONE_HALF[1] * 2.0],
            ZONE_RGBA,
        ),
        Transform::from_translation(Vec2::new(0.0, 0.0)),
    ));

    // Os QUATRO, cada um na sua raia, todos na origem em `x` — e a ordem na tela é a
    // ordem da história: o que a zona sempre fez, o que já funcionava, os dois que não
    // recebiam nada.
    subject(world, "Loose Crate", 1.5 * LANE, CRATE_RGBA, None);
    subject(
        world,
        "Dynamic Player",
        0.5 * LANE,
        DYNAMIC_RGBA,
        Some(PlayerMode::Dynamic),
    );
    subject(
        world,
        "Kinematic Player",
        -0.5 * LANE,
        KINEMATIC_RGBA,
        Some(PlayerMode::Kinematic),
    );
    subject(
        world,
        "Pure Player",
        -1.5 * LANE,
        PURE_RGBA,
        Some(PlayerMode::Pure),
    );
}

#[cfg(test)]
#[path = "physics_smoke_zone_force_tests.rs"]
mod tests;

impl crate::App {
    pub(crate) fn physics_smoke_zone_force(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        // ⚠️ **Sem gravidade, e é a premissa da cena** — a correnteza passa a ser a
        // única coisa a agir, então *onde eles pararam* É *o que ela fez*. Escrito
        // pela porta que o painel de física do W2b usa, não num campo privado.
        gfx.physics.set_settings(ph2d_physics_ecs::PhysicsSettings {
            gravity_y: 0.0,
            ..Default::default()
        });
        build_zone_force_scene(gfx.sim.world_mut());
        gfx.camera.center = CAMERA_CENTRE;
        gfx.camera.height_world = CAMERA_HEIGHT;
        if let Some(hero) = gfx.hero_screen.as_mut() {
            hero.panel_visibility.insert("physics", true);
        }

        eprintln!(
            "[physics-smoke 106] A CORRENTEZA LEVA OS TRES MODOS (W-ZoneForce).\n  \
               Uma correnteza de {f:.0} N, SEM gravidade, e quatro capsulas identicas\n  \
               em raias proprias -- todas com o MESMO freio de caminhada:\n    \
                 VERDE = caixote solto, sem lei de player (o TETO do que a zona faz)\n    \
                 AMBAR = player DINAMICO (o que ja' funcionava)\n    \
                 AZUL  = player CINEMATICO (Snap/Push)\n    \
                 ROXO  = player PURO (o mundo fisico como cenario)\n\n  \
               1. OS TRES PLAYERS TEM DE VIAJAR JUNTOS. Deixe correr 2 s: eles andam\n     \
                  {d:.1} / {k:.1} / {p:.1} m e o caixote {c:.1}.\n     \
                  O QUE ESTAVA QUEBRADO: o AZUL e o ROXO andavam 0,0 m -- em QUALQUER\n     \
                  forca. Se eles ficarem parados enquanto o ambar viaja, PARE.\n\n  \
               2. ELES ANDAM MENOS QUE O CAIXOTE, e isso e' o certo: a caminhada\n     \
                  resiste a' correnteza (pode-se andar contra a corrente) -- o VERDE\n     \
                  some de quadro em ~1 s, e e' assim que se ve' o teto. O que se julga\n     \
                  aqui e' a CONCORDANCIA entre os tres players, nao a distancia.\n\n  \
               3. ANDE CONTRA (A com um deles selecionado no Inspector, §14). Ele tem\n     \
                  de conseguir progredir contra a correnteza nos tres modos -- e ceder\n     \
                  quando voce solta.\n\n  \
               4. GIRE A ZONA (selecione 'Current' e mude a rotacao no Inspector). O\n     \
                  sopro gira com ela e leva os tres para o novo lado -- o frame\n     \
                  (W-AreaFrame) chega ao cinematico pela MESMA porta do solver, sem\n     \
                  uma segunda derivacao.\n\n  \
               (!) ABLACAO: no Inspector da zona ponha Falloff em 1. Quem esta' na\n      \
                   margem da correnteza passa a andar bem menos que quem esta' no eixo\n      \
                   -- nos tres modos.\n\n  \
               (!) Toque B para o contorno: a zona fica magenta (sensor), com a SETA\n      \
                   laranja a dizer para que lado ela sopra.\n",
            f = FORCE,
            c = CARRIED[0],
            d = CARRIED[1],
            k = CARRIED[2],
            p = CARRIED[3],
        );
    }
}
