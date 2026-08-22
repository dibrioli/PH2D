//! **A costura da shell com o DIAGRAMA da booleana viva** — a vista, a estrela e as intenções.
//!
//! O card ([`ph2d_editor::screens::hero::chrome::paint_bool_graph_modal`]) desenha e publica
//! intenções; o motor ([`ph2d_vec_boolean::graph`]) resolve. Aqui mora a única coisa que nenhum
//! dos dois alcança: **o mundo**. Quem sabe que uma forma se chama *"Estrela"*, quem sabe quais
//! formas o grupo considerou, e quem escreve o componente é a shell.
//!
//! # A vista vem do REGISTO do produtor, nunca de uma segunda triagem
//!
//! ⚠️ [`crate::bool_live::BoolLive::roster`] é a lista que o motor de facto considerou neste
//! frame. Recalcular *"quais são os operandos?"* aqui daria uma segunda definição de **operando**,
//! e ela divergiria da primeira no dia em que a triagem mudasse — o artista veria no diagrama
//! formas que o motor não opera, e o diagrama mentiria sobre o desenho ao lado dele.
//!
//! E o registo sobrevive à RECUSA, que é onde ele mais importa: com um ciclo não há plano nenhum, e
//! é exatamente aí que o diagrama tem de mostrar os círculos e dizer o que está errado.
//!
//! # A estrela materializa-se ao ABRIR, e isso é seguro por PROVA
//!
//! [`materialize_star`] escreve o grafo equivalente ao grupo de hoje. Dois gates prendem a
//! igualdade (`a_estrela_derivada_desenha_o_que_o_grupo_de_hoje_desenha` no motor,
//! `a_estrela_materializada_no_componente_nao_move_a_arte` na costura) — sem eles, abrir o
//! diagrama moveria a arte no instante em que o artista olhasse para ela.

use ph2d_ecs::{Entity, Name, SimWorld, VecBoolEdge, VecBoolEdges, VecBoolGraphPos, VecBoolGroup};
use ph2d_editor::widget::{BoolGraphIntent, BoolGraphLink, BoolGraphNode, BoolGraphView};
use ph2d_vec_scene::VecPathId;

use crate::bool_live::{BoolLive, code_of_op, op_of_code};
use crate::vec_entities::VecEntityMap;

/// O nome que o artista lê na Hierarquia, para o `VecPathId` dado.
///
/// ⚠️ Um caminho sem entidade (ou sem `Name`) recebe um rótulo derivado do id em vez de vazio: um
/// círculo sem nome é indistinguível dos outros, e o diagrama inteiro depende de os distinguir.
fn label_of(sim: &SimWorld, map: &VecEntityMap, id: VecPathId) -> String {
    map.get(&id)
        .and_then(|bits| sim.world().get::<Name>(Entity::from_bits(*bits)))
        .map_or_else(|| format!("Path {id}"), |n| n.0.to_string())
}

/// As ligações do grupo, **filtradas pelos operandos vivos** — a mesma rede que o `bool_live` põe.
fn live_edges(sim: &SimWorld, g: Entity, roster: &[VecPathId]) -> Vec<VecBoolEdge> {
    sim.world()
        .get::<VecBoolEdges>(g)
        .map(|e| {
            e.edges
                .iter()
                .copied()
                .filter(|l| roster.contains(&l.from) && roster.contains(&l.to))
                .collect()
        })
        .unwrap_or_default()
}

