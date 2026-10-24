//! **As etapas da família «mundo»** pela porta comum ([`EtapasDaFamilia`], W19 §5): uma etapa por
//! evento da trilha — o passo do undo (componentes, objectos que nascem ou se apagam) e «Simular».
//! Cada etapa parte do que as anteriores fizeram: a espera e o instante servem (mantida a ordem);
//! apagar ou trocar o conteúdo recusa-se pelo nome.

use crate::mundo::formato::{Evento, FAMILIA, Inicial, Mudanca};
use ph2d_editor_core::macro_edits::{
    CampoDaEtapa, ErroDaMacroView, EtapaView, FamiliaDaMacro, PropsDaEtapa, TipoDaEtapaView,
};
use ph2d_editor_core::macro_etapas::erros::{self as e, erro};
use ph2d_editor_core::macro_etapas::{EtapasDaFamilia, desloca_para, espera_s, segundos, us};
use ph2d_macro::{Etapa, Event, LeitorDaFamilia, MacroTime, Track};
use std::collections::{BTreeMap, BTreeSet};

/// O componente do nome de um objecto, pelo nome canónico.
const NOME: &str = "ph2d::ecs::Name";

/// O que o núcleo pergunta à trilha do mundo: um evento = uma etapa, sem relógio dentro (a espera é só
/// tempo que anda; a carga não guarda instantes da macro).
pub struct LeitorDoMundo;

impl LeitorDaFamilia for LeitorDoMundo {
    fn etapas(&self, eventos: &[Event]) -> Vec<Etapa> {
        (0..eventos.len())
            .map(|i| Etapa { inicio: i, fim: i })
            .collect()
    }

    fn e_relogio(&self, _carga: &[u8]) -> bool {
        false
    }

    fn espera(&self, _: &[Event], _: usize, _: MacroTime, _: u64, _: i64) -> Vec<Event> {
        Vec::new()
    }

    fn desloca(&self, _carga: &[u8], _us: i64) -> Option<Vec<u8>> {
        None
    }

    fn sai_da_gravacao(&self) -> Option<Vec<u8>> {
        None
    }
}

/// O nome curto de um componente (`Collider`, não `ph2d::physics::Collider`).
#[must_use]
pub fn curto(canonico: &str) -> String {
    canonico.rsplit("::").next().unwrap_or(canonico).to_owned()
}

/// A duração de um «Simular» em segundos: os tiques do passo fixo da física.
#[must_use]
pub fn segundos_da_corrida(de: u32, ate: u32) -> f64 {
    f64::from(ate.saturating_sub(de)) / ph2d_core::time::DEFAULT_HZ
}

/// O que se sabe de cada objecto local enquanto se lêem os passos: o nome e os componentes.
#[derive(Default)]
struct Saber {
    nomes: BTreeMap<u32, String>,
    comps: BTreeMap<u32, BTreeSet<String>>,
}

impl Saber {
    fn poe(&mut self, id: u32, nome: &str, bytes: Option<&[u8]>) {
        match bytes {
            Some(b) => {
                if nome == NOME
                    && let Ok(n) = postcard::from_bytes::<ph2d_ecs::Name>(b)
                {
                    self.nomes.insert(id, n.0);
                }
                self.comps.entry(id).or_default().insert(nome.to_owned());
            }
            None => {
                if nome == NOME {
                    self.nomes.remove(&id);
                }
                if let Some(c) = self.comps.get_mut(&id) {
                    c.remove(nome);
                }
            }
        }
    }

    fn nome(&self, id: u32) -> Option<String> {
        self.nomes.get(&id).cloned()
    }

    /// O tipo da etapa de um passo, e o saber passa ao depois dele.
    fn passo(&mut self, mudancas: &[Mudanca]) -> TipoDaEtapaView {
        let tocados: BTreeSet<u32> = mudancas
            .iter()
            .map(|m| match m {
                Mudanca::Nasce(l) => l.id,
                Mudanca::Morre(id) | Mudanca::Muda { id, .. } => *id,
            })
            .collect();
        let outros = tocados.len().saturating_sub(1);
        // Quem nomeia a etapa: o que nasce (o de topo), senão o apagado, senão o primeiro mudado.
        let nascidos: BTreeSet<u32> = mudancas
            .iter()
            .filter_map(|m| match m {
                Mudanca::Nasce(l) => Some(l.id),
                _ => None,
            })
            .collect();
        let topo = mudancas.iter().find_map(|m| match m {
            Mudanca::Nasce(l) if l.pai.is_none_or(|p| !nascidos.contains(&p)) => Some(l),
            _ => None,
        });
        let apagado = mudancas.iter().find_map(|m| match m {
            Mudanca::Morre(id) => Some(*id),
            _ => None,
        });
        let antes_de_apagar = apagado.and_then(|id| self.nome(id));
        let mut postos = Vec::new();
        let mut mudados = Vec::new();
        let mut tirados = Vec::new();
        let primeiro_mudado = mudancas.iter().find_map(|m| match m {
            Mudanca::Muda { id, .. } => Some(*id),
            _ => None,
        });
        for m in mudancas {
            match m {
                Mudanca::Nasce(l) => {
                    self.comps.remove(&l.id);
                    self.nomes.remove(&l.id);
                    for c in &l.componentes {
                        self.poe(l.id, &c.nome, Some(&c.bytes));
                    }
                }
                Mudanca::Morre(id) => {
                    self.comps.remove(id);
                }
                Mudanca::Muda {
                    id, componentes, ..
                } => {
                    for (n, b) in componentes {
                        if Some(*id) == primeiro_mudado {
                            let tinha = self.comps.get(id).is_some_and(|c| c.contains(n));
                            match (b, tinha) {
                                (Some(_), false) => postos.push(curto(n)),
                                (Some(_), true) => mudados.push(curto(n)),
                                (None, _) => tirados.push(curto(n)),
                            }
                        }
                        self.poe(*id, n, b.as_deref());
                    }
                }
            }
        }
        if let Some(l) = topo {
            return TipoDaEtapaView::Nasce {
                objeto: self.nome(l.id),
                outros,
            };
        }
        if apagado.is_some() {
            return TipoDaEtapaView::Apagado {
                objeto: antes_de_apagar,
                outros,
            };
        }
        TipoDaEtapaView::Componentes {
            objeto: primeiro_mudado.and_then(|id| self.nome(id)),
            postos,
            mudados,
            tirados,
            outros,
        }
    }
}

