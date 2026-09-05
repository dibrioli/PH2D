//! ⭐⭐⭐ **AS VARIANTES, e a TROCA que preserva as excepções** (ADR-0164 / plano F5, critério 2).
//!
//! # O que uma variante é aqui — e porque não custou schema nenhum
//!
//! Uma **variante derivada** (o modelo Unity / flecs `IsA`) é *um mestre cuja raiz é ela própria
//! uma instância de outro mestre*: `MasterRoot` **e** [`InstanceOf`] na mesma entidade. O sync não
//! precisou de uma linha para isso — [`crate::instance_sync`] procura *toda* entidade cujo elo
//! aponta para um `MasterRoot` vivo, e uma variante é exactamente isso.
//!
//! ⭐ **Medido por sonda antes de se escrever código** (2026-08-27): editar a base leva a alteração
//! à variante **e às instâncias da variante** num passe só (`passe = 2`, duas escritas). A ordem
//! topológica sai de graça pela mesma coincidência que a F5.1 nomeou — a ordem de criação é a de
//! dependência, e `live_instances` ordena por `StableId`.
//!
//! # ⭐⭐ O MAPA de re-key já vive no mundo: são os próprios elos
//!
//! A tabela do doc 04 §2.6 promete *«trocar variant↔base preserva os overrides (re-key
//! determinístico)»*, e a pesquisa do endereçamento chama-lhe *«a operação que nenhum outro sistema
//! consegue, porque nenhum tem chave mestre-relativa com caminho»*. Aqui ela é mais barata do que a
//! pesquisa previa, e a razão é a tese do ADR-0164: **a instância é uma entidade real**, então as
//! peças da variante *já dizem* de que peça da base nasceram.
//!
//! ```text
//! sonda 2026-08-27:  sid peca da base = 2 | sid peca da variante = 4 | link da peca da variante = 2
//! ```
//!
//! ⇒ `base 2 → variante 4` lê-se invertendo os elos da variante. **Sem nomes, sem caminhos, sem
//! heurística** — é a diferença entre este re-key e o `ByName`/`ByHierarchy` do Unity, que o próprio
//! Unity documenta como não-reproduzível com nomes duplicados (HR-5).
//!
//! # A troca é um RE-KEY, e o resto já estava construído
//!
//! [`swap`] muda três coisas — o elo da raiz, os elos das peças e as chaves de override — e **para
//! aí**. Quem materializa as peças que o mestre novo tem a mais, quem apaga as que ele não tem, e
//! quem guarda a excepção de uma peça que morre ([`ObjectInstance::orphans`]) já é o passe
//! estrutural da F5.1 + F5.3. *A peça que falta pode já estar construída — meça a estrutura dela
//! primeiro.*
//!
//! ⭐ E é isso que faz a troca ser **reversível de graça**: trocar para uma variante que não tem a
//! peça `X` sepulta a excepção de `X` nos órfãos; trocar de volta materializa `X` e **exuma-a**.
//!
//! # ⛔ Mestre NÃO aparentado: `None`, e é uma decisão
//!
//! Sem antepassado comum não há mapa — só heurística por nome ou por caminho. Os três modos do
//! Unity (`Nenhum` · `Por nome` · `Por hierarquia`) são um gesto próprio, com relatório, e **nunca
//! automáticos** (doc 04 §2.6). Aqui a resposta é a recusa, e ela é honesta: a alternativa seria
//! aplicar a excepção do artista a um objeto que calhou de ter o mesmo nome.

use ph2d_ecs::{Children, Entity, InstanceOf, MasterRoot, ObjectInstance, StableId};
use std::collections::{BTreeMap, BTreeSet};

use crate::instance_swap_match::WhenUnrelated;

use ph2d_ecs::SimWorld;

