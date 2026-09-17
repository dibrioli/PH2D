//! ⭐⭐⭐ **A PONTE entre a CENA da escultura e a árvore do editor** — irmã de
//! [`crate::flip::entities`] (o Flip) e [`crate::vec_entities`] (o vetor), e pela mesma lei.
//!
//! Cada peça da cena 3D tem uma entidade ECS que a referencia
//! ([`ph2d_ecs::Sculpt3dPieceRef`]). A cena é dona da **geometria** (a pilha de multires, a
//! pose, o pincel); a entidade é dona da **identidade e do lugar na árvore** — nome,
//! visibilidade, trava, pai, ordem. É isso que faz a Hierarquia **listar** o mesh e os verbos
//! de linha (apagar, duplicar, renomear, agrupar) alcançarem-no sem uma lei nova.
//!
//! # O report que a abriu
//!
//! Enio, 2026-09-04: *«implemente o mesh na Hierarchy pois ele ainda não aparece lá. Implemente
//! as funções na hierarquia para o mesh como del, duplicate, etc»*.
//!
//! # ⭐⭐⭐ Por que o estado guardado é um CONJUNTO DE IDS, e não um mapa de bits
//!
//! As irmãs guardam `id → bits da entidade`, e por isso **precisam de um `rebuild_map` a seguir
//! a cada restore**: o undo respawna tudo com bits novos, e um mapa velho aponta para entidades
//! mortas. ⛔ Aqui isso seria pior que uma lista errada — o plano leria *«a entidade sumiu»* e
//! **apagaria a escultura do artista a cada Ctrl+Z**.
//!
//! ⇒ o mundo é lido a cada quadro (quem tem `Sculpt3dPieceRef` **é** o mapa), e o que se guarda
//! é só *«que peças já tiveram linha»* — o único facto que o mundo não sabe, e o que distingue
//! **a entidade que o artista apagou** de **a peça que ainda não tem linha**. *Um restore não
//! pode depender de alguém se lembrar de o chamar.*
//!
//! # ⭐⭐ A lei é PURA, e por isso tem gate
//!
//! Uma `Sculpt3dScene` pede `device`, superfície e viewport; nada disso existe num teste. ⇒ o
//! que decide mora em [`plan`], que só vê **ids**, e o executor abaixo é o braço curto que a
//! aplica. *Uma lei escrita dentro de um `impl App` só se pode gatear por texto, e um gate
//! textual não distingue a chamada viva da chamada atrás de um `if false`.*

use std::collections::{BTreeMap, BTreeSet};

use ph2d_ecs::{Entity, Name, RootOrder, Sculpt3dPieceRef, SimWorld, Transform};

use super::{ObjectId, Sculpt3dScene};

/// **As peças que já tiveram linha na Hierarquia** — o (pouco) estado que esta ponte guarda.
pub(crate) type SculptRowsSeen = BTreeSet<u32>;

/// `ObjectId.0` → `Entity::to_bits()`, **lido do mundo** a cada quadro.
pub(crate) type SculptEntityMap = BTreeMap<u32, u64>;

/// **O que uma sincronia tem de fazer** — a saída de [`plan`].
#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct SyncPlan {
    /// A entidade sumiu (o `Delete` da Hierarquia) ⇒ a peça vai junto.
    pub(crate) remove_pieces: Vec<u32>,
    /// A peça sumiu (o `Delete` do canvas) ⇒ a entidade vai junto.
    pub(crate) despawn: Vec<u64>,
    /// Peça sem linha, e que nunca teve ⇒ entidade nova.
    pub(crate) spawn: Vec<u32>,
}

impl SyncPlan {
    /// Nada a fazer — o caso normal de todo quadro em que ninguém criou nem apagou.
    pub(crate) fn is_empty(&self) -> bool {
        self.remove_pieces.is_empty() && self.despawn.is_empty() && self.spawn.is_empty()
    }
}

