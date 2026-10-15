//! **O tutorial «fazer» do Motion** (W6, `docs/Macros/03_plano_W6.md` §2): o critério de cada etapa é o
//! que ela MUDOU no grafo gravado (Δ entre o palco antes e depois dela, pela mesma reprodução que dá os
//! nomes, [`super::etapas::percorre`]), e cumpre-se quando o grafo do ALUNO o tem.
//!
//! Os nós casam por TIPO e ORDEM DE CRIAÇÃO entre os nós do dono ([`Vaga`]): o i-ésimo nó gravado do
//! tipo T é o i-ésimo do aluno — os ids crus nunca se comparam (o documento do aluno tem outros nós e
//! outro `next_id`). Um param cumpre-se pelo número que o aluno VÊ (`format_number`, o do cartão).

use super::etapas::percorre;
use super::tocar::ErroDaMacro;
use crate::motion_state::MotionState;
use ph2d_editor_core::interaction::format_number;
use ph2d_editor_core::macro_edits::ErroDaMacroView;
use ph2d_editor_core::macro_tutorial::{EtapaDoTutorial, PedidoDoTutorial, TutorialDaFamilia};
use ph2d_macro::Track;
use ph2d_nodegraph::cook::OpResolver;
use ph2d_nodegraph::graph::{Edge, NodeId, Pos};
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;

/// Um nó pelo tipo e pela ordem de criação entre os nós do dono desse tipo.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Vaga {
    pub tipo: String,
    pub ordem: usize,
}

/// Um fio entre duas vagas (a porta de saída de `de`, a de entrada de `para`).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Fio {
    pub de: Vaga,
    pub porta_de: u16,
    pub para: Vaga,
    pub porta_para: u16,
    /// O fio de realimentação (`pre`): fecha um laço sem ciclo.
    pub atraso: bool,
}

/// Uma coisa que a etapa mudou.
#[derive(Clone, Debug, PartialEq)]
pub enum Item {
    /// Pelo menos `quantos` nós do tipo (a etapa acrescentou-os); `lugar` = onde o gravado nasceu.
    Ha {
        tipo: String,
        quantos: usize,
        lugar: Option<Pos>,
    },
    /// No máximo `quantos` (a etapa apagou).
    NaoHaMais {
        tipo: String,
        quantos: usize,
    },
    Liga(Fio),
    Desliga(Fio),
    /// O param ficou em `valor`; `antes` = o que tinha antes da etapa.
    Valor {
        no: Vaga,
        param: String,
        valor: f32,
        antes: f32,
    },
}

/// O critério de uma etapa: `k` a contar da gravação.
#[derive(Clone, Debug, PartialEq)]
pub struct Criterio {
    pub k: usize,
    pub itens: Vec<Item>,
}

/// O grafo de UM dono pelas vagas.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Retrato {
    pub nos: BTreeMap<Vaga, NodeId>,
    pub fios: BTreeSet<Fio>,
    contagem: BTreeMap<String, usize>,
    /// `(vaga, param)` → (o efectivo, o de omissão), de todo param do manifesto.
    valores: BTreeMap<(Vaga, String), (f32, f32)>,
    lugares: BTreeMap<Vaga, Pos>,
}

/// O retrato do grafo de `dono` em `m`.
#[must_use]
pub fn retrato(m: &MotionState, dono: u64) -> Retrato {
    let mut ids = m.doc.nodes_of(dono);
    ids.sort_unstable();
    let mut r = Retrato::default();
    let mut vaga_de: BTreeMap<NodeId, Vaga> = BTreeMap::new();
    for id in ids {
        let Some(n) = m.doc.graph.node(id) else {
            continue;
        };
        let c = r.contagem.entry(n.type_name.clone()).or_default();
        let v = Vaga {
            tipo: n.type_name.clone(),
            ordem: *c,
        };
        *c += 1;
        let sobre = m.doc.graph.node_param_overrides(id);
        if let Some(op) = m.registry.resolve(n.type_id()) {
            for p in op.manifest().params {
                let ef = sobre
                    .and_then(|s| s.get(p.name))
                    .copied()
                    .unwrap_or(p.default);
                r.valores
                    .insert((v.clone(), p.name.to_owned()), (ef, p.default));
            }
        }
        if let Some(p) = m.doc.graph.pos(id) {
            r.lugares.insert(v.clone(), p);
        }
        vaga_de.insert(id, v.clone());
        r.nos.insert(v, id);
    }
    for e in m.doc.graph.edges() {
        if let (Some(a), Some(b)) = (vaga_de.get(&e.from.0), vaga_de.get(&e.to.0)) {
            r.fios.insert(Fio {
                de: a.clone(),
                porta_de: e.from.1,
                para: b.clone(),
                porta_para: e.to.1,
                atraso: e.delayed,
            });
        }
    }
    r
}