/// **Porque uma troca não aconteceu.**
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum SwapRefusal {
    /// A entidade clicada não é a raiz de uma instância.
    NotAnInstance,
    /// O alvo não é um `MasterRoot` vivo.
    NotAMaster,
    /// Já é esse o mestre — o gesto não tem o que fazer.
    Already,
    /// ⛔ Sem antepassado comum: não há mapa determinístico. Ver o cabeçalho.
    Unrelated,
    /// ⛔⛔⛔ **O alvo é a PRÓPRIA raiz** — uma variante nunca pode ser mestre de si mesma
    /// (auditoria multiagêntica de 2026-08-31, achado P0).
    ///
    /// Sem esta recusa, `swap(root=variante, id=sid da variante)` escrevia
    /// `InstanceOf {{ master: <o próprio sid> }}` e re-chaveava as peças para elos-a-si-mesmas: a
    /// derivação base→variante era **cortada em silêncio**, a variante saía da família, e o estado
    /// era estável (o `follow` seguinte assentava sobre a corrupção). ⚠️ E a porta era um CLIQUE —
    /// seleccionar a linha da variante na Hierarquia bastava, via o `follow` de então.
    ItselfAsMaster,
}

/// **O que a troca fez**, para a voz do gesto.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct SwapReport {
    /// Peças cujo elo foi traduzido para o mestre novo.
    pub(crate) rekeyed: usize,
    /// Peças sem correspondência no mestre novo — o passe estrutural apaga-as, e a excepção de
    /// cada uma fica nos órfãos.
    pub(crate) dropped: usize,
    /// Chaves de override que sobreviveram à tradução.
    pub(crate) overrides_kept: usize,
    /// ⭐ **Caminhos que apareceram mais que uma vez** e por isso não emparelharam — só o
    /// emparelhamento SEM parentesco o produz (o mapa derivado sai dos elos, e um elo não é
    /// ambíguo). ⛔ Escrito, nunca engolido: é a metade do relatório que diz *porquê* uma excepção
    /// que o artista esperava não veio.
    pub(crate) ambiguous: usize,
}

/// **A ASCENDÊNCIA de um mestre** — ele, a base dele, a base da base…, da mais interna para a mais
/// externa.
///
/// ⚠️ **Com conjunto de visitados**, ao contrário da travessia de `ChildOf` da casa: ali a ausência
/// de guarda é deliberada (o reparent recusa ciclos, e uma segunda defesa esconderia a primeira);
/// aqui o elo é um `u64` que um ficheiro pode trazer corrompido, e um laço infinito no passe de
/// quadro é a janela do artista a congelar.
///
/// ⚠️ **Sem porta pública própria** — a 1.ª versão tinha um `chain(sim, id)` que construía o índice
/// e chamava esta; ele nasceu **sem chamador**, e o `dead_code` disse-o. *Uma conveniência que
/// ninguém usa é uma segunda porta à espera de divergir da que se usa.*
fn chain_with(sim: &SimWorld, by_id: &Index, master_id: u64) -> Vec<u64> {
    let mut out = Vec::new();
    let mut seen = BTreeSet::new();
    let mut id = master_id;
    while seen.insert(id) {
        out.push(id);
        let Some(&e) = by_id.get(&id) else { break };
        let Some(link) = sim.world().get::<InstanceOf>(e) else {
            break;
        };
        // ⚠️ Só sobe por elo para um mestre VIVO: um elo para uma entidade comum não é uma base,
        // é lixo de um mestre apagado.
        match by_id.get(&link.master) {
            Some(&b) if sim.world().get::<MasterRoot>(b).is_some() => id = link.master,
            _ => break,
        }
    }
    out
}

/// **`peça do mestre `from`` → `peça do mestre `to``**, ou `None` sem antepassado comum.
///
/// ⚠️ **Parcial de propósito.** Uma peça que a variante acrescentou não existe na base, e uma peça
/// que a base tem e a variante perdeu não existe na variante. A ausência de imagem é a informação:
/// é ela que manda o passe estrutural apagar a peça e sepultar a excepção dela.
#[must_use]
pub(crate) fn piece_map(sim: &mut SimWorld, from: u64, to: u64) -> Option<BTreeMap<u64, u64>> {
    let by_id = stable_index(sim);
    piece_map_with(sim, &by_id, from, to)
}

