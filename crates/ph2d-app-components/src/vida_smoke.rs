//! ⭐⭐⭐ **Smoke da VIDA** (plano 28, W2). `PH2D_VIDA_SMOKE=1`.
//!
//! # A cena: **quatro alvos numa coluna, e o herói que atira**
//!
//! O herói azul atira com o **`Q`** (a mesma arma da cena do golpe). Cada alvo tem uma VIDA
//! ([`ph2d_physics_ecs::Health`]) e cada bala um DANO ([`ph2d_physics_ecs::Damage`]) de `10`:
//!
//! | alvo | vida | o que acontece |
//! |---|---|---|
//! | vermelho | `10` | morre ao **1.º** tiro |
//! | laranja | `20` | morre ao **2.º** |
//! | roxo | `30` | morre ao **3.º** |
//! | cinzento (o CONTROLO) | `10`, **mesma equipa do herói** | **nunca** morre — as balas dele não o ferem |
//!
//! ⚠️ **Nenhum alvo tem tabela de acções**: a morte é da VIDA, e quem os tira da cena é o dreno da
//! shell (`mortes_anunciadas`). Na cena do golpe (#24) era uma linha `Destroy` por alvo; aqui é
//! `0` linhas — *é essa a diferença que a W2 compra*.
//!
//! # ⛔ Os alvos são SÓLIDOS, e a razão foi MEDIDA
//!
//! A bala é um mover que **pára rente** ao obstáculo e só vê formas SÓLIDAS (a sonda
//! `mede_o_golpe_que_chega`, casos E e G): um alvo-sensor seria atravessado sem ser visto, e o dano
//! chegaria pelo canal do mover só se o alvo fosse sólido. Na cena do golpe os alvos eram sensores
//! porque quem reportava era o `SignalOnHit`.
//!
//! # ⛔ E os alvos NASCEM, não são postos à mão
//!
//! Só sai da cena quem nasceu numa corrida (`ph2d_ecs::is_transient`): um alvo de DOCUMENTO morre
//! (deixa de levar tiros, grita o sinal) e **fica**. ⇒ cada alvo sai da sua **fábrica** (TOP-20
//! #11), que um **relógio** (#2) arranca ao entrar a corrida — a lição da cena do golpe.
//!
//! ⚠️ Se a linha `[vida-smoke]` não aparecer, **PARE**: a cena não montou.

use ph2d_core::Vec2;
use ph2d_ecs::{
    Entity, Factory, Lifetime, MasterRoot, Name, SignalOnAction, Timer, Timers, Transform,
    Visibility, World,
};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, Damage, Health, OnHit, ProjectileMotion, RigidBody,
};
use ph2d_projectile::ProjectileLaw;
use ph2d_render::{Sprite, WHITE_TILE_KEY};
use ph2d_topdown::{TopDownLaw, direction::DirectionMode};

/// ⭐⭐ **Quantas cenas este roteador serve** — CONTADO do `match` do [`montar`].
pub const CENAS: u32 = 1;

/// A acção e a tecla — as MESMAS da cena do golpe, que é a fonte (a tecla foi medida lá).
pub const ACCAO: &str = crate::dano_smoke::ACCAO;
/// O nome da tecla, para o roteiro.
pub const TECLA_NOME: &str = crate::dano_smoke::TECLA_NOME;
/// O sinal do gatilho.
const SINAL: &str = "tiro-vida";
/// O sinal que arranca as quatro fábricas.
const COMECAR: &str = "comecar-vida";
/// O que um alvo grita ao levar dano, e ao morrer.
pub const AI: &str = "ai";
/// O que um alvo grita ao morrer.
pub const MORREU: &str = "morreu";
/// A equipa do herói — e do aliado cinzento.
pub const HEROIS: &str = "herois";
/// A equipa dos três alvos que morrem.
pub const MONSTROS: &str = "monstros";
/// O dano de uma bala.
pub const DANO: f32 = 10.0;

const CHAO_RGBA: [f32; 4] = [0.16, 0.18, 0.22, 1.0];
const HEROI_RGBA: [f32; 4] = [0.35, 0.62, 0.95, 1.0];
const BALA_RGBA: [f32; 4] = [0.98, 0.82, 0.25, 1.0];

