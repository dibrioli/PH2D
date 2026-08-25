//! **`StableId` — A identidade de objeto do PH2D** ([ADR-0164](../../../docs/architecture/decisions/0164-instances-are-real-entities-linked-by-stableid-with-live-sync-and-incremental-undo.md) §1, plano F1).
//!
//! # O buraco que ele fecha, com a citação
//!
//! O doc-comment de [`crate::name`] diz, desde antes desta wave: *"**O ECS não tem id
//! estável.** `Entity::to_bits()` é um id de ALOCAÇÃO"*, e prescreve a cura — *"um `StableId`
//! de verdade atribuído no spawn"* — que não existia. O que existia era o
//! [`crate::stable_name_id`], hash FNV-1a do `Name`, e o mesmo comentário avisa: *"**Renomear
//! um objeto muda o id dele**, e portanto desliga o que apontava para ele"*.
//!
//! Duas famílias dependem disso hoje e **quebram ao renomear**: os bindings da timeline
//! (`WireId`) e as referências da física (`PhysicsJoint.body_a/b`, `PulleyWheel.rope/.body`).
//! É por isso que a cópia de um ragdoll prende os corpos do MESTRE: a cópia recebe o nome
//! `" (1)"`, e o hash muda com ele.
//!
//! # As quatro invariantes (ADR-0164 §2.7 item 6)
//!
//! 1. **Toda entidade que o snapshot CAPTURA tem um** — o critério é ter
//!    [`crate::Transform`] **ou** `ChildOf`. ⚠️ **Não** é *"ter `Transform`"*, e a diferença
//!    custou um gate vermelho: os filhos de uma peça 3D não o têm (ver a nota dentro de
//!    [`assign_missing_stable_ids`]). O critério é o do CONSUMIDOR — o que a DFS do snapshot
//!    alcança —, e não o de um irmão que responde a outra pergunta.
//! 2. **Único por documento** — gate com prova de mutação.
//! 3. **Nunca reusado**, e o contador é **monotónico e vive FORA do `ProjectState`**: um undo
//!    não o pode rebobinar, senão um *redo* reusaria um id que ainda está vivo na pilha.
//! 4. **`0` é reservado** para *"nenhum"* — a mesma convenção do `stable_name_id`, do
//!    `PaintedDoc` e do `BakedForm`.
//!
//! # ⚠️ Por que uma VARREDURA idempotente, e não um hook `on_add`
//!
//! O ADR escreve *"alocado num hook `on_add` de `Transform` (ou `#[require]`)"*, e a escolha
//! aqui é a terceira porta, com três razões medidas:
//!
//! - **O repo nunca usou um hook de componente** (`grep on_add|HookContext` = 0). O idioma
//!   que ele TEM para exatamente este problema é a varredura idempotente do
//!   [`crate::assign_missing_root_order`], escrita para a mesma classe de defeito
//!   (`to_bits` a decidir o que devia ser dado).
//! - ⚠️ **Um hook colide com o `snapshot_to_world`.** O restore insere os componentes em
//!   ordem de `type_id` (blake3), então o `Transform` pode chegar ANTES do blob de
//!   `StableId`: o hook alocaria um id novo que o blob logo sobrescreveria — gastando o
//!   contador uma vez por entidade por restore, e tornando-o função do **número de undos**.
//!   A varredura não vê essas entidades: elas já chegam com id.
//! - **O determinismo é o mesmo**, e é o que o repo já garante: a ordem de alocação é função
//!   da sequência de gestos, que é a premissa sob a qual o `world_to_snapshot` promete
//!   *"byte output is invariant given the same spawn sequence"*.
//!
//! # ⚠️ MEDIDO nesta wave: `to_bits()` NÃO é a ordem de criação no bevy 0.18
//!
//! A 1.ª versão desta varredura copiou o `sort_unstable_by_key(|e| e.to_bits())` do
//! [`crate::assign_missing_root_order`] a acreditar que ele congelava a ordem de SPAWN. Três
//! entidades criadas em sequência saíram com os ids **`3, 2, 1`** — o `to_bits` do 0.18
//! **inverte** a ordem de criação (a codificação guarda o índice de forma a caber o nicho de
//! `Option<Entity>`).
//!
//! Para o `RootOrder` isso é indiferente e o doc-comment dele continua certo, porque ele e a
//! árvore usam **a mesma** chave — o que importa lá é concordarem. Aqui não: o id é lido por
//! humanos e pela migração, e *o primeiro objeto criado ter o número mais alto* é uma
//! surpresa gratuita. Esta varredura ordena por [`Entity::index`], que **é** ascendente com o
//! spawn, e há gate a fixá-lo. ⛔ Não o troque de volta por `to_bits` "para ficar igual ao
//! irmão": eles respondem a perguntas diferentes.
//!
//! ⛔ **O preço, nomeado:** uma entidade criada e capturada no MESMO quadro, antes de a
//! varredura correr, não teria id. Por isso ela roda no passe de quadro **imediatamente ao
//! lado do `assign_missing_root_order`**, que já existe por essa razão e antes da captura do
//! fim do quadro — e há gate a fixar essa ordem.

