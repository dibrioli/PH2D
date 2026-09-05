//! ⭐⭐⭐ **Emparelhar as peças quando NÃO há parentesco** (ADR-0164 / plano F5, o último critério).
//!
//! # A fronteira, e por que ela existe
//!
//! [`crate::instance_variant`] troca o mestre de uma cópia **lendo os elos**: as peças de uma
//! variante dizem de que peça da base nasceram, então o mapa `de → para` está no mundo e não se
//! adivinha nada. Esse é o caminho da fileira de versões, e ele **recusa** dois mestres sem
//! antepassado comum ([`crate::instance_variant::SwapRefusal::Unrelated`]) — não por preguiça:
//! sem elo não existe resposta derivada, só palpite.
//!
//! Este ficheiro é o palpite, **pedido em voz alta**. Ele existe porque o artista às vezes quer
//! exactamente isso — *«troca este Carro por um Camião e tenta levar o que eu mexi»* — e a única
//! coisa que o plano proíbe é que aconteça **sozinho**:
//!
//! > *«Trocar para mestre NÃO aparentado: só por gesto, com os 3 modos + relatório.
//! > ⛔ Nunca automático (HR-5).»*
//!
//! ⇒ os três modos são **três itens de menu diferentes**, e o caminho de omissão
//! ([`WhenUnrelated::Refuse`]) é o que todos os chamadores de dentro do app passam.
//!
//! # ⚠️ O mapa derivado GANHA sempre
//!
//! Quando os dois mestres **são** aparentados, [`crate::instance_variant::piece_map`] responde e
//! este ficheiro nem chega a correr — mesmo que o artista tenha escolhido *«por nome»*. Um modo de
//! emparelhamento é a resposta para a **ausência** de uma verdade, nunca uma alternativa a ela.
//!
//! # ⭐⭐ A chave é um CAMINHO, e não um nome solto
//!
//! A 1.ª redacção emparelhava por `Name` cru, e ela produzia mapas que **partem a árvore**: a peça
//! `Wheel` que no Carro pende da raiz e no Camião pende da `Cabin` seria emparelhada, e a cópia
//! ficava com a roda debaixo da raiz enquanto a receita a diz debaixo da cabina —
//! [`crate::instance_structure::reconcile`] **não muda peças de pai** (ele materializa as que
//! faltam e apaga as que sobram), então o defeito ficaria estável e mudo.
//!
//! ⇒ a chave dos DOIS modos é o **caminho desde a raiz**, e a diferença entre eles é só o degrau:
//!
//! | modo | um degrau é | sobrevive a | perde-se com |
//! |---|---|---|---|
//! | [`WhenUnrelated::ByName`] | o `Name` da peça | reordenar os irmãos | renomear |
//! | [`WhenUnrelated::ByHierarchy`] | o índice entre os irmãos | renomear | reordenar |
//!
//! São as duas metades do `ObjectMatchMode` do Unity, e a razão de haver duas é que **nenhuma
//! contém a outra**.
//!
//! ⚠️ **E um caminho só emparelha se o PAI dele emparelhar** — a lei que torna o mapa
//! estruturalmente consistente por construção. Ela morde de facto: com dois irmãos chamados `Arm`
//! o caminho `Arm` é ambíguo e cai, e sem esta lei o `Arm/Hand` de um deles (que é único!)
//! emparelharia e a mão ficaria pendurada num pai que não existe daquele lado.
//!
//! # ⭐⭐⭐ A RAIZ emparelha SEMPRE — inclusive no modo *«não leves nada»*
//!
//! Ela é o **objecto**, e não uma peça: *«não leves nada»* é sobre o que a receita deu. Mas a razão
//! que fecha a questão é **mecânica, e não de gosto** — só ela explica por que a excepção da raiz
//! não pode ficar sem imagem:
//!
//! > **A chave da raiz não tem sepultador.** [`crate::instance_structure`] enterra a excepção de uma
//! > peça (`entomb`) **no instante em que a peça morre**, e a raiz de uma instância **nunca morre**
//! > numa troca — o que muda é o elo dela. ⇒ uma chave de raiz sem imagem no mapa não fica *viva*
//! > (o passe compara com a peça do mestre NOVO, e a chave aponta para a do velho) nem *sepultada*
//! > (ninguém a enterra): ela fica **invisível**, a impedir para sempre que a receita nova alcance
//! > aquele componente da cópia, sem uma linha em lado nenhum do painel a dizê-lo.
//!
//! ⚠️ E é a mesma razão pela qual o **eco** do mestre novo é esquecido na troca — o
//! [`crate::instance_variant::swap`] percorre os valores do mapa, e a raiz estar lá é o que faz o
//! primeiro passe cair na regra do 1.º encontro em vez de congelar a cópia com o valor do velho.
//!
//! ⚠️ **Nada disto move o objecto:** onde ele está, como se chama e em que ordem aparece são da raiz
//! e **nunca foram do mestre** (a lista `ROOT_IS_ITS_OWN` do [`crate::instance_sync`]), logo não são
//! excepções e não passam por mapa nenhum.
//!
//! # ⛔ O que NÃO se emparelha, e as duas ausências são decisões
//!
//! 1. **Uma peça SEM nome, no modo `ByName`** — e com ela a sub-árvore inteira, porque um caminho
//!    de nomes não atravessa um degrau que não tem nome. Emparelhar dois anónimos *«porque os dois
//!    são o primeiro filho»* seria o modo `ByHierarchy` a correr com o rótulo do outro.
//! 2. **O que o ARTISTA pendurou na cópia** — ele não tem elo, logo não é peça de receita nenhuma,
//!    e o passe estrutural já o deixa em paz. *Só o que a receita deu é que a receita tira.*