/// ⭐⭐ **Os quatro alvos, de cima para baixo** — `(nome, vida, equipa, cor, y)`.
///
/// ⚠️ **Os `y` cabem na banda que a régua deixa visível** (`+4,09` / `−1,19` m, MEDIDA na foto da
/// cena da arma): com a timeline aberta a vista não é centrada na origem, e uma coluna simétrica
/// punha os dois de baixo fora do ecrã.
pub const ALVOS: [(&str, f32, &str, [f32; 4], f32); 4] = [
    (
        "Alvo de 1 tiro",
        10.0,
        MONSTROS,
        [0.88, 0.30, 0.28, 1.0],
        3.4,
    ),
    (
        "Alvo de 2 tiros",
        20.0,
        MONSTROS,
        [0.95, 0.60, 0.22, 1.0],
        2.1,
    ),
    (
        "Alvo de 3 tiros",
        30.0,
        MONSTROS,
        [0.62, 0.38, 0.85, 1.0],
        0.8,
    ),
    (
        "Aliado (nao morre)",
        10.0,
        HEROIS,
        [0.55, 0.57, 0.60, 1.0],
        -0.5,
    ),
];
/// O `x` da coluna de alvos.
pub const X_ALVOS: f32 = 4.0;
/// O lado de um alvo. ⚠️ Com `1,3` m entre filas, a janela de mira do herói é `LADO/2 + 0,1` =
/// `0,6` m para cada lado — o dobro do raio do herói.
pub const LADO: f32 = 1.0;

/// **A receita que esta fábrica ainda vai apontar** — o marcador de MONTAGEM (o molde da cena do
/// golpe): a identidade só é atribuída depois.
#[derive(bevy_ecs::component::Component, Clone, Copy)]
struct Pendente(Entity);

/// **A RECEITA de um alvo** — um corpo SÓLIDO com uma VIDA, e nada mais.
fn receita_do_alvo(
    world: &mut World,
    (nome, vida, equipa, cor, _): (&str, f32, &str, [f32; 4], f32),
) -> Entity {
    world
        .spawn((
            Name::new(nome),
            MasterRoot,
            Transform::from_translation(Vec2::ZERO),
            Visibility::visible(),
            Sprite::atlas(WHITE_TILE_KEY, [LADO, LADO], cor),
            RigidBody {
                kind: BodyKind::Static,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: LADO / 2.0,
                    half_y: LADO / 2.0,
                },
                ..Collider::default()
            },
            Health {
                max: vida,
                start: vida,
                team: equipa.to_owned(),
                on_damage: AI.to_owned(),
                on_death: MORREU.to_owned(),
                ..Health::default()
            },
        ))
        .id()
}

/// **A RECEITA da bala** — o projéctil da cena do golpe, com um DANO em vez de uma tag.
fn receita_da_bala(world: &mut World) -> Entity {
    world
        .spawn((
            Name::new("Bala"),
            MasterRoot,
            Transform::from_translation(Vec2::ZERO),
            Visibility::visible(),
            Sprite::atlas(WHITE_TILE_KEY, [0.5, 0.16], BALA_RGBA),
            RigidBody {
                kind: BodyKind::Kinematic,
            },
            Collider {
                shape: ColliderShape::Ball { radius: 0.1 },
                ..Collider::default()
            },
            ProjectileMotion::from_law(
                ProjectileLaw {
                    initial_speed: 9.0,
                    range: 14.0,
                    face_velocity: true,
                    ..ProjectileLaw::default()
                },
                0,
            ),
            // ⚠️ Sem ele cada tiro que não acerta fica parado no fim do alcance (a lei do #12).
            Lifetime {
                duration_us: 3_000_000,
                ..Lifetime::default()
            },
            Damage {
                amount: DANO,
                team: HEROIS.to_owned(),
                on_hit: OnHit::Vanish,
                ..Damage::default()
            },
        ))
        .id()
}

