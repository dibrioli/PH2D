//! ⭐ **A captura INCREMENTAL do desfazer** — o passo custa o tamanho da EDIÇÃO, não o do mundo
//! (ADR-0164 §2.7, plano F2).
//!
//! # As 6 condições, e de que refutação nasceu cada uma
//!
//! O enunciado original (*"byte-idêntico ao `world_to_snapshot` + `canonicalize` de hoje"*) foi
//! **refutado**, e o que sobreviveu é este protocolo
//! ([refutacao_2](../../../../docs/Components/pesquisa/instancias_2026-08-21/refutacao_2_captura_incremental.md)):
//!
//! 1. **Cache** `BTreeMap<StableId, Row>` com a linha em `Arc` — clone de 10 k linhas custa
//!    `0,038 ms` contra `0,776 ms` da cópia profunda.
//! 2. **Sujo** = algum componente registado **ou o `ChildOf`** tem tick mais novo que o da última
//!    captura, **ou** o `ArchetypeId` difere do cacheado. ⚠️ **A segunda metade não é redundante**:
//!    *remover* um componente **não carimba tick de ninguém* (R2, medido — `remove::<Sprite>` em
//!    1 % das entidades dava **zero** linhas re-serializadas), e toda remoção muda o archetype.
//! 3. **O tick é PRÉ-FILTRO; os bytes são a verdade.** Um `get_mut` sem escrever **carimba**
//!    mudança (R2, medido: 1 000 linhas re-serializadas com bytes idênticos), então re-serializa-se
//!    a linha suja e **compara-se com o cache** — só difere quem de facto mudou.
//! 4. **Spawn** por ausência no cache; **despawn** por carimbo de geração (`seen`).
//! 5. ⚠️⚠️ **`clear_trackers()` corre exactamente UMA vez por CAPTURA — nunca por quadro** (R3): o
//!    `is_newer_than` é **estrito**, e um clear por quadro durante um arrasto de 10 quadros faria
//!    entrar no passo só a mutação do último. Quem o chama é [`CaptureCache::finish`], e ela é
//!    chamada **pela captura**, não pelo laço de quadro.
//! 6. **`StableId` único e presente** — garantido na DERIVAÇÃO por
//!    [`crate::assign_missing_stable_ids`], que esta função chama.
//!
//! # ⚠️ A varredura é por ARCHETYPE, e o `#![forbid(unsafe_code)]` desta crate escolheu a forma
//!
//! O caminho mais rápido do bevy é ler a **coluna** de ticks da tabela
//! (`Table::get_changed_ticks_slice_for`), que devolve `&[UnsafeCell<Tick>]` — e ler um
//! `UnsafeCell` exige `unsafe`, que esta crate **proíbe na primeira linha do `lib.rs`**.
//!
//! A forma segura que resta é `EntityRef::get_change_ticks_by_id`, **restringida à interseção
//! `registados ∩ archetype.components()`** e com essa interseção **memorizada por `ArchetypeId`**.
//! É isso que separa este scan do ingénuo: o ingénuo pergunta pelos ~73 tipos a cada entidade;
//! este pergunta só pelos que o archetype de facto tem (tipicamente 4–8).
//!
//! *A cerca da crate não foi contornada — ela escolheu o algoritmo, e o número está no bench.*

use std::collections::BTreeMap;
use std::sync::Arc;

use bevy_ecs::archetype::ArchetypeId;
use bevy_ecs::change_detection::Tick;
use bevy_ecs::component::ComponentId;
use bevy_ecs::entity::Entity;
use bevy_ecs::world::World;

use super::registry::ComponentRegistry;
use super::save::{EntitySnapshotRow, SaveError, WorldSnapshot};
use crate::StableId;