use ph2d_ecs::{Children, Entity, Name, SimWorld, StableId};
use std::collections::BTreeMap;

/// **O que fazer quando os dois mestres não têm antepassado comum.**
///
/// ⚠️ **O caminho de omissão é [`Self::Refuse`]**, e é ele que a fileira de versões do cartão
/// passa: ali os mestres são aparentados por construção, e uma queda para heurística seria a
/// operação automática que o plano proíbe.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub(crate) enum WhenUnrelated {
    /// ⛔ Recusar a troca. *Sem elo não há resposta derivada, e ninguém pediu um palpite.*
    #[default]
    Refuse,
    /// Trocar **sem levar excepção nenhuma** — o `None` do Unity, e o item de menu sem adjectivo.
    CarryNothing,
    /// Emparelhar pelo **caminho de nomes** desde a raiz.
    ByName,
    /// Emparelhar pelo **caminho de índices** entre irmãos, desde a raiz.
    ByHierarchy,
}

/// O mapa `peça do mestre velho → peça do mestre novo`, e o que ficou de fora por ambiguidade.
pub(crate) struct Rematch {
    /// ⚠️ **Injectivo por construção** — cada caminho aceite tem exactamente uma peça de cada
    /// lado. Duas peças da cópia a apontar para a mesma peça do mestre seriam dois pares no passe
    /// de valores a escrever a mesma origem em dois destinos, e o passe estrutural veria a segunda
    /// como *«já existe»*.
    pub(crate) map: BTreeMap<u64, u64>,
    /// Quantos caminhos apareceram **mais do que uma vez** de um dos lados e por isso não
    /// emparelharam. ⛔ Escrito, nunca engolido: é a metade do relatório que diz ao artista *porquê*
    /// uma excepção que ele esperava não veio.
    pub(crate) ambiguous: usize,
}

/// **Um degrau do caminho.** `Vec<String>` e não uma string junta por `/`: um `Name` pode conter
/// qualquer carácter, e uma peça chamada `a/b` faria dois caminhos diferentes colidirem.
type Path = Vec<String>;

/// ⭐⭐⭐ **O mapa de emparelhamento para dois mestres sem parentesco.**
///
/// `None` quando o modo é [`WhenUnrelated::Refuse`] — o chamador transforma isso na recusa que o
/// artista lê.
#[must_use]
pub(crate) fn rematch(
    sim: &SimWorld,
    by_id: &BTreeMap<u64, Entity>,
    from: u64,
    to: u64,
    how: WhenUnrelated,
) -> Option<Rematch> {
    let (by_path, ambiguous) = match how {
        WhenUnrelated::Refuse => return None,
        // ⚠️ Nenhuma PEÇA — a raiz entra logo a seguir, e o porquê está no cabeçalho.
        WhenUnrelated::CarryNothing => (BTreeMap::new(), 0),
        WhenUnrelated::ByName | WhenUnrelated::ByHierarchy => pair_up(sim, by_id, from, to, how),
    };
    let mut map: BTreeMap<u64, u64> = by_path.into_values().collect();
    // ⭐⭐⭐ **A raiz emparelha com a raiz, nos TRÊS modos** — ver *«a chave da raiz não tem
    // sepultador»* no cabeçalho. ⛔ Ela fica FORA da tabela de caminhos de propósito: lá a chave é o
    // caminho *desde* a raiz, e uma peça do mestre novo que calhe de ter o nome da raiz velha não é
    // a raiz.
    map.insert(from, to);
    Some(Rematch { map, ambiguous })
}