fn mesmo_numero(a: f32, b: f32) -> bool {
    format_number(f64::from(a)) == format_number(f64::from(b))
}

/// **O que a etapa mudou**, de `a` (antes) para `d` (depois).
#[must_use]
pub fn delta(a: &Retrato, d: &Retrato) -> Vec<Item> {
    let mut out = Vec::new();
    for (tipo, &n) in &d.contagem {
        if n > a.contagem.get(tipo).copied().unwrap_or(0) {
            let ultimo = Vaga {
                tipo: tipo.clone(),
                ordem: n - 1,
            };
            out.push(Item::Ha {
                tipo: tipo.clone(),
                quantos: n,
                lugar: d.lugares.get(&ultimo).copied(),
            });
        }
    }
    for (tipo, &na) in &a.contagem {
        let n = d.contagem.get(tipo).copied().unwrap_or(0);
        if n < na {
            out.push(Item::NaoHaMais {
                tipo: tipo.clone(),
                quantos: n,
            });
        }
    }
    out.extend(d.fios.difference(&a.fios).cloned().map(Item::Liga));
    out.extend(a.fios.difference(&d.fios).cloned().map(Item::Desliga));
    for ((no, param), &(valor, padrao)) in &d.valores {
        let antes = a
            .valores
            .get(&(no.clone(), param.clone()))
            .map_or(padrao, |x| x.0);
        if !mesmo_numero(valor, antes) {
            out.push(Item::Valor {
                no: no.clone(),
                param: param.clone(),
                valor,
                antes,
            });
        }
    }
    out
}

/// **Os critérios de uma trilha**: só as etapas que mudam o grafo do dono (mover, renomear e as
/// molduras não esperam pelo aluno).
///
/// # Errors
/// [`ErroDaMacro`] — a trilha não se re-toca.
pub fn criterios(track: &Track) -> Result<Vec<Criterio>, ErroDaMacro> {
    let dono = track.target;
    let mut out = Vec::new();
    percorre(
        track,
        |t, _, _| retrato(t.palco(), dono),
        |k, _, _, _, t, antes| {
            let itens = delta(&antes, &retrato(t.palco(), dono));
            if !itens.is_empty() {
                out.push(Criterio { k, itens });
            }
        },
    )?;
    Ok(out)
}

/// O item cumpre-se no retrato do aluno `s`. Um VALOR cumpre-se também pelo valor de uma etapa
/// POSTERIOR sobre o mesmo param (`seguintes`), salvo se esse for o que havia antes desta — aí o aluno
/// não fez nada e o estado não o distingue de ter feito.
fn cumpre(item: &Item, s: &Retrato, seguintes: &[Criterio]) -> bool {
    let quantos = |t: &str| s.contagem.get(t).copied().unwrap_or(0);
    match item {
        Item::Ha {
            tipo, quantos: n, ..
        } => quantos(tipo) >= *n,
        Item::NaoHaMais { tipo, quantos: n } => quantos(tipo) <= *n,
        Item::Liga(f) => s.fios.contains(f),
        Item::Desliga(f) => !s.fios.contains(f),
        Item::Valor {
            no,
            param,
            valor,
            antes,
        } => {
            let Some(&(v, _)) = s.valores.get(&(no.clone(), param.clone())) else {
                return false;
            };
            mesmo_numero(v, *valor)
                || seguintes.iter().flat_map(|c| &c.itens).any(|x| match x {
                    Item::Valor {
                        no: n2,
                        param: p2,
                        valor: v2,
                        ..
                    } => {
                        n2 == no
                            && p2 == param
                            && mesmo_numero(v, *v2)
                            && !mesmo_numero(*v2, *antes)
                    }
                    _ => false,
                })
        }
    }
}