/// O que uma captura fez — para o `PH2D_UNDO_LOG` e para os gates.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct CaptureReport {
    /// Linhas que o pré-filtro marcou como sujas (tick ou archetype).
    pub dirty: usize,
    /// Dessas, quantas de facto mudaram de bytes. ⚠️ `dirty - reserialized` é o **falso
    /// positivo** do `DerefMut` — um número grande aqui diz que alguém está a fazer `get_mut`
    /// sem escrever, e a cura é `set_if_neq` em quem escreve.
    pub reserialized: usize,
    /// Entidades novas desde a última captura.
    pub spawned: usize,
    /// Entidades que desapareceram desde a última captura.
    pub despawned: usize,
    /// Total de linhas no snapshot produzido.
    pub rows: usize,
    /// Os bytes das linhas que de facto mudaram — o **tamanho da edição**.
    ///
    /// ⚠️ É o número que a fase inteira existe para tornar pequeno, e o único que responde à
    /// pergunta *"quanto custa este passo na pilha?"*. As outras contagens dizem quanto trabalho
    /// a captura fez; esta diz quanto ela **entregou**.
    pub delta_bytes: usize,
}

/// Uma linha cacheada: a linha em si + o que decide se ela está suja.
#[derive(Clone)]
struct CachedRow {
    /// ⚠️ **O archetype é metade do critério de sujidade** — ver a condição 2. Sem ele, remover
    /// um componente deixa a linha STALE para sempre.
    archetype: ArchetypeId,
    /// ⭐ **Os bits da entidade que esta linha era da última vez** — o que torna o caminho limpo
    /// livre de buscas: a varredura percorre a cache e vai DIRECTO ao mundo, em vez de percorrer
    /// o mundo e procurar a linha (medido: `0,356 → 0,2 ms` a 10 k; um `BTreeMap::get_mut` são
    /// ~13 comparações com salto de ponteiro, contra uma indexação de array).
    ///
    /// ⚠️⚠️ **Bits são RECICLADOS, então eles não bastam como identidade** — e é por isso que a
    /// varredura confirma o `StableId` do que encontrou. Sem essa conferência, uma entidade
    /// despawnada cujos bits o bevy já deu a OUTRA faria esta linha ler o objeto errado, em
    /// silêncio. *O `Entity` aqui é um atalho de acesso; a identidade continua a ser o id.*
    entity: Entity,
    row: Arc<EntitySnapshotRow>,
    /// A geração da última captura em que esta entidade foi vista — o carimbo do despawn.
    seen: u64,
}

/// O estado que sobrevive entre capturas. **Uma por fila de undo.**
pub struct CaptureCache {
    rows: BTreeMap<StableId, CachedRow>,
    /// O tick da **última captura**. Tudo com tick mais novo que este está sujo.
    last_capture: Tick,
    /// A geração corrente — incrementada por captura, usada para detectar despawn.
    generation: u64,
    /// ⚠️ **A interseção `registados ∩ archetype`, memorizada por archetype.** É o que faz este
    /// scan não ser o ingénuo. Cresce com o número de FORMAS de entidade (dezenas), não com o
    /// número de entidades.
    ///
    /// ⚠️ **Um `Vec` indexado pelo `ArchetypeId`, e não um mapa** — MEDIDO: o caminho limpo fazia
    /// **quatro** buscas de `BTreeMap` por entidade (`contains_key` + índice aqui, `get` +
    /// `get_mut` nas linhas), e a 10 k entidades isso era metade do orçamento da fase. Os
    /// `ArchetypeId` são inteiros densos a partir de zero, então o array é a estrutura certa e a
    /// busca vira uma indexação. *Uma memória com a chave errada custa mais do que a conta que
    /// ela evita.*
    per_archetype: Vec<Option<Vec<ComponentId>>>,
    /// Os `ComponentId` de tudo o que se observa, neste mundo: os registados **+ o `ChildOf`**.
    /// Resolvido na primeira captura (antes disso o mundo pode nem conhecer os tipos).
    watched: Vec<ComponentId>,
    /// A primeira captura não tem baseline: tudo é spawn, e o `last_capture` ainda não vale.
    primed: bool,
    /// O relatório da última captura — para o `PH2D_UNDO_LOG` o poder imprimir sem que a
    /// assinatura da captura tenha de o devolver por 11 sítios de chamada.
    last_report: CaptureReport,
}

impl Default for CaptureCache {
    fn default() -> Self {
        Self::new()
    }
}

