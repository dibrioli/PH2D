//! **O GRAFO da booleana viva** — a operação passa a ser da LIGAÇÃO, e não do grupo.
//!
//! # A pergunta que um grupo não sabe responder
//!
//! Um grupo booleano tem **uma** operação: os filhos dele combinam todos do mesmo jeito. O pedido
//! do artista — *"esta forma SOMA com aquela e SUBTRAI desta outra"* — não cabe nisso. Ele já é
//! exprimível hoje, aninhando grupos dentro de grupos (o [`crate::pathfinder`] compõe), mas aí a
//! relação está **escondida numa árvore**: para saber o que se combina com o quê é preciso abrir a
//! Hierarquia e reconstruir a intenção de cabeça.
//!
//! Aqui a relação é o próprio dado: um conjunto de nós (as formas) e um conjunto de ligações
//! **dirigidas**, cada uma com a sua operação.
//!
//! # A lei, em cinco frases
//!
//! 1. **A seta É a ordem do fold** — `from` OPERA, `to` RECEBE. É ela que resolve a assimetria do
//!    Subtract: `A−B ≠ B−A`, e círculos ligados por linhas *sem* direção não dizem por onde
//!    começar. Um diagrama sem seta desenha duas coisas diferentes com a mesma aparência.
//! 2. **Várias ligações a chegar no mesmo nó dobram na ordem de z do `from`** (fundo → topo). A
//!    régua não é nova: é a MESMA do [`crate::apply_many`], e é o que faz a lista de camadas
//!    continuar a explicar o resultado.
//! 3. **Só as quatro operações de CONJUNTO valem numa ligação.** As quatro receitas
//!    (`MinusBack`/`Trim`/`Crop`/`Merge`) são afirmações sobre uma PILHA inteira — *"cada forma
//!    menos a união do que está acima dela"* não é uma relação entre DOIS, e escrevê-la numa seta
//!    seria prometer o que o modelo não entrega. Elas continuam a ser do grupo.
//! 4. **Ciclo é RECUSA**, não um resultado esquisito: o grafo inteiro não cozinha e a arte fica
//!    exatamente como estava — a mesma lei que o `Err` do motor já tem na booleana viva.
//! 5. **Nó consumido desenha VAZIO; sumidouro desenha no PRÓPRIO id.** É a regra que a booleana
//!    viva já segue, com *"a base"* trocado por *"quem não opera sobre ninguém"* — e é o que faz o
//!    Apply não mover a arte um pixel.
//!
//! # A ESTRELA, e por que é ela que torna a migração segura
//!
//! [`derive_star`] escreve o grafo equivalente a um grupo de hoje: **todos apontam para a base**,
//! com a operação do grupo. O resultado é o mesmo — e não por sorte: o
//! [`crate::apply_many_checked`] **já é um fold binário da esquerda para a direita**, e a estrela
//! reproduz exatamente esse fold, na mesma ordem, com o mesmo doador de estilo (o último dobrado,
//! que na estrela é o operando do topo).
//!
//! O gate `a_estrela_derivada_desenha_o_que_o_grupo_de_hoje_desenha` prende os dois com
//! `assert_eq!` sobre a geometria inteira. Sem essa igualdade, **abrir a janela do grafo sobre um
//! grupo existente moveria a arte no instante em que o artista olhasse para ela** — que é o defeito
//! que uma feature de visualização nunca pode ter.
//!
//! # Uma divergência declarada: o nó com VÁRIAS geometrias
//!
//! Um operando pode chegar com mais de um caminho (é o que um pattern-along-path ou um offset vivo
//! produzem). O [`crate::apply_many`] os dobra **com a operação do grupo**, como se fossem
//! operandos independentes — num grupo Subtract, as cópias de um pattern subtraem-se umas às
//! outras antes de a operação do artista acontecer.
//!
//! Aqui não: as geometrias de um nó são **a forma dele**, e unem-se entre si (`Union`) antes de
//! qualquer ligação. Um círculo no diagrama é um objeto, e um objeto não opera sobre si mesmo.
//! ⚠️ A divergência é real e está gateada (`as_geometrias_de_um_no_sao_uma_forma_so`); ela não
//! alcança o caso de um caminho por nó, que é o que a igualdade da estrela cobre.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use kurbo::BezPath;
use linesweeper::FillRule as LsFillRule;
use ph2d_vec_scene::{FillRule, VecPath};