/// **O que o diagrama mostra** para o grupo `g`. Vazio quando ele não cozinhou nada neste frame.
#[must_use]
pub(crate) fn view_of(
    sim: &SimWorld,
    map: &VecEntityMap,
    bl: &BoolLive,
    g: Entity,
) -> BoolGraphView {
    let Some(roster) = bl.roster(g) else {
        return BoolGraphView::default();
    };
    let edges = live_edges(sim, g, roster);
    let links: Vec<BoolGraphLink> = edges
        .iter()
        .map(|e| BoolGraphLink {
            from: e.from,
            to: e.to,
            op: e.op,
        })
        .collect();
    // ⚠️ **Consumido = tem ligação de SAÍDA**, exatamente o predicado do resolvedor. Derivá-lo do
    // plano desenhado seria mais frágil e daria a resposta errada na recusa (onde não há plano).
    let pos = sim.world().get::<VecBoolGraphPos>(g);
    let nodes: Vec<BoolGraphNode> = roster
        .iter()
        .map(|id| BoolGraphNode {
            id: *id,
            label: label_of(sim, map, *id),
            consumed: links.iter().any(|l| l.from == *id),
            // ⚠️ `None` = ainda não foi arrastado, e o diagrama arruma no anel default. Inventar
            // uma posição aqui faria o anel deixar de existir e todo círculo nascer no mesmo sítio.
            at: pos.and_then(|p| p.get(*id)),
        })
        .collect();
    let bool_edges: Vec<ph2d_vec_boolean::BoolEdge> = edges
        .iter()
        .filter_map(|e| {
            op_of_code(e.op).map(|op| ph2d_vec_boolean::BoolEdge {
                from: e.from,
                to: e.to,
                op,
            })
        })
        .collect();
    let cycle = ph2d_vec_boolean::has_cycle(roster, &bool_edges);
    BoolGraphView {
        nodes,
        links,
        cycle,
    }
}

/// **Materializa a estrela** se o grupo ainda não tem grafo. Devolve `true` se escreveu.
///
/// ⚠️ Só age quando o componente está **AUSENTE**: uma lista vazia é um grafo deliberado (o artista
/// cortou todas as ligações), e re-semeá-la faria as formas voltarem a fundir-se sozinhas — o
/// oposto exato do que ele acabou de fazer.
///
/// ⚠️ E só para as quatro operações de CONJUNTO. Um grupo em `Trim`/`Crop`/`Merge`/`MinusBack` não
/// tem estrela equivalente (a receita é sobre a pilha inteira, não sobre pares), então abrir o
/// diagrama sobre ele mostra os círculos **sem ligação nenhuma** em vez de inventar uma tradução
/// que mudaria o desenho.
pub(crate) fn materialize_star(sim: &mut SimWorld, bl: &BoolLive, g: Entity) -> bool {
    if sim.world().get::<VecBoolEdges>(g).is_some() {
        return false;
    }
    let Some(roster) = bl.roster(g) else {
        return false;
    };
    let Some(op) = sim.world().get::<VecBoolGroup>(g).map(|c| c.op) else {
        return false;
    };
    let Some(pf) = op_of_code(op) else {
        return false;
    };
    if pf.as_bool().is_none() {
        return false;
    }
    let edges: Vec<VecBoolEdge> = ph2d_vec_boolean::derive_star(roster, pf)
        .into_iter()
        .map(|e| VecBoolEdge {
            from: e.from,
            to: e.to,
            op: code_of_op(e.op),
        })
        .collect();
    sim.world_mut()
        .entity_mut(g)
        .insert(VecBoolEdges::new(edges));
    true
}

/// **Escreve as intenções** do diagrama no componente. Devolve `true` se alguma coisa mudou (e o
/// `post_frame_undo` a capturará).
///
/// ⚠️ Uma ligação `A → B` **substitui** a que já existia entre o mesmo par ordenado — é o que
/// `VecBoolEdges::set` garante. Sem isso, ligar duas formas outra vez empilharia uma segunda
/// ligação invisível no diagrama (é uma linha só entre dois círculos) e o resolvedor dobraria `A`
/// em `B` duas vezes.
/// O que uma leva de intenções produziu.
pub(crate) struct IntentOutcome {
    /// O documento mudou (e o `post_frame_undo` a capturará).
    pub(crate) changed: bool,
    /// A forma que o artista pediu para SELECIONAR no canvas, se pediu.
    ///
    /// ⚠️ Ela sai daqui em vez de ser aplicada lá dentro porque a seleção não é do mundo ECS — ela
    /// é do `PenTool` da shell, e escrevê-la aqui daria a este módulo dois donos.
    pub(crate) select: Option<VecPathId>,
}

