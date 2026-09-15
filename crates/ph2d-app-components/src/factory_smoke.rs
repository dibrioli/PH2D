//! ⭐⭐⭐ **Smoke da FÁBRICA e do CICLO DE VIDA** (TOP-20 #11 e #12, W4). `PH2D_FACTORY_SMOKE=1|2`.
//!
//! # `=1` — **a chuva que nunca enche a cena**
//!
//! Uma nuvem no alto com um relógio de `0,7 s` e uma fábrica ligada a ele. A receita é uma **moeda**
//! que cai, e cada moeda vive **2 s**.
//!
//! **O que tem de acontecer:** as moedas caem sem parar, e as mais velhas **somem**. O número de
//! objectos na cena **estabiliza** (≈ 3 moedas vivas), e o painel diz `N alive now`.
//!
//! ⚠️ **O contador é o oráculo, não o ecrã cheio:** uma fábrica sem higiene enche a memória e o
//! ecrã continua a parecer igual durante um minuto. *A prova de que a vida funciona é a contagem
//! parar de subir.*
//!
//! ⚠️ **Rebobinar (`Home`) limpa o ecrã**, e isso é a LEI da wave: o que nasce numa corrida não é
//! documento — ele não entra no ficheiro, não entra no `Ctrl+Z`, e não sobrevive ao relógio voltar
//! ao princípio.
//!
//! # `=2` — **nascer NUM PONTO marcado, com limite e sem lixo fora do ecrã**
//!
//! Três marcas no chão levam a tag `SpawnPoint`. Uma fábrica com *At Tag* nasce **numa de cada
//! vez, em roda-viva**, com `Max Alive = 6`; cada cópia anda para a direita e leva um
//! *Destroy Outside*, que a colhe quando ela sai do ecrã da **câmera do jogo**.
//!
//! **O que tem de acontecer:** as cópias aparecem **alternadamente** nas três marcas, nunca há mais
//! de **seis** vivas, e as que saem pela direita desaparecem em vez de se acumularem.
//!
//! ⚠️ **O `Max Alive` é o controlo do fora-do-ecrã:** se ele fosse a única coisa a segurar a
//! população, a cena não distinguiria *«o colhedor funciona»* de *«o limite está a segurar»*. Aqui
//! a fábrica **volta** a nascer assim que uma sai — e é isso que prova que ela foi de facto colhida.
//!
//! ⚠️ Ligue `PH2D_SIGNAL_LOG=1` junto: o terminal diz quantas nasceram e quantas saíram.
//!
//! ⚠️ Se a linha `[factory-smoke]` não aparecer, **PARE**: a cena não montou.

use ph2d_core::Vec2;
use ph2d_ecs::tags::Tags;
use ph2d_ecs::{
    DestroyOutside, Entity, Factory, GameCamera, Lifetime, MasterRoot, Name, Pick, SpawnAt, Timer,
    Timers, Transform, Visibility, World,
};
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, RigidBody};
use ph2d_render::{Sprite, WHITE_TILE_KEY};
use ph2d_tags::TagTree;

/// ⭐⭐ **Quantas cenas este roteador serve** — o `max_level` que o [`crate::FAMILY`] declara.
///
/// ⚠️ **CONTADO do `match` do [`montar`], nunca escrito de memória** (`CLAUDE.md` §5.0), com gate
/// nas duas pontas. *Uma cena acima do tecto é simplesmente muda, e nada fica vermelho.*
pub const CENAS: u32 = 2;

const MOEDA_RGBA: [f32; 4] = [0.95, 0.78, 0.25, 1.0];
const NUVEM_RGBA: [f32; 4] = [0.45, 0.50, 0.62, 1.0];
const CHAO_RGBA: [f32; 4] = [0.35, 0.45, 0.40, 1.0];
const MARCA_RGBA: [f32; 4] = [0.30, 0.70, 0.95, 1.0];
const BICHO_RGBA: [f32; 4] = [0.85, 0.32, 0.30, 1.0];

fn cuboide(hx: f32, hy: f32) -> Collider {
    Collider {
        shape: ColliderShape::Cuboid {
            half_x: hx,
            half_y: hy,
        },
        density: 1.0,
        ..Collider::default()
    }
}

