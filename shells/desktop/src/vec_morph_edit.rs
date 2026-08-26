//! **A COSTURA das setas do Morph** (plano 32 W4) — irmã do [`crate::vec_ui_state_edit`], com a
//! mesma divisão: a verdade mora no `VecMorphMachine` (mundo) e isto é o que a shell **publica**
//! por quadro e o que ela **aplica** quando um clique volta pelo barramento.
//!
//! ⚠️ **O painel não alcança o mundo, e não pode:** se alcançasse, a resposta que decide QUE linha
//! pintar divergiria da que HONRA o clique.

use ph2d_ecs::{Entity, Name, SimWorld, VecMorph, VecMorphMachine};
use ph2d_panel_vector::state::{MorphArrowRow, MorphStatesState};
use ph2d_vec_scene::{VecPathId, VecScene};

use crate::vec_entities::VecEntityMap;

/// **O que um clique numa linha de seta PEDE.**
///
/// ⚠️ Uma tabela e não uma cadeia de `if`: o verbo seguinte entra numa linha, e quem esquecer a
/// linha vê o botão morto **no gate de costura** — em vez de o ver a cair no `None` em silêncio.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ArrowCmd {
    /// **Fazer o conjunto** com as formas da seleção (plano 32 W8).
    MakeSet,
    /// Pôr a condição da seta `row` na acção `action` — **`0` é o «—»**, sem condição.
    ///
    /// ⛔ **É o ÚNICO verbo que age sobre uma seta**, e a ausência de um `Delete` é a lei da W8: o
    /// grafo é o completo dirigido sobre as formas do conjunto, **derivado**. Apagar uma linha
    /// seria apagar uma passagem que a próxima derivação repõe — *desligar uma transição é
    /// tirar-lhe a condição*, e uma seta sem condição existe e nunca acontece.
    SetWhen { row: usize, action: usize },
}

/// **O comando que um `NodeId` pede**, ou `None` se ele não é de uma seta.
#[must_use]
pub(crate) fn arrow_cmd_for_id(id: ph2d_editor::NodeId) -> Option<ArrowCmd> {
    use ph2d_editor::ids as i;
    if id == i::VECTOR_MORPH_STATES_MAKE {
        return Some(ArrowCmd::MakeSet);
    }
    for row in 0..i::MAX_MORPH_ARROWS {
        for action in 0..i::MAX_MORPH_ACTIONS {
            if id == i::morph_arrow_when_option_id(row, action) {
                return Some(ArrowCmd::SetWhen { row, action });
            }
        }
    }
    None
}

/// **O objecto de Morph da seleção**, se houver um.
///
/// ⚠️ **Percorre a seleção INTEIRA à procura de um `VecMorph`**, e não `selected.first()`: tocar num
/// morph pode trazer o grupo, e o primeiro operando seria um qualquer — a seção mostraria as setas
/// de um objecto e o clique escreveria noutro. É a lição, palavra por palavra, do
/// `host_of_selection`.
///
/// ⛔⛔ **A seleção do vetor é uma lista de `VecPathId`, NUNCA de bits de entidade — e esta função
/// tratou-a como bits durante uma wave inteira.** O `VecPathId` é um alias de `u64` e o
/// `Entity::to_bits()` também devolve `u64`, então **o compilador não tinha como ajudar**: o código
/// compilou, os gates ficaram verdes, e o app **entrou em pânico no quadro 1639** do smoke do
/// Enio (`Attempted to initialize invalid bits as an entity` — o id `1` de um path não é uma
/// entidade válida).
///
/// ⚠️ **O que teria evitado isto era a CONVENÇÃO da casa**, e ela está a dois ficheiros de
/// distância: o `host_of_selection` declara `selected: &[VecPathId]` e resolve pelo mapa. *Quando
/// dois significados partilham um tipo primitivo, o nome do parâmetro é a única barreira que
/// resta* — e eu escrevi `sel: &[u64]`.
#[must_use]
pub(crate) fn morph_of_selection(
    sim: &SimWorld,
    map: &VecEntityMap,
    selected: &[VecPathId],
) -> Option<Entity> {
    selected
        .iter()
        .filter_map(|id| map.get(id).map(|&bits| Entity::from_bits(bits)))
        .find(|&e| sim.world().get::<VecMorph>(e).is_some())
}

/// O nome de uma forma, ou o rótulo genérico.
///
/// ⚠️ **Nome e não id**: o artista escolheu formas, e um número não lhe diz qual delas é. Sem
/// `Name` no documento cai num rótulo derivado do id — que é feio, e é honesto.
fn shape_name(sim: &SimWorld, map: &VecEntityMap, scene: &VecScene, id: VecPathId) -> String {
    let _ = scene;
    map.iter()
        .find(|&(&k, _)| k == id)
        .and_then(|(_, &bits)| sim.world().get::<Name>(Entity::from_bits(bits)))
        .map(|n| n.0.clone())
        .filter(|n| !n.is_empty())
        .unwrap_or_else(|| format!("#{id}"))
}