pub(crate) fn apply_intents(
    sim: &mut SimWorld,
    g: Entity,
    intents: &[BoolGraphIntent],
) -> IntentOutcome {
    let mut out = IntentOutcome {
        changed: false,
        select: None,
    };
    if intents.is_empty() {
        return out;
    }
    let mut edges = sim
        .world()
        .get::<VecBoolEdges>(g)
        .cloned()
        .unwrap_or_default();
    let mut pos = sim
        .world()
        .get::<VecBoolGraphPos>(g)
        .cloned()
        .unwrap_or_default();
    let (mut edges_dirty, mut pos_dirty) = (false, false);
    for intent in intents {
        match *intent {
            BoolGraphIntent::Link { from, to, op } => {
                if edges.get(from, to) != Some(op) {
                    edges.set(from, to, op);
                    edges_dirty = true;
                }
            }
            BoolGraphIntent::Unlink { from, to } => {
                edges_dirty |= edges.remove(from, to);
            }
            BoolGraphIntent::Move { id, at } => {
                if pos.get(id) != Some(at) {
                    pos.set(id, at);
                    pos_dirty = true;
                }
            }
            // ⚠️ Selecionar NÃO muda o documento — não entra no `changed`, senão cada clique num
            // círculo viraria um passo de undo que não mexeu em nada.
            BoolGraphIntent::Select { id } => out.select = Some(id),
        }
    }
    if edges_dirty {
        sim.world_mut().entity_mut(g).insert(edges);
    }
    if pos_dirty {
        sim.world_mut().entity_mut(g).insert(pos);
    }
    out.changed = edges_dirty || pos_dirty;
    out
}

/// **Re-mira o GRAFO** quando o artista clica num dos oito verbos com um grupo vivo em mãos.
///
/// - Uma das quatro de **CONJUNTO** reescreve TODAS as ligações. O botão passa a significar *"ponha
///   tudo neste verbo"*, e o diagrama continua lá para as diferenciar de novo.
/// - Uma das quatro **RECEITAS** (`MinusBack`/`Trim`/`Crop`/`Merge`) **remove o grafo**: ela é uma
///   afirmação sobre a pilha inteira e não tem tradução em pares.
///
/// ⚠️ Sem isto os oito botões ficariam MORTOS sobre um grupo com diagrama — o artista clicaria e a
/// arte não mudaria, porque quem manda passou a ser a operação de cada ligação. Devolve `true` se
/// escreveu.
pub(crate) fn retarget_graph(sim: &mut SimWorld, g: Entity, op: u8) -> bool {
    let Some(edges) = sim.world().get::<VecBoolEdges>(g).cloned() else {
        return false;
    };
    let de_conjunto = op_of_code(op).is_some_and(|p| p.as_bool().is_some());
    if !de_conjunto {
        sim.world_mut().entity_mut(g).remove::<VecBoolEdges>();
        return true;
    }
    if edges.edges.iter().all(|e| e.op == op) {
        return false;
    }
    let mut novo = edges;
    for e in &mut novo.edges {
        e.op = op;
    }
    sim.world_mut().entity_mut(g).insert(novo);
    true
}

