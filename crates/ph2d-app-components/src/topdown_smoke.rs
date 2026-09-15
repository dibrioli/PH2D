//! ⭐⭐⭐ **Smoke do MOVER DE VISTA DE CIMA** (TOP-20 #13, W2). `PH2D_TOPDOWN_SMOKE=1|2`.
//!
//! # `=1` — **o corredor, e a parede que não te trava**
//!
//! Um recinto de paredes em **L**. O boneco anda com as **setas**, em 8 direcções.
//!
//! **O que tem de acontecer:** encostado a uma parede e a empurrar na diagonal (duas setas ao mesmo
//! tempo), ele **desliza ao longo dela à velocidade de sempre** — não trava, e não abranda.
//!
//! ⚠️⚠️ **É esta a coisa que a wave traz, e ela é medível a olho:** o que a casa fazia até aqui era
//! a **projecção** — o corpo rastejava a `70 %` da velocidade numa diagonal a 45°, e a `34 %` num
//! ângulo mais fechado. *Se ele abrandar ao encostar, a lei do orçamento não está a correr.*
//!
//! ⚠️ **De cabeça contra a parede ele PÁRA**, e isso é o desenho: só um toque de raspão desliza.
//!
//! # `=2` — **a isometria: a mesma seta, noutro tabuleiro**
//!
//! O chão desenha um **losango** (a grelha 2:1), e o mesmo componente está em *Isometric 2:1* com
//! *Face Movement*.
//!
//! **O que tem de acontecer:** a seta `→` manda o boneco para **nordeste**, ao longo da linha do
//! tabuleiro — não para a direita do ecrã. E ele **vira-se** para onde anda.
//!
//! ⚠️ **O controlo está na própria cena:** o quadrado cinzento ao lado tem o MESMO componente em
//! *Top-Down*. Carregue na mesma seta e compare — se os dois forem para o mesmo sítio, o menu de
//! viewpoint não está a chegar ao movimento.
//!
//! ⭐⭐⭐ **E é aqui que a ÚLTIMA SETA MANDA se vê** (ordem do dono, 2026-09-15): os dois estão em
//! **4 direcções**. Segure a `→` e, sem a largar, carregue na `↑` — o quadrado **cinzento** tem de
//! deixar de ir para a direita e passar a subir. Largue a `↑` e ele volta à direita.
//!
//! ⚠️ **O cinzento é o sujeito deste passo, e não o amarelo**: o amarelo está em isometria, logo o
//! «para cima» dele é a diagonal do tabuleiro, e a troca de eixo lê-se pior.
//!
//! ⛔ **Na cena `=1` esta regra NÃO vale, e isso é o desenho:** ali o modo é *8 direcções*, onde a
//! diagonal é uma resposta legítima — apagá-la seria tirar metade dos rumos ao modo cuja razão de
//! existir são eles.
//!
//! ⚠️ Se a linha `[topdown-smoke]` não aparecer, **PARE**: a cena não montou.

use ph2d_core::Vec2;
use ph2d_ecs::{Name, Transform, World};
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, RigidBody, TopDownPlayer};
use ph2d_render::{Sprite, WHITE_TILE_KEY};
use ph2d_topdown::{
    TopDownLaw, direction::DirectionMode, rotation::RotationMode, viewpoint::Viewpoint,
};

/// ⭐⭐ **Quantas cenas este roteador serve** — o `max_level` que o [`crate::FAMILY`] declara.
///
/// ⚠️ **CONTADO do `match` do [`montar`]**, nunca escrito de memória (`CLAUDE.md` §5.0).
pub const CENAS: u32 = 2;

const PAREDE_RGBA: [f32; 4] = [0.38, 0.40, 0.46, 1.0];
const CHAO_RGBA: [f32; 4] = [0.16, 0.18, 0.22, 1.0];
const HEROI_RGBA: [f32; 4] = [0.95, 0.72, 0.25, 1.0];
const CONTROLO_RGBA: [f32; 4] = [0.55, 0.57, 0.60, 1.0];
const GRELHA_RGBA: [f32; 4] = [0.30, 0.34, 0.42, 1.0];

/// A velocidade do boneco, m/s — ⚠️ escolhida para o ecrã: a `4 m/s` ele atravessa o recinto em
/// ~3 s, que é tempo de ver o deslize sem perseguir a câmara.
const VELOCIDADE: f32 = 4.0;

fn parede(world: &mut World, nome: &str, em: Vec2, meio: Vec2) {
    world.spawn((
        Name::new(nome),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: meio.x,
                half_y: meio.y,
            },
            ..Collider::default()
        },
        Sprite::atlas(WHITE_TILE_KEY, [meio.x * 2.0, meio.y * 2.0], PAREDE_RGBA),
        Transform::from_translation(em),
    ));
}