/// A RECEITA: uma entidade `MasterRoot`, que é o que a fábrica instancia.
///
/// ⚠️ **Um mestre não se DESENHA** (a receita fica escondida, F4.6) — o que se vê são as cópias. É
/// por isso que a nuvem é outro objecto: sem ela, a cena não teria nada no alto a explicar de onde
/// vêm as moedas.
fn receita(world: &mut World, nome: &str, extra: impl FnOnce(&mut World, Entity)) -> Entity {
    let e = world
        .spawn((
            Transform::from_translation(Vec2::new(0.0, 0.0)),
            Name::new(nome),
            MasterRoot,
        ))
        .id();
    extra(world, e);
    e
}

/// A cena `=1` — a chuva que não cresce.
fn cena_um(world: &mut World) {
    world.spawn((
        Name::new("Ground"),
        RigidBody {
            kind: BodyKind::Static,
        },
        cuboide(9.0, 0.3),
        Sprite::atlas(WHITE_TILE_KEY, [18.0, 0.6], CHAO_RGBA),
        Transform::from_translation(Vec2::new(0.0, -3.0)),
    ));
    // A moeda: cai, e vive dois segundos.
    let moeda = receita(world, "Coin", |w, e| {
        w.entity_mut(e).insert((
            Sprite::atlas(WHITE_TILE_KEY, [0.45, 0.45], MOEDA_RGBA),
            Visibility::visible(),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            cuboide(0.22, 0.22),
            // ⭐⭐ **A vida vive na RECEITA e corre nas CÓPIAS** — é a lei do §2.6 do plano, e é
            // isto que a torna alcançável: num objecto solto ela seria inerte, e o painel di-lo.
            Lifetime {
                duration_us: 2_000_000,
                on_death: "coin_gone".to_string(),
            },
        ));
    });
    // ⭐⭐⭐ **A NUVEM** — o relógio que o componente requerido puxa, e a fábrica que o escuta.
    //
    // ⚠️ **O ritmo vem do `Timers`, e não de um `rate` da fábrica** (plano §2.4): um relógio próprio
    // não compraria capacidade nenhuma e daria à casa DOIS motores para a mesma lei.
    world.spawn((
        Transform::from_translation(Vec2::new(0.0, 4.5)),
        Sprite::atlas(WHITE_TILE_KEY, [3.0, 0.8], NUVEM_RGBA),
        Name::new("Rain Cloud"),
        Timers(vec![Timer {
            name: "drop".to_string(),
            duration_us: 700_000,
            repeat: true,
            autostart: true,
            signal: "drop".to_string(),
        }]),
        Factory {
            master: 0, // resolvido em `resolver_receitas` — o mestre ainda não tem identidade
            on_signal: "drop".to_string(),
            at: SpawnAt::Area { w: 6.0, h: 0.2 },
            burst: 1,
            seed: 7,
            ..Factory::default()
        },
        Pendente(moeda),
    ));
}

