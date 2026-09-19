//! ⭐⭐⭐ **Smoke das TAGS** (TOP-20 #9, W4). `PH2D_TAGS_SMOKE=1|2`.
//!
//! # `=1` — **um sinal fala com a FAMÍLIA, não com os nomes**
//!
//! Sete objectos lado a lado, cada um com o nome por baixo, e uma taxonomia:
//!
//! ```text
//!   Enemy ........ 5      Goblin A · Goblin B (Enemy)
//!     Flying ..... 3      Bat A · Bat B (Flying)
//!       Boss ..... 1      Dragon (Boss)
//!   Player ....... 1      Hero
//!   Statue ....... 1      Statue
//! ```
//!
//! Um oitavo objecto — o **Scene Brain** — tem um relógio de **2 s** que publica `alarm`, e uma
//! tabela com **uma linha só**: *on `alarm` → Tag `Enemy` → Hide*.
//!
//! **O que tem de acontecer:** aos 2 s somem **CINCO** objectos — os dois goblins, os dois morcegos
//! e o dragão. A **Statue** e o **Hero** ficam.
//!
//! ⚠️ **É a HIERARQUIA que se vê ali:** só dois objectos carregam a tag `Enemy`, e os outros três
//! chegam por descendência (`Flying` e `Boss` estão debaixo dela). Uma linha do painel diz
//! `Enemy (5)` antes de o relógio disparar — e é esse `5` que o sinal alcança.
//!
//! ⚠️ **E a Statue é o CONTROLO:** ela é uma raiz IRMÃ, não uma descendente. Se ela sumir, o alcance
//! está a varrer a árvore inteira em vez da subárvore.
//!
//! # `=2` — **uma armadilha que só dispara para QUEM TEM a tag**
//!
//! Uma placa-sensor no chão com *On Hit* `trap` e *Only for tag* `Player`. Dois corpos caem sobre
//! ela, um de cada vez: primeiro um **Goblin** (`Enemy`), depois o **Hero** (`Player`).
//!
//! **O que tem de acontecer:** o Goblin atravessa e **nada** aparece; o Hero atravessa e a **porta
//! abre** (o rectângulo laranja some).
//!
//! ⚠️ **O Goblin é o controlo, e sem ele a cena não distingue *«o filtro decide»* de *«tudo
//! dispara»*.**
//!
//! ⚠️ Ligue `PH2D_SIGNAL_LOG=1` junto: o terminal diz a origem de cada sinal.
//!
//! ⚠️ Se a linha `[tags-smoke]` não aparecer, **PARE**: a cena não montou.

use ph2d_core::Vec2;
use ph2d_ecs::tags::Tags;
use ph2d_ecs::{
    Entity, Name, SignalAction, SignalActions, SignalTarget, SignalVerb, Timer, Timers, Transform,
    Visibility, World,
};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, RigidBody, SignalOnHit, SignalTagFilter,
};
use ph2d_render::{Sprite, WHITE_TILE_KEY};
use ph2d_tags::{TagId, TagTree};

/// ⭐⭐ **Quantas cenas este roteador serve** — o `max_level` que o [`crate::FAMILY`] declara.
///
/// ⚠️ **CONTADO do `match` do [`tags_smoke`], nunca escrito de memória** (`CLAUDE.md` §5.0), e há
/// gate nas duas pontas: um que lê os braços do próprio ficheiro e outro que confere o valor que a
/// família publica. *Uma cena acima do tecto é simplesmente muda, e nada fica vermelho.*
pub const CENAS: u32 = 2;

const INIMIGO_RGBA: [f32; 4] = [0.85, 0.32, 0.30, 1.0];
const VOADOR_RGBA: [f32; 4] = [0.90, 0.55, 0.25, 1.0];
const CHEFE_RGBA: [f32; 4] = [0.70, 0.25, 0.60, 1.0];
const HEROI_RGBA: [f32; 4] = [0.30, 0.70, 0.95, 1.0];
const ESTATUA_RGBA: [f32; 4] = [0.55, 0.58, 0.62, 1.0];
const CEREBRO_RGBA: [f32; 4] = [0.35, 0.35, 0.40, 1.0];
const PORTA_RGBA: [f32; 4] = [0.95, 0.62, 0.15, 1.0];
const ARMADILHA_RGBA: [f32; 4] = [0.55, 0.30, 0.55, 1.0];
const CHAO_RGBA: [f32; 4] = [0.35, 0.45, 0.40, 1.0];

/// A árvore do plano §5.1 — e **a ordem de criação NÃO é a da árvore**, de propósito: a `Statue`
/// nasce antes do `Enemy`, então quem lê o painel vê a ordem da ÁRVORE e não a de criação.
struct Arvore {
    enemy: TagId,
    flying: TagId,
    boss: TagId,
    statue: TagId,
    player: TagId,
}