fn piece_map_with(sim: &SimWorld, by_id: &Index, from: u64, to: u64) -> Option<BTreeMap<u64, u64>> {
    let up_from = chain_with(sim, by_id, from);
    let up_to = chain_with(sim, by_id, to);
    // O antepassado comum mais PRÓXIMO de `from`: a primeira base dele que `to` também tem.
    let common = *up_from.iter().find(|id| up_to.contains(id))?;

    // ── `from` → antepassado ─────────────────────────────────────────────────────────────────
    //
    // ⚠️ Quando `from` **é** o antepassado (trocar a base por uma variante dela) a subida não tem
    // um degrau, e a composição tem de começar na IDENTIDADE — não num mapa vazio, que
    // aniquilaria tudo o que viesse a seguir.
    let climb: Vec<u64> = up_from
        .iter()
        .copied()
        .take_while(|&id| id != common)
        .collect();
    let mut map = identity(sim, by_id, from);
    for step in climb {
        let up = step_up(sim, by_id, step);
        map = map
            .into_iter()
            .filter_map(|(k, v)| up.get(&v).map(|&w| (k, w)))
            .collect();
    }

    // ── antepassado → `to` ───────────────────────────────────────────────────────────────────
    //
    // Descer é subir ao contrário: `step_up(X)` diz de que peça da base cada peça de X nasceu, e
    // invertê-lo dá a base → X. A ordem é do mais externo para o mais interno.
    let descend: Vec<u64> = up_to
        .iter()
        .copied()
        .take_while(|&id| id != common)
        .collect();
    for step in descend.into_iter().rev() {
        let down = invert(&step_up(sim, by_id, step));
        map = map
            .into_iter()
            .filter_map(|(k, v)| down.get(&v).map(|&w| (k, w)))
            .collect();
    }
    Some(map)
}

/// `peça de X` → `peça da base de X`, lendo os elos da própria sub-árvore de X.
fn step_up(sim: &SimWorld, by_id: &Index, master_id: u64) -> BTreeMap<u64, u64> {
    let Some(&root) = by_id.get(&master_id) else {
        return BTreeMap::new();
    };
    subtree(sim, root)
        .into_iter()
        .filter_map(|e| {
            let sid = sim.world().get::<StableId>(e)?.0;
            let link = sim.world().get::<InstanceOf>(e)?.master;
            Some((sid, link))
        })
        .collect()
}

/// A identidade sobre as peças de um mestre — o caso em que `from` já é o antepassado comum.
fn identity(sim: &SimWorld, by_id: &Index, master_id: u64) -> BTreeMap<u64, u64> {
    let Some(&root) = by_id.get(&master_id) else {
        return BTreeMap::new();
    };
    subtree(sim, root)
        .into_iter()
        .filter_map(|e| sim.world().get::<StableId>(e).map(|s| (s.0, s.0)))
        .collect()
}

/// ⚠️ **Um mapa de elos NÃO é injectivo por construção** — duas peças de uma variante podiam
/// nascer da mesma peça da base se alguém as duplicasse lá dentro. A inversão fica com a de
/// `StableId` MENOR, que é a escolha determinística; escolher pelos bits daria árvores diferentes
/// em máquinas diferentes.
fn invert(m: &BTreeMap<u64, u64>) -> BTreeMap<u64, u64> {
    let mut out: BTreeMap<u64, u64> = BTreeMap::new();
    for (&k, &v) in m {
        out.entry(v).and_modify(|e| *e = (*e).min(k)).or_insert(k);
    }
    out
}