/// **Publica as setas da seleção** (shell → painel). `None` = a seleção não é um Morph.
#[must_use]
pub(crate) fn publish(
    sim: &SimWorld,
    scene: &VecScene,
    map: &VecEntityMap,
    sel: &[VecPathId],
    preview: bool,
    actions: Vec<String>,
) -> Option<MorphStatesState> {
    let Some(e) = morph_of_selection(sim, map, sel) else {
        // ⭐ **Sem Morph na seleção a seção AINDA existe, se houver formas que possam virar um**
        // (plano 32 W8): é ela que traz o botão que os cria. ⛔ Devolver `None` aqui faria a única
        // porta para a feature aparecer só depois de a feature existir.
        let n = crate::morph_set::eligible(sim, map, sel).len();
        // ⛔ **`preview: false` aqui, e não o `preview` recebido.** Sem máquina não há modo a
        // anunciar — e o botão não é sequer pintado nesta face. Passar o valor real acenderia um
        // interruptor que não existe.
        return (n >= 2).then(|| MorphStatesState {
            can_make: n,
            actions,
            ..Default::default()
        });
    };
    // ⚠️ **Um Morph SEM máquina publica a face VAZIA, e não `None`.** As duas coisas pintam faces
    // diferentes: `None` = *"a seleção não é um Morph"* (a seção nem fala de setas); vazio =
    // *"é um Morph e ainda não tem setas"*, e essa face diz COMO desenhar a primeira.
    let graph = sim
        .world()
        .get::<VecMorphMachine>(e)
        .map(|m| m.graph.clone());
    let live = sim
        .world()
        .get::<VecMorph>(e)
        .map(|v| (v.sources[0], v.sources[1]));
    let rows = graph.as_ref().map_or_else(Vec::new, |g| {
        g.edges
            .iter()
            .map(|edge| MorphArrowRow {
                from: shape_name(sim, map, scene, edge.from),
                to: shape_name(sim, map, scene, edge.to),
                when: edge.when.clone(),
                live: live == Some((edge.from, edge.to)),
            })
            .collect()
    });
    // ⭐ **O readout é a forma que a cena MOSTRA agora**, e ele sai do mesmo `VecMorph` que a
    // desenha — um número derivado noutro sítio diria uma forma e a cena mostraria outra.
    let current = live.map(|(_, to)| shape_name(sim, map, scene, to));
    Some(MorphStatesState {
        rows,
        actions,
        current,
        // ⛔ **Zero: um conjunto já feito não se refaz por cima de si próprio.** Oferecer o botão
        // aqui daria dois objectos de estados sobre as mesmas formas, e o segundo nasceria a
        // governar formas que o primeiro já esconde.
        can_make: 0,
        preview,
    })
}

/// **Aplica um comando de seta ao mundo.** Devolve `true` se alguma coisa mudou.
///
/// ⚠️ **`actions` é a MESMA lista que o menu mostrou**, passada de fora: resolvê-la aqui a partir
/// do mapa seria uma segunda leitura, e as duas divergiriam no quadro em que o artista criasse uma
/// acção — o índice escolhido apontaria para outro nome.
pub(crate) fn apply(sim: &mut SimWorld, morph: Entity, cmd: ArrowCmd, actions: &[String]) -> bool {
    let Some(mut m) = sim.world_mut().get_mut::<VecMorphMachine>(morph) else {
        return false;
    };
    match cmd {
        // ⚠️ **O `MakeSet` não vive aqui:** ele cria uma ENTIDADE, reparenta formas e escreve na
        // cena vetorial — nada disso cabe numa função que só tem o componente de um objecto que
        // ainda não existe. Ele é servido pelo [`crate::morph_set`], e este braço é a prova de que
        // a tabela é exaustiva.
        ArrowCmd::MakeSet => false,
        ArrowCmd::SetWhen { row, action } => {
            let Some(e) = m.graph.edges.get_mut(row) else {
                return false;
            };
            // ⚠️ **O índice `0` é o «—»** — tirar a condição tem de ser um gesto, senão o artista
            // só poderia apagar a seta inteira para se arrepender.
            let want = if action == 0 {
                String::new()
            } else {
                match actions.get(action - 1) {
                    Some(n) => n.clone(),
                    // Um índice fora da lista publicada: o mapa mudou entre o menu abrir e o
                    // clique chegar. **Recusa** em vez de escrever um nome inventado.
                    None => return false,
                }
            };
            if e.when == want {
                return false;
            }
            e.when = want;
            true
        }
    }
}

#[cfg(test)]
#[path = "vec_morph_edit_tests.rs"]
mod tests;
