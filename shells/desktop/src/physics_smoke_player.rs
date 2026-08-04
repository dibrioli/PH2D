//! **As cenas do PLAYER DE PLATAFORMA** — `=80` (a mola, W2) e `=81` (andar, W3).
//!
//! ⚠️ **A cena do CAST (`=80` no plano) foi CORTADA, e o corte é a decisão.**
//! O plano prometia *"três raios contra chão plano / rampa / vão, imprimindo
//! distância e normal"* — e isso é um TESTE, não um smoke: o `world::cast` já
//! tem sete gates, com os dois lados do BVH medidos, e uma cena que imprime
//! números pede ao artista para conferir exatamente o que uma máquina confere
//! melhor. Uma cena de smoke existe para julgar o que **só o olho julga**.
//! As duas cenas daqui sobem um número cada, e o plano foi corrigido junto.
//!
//! ⚠️ **A `float_height` destas cenas é 0,9 e não o `0,5` do ponto de partida**,
//! e o motivo é geometria medida, não gosto: flutuar de verdade exige
//! `float_height > half_height + radius / cos(max_slope)`
//! ([`ph2d_platformer::RideConfig::min_float_height`], com a tabela). Para a
//! cápsula daqui (`0,3` / `0,2`) o mínimo já é `0,5` **no plano** — ou seja, o
//! ponto de partida a deixa TANGENTE, e ela deixa de pairar na primeira rampa.

use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, LockRotation, PlatformPlayer, RigidBody,
};
use ph2d_render::{Sprite, WHITE_TILE_KEY};
use ph2d_timeline::{PropKind, TimelineDoc};

/// A altura de flutuação destas cenas — ver o aviso do módulo.
const FLOAT: f32 = 0.9;

/// O personagem: cápsula dinâmica, rotação travada (D4), com a config do plano.
fn spawn_player(world: &mut bevy_ecs::world::World, at: Vec2) -> Entity {
    world
        .spawn((
            Name::new("Player"),
            Transform::from_translation(at),
            Sprite::atlas(WHITE_TILE_KEY, [0.4, 1.0], [0.25, 0.85, 1.0, 1.0]),
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
                ..PlatformPlayer::default()
            },
        ))
        .id()
}

/// Um bloco estático, opcionalmente inclinado.
fn slab(
    world: &mut bevy_ecs::world::World,
    name: &str,
    at: Vec2,
    half: [f32; 2],
    rot: f32,
    tint: [f32; 4],
) {
    world.spawn((
        Name::new(name.to_string()),
        Transform {
            rotation: rot,
            ..Transform::from_translation(at)
        },
        Sprite::atlas(WHITE_TILE_KEY, [half[0] * 2.0, half[1] * 2.0], tint),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: half[0],
                half_y: half[1],
            },
            ..Collider::default()
        },
    ));
}

/// A plataforma vai e volta.
///
/// ⚠️ **Dentro de 4 s de propósito** — é a duração com que toda composição
/// nasce, e uma track mais longa ficaria congelada no corte sem que nada na tela
/// dissesse por quê. Para o vagão repetir, arme o Loop na barra de transporte.
pub(crate) fn author_platform_track(doc: &mut TimelineDoc, platform: Entity) {
    for (t, x) in [(0.0, 6.0_f32), (2.0, 11.0), (4.0, 6.0)] {
        doc.insert_key(
            platform.to_bits(),
            PropKind::TranslationX,
            RationalTime::from_seconds(t),
            AnimValue::Float(x),
            Interp::Linear,
        );
    }
}

impl crate::App {
    /// **Cena 80 (W2).** O personagem PAIRA.
    ///
    /// A pergunta é de olho e é uma só: *ele fica no ar, na mesma altura, sem
    /// tremer?* Depois, empurre-o com a **MÃO** (a ferramenta de interação) e
    /// solte: ele tem de voltar à altura, sem pipocar.
    pub(crate) fn physics_smoke_float(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        let world = gfx.sim.world_mut();
        slab(
            world,
            "Floor",
            Vec2::new(0.0, -0.5),
            [14.0, 0.5],
            0.0,
            [0.35, 0.35, 0.4, 1.0],
        );
        // Um degrau: subir nele é o caso que a `cling_distance` governa.
        slab(
            world,
            "Step",
            Vec2::new(4.0, 0.15),
            [1.2, 0.65],
            0.0,
            [0.45, 0.4, 0.35, 1.0],
        );
        spawn_player(world, Vec2::new(-3.0, 2.5));

        eprintln!(
            "[physics-smoke 80] O PLAYER PAIRA (W2). Um chao, um degrau e um personagem.\n\
             \n\
             ⚠️ Se a linha acima nao aparecer, pare: a cena nao montou.\n\
             \n\
             Julgue, de olho:\n\
             · ele CAI e para NO AR, {FLOAT:.2} m acima do chao -- nao encostado.\n\
             · em repouso ele NAO treme nem sobe e desce (a mola nao pode oscilar).\n\
             · pegue-o com a MAO (a ferramenta de interacao) e puxe para cima: ao\n\
               soltar ele volta a MESMA altura, sem pipocar.\n\
             · empurre-o para o degrau: a perna o levanta sem que ele bata nele."
        );
    }