fn pedido(item: &Item) -> PedidoDoTutorial {
    match item {
        Item::Ha { tipo, .. } => PedidoDoTutorial::Acrescenta { tipo: tipo.clone() },
        Item::NaoHaMais { tipo, .. } => PedidoDoTutorial::Apaga { tipo: tipo.clone() },
        Item::Liga(f) => PedidoDoTutorial::Liga {
            de: f.de.tipo.clone(),
            para: f.para.tipo.clone(),
        },
        Item::Desliga(f) => PedidoDoTutorial::Desliga {
            para: f.para.tipo.clone(),
        },
        Item::Valor {
            no, param, valor, ..
        } => PedidoDoTutorial::Valor {
            no: no.tipo.clone(),
            param: param.clone(),
            valor: f64::from(*valor),
        },
    }
}

/// Um sítio onde o aluno faz o item, num nó dele.
#[derive(Clone, Copy)]
enum Sitio {
    Saida(u16),
    Entrada(u16),
    Cartao,
}

/// Onde o aluno faz o item: as portas de um fio, o cartão de um nó (ids crus do aluno).
fn sitios(item: &Item, s: &Retrato) -> Vec<(u32, Sitio)> {
    let no = |v: &Vaga| s.nos.get(v).map(|n| n.0);
    match item {
        Item::Liga(f) => [
            no(&f.de).map(|n| (n, Sitio::Saida(f.porta_de))),
            no(&f.para).map(|n| (n, Sitio::Entrada(f.porta_para))),
        ]
        .into_iter()
        .flatten()
        .collect(),
        Item::Desliga(f) => no(&f.para)
            .map(|n| (n, Sitio::Entrada(f.porta_para)))
            .into_iter()
            .collect(),
        Item::Valor { no: v, .. } => no(v).map(|n| (n, Sitio::Cartao)).into_iter().collect(),
        Item::NaoHaMais { tipo, .. } => s
            .nos
            .range(
                Vaga {
                    tipo: tipo.clone(),
                    ordem: 0,
                }..,
            )
            .take_while(|(v, _)| &v.tipo == tipo)
            .last()
            .map(|(_, n)| (n.0, Sitio::Cartao))
            .into_iter()
            .collect(),
        Item::Ha { .. } => Vec::new(),
    }
}

/// Os ids de interação dos [`sitios`] que o painel do grafo regista.
#[cfg(feature = "panel-motion-graph")]
fn alvos(item: &Item, s: &Retrato) -> Vec<ph2d_editor_core::ids::NodeId> {
    use ph2d_panel_motion_graph::alvo;
    sitios(item, s)
        .into_iter()
        .map(|(n, sitio)| match sitio {
            Sitio::Saida(p) => alvo::saida(n, usize::from(p)),
            Sitio::Entrada(p) => alvo::entrada(n, usize::from(p)),
            Sitio::Cartao => alvo::no(n),
        })
        .collect()
}

#[cfg(not(feature = "panel-motion-graph"))]
fn alvos(_: &Item, _: &Retrato) -> Vec<ph2d_editor_core::ids::NodeId> {
    Vec::new()
}

/// **As etapas no estado do aluno**: para cada critério, o 1.º item por cumprir e onde se faz.
#[must_use]
pub fn avalia(m: &MotionState, dono: u64, crits: &[Criterio]) -> Vec<EtapaDoTutorial> {
    avalia_e_onde(m, dono, crits).0
}