fn arvore(tree: &mut TagTree) -> Arvore {
    let statue = tree.create("Statue").expect("cria");
    let boss = tree.create("Enemy/Flying/Boss").expect("cria");
    let enemy = tree.find("Enemy").expect("o ancestral nasceu");
    let flying = tree.find("Enemy/Flying").expect("o ancestral nasceu");
    let player = tree.create("Player").expect("cria");
    Arvore {
        enemy,
        flying,
        boss,
        statue,
        player,
    }
}

/// Um objecto marcado, com nome e cor.
fn marcado(world: &mut World, nome: &str, x: f32, cor: [f32; 4], tag: TagId) -> Entity {
    world
        .spawn((
            Transform::from_translation(Vec2::new(x, 0.0)),
            Sprite::atlas(WHITE_TILE_KEY, [1.1, 1.6], cor),
            Name::new(nome),
            Visibility::visible(),
            Tags::from_ids([tag]),
        ))
        .id()
}

/// A cena `=1` — a fixtura inteira mais o cérebro que fala com a família.
fn cena_um(world: &mut World, a: &Arvore) -> Option<u64> {
    let mut heroi = None;
    for (nome, x, cor, tag) in [
        ("Goblin A", -3.6, INIMIGO_RGBA, a.enemy),
        ("Goblin B", -2.4, INIMIGO_RGBA, a.enemy),
        ("Bat A", -1.2, VOADOR_RGBA, a.flying),
        ("Bat B", 0.0, VOADOR_RGBA, a.flying),
        ("Dragon", 1.2, CHEFE_RGBA, a.boss),
        ("Statue", 2.4, ESTATUA_RGBA, a.statue),
        ("Hero", 3.6, HEROI_RGBA, a.player),
    ] {
        let e = marcado(world, nome, x, cor, tag);
        // ⭐⭐⭐ **O HERÓI é quem a cena ESCOLHE, e a escolha é medida.**
        //
        // ⛔⛔ O smoke desta cena mandava ler a secção *Tags* do Inspector e **isso era impossível**
        // (report do dono, 2026-09-19: *«não faço ideia do que seja»*): a cena abre sem objecto
        // escolhido, logo o Inspector mostra o estado vazio e não existe um único chip no ecrã.
        // *A cena estava certa como DADOS e era impossível como GESTO* — a mesma forma que o #15
        // pagou, e que o prólogo do #18 já cura trazendo o Inspector à frente.
        //
        // ⚠️ **E é o Herói e não um Goblin por MEDIÇÃO:** o cérebro desta cena esconde os cinco
        // inimigos aos 2 s, logo um deles como sujeito deixaria o artista a olhar para um objecto
        // que desaparece. O `Statue` e o `Hero` sobrevivem, e o rótulo do Herói (`Player`) é o mais
        // largo dos dois — é ele que mostra o que a pílula faz.
        if nome == "Hero" {
            // ⚠️ **Em BITS, como as cenas irmãs** — a `hero.gizmo.selection` é um `u64`, e é o que
            //    a ponte da shell sabe traduzir de volta para uma linha da Hierarquia.
            heroi = Some(e.to_bits());
        }
    }
    // ⭐⭐⭐ **O cérebro da cena** — um relógio de 2 s e UMA linha de tabela. Antes desta wave, a
    // mesma coisa pedia cinco linhas, uma por nome, e cada objecto novo pedia a sexta.
    world.spawn((
        Transform::from_translation(Vec2::new(0.0, -2.2)),
        Sprite::atlas(WHITE_TILE_KEY, [1.4, 0.5], CEREBRO_RGBA),
        Name::new("Scene Brain"),
        Timers(vec![Timer {
            name: "alarm".to_string(),
            duration_us: 2_000_000,
            repeat: false,
            autostart: true,
            signal: "alarm".to_string(),
        }]),
        SignalActions(vec![SignalAction {
            on: "alarm".into(),
            // ⚠️ **O `target` fica VAZIO de propósito** — com o alvo por TAG ele não é lido, e um
            // nome escrito aqui leria como se os dois estivessem a decidir.
            target: String::new(),
            verb: SignalVerb::Hide,
            arg: String::new(),
            target_by: SignalTarget::Tagged(a.enemy.0),
        }]),
    ));
    heroi
}

