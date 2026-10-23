//! **A trilha do Motion noutro documento** (W18, `docs/Macros/16_W18_plano.md` §2.5 e §2.7): o mesmo
//! gesto gravado, com o estado inicial reescrito sobre OUTRO documento e outro dono.
//!
//! - [`so_do_dono`] — exportar: o inicial leva só o grafo do dono (o resto do projecto não viaja).
//! - [`aplica`] — aplicar (novo ou por cima): os nós da macro juntam-se aos do dono novo no documento
//!   vivo, e o fim da reprodução é o documento que o editor passa a mostrar.
//!
//! Uma porta só reescreve ([`Mapa`]): cada `Alvo::Existente` pelo mapa do `import_owner`, a base dos
//! criados pelo `next_id` novo, os grupos e as molduras criados a gravar pelo deslocamento do próximo
//! id, e cada `Passo::Documento` reduzido ao grafo do dono com esses ids. O que não se reescreve AO BIT
//! recusa-se pelo nome ([`ErroAoTranspor`]) — nunca se larga um passo em silêncio. Os grupos e as
//! molduras que já existiam ao armar não viajam (são da vista, a lei do `import_owner`).

use super::etapas::LeitorDoMotion;
use super::inicial::{self, ErroDoInicial, Inicial};
use super::passo::{Alvo, Escrita, Intencao, IntencaoDeParam, Nomeado, Passo};
use super::tocar::{ErroDaMacro, Modo, TocadorDoMotion};
use super::{Destino, FAMILIA, documento_de};
use ph2d_macro::{CodecError, LeitorDaFamilia, Track};
use ph2d_motion_doc::subgraph::{self, Subgraph};
use ph2d_motion_doc::{Backdrop, MotionDoc};
use ph2d_nodegraph::graph::{Edge, NodeId};
use std::collections::{BTreeMap, BTreeSet};

/// Porque a trilha não se transpõe.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ErroAoTranspor {
    /// A trilha não é do Motion.
    NaoEDoMotion,
    /// A trilha (ou o destino) não tem objecto dono (`StableId` `0`).
    SemDono,
    Inicial(ErroDoInicial),
    Passo(CodecError),
    /// Um passo, intenção ou edição de param de uma versão mais nova: os ids dele não se conhecem.
    Desconhecido(u64),
    /// Um passo (ou o documento de um passo) nomeia um nó ou uma moldura de OUTRO objecto.
    DoutroDono(u32),
    /// Um fio ou um param dirigido entre um nó do dono e um de fora.
    FioDeFora,
    /// Um grupo ou uma moldura que já existia ao armar (a vista não viaja).
    VistaDeAntes(u32),
    /// Nós do dono dentro de um grupo DESLIGADO que não viaja: o cozimento mudaria.
    GrupoDesligado,
    /// O documento de um passo é de antes de armar (`next_id` abaixo da base, ou a vista de antes mudou).
    DocumentoDeAntes,
    /// O documento de um passo não se lê.
    Documento(ErroDaMacro),
    /// O documento reduzido não volta ao bit pelo texto.
    DocumentoSemTexto,
    /// A reprodução da trilha transposta parou.
    Tocar(ErroDaMacro),
    /// A reprodução mudou um nó que o documento vivo já tinha.
    TocouOsNosDeAntes,
}

type E = ErroAoTranspor;

/// **A trilha a exportar**: a mesma, com o inicial reduzido aos nós de `track.target` (importados para
/// um documento vazio, sob o mesmo dono).
///
/// # Errors
/// [`ErroAoTranspor`].
pub fn so_do_dono(track: &Track) -> Result<Track, ErroAoTranspor> {
    let ini = le(track)?;
    let mut vazio = MotionDoc::new();
    vazio.base_z = ini.doc.base_z;
    transpoe(track, &ini, &vazio, track.target)
}

/// **Aplicar** a trilha a `novo_dono` em `vivo` — um objecto acabado de nascer (sem nós) ou um objecto
/// Motion que já tem os seus (os da macro juntam-se-lhes). Devolve a trilha a instalar (inicial = `vivo`
/// + os nós da macro sob `novo_dono`) e o documento do FIM dela, que substitui o vivo.
///
/// # Errors
/// [`ErroAoTranspor`].
pub fn aplica(
    track: &Track,
    vivo: &MotionDoc,
    novo_dono: u64,
) -> Result<(Track, MotionDoc), ErroAoTranspor> {
    if novo_dono == 0 {
        return Err(E::SemDono);
    }
    let ini = le(track)?;
    let t = transpoe(track, &ini, vivo, novo_dono)?;
    let doc = fim(&t).map_err(E::Tocar)?;
    if !intactos(vivo, &doc) {
        return Err(E::TocouOsNosDeAntes);
    }
    Ok((t, doc))
}

