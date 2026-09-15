//! ⭐⭐⭐ **O instantâneo e os gestos das secções FACTORY e LIFECYCLE** (TOP-20 #11 e #12, W3).
//!
//! Irmão do [`super::inspector_camera`], com a mesma fronteira: aqui derivam-se os factos que o
//! painel lê, e aplicam-se as edições que ele manda. O painel não sabe o que é um `StableId`.
//!
//! # ⚠️ A RECEITA viaja pelo NOME, e a conversão é AQUI
//!
//! O componente guarda o `StableId` do mestre; o painel mostra e aceita um **nome**. É a lei da
//! casa (*referência durável entre objectos é o NOME, nunca os bits*) aplicada à porta: o snapshot
//! resolve id → nome, e a edição resolve nome → id. ⛔ Guardar o nome no componente faria renomear
//! a receita partir toda fábrica que a usa; mostrar o id faria o artista escrever um número.
//!
//! # ⚠️ O que o snapshot deriva do MUNDO, e que o componente não sabe
//!
//! Quantas cópias estão vivas · se a receita ainda existe · se há câmera de jogo · se o relógio
//! anda. *São as quatro perguntas que separam «a fábrica está avariada» de «a fábrica está à
//! espera», e nenhuma delas se lê olhando para os campos.*

use ph2d_ecs::{
    DestroyOutside, Entity, Factory, Lifetime, MasterRoot, Pick, SpawnAt, StableId, World,
    alive_by_factory,
};
use ph2d_editor_core::screens::hero::{
    FactoryFieldEdit, InspectorFactory, InspectorFactoryInfo, InspectorLifecycle,
    InspectorSpawnWhere,
};
use ph2d_tags::TagTree;

/// Microssegundos num segundo — a conversão do painel (segundos) para o motor (µs).
const US: f32 = 1_000_000.0;

/// O nome do mestre de `id`, se ele existir e for mesmo um mestre.
fn nome_do_mestre(world: &mut World, id: u64) -> Option<String> {
    if id == 0 {
        return None;
    }
    let e = ph2d_ecs::entity_of_stable_id(world, StableId(id))?;
    // ⚠️ **Tem de ser um MESTRE** — um id que aponte a um objecto comum é o mesmo que não apontar
    // a nada: a porta de instanciar recusa-o, e o painel tem de o dizer ANTES da corrida.
    world.get::<MasterRoot>(e)?;
    world.get::<ph2d_ecs::Name>(e).map(|n| n.0.clone())
}

/// A entidade do mestre chamado `nome` — `0` se não houver nenhum com esse nome.
fn mestre_por_nome(world: &mut World, nome: &str) -> u64 {
    if nome.trim().is_empty() {
        return 0;
    }
    let mut q = world.query::<(Entity, &ph2d_ecs::Name, &MasterRoot)>();
    let achado: Option<Entity> = q
        .iter(world)
        .find(|(_, n, _)| n.0 == nome)
        .map(|(e, _, _)| e);
    let Some(e) = achado else {
        return 0;
    };
    ph2d_ecs::assign_missing_stable_ids(world);
    world.get::<StableId>(e).map_or(0, |s| s.0)
}

/// **O instantâneo.** `None` para quem não tem nenhum dos três componentes (ADR-0166).
///
/// ⚠️ **`&mut World` porque ele faz QUERIES** — contar as cópias vivas, achar o mestre e procurar
/// uma câmera de jogo são consultas, e um `QueryState` do bevy prepara-se com `&mut`. *Não é
/// escrita: é o preço de perguntar à cena em vez de adivinhar a partir do componente.*
pub(crate) fn build_factory_info(
    world: &mut World,
    tree: &TagTree,
    bits: u64,
    selected_count: usize,
    clock_playing: bool,
) -> Option<InspectorFactoryInfo> {
    let e = Entity::from_bits(bits);
    world.get_entity(e).ok()?;
    let tem_fab = world.get::<Factory>(e).is_some();
    let tem_vida = world.get::<Lifetime>(e).is_some();
    let tem_fora = world.get::<DestroyOutside>(e).is_some();
    if !tem_fab && !tem_vida && !tem_fora {
        return None;
    }
    let is_spawned = ph2d_ecs::is_transient(world, e);
    let has_game_camera = ph2d_ecs::camera_2d::camera_count(world) > 0;
    let vivos = alive_by_factory(world);

    let factory = tem_fab.then(|| {
        let f = world
            .get::<Factory>(e)
            .expect("acabou de ser visto")
            .clone();
        let meu_id = world.get::<StableId>(e).map_or(0, |s| s.0);
        let recipe = nome_do_mestre(world, f.master);
        let (spawn_where, area, tag, pick_random) = match f.at {
            SpawnAt::Here => (InspectorSpawnWhere::Here, [0.0, 0.0], String::new(), false),
            SpawnAt::Area { w, h } => (InspectorSpawnWhere::Area, [w, h], String::new(), false),
            SpawnAt::Tagged { tag, pick } => (
                InspectorSpawnWhere::Tagged,
                [0.0, 0.0],
                tree.get(ph2d_tags::TagId(tag))
                    .map(|t| t.path.clone())
                    .unwrap_or_default(),
                pick == Pick::Random,
            ),
        };
        InspectorFactory {
            recipe_found: recipe.is_some(),
            recipe: recipe.unwrap_or_default(),
            on_signal: f.on_signal.clone(),
            spawn_where,
            area,
            tag,
            pick_random,
            burst: f.burst,
            alive_max: f.alive_max,
            total_max: f.total_max,
            on_spawned: f.on_spawned.clone(),
            on_exhausted: f.on_exhausted.clone(),
            seed: f.seed,
            alive: vivos.get(&meu_id).copied().unwrap_or(0),
        }
    });
    let lifecycle = (tem_vida || tem_fora).then(|| {
        let vida = world.get::<Lifetime>(e);
        InspectorLifecycle {
            #[allow(clippy::cast_precision_loss)]
            lifetime_s: vida.map(|v| v.duration_us as f32 / US),
            on_death: vida.map(|v| v.on_death.clone()).unwrap_or_default(),
            outside_margin: world.get::<DestroyOutside>(e).map(|d| d.margin),
        }
    });
    Some(InspectorFactoryInfo {
        entity_bits: bits,
        factory,
        lifecycle,
        is_spawned,
        has_game_camera,
        clock_playing,
        selected_count,
    })
}

