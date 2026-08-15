//! **A CENA 118 — O ELEVADOR** (`W-Leave`): o que a plataforma dá ao pulo
//! quando se larga ela.
//!
//! A pergunta que esta cena põe na tela é a que a medição achou e que nenhum
//! smoke anterior fazia: *o artista autora dois metros de pulo — dois metros
//! contra o quê?*
//!
//! ⚠️ **As três raias são a MESMA cena com a MESMA autoria**, e só a política
//! difere. É isso que torna o contraste legível: o que muda entre elas não é o
//! elevador, não é o personagem e não é a altura — é uma escolha.
//!
//! # ⚠️ O elevador é DINÂMICO, e a nota já existia no repo
//!
//! Um corpo cinemático é dirigido por **uma pose por tique** (o `SceneAtTick` da
//! timeline), e uma cena de smoke não tem timeline — é a mesma razão pela qual o
//! vagão do gate da W10 (`platform_lift.rs`) é dinâmico com massa enorme. Aqui
//! ele desce por **velocidade terminal**: com o arrasto em [`LIFT_DRAG`] o
//! regime é `v = g / d`, que é física do próprio motor em vez de um empurrão
//! escrito à mão.
//!
//! ⚠️ **A massa enorme não é folclore:** o personagem em cima carrega o
//! elevador pela 3.ª lei (W6), e sem ela a plataforma afundaria conforme quem
//! está nela — a cena mediria o peso do passageiro em vez da política.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, Transform, World};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, DampMode, DampingOverride, LockPositionX, LockRotation,
    MassOverride, PlatformLift, PlatformPlayer, RigidBody,
};

/// **O arrasto do elevador** — escolhido para a velocidade terminal ser
/// [`LIFT_SPEED`]: em regime `v = g / d`, e `9,81 / 2,4525 = 4,00 m/s`.
///
/// ⚠️ **Ele é `Replace`, nunca `Combine`:** o arrasto de MUNDO é autorável no
/// painel de física, e sob `Combine` um artista que o mexesse mudaria a
/// velocidade desta cena sem saber que a tinha mexido.
pub(crate) const LIFT_DRAG: f32 = 2.4525;

/// A velocidade que o elevador atinge — **medida** pelo gate
/// (`the_lift_descends_at_the_speed_the_scene_claims`), não assumida.
pub(crate) const LIFT_SPEED: f32 = 4.0;

/// A altura autorada, metros — a promessa que as três raias cobram.
pub(crate) const JUMP_HEIGHT: f32 = 2.0;

/// A que distância uma raia fica da outra. ⚠️ Larga o bastante para o
/// personagem de uma raia nunca pousar no elevador da vizinha, que é o modo
/// óbvio de uma cena de contraste medir a coisa errada.
const LANE_SPAN: f32 = 14.0;

/// Meia-largura de cada elevador, e do chão de cada raia.
const LIFT_HALF: f32 = 3.0;
const FLOOR_HALF: f32 = 5.0;

/// Onde cada elevador começa, e onde o chão da raia fica.
///
/// ⚠️ A queda útil é `START_Y − FLOOR_Y` = 12 m, ou **três segundos** à
/// velocidade terminal: tempo de sobra para o artista pular enquanto desce, e
/// depois um chão parado para o CONTROLE do passo 3.
const START_Y: f32 = 14.0;
const FLOOR_Y: f32 = 2.0;

/// A perna do personagem (o mesmo valor das outras cenas de player).
const FLOAT: f32 = 0.9;

/// As três raias, da esquerda para a direita — **e a primeira é o defeito**.
pub(crate) const LANES: [(&str, PlatformLift); 3] = [
    ("Full", PlatformLift::Full),
    ("UpOnly", PlatformLift::UpOnly),
    ("Nothing", PlatformLift::Nothing),
];

pub(crate) fn lane_x(i: usize) -> f32 {
    (i as f32 - 1.0) * LANE_SPAN
}

