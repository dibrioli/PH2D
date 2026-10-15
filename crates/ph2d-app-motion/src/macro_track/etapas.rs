//! **As etapas da trilha do Motion** (W5, `docs/Macros/02_plano_W5.md` §1/§3): uma etapa é um lote do
//! grafo (um arrasto de nós `BeginDrag..EndDrag` é um só), uma SESSÃO de param (os lotes do mesmo
//! `(nó, param)` em quadros seguidos — não há início/fim de arrasto gravável), uma edição direta
//! (`Escritas`/`Documento`: um gizmo, desfazer) ou uma porta recusada. O `Quadro` é o relógio entre elas;
//! uma espera sintetiza quadros na cadência gravada e o tempo do playhead de dentro anda com a trilha.

use super::passo::{Alvo, Escrita, Intencao, IntencaoDeParam, Passo};
use super::tocar::{ErroDaMacro, Modo, TocadorDoMotion};
use ph2d_macro::{Etapa, Event, LeitorDaFamilia, MacroTime, Track};
use std::collections::{BTreeMap, BTreeSet};

pub struct LeitorDoMotion;

fn passo(c: &[u8]) -> Option<Passo> {
    Passo::descodifica(c).ok()
}

/// A chave de uma sessão: os `(nó, param)` que o lote edita.
fn chave(p: &Passo) -> Option<BTreeSet<(u64, String)>> {
    let k: BTreeSet<_> = match p {
        Passo::Params { intencoes, .. } => intencoes
            .iter()
            .filter_map(|i| match i {
                IntencaoDeParam::SetParam { node, param, .. }
                | IntencaoDeParam::SetTextParam { node, param, .. }
                | IntencaoDeParam::ResetParam { node, param } => {
                    Some((node.varint(), param.clone()))
                }
                IntencaoDeParam::Desconhecida(_) => None,
            })
            .collect(),
        Passo::Escritas(v) => v
            .iter()
            .map(|e| match e {
                Escrita::Valor { node, param, .. } | Escrita::Texto { node, param, .. } => {
                    (node.varint(), param.clone())
                }
            })
            .collect(),
        _ => return None,
    };
    Some(k)
}

impl LeitorDaFamilia for LeitorDoMotion {
    fn etapas(&self, eventos: &[Event]) -> Vec<Etapa> {
        let mut out: Vec<Etapa> = Vec::new();
        // A sessão aberta: a chave, e se o último evento dela foi seguido só de UM quadro.
        let mut sessao: Option<(BTreeSet<(u64, String)>, usize)> = None;
        let mut arrasto: Option<usize> = None;
        let mut quadros_desde = 0;
        for (i, e) in eventos.iter().enumerate() {
            let Some(p) = passo(&e.payload) else {
                continue;
            };
            match &p {
                Passo::Quadro { .. } => {
                    quadros_desde += 1;
                    continue;
                }
                Passo::Toque(_) => continue,
                Passo::Grafo { intencoes, .. } => {
                    sessao = None;
                    let comeca = intencoes.iter().any(|x| matches!(x, Intencao::BeginDrag));
                    let acaba = intencoes.iter().any(|x| matches!(x, Intencao::EndDrag));
                    match (arrasto, comeca, acaba) {
                        (None, true, false) => arrasto = Some(i),
                        (Some(s), _, true) => {
                            out.push(Etapa { inicio: s, fim: i });
                            arrasto = None;
                        }
                        (Some(_), _, false) => {}
                        (None, _, _) => out.push(Etapa { inicio: i, fim: i }),
                    }
                }
                Passo::Params { .. } | Passo::Escritas(_) => {
                    let k = chave(&p).unwrap_or_default();
                    match &mut sessao {
                        Some((kk, s)) if *kk == k && quadros_desde <= 1 => {
                            if let Some(last) = out.last_mut().filter(|l| l.inicio == *s) {
                                last.fim = i;
                            }
                        }
                        _ => {
                            out.push(Etapa { inicio: i, fim: i });
                            sessao = Some((k, i));
                        }
                    }
                }
                Passo::Documento { .. } | Passo::Recusada(_) | Passo::Desconhecido(_) => {
                    sessao = None;
                    out.push(Etapa { inicio: i, fim: i });
                }
            }
            quadros_desde = 0;
        }
        if let Some(s) = arrasto {
            out.push(Etapa {
                inicio: s,
                fim: eventos.len() - 1,
            });
        }
        out
    }