/// ⭐⭐⭐ **A LEI, pura.** `pieces` são as peças da cena, `world` é quem tem
/// [`Sculpt3dPieceRef`] **agora**, e `seen` são as peças que **já tiveram** linha.
///
/// | tem entidade? | já teve? | o que isso quer dizer | o que fazer |
/// |---|---|---|---|
/// | sim | — | em dia | nada |
/// | não | **sim** | o artista apagou a LINHA | apagar a peça |
/// | não | não | a peça é nova | criar a linha |
/// | *(entidade sem peça)* | — | o artista apagou a PEÇA (no canvas) | despawnar a linha |
pub(crate) fn plan(pieces: &[u32], world: &SculptEntityMap, seen: &SculptRowsSeen) -> SyncPlan {
    let mut out = SyncPlan::default();
    for &id in pieces {
        if world.contains_key(&id) {
            continue;
        }
        if seen.contains(&id) {
            out.remove_pieces.push(id);
        } else {
            out.spawn.push(id);
        }
    }
    for (id, &bits) in world {
        if !pieces.contains(id) {
            out.despawn.push(bits);
        }
    }
    out
}

/// Reconcilia a cena e a árvore. Chamado **uma vez por quadro**, antes de a Hierarquia ser
/// desenhada, para o quadro ver um estado consistente.
pub(crate) fn sync(
    sim: &mut SimWorld,
    scene: &mut Sculpt3dScene,
    seen: &mut SculptRowsSeen,
    // ⚠️⚠️ **O que esta função PRECISA é um NOME, não um gerador de nomes** — e ela recebe o
    // mintador em vez do `String` pronto porque a unicidade depende do mundo **enquanto ele
    // muta**: a segunda peça de um mesmo quadro tem de ver a primeira já nascida.
    //
    // ⛔ **A porta que o deriva NÃO é desta linha.** O `name_unique` é uma das três folhas
    // partilhadas da shell que a batedora da `line/app-physics` nomeou como alvo de **linha
    // própria** (11/38/14 consumidores de famílias diferentes), e reimplementá-la aqui seria
    // a segunda resposta a *«que nome é livre?»* — exactamente o empate de IDENTIDADE que a
    // porta existe para impedir desde a W4.T6. *Quem a possui passa-a.*
    nome_novo: &mut dyn FnMut(&mut SimWorld) -> String,
) {
    let pieces: Vec<u32> = scene.objects.iter().map(|o| o.id.0).collect();
    let world = world_map(sim);
    let plano = plan(&pieces, &world, seen);
    if !plano.is_empty() {
        for id in plano.remove_pieces {
            scene.delete_by_id(ObjectId(id));
        }
        for bits in plano.despawn {
            if let Ok(e) = sim.world_mut().get_entity_mut(Entity::from_bits(bits)) {
                e.despawn();
            }
        }
        if !plano.spawn.is_empty() {
            let mut next_order = next_root_order(sim);
            for id in plano.spawn {
                // ⚠️ **Pela porta do nome ÚNICO** — a mesma que o Flip e o vetor usam: desde o
                // W4.T6 um empate de nome é um empate de IDENTIDADE.
                let name = nome_novo(sim);
                sim.world_mut().spawn((
                    Transform::default(),
                    Name::new(name),
                    Sculpt3dPieceRef(id),
                    RootOrder(next_order),
                ));
                next_order = next_order.saturating_add(1);
            }
        }
    }
    // ⚠️ **O `seen` é REDERIVADO do mundo**, e não incrementado: uma linha que o undo devolveu
    // volta a contar, e uma que saiu de vez deixa de contar. *Um contador que só cresce era o
    // que fazia a segunda leitura discordar da primeira.*
    *seen = world_map(sim).keys().copied().collect();
}

/// **Quem tem [`Sculpt3dPieceRef`] agora** — o mapa é o MUNDO, não uma cópia dele.
#[must_use]
pub(crate) fn world_map(sim: &mut SimWorld) -> SculptEntityMap {
    let mut map = SculptEntityMap::new();
    let mut q = sim.world_mut().query::<(Entity, &Sculpt3dPieceRef)>();
    for (e, piece) in q.iter(sim.world()) {
        map.insert(piece.0, e.to_bits());
    }
    map
}