/// **Aplica uma edição.** `true` = o documento mudou.
///
/// ⚠️ **O `Where` preserva o que o modo VIZINHO não usa?** ⛔ Não, e é deliberado: o `SpawnAt` é um
/// enum, não três campos ao lado uns dos outros. Trocar de modo e voltar devolve os **defaults**,
/// que é o que a forma do dado diz. *Guardar os três lados para os repor seria um estado paralelo
/// ao componente — o vector paralelo que esta casa proíbe por escrito.*
pub(crate) fn apply_factory_edit(
    world: &mut World,
    tree: &TagTree,
    bits: u64,
    edit: &FactoryFieldEdit,
) -> bool {
    let e = Entity::from_bits(bits);
    if world.get_entity(e).is_err() {
        return false;
    }
    match edit {
        FactoryFieldEdit::Recipe(nome) => {
            let id = mestre_por_nome(world, nome);
            escreve(world, e, |f| f.master = id)
        }
        FactoryFieldEdit::OnSignal(s) => escreve(world, e, |f| f.on_signal = s.clone()),
        FactoryFieldEdit::Where(modo) => {
            let at = match modo {
                InspectorSpawnWhere::Here => SpawnAt::Here,
                InspectorSpawnWhere::Area => SpawnAt::Area { w: 4.0, h: 1.0 },
                InspectorSpawnWhere::Tagged => SpawnAt::Tagged {
                    tag: 0,
                    pick: Pick::Cycle,
                },
            };
            escreve(world, e, |f| f.at = at)
        }
        FactoryFieldEdit::Area([w, h]) => {
            escreve(world, e, |f| f.at = SpawnAt::Area { w: *w, h: *h })
        }
        FactoryFieldEdit::Tag(caminho) => {
            // ⚠️ **Uma tag que não existe fica a `0`**, que a lei lê como *ninguém nasce* — e o
            // painel mostra o campo vazio. ⛔ Criar a tag aqui seria o painel de um assunto a
            // autorar o documento de outro: a árvore tem o painel dela.
            //
            // ⚠️ **A busca é pela porta `find`, que compara DOBRADO** — `Player` e `player` são a
            // mesma tag (a decisão do dono na wave #9), e uma comparação crua aqui faria o campo
            // recusar o que o painel das tags aceita.
            let id = tree.find(caminho).map_or(0, |t| t.0);
            let pick = match world.get::<Factory>(e).map(|f| f.at) {
                Some(SpawnAt::Tagged { pick, .. }) => pick,
                _ => Pick::Cycle,
            };
            escreve(world, e, |f| f.at = SpawnAt::Tagged { tag: id, pick })
        }
        FactoryFieldEdit::PickRandom(on) => {
            let pick = if *on { Pick::Random } else { Pick::Cycle };
            let tag = match world.get::<Factory>(e).map(|f| f.at) {
                Some(SpawnAt::Tagged { tag, .. }) => tag,
                _ => 0,
            };
            escreve(world, e, |f| f.at = SpawnAt::Tagged { tag, pick })
        }
        FactoryFieldEdit::Burst(n) => escreve(world, e, |f| f.burst = *n),
        FactoryFieldEdit::AliveMax(n) => escreve(world, e, |f| f.alive_max = *n),
        FactoryFieldEdit::TotalMax(n) => escreve(world, e, |f| f.total_max = *n),
        FactoryFieldEdit::OnSpawned(s) => escreve(world, e, |f| f.on_spawned = s.clone()),
        FactoryFieldEdit::OnExhausted(s) => escreve(world, e, |f| f.on_exhausted = s.clone()),
        FactoryFieldEdit::Seed(n) => escreve(world, e, |f| f.seed = *n),
        FactoryFieldEdit::LifetimeSeconds(s) => {
            let Some(mut v) = world.get_mut::<Lifetime>(e) else {
                return false;
            };
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let us = (s.max(0.0) * US) as u64;
            if v.duration_us == us {
                return false;
            }
            v.duration_us = us;
            true
        }
        FactoryFieldEdit::OnDeath(s) => {
            let Some(mut v) = world.get_mut::<Lifetime>(e) else {
                return false;
            };
            if v.on_death == *s {
                return false;
            }
            v.on_death = s.clone();
            true
        }
        FactoryFieldEdit::OutsideMargin(m) => {
            let Some(mut d) = world.get_mut::<DestroyOutside>(e) else {
                return false;
            };
            if (d.margin - m).abs() < f32::EPSILON {
                return false;
            }
            d.margin = *m;
            true
        }
    }
}

/// Escreve um campo da fábrica. `false` se o objecto não a tem (nunca um pânico).
fn escreve(world: &mut World, e: Entity, f: impl FnOnce(&mut Factory)) -> bool {
    let Some(mut fab) = world.get_mut::<Factory>(e) else {
        return false;
    };
    f(&mut fab);
    true
}

#[cfg(test)]
#[path = "inspector_factory_tests.rs"]
mod tests;