    fn e_relogio(&self, carga: &[u8]) -> bool {
        matches!(passo(carga), Some(Passo::Quadro { .. }))
    }

    fn espera(&self, ev: &[Event], antes_de: usize, t0: MacroTime, us: u64, ja: i64) -> Vec<Event> {
        let (tempo, a_tocar, dt) = ev[..antes_de]
            .iter()
            .rev()
            .find_map(|e| match passo(&e.payload) {
                Some(Passo::Quadro {
                    tempo, a_tocar, dt, ..
                }) if dt > 0.0 => Some((tempo, a_tocar, dt)),
                _ => None,
            })
            .unwrap_or((0.0, true, 1.0 / 60.0));
        let tempo = tempo + segundos(ja);
        let quadro = |t: u64, tempo: f64, dt: f64| Event {
            t: MacroTime(t0.0 + t),
            payload: Passo::Quadro {
                tempo,
                a_tocar,
                dt,
                sujou: false,
            }
            .codifica(),
        };
        let (n, resto) = ph2d_macro::etapa::quadros_da_espera(us, dt * 1e6);
        let mut out: Vec<Event> = (0..n)
            .map(|j| {
                quadro(
                    ph2d_macro::etapa::no_quadro(j, dt * 1e6),
                    tempo + (j + 1) as f64 * dt,
                    dt,
                )
            })
            .collect();
        if resto > 0 {
            let r = segundos(resto as i64);
            out.push(quadro(us - resto, tempo + n as f64 * dt + r, r));
        }
        out
    }

    fn desloca(&self, carga: &[u8], us: i64) -> Option<Vec<u8>> {
        match passo(carga)? {
            Passo::Quadro {
                tempo,
                a_tocar,
                dt,
                sujou,
            } => Some(
                Passo::Quadro {
                    tempo: tempo + segundos(us),
                    a_tocar,
                    dt,
                    sujou,
                }
                .codifica(),
            ),
            _ => None,
        }
    }

    fn sai_da_gravacao(&self) -> Option<Vec<u8>> {
        None
    }
}

#[expect(clippy::cast_precision_loss, reason = "µs de uma macro cabem num f64")]
fn segundos(us: i64) -> f64 {
    us as f64 / 1e6
}

/// O que uma etapa fez, para o painel lhe dar nome (os tipos de nó e os params são os nomes canónicos;
/// o texto é do i18n, `node.<tipo>.name`).
#[derive(Clone, Debug, PartialEq)]
pub enum TipoDaEtapa {
    /// Nós acrescentados, pelo tipo.
    Acrescenta(Vec<String>),
    Liga {
        de: Option<String>,
        para: Option<String>,
    },
    Desliga {
        para: Option<String>,
    },
    Apaga(usize),
    Move,
    Renomeia(String),
    /// Uma sessão de param: o valor antes (`None` = o de omissão) e o último.
    Param {
        no: Option<String>,
        param: String,
        de: Option<f32>,
        para: f64,
    },
    /// Outro lote do grafo (agrupar, fundos, atalhos de ligação…).
    Grafo,
    /// Um gizmo, o conta-gotas, desfazer: o documento mudou por fora das intenções.
    Direta,
    Recusada(String),
}

#[derive(Clone, Debug, PartialEq)]
pub struct EtapaDoMotion {
    pub etapa: Etapa,
    pub tipo: TipoDaEtapa,
    /// Início e fim na trilha QUE TOCA.
    pub t0: MacroTime,
    pub t1: MacroTime,
    pub nome: Option<String>,
}

type Tipos = BTreeMap<u32, String>;

fn tipos(t: &TocadorDoMotion) -> Tipos {
    t.palco()
        .doc
        .graph
        .nodes()
        .iter()
        .map(|n| (n.id.0, n.type_name.clone()))
        .collect()
}