/// A cena `=2` — a armadilha filtrada, o corpo que passa e o que não passa.
fn cena_dois(world: &mut World, a: &Arvore) {
    let cuboide = |hx: f32, hy: f32| Collider {
        shape: ColliderShape::Cuboid {
            half_x: hx,
            half_y: hy,
        },
        density: 1.0,
        ..Collider::default()
    };
    world.spawn((
        Name::new("Ground"),
        RigidBody {
            kind: BodyKind::Static,
        },
        cuboide(8.0, 0.3),
        Sprite::atlas(WHITE_TILE_KEY, [16.0, 0.6], CHAO_RGBA),
        Transform::from_translation(Vec2::new(0.0, -0.3)),
    ));
    // ⭐⭐⭐ **A ARMADILHA** — um sensor que grita `trap`, **só** para quem pertence a `Player`.
    world.spawn((
        Name::new("Trap"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            is_sensor: true,
            ..cuboide(3.2, 0.35)
        },
        SignalOnHit("trap".to_string()),
        SignalTagFilter(a.player.0),
        Sprite::atlas(WHITE_TILE_KEY, [6.4, 0.7], ARMADILHA_RGBA),
        Transform::from_translation(Vec2::new(0.0, 1.4)),
    ));
    // A PORTA: o que o sinal abre. Ela não tem física — é só o que se vê acontecer.
    world.spawn((
        Transform::from_translation(Vec2::new(0.0, 4.2)),
        Sprite::atlas(WHITE_TILE_KEY, [1.6, 2.2], PORTA_RGBA),
        Name::new("Door"),
        Visibility::visible(),
        SignalActions(vec![SignalAction {
            on: "trap".into(),
            target: String::new(),
            verb: SignalVerb::Hide,
            arg: String::new(),
            target_by: SignalTarget::Named,
        }]),
    ));
    // ⚠️ **Os dois caem de alturas DIFERENTES**, para atravessarem um de cada vez: o Goblin
    // primeiro (nada), o Hero depois (a porta abre). Se os dois caíssem juntos, o artista não
    // saberia qual deles disparou.
    for (nome, y, cor, tag) in [
        ("Goblin", 6.0, INIMIGO_RGBA, a.enemy),
        ("Hero", 12.0, HEROI_RGBA, a.player),
    ] {
        world.spawn((
            Name::new(nome),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            cuboide(0.35, 0.35),
            Sprite::atlas(WHITE_TILE_KEY, [0.7, 0.7], cor),
            Transform::from_translation(Vec2::new(0.0, y)),
            Tags::from_ids([tag]),
        ));
    }
}

/// **Monta a cena que `nivel` pede, e devolve QUAL montou.**
///
/// ⚠️ **Ela existe separada do [`tags_smoke`] porque é a metade que um gate consegue correr** — o
/// resto daquele pede o `SceneCtx` inteiro (o ecrã, o relógio, a cena vectorial), e um gate sobre
/// um roteador que não se pode chamar mede o texto em vez do produto.
///
/// ⚠️ **Um nível que o roteador não conhece cai na `=1`** — um ecrã vazio não ensina nada.
/// Monta a cena pedida e devolve **(que cena montou, quem ela escolhe)**.
///
/// ⚠️ O sujeito é `None` na `=2`: ali o que se lê é a armadilha a decidir, e escolher um corpo
/// poria o Inspector à frente de uma cena cujo assunto é o CANVAS.
pub(crate) fn montar(world: &mut World, tree: &mut TagTree, nivel: u32) -> (u32, Option<u64>) {
    let a = arvore(tree);
    match nivel {
        2 => {
            cena_dois(world, &a);
            (2, None)
        }
        _ => (1, cena_um(world, &a)),
    }
}

/// Monta a cena e abre o painel. **Devolve qual cena montou** — o prólogo da shell precisa de
/// saber, porque a `=2` é de FÍSICA e o relógio dela é dele.
///
/// ⚠️ **O transporte NÃO se arma aqui**, e é a lei que a `line/app-physics` pagou na Fase C: *o que
/// sai são os CORPOS; o que decide a ordem do quadro fica*. Uma cena que armasse o `simulate_physics`
/// seria a família a ter opinião sobre o relógio.
pub fn tags_smoke(cx: &mut crate::scene_ctx::SceneCtx, nivel: u32) -> (u32, Option<u64>) {
    let (cena, sujeito) = montar(cx.sim.world_mut(), cx.tags, nivel);
    // ⭐⭐ **O painel abre-se**, e é ele o sujeito desta wave: sem ele o artista vê objectos a sumir
    // e não tem onde ler PORQUÊ.
    if let Some(hero) = cx.hero_screen.as_mut() {
        hero.panel_visibility.insert("tags", true);
    }
    // ⚠️ **A linha que o doc manda procurar** — uma cena que monta em silêncio é uma cena que o
    // smoke julga errado.
    match cena {
        2 => eprintln!(
            "[tags-smoke] =2 armadilha *Only for tag* `Player`: o Goblin atravessa (nada), o Hero abre a porta"
        ),
        _ => eprintln!(
            "[tags-smoke] =1 Enemy(5) > Flying(3) > Boss(1) · Statue(1) · Player(1) — aos 2 s somem CINCO; \
             o HERO abre escolhido, com a etiqueta `Player` na seccao Tags do Inspector"
        ),
    }
    (cena, sujeito)
}

#[cfg(test)]
#[path = "tags_smoke_tests.rs"]
mod tests;
