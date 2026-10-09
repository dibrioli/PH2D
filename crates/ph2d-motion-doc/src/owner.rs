//! ⭐⭐ **O DONO de cada nó** — o objecto Motion da cena a que ele pertence (doc ObjectsAndComponents
//! 01 §13, F5; escolha do dono 9.3: *«Motion vira objecto na árvore; pode haver vários»*).
//!
//! UM documento, e cada nó com dono — o molde da Animação da onda 2 (`NamedClip.owner`): 258
//! ficheiros contam com um `MotionDoc` só, o undo é um (`MotionHistory`) e o ficheiro guarda um campo.
//! O dono é o `StableId` do objecto (`u64`, nunca `0`); um nó SEM dono é o estado de um documento
//! de antes desta lei, e quem lhe dá um é a migração da app (`motion_objeto`).
//!
//! ## A secção `[owner]`
//!
//! ```text
//! [owner]
//! o <node_id> <stable_id>
//! ```
//!
//! Escrita por ÚLTIMO e só quando há donos — um documento sem dono grava byte a byte o que sempre
//! gravou (fixturas `tests/fixtures/*_antes_do_dono.txt`, gravadas antes desta lei).

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use ph2d_nodegraph::format::ParseError;
use ph2d_nodegraph::graph::{Graph, NodeId};

use crate::MotionDoc;

/// nó → `StableId` do objecto Motion que é dono dele.
pub type Owners = BTreeMap<NodeId, u64>;

/// O cabeçalho da secção (sempre no início de uma linha).
pub(crate) const HEADER: &str = "\n[owner]";

/// Escreve a secção, ou nada sem donos.
pub(crate) fn write_section(out: &mut String, owners: &Owners) {
    if owners.is_empty() {
        return;
    }
    out.push_str("[owner]\n");
    for (n, owner) in owners {
        let _ = writeln!(out, "o {} {owner}", n.0);
    }
}

/// Lê a secção (o texto DEPOIS do cabeçalho).
pub(crate) fn parse_section(part: &str) -> Result<Owners, ParseError> {
    let mut owners = Owners::new();
    for line in part.lines().map(str::trim).filter(|l| !l.is_empty()) {
        let Some(rest) = line.strip_prefix("o ") else {
            if line == "[owner]" {
                continue;
            }
            return Err(ParseError::BadLine(line.into()));
        };
        let mut tok = rest.split_whitespace();
        let node: Option<u32> = tok.next().and_then(|s| s.parse().ok());
        let owner: Option<u64> = tok.next().and_then(|s| s.parse().ok());
        match (node, owner, tok.next()) {
            (Some(n), Some(o), None) if o != 0 && owners.insert(NodeId(n), o).is_none() => {}
            _ => return Err(ParseError::BadLine(line.into())),
        }
    }
    Ok(owners)
}

/// ⛔ Um dono que nomeia um nó que não existe morre AQUI, na fronteira.
pub(crate) fn validate(graph: &Graph, owners: &Owners) -> Result<(), ParseError> {
    for n in owners.keys() {
        if graph.node(*n).is_none() {
            return Err(ParseError::BadLine(format!("o {} — node does not exist", n.0)));
        }
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

    /// Os donos que o documento nomeia.
    #[must_use]
    pub fn owner_ids(&self) -> BTreeSet<u64> {
        self.owners.values().copied().collect()
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

    /// Dá `owner` a todo nó sem dono; devolve quantos adoptou.
    pub fn adopt_unowned(&mut self, owner: u64) -> usize {
        let orphans = self.unowned();
        for n in &orphans {
            self.set_owner(*n, owner);
        }
        orphans.len()
    }

    /// Passa os nós de `from` para `to` (um dono que nunca existiu no mundo é re-adoptado).
    pub fn rehome(&mut self, from: u64, to: u64) -> usize {
        let nodes = self.nodes_of(from);
        for n in &nodes {
            self.set_owner(*n, to);
        }
        nodes.len()
    }

    /// ⛔ **O objecto morreu: os nós dele saem do grafo** (com as arestas, o lugar, os params e a
    /// filiação nos subgrafos). Devolve os nós removidos.
    pub fn remove_owner(&mut self, owner: u64) -> Vec<NodeId> {
        let dead = self.nodes_of(owner);
        for n in &dead {
            self.graph.remove_node(*n);
        }
        self.forget_nodes(&dead);
        dead
    }
}

#[cfg(test)]
#[path = "owner_tests.rs"]
mod tests;