use crate::{
    BoolOp, PathfinderOp, SweepFailed, binary_grouped_checked, compound_from, flatten_groups,
    to_bez,
};

/// **Uma ligação dirigida do grafo**: `from` OPERA sobre `to`, que RECEBE.
///
/// Os ids são `VecPathId` crus (`u64`) — o mesmo `u64` que o `VecPathRef` do ECS carrega, e pela
/// mesma razão: quem guarda a relação não pode depender de quem guarda a geometria.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BoolEdge {
    /// Quem OPERA — o operando consumido.
    pub from: u64,
    /// Quem RECEBE — o nó cuja geometria é reescrita pela operação.
    pub to: u64,
    /// A operação desta ligação. Tem de ser de CONJUNTO (ver [`GraphRefusal::NotBinary`]).
    pub op: PathfinderOp,
}

/// **Por que o grafo não cozinhou.** Toda variante é uma recusa INTEIRA — o grafo é uma resposta
/// só, e desenhar metade dele seria mostrar arte que nenhuma leitura do diagrama explica.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GraphRefusal {
    /// Uma ligação nomeia um id que não está na lista de nós. É o que acontece quando uma forma é
    /// apagada e a ligação dela sobrevive — a limpeza é de quem escreve o grafo, e a recusa aqui
    /// é a rede que impede o motor de inventar uma resposta com um operando a menos.
    UnknownNode(u64),
    /// A ligação carrega uma das quatro RECEITAS. Ver a lei 3 no topo do módulo.
    NotBinary(PathfinderOp),
    /// Há um ciclo (uma forma que, por algum caminho, opera sobre si mesma).
    Cycle,
    /// O motor exato recusou a entrada.
    Sweep(SweepFailed),
}

impl std::fmt::Display for GraphRefusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GraphRefusal::UnknownNode(id) => {
                write!(f, "a ligacao nomeia a forma {id}, que nao esta no grupo")
            }
            GraphRefusal::NotBinary(op) => write!(
                f,
                "{op:?} e' uma receita sobre a pilha, nao uma relacao entre duas formas"
            ),
            GraphRefusal::Cycle => f.write_str("ha um ciclo: uma forma opera sobre si mesma"),
            GraphRefusal::Sweep(e) => write!(f, "{e}"),
        }
    }
}

/// **O grafo equivalente a um grupo de hoje** — todos os operandos apontam para a BASE (o primeiro
/// da lista, que é o mais ao fundo), com a operação do grupo.
///
/// É a porta da migração: materializar isto sobre um grupo existente e resolver com
/// [`resolve_graph`] desenha exatamente o que o [`crate::pathfinder`] desenhava. Ver a secção *A
/// ESTRELA* no topo do módulo.
#[must_use]
pub fn derive_star(nodes: &[u64], op: PathfinderOp) -> Vec<BoolEdge> {
    match nodes.split_first() {
        Some((base, rest)) => rest
            .iter()
            .map(|from| BoolEdge {
                from: *from,
                to: *base,
                op,
            })
            .collect(),
        None => Vec::new(),
    }
}

/// **O grafo morde-se a si próprio?** — a mesma pergunta que [`resolve_graph`] faz, sem geometria.
///
/// Existe para quem precisa da resposta **sem cozinhar**: o diagrama tem de DIZER na tela que há um
/// ciclo, e no motor ele é uma recusa silenciosa (a arte fica como estava e nada explica por quê).
///
/// ⚠️ **Ela corre o MESMO Kahn** do resolvedor, e é isso que impede as duas respostas de
/// divergirem. Uma segunda deteção — uma busca em profundidade escrita à parte — concordaria com
/// esta em quase todo grafo e discordaria exatamente nos casos raros, que é onde ninguém olha.
///
/// Uma ligação que nomeia um nó ausente é IGNORADA aqui (ela é um problema de limpeza, não um
/// ciclo); o `resolve_graph` continua a recusá-la, e essa recusa é dele.
#[must_use]
pub fn has_cycle(node_ids: &[u64], edges: &[BoolEdge]) -> bool {
    let known: BTreeSet<u64> = node_ids.iter().copied().collect();
    let mut incoming: BTreeMap<u64, Vec<(u64, BoolOp)>> = BTreeMap::new();
    for e in edges {
        if !known.contains(&e.from) || !known.contains(&e.to) {
            continue;
        }
        if e.from == e.to {
            return true;
        }
        incoming
            .entry(e.to)
            .or_default()
            .push((e.from, BoolOp::Union));
    }
    let nodes: Vec<(u64, Vec<VecPath>)> = node_ids.iter().map(|id| (*id, Vec::new())).collect();
    topological(&nodes, &incoming).is_err()
}