/// Os pares `caminho → (peça de `from`, peça de `to`)` que sobrevivem à ambiguidade **e** à lei do
/// pai, mais a contagem dos ambíguos.
fn pair_up(
    sim: &SimWorld,
    by_id: &BTreeMap<u64, Entity>,
    from: u64,
    to: u64,
    how: WhenUnrelated,
) -> (BTreeMap<Path, (u64, u64)>, usize) {
    let a = keyed(sim, by_id, from, how);
    let b = keyed(sim, by_id, to, how);
    let mut ambiguous = 0;
    let mut candidates: BTreeMap<Path, (u64, u64)> = BTreeMap::new();
    for (path, from_ids) in &a {
        let Some(to_ids) = b.get(path) else { continue };
        // ⚠️ **Um dos lados chega:** com dois `Arm` do lado de cá e um do lado de lá não há resposta
        // determinística, e escolher *«o de `StableId` menor»* seria aplicar a excepção do artista
        // ao braço que calhou de nascer primeiro.
        if from_ids.len() > 1 || to_ids.len() > 1 {
            ambiguous += 1;
            continue;
        }
        candidates.insert(path.clone(), (from_ids[0], to_ids[0]));
    }
    // ── a LEI DO PAI ─────────────────────────────────────────────────────────────────────────
    //
    // ⭐ **Um só percurso chega, e a razão é a ordem do `BTreeMap`:** um caminho ordena sempre
    // depois de todos os prefixos dele (`[a] < [a, b]`), então quando se chega a um filho o pai
    // dele já foi decidido — e a decisão do pai já honrou o avô. *Verificar só o pai imediato é
    // verificar a cadeia inteira.*
    let mut accepted: BTreeMap<Path, (u64, u64)> = BTreeMap::new();
    for (path, pair) in &candidates {
        let parent = &path[..path.len() - 1];
        if parent.is_empty() || accepted.contains_key(parent) {
            accepted.insert(path.clone(), *pair);
        }
    }
    (accepted, ambiguous)
}

/// `caminho → as peças que o têm`, para a sub-árvore de um mestre. **A raiz fica de fora** (o
/// caminho dela é vazio).
///
/// ⚠️ **A lista é um `Vec` e não um `u64`**, porque a ambiguidade é precisamente a informação que
/// se perderia num mapa de valor único: com `insert` o último a chegar ganhava, em silêncio.
fn keyed(
    sim: &SimWorld,
    by_id: &BTreeMap<u64, Entity>,
    master_id: u64,
    how: WhenUnrelated,
) -> BTreeMap<Path, Vec<u64>> {
    let mut out: BTreeMap<Path, Vec<u64>> = BTreeMap::new();
    let Some(&root) = by_id.get(&master_id) else {
        return out;
    };
    let mut stack: Vec<(Entity, Path)> = vec![(root, Path::new())];
    while let Some((e, path)) = stack.pop() {
        if !path.is_empty()
            && let Some(sid) = sim.world().get::<StableId>(e).map(|s| s.0)
        {
            out.entry(path.clone()).or_default().push(sid);
        }
        let Some(kids) = sim.world().get::<Children>(e) else {
            continue;
        };
        let mut k: Vec<Entity> = kids.iter().copied().collect();
        // A MESMA ordem que a Hierarquia mostra e que o passe estrutural percorre — o índice de um
        // irmão só é uma chave se as duas travessias concordarem sobre qual é.
        k.sort_by_key(|&c| ph2d_ecs::sibling_key(sim.world(), c));
        for (i, c) in k.into_iter().enumerate() {
            // ⚠️ Sem degrau, a sub-árvore inteira sai — ver a ausência nº 2 do cabeçalho.
            let Some(step) = step_of(sim, c, i, how) else {
                continue;
            };
            let mut p = path.clone();
            p.push(step);
            stack.push((c, p));
        }
    }
    out
}

/// O degrau que esta peça acrescenta ao caminho do pai, ou `None` quando ela não é endereçável
/// naquele modo.
fn step_of(sim: &SimWorld, e: Entity, index: usize, how: WhenUnrelated) -> Option<String> {
    match how {
        WhenUnrelated::ByHierarchy => Some(index.to_string()),
        WhenUnrelated::ByName => sim
            .world()
            .get::<Name>(e)
            .map(|n| n.0.clone())
            .filter(|n| !n.is_empty()),
        // ⛔ Inalcançável pela porta: os dois modos que não emparelham nunca chegam ao `keyed`.
        WhenUnrelated::Refuse | WhenUnrelated::CarryNothing => None,
    }
}

#[cfg(test)]
#[path = "instance_swap_match_tests.rs"]
mod tests;
