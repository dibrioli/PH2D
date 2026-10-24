//! O [`Mapa`] da [`super`]: cada id da trilha (nó, grupo, moldura, `Passo::Documento`) reescrito ao bit
//! do documento de origem para o destino; o que não se reescreve recusa-se pelo nome.

use super::super::passo::{Alvo, Escrita, Intencao, IntencaoDeParam, Nomeado, Passo};
use super::super::{Destino, documento_de};
use super::{E, em_grupo_desligado, fios_so_do_dono, vista_de_antes};
use ph2d_motion_doc::subgraph::Subgraph;
use ph2d_motion_doc::{Backdrop, MotionDoc};
use ph2d_nodegraph::graph::{Edge, NodeId};
use std::collections::BTreeMap;

/// Os ids de uma trilha no documento de origem → no destino.
pub(super) struct Mapa<'a> {
    pub(super) ini: &'a MotionDoc,
    pub(super) destino: &'a MotionDoc,
    pub(super) dono: u64,
    pub(super) novo: u64,
    /// Os nós do dono ao armar → os gémeos no destino.
    pub(super) nos: BTreeMap<NodeId, NodeId>,
    pub(super) base: u32,
    pub(super) base_nova: u32,
    /// O próximo id de grupo (e de moldura) na origem e no destino.
    pub(super) grupos: (u32, u32),
    pub(super) fundos: (u32, u32),
}

fn desloca(id: u32, (de, para): (u32, u32)) -> Result<u32, E> {
    id.checked_sub(de)
        .and_then(|k| para.checked_add(k))
        .ok_or(E::VistaDeAntes(id))
}