/// **Resolve o grafo**: o que cada nó desenha.
///
/// `nodes` em ordem de **z (fundo → topo)**, tudo em MUNDO, ids únicos. A saída tem uma entrada
/// por nó, na mesma ordem: a geometria do sumidouro, ou uma lista **vazia** para quem foi
/// consumido (a lei 5).
///
/// # Errors
/// [`GraphRefusal`] — ver as variantes. Numa recusa **nada** é desenhado diferente: quem chama
/// deixa o mapa intocado.
pub fn resolve_graph(
    nodes: &[(u64, Vec<VecPath>)],
    edges: &[BoolEdge],
) -> Result<Vec<(u64, Vec<VecPath>)>, GraphRefusal> {
    let z: BTreeMap<u64, usize> = nodes
        .iter()
        .enumerate()
        .map(|(i, (id, _))| (*id, i))
        .collect();
    let mut incoming: BTreeMap<u64, Vec<(u64, BoolOp)>> = BTreeMap::new();
    let mut consumed: BTreeSet<u64> = BTreeSet::new();
    for e in edges {
        for id in [e.from, e.to] {
            if !z.contains_key(&id) {
                return Err(GraphRefusal::UnknownNode(id));
            }
        }
        let Some(b) = e.op.as_bool() else {
            return Err(GraphRefusal::NotBinary(e.op));
        };
        if e.from == e.to {
            // O laço de um nó consigo mesmo é um ciclo de comprimento 1, e o Kahn abaixo o
            // apanharia na mesma — mas apanhá-lo aqui poupa a caminhada e nomeia a causa.
            return Err(GraphRefusal::Cycle);
        }
        incoming.entry(e.to).or_default().push((e.from, b));
        consumed.insert(e.from);
    }
    // A ordem das ligações que chegam é a de z de QUEM OPERA — nunca a ordem em que elas foram
    // escritas. É isso que torna a lista guardada uma questão cosmética: reordená-la no disco não
    // pode mudar o desenho.
    for v in incoming.values_mut() {
        v.sort_by_key(|(from, _)| z.get(from).copied().unwrap_or(0));
    }
    let order = topological(nodes, &incoming)?;

    let mut done: BTreeMap<u64, Resolved> = BTreeMap::new();
    for id in order {
        let paths = &nodes[z[&id]].1;
        let mut r = seed(paths)?;
        for (from, op) in incoming.get(&id).into_iter().flatten() {
            let other = &done[from];
            let rule = mix(r.rule, other.rule);
            let g = binary_grouped_checked(&r.bez, &other.bez, rule, op.to_linesweeper())
                .map_err(GraphRefusal::Sweep)?;
            r.bez = flatten_groups(&g);
            r.grouped = Some(g);
            r.rule = FillRule::NonZero;
            // O ÚLTIMO dobrado doa o estilo — a lei do `apply_many`, e a razão pela qual a estrela
            // derivada veste a roupa do operando do TOPO, como o Illustrator.
            if other.style.is_some() {
                r.style.clone_from(&other.style);
            }
        }
        done.insert(id, r);
    }

    Ok(nodes
        .iter()
        .map(|(id, paths)| {
            let r = &done[id];
            let out = match (consumed.contains(id), &r.grouped, &r.style) {
                // Consumido: desenha nada. A geometria dele vive dentro do resultado de quem o
                // recebeu, e desenhá-la outra vez seria mostrá-la duas vezes.
                (true, _, _) => Vec::new(),
                // Sumidouro sem ligação de entrada: a forma desenha-se a si própria, verbatim.
                // ⚠️ Verbatim é de propósito — passar pelo motor um caminho que ninguém operou
                // trocaria a geometria autorada por uma varredura dela.
                (false, None, _) => paths.clone(),
                (false, Some(g), Some(style)) => {
                    g.iter().filter_map(|c| compound_from(c, style)).collect()
                }
                (false, Some(_), None) => Vec::new(),
            };
            (*id, out)
        })
        .collect())
}