/// **O documento do fim da trilha** — o tocador ([`TocadorDoMotion`]) passo a passo, sem cozer os
/// quadros (o relógio não muda o grafo; a mesma escolha de [`super::etapas`]).
///
/// # Errors
/// [`ErroDaMacro`] do passo que parou.
pub fn fim(track: &Track) -> Result<MotionDoc, ErroDaMacro> {
    let t = track
        .tocavel(&LeitorDoMotion)
        .map_err(ErroDaMacro::Edicao)?;
    let mut tocador = TocadorDoMotion::da_trilha(&t, Modo::Explicativo)?;
    for e in &t.events {
        if !LeitorDoMotion.e_relogio(&e.payload) {
            tocador.aplica(&e.payload, None)?;
        }
    }
    Ok(tocador.palco().doc.clone())
}

fn le(track: &Track) -> Result<Inicial, E> {
    if track.family != FAMILIA {
        return Err(E::NaoEDoMotion);
    }
    if track.target == 0 {
        return Err(E::SemDono);
    }
    inicial::le(&track.initial).map_err(E::Inicial)
}

fn transpoe(track: &Track, ini: &Inicial, destino: &MotionDoc, novo: u64) -> Result<Track, E> {
    let dono = track.target;
    let i = &ini.doc;
    fios_so_do_dono(i, |n| i.owner_of(n) == Some(dono))?;
    if i.nodes_of(dono)
        .iter()
        .any(|n| em_grupo_desligado(i, *n, u32::MAX))
    {
        return Err(E::GrupoDesligado);
    }
    let mut j = destino.clone();
    let nos = j.import_owner(i, dono, novo);
    if i.arrumados_a_mao.contains(&dono) {
        j.arrumados_a_mao.insert(novo);
    }
    let mapa = Mapa {
        ini: i,
        destino,
        dono,
        novo,
        nos,
        base: ini.base,
        base_nova: j.graph.next_id(),
        grupos: (
            subgraph::next_id(&i.subgraphs),
            subgraph::next_id(&destino.subgraphs),
        ),
        fundos: (proxima_moldura(i), proxima_moldura(destino)),
    };
    let initial = inicial::escreve(&Inicial {
        doc: j,
        base: mapa.base_nova,
        lei: ini.lei,
        uv: ini.uv,
        tamanho: ini.tamanho,
    });
    let mut events = track.events.clone();
    for e in &mut events {
        e.payload = mapa.carga(&e.payload)?;
    }
    let mut edicoes = track.edicoes.clone();
    for ed in &mut edicoes {
        for (_, c) in ed.trocas.iter_mut().chain(ed.insercoes.iter_mut()) {
            *c = mapa.carga(c)?;
        }
    }
    Ok(Track {
        family: track.family.clone(),
        target: novo,
        initial,
        events,
        edicoes,
        nasce: track.nasce.clone(),
    })
}

/// A lei do `next_id` das molduras (`motion_bridge_backdrops`).
fn proxima_moldura(d: &MotionDoc) -> u32 {
    d.backdrops
        .iter()
        .map(|b| b.id)
        .max()
        .map_or(0, |m| m.saturating_add(1))
}

/// Nenhum fio nem param dirigido atravessa a fronteira do dono.
fn fios_so_do_dono(d: &MotionDoc, e_do_dono: impl Fn(NodeId) -> bool) -> Result<(), E> {
    let cruza = |a: NodeId, b: NodeId| e_do_dono(a) != e_do_dono(b);
    if d.graph.edges().iter().any(|e| cruza(e.from.0, e.to.0))
        || d.graph
            .all_param_sources()
            .iter()
            .any(|(n, s)| s.values().any(|(src, _)| cruza(*n, *src)))
    {
        return Err(E::FioDeFora);
    }
    Ok(())
}