use bevy_ecs::component::Component;
use bevy_ecs::hierarchy::ChildOf;
use bevy_ecs::prelude::{Entity, Or, Resource, With, Without, World};
use serde::{Deserialize, Serialize};

/// A identidade durável de um objeto. Opaca: **não** derive significado do número, e
/// **nunca** o guarde num campo que o utilizador edite.
///
/// ⚠️ Ao contrário do `Entity::to_bits()`, ele sobrevive ao respawn do undo — que é a
/// propriedade inteira pela qual esta wave existe.
/// ⛔ **NÃO deriva `Default`, e a ausência é a cerca.** Um `StableId::default()` seria
/// `StableId(0)` — o id **reservado para «nenhum»** —, e uma entidade que o recebesse ficaria
/// com um id sem significado que a varredura **nunca corrigiria**, porque ela pergunta
/// `Without<StableId>` e o componente estaria lá. Sem `Default`, esse estado não é
/// construível por acidente: o registo entra por `register` (e não `register_default`), e o
/// censo da shell confirma que ele não é oferecido em paleta nenhuma.
#[derive(
    Component, Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct StableId(pub u64);

impl StableId {
    /// `0` = *nenhum*. Reservado, nunca alocado — a mesma convenção do
    /// [`crate::stable_name_id`], do `PaintedDoc` e do `BakedForm`.
    pub const NONE: StableId = StableId(0);

    /// O primeiro id que a alocação pode entregar.
    pub const FIRST: u64 = 1;

    #[must_use]
    pub const fn is_none(self) -> bool {
        self.0 == 0
    }
}

/// **O contador do documento** — o próximo id livre.
///
/// ⚠️ **Vive fora do `ProjectState`** e por isso **fora do undo** (ADR-0164 §2.7 item 6). Um
/// undo que o rebobinasse faria um *redo* entregar um id que ainda está vivo na pilha, e duas
/// entidades com o mesmo `StableId` é a corrupção que todos os gates desta wave existem para
/// tornar impossível.
///
/// Persistido em `ProjectFile.stable_id_counter`. É **monotónico por construção**: a única
/// escrita é [`Self::reconcile_at_least`], que nunca desce.
#[derive(Resource, Copy, Clone, Debug, PartialEq, Eq)]
pub struct StableIdCounter(u64);

impl Default for StableIdCounter {
    fn default() -> Self {
        Self(StableId::FIRST)
    }
}

impl StableIdCounter {
    #[must_use]
    pub const fn new(next: u64) -> Self {
        // `0` é reservado: um contador semeado a zero entregaria o id "nenhum".
        Self(if next < StableId::FIRST {
            StableId::FIRST
        } else {
            next
        })
    }