/// O estado de um nó a meio da resolução.
struct Resolved {
    /// A geometria acumulada, achatada — o que a próxima operação consome.
    bez: BezPath,
    /// A regra de preenchimento do acumulador.
    rule: FillRule,
    /// De quem o resultado veste a roupa. `None` = o nó não tem geometria nenhuma.
    style: Option<VecPath>,
    /// Os contornos agrupados por containment da ÚLTIMA operação. `None` = nenhuma operação
    /// aconteceu, e o nó desenha a geometria autorada.
    grouped: Option<Vec<Vec<BezPath>>>,
}

/// A geometria PRÓPRIA de um nó, antes de qualquer ligação: as suas partes unidas numa forma só.
///
/// ⚠️ `Union`, e não a operação de nenhuma ligação — ver *"Uma divergência declarada"* no topo.
fn seed(paths: &[VecPath]) -> Result<Resolved, GraphRefusal> {
    let Some(first) = paths.first() else {
        return Ok(Resolved {
            bez: BezPath::new(),
            rule: FillRule::NonZero,
            style: None,
            grouped: None,
        });
    };
    let mut bez = to_bez(first);
    let mut rule = first.fill_rule;
    for other in &paths[1..] {
        let r = mix(rule, other.fill_rule);
        let g = binary_grouped_checked(&bez, &to_bez(other), r, BoolOp::Union.to_linesweeper())
            .map_err(GraphRefusal::Sweep)?;
        bez = flatten_groups(&g);
        rule = FillRule::NonZero;
    }
    Ok(Resolved {
        bez,
        rule,
        style: Some(first.clone()),
        grouped: None,
    })
}

/// A regra de preenchimento com que duas geometrias entram no motor. Cópia deliberada da do
/// [`crate::apply_many_checked`]: só um compound depende de `EvenOdd`, e um contorno único lê igual
/// sob as duas.
fn mix(a: FillRule, b: FillRule) -> LsFillRule {
    if a == FillRule::EvenOdd || b == FillRule::EvenOdd {
        LsFillRule::EvenOdd
    } else {
        LsFillRule::NonZero
    }
}

/// A ordem de resolução: um nó só é resolvido depois de todos os que operam sobre ele (Kahn).
///
/// ⚠️ **O ciclo é visto pela CONTAGEM, não por uma busca** — se a caminhada não alcançou todos os
/// nós, os que ficaram de fora estão presos num ciclo. É a deteção que não precisa de um segundo
/// percurso, e não tem como discordar do resolvedor: é o mesmo passo.
fn topological(
    nodes: &[(u64, Vec<VecPath>)],
    incoming: &BTreeMap<u64, Vec<(u64, BoolOp)>>,
) -> Result<Vec<u64>, GraphRefusal> {
    let mut pending: BTreeMap<u64, usize> = nodes
        .iter()
        .map(|(id, _)| (*id, incoming.get(id).map_or(0, Vec::len)))
        .collect();
    let mut dependents: BTreeMap<u64, Vec<u64>> = BTreeMap::new();
    for (to, ins) in incoming {
        for (from, _) in ins {
            dependents.entry(*from).or_default().push(*to);
        }
    }
    // A fila nasce em ordem de z porque `nodes` está em ordem de z — a resolução é determinista
    // entre frames sem uma segunda ordenação.
    let mut queue: VecDeque<u64> = nodes
        .iter()
        .map(|(id, _)| *id)
        .filter(|id| pending[id] == 0)
        .collect();
    let mut out = Vec::with_capacity(nodes.len());
    while let Some(id) = queue.pop_front() {
        out.push(id);
        for to in dependents.get(&id).into_iter().flatten() {
            let Some(p) = pending.get_mut(to) else {
                continue;
            };
            *p -= 1;
            if *p == 0 {
                queue.push_back(*to);
            }
        }
    }
    if out.len() == nodes.len() {
        Ok(out)
    } else {
        Err(GraphRefusal::Cycle)
    }
}

#[cfg(test)]
#[path = "graph_tests.rs"]
mod tests;