impl Mapa<'_> {
    fn grupo(&self, s: u32) -> Result<u32, E> {
        desloca(s, self.grupos)
    }

    fn fundo(&self, b: u32) -> Result<u32, E> {
        desloca(b, self.fundos)
    }

    /// Um id cru de nó: um criado anda com a base; um de antes é um do dono.
    fn no_cru(&self, id: u32) -> Result<u32, E> {
        if id >= self.base {
            self.base_nova
                .checked_add(id - self.base)
                .ok_or(E::DocumentoSemTexto)
        } else {
            self.nos
                .get(&NodeId(id))
                .map(|n| n.0)
                .ok_or(E::DoutroDono(id))
        }
    }

    fn alvo(&self, a: Alvo) -> Result<Alvo, E> {
        Ok(match a {
            Alvo::Existente(id) => Alvo::Existente(self.no_cru(id)?),
            Alvo::Criado(k) => Alvo::Criado(k),
            Alvo::Grupo(s) => Alvo::Grupo(self.grupo(s)?),
        })
    }

    fn alvos(&self, v: &[Alvo]) -> Result<Vec<Alvo>, E> {
        v.iter().map(|a| self.alvo(*a)).collect()
    }

    /// A carga reescrita (a mesma, ao bit, quando nada muda).
    pub(super) fn carga(&self, c: &[u8]) -> Result<Vec<u8>, E> {
        let p = Passo::descodifica(c).map_err(E::Passo)?;
        let q = self.passo(&p)?;
        Ok(if q == p { c.to_vec() } else { q.codifica() })
    }

    fn passo(&self, p: &Passo) -> Result<Passo, E> {
        Ok(match p {
            Passo::Grafo {
                nivel,
                ajuda,
                intencoes,
            } => Passo::Grafo {
                nivel: nivel.map(|s| self.grupo(s)).transpose()?,
                ajuda: *ajuda,
                intencoes: intencoes
                    .iter()
                    .map(|i| self.intencao(i))
                    .collect::<Result<_, _>>()?,
            },
            Passo::Params { destino, intencoes } => Passo::Params {
                destino: match destino {
                    Destino::No => Destino::No,
                    Destino::Fundo(b) => Destino::Fundo(self.fundo(*b)?),
                    Destino::Cartao(s) => Destino::Cartao(self.grupo(*s)?),
                },
                intencoes: intencoes
                    .iter()
                    .map(|i| self.param(i))
                    .collect::<Result<_, _>>()?,
            },
            Passo::Escritas(v) => Passo::Escritas(
                v.iter()
                    .map(|e| self.escrita(e))
                    .collect::<Result<_, _>>()?,
            ),
            Passo::Documento {
                texto,
                proximo_id,
                arestas,
            } => self.documento(texto, *proximo_id, arestas)?,
            Passo::Toque(_) | Passo::Quadro { .. } | Passo::Recusada(_) => p.clone(),
            Passo::Desconhecido(n) => return Err(E::Desconhecido(*n)),
        })
    }

    fn param(&self, i: &IntencaoDeParam) -> Result<IntencaoDeParam, E> {
        use IntencaoDeParam as P;
        Ok(match i {
            P::SetParam { node, param, value } => P::SetParam {
                node: self.alvo(*node)?,
                param: param.clone(),
                value: *value,
            },
            P::SetTextParam { node, param, value } => P::SetTextParam {
                node: self.alvo(*node)?,
                param: param.clone(),
                value: value.clone(),
            },
            P::ResetParam { node, param } => P::ResetParam {
                node: self.alvo(*node)?,
                param: param.clone(),
            },
            P::Desconhecida(n) => return Err(E::Desconhecido(*n)),
        })
    }

    fn escrita(&self, e: &Escrita) -> Result<Escrita, E> {
        Ok(match e {
            Escrita::Valor { node, param, valor } => Escrita::Valor {
                node: self.alvo(*node)?,
                param: param.clone(),
                valor: *valor,
            },
            Escrita::Texto { node, param, valor } => Escrita::Texto {
                node: self.alvo(*node)?,
                param: param.clone(),
                valor: valor.clone(),
            },
        })
    }

    fn fio(&self, w: &(Alvo, u16)) -> Result<(Alvo, u16), E> {
        Ok((self.alvo(w.0)?, w.1))
    }

    fn intencao(&self, i: &Intencao) -> Result<Intencao, E> {
        use Intencao as I;
        let al = |a: &Alvo| self.alvo(*a);
        Ok(match i {
            I::BeginDrag
            | I::EndDrag
            | I::AddNode { .. }
            | I::AddBackdrop { .. }
            | I::ArrangeLayout
            | I::Paste
            | I::SetNodeHelp(_)
            | I::GoToLevel(None) => i.clone(),
            I::MoveNodes { nodes, dx, dy } => I::MoveNodes {
                nodes: self.alvos(nodes)?,
                dx: *dx,
                dy: *dy,
            },
            I::DriveParam {
                from,
                from_port,
                to,
                param,
            } => I::DriveParam {
                from: al(from)?,
                from_port: *from_port,
                to: al(to)?,
                param: param.clone(),
            },
            I::PickSelection {
                node,
                param,
                escolha,
            } => I::PickSelection {
                node: al(node)?,
                param: param.clone(),
                escolha: escolha.clone(),
            },
            I::Connect {
                from,
                from_port,
                to,
                to_port,
            } => I::Connect {
                from: al(from)?,
                from_port: *from_port,
                to: al(to)?,
                to_port: *to_port,
            },
            I::Disconnect { to, to_port } => I::Disconnect {
                to: al(to)?,
                to_port: *to_port,
            },
            I::DeleteSelection(v) => I::DeleteSelection(self.alvos(v)?),
            I::MoveBackdrop { id, dx, dy } => I::MoveBackdrop {
                id: self.fundo(*id)?,
                dx: *dx,
                dy: *dy,
            },
            I::ResizeBackdrop { id, left, dx, dy } => I::ResizeBackdrop {
                id: self.fundo(*id)?,
                left: *left,
                dx: *dx,
                dy: *dy,
            },
            I::DeleteBackdrop(b) => I::DeleteBackdrop(self.fundo(*b)?),
            I::Rename { alvo, nome } => I::Rename {
                alvo: match alvo {
                    Nomeado::No(a) => Nomeado::No(al(a)?),
                    Nomeado::Grupo(s) => Nomeado::Grupo(self.grupo(*s)?),
                    Nomeado::Fundo(b) => Nomeado::Fundo(self.fundo(*b)?),
                },
                nome: nome.clone(),
            },
            I::SetBackdropColor { id, cor } => I::SetBackdropColor {
                id: self.fundo(*id)?,
                cor: *cor,
            },
            I::DuplicateSelection(v) => I::DuplicateSelection(self.alvos(v)?),
            I::SmartConnect {
                from,
                from_port,
                tipo,
                x,
                y,
            } => I::SmartConnect {
                from: al(from)?,
                from_port: *from_port,
                tipo: tipo.clone(),
                x: *x,
                y: *y,
            },
            I::SwapInChain {
                a,
                b,
                back_dx,
                back_dy,
            } => I::SwapInChain {
                a: al(a)?,
                b: al(b)?,
                back_dx: *back_dx,
                back_dy: *back_dy,
            },
            I::SpliceExistingIntoWire { node, to, to_port } => I::SpliceExistingIntoWire {
                node: al(node)?,
                to: al(to)?,
                to_port: *to_port,
            },
            I::SmartConnectBack {
                to,
                to_port,
                tipo,
                x,
                y,
            } => I::SmartConnectBack {
                to: al(to)?,
                to_port: *to_port,
                tipo: tipo.clone(),
                x: *x,
                y: *y,
            },
            I::CutWires(v) => I::CutWires(v.iter().map(|w| self.fio(w)).collect::<Result<_, _>>()?),
            I::MoveWireEnd {
                from,
                from_port,
                old_to,
                old_to_port,
                new_to,
            } => I::MoveWireEnd {
                from: al(from)?,
                from_port: *from_port,
                old_to: al(old_to)?,
                old_to_port: *old_to_port,
                new_to: new_to.as_ref().map(|w| self.fio(w)).transpose()?,
            },
            I::GroupSelection(v) => I::GroupSelection(self.alvos(v)?),
            I::Ungroup(s) => I::Ungroup(self.grupo(*s)?),
            I::EnterSubgraph(s) => I::EnterSubgraph(self.grupo(*s)?),
            I::GoToLevel(Some(s)) => I::GoToLevel(Some(self.grupo(*s)?)),
            I::SpliceReroute { to, to_port, x, y } => I::SpliceReroute {
                to: al(to)?,
                to_port: *to_port,
                x: *x,
                y: *y,
            },
            I::SpliceNode {
                to,
                to_port,
                tipo,
                x,
                y,
            } => I::SpliceNode {
                to: al(to)?,
                to_port: *to_port,
                tipo: tipo.clone(),
                x: *x,
                y: *y,
            },
            I::SetBypass { nodes, on } => I::SetBypass {
                nodes: self.alvos(nodes)?,
                on: *on,
            },
            I::CopySelection(v) => I::CopySelection(self.alvos(v)?),
            I::FixInert(a) => I::FixInert(al(a)?),
            I::Desconhecida(n) => return Err(E::Desconhecido(*n)),
        })
    }

    /// **O documento de um passo, só com o grafo do dono**: o destino como estava ao aplicar, mais os nós
    /// do dono nesse documento com os ids que os passos seguintes nomeiam, os fios entre eles e a vista
    /// criada a gravar.
    fn documento(&self, texto: &str, proximo_id: u32, arestas: &[Edge]) -> Result<Passo, E> {
        let d = documento_de(texto, proximo_id, arestas).map_err(E::Documento)?;
        let (g0, f0) = (self.grupos.0, self.fundos.0);
        if proximo_id < self.base || vista_de_antes(&d, g0, f0) != vista_de_antes(self.ini, g0, f0)
        {
            return Err(E::DocumentoDeAntes);
        }
        let e_do_dono = |n: NodeId| d.owner_of(n) == Some(self.dono);
        fios_so_do_dono(&d, e_do_dono)?;
        let mut nos = Vec::new();
        for n in d.graph.nodes() {
            if e_do_dono(n.id) {
                if em_grupo_desligado(&d, n.id, g0) {
                    return Err(E::GrupoDesligado);
                }
                nos.push((n.id, NodeId(self.no_cru(n.id.0)?)));
            } else if n.id.0 >= self.base {
                return Err(E::DoutroDono(n.id.0));
            }
        }
        nos.sort_by_key(|(_, para)| para.0);
        let mut o = self.destino.clone();
        for (de, para) in &nos {
            if !o.import_node_as(&d, *de, *para, self.novo) {
                return Err(E::DocumentoSemTexto);
            }
        }
        let mapa: BTreeMap<NodeId, NodeId> = nos.iter().copied().collect();
        for e in d.graph.edges() {
            if let (Some(a), Some(b)) = (mapa.get(&e.from.0), mapa.get(&e.to.0)) {
                o.graph
                    .connect(Edge {
                        from: (*a, e.from.1),
                        to: (*b, e.to.1),
                        ..*e
                    })
                    .map_err(|_| E::DocumentoSemTexto)?;
            }
        }
        for (de, para) in &nos {
            for (k, (s, porta)) in d.graph.param_sources(*de).cloned().unwrap_or_default() {
                let s = *mapa.get(&s).ok_or(E::FioDeFora)?;
                o.graph
                    .drive_param(*para, k, (s, porta))
                    .map_err(|_| E::DocumentoSemTexto)?;
            }
            if let Some(s) = d.members.get(de).filter(|s| **s >= g0) {
                o.members.insert(*para, self.grupo(*s)?);
            }
        }
        self.vista_criada(&d, &mut o)?;
        if d.arrumados_a_mao.contains(&self.dono) {
            o.arrumados_a_mao.insert(self.novo);
        } else if self.ini.arrumados_a_mao.contains(&self.dono) {
            o.arrumados_a_mao.remove(&self.novo);
        }
        if d.base_z != self.ini.base_z {
            o.base_z = d.base_z;
        }
        let proximo = self.no_cru(proximo_id)?;
        o.graph.restore_next_id(proximo);
        let texto = o.to_text();
        let arestas = o.graph.edges().to_vec();
        match documento_de(&texto, proximo, &arestas) {
            Ok(x) if x == o => Ok(Passo::Documento {
                texto,
                proximo_id: proximo,
                arestas,
            }),
            _ => Err(E::DocumentoSemTexto),
        }
    }

    /// Os grupos e as molduras que a gravação criou, nos ids do destino.
    fn vista_criada(&self, d: &MotionDoc, o: &mut MotionDoc) -> Result<(), E> {
        let (g0, f0) = (self.grupos.0, self.fundos.0);
        for s in d.subgraphs.iter().filter(|s| s.id >= g0) {
            o.subgraphs.push(Subgraph {
                id: self.grupo(s.id)?,
                parent: s.parent.map(|p| self.grupo(p)).transpose()?,
                ..s.clone()
            });
        }
        for s in d.bypassed_subgraphs.iter().filter(|s| **s >= g0) {
            o.bypassed_subgraphs.insert(self.grupo(*s)?);
        }
        for b in d.backdrops.iter().filter(|b| b.id >= f0) {
            let id = self.fundo(b.id)?;
            match d.backdrop_owners.get(&b.id) {
                Some(x) if *x == self.dono => {
                    o.backdrop_owners.insert(id, self.novo);
                }
                None => {}
                Some(_) => return Err(E::DoutroDono(b.id)),
            }
            if let Some(s) = d.backdrop_members.get(&b.id) {
                o.backdrop_members.insert(id, self.grupo(*s)?);
            }
            o.backdrops.push(Backdrop { id, ..b.clone() });
        }
        Ok(())
    }
}