fn tipo_de(a: Alvo, base: u32, antes: &Tipos, depois: &Tipos) -> Option<String> {
    let id = a.no(base)?;
    depois.get(&id).or_else(|| antes.get(&id)).cloned()
}

fn descreve(
    eventos: &[Event],
    e: Etapa,
    base: u32,
    antes: &Tipos,
    depois: &Tipos,
    valor_antes: Option<f32>,
) -> TipoDaEtapa {
    let passos: Vec<Passo> = eventos[e.inicio..=e.fim]
        .iter()
        .filter_map(|x| passo(&x.payload))
        .collect();
    let intencoes: Vec<&Intencao> = passos
        .iter()
        .filter_map(|p| match p {
            Passo::Grafo { intencoes, .. } => Some(intencoes.iter()),
            _ => None,
        })
        .flatten()
        .collect();
    let tipo = |a: Alvo| tipo_de(a, base, antes, depois);
    match passos.first() {
        Some(Passo::Recusada(porta)) => return TipoDaEtapa::Recusada(porta.clone()),
        Some(Passo::Documento { .. } | Passo::Escritas(_)) => return TipoDaEtapa::Direta,
        Some(Passo::Params { .. }) => {
            let ultimo = passos.iter().rev().find_map(|p| match p {
                Passo::Params { intencoes, .. } => intencoes.iter().rev().find_map(|i| match i {
                    IntencaoDeParam::SetParam { node, param, value } => {
                        Some((*node, param.clone(), *value))
                    }
                    _ => None,
                }),
                _ => None,
            });
            return match ultimo {
                Some((node, param, para)) => TipoDaEtapa::Param {
                    no: tipo(node),
                    param,
                    de: valor_antes,
                    para,
                },
                None => TipoDaEtapa::Direta,
            };
        }
        _ => {}
    }
    let novos: Vec<String> = intencoes
        .iter()
        .filter_map(|i| match i {
            Intencao::AddNode { tipo, .. } | Intencao::SmartConnect { tipo, .. } => {
                Some(tipo.clone())
            }
            _ => None,
        })
        .collect();
    if !novos.is_empty() {
        return TipoDaEtapa::Acrescenta(novos);
    }
    for i in &intencoes {
        match i {
            Intencao::Connect { from, to, .. } => {
                return TipoDaEtapa::Liga {
                    de: tipo(*from),
                    para: tipo(*to),
                };
            }
            Intencao::Disconnect { to, .. } => return TipoDaEtapa::Desliga { para: tipo(*to) },
            Intencao::DeleteSelection(v) => return TipoDaEtapa::Apaga(v.len()),
            Intencao::Rename { nome, .. } => return TipoDaEtapa::Renomeia(nome.clone()),
            _ => {}
        }
    }
    if intencoes.iter().all(|i| {
        matches!(
            i,
            Intencao::MoveNodes { .. } | Intencao::BeginDrag | Intencao::EndDrag
        )
    }) {
        return TipoDaEtapa::Move;
    }
    TipoDaEtapa::Grafo
}

/// O valor (override) de `(nó, param)` no palco, `None` = o de omissão.
fn valor(t: &TocadorDoMotion, a: Alvo, param: &str) -> Option<f32> {
    let id = ph2d_nodegraph::graph::NodeId(a.no(t.base())?);
    t.palco()
        .doc
        .graph
        .node_param_overrides(id)?
        .get(param)
        .copied()
}