    /// **Cena 81 (W3).** ANDAR — a cena que se dirige.
    ///
    /// ⚠️ É a primeira cena desta jornada com **controle**: as setas ←/→ (ou
    /// A/D) andam. Sem isso o W3 seria invisível — a caminhada é uma resposta a
    /// um dedo, e nada num log a mostra.
    pub(crate) fn physics_smoke_walk(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        let world = gfx.sim.world_mut();

        // Um chão longo, uma rampa RASA (30°, sobe) e uma ÍNGREME (60°, escorrega).
        slab(
            world,
            "Floor",
            Vec2::new(0.0, -0.5),
            [10.0, 0.5],
            0.0,
            [0.35, 0.35, 0.4, 1.0],
        );
        let ramp30 = 30.0_f32.to_radians();
        slab(
            world,
            "Ramp30",
            Vec2::new(-13.0, 1.2),
            [4.5, 0.5],
            ramp30,
            [0.3, 0.5, 0.35, 1.0],
        );
        let ramp60 = 60.0_f32.to_radians();
        slab(
            world,
            "Ramp60",
            Vec2::new(13.0, 2.4),
            [3.5, 0.5],
            -ramp60,
            [0.55, 0.32, 0.3, 1.0],
        );

        // A plataforma KINEMATIC, dirigida pela timeline — o vagão.
        let platform = world
            .spawn((
                Name::new("Wagon"),
                Transform::from_translation(Vec2::new(6.0, 1.6)),
                Sprite::atlas(WHITE_TILE_KEY, [4.0, 0.5], [0.6, 0.55, 0.25, 1.0]),
                RigidBody {
                    kind: BodyKind::Kinematic,
                },
                Collider {
                    shape: ColliderShape::Cuboid {
                        half_x: 2.0,
                        half_y: 0.25,
                    },
                    ..Collider::default()
                },
            ))
            .id();

        spawn_player(world, Vec2::new(0.0, 2.0));
        author_platform_track(&mut self.timeline.doc, platform);

        eprintln!(
            "[physics-smoke 81] ANDAR (W3). Chao + rampa de 30deg + rampa de 60deg + um vagao.\n\
             \n\
             ⚠️ Se a linha acima nao aparecer, pare: a cena nao montou.\n\
             \n\
             CONTROLE: as setas <- / -> (ou A / D) andam. O limite de rampa autorado e' 45deg.\n\
             \n\
             Julgue, de olho:\n\
             · ande: ele acelera ate' uma velocidade CONSTANTE e nao passa dela.\n\
             · SOLTE a tecla: ele freia e PARA -- e depois nao escorrega mais nada.\n\
             · segure a tecla oposta em movimento: virar responde mais rapido que\n\
               arrancar do zero (e' o fator de mudanca de direcao, ate' 2x).\n\
             · va' para a ESQUERDA, ate' a rampa de 30deg: ele SOBE, e sobe com a\n\
               mesma velocidade com que andava no plano (a velocidade e' a do\n\
               PERCURSO, nao a horizontal).\n\
             · va' para a DIREITA, ate' a rampa de 60deg: ele NAO sobe -- escorrega.\n\
               Suba 'Max Slope' acima de 60 e ela passa a ser escalavel.\n\
             · suba no VAGAO e SOLTE a tecla: ele viaja junto, parado em relacao\n\
               ao vagao. Ande em cima dele: anda normal, sobre um chao que se move.\n\
               (O vagao faz uma ida-e-volta nos primeiros 4 s; arme o Loop na barra\n\
                de transporte para ele repetir.)"
        );
    }
}