/// ⭐⭐⭐ **TROCAR o mestre de uma instância, preservando as excepções.**
///
/// Ver o cabeçalho: ela re-chaveia e **para**. O passe estrutural da F5.1 materializa, apaga e
/// sepulta; o passe de valores traz os bytes do mestre novo no mesmo quadro.
///
/// ⭐⭐⭐ **`unrelated` é o que fazer quando NÃO há antepassado comum** (F5, o último critério).
/// O caminho de omissão é [`WhenUnrelated::Refuse`] — é ele que a fileira de versões passa, porque
/// ali os mestres são aparentados por construção e uma queda para heurística seria a operação
/// automática que o plano proíbe. Os três modos só chegam aqui por um item de menu que os nomeia.
///
/// ⚠️ **O mapa derivado GANHA sempre:** com parentesco, `unrelated` nem é lido. *Um modo de
/// emparelhamento é a resposta para a ausência de uma verdade, nunca uma alternativa a ela.*
pub(crate) fn swap(
    sim: &mut SimWorld,
    echo: &mut crate::instance_sync::MasterEcho,
    root: Entity,
    new_master_id: u64,
    unrelated: WhenUnrelated,
) -> Result<SwapReport, SwapRefusal> {
    let by_id: Index = stable_index(sim);
    let Some(&target) = by_id.get(&new_master_id) else {
        return Err(SwapRefusal::NotAMaster);
    };
    if sim.world().get::<MasterRoot>(target).is_none() {
        return Err(SwapRefusal::NotAMaster);
    }
    let old = sim
        .world()
        .get::<InstanceOf>(root)
        .map(|l| l.master)
        .filter(|id| {
            by_id
                .get(id)
                .is_some_and(|&m| sim.world().get::<MasterRoot>(m).is_some())
        })
        .ok_or(SwapRefusal::NotAnInstance)?;
    if old == new_master_id {
        return Err(SwapRefusal::Already);
    }
    // ⛔⛔⛔ Ver [`SwapRefusal::ItselfAsMaster`] — a cerca fica AQUI, na porta única, e não em cada
    // chamador: foi um chamador novo (o `follow`) que provou que a pergunta não estava feita.
    if sim
        .world()
        .get::<ph2d_ecs::StableId>(root)
        .is_some_and(|s| s.0 == new_master_id)
    {
        return Err(SwapRefusal::ItselfAsMaster);
    }
    let mut out = SwapReport::default();
    // ⭐⭐⭐ **Primeiro a verdade, depois o palpite** — e o palpite só existe se um gesto o nomeou.
    let map = match piece_map_with(sim, &by_id, old, new_master_id) {
        Some(derived) => derived,
        None => {
            let r = crate::instance_swap_match::rematch(sim, &by_id, old, new_master_id, unrelated)
                .ok_or(SwapRefusal::Unrelated)?;
            out.ambiguous = r.ambiguous;
            r.map
        }
    };
    // ── os ELOS das peças ────────────────────────────────────────────────────────────────────
    //
    // ⚠️ A RAIZ é o caso particular, e é ela que decide de que mestre a instância é: sem esta
    // linha o passe estrutural continuava a comparar com o mestre antigo e desfazia tudo o resto.
    for e in subtree(sim, root) {
        let Some(link) = sim.world().get::<InstanceOf>(e).copied() else {
            continue;
        };
        if e == root {
            sim.world_mut().entity_mut(e).insert(InstanceOf {
                master: new_master_id,
            });
            continue;
        }
        match map.get(&link.master) {
            Some(&to) => {
                sim.world_mut()
                    .entity_mut(e)
                    .insert(InstanceOf { master: to });
                out.rekeyed += 1;
            }
            // ⚠️ **Deixado a apontar para a peça velha, de propósito.** O passe estrutural lê isso
            // como *«esta peça não é do mestre»*, sepulta a excepção dela e apaga-a — que é
            // exactamente o comportamento que a torna reversível.
            None => out.dropped += 1,
        }
    }

    // ── as CHAVES de override ────────────────────────────────────────────────────────────────
    //
    // ⚠️ Os órfãos re-chaveiam **também**: um órfão cuja peça existe no mestre novo é uma excepção
    // que volta a pegar assim que o passe estrutural a materializar (o `exhume` da F5.3).
    if let Some(mut inst) = sim.world_mut().get_mut::<ObjectInstance>(root) {
        // ⚠️⚠️ **Uma chave SEM imagem fica como está — apagá-la perdia a excepção em silêncio.**
        //
        // Medido: a 1.ª versão filtrava-as fora, e o gate da peça que a variante não tem ficou
        // vermelho por uma razão que parecia ser de outro sítio. A peça sem imagem é precisamente
        // a que o passe estrutural vai apagar, e é o `entomb` da F5.3 que lhe **serializa os
        // bytes** para os órfãos — mas só se ele ainda encontrar a chave lá. *A troca não tem de
        // saber o que é um byte: ela deixa a chave onde o sepultador a procura.*
        let kept: BTreeSet<_> = inst
            .overrides
            .iter()
            .map(|k| ph2d_ecs::OverrideKey {
                piece: map.get(&k.piece).copied().unwrap_or(k.piece),
                type_id: k.type_id,
            })
            .collect();
        out.overrides_kept = inst
            .overrides
            .iter()
            .filter(|k| map.contains_key(&k.piece))
            .count();
        inst.overrides = kept;
        inst.orphans = inst
            .orphans
            .iter()
            .map(|(k, v)| {
                let piece = map.get(&k.piece).copied().unwrap_or(k.piece);
                (
                    ph2d_ecs::OverrideKey {
                        piece,
                        type_id: k.type_id,
                    },
                    v.clone(),
                )
            })
            .collect();
    }

    // ── ⭐⭐⭐ o ECO das peças do mestre NOVO é ESQUECIDO ───────────────────────────────────────
    //
    // ⚠️ **Sem isto a troca é um no-op visível, e o mecanismo é o mesmo do REVERT.** No passe
    // seguinte a peça difere do mestre novo e o mestre novo não mexeu — que é exactamente a
    // assinatura de *«a instância mexeu-se»*. A instância capturava um override com o valor do
    // mestre **velho** e ficava surda ao novo para sempre. Medido: trocar da variante de volta
    // para a base deixava o corpo vermelho.
    //
    // ⇒ esquecer faz o passe cair na regra do 1.º encontro — *o mestre ganha* —, que já estava
    // escrita e justificada. ⭐ E as excepções do artista **sobrevivem à mesma**, porque a regra
    // (1) do passe (*«a instância possui este componente»*) corta antes da escada.
    //
    // ⚠️ **O eco é do MESTRE, logo isto alcança as instâncias IRMÃS** — e a colateral está
    // medida e é aceitável: uma irmã cuja peça já bate com o mestre sai por `want == have`, e uma
    // cuja peça o artista acabou de editar **neste mesmo quadro, antes de o passe correr** perdia
    // essa edição. São dois gestos no mesmo quadro em duas cópias diferentes — inalcançável por
    // mão. *Nomeado por ser o preço, não por ser um risco.*
    //
    // ⚠️⚠️ **A RAIZ do mestre novo está SEMPRE entre estes valores**, nos três modos de
    // emparelhamento e no mapa derivado — e é ela que este bloco mais precisa de alcançar. Sem a
    // raiz aqui, o componente que ela herda da receita cai exactamente no defeito descrito acima:
    // difere do mestre novo, o mestre novo não mexeu, e o passe lê *«a instância mexeu-se»*. A lei
    // que a garante vive em [`crate::instance_swap_match`] (*«a chave da raiz não tem sepultador»*),
    // e é ela que torna esta linha um percurso simples.
    for piece in map.values() {
        echo.master.retain(|&(p, _), _| p != *piece);
    }
    Ok(out)
}

