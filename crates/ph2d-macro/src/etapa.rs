//! **As ETAPAS de uma trilha** (W5, `docs/Macros/02_plano_W5.md`): o gesto do artista é a unidade que se
//! nomeia, re-cronometra e edita; o relógio entre gestos não é etapa. A gravação (`Track::events`) nunca
//! muda — as edições vivem ao lado ([`EdicaoDeEtapa`], ancoradas no 1.º evento da etapa NA GRAVAÇÃO) e a
//! trilha que toca é uma função pura das duas ([`efetiva`]).
//!
//! Só a família lê a sua carga: ela diz onde estão as etapas, o que é relógio, como se sintetiza uma
//! espera ([`LeitorDaFamilia`]). Uma espera é tempo a CORRER (a água escorre, o vento sopra), nunca
//! câmara lenta — a câmara lenta é a faixa «Macro Time» da Timeline.

use crate::doc::{Event, MacroTime, Track};
use std::collections::BTreeMap;

/// Uma etapa: os eventos `inicio..=fim` da gravação.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Etapa {
    pub inicio: usize,
    pub fim: usize,
}

/// O que a família diz da sua carga — o núcleo nunca a lê.
pub trait LeitorDaFamilia {
    /// As etapas da gravação, por ordem e sem sobreposição.
    fn etapas(&self, eventos: &[Event]) -> Vec<Etapa>;
    /// O evento é só relógio: uma espera encurtada pode tirá-lo.
    fn e_relogio(&self, carga: &[u8]) -> bool;
    /// **`us` de relógio a correr**, a começar em `t0`, na cadência do relógio gravado antes do evento
    /// `antes_de`; `ja_andou_us` é quanto a trilha já se deslocou ali (o Motion leva o tempo do playhead
    /// dentro do quadro).
    fn espera(
        &self,
        eventos: &[Event],
        antes_de: usize,
        t0: MacroTime,
        us: u64,
        ja_andou_us: i64,
    ) -> Vec<Event>;
    /// A carga de um evento que andou `us` no tempo. `None` = a mesma.
    fn desloca(&self, carga: &[u8], us: i64) -> Option<Vec<u8>>;
    /// O passo que avisa o tocador de que dali em diante a trilha já não é a gravação no TEMPO (o Painter:
    /// a água larga a fita do worker). `None` = a família não precisa.
    fn sai_da_gravacao(&self) -> Option<Vec<u8>>;
}

/// As edições de UMA etapa.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EdicaoDeEtapa {
    /// O 1.º evento da etapa na gravação (`Etapa::inicio`).
    pub ancora: usize,
    /// O nome que o artista escreveu (o derivado não se grava).
    pub nome: Option<String>,
    /// Tempo acrescentado (+) ou tirado (−) ANTES da etapa, µs; tudo o que vem depois anda com ela.
    pub espera_us: i64,
    /// Cargas trocadas: `(evento da gravação, carga nova)`.
    pub trocas: Vec<(usize, Vec<u8>)>,
    /// Cargas acrescentadas ANTES de um evento da gravação, no instante dele: `(evento, carga)`.
    pub insercoes: Vec<(usize, Vec<u8>)>,
    /// A trilha sai da gravação nesta etapa mesmo sem mudar o tempo (a conferência de uma edição de
    /// valor divergiu: a água do Painter passa ao relógio fixo).
    pub sai_da_gravacao: bool,
}

/// Uma edição que não cabe na gravação — recusada, nunca adivinhada.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ErroDeEdicao {
    /// A âncora não é o início de nenhuma etapa.
    AncoraSemEtapa(usize),
    /// Duas edições na mesma etapa.
    AncoraRepetida(usize),
    /// Uma troca ou inserção aponta para fora da gravação.
    ForaDaGravacao(usize),
    /// Encurtar mais do que o intervalo até à etapa anterior.
    EsperaMaiorQueOIntervalo(usize),
}

