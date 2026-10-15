//! ⭐⭐ **O DONO de cada nó** — o objecto Motion da cena a que ele pertence (doc ObjectsAndComponents
//! 01 §13, F5; escolha do dono 9.3: *«Motion vira objecto na árvore; pode haver vários»*).
//!
//! UM documento, e cada nó com dono — o molde da Animação da onda 2 (`NamedClip.owner`): 258
//! ficheiros contam com um `MotionDoc` só, o undo é um (`MotionHistory`) e o ficheiro guarda um campo.
//! O dono é o `StableId` do objecto (`u64`, nunca `0`); um nó SEM dono é o estado de um documento
//! de antes desta lei, e quem lhe dá um é a migração da app (`motion_objeto`). As molduras
//! (*backdrops*) também têm dono: desenhadas a editar um objecto, são do grafo dele.
//!
//! ## A secção `[owner]`
//!
//! ```text
//! [owner]
//! o <node_id> <stable_id>
//! ob <backdrop_id> <stable_id>
//! ```
//!
//! Escrita por ÚLTIMO e só quando há donos — um documento sem dono grava byte a byte o que sempre
//! gravou (fixturas `tests/fixtures/*_antes_do_dono.txt`, gravadas antes desta lei).

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use ph2d_nodegraph::format::ParseError;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

use crate::{Backdrop, MotionDoc, Subgraph};

/// nó → `StableId` do objecto Motion que é dono dele.
pub type Owners = BTreeMap<NodeId, u64>;

/// O cabeçalho da secção (sempre no início de uma linha).
pub(crate) const HEADER: &str = "\n[owner]";

/// Escreve a secção, ou nada sem donos.
pub(crate) fn write_section(out: &mut String, owners: &Owners, backdrops: &BTreeMap<u32, u64>) {
    if owners.is_empty() && backdrops.is_empty() {
        return;
    }
    out.push_str("[owner]\n");
    for (n, owner) in owners {
        let _ = writeln!(out, "o {} {owner}", n.0);
    }
    for (b, owner) in backdrops {
        let _ = writeln!(out, "ob {b} {owner}");
    }
}

/// Lê a secção (o texto DEPOIS do cabeçalho).
pub(crate) fn parse_section(part: &str) -> Result<(Owners, BTreeMap<u32, u64>), ParseError> {
    let mut owners = Owners::new();
    let mut backdrops: BTreeMap<u32, u64> = BTreeMap::new();
    for line in part.lines().map(str::trim).filter(|l| !l.is_empty()) {
        if line == "[owner]" {
            continue;
        }
        let mut tok = line.split_whitespace();
        let kind = tok.next();
        let id: Option<u32> = tok.next().and_then(|s| s.parse().ok());
        let owner: Option<u64> = tok.next().and_then(|s| s.parse().ok());
        let ok = match (kind, id, owner, tok.next()) {
            (Some("o"), Some(n), Some(o), None) if o != 0 => owners.insert(NodeId(n), o).is_none(),
            (Some("ob"), Some(b), Some(o), None) if o != 0 => backdrops.insert(b, o).is_none(),
            _ => false,
        };
        if !ok {
            return Err(ParseError::BadLine(line.into()));
        }
    }
    Ok((owners, backdrops))
}

/// ⛔ Um dono que nomeia um nó ou uma moldura que não existe morre AQUI, na fronteira.
pub(crate) fn validate(
    graph: &Graph,
    owners: &Owners,
    backdrops: &[Backdrop],
    backdrop_owners: &BTreeMap<u32, u64>,
) -> Result<(), ParseError> {
    if let Some(n) = owners.keys().find(|n| graph.node(**n).is_none()) {
        return Err(ParseError::BadLine(format!(
            "o {} — node does not exist",
            n.0
        )));
    }
    if let Some(b) = backdrop_owners
        .keys()
        .find(|b| !backdrops.iter().any(|x| x.id == **b))
    {
        return Err(ParseError::BadLine(format!(
            "ob {b} — backdrop does not exist"
        )));
    }
    Ok(())
}

impl MotionDoc {
    /// O dono de `node`, se tem.
    #[must_use]
    pub fn owner_of(&self, node: NodeId) -> Option<u64> {
        self.owners.get(&node).copied()
    }

    /// Os nós de `owner`, por ordem de id.
    #[must_use]
    pub fn nodes_of(&self, owner: u64) -> Vec<NodeId> {
        self.owners
            .iter()
            .filter(|(_, o)| **o == owner)
            .map(|(n, _)| *n)
            .collect()
    }

    /// Os donos que o documento nomeia (nós e molduras).
    #[must_use]
    pub fn owner_ids(&self) -> BTreeSet<u64> {
        self.owners
            .values()
            .chain(self.backdrop_owners.values())
            .copied()
            .collect()
    }