/// A cena `=1`. Devolve **quem nasce ESCOLHIDO** — o herói.
fn cena_um(world: &mut World) -> Entity {
    // ⚠️⚠️ **O CHÃO PRIMEIRO**: a ordem das raízes é a de CRIAÇÃO (a cura de 15/09).
    world.spawn((
        Name::new("Ground"),
        Sprite::atlas(WHITE_TILE_KEY, [22.0, 12.0], CHAO_RGBA),
        Transform::from_translation(Vec2::ZERO),
    ));
    // O RELÓGIO que arranca as fábricas — `autostart`, sem repetição (o molde da cena do golpe).
    world.spawn((
        Name::new("Arranque"),
        Transform::from_translation(Vec2::new(0.0, 5.0)),
        Timers(vec![Timer {
            duration_us: 250_000,
            signal: COMECAR.to_owned(),
            autostart: true,
            repeat: false,
            ..Timer::default()
        }]),
    ));
    for alvo in ALVOS {
        let receita = receita_do_alvo(world, alvo);
        world.spawn((
            Name::new(format!("Fabrica: {}", alvo.0)),
            Transform::from_translation(Vec2::new(X_ALVOS, alvo.4)),
            Factory {
                master: 0,
                on_signal: COMECAR.to_owned(),
                burst: 1,
                total_max: 1,
                ..Factory::default()
            },
            Pendente(receita),
        ));
    }
    let bala = receita_da_bala(world);

    // ⭐ O HERÓI: anda com as setas, roda para onde anda, e a arma aponta para onde ele aponta.
    // ⚠️ Nasce À ALTURA do alvo de cima — o 1.º tiro não pede pontaria.
    let heroi = world
        .spawn((
            Name::new("Heroi"),
            RigidBody {
                kind: BodyKind::Kinematic,
            },
            Collider {
                shape: ColliderShape::Ball { radius: 0.3 },
                ..Collider::default()
            },
            Sprite::atlas(WHITE_TILE_KEY, [0.9, 0.4], HEROI_RGBA),
            Transform::from_translation(Vec2::new(-6.0, ALVOS[0].4)),
            ph2d_physics_ecs::TopDownPlayer::from_law(TopDownLaw {
                speed: 4.0,
                direction: DirectionMode::Free,
                rotation: ph2d_topdown::rotation::RotationMode::ToMovement,
                ..TopDownLaw::default()
            }),
            SignalOnAction(vec![ph2d_ecs::ActionTriggerRow {
                action: ACCAO.to_owned(),
                edge: ph2d_ecs::ActionEdge::Press,
                signal: SINAL.to_owned(),
            }]),
            Factory {
                master: 0,
                on_signal: SINAL.to_owned(),
                burst: 1,
                aim_from_spawner: true,
                ..Factory::default()
            },
        ))
        .id();
    world.entity_mut(heroi).insert(Pendente(bala));
    heroi
}

/// Troca cada [`Pendente`] pelo `StableId` do mestre. ⚠️ Corre DEPOIS de a identidade existir.
fn resolver_receitas(world: &mut World) {
    ph2d_ecs::assign_missing_stable_ids(world);
    let mut q = world.query::<(Entity, &Pendente)>();
    let pares: Vec<(Entity, Entity)> = q.iter(world).map(|(e, p)| (e, p.0)).collect();
    for (fab, mestre) in pares {
        let id = world.get::<ph2d_ecs::StableId>(mestre).map_or(0, |s| s.0);
        if let Some(mut f) = world.get_mut::<Factory>(fab) {
            f.master = id;
        }
        world.entity_mut(fab).remove::<Pendente>();
    }
}

/// O que o prólogo precisa de saber da cena montada.
pub struct Montada {
    /// Qual cena foi montada.
    pub nivel: u32,
    /// O HERÓI, que nasce escolhido.
    pub escolhido: u64,
}

/// **Monta a cena que `nivel` pede, e devolve QUAL montou.**
pub fn montar(world: &mut World, _nivel: u32) -> Montada {
    let escolhido = cena_um(world);
    resolver_receitas(world);
    println!(
        "[vida-smoke] cena=1  tecla={TECLA_NOME}  dano de uma bala={DANO}\n\
         (0) o quadrado de contorno verde no MEIO do ecra' sao os MOLDES (o do alvo e o da bala): \
         e' deles que cada alvo e cada bala nascem. Nao levam tiros e nao se mexem\n\
         (1) espere um instante: nascem QUATRO quadrados numa coluna a' direita — vermelho, \
         laranja, roxo e, em baixo, cinzento\n\
         (2) carregue no {TECLA_NOME}: o heroi azul ja' nasce a' altura do VERMELHO. Ele some ao \
         1.o tiro, e aparece o aviso `{AI}` e depois `{MORREU}`\n\
         (3) segure a seta para BAIXO ate' ficar a' altura do LARANJA, depois a seta para a \
         DIREITA (o heroi vira-se para onde anda) e atire: o 1.o tiro so' mostra `{AI}`, o 2.o \
         mata-o\n\
         (4) faca o mesmo no ROXO: precisa de TRES tiros\n\
         (5) o CONTROLO: atire no CINZENTO — ele e' da mesma equipa do heroi, e as balas batem nele \
         e somem sem o ferir. Nunca aparece `{AI}`\n\
         (6) na barra de CIMA carregue em `Reset` e depois em `Play`: os quatro voltam, cada um \
         com a vida inteira\n\
         (7) clique no ROXO: no painel da direita (Inspector) aparece a seccao `Health` com \
         `Now: 30 of 30`. Atire nele e o numero desce 10 por tiro. A bala tem a seccao `Damage`\n\
         (8) deu errado se: o laranja ou o roxo morrem ao 1.o tiro · o cinzento some · a bala \
         atravessa um quadrado · carregar no {TECLA_NOME} nao faz nada · ou nenhum quadrado some \
         depois de muitos tiros"
    );
    Montada {
        nivel: 1,
        escolhido: escolhido.to_bits(),
    }
}

#[cfg(test)]
#[path = "vida_smoke_tests.rs"]
mod tests;