/// A cena `=2` — os pontos marcados, o limite de vivas e o colhedor do fora-do-ecrã.
fn cena_dois(world: &mut World, tree: &mut TagTree) {
    let ponto = tree.create("SpawnPoint").expect("cria");
    world.spawn((
        Name::new("Ground"),
        RigidBody {
            kind: BodyKind::Static,
        },
        cuboide(9.0, 0.3),
        Sprite::atlas(WHITE_TILE_KEY, [18.0, 0.6], CHAO_RGBA),
        Transform::from_translation(Vec2::new(0.0, -3.0)),
    ));
    // ⭐⭐⭐ **Os pontos de nascimento são OBJECTOS MARCADOS** — não há componente `SpawnPoint`, e
    // essa é a recusa medida do plano §2.3: as tags já o exprimem.
    for (i, x) in [-5.0f32, -3.0, -1.0].into_iter().enumerate() {
        world.spawn((
            Transform::from_translation(Vec2::new(x, -1.5)),
            Sprite::atlas(WHITE_TILE_KEY, [0.5, 0.5], MARCA_RGBA),
            Name::new(format!("Spawn {}", i + 1)),
            Visibility::visible(),
            Tags::from_ids([ponto]),
        ));
    }
    // ⭐ **A câmera do JOGO** — sem ela o fora-do-ecrã não mede nada, e o painel di-lo.
    world.spawn((
        Transform::from_translation(Vec2::new(0.0, 0.0)),
        Name::new("Game Camera"),
        GameCamera {
            height_world: 9.0,
            ..GameCamera::default()
        },
    ));
    // O bicho: anda para a direita e some quando sai do ecrã.
    let bicho = receita(world, "Walker", |w, e| {
        w.entity_mut(e).insert((
            Sprite::atlas(WHITE_TILE_KEY, [0.6, 0.6], BICHO_RGBA),
            Visibility::visible(),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                // ⚠️ **Sem gravidade nem atrito**: a cena é sobre nascer e ser colhido, e um bicho
                // que cai sai do ecrã por baixo antes de chegar à borda direita.
                friction: 0.0,
                ..cuboide(0.3, 0.3)
            },
            ph2d_physics_ecs::GravityScale(0.0),
            ph2d_physics_ecs::InitialVelocity {
                linvel: [2.5, 0.0],
                angvel: 0.0,
            },
            DestroyOutside { margin: 0.5 },
        ));
    });
    world.spawn((
        Transform::from_translation(Vec2::new(0.0, 3.5)),
        Sprite::atlas(WHITE_TILE_KEY, [2.4, 0.8], NUVEM_RGBA),
        Name::new("Spawner"),
        Timers(vec![Timer {
            name: "beat".to_string(),
            duration_us: 500_000,
            repeat: true,
            autostart: true,
            signal: "beat".to_string(),
        }]),
        Factory {
            master: 0,
            on_signal: "beat".to_string(),
            at: SpawnAt::Tagged {
                tag: ponto.0,
                pick: Pick::Cycle,
            },
            burst: 1,
            alive_max: 6,
            on_spawned: "born".to_string(),
            ..Factory::default()
        },
        Pendente(bicho),
    ));
}

/// **A receita que esta fábrica ainda vai apontar** — um marcador de MONTAGEM, não do produto.
///
/// ⚠️ **Ele existe porque a identidade só é atribuída depois**: o `Factory::master` é um `StableId`,
/// e no instante em que a cena monta o mestre ainda não tem um. ⛔ Semear o campo com os bits da
/// entidade seria escrever no componente a coisa que o `CLAUDE.md` proíbe — *referência durável é a
/// IDENTIDADE, nunca os bits*. ⇒ a montagem guarda a entidade e o [`resolver_receitas`] troca-a
/// pelo id, uma vez.
#[derive(bevy_ecs::component::Component, Clone, Copy)]
struct Pendente(Entity);

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

/// **Monta a cena que `nivel` pede, e devolve QUAL montou.**
///
/// ⚠️ Separada do [`factory_smoke`] pela razão da irmã das tags: é a metade que um gate consegue
/// correr — o resto pede o `SceneCtx` inteiro.
pub(crate) fn montar(world: &mut World, tree: &mut TagTree, nivel: u32) -> u32 {
    let cena = match nivel {
        2 => {
            cena_dois(world, tree);
            2
        }
        _ => {
            cena_um(world);
            1
        }
    };
    resolver_receitas(world);
    cena
}

/// Monta a cena. **Devolve qual montou** — o prólogo da shell precisa de saber: as duas são de
/// FÍSICA e o relógio é dele.
///
/// ⚠️ **O transporte NÃO se arma aqui** (a lei da Fase C da `line/app-physics`): *o que sai são os
/// corpos; o que decide a ordem do quadro fica*.
pub fn factory_smoke(cx: &mut crate::scene_ctx::SceneCtx, nivel: u32) -> u32 {
    let cena = montar(cx.sim.world_mut(), cx.tags, nivel);
    match cena {
        2 => eprintln!(
            "[factory-smoke] =2 tres marcas `SpawnPoint` em roda-viva, Max Alive 6, e o fora-do-ecra a colher"
        ),
        _ => eprintln!(
            "[factory-smoke] =1 uma moeda a cada 0,7 s, cada uma vive 2 s — a cena NAO cresce"
        ),
    }
    cena
}

#[cfg(test)]
#[path = "factory_smoke_tests.rs"]
mod tests;