    /// O valor a persistir.
    #[must_use]
    pub const fn next_free(self) -> u64 {
        self.0
    }

    /// Sobe o contador até `at_least`, **nunca o desce**.
    ///
    /// É a porta que torna a monotonicidade uma propriedade do tipo e não uma promessa: um
    /// load que traga um ficheiro com o contador atrasado (editado à mão, ou de um branch
    /// antigo) é reconciliado contra os ids que o mundo de facto tem, e o resultado ainda é
    /// seguro.
    pub const fn reconcile_at_least(&mut self, at_least: u64) {
        if at_least > self.0 {
            self.0 = at_least;
        }
    }

    fn take(&mut self) -> StableId {
        let id = self.0;
        self.0 = self.0.saturating_add(1);
        StableId(id)
    }
}

/// **Dá um `StableId` a toda entidade editável que ainda não tem um.** Idempotente: rodar de
/// novo é no-op, e devolve `false`.
///
/// Gémea de [`crate::assign_missing_root_order`], com a mesma razão (ver o cabeçalho do
/// módulo) e **uma chave diferente**: as sem-id recebem números na ordem de
/// [`Entity::index`], que é a ordem de SPAWN — ⚠️ e **não** a de `to_bits()`, que o bevy 0.18
/// entrega invertida (medido; há gate). É a mesma premissa de determinismo que o
/// `world_to_snapshot` já assume: *"byte output is invariant given the same spawn sequence"*.
///
/// ⚠️ **Reconcilia o contador contra o mundo ANTES de alocar.** Sem isto, um documento
/// carregado cujo contador venha atrasado entregaria um id que já está vivo — e a unicidade
/// é a invariante de que tudo o resto depende.
pub fn assign_missing_stable_ids(world: &mut World) -> bool {
    // 1. O contador nunca pode estar atrás do mundo.
    let highest = world
        .query::<&StableId>()
        .iter(world)
        .map(|s| s.0)
        .max()
        .unwrap_or(0);
    let mut counter = world
        .remove_resource::<StableIdCounter>()
        .unwrap_or_default();
    counter.reconcile_at_least(highest.saturating_add(1));

    // 2. Quem ainda não tem.
    //
    // ⚠️⚠️ **O critério é `Transform` OU `ChildOf`, e a segunda metade custou um gate
    // vermelho.** A 1.ª versão perguntava só `With<Transform>`, seguindo o `RootOrder` e a
    // frase que o `undo.rs` repete há meses — *"toda entidade editável tem `Transform`
    // (sprites, formas, objetos Flip, grupos)"*. **Essa frase envelheceu**: desde o módulo de
    // modelagem 3D (ADR-0161) os FILHOS de uma peça não têm `Transform` — o `spawn_doc`
    // declara-o, *"a raiz recebe `FieldObject`, `Transform` e `RootOrder`; os filhos recebem
    // só o que é deles: nome, forma e pose"*.
    //
    // Sem id, a linha deles saía com `StableId::NONE` — **todas com o mesmo** —, o mapa
    // `id → entidade` do restore colapsava-as numa só, e uma peça de 5 nós voltava com 2.
    //
    // A regra certa é a do CONSUMIDOR: o snapshot captura o que a DFS alcança, e a DFS parte
    // das raízes (que têm `Transform`) e desce por `Children` — logo tudo o que ela visita ou
    // tem `Transform` ou tem `ChildOf`. ⛔ Não estreite isto de volta a `Transform` "para
    // ficar igual ao `RootOrder`": aquele responde *"que raízes ordenar?"*, este responde
    // *"o que é que o ficheiro guarda?"*.
    let mut missing = world
        .query_filtered::<Entity, (
            Without<StableId>,
            Or<(With<crate::Transform>, With<ChildOf>)>,
        )>()
        .iter(world)
        .collect::<Vec<_>>();
    let changed = !missing.is_empty();
    // A ordem de SPAWN — congelá-la é o que faz a atribuição ser função do gesto, e não da
    // ordem em que o archetype por acaso lista as entidades.
    // ⚠️ `index()`, **não** `to_bits()`: medido nesta wave, o `to_bits` do bevy 0.18 sai
    // INVERTIDO em relação à criação (ver o cabeçalho do módulo).
    missing.sort_unstable_by_key(|e| e.index());
    for e in missing {
        let id = counter.take();
        world.entity_mut(e).insert(id);
    }
    world.insert_resource(counter);
    changed
}