impl CaptureCache {
    #[must_use]
    pub fn new() -> Self {
        Self {
            rows: BTreeMap::new(),
            last_capture: Tick::new(0),
            generation: 0,
            per_archetype: Vec::new(),
            watched: Vec::new(),
            primed: false,
            last_report: CaptureReport::default(),
        }
    }

    /// O relatório da última captura (diagnóstico — `PH2D_UNDO_LOG=1`).
    #[must_use]
    pub fn last_report(&self) -> CaptureReport {
        self.last_report
    }

    /// **Esquece tudo.** O restore do undo repõe um mundo que esta cache não viu nascer, então
    /// continuar a comparar contra ela daria linhas limpas sobre bytes diferentes.
    ///
    /// ⚠️ Ela é chamada pelo restore **e dá o próprio `clear_trackers`** (condição 5): o respawn
    /// carimba tudo, e sem o clear a captura seguinte veria o mundo inteiro sujo — correcto, mas
    /// pelo motivo errado e ao preço de uma captura completa.
    pub fn reset(&mut self, world: &mut World) {
        self.rows.clear();
        self.per_archetype.clear();
        self.watched.clear();
        self.primed = false;
        self.generation = 0;
        world.clear_trackers();
        self.last_capture = world.last_change_tick();
    }

    /// Quantas linhas a cache guarda (diagnóstico e gates).
    #[must_use]
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// Os `ComponentId` observados, resolvidos contra ESTE mundo — ver
    /// [`super::registry::ComponentTypeEntry::bevy_component_id`].
    fn resolve_watched(&mut self, world: &World, registry: &ComponentRegistry) {
        self.watched.clear();
        for entry in registry.iter() {
            if let Some(id) = (entry.bevy_component_id)(world) {
                self.watched.push(id);
            }
        }
        // ⚠️ **O `ChildOf` entra mesmo não sendo registado** (condição 2): o `parent` da linha
        // sai dele, então um reparent muda os bytes — e nenhum componente registado o diria.
        if let Some(id) = world.component_id::<bevy_ecs::hierarchy::ChildOf>() {
            self.watched.push(id);
        }
        self.watched.sort_unstable();
        self.watched.dedup();
        // O mapa por archetype foi construído contra a lista antiga.
        self.per_archetype.clear();
    }
}