/// **As etapas de uma trilha do Motion**, com as edições dela aplicadas. Os nomes resolvem-se num palco
/// que recebe só os passos do documento (sem cozer: o grafo basta).
///
/// # Errors
/// [`ErroDaMacro`].
pub fn etapas(track: &Track) -> Result<Vec<EtapaDoMotion>, ErroDaMacro> {
    let tocavel = track
        .tocavel(&LeitorDoMotion)
        .map_err(ErroDaMacro::Edicao)?;
    let gravadas = LeitorDoMotion.etapas(&track.events);
    let tocadas = LeitorDoMotion.etapas(&tocavel.events);
    let mut t = TocadorDoMotion::da_trilha(&tocavel, Modo::Explicativo)?;
    let base = t.base();
    let mut out = Vec::with_capacity(tocadas.len());
    let mut i = 0;
    for (g, e) in gravadas.iter().zip(&tocadas) {
        let antes = tipos(&t);
        let valor_antes = match passo(&tocavel.events[e.inicio].payload) {
            Some(Passo::Params { intencoes, .. }) => intencoes.iter().find_map(|i| match i {
                IntencaoDeParam::SetParam { node, param, .. } => valor(&t, *node, param),
                _ => None,
            }),
            _ => None,
        };
        while i <= e.fim {
            let c = &tocavel.events[i].payload;
            if !LeitorDoMotion.e_relogio(c) && !matches!(passo(c), Some(Passo::Recusada(_))) {
                t.aplica(c, None)?;
            }
            i += 1;
        }
        let depois = tipos(&t);
        out.push(EtapaDoMotion {
            etapa: *g,
            tipo: descreve(&tocavel.events, *e, base, &antes, &depois, valor_antes),
            t0: tocavel.events[e.inicio].t,
            t1: tocavel.events[e.fim].t,
            nome: track
                .edicoes
                .iter()
                .find(|x| x.ancora == g.inicio)
                .and_then(|x| x.nome.clone()),
        });
    }
    Ok(out)
}

#[derive(Clone, Debug, PartialEq)]
pub enum ErroDeEdicaoDoMotion {
    SemEtapa(usize),
    NaoEParam(usize),
}

/// **O valor final da sessão de param `k`** — cada amostra do arrasto anda em proporção ao progresso,
/// então o gesto mantém a forma e acaba em `novo`.
///
/// # Errors
/// [`ErroDeEdicaoDoMotion`].
pub fn edita_valor(track: &mut Track, k: usize, novo: f64) -> Result<(), ErroDeEdicaoDoMotion> {
    let e = *LeitorDoMotion
        .etapas(&track.events)
        .get(k)
        .ok_or(ErroDeEdicaoDoMotion::SemEtapa(k))?;
    let lotes: Vec<(usize, Passo)> = (e.inicio..=e.fim)
        .filter_map(|i| match passo(&track.events[i].payload)? {
            p @ Passo::Params { .. } => Some((i, p)),
            _ => None,
        })
        .collect();
    let ultimo = lotes.iter().rev().find_map(|(_, p)| match p {
        Passo::Params { intencoes, .. } => intencoes.iter().rev().find_map(|i| match i {
            IntencaoDeParam::SetParam { value, .. } => Some(*value),
            _ => None,
        }),
        _ => None,
    });
    let Some(ultimo) = ultimo else {
        return Err(ErroDeEdicaoDoMotion::NaoEParam(k));
    };
    let n = lotes.len() as f64;
    let trocas: Vec<(usize, Vec<u8>)> = lotes
        .into_iter()
        .enumerate()
        .map(|(j, (i, p))| {
            let Passo::Params {
                destino,
                mut intencoes,
            } = p
            else {
                unreachable!("filtrado acima")
            };
            let anda = (novo - ultimo) * (j + 1) as f64 / n;
            for x in &mut intencoes {
                if let IntencaoDeParam::SetParam { value, .. } = x {
                    *value += anda;
                }
            }
            (i, Passo::Params { destino, intencoes }.codifica())
        })
        .collect();
    let ed = track.edicao_mut(e.inicio);
    ed.trocas.retain(|(i, _)| !(e.inicio..=e.fim).contains(i));
    ed.trocas.extend(trocas);
    Ok(())
}

/// **A espera antes da etapa `k`**, em µs (+ abre, − encurta).
///
/// # Errors
/// [`ErroDeEdicaoDoMotion::SemEtapa`].
pub fn espera(track: &mut Track, k: usize, us: i64) -> Result<(), ErroDeEdicaoDoMotion> {
    let e = *LeitorDoMotion
        .etapas(&track.events)
        .get(k)
        .ok_or(ErroDeEdicaoDoMotion::SemEtapa(k))?;
    track.edicao_mut(e.inicio).espera_us = us;
    Ok(())
}
