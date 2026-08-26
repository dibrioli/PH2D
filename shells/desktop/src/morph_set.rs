//! ⭐⭐ **O CONJUNTO DE ESTADOS — um botão, e as formas viram uma máquina** (plano 32 W8).
//!
//! Enio, 2026-08-25:
//!
//! > *"o usuário seleciona todas as peças que estarão envolvidas na máquina de estados do morph.
//! > Com o clique de um único botão um objeto novo surge na hierarquia tendo como filhos as shapes
//! > escolhidas. Todas as setas são atribuídas automaticamente cobrindo todas as morphs possíveis
//! > entre todas as formas (tanto de ida como de volta). As setas são virtuais e ninguém jamais vê.
//! > No canvas uma única shape aparece (a shape do estado atual) e as demais ficam ocultas."*
//!
//! # As quatro coisas que o clique faz, e por que são um só passo
//!
//! 1. nasce o objecto (um `VecPath` vazio + [`VecMorph`], que é o que a cena **desenha**);
//! 2. as formas escolhidas viram **filhos** dele (`ChildOf`), na ordem de z;
//! 3. cada uma ganha `Visibility::hidden()` — *no canvas aparece uma forma só*;
//! 4. o [`VecMorphMachine`] recebe o **grafo completo dirigido** sobre elas.
//!
//! ⚠️ **Um passo de undo, não quatro.** As quatro escritas acontecem no mesmo quadro e o
//! `post_frame_undo` regista por DIFF — um Ctrl+Z desfaz o conjunto inteiro, que é o que o gesto
//! promete. Reparentar num quadro e esconder no seguinte daria dois passos, e o primeiro deixaria
//! o artista com nove formas empilhadas.
//!
//! # ⛔ O que esta wave APAGOU, e não se reconstrói sem ler isto
//!
//! A W3a desenhava as setas no canvas (âmbar, entre as formas) e a W3b tinha um **modo** de arrasto
//! forma→forma que criava uma aresta. As duas morreram aqui, e não por gosto:
//!
//! - *"as setas são virtuais e ninguém jamais vê"* mata o desenho por decisão directa;
//! - o grafo passou a ser **completo por construção**, então o arrasto criaria uma aresta **que já
//!   existe**. Um gesto cujo produto já está lá é um gesto que não faz nada — e o modo dele seria
//!   um pill na fileira a competir com treze irmãos por uma resposta que ninguém pode ver.
//!
//! ⚠️ *Duas portas para a mesma pergunta divergem em silêncio*: com o botão a gerar `n(n-1)` e o
//! arrasto a acrescentar à mão, a lista deixaria de ser derivável e a próxima derivação apagaria
//! o trabalho do arrasto.

use ph2d_ecs::{ChildOf, Entity, Name, SimWorld, VecMorph, VecMorphMachine, Visibility};
use ph2d_morph_machine::{MorphEdge, MorphGraph};
use ph2d_vec_scene::{VecPath, VecPathId, VecScene};

use crate::vec_entities::VecEntityMap;

/// **O conjunto à espera da entidade dele nascer** — o `sync` do quadro seguinte é que cria a
/// entidade do path novo, e só aí há onde pendurar os componentes.
///
/// ⚠️ **Espelho do `vec_morph_pending`**, e um slot PRÓPRIO porque o payload é outro: aquele leva
/// um componente, este leva a máquina **e** a lista de quem vai ser reparentado.
#[derive(Clone, Debug)]
pub(crate) struct MorphSetPending {
    /// O path do objecto novo (a forma morfada, ainda vazia).
    pub(crate) path: VecPathId,
    /// O nome que a Hierarquia mostra.
    pub(crate) name: String,
    /// As formas-membro, na ordem de z — **a primeira é o estado inicial**.
    pub(crate) members: Vec<VecPathId>,
}

/// **AS FORMAS DA SELEÇÃO QUE PODEM VIRAR ESTADOS.**
///
/// ⚠️ **Um estado é uma forma DESENHADA**, então o que entra é o que a cena sabe desenhar e o mapa
/// conhece. ⛔ Uma entidade que já é um Morph fica de fora: um conjunto sobre um conjunto daria uma
/// máquina cujos estados se re-escrevem a cada quadro por baixo dela.
///
/// ⚠️ **A ordem é a da SELEÇÃO**, que é a de z — e ela é load-bearing: o primeiro membro é o
/// `start` do grafo, e é a forma que o artista vê quando o conjunto nasce.
#[must_use]
pub(crate) fn eligible(sim: &SimWorld, map: &VecEntityMap, sel: &[VecPathId]) -> Vec<VecPathId> {
    let mut out: Vec<VecPathId> = Vec::new();
    for id in sel {
        let Some(&bits) = map.get(id) else { continue };
        let e = Entity::from_bits(bits);
        if sim.world().get_entity(e).is_err() || sim.world().get::<VecMorph>(e).is_some() {
            continue;
        }
        if !out.contains(id) {
            out.push(*id);
        }
    }
    out
}