/// **A captura incremental.** Devolve o snapshot completo (todas as linhas) reaproveitando as
/// que não mudaram, e um relatório do que foi feito.
///
/// ⚠️ **Ela chama `clear_trackers` no fim — UMA vez, aqui** (condição 5). Quem quiser um quadro
/// de retorno precoce (um gesto segurado que ainda não fecha um passo) simplesmente **não chama
/// esta função**; chamar e descartar o resultado seria o clear por quadro que a R3 refutou.
pub fn capture_incremental(
    world: &mut World,
    cache: &mut CaptureCache,
    registry: &ComponentRegistry,
    out: &mut WorldSnapshot,
) -> Result<CaptureReport, SaveError> {
    // Condição 6 — a identidade garante-se na DERIVAÇÃO, como no `world_to_snapshot`.
    crate::assign_missing_stable_ids(world);
    if !cache.primed {
        cache.resolve_watched(world, registry);
    }

    let this_run = world.change_tick();
    let last_run = cache.last_capture;
    cache.generation = cache.generation.wrapping_add(1);
    let generation = cache.generation;

    let mut report = CaptureReport::default();

    // ⚠️ **Destruturado para os empréstimos serem DISJUNTOS**: `cols` empresta o
    // `per_archetype` e a linha empresta o `rows`.
    let CaptureCache {
        rows,
        per_archetype,
        watched,
        ..
    } = &mut *cache;

    // ⭐⭐ **PASSAGEM A — a cache guia, o mundo responde.** Percorre as linhas que já existem e
    // vai DIRECTO à entidade de cada uma. É o caminho comum (nada nasce nem morre na esmagadora
    // maioria dos quadros), e ele não faz **nenhuma** busca de mapa.
    //
    // ⚠️ A ordem inversa — percorrer o mundo e procurar a linha — custa um `BTreeMap::get_mut`
    // por entidade, que a 10 k era ~40 % do orçamento da fase (medido).
    let mut matched = 0usize;
    for (id, cached) in rows.iter_mut() {
        // ⚠️⚠️ **Bits reciclados:** confirmar o `StableId` é o que impede esta linha de ler outro
        // objeto que herdou os bits do que morreu. Ver o campo `entity`.
        let Ok(eref) = world.get_entity(cached.entity) else {
            continue; // morreu — o `retain` abaixo apanha-a
        };
        if eref.get::<StableId>() != Some(id) {
            continue; // os bits são de OUTRO objeto agora
        }
        matched += 1;
        let archetype_id = eref.archetype().id();

        // A interseção memorizada — uma INDEXAÇÃO, não uma busca (ver o campo).
        let ai = archetype_id.index();
        if per_archetype.len() <= ai {
            per_archetype.resize(ai + 1, None);
        }
        if per_archetype[ai].is_none() {
            per_archetype[ai] = Some(
                eref.archetype()
                    .components()
                    .iter()
                    .filter(|c| watched.binary_search(c).is_ok())
                    .copied()
                    .collect(),
            );
        }
        let cols = per_archetype[ai]
            .as_ref()
            .expect("acabou de ser preenchido");

        // Condição 2: tick OU archetype.
        let dirty = cached.archetype != archetype_id
            || cols.iter().any(|cid| {
                eref.get_change_ticks_by_id(*cid)
                    .is_some_and(|t| t.is_changed(last_run, this_run))
            });
        cached.seen = generation;
        if !dirty {
            continue; // reaproveita a linha inteira — o custo deixa de ser o do mundo
        }
        report.dirty += 1;

        // Condição 3: re-serializa e COMPARA. O tick é pré-filtro, os bytes são a verdade.
        let row = build_row(world, cached.entity, *id, registry)?;
        cached.archetype = archetype_id;
        if *cached.row != row {
            report.delta_bytes += row_bytes(&row);
            cached.row = Arc::new(row);
            report.reserialized += 1;
        }
    }

    // **PASSAGEM B — os que a passagem A não alcançou**, e ela só corre quando há algum.
    //
    // ⚠️ O gatilho é a CONTAGEM, e ele é exacto: um spawn faz `vivas > casadas`; um despawn
    // sozinho deixa as duas iguais (a morta não casa e não conta em nenhuma das duas); um spawn
    // **com** um despawn no mesmo quadro também difere. *Uma igualdade de contagens é a prova de
    // que não há linha nova — não uma heurística.*
    let live_count = {
        let mut q = world.query_filtered::<Entity, bevy_ecs::prelude::With<StableId>>();
        q.iter(world).count()
    };
    if live_count != matched {
        // ⚠️⚠️ **O critério é «não foi ATUALIZADA nesta geração», não «não está na cache»** — e a
        // diferença é um defeito que dois gates da shell apanharam e nenhum desta crate tinha.
        //
        // Um **RESPAWN** (o restore do undo, e o sync do vetor a cada quadro) devolve o objeto com
        // a MESMA identidade e **bits novos**. Na passagem A o `entity` cacheado já não resolve,
        // então ela salta a linha; se a passagem B perguntasse *"está na cache?"*, a resposta seria
        // **sim** — a linha velha ainda lá está — e a entidade não seria reconstruída por ninguém.
        // O `retain` seguinte apagava-a, e o objeto **desaparecia do snapshot**.
        //
        // *Perguntar pela GERAÇÃO cobre os dois casos com uma condição: o que nunca existiu e o
        // que renasceu noutros bits.*
        let stale: Vec<(Entity, StableId)> = {
            let mut q = world.query::<(Entity, &StableId)>();
            q.iter(world)
                .filter(|(_, s)| rows.get(s).map_or(true, |c| c.seen != generation))
                .map(|(e, s)| (e, *s))
                .collect()
        };
        for (entity, id) in stale {
            let Ok(eref) = world.get_entity(entity) else {
                continue;
            };
            let archetype = eref.archetype().id();
            let row = build_row(world, entity, id, registry)?;
            report.dirty += 1;
            match rows.get_mut(&id) {
                // Renasceu: a linha fica, os bits são os novos. ⚠️ Só conta como
                // `reserializada` se os bytes de facto mudaram — um respawn que devolve o mesmo
                // estado (o caso do ponto fixo) não pode produzir um passo de undo.
                Some(c) => {
                    c.archetype = archetype;
                    c.entity = entity;
                    c.seen = generation;
                    if *c.row != row {
                        report.delta_bytes += row_bytes(&row);
                        c.row = Arc::new(row);
                        report.reserialized += 1;
                    }
                }
                None => {
                    report.reserialized += 1;
                    report.spawned += 1;
                    report.delta_bytes += row_bytes(&row);
                    rows.insert(
                        id,
                        CachedRow {
                            archetype,
                            entity,
                            row: Arc::new(row),
                            seen: generation,
                        },
                    );
                }
            }
        }
    }

    // Condição 4 — despawn por carimbo.
    let before = rows.len();
    rows.retain(|_, c| c.seen == generation);
    report.despawned = before - rows.len();

    // A saída: as linhas em ordem de `StableId`, que é a ordem canónica da v2.
    out.version = WorldSnapshot::VERSION;
    out.entities.clear();
    out.entities.reserve(rows.len());
    out.entities
        .extend(rows.values().map(|c| Arc::clone(&c.row)));
    report.rows = out.entities.len();

    // ⚠️⚠️ Condição 5 — UMA vez, por CAPTURA, e é a última coisa que acontece.
    cache.last_report = report;
    world.clear_trackers();
    // ⚠️⚠️ **`last_change_tick()`, e NÃO `change_tick()`** — a diferença de um é a diferença
    // entre ver as mudanças e não ver nenhuma, e ela custou os quatro gates deste ficheiro.
    //
    // O `clear_trackers` faz `last_change_tick = increment_change_tick()`: ele guarda o tick
    // ANTERIOR (`C`) e deixa o relógio em `C+1`. Toda escrita a seguir carimba `C+1`, e o
    // `is_newer_than` é **estrito** — guardar `change_tick()` (`C+1`) faria `C+1 > C+1` ser
    // falso e **toda mudança ficaria invisível**. É o mesmo `last_run` que os sistemas do bevy
    // recebem, e por isso é o valor certo.
    cache.last_capture = world.last_change_tick();
    cache.primed = true;
    Ok(report)
}