/// **A geometria da cena 118**, separada do `App` de propósito — é ela que os
/// gates dirigem, e não uma reconstrução deles.
pub(crate) fn build_leave_scene(world: &mut World) -> Vec<Entity> {
    let mut riders = Vec::new();
    for (i, (tag, lift)) in LANES.iter().enumerate() {
        let x = lane_x(i);
        // O chão da raia — o CONTROLE do passo 3, e o fundo do elevador.
        world.spawn((
            Name::new(format!("Floor {tag}")),
            RigidBody {
                kind: BodyKind::Static,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: FLOOR_HALF,
                    half_y: 0.5,
                },
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(x, FLOOR_Y)),
        ));
        world.spawn((
            Name::new(format!("Lift {tag}")),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: LIFT_HALF,
                    half_y: 0.25,
                },
                ..Collider::default()
            },
            LockRotation,
            // ⚠️ Sem isto um passageiro que ande de lado empurraria o elevador
            // pela 3.ª lei, e as três raias deixariam de partir do mesmo lugar.
            LockPositionX,
            MassOverride(1000.0),
            DampingOverride {
                linear: LIFT_DRAG,
                angular: 0.0,
                mode: DampMode::Replace,
            },
            Transform::from_translation(Vec2::new(x, START_Y)),
        ));
        let rider = world
            .spawn((
                Name::new(format!("Rider {tag}")),
                RigidBody {
                    kind: BodyKind::Dynamic,
                },
                Collider {
                    shape: ColliderShape::Capsule {
                        half_height: 0.3,
                        radius: 0.2,
                    },
                    ..Collider::default()
                },
                LockRotation,
                PlatformPlayer {
                    float_height: FLOAT,
                    jump_height: JUMP_HEIGHT,
                    platform_lift: *lift,
                    ..PlatformPlayer::default()
                },
                Transform::from_translation(Vec2::new(x, START_Y + 0.25 + FLOAT)),
            ))
            .id();
        riders.push(rider);
    }
    riders
}

/// ⚠️ **A cena IMPRIME o que montou** — se esta mensagem não aparecer, o resto
/// do smoke não diz nada.
pub(crate) const LEAVE_SMOKE_MESSAGE: &str = concat!(
    "[physics-smoke 118] O ELEVADOR (W-Leave) -- o que a plataforma da' ao pulo.\n",
    "Tres raias IDENTICAS: mesmo personagem, altura autorada 2.00 m, elevador a\n",
    "DESCER a 4.00 m/s. So' a politica difere.\n",
    "  ESQUERDA  Full     a altura e' medida contra a PLATAFORMA (o que shipava)\n",
    "  MEIO      Up Only  a descida deixa de roubar o pulo\n",
    "  DIREITA   Nothing  a altura e' sempre medida contra o MUNDO\n",
    "\n",
    "⚠️ Se a linha acima nao aparecer, pare: a cena nao montou.\n",
    "\n",
    "1. Deixe os tres descerem e pule com cada um (WASD + espaco, um de cada vez).\n",
    "   ⚠️ O da ESQUERDA quase nao sai do elevador -- e' o defeito que esta wave\n",
    "   nomeia, e ele e' o comportamento que ja' shipava. Os outros dois sobem a\n",
    "   altura que voce autorou. Medido: pico 0.38 m contra 1.90.\n",
    "2. Selecione um deles: a row 'Platform Lift' fica na secao Platform Player,\n",
    "   logo abaixo dos cards. Troque a politica e pule de novo -- a mudanca vale\n",
    "   para o PROXIMO pulo, sem re-simular nada.\n",
    "3. CONTROLE: espere os tres pousarem no chao (12 m, ~3 s) e pule de novo.\n",
    "   Com o chao PARADO as tres politicas sao a MESMA coisa, ao bit. Se uma\n",
    "   delas saltar diferente ali, PARE -- a politica esta a alcancar chao que\n",
    "   nao se move.\n",
);

impl crate::App {
    /// **A cena 118** — o roteiro está em [`LEAVE_SMOKE_MESSAGE`].
    pub(crate) fn physics_smoke_leave(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        let _ = build_leave_scene(gfx.sim.world_mut());
        eprintln!("{LEAVE_SMOKE_MESSAGE}");
    }
}

#[cfg(test)]
#[path = "physics_smoke_leave_tests.rs"]
mod tests;