/// ⭐ **O GRAFO COMPLETO DIRIGIDO** sobre `shapes` — *"todas as morphs possíveis entre todas as
/// formas (tanto de ida como de volta)"*.
///
/// `n(n-1)` arestas, todas **sem condição**: uma passagem que existe e nunca acontece até o artista
/// lhe dar uma acção. ⚠️ É esta a razão de não haver lixeira na lista — desligar é tirar a
/// condição, e o conjunto de arestas é uma função pura das formas.
///
/// ⚠️ **A ordem das arestas é determinística** (`from` externo, `to` interno, ambos na ordem dos
/// membros): a lista do painel indexa por posição, e uma ordem que dependesse de iteração de mapa
/// faria o menu de uma linha escrever a condição noutra depois de um undo.
#[must_use]
pub(crate) fn complete_digraph(shapes: &[VecPathId]) -> MorphGraph {
    let mut edges = Vec::with_capacity(shapes.len().saturating_mul(shapes.len().saturating_sub(1)));
    for &from in shapes {
        for &to in shapes {
            if from != to {
                edges.push(MorphEdge::new(from, to));
            }
        }
    }
    MorphGraph {
        start: shapes.first().copied().unwrap_or_default(),
        edges,
    }
}

/// **Cria o conjunto**: põe o path novo na cena e devolve o pendente. `None` se a seleção não dá
/// para um conjunto.
///
/// ⚠️ **O path nasce VAZIO de propósito** — a geometria é DERIVADA pelo `morph_live::recook` de
/// todo quadro, e inventá-la aqui seria uma 2ª porta para a mesma pergunta.
pub(crate) fn create(
    sim: &SimWorld,
    scene: &mut VecScene,
    map: &VecEntityMap,
    sel: &[VecPathId],
    max_states: usize,
) -> Option<MorphSetPending> {
    let members = eligible(sim, map, sel);
    if members.len() < 2 || members.len() > max_states {
        return None;
    }
    let path = scene.push_path(VecPath::default());
    Some(MorphSetPending {
        // O nome diz **quantos estados**, que é a coisa que o artista quer reconhecer na árvore.
        name: format!("Morph States {}", members.len()),
        path,
        members,
    })
}

/// **Drena o pendente** — pendura os componentes, reparenta os membros e esconde-os.
///
/// Roda entre o `vec_entities::sync` (a entidade do path já existe) e o `morph_live::recook` — o
/// mesmo lugar do `morph_live::upkeep`, e pela mesma razão.
///
/// ⚠️ **Devolve `true` quando consumiu**, e o chamador limpa o slot. Se a forma sumiu entretanto
/// (o artista apagou), consome à mesma: um pendente que nunca resolve ficaria a tentar para sempre.
pub(crate) fn upkeep(
    sim: &mut SimWorld,
    scene: &VecScene,
    map: &VecEntityMap,
    pending: &mut Option<MorphSetPending>,
) {
    let Some(p) = pending.as_ref() else { return };
    if !scene.paths().iter().any(|q| q.id == p.path) {
        *pending = None;
        return;
    }
    let Some(&bits) = map.get(&p.path) else {
        return;
    };
    let host = Entity::from_bits(bits);
    if sim.world().get_entity(host).is_err() {
        return;
    }
    let start = p.members[0];
    let graph = complete_digraph(&p.members);
    let members: Vec<Entity> = p
        .members
        .iter()
        .filter_map(|id| map.get(id).map(|&b| Entity::from_bits(b)))
        .collect();
    let name = p.name.clone();

    if let Ok(mut e) = sim.world_mut().get_entity_mut(host) {
        // ⚠️ **`sources = [start, start]`, e `t` no zero.** O `VecMorph::new` nasce a meio caminho
        // de propósito (um morph a `t=0` sobre a forma A não se anuncia), e aqui é o contrário: o
        // conjunto tem de mostrar **exactamente** o estado inicial, senão a primeira coisa que o
        // artista vê é uma forma que ele nunca desenhou.
        e.insert((
            VecMorph {
                sources: [start, start],
                t: 0.0,
            },
            VecMorphMachine { graph },
            Name::new(name),
        ));
    }
    for m in members {
        if let Ok(mut e) = sim.world_mut().get_entity_mut(m) {
            // ⚠️ **Esconder E reparentar, nesta ordem, na mesma escrita.** O `visible_chain` lê o
            // próprio E os ancestrais, então esconder o membro é o suficiente — e esconder o PAI
            // apagaria o conjunto inteiro, que é o objecto que se quer ver.
            e.insert((Visibility::hidden(), ChildOf(host)));
            // ⛔ O `RootOrder` sai: ele só vale para raízes, e um membro deixou de o ser. É o
            // mesmo par de operações do `vec_entities::group_entities`, e a razão é a mesma.
            e.remove::<ph2d_ecs::RootOrder>();
        }
    }
    *pending = None;
}

#[cfg(test)]
#[path = "morph_set_tests.rs"]
mod tests;