/// Uma etapa do mundo lida: o tipo, o instante na trilha QUE TOCA e a duração.
struct Lida {
    tipo: TipoDaEtapaView,
    t0: MacroTime,
    dur_s: f64,
}

fn erro_de_leitura() -> ErroDaMacroView {
    erro(e::ESTRAGADA)
}

/// **As etapas da trilha `track`**, por ordem da gravação.
fn le(track: &Track) -> Result<Vec<Lida>, ErroDaMacroView> {
    let inicial = Inicial::le(&track.initial).map_err(|_| erro(e::INICIAL))?;
    let tocavel = track
        .tocavel(&LeitorDoMundo)
        .map_err(|x| e::de_edicao(&x))?;
    let mut saber = Saber::default();
    for l in &inicial.linhas {
        for c in &l.componentes {
            saber.poe(l.id, &c.nome, Some(&c.bytes));
        }
    }
    let mut out = Vec::with_capacity(track.events.len());
    for (k, ev) in track.events.iter().enumerate() {
        let evento = Evento::le(&ev.payload).map_err(|_| erro_de_leitura())?;
        // As edições do mundo nunca mudam a ordem nem o número de eventos: o k-ésimo que toca é este.
        let t0 = tocavel.events.get(k).map_or(ev.t, |x| x.t);
        let (tipo, dur_s) = match &evento {
            Evento::Passo { mudancas, .. } => (saber.passo(mudancas), 0.0),
            Evento::Simula(s) => {
                let d = segundos_da_corrida(s.de, s.ate);
                (TipoDaEtapaView::Simula { segundos: d }, d)
            }
        };
        out.push(Lida { tipo, t0, dur_s });
    }
    Ok(out)
}

/// As etapas do mundo — desta crate, registadas ao lado das do Painter.
pub struct EtapasDoMundo;

impl EtapasDaFamilia for EtapasDoMundo {
    fn familia(&self) -> FamiliaDaMacro {
        FamiliaDaMacro::Mundo
    }

    fn chave(&self) -> &'static str {
        FAMILIA
    }

    fn leitor(&self) -> &'static dyn LeitorDaFamilia {
        &LeitorDoMundo
    }

    fn vistas(&self, trilha: usize, track: &Track) -> Result<Vec<EtapaView>, ErroDaMacroView> {
        Ok(le(track)?
            .into_iter()
            .enumerate()
            .map(|(k, l)| {
                let t0_s = segundos(l.t0);
                EtapaView {
                    trilha,
                    k,
                    familia: FamiliaDaMacro::Mundo,
                    tipo: l.tipo,
                    nome: None,
                    dica: None,
                    t0_s,
                    t1_s: t0_s + l.dur_s,
                    espera_s: espera_s(track, k),
                    props: PropsDaEtapa::Nenhuma,
                }
            })
            .collect())
    }

    fn edita(
        &self,
        track: &mut Track,
        k: usize,
        campo: &CampoDaEtapa,
    ) -> Result<(), ErroDaMacroView> {
        if k >= track.events.len() {
            return Err(e::sem_etapa());
        }
        match *campo {
            CampoDaEtapa::Espera(s) => {
                track.edicao_mut(k).espera_us = us(s);
                Ok(())
            }
            CampoDaEtapa::Instante(s) => {
                let es = le(track)?;
                let fim = segundos(es[k].t0) + es[k].dur_s;
                let novo_t0 = s - es[k].dur_s;
                let antes = k.checked_sub(1).map(|j| segundos(es[j].t0));
                let depois = es.get(k + 1).map(|x| segundos(x.t0));
                if antes.is_some_and(|a| novo_t0 < a) || depois.is_some_and(|d| novo_t0 > d) {
                    return Err(erro(e::MUNDO_ORDEM));
                }
                let d = desloca_para(track, k, fim, s);
                track.edicao_mut(k).desloca_us = d;
                Ok(())
            }
            // O conteúdo de uma etapa do mundo é o ponto de partida das seguintes.
            CampoDaEtapa::Cor(_) | CampoDaEtapa::Tamanho(_) | CampoDaEtapa::Valor(_) => {
                Err(erro(e::MUNDO_FIXO))
            }
            CampoDaEtapa::Nome(_) | CampoDaEtapa::Dica(_) => Err(e::sem_etapa()),
        }
    }

    fn pode_apagar(&self, _track: &Track, _k: usize) -> Result<(), ErroDaMacroView> {
        Err(erro(e::MUNDO_FIXO))
    }
}