/// ⭐⭐⭐ **QUEM ESTÁ COM O OLHO FECHADO NA HIERARQUIA** — o espelho que a
/// [`Sculpt3dScene::escondidas`] guarda, **lido do mundo** como o [`world_map`]
/// ao lado.
///
/// ⚠️ **`Option<&Visibility>` e não `&Visibility`**, e é a invariante HR-5 da
/// casa: *ausência do componente = visível*. Uma peça acabada de nascer não o
/// tem, e um query obrigatório deixaria o espelho vazio — lendo-se como *«nada
/// está escondido»*, que é o valor conservador e por isso invisível.
///
/// ⚠️ **PER-ENTIDADE, e o limite é INVARIANTE DA CASA e não dívida desta crate:**
/// o `ph2d_ecs::Visibility` **não propaga para descendentes** — está escrito no
/// próprio componente (*«absence of the component equals visible»*, HR-5) e
/// **nada no repo anda a árvore para o resolver**, medido em 2026-09-17.
///
/// ⛔⛔ **A redacção anterior desta nota dizia que a cura «é andar os
/// ascendentes, que é o que a `ph2d_entity_visibility` faz para outro meio», e
/// isso é FALSO:** aquela crate resolve o caso da RECEITA com uma marca
/// **derivada** (`MasterPiece`, re-carimbada por quadro em toda a descendência)
/// — precisamente **porque** escrever `Visibility` nas peças estaria errado: a
/// `Visibility` de uma peça é **autoria** e propaga para as instâncias, logo
/// toda cópia nasceria invisível. *O doc dela escreve o argumento por extenso.*
///
/// ⇒ **esconder um grupo e esperar que as peças dele sumam é uma pergunta de
/// PRODUTO que atravessa TODOS os meios** (sprite · vector · flip · escultura),
/// e curá-la só aqui faria a escultura divergir do resto do app em silêncio.
/// *Decisão do dono, com ADR — nunca uma linha nesta função.*
#[must_use]
pub(crate) fn escondidas_do_mundo(sim: &mut SimWorld) -> BTreeSet<ObjectId> {
    let mut fechadas = BTreeSet::new();
    let mut q = sim
        .world_mut()
        .query::<(&Sculpt3dPieceRef, Option<&ph2d_ecs::Visibility>)>();
    for (piece, vis) in q.iter(sim.world()) {
        if vis.is_some_and(|v| v.hidden) {
            fechadas.insert(ObjectId(piece.0));
        }
    }
    fechadas
}

/// O próximo `RootOrder` livre (o maior em uso + 1) — idêntico ao do Flip e do vetor, porque a
/// árvore é **uma** e a ordem de raiz é partilhada entre os meios.
fn next_root_order(sim: &mut SimWorld) -> u32 {
    let mut q = sim.world_mut().query::<&RootOrder>();
    let max = q
        .iter(sim.world())
        .map(|r| r.0)
        .filter(|&o| o != u32::MAX)
        .max();
    max.map_or(0, |m| m.saturating_add(1))
}

impl Sculpt3dScene {
    // ⚠️ **O `index_of` já existia** (`space.rs`) e é o que estes dois verbos usam —
    // a 1.ª redacção deste ficheiro escreveu-o outra vez, e o compilador acusou. *A lei de
    // «onde está a peça `id`» é uma só.*

    /// ⭐ **APAGA a peça `id`** — o mesmo verbo do [`Sculpt3dScene::delete_active`], endereçado.
    ///
    /// ⚠️ **A peça ACTIVA é restaurada**, e isso não é cosmética: apagar uma linha da Hierarquia
    /// não pode mover a mão do artista para outra peça. Só quando a apagada **era** a activa é
    /// que a mão muda — e aí ela vai para onde o `delete_active` a põe.
    pub(super) fn delete_by_id(&mut self, id: ObjectId) -> bool {
        let Some(i) = self.index_of(id) else {
            return false;
        };
        let antes = self.active;
        self.active = i;
        let ok = self.delete_active();
        if ok && antes != i {
            // A lista encolheu em `i`: quem estava depois desceu uma casa.
            let novo = if antes > i { antes - 1 } else { antes };
            self.active = novo.min(self.objects.len().saturating_sub(1));
        }
        ok
    }