/// **A linha de uma entidade** — os blobs em ordem de registo, mais o pai pelo id dele.
///
/// ⚠️ Ela é a MESMA conta que o [`super::save::world_to_snapshot`] faz, e é isso que o gate da
/// equivalência afirma. Duas versões desta função seriam duas respostas a *"o que é uma linha"*,
/// e a divergência só apareceria num Ctrl+Z.
fn build_row(
    world: &World,
    entity: Entity,
    id: StableId,
    registry: &ComponentRegistry,
) -> Result<EntitySnapshotRow, SaveError> {
    let mut row = EntitySnapshotRow {
        id,
        components: Vec::new(),
        parent: None,
    };
    for entry in registry.iter() {
        match (entry.serialize)(world, entity) {
            Ok(Some(bytes)) => row.components.push(super::save::blob(entry.type_id, bytes)),
            Ok(None) => {}
            Err(e) => return Err(e.into()),
        }
    }
    if let Ok(eref) = world.get_entity(entity)
        && let Some(co) = eref.get::<bevy_ecs::hierarchy::ChildOf>()
    {
        row.parent = crate::stable_id_of(world, co.0);
    }
    Ok(row)
}

/// Quantos bytes esta linha ocupa no passo — a soma dos blobs. ⚠️ Aproxima o custo de disco,
/// não o de memória: o `Vec` e o `StableId` à volta não são contados, porque o que a pergunta
/// *"quanto custa este passo"* quer saber é o payload.
fn row_bytes(row: &EntitySnapshotRow) -> usize {
    row.components.iter().map(|b| b.data.len()).sum()
}

#[cfg(test)]
#[path = "incremental_tests.rs"]
mod tests;