/// **A trilha que toca**: a gravação com as edições aplicadas. Sem edições = a gravação, ao bit.
///
/// # Errors
/// [`ErroDeEdicao`].
pub fn efetiva(track: &Track, leitor: &dyn LeitorDaFamilia) -> Result<Vec<Event>, ErroDeEdicao> {
    let ev = &track.events;
    if track.edicoes.is_empty() {
        return Ok(ev.clone());
    }
    let etapas = leitor.etapas(ev);
    let mut por_ancora: BTreeMap<usize, &EdicaoDeEtapa> = BTreeMap::new();
    let mut trocas: BTreeMap<usize, &[u8]> = BTreeMap::new();
    let mut insercoes: BTreeMap<usize, Vec<&[u8]>> = BTreeMap::new();
    // Eventos da janela que uma espera NEGATIVA encurta → a âncora dela.
    let mut janela: BTreeMap<usize, usize> = BTreeMap::new();
    for ed in &track.edicoes {
        let k = etapas
            .iter()
            .position(|e| e.inicio == ed.ancora)
            .ok_or(ErroDeEdicao::AncoraSemEtapa(ed.ancora))?;
        if por_ancora.insert(ed.ancora, ed).is_some() {
            return Err(ErroDeEdicao::AncoraRepetida(ed.ancora));
        }
        for (i, c) in &ed.trocas {
            if *i >= ev.len() {
                return Err(ErroDeEdicao::ForaDaGravacao(*i));
            }
            trocas.insert(*i, c);
        }
        for (i, c) in &ed.insercoes {
            if *i >= ev.len() {
                return Err(ErroDeEdicao::ForaDaGravacao(*i));
            }
            insercoes.entry(*i).or_default().push(c);
        }
        if ed.espera_us < 0 {
            let desde = if k == 0 { 0 } else { etapas[k - 1].fim + 1 };
            let antes = if k == 0 { 0 } else { ev[etapas[k - 1].fim].t.0 };
            let corte = ev[ed.ancora].t.0.checked_sub(ed.espera_us.unsigned_abs());
            let Some(corte) = corte.filter(|&c| c >= antes) else {
                return Err(ErroDeEdicao::EsperaMaiorQueOIntervalo(ed.ancora));
            };
            for (j, e) in ev.iter().enumerate().take(ed.ancora).skip(desde) {
                if e.t.0 >= corte {
                    janela.insert(j, ed.ancora);
                }
            }
        }
    }

    let mut out = Vec::with_capacity(ev.len());
    let mut andou: i64 = 0;
    let mut saiu = false;
    let marca = leitor.sai_da_gravacao();
    let sai = |out: &mut Vec<Event>, t: u64, saiu: &mut bool| {
        if !*saiu {
            *saiu = true;
            if let Some(m) = &marca {
                out.push(Event {
                    t: MacroTime(t),
                    payload: m.clone(),
                });
            }
        }
    };
    let em = |t: MacroTime, d: i64| t.0.saturating_add_signed(d);
    for (i, e) in ev.iter().enumerate() {
        if let Some(&ancora) = janela.get(&i) {
            let ed = por_ancora[&ancora];
            let t_ancora = em(ev[ancora].t, andou + ed.espera_us);
            if janela.range(..i).next_back().is_none_or(|(_, &a)| a != ancora) {
                sai(&mut out, t_ancora.min(em(e.t, andou)), &mut saiu);
            }
            if leitor.e_relogio(&e.payload) && !insercoes.contains_key(&i) && !trocas.contains_key(&i) {
                continue;
            }
            empurra(&mut out, leitor, e, i, t_ancora.min(em(e.t, andou)), andou, &trocas, &insercoes);
            continue;
        }
        if let Some(ed) = por_ancora.get(&i) {
            let aqui = em(e.t, andou);
            if ed.espera_us != 0 || ed.sai_da_gravacao {
                sai(&mut out, aqui, &mut saiu);
            }
            if ed.espera_us > 0 {
                out.extend(leitor.espera(
                    ev,
                    i,
                    MacroTime(aqui),
                    ed.espera_us.unsigned_abs(),
                    andou,
                ));
            }
            andou += ed.espera_us;
        }
        empurra(&mut out, leitor, e, i, em(e.t, andou), andou, &trocas, &insercoes);
    }
    Ok(out)
}

#[expect(clippy::too_many_arguments, reason = "o passo de saída e as duas tabelas da edição")]
fn empurra(
    out: &mut Vec<Event>,
    leitor: &dyn LeitorDaFamilia,
    e: &Event,
    i: usize,
    t: u64,
    andou: i64,
    trocas: &BTreeMap<usize, &[u8]>,
    insercoes: &BTreeMap<usize, Vec<&[u8]>>,
) {
    let anda = |c: &[u8]| {
        if andou == 0 {
            c.to_vec()
        } else {
            leitor.desloca(c, andou).unwrap_or_else(|| c.to_vec())
        }
    };
    for c in insercoes.get(&i).into_iter().flatten() {
        out.push(Event {
            t: MacroTime(t),
            payload: anda(c),
        });
    }
    let carga = trocas.get(&i).copied().unwrap_or(&e.payload);
    out.push(Event {
        t: MacroTime(t),
        payload: anda(carga),
    });
}

impl Track {
    /// A edição da etapa que começa em `ancora`, criada vazia se ainda não existe.
    pub fn edicao_mut(&mut self, ancora: usize) -> &mut EdicaoDeEtapa {
        let i = match self.edicoes.iter().position(|e| e.ancora == ancora) {
            Some(i) => i,
            None => {
                self.edicoes.push(EdicaoDeEtapa {
                    ancora,
                    ..Default::default()
                });
                self.edicoes.len() - 1
            }
        };
        &mut self.edicoes[i]
    }

    /// **A trilha que toca**, sem edições (as dela aplicadas por [`efetiva`]).
    ///
    /// # Errors
    /// [`ErroDeEdicao`].
    pub fn tocavel(&self, leitor: &dyn LeitorDaFamilia) -> Result<Track, ErroDeEdicao> {
        Ok(Track {
            family: self.family.clone(),
            target: self.target,
            initial: self.initial.clone(),
            events: efetiva(self, leitor)?,
            edicoes: Vec::new(),
        })
    }
}