    /// ⭐ **DUPLICA a peça `id`** e devolve o id da cópia — o mesmo verbo do
    /// [`Sculpt3dScene::duplicate_active`], endereçado.
    ///
    /// ⚠️ **A cópia fica ACTIVA**, ao contrário do apagar: é o que o `duplicate_active` já faz,
    /// e é o que a Hierarquia faz com a linha nova (o artista acabou de pedir a cópia).
    pub(super) fn duplicate_by_id(&mut self, id: ObjectId) -> Option<ObjectId> {
        let i = self.index_of(id)?;
        self.active = i;
        if !self.duplicate_active() {
            return None;
        }
        // ⚠️ **A cópia é a ÚLTIMA da lista** — é onde o `duplicate_active` a põe.
        self.objects.last().map(|o| o.id)
    }
}

/// ⭐⭐⭐ **A sincronia do quadro** — ver [`sync`]. Sem cena armada (o caso normal de quem
/// nunca abriu o módulo) ela sai no primeiro `let ... else`.
pub fn entities_sync(
    sim: &mut SimWorld,
    scene: &mut Sculpt3dScene,
    shell: &mut crate::Sculpt3dShellState,
    // ⚠️ **A selecção da Hierarquia é um FACTO da moldura, colhido antes** — ela mora no
    // `hero_screen`, que é da shell. *Pedir uma porta para «quem está seleccionado?» seria
    // pedir uma capacidade onde o que atravessa é um `u64`.*
    hier_sel: Option<u64>,
    // ⚠️ **O nome único chega PRONTO** — ver [`sync`].
    nome_novo: &mut dyn FnMut(&mut SimWorld) -> String,
) {
    let pedido = shell.dup.take();
    // ⭐⭐⭐ **O *Duplicate* da Hierarquia, e ele corre ANTES do plano** — senão o plano veria
    // uma peça nova sem linha e spawnaria uma SEGUNDA, ao lado da cópia que o artista pediu.
    if let Some((origem, copia)) = pedido
        && let Some(nova) = scene.duplicate_by_id(ObjectId(origem))
        && let Ok(mut e) = sim.world_mut().get_entity_mut(Entity::from_bits(copia))
    {
        e.insert(Sculpt3dPieceRef(nova.0));
    }
    sync(sim, scene, &mut shell.rows, nome_novo);
    // ⭐⭐⭐ **E O OLHO DA HIERARQUIA CHEGA À CENA** — a espec §6.1 diz que um
    // alvo ESCONDIDO não conta, e antes disto o pen-down não tinha como saber.
    //
    // ⚠️ **Aqui e não numa porta do `AppHost`:** este é o único sítio da família
    // que segura o mundo e a cena ao mesmo tempo, e ele já corre uma vez por
    // quadro **antes** de a Hierarquia ser desenhada. Uma sétima porta no host
    // reprovaria o censo por licença que a Fase B pagou.
    //
    // ⚠️ **DEPOIS do `sync`**, que é quem faz nascer e morrer peças: lido antes,
    // o espelho descreveria uma lista que o próprio quadro acabou de mudar.
    scene.escondidas = escondidas_do_mundo(sim);
    // ⭐⭐⭐ **ESCOLHER A LINHA PÕE A MÃO NA PEÇA** — a outra metade de *«as funções na
    // hierarquia para o mesh»*.
    //
    // ⚠️ **Só na MUDANÇA da selecção**, e isso não é optimização: a peça activa tem DOIS
    // escritores — a linha escolhida e o `aim` do pen-down (mirar uma peça torna-a activa).
    // Aplicar a cada quadro faria a linha ganhar sempre, e esculpir noutra peça seria
    // impossível enquanto a Hierarquia tivesse uma seleccionada. *É a mesma lei do
    // `vec_selection`: quem MUDOU neste quadro é quem manda.*
    let sel = hier_sel;
    if sel != shell.sel {
        shell.sel = sel;
        if let Some(bits) = sel
            && let Some(piece) = sim.world().get::<Sculpt3dPieceRef>(Entity::from_bits(bits))
            && let Some(i) = scene.index_of(ObjectId(piece.0))
        {
            scene.active = i;
        }
    }
}

#[cfg(test)]
#[path = "entities_tests.rs"]
mod tests;