/// A metade da varredura que cuida das POSIÇÕES — mesma lei do irmão: só escreve quando apaga.
fn prune_dead_positions(sim: &mut SimWorld, vivos: &std::collections::BTreeSet<VecPathId>) -> bool {
    let mut q = sim.world_mut().query::<(Entity, &VecBoolGraphPos)>();
    let mortos: Vec<(Entity, VecBoolGraphPos)> = q
        .iter(sim.world())
        .filter_map(|(e, pos)| {
            let mut limpo = pos.clone();
            for id in pos
                .nodes
                .iter()
                .map(|n| n.id)
                .filter(|id| !vivos.contains(id))
                .collect::<Vec<_>>()
            {
                limpo.forget(id);
            }
            (limpo != *pos).then_some((e, limpo))
        })
        .collect();
    // ⚠️ O `bool` não é cerimônia: sem ele a varredura escreveria o componente e diria *"nada
    // mudou"*, e quem lê essa resposta (o sinal de documento sujo) discordaria do documento. Foi um
    // gate que apanhou isto.
    let mudou = !mortos.is_empty();
    for (e, limpo) in mortos {
        sim.world_mut().entity_mut(e).insert(limpo);
    }
    mudou
}

/// **Varre as ligações ÓRFÃS** — as que nomeiam uma forma que já não está no documento. Devolve
/// `true` se apagou alguma.
///
/// ⚠️ Isto **não é a correção de um defeito**: o `bool_live` já filtra as órfãs ao cozinhar, então
/// a arte está certa sem esta varredura. O que ela cura é o DOCUMENTO — uma ligação morta que fica
/// gravada no save e reaparece, silenciosa, se um id for reciclado noutra sessão.
///
/// ⚠️ E ela só ESCREVE quando de facto apaga. O passo de undo é registado por diff de bytes: uma
/// varredura que reescrevesse o componente todo frame criaria um passo por frame, e desfazer
/// pareceria não fazer nada.
pub(crate) fn prune_dead_edges(sim: &mut SimWorld, scene: &ph2d_vec_scene::VecScene) -> bool {
    let vivos: std::collections::BTreeSet<VecPathId> = scene.paths().iter().map(|p| p.id).collect();
    let pos_mudou = prune_dead_positions(sim, &vivos);
    let mut q = sim.world_mut().query::<(Entity, &VecBoolEdges)>();
    let mortos: Vec<(Entity, VecBoolEdges)> = q
        .iter(sim.world())
        .filter_map(|(e, edges)| {
            let mut limpo = edges.clone();
            for id in edges
                .edges
                .iter()
                .flat_map(|l| [l.from, l.to])
                .filter(|id| !vivos.contains(id))
                .collect::<Vec<_>>()
            {
                limpo.forget(id);
            }
            (limpo != *edges).then_some((e, limpo))
        })
        .collect();
    let mudou = !mortos.is_empty() || pos_mudou;
    for (e, limpo) in mortos {
        sim.world_mut().entity_mut(e).insert(limpo);
    }
    mudou
}

/// **A operação seguinte no ciclo dos quatro** — o que um clique numa ligação faz.
///
/// ⚠️ Só as quatro de CONJUNTO entram no ciclo, e um código fora delas cai em `Union` em vez de
/// avançar: um clique tem de deixar a ligação num estado VÁLIDO, e girar dentro dos códigos
/// inválidos deixaria o artista preso a rodar sem nunca chegar a uma operação que desenha.
#[must_use]
pub(crate) fn next_op(op: u8) -> u8 {
    if op < 3 { op + 1 } else { 0 }
}

/// **A operação uniforme de um grafo**, ou `None` quando as ligações discordam.
///
/// É o que a seção Boolean do painel mostra: o verbo escolhido, ou *misto*. ⚠️ Um grafo SEM
/// ligações também devolve `None` — não há operação a mostrar, e afirmar uma seria inventar.
#[must_use]
pub(crate) fn uniform_op(sim: &SimWorld, g: Entity) -> Option<u8> {
    let edges = sim.world().get::<VecBoolEdges>(g)?;
    let first = edges.edges.first()?.op;
    edges.edges.iter().all(|e| e.op == first).then_some(first)
}

#[cfg(test)]
#[path = "bool_graph_ui_tests.rs"]
mod tests;