/// `n` está (a qualquer profundidade) num grupo desligado de id abaixo de `limite`.
fn em_grupo_desligado(d: &MotionDoc, n: NodeId, limite: u32) -> bool {
    let mut s = d.members.get(&n).copied();
    let mut vistos = BTreeSet::new();
    while let Some(id) = s {
        if !vistos.insert(id) {
            return false;
        }
        if id < limite && d.bypassed_subgraphs.contains(&id) {
            return true;
        }
        s = d
            .subgraphs
            .iter()
            .find(|x| x.id == id)
            .and_then(|x| x.parent);
    }
    false
}

/// A vista que já existia ao armar: os grupos, os desligados e as molduras abaixo dos próximos ids.
fn vista_de_antes(d: &MotionDoc, grupos: u32, fundos: u32) -> [BTreeSet<u32>; 3] {
    [
        d.subgraphs
            .iter()
            .map(|s| s.id)
            .filter(|s| *s < grupos)
            .collect(),
        d.bypassed_subgraphs
            .iter()
            .copied()
            .filter(|s| *s < grupos)
            .collect(),
        d.backdrops
            .iter()
            .map(|b| b.id)
            .filter(|b| *b < fundos)
            .collect(),
    ]
}

/// Cada nó que `vivo` tinha está no `fim` como estava, e os fios dele também.
fn intactos(vivo: &MotionDoc, fim: &MotionDoc) -> bool {
    let (a, b) = (&vivo.graph, &fim.graph);
    a.nodes().iter().all(|n| {
        let id = n.id;
        b.node(id).map(|x| &x.type_name) == Some(&n.type_name)
            && vivo.owner_of(id) == fim.owner_of(id)
            && a.node_param_overrides(id) == b.node_param_overrides(id)
            && a.node_text_param_overrides(id) == b.node_text_param_overrides(id)
            && a.label(id) == b.label(id)
            && a.node_bypassed(id) == b.node_bypassed(id)
            && a.pos(id) == b.pos(id)
            && a.param_sources(id) == b.param_sources(id)
    }) && a.edges().iter().all(|e| b.edges().contains(e))
}

/// Os ids de uma trilha no documento de origem → no destino.
struct Mapa<'a> {
    ini: &'a MotionDoc,
    destino: &'a MotionDoc,
    dono: u64,
    novo: u64,
    /// Os nós do dono ao armar → os gémeos no destino.
    nos: BTreeMap<NodeId, NodeId>,
    base: u32,
    base_nova: u32,
    /// O próximo id de grupo (e de moldura) na origem e no destino.
    grupos: (u32, u32),
    fundos: (u32, u32),
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
    fn carga(&self, c: &[u8]) -> Result<Vec<u8>, E> {
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

/// A frase do porquê a trilha não viaja.
#[must_use]
pub fn erro_view(e: &ErroAoTranspor) -> ph2d_editor_core::macro_edits::ErroDaMacroView {
    use ph2d_editor_core::macro_edits::MACRO_LIB_NAO_TOCA;
    match e {
        E::Tocar(x) | E::Documento(x) => super::vistas::do_motion(x),
        outro => ph2d_editor_core::macro_etapas::erros::erro_com(
            MACRO_LIB_NAO_TOCA,
            format!("{outro:?}"),
        ),
    }
}

/// **A família MACROS leva a trilha por esta porta** (W18): exportar = só o grafo do dono, tocado até ao
/// fim; aplicar = os nós da macro no documento vivo, que passa ao fim dela num passo de undo.
impl ph2d_editor_core::macro_tutorial::TranspoeDaFamilia for crate::motion_state::MotionState {
    fn exporta(
        &self,
        track: &Track,
    ) -> Result<Track, ph2d_editor_core::macro_edits::ErroDaMacroView> {
        let t = so_do_dono(track).map_err(|e| erro_view(&e))?;
        fim(&t).map_err(|e| super::vistas::do_motion(&e))?;
        Ok(t)
    }

    fn aplica(
        &mut self,
        track: &Track,
        dono: u64,
        vida: u64,
    ) -> Result<Track, ph2d_editor_core::macro_edits::ErroDaMacroView> {
        let (t, doc) = aplica(track, &self.doc, dono).map_err(|e| erro_view(&e))?;
        if doc != self.doc {
            let pre = std::mem::replace(&mut self.doc, doc);
            self.objeto
                .atados
                .ata(&mut self.history, vida, pre, &self.doc);
            self.pump.mark_dirty();
        }
        Ok(t)
    }
}

#[cfg(all(test, feature = "panel-motion-graph"))]
#[path = "transpor_tests.rs"]
pub(crate) mod tests;