/// [`avalia`] e a etapa CORRENTE (a 1.ª por cumprir): `(k, os nós do aluno onde ela se faz)`.
fn avalia_e_onde(
    m: &MotionState,
    dono: u64,
    crits: &[Criterio],
) -> (Vec<EtapaDoTutorial>, Option<(usize, Vec<u32>)>) {
    let s = retrato(m, dono);
    let mut corrente = None;
    let etapas = crits
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let falta = c.itens.iter().find(|x| !cumpre(x, &s, &crits[i + 1..]));
            if let Some(x) = falta
                && corrente.is_none()
            {
                corrente = Some((c.k, sitios(x, &s).into_iter().map(|(n, _)| n).collect()));
            }
            EtapaDoTutorial {
                k: c.k,
                falta: falta.map(pedido),
                alvos: falta.map(|x| alvos(x, &s)).unwrap_or_default(),
            }
        })
        .collect();
    (etapas, corrente)
}

#[cfg(feature = "panel-motion-graph")]
thread_local! {
    /// O último «traz à vista» pedido ao painel: `(dono, k, nós)` — só se volta a pedir quando muda,
    /// para o aluno poder deslocar o editor à vontade.
    static VISTO: RefCell<Option<(u64, usize, Vec<u32>)>> = const { RefCell::new(None) };
}

/// Pede ao painel do grafo que traga à vista os nós da etapa corrente, uma vez por etapa.
#[cfg(feature = "panel-motion-graph")]
fn traz_a_vista(dono: u64, corrente: Option<(usize, Vec<u32>)>) {
    let Some((k, nos)) = corrente.filter(|(_, n)| !n.is_empty()) else {
        return;
    };
    let chave = (dono, k, nos);
    if VISTO.with(|v| v.borrow().as_ref() != Some(&chave)) {
        ph2d_panel_motion_graph::request_frame_nodes(chave.2.clone());
        VISTO.with(|v| *v.borrow_mut() = Some(chave));
    }
}

#[cfg(not(feature = "panel-motion-graph"))]
fn traz_a_vista(_: u64, _: Option<(usize, Vec<u32>)>) {}

/// **«Mostra-me»**: o grafo do aluno passa a ter o que a etapa mudou — um passo de undo.
///
/// # Errors
/// A chave da frase: um nó da etapa que o aluno não tem, um fio que o grafo recusa.
pub fn mostra(m: &mut MotionState, dono: u64, c: &Criterio) -> Result<(), ErroDaMacroView> {
    use ph2d_editor_core::macro_etapas::erros::{ALVO, erro};
    let pre = m.doc.clone();
    // Tirar antes de pôr: um fio novo pode ir para a entrada que um fio desfeito ocupava.
    let ordem = |x: &Item| match x {
        Item::NaoHaMais { .. } | Item::Desliga(_) => 0,
        Item::Ha { .. } => 1,
        Item::Liga(_) | Item::Valor { .. } => 2,
    };
    let mut itens: Vec<&Item> = c.itens.iter().collect();
    itens.sort_by_key(|x| ordem(x));
    let r = (|| {
        for item in itens {
            let s = retrato(m, dono);
            match item {
                Item::Ha {
                    tipo,
                    quantos,
                    lugar,
                } => {
                    for _ in s.contagem.get(tipo).copied().unwrap_or(0)..*quantos {
                        let id = m.doc.graph.add_node(tipo.clone());
                        m.doc.set_owner(id, dono);
                        if let Some(p) = lugar {
                            m.doc.graph.set_pos(id, *p);
                        }
                    }
                }
                Item::NaoHaMais { tipo, quantos } => {
                    let mortos: Vec<NodeId> = s
                        .nos
                        .iter()
                        .filter(|(v, _)| &v.tipo == tipo && v.ordem >= *quantos)
                        .map(|(_, n)| *n)
                        .collect();
                    for n in &mortos {
                        m.doc.graph.remove_node(*n);
                    }
                    m.doc.forget_nodes(&mortos);
                }
                Item::Liga(f) => {
                    let (Some(a), Some(b)) = (s.nos.get(&f.de), s.nos.get(&f.para)) else {
                        return Err(erro(ALVO));
                    };
                    m.doc.graph.disconnect(*b, f.porta_para);
                    m.doc
                        .graph
                        .connect(Edge {
                            from: (*a, f.porta_de),
                            to: (*b, f.porta_para),
                            delayed: f.atraso,
                        })
                        .map_err(|_| erro(ALVO))?;
                }
                Item::Desliga(f) => {
                    // Só ESSE fio: a entrada pode já ter outro.
                    if let (Some(a), Some(b)) = (s.nos.get(&f.de), s.nos.get(&f.para))
                        && m.doc.graph.input_edge(*b, usize::from(f.porta_para))
                            == Some((*a, usize::from(f.porta_de), f.atraso))
                    {
                        m.doc.graph.disconnect(*b, f.porta_para);
                    }
                }
                Item::Valor {
                    no, param, valor, ..
                } => {
                    let Some(n) = s.nos.get(no) else {
                        return Err(erro(ALVO));
                    };
                    m.doc.graph.set_param(*n, param.clone(), *valor);
                }
            }
        }
        Ok(())
    })();
    if r.is_err() {
        m.doc = pre;
        return r;
    }
    if m.doc != pre {
        m.history.push_undo(pre);
        m.pump.mark_dirty();
    }
    Ok(())
}