/// O boneco: um corpo **cinemático** com o componente da wave.
///
/// ⚠️ **Cinemático e não dinâmico**, e é o desenho do componente: o mover escreve a própria pose
/// (ver [`ph2d_physics_ecs`] `bridge::topdown`). Um corpo dinâmico seria do solver, e o
/// componente ficaria sem dono do `Transform` para escrever.
fn heroi(world: &mut World, nome: &str, em: Vec2, cor: [f32; 4], law: TopDownLaw) {
    world.spawn((
        Name::new(nome),
        RigidBody {
            kind: BodyKind::Kinematic,
        },
        Collider {
            shape: ColliderShape::Ball { radius: 0.35 },
            ..Collider::default()
        },
        Sprite::atlas(WHITE_TILE_KEY, [0.7, 0.7], cor),
        TopDownPlayer::from_law(law),
        Transform::from_translation(em),
    ));
}

/// A cena `=1` — o corredor em L.
fn cena_um(world: &mut World) {
    world.spawn((
        Name::new("Floor"),
        Sprite::atlas(WHITE_TILE_KEY, [16.0, 12.0], CHAO_RGBA),
        Transform::from_translation(Vec2::new(0.0, 0.0)),
    ));
    // O recinto: quatro paredes, e um dente no meio que faz o L.
    parede(world, "Wall N", Vec2::new(0.0, 5.5), Vec2::new(8.0, 0.5));
    parede(world, "Wall S", Vec2::new(0.0, -5.5), Vec2::new(8.0, 0.5));
    parede(world, "Wall W", Vec2::new(-7.5, 0.0), Vec2::new(0.5, 6.0));
    parede(world, "Wall E", Vec2::new(7.5, 0.0), Vec2::new(0.5, 6.0));
    // ⭐ **O dente**: é contra ele que o artista encosta na diagonal. Ele fica no MEIO do caminho
    // de propósito — uma parede só na borda faz o teste acontecer fora do olhar.
    parede(
        world,
        "Wall Corner",
        Vec2::new(1.5, 1.0),
        Vec2::new(4.0, 0.5),
    );

    heroi(
        world,
        "Hero",
        Vec2::new(-4.0, -3.0),
        HEROI_RGBA,
        TopDownLaw {
            speed: VELOCIDADE,
            direction: DirectionMode::EightWay,
            ..TopDownLaw::default()
        },
    );
}

/// A cena `=2` — a isometria, com o controlo ao lado.
fn cena_dois(world: &mut World) {
    world.spawn((
        Name::new("Floor"),
        Sprite::atlas(WHITE_TILE_KEY, [20.0, 12.0], CHAO_RGBA),
        Transform::from_translation(Vec2::new(0.0, 0.0)),
    ));
    // ⭐⭐ **A GRELHA DO TABULEIRO, desenhada** — sem ela a isometria é uma afirmação sobre números
    // que ninguém vê. Duas famílias de linhas, nas duas arestas do losango `2:1`
    // (elevação `arctan 0,5 = 26,565°`), que é exactamente a base que o `Viewpoint` usa.
    let elev = 26.565_052_f32.to_radians();
    for k in -6..=6 {
        let d = k as f32 * 1.2;
        for (nome, ang) in [("Grid A", elev), ("Grid B", -elev)] {
            let mut t = Transform::from_translation(Vec2::new(-4.5, d));
            t.rotation = ang;
            world.spawn((
                Name::new(format!("{nome} {k}")),
                Sprite::atlas(WHITE_TILE_KEY, [14.0, 0.04], GRELHA_RGBA),
                t,
            ));
        }
    }

    heroi(
        world,
        "Hero (Isometric)",
        Vec2::new(-4.5, 0.0),
        HEROI_RGBA,
        TopDownLaw {
            speed: VELOCIDADE,
            direction: DirectionMode::FourWay,
            viewpoint: Viewpoint::Isometric2to1,
            rotation: RotationMode::ToMovement,
            rotation_speed_deg: 540.0,
            ..TopDownLaw::default()
        },
    );
    // ⭐⭐ **O CONTROLO, na mesma cena**: o MESMO componente sem o viewpoint. Sem ele, o artista não
    // tem como saber se o menu chegou ao movimento ou se o boneco anda assim de qualquer maneira.
    heroi(
        world,
        "Control (Top-Down)",
        Vec2::new(-4.5, -4.0),
        CONTROLO_RGBA,
        TopDownLaw {
            speed: VELOCIDADE,
            direction: DirectionMode::FourWay,
            viewpoint: Viewpoint::TopDown,
            ..TopDownLaw::default()
        },
    );
}

/// **Monta a cena `nivel`** e devolve qual foi — o roteador.
pub fn montar(world: &mut World, nivel: u32) -> u32 {
    match nivel {
        2 => {
            cena_dois(world);
            println!(
                "[topdown-smoke] =2 a isometria: a seta `→` anda para NORDESTE na linha do \
                 tabuleiro, e o quadrado cinzento (o controlo, sem isometria) vai para a direita. \
                 E com a `→` SEGURADA, carregar na `↑` faz o cinzento trocar de eixo — a ultima \
                 seta manda"
            );
            2
        }
        _ => {
            cena_um(world);
            println!(
                "[topdown-smoke] =1 setas para andar; encoste numa parede na DIAGONAL e ele \
                 desliza a' velocidade CHEIA (de cabeca, ele para)"
            );
            1
        }
    }
}

#[cfg(test)]
#[path = "topdown_smoke_tests.rs"]
mod tests;