/// `StableId` → entidade. ⚠️ **Construído UMA vez por gesto e passado adiante**: a composição do
/// mapa percorre um degrau por nível da ascendência, e reconstruí-lo em cada um seria `O(mundo ×
/// profundidade)` para responder à mesma pergunta.
type Index = BTreeMap<u64, Entity>;

fn stable_index(sim: &mut SimWorld) -> Index {
    let mut q = sim.world_mut().query::<(Entity, &StableId)>();
    q.iter(sim.world()).map(|(e, s)| (s.0, e)).collect()
}

/// A sub-árvore de `root`, ela incluída, em ordem determinística.
fn subtree(sim: &SimWorld, root: Entity) -> Vec<Entity> {
    let mut out = Vec::new();
    let mut stack = vec![root];
    while let Some(e) = stack.pop() {
        out.push(e);
        if let Some(kids) = sim.world().get::<Children>(e) {
            let mut k: Vec<Entity> = kids.iter().copied().collect();
            k.sort_by_key(|&c| ph2d_ecs::sibling_key(sim.world(), c));
            stack.extend(k.into_iter().rev());
        }
    }
    out
}

#[cfg(test)]
#[path = "instance_variant_tests.rs"]
mod tests;

/// ⚠️ **A AUTORIA de uma variante é outro assunto** — ali a lei do mapa e da troca, aqui o gesto
/// que faz uma cópia virar receita sem perder o elo. O precedente do corte é o
/// `instance_place_tests.rs` do módulo dos verbos.
#[cfg(test)]
#[path = "instance_variant_verb_tests.rs"]
mod verb_tests;