    /// Os nós do grafo que ainda não têm dono.
    #[must_use]
    pub fn unowned(&self) -> Vec<NodeId> {
        self.graph
            .nodes()
            .iter()
            .map(|n| n.id)
            .filter(|n| !self.owners.contains_key(n))
            .collect()
    }

    /// Dá `owner` a `node`. `owner == 0` não é dono (é o «nenhum» do `StableId`) e é recusado.
    pub fn set_owner(&mut self, node: NodeId, owner: u64) {
        if owner != 0 && self.graph.node(node).is_some() {
            self.owners.insert(node, owner);
        }
    }

    /// Dá `owner` a todo nó e moldura sem dono; devolve quantos nós adoptou.
    pub fn adopt_unowned(&mut self, owner: u64) -> usize {
        if owner == 0 {
            return 0;
        }
        let orphans = self.unowned();
        for n in &orphans {
            self.set_owner(*n, owner);
        }
        for b in &self.backdrops {
            self.backdrop_owners.entry(b.id).or_insert(owner);
        }
        orphans.len()
    }

    /// Passa nós e molduras de `from` para `to` (um dono que nunca existiu no mundo é re-adoptado).
    pub fn rehome(&mut self, from: u64, to: u64) -> usize {
        if to == 0 {
            return 0;
        }
        let nodes = self.nodes_of(from);
        for n in &nodes {
            self.set_owner(*n, to);
        }
        for o in self.backdrop_owners.values_mut() {
            if *o == from {
                *o = to;
            }
        }
        nodes.len()
    }

    /// ⛔ **O objecto morreu: os nós dele saem do grafo** (com as arestas, o lugar, os params e a
    /// filiação nos subgrafos), e as molduras dele também. Devolve os nós removidos.
    pub fn remove_owner(&mut self, owner: u64) -> Vec<NodeId> {
        let dead = self.nodes_of(owner);
        for n in &dead {
            self.graph.remove_node(*n);
        }
        self.forget_nodes(&dead);
        let gone: Vec<u32> = self
            .backdrop_owners
            .iter()
            .filter(|(_, o)| **o == owner)
            .map(|(b, _)| *b)
            .collect();
        self.backdrops.retain(|b| !gone.contains(&b.id));
        for b in &gone {
            self.backdrop_owners.remove(b);
            self.backdrop_members.remove(b);
        }
        dead
    }

    /// ⭐ **A cópia do objecto leva um grafo PRÓPRIO** (escolha do dono, 08/10 — o molde dos clips):
    /// cada nó de `from` ganha um gémeo de `to`, com os params, os textos, o rótulo, o bypass e o
    /// lugar; as arestas e os params dirigidos entre eles são refeitos entre os gémeos, e o que vinha
    /// de FORA continua a vir do mesmo nó. Os subgrafos que só têm nós de `from` e as molduras dele
    /// copiam-se também. Devolve original → gémeo.
    pub fn copy_owner(&mut self, from: u64, to: u64) -> BTreeMap<NodeId, NodeId> {
        let mut map: BTreeMap<NodeId, NodeId> = BTreeMap::new();
        if to == 0 || from == to {
            return map;
        }
        for n in self.nodes_of(from) {
            let Some(a) = Atributos::de(&self.graph, n) else {
                continue;
            };
            map.insert(n, self.planta(a, to));
        }
        let side = |n: NodeId| map.get(&n).copied().unwrap_or(n);
        let edges: Vec<Edge> = self
            .graph
            .edges()
            .iter()
            .filter(|e| map.contains_key(&e.to.0))
            .cloned()
            .collect();
        for e in edges {
            let _ = self.graph.connect(Edge {
                from: (side(e.from.0), e.from.1),
                to: (side(e.to.0), e.to.1),
                ..e
            });
        }
        let driven: Vec<(NodeId, String, NodeId, u16)> = map
            .keys()
            .filter_map(|n| self.graph.param_sources(*n).map(|s| (*n, s.clone())))
            .flat_map(|(n, s)| s.into_iter().map(move |(k, src)| (n, k, src.0, src.1)))
            .collect();
        for (n, k, src, port) in driven {
            let _ = self.graph.drive_param(side(n), k, (side(src), port));
        }
        self.copy_nesting(from, to, &map);
        map
    }