/// O `StableId` de uma entidade, se ela tiver um.
#[must_use]
pub fn stable_id_of(world: &World, entity: Entity) -> Option<StableId> {
    world.get::<StableId>(entity).copied()
}

/// ⭐ **O id do objeto que se chama `name` — a resolução de AUTORIA.**
///
/// # Esta função é a cura, e não uma conveniência
///
/// Antes desta wave uma junta guardava `stable_name_id(nome)` — o **hash do nome** — e a ponte
/// da física reconstruía o mapa `hash → entidade` **a cada dispatch**. Isso põe a resolução no
/// RUNTIME, e é de lá que vêm os dois defeitos que o `name.rs` já documentava:
///
/// - **Renomear desliga.** O hash muda, a junta deixa de achar o corpo, e nada avisa.
/// - **Copiar prende no original.** A cópia recebe o nome `" (1)"`, o hash muda, e a junta da
///   cópia continua a apontar para os corpos do MESTRE (ADR-0164 — é o defeito que a wave da
///   instância existe para curar).
///
/// A cura não é um hash melhor: é **resolver uma vez, no momento em que o artista aponta para
/// o objeto**, e guardar a identidade. O nome é para humanos, na hora de autorar; o id é para
/// o documento. Depois disto, renomear é só renomear.
///
/// ⚠️ **`0` (o reservado) quando ninguém se chama assim** — a mesma resposta que um hash sem
/// dono dava, e ela continua a significar *"não aponta para nada"*.
///
/// ⚠️ **Nomes não são únicos pelo tipo** (o `Name` documenta-o; a unicidade é imposta pelo
/// editor, em `name_unique.rs`). Com dois homónimos esta função devolve o de **menor
/// `StableId`** — o mais antigo —, que é determinístico e não *"o que o archetype listou
/// primeiro"*. ⛔ Não é uma escolha boa, é uma escolha REPRODUTÍVEL: quem depende de a
/// resolução acertar tem de garantir o nome único, e o editor garante.
#[must_use]
pub fn stable_id_for_name(world: &mut World, name: &str) -> u64 {
    assign_missing_stable_ids(world);
    world
        .query::<(&crate::Name, &StableId)>()
        .iter(world)
        .filter(|(n, _)| n.as_str() == name)
        .map(|(_, s)| s.0)
        .min()
        .unwrap_or(StableId::NONE.0)
}

/// A entidade que carrega este `StableId`, se existir.
///
/// ⚠️ Varredura linear, e é de propósito **não** haver um índice: um mapa
/// `StableId → Entity` seria estado derivado a manter coerente com o mundo, e o repo já
/// registou o preço disso (as pontes `VecPathRef`/`FlipObjectRef` precisam de `rebuild_map`
/// depois de todo restore). Quem precisa de resolver muitos ids num quadro constrói o mapa
/// uma vez e passa-o adiante — a F4 fá-lo-á para o sync.
#[must_use]
pub fn entity_of_stable_id(world: &mut World, id: StableId) -> Option<Entity> {
    if id.is_none() {
        return None;
    }
    world
        .query::<(Entity, &StableId)>()
        .iter(world)
        .find(|(_, s)| **s == id)
        .map(|(e, _)| e)
}

#[cfg(test)]
#[path = "stable_id_tests.rs"]
mod tests;