/// **«Recomeçar»**: o grafo de `dono` volta ao do estado inicial da trilha (os nós que eram do
/// objecto gravado) — um passo de undo.
///
/// # Errors
/// O estado inicial não se lê.
pub fn recomeca(m: &mut MotionState, dono: u64, track: &Track) -> Result<(), ErroDaMacroView> {
    let ini = super::inicial::le(&track.initial).map_err(|_| {
        ph2d_editor_core::macro_etapas::erros::erro(ph2d_editor_core::macro_etapas::erros::INICIAL)
    })?;
    let pre = m.doc.clone();
    m.doc.remove_owner(dono);
    m.doc.import_owner(&ini.doc, track.target, dono);
    // O grafo recomeçado nasce arrumado, no MESMO passo de undo.
    crate::motion_arrumar::arrumar_ao_abrir(m);
    if m.doc != pre {
        m.history.push_undo(pre);
        m.pump.mark_dirty();
    }
    Ok(())
}

thread_local! {
    /// Os critérios de cada objecto do tutorial, pela versão da `Macro` de que saíram.
    static CRITERIOS: RefCell<BTreeMap<u64, (u64, Rc<Vec<Criterio>>)>> =
        const { RefCell::new(BTreeMap::new()) };
}

fn em_cache(dono: u64, track: &Track, versao: u64) -> Result<Rc<Vec<Criterio>>, ErroDaMacroView> {
    if let Some(c) = CRITERIOS.with(|c| {
        c.borrow()
            .get(&dono)
            .filter(|(v, _)| *v == versao)
            .map(|(_, c)| Rc::clone(c))
    }) {
        return Ok(c);
    }
    let c = Rc::new(criterios(track).map_err(|e| super::vistas::do_motion(&e))?);
    CRITERIOS.with(|x| x.borrow_mut().insert(dono, (versao, Rc::clone(&c))));
    Ok(c)
}

impl TutorialDaFamilia for MotionState {
    fn avalia(
        &mut self,
        dono: u64,
        track: &Track,
        versao: u64,
    ) -> Result<Vec<EtapaDoTutorial>, ErroDaMacroView> {
        if track.family != super::FAMILIA {
            return Ok(Vec::new());
        }
        let (etapas, corrente) = avalia_e_onde(self, dono, &em_cache(dono, track, versao)?);
        traz_a_vista(dono, corrente);
        Ok(etapas)
    }

    fn mostra(
        &mut self,
        dono: u64,
        track: &Track,
        versao: u64,
        k: usize,
    ) -> Result<(), ErroDaMacroView> {
        let crits = em_cache(dono, track, versao)?;
        let c = crits
            .iter()
            .find(|c| c.k == k)
            .ok_or_else(ph2d_editor_core::macro_etapas::erros::sem_etapa)?;
        mostra(self, dono, c)
    }

    fn recomeca(&mut self, dono: u64, track: &Track) -> Result<(), ErroDaMacroView> {
        recomeca(self, dono, track)
    }
}

#[cfg(test)]
#[path = "tutorial_tests.rs"]
mod tests;