    /// **Os nós de `from` noutro documento entram neste como de `to`** (o tutorial «Recomeçar» volta o
    /// grafo do aluno ao estado inicial da trilha): cada nó com os atributos de [`Self::copy_owner`], os
    /// fios e os params dirigidos ENTRE eles. Os subgrafos e as molduras não viajam (são da vista).
    /// Devolve original (de `src`) → gémeo.
    pub fn import_owner(
        &mut self,
        src: &MotionDoc,
        from: u64,
        to: u64,
    ) -> BTreeMap<NodeId, NodeId> {
        let mut map: BTreeMap<NodeId, NodeId> = BTreeMap::new();
        for n in src.nodes_of(from) {
            if let Some(a) = Atributos::de(&src.graph, n) {
                map.insert(n, self.planta(a, to));
            }
        }
        for e in src.graph.edges() {
            if let (Some(a), Some(b)) = (map.get(&e.from.0), map.get(&e.to.0)) {
                let _ = self.graph.connect(Edge {
                    from: (*a, e.from.1),
                    to: (*b, e.to.1),
                    ..*e
                });
            }
        }
        for (n, twin) in &map {
            for (k, (s, port)) in src.graph.param_sources(*n).cloned().unwrap_or_default() {
                if let Some(s) = map.get(&s) {
                    let _ = self.graph.drive_param(*twin, k, (*s, port));
                }
            }
        }
        map
    }

    /// Um nó novo de `to` com os atributos `a`.
    fn planta(&mut self, a: Atributos, to: u64) -> NodeId {
        let twin = self.graph.add_node(a.tipo);
        for (k, v) in a.params {
            self.graph.set_param(twin, k, v);
        }
        for (k, v) in a.textos {
            self.graph.set_text_param(twin, k, v);
        }
        if let Some(label) = a.rotulo {
            self.graph.set_label(twin, label);
        }
        self.graph.set_bypassed(twin, a.bypass);
        if let Some(p) = a.lugar {
            self.graph.set_pos(twin, p);
        }
        self.owners.insert(twin, to);
        twin
    }

    /// Os subgrafos e as molduras de `from` ganham gémeos de `to` (ver [`Self::copy_owner`]).
    fn copy_nesting(&mut self, from: u64, to: u64, map: &BTreeMap<NodeId, NodeId>) {
        // Um subgrafo é de `from` quando todos os nós membros (a qualquer profundidade) são dele.
        let of_from: Vec<Subgraph> = self
            .subgraphs
            .iter()
            .filter(|s| {
                let deep = crate::subgraph::member_nodes_deep(&self.subgraphs, &self.members, s.id);
                !deep.is_empty() && deep.iter().all(|n| self.owner_of(*n) == Some(from))
            })
            .cloned()
            .collect();
        let mut sub_map: BTreeMap<u32, u32> = BTreeMap::new();
        for s in &of_from {
            let id = crate::subgraph::next_id(&self.subgraphs);
            sub_map.insert(s.id, id);
            self.subgraphs.push(Subgraph { id, ..s.clone() });
        }
        for s in self.subgraphs.iter_mut() {
            if sub_map.values().any(|v| *v == s.id)
                && let Some(p) = s.parent.and_then(|p| sub_map.get(&p))
            {
                s.parent = Some(*p);
            }
        }
        for (orig, twin) in map {
            if let Some(sid) = self.members.get(orig).copied() {
                self.members
                    .insert(*twin, sub_map.get(&sid).copied().unwrap_or(sid));
            }
        }
        let mine: Vec<Backdrop> = self
            .backdrops
            .iter()
            .filter(|b| self.backdrop_owners.get(&b.id) == Some(&from))
            .cloned()
            .collect();
        for b in mine {
            let id = self
                .backdrops
                .iter()
                .map(|x| x.id)
                .max()
                .map_or(0, |m| m + 1);
            if let Some(sid) = self.backdrop_members.get(&b.id).copied() {
                self.backdrop_members
                    .insert(id, sub_map.get(&sid).copied().unwrap_or(sid));
            }
            self.backdrops.push(Backdrop { id, ..b });
            self.backdrop_owners.insert(id, to);
        }
    }
}

/// O que o gémeo de um nó leva: o tipo, os params, os textos, o rótulo, o bypass e o lugar.
struct Atributos {
    tipo: String,
    params: BTreeMap<String, f32>,
    textos: BTreeMap<String, String>,
    rotulo: Option<String>,
    bypass: bool,
    lugar: Option<ph2d_nodegraph::graph::Pos>,
}

impl Atributos {
    fn de(g: &Graph, n: NodeId) -> Option<Self> {
        Some(Atributos {
            tipo: g.node(n)?.type_name.clone(),
            params: g.node_param_overrides(n).cloned().unwrap_or_default(),
            textos: g.node_text_param_overrides(n).cloned().unwrap_or_default(),
            rotulo: g.label(n).map(str::to_string),
            bypass: g.node_bypassed(n),
            lugar: g.pos(n),
        })
    }
}

#[cfg(test)]
#[path = "owner_tests.rs"]
mod tests;
