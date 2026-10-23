//! **As ETAPAS de uma trilha** (W5, `docs/Macros/02_plano_W5.md`): o gesto do artista é a unidade que se
//! nomeia, re-cronometra e edita; o relógio entre gestos não é etapa. A gravação (`Track::events`) nunca
//! muda — as edições vivem ao lado ([`EdicaoDeEtapa`], ancoradas no 1.º evento da etapa NA GRAVAÇÃO) e a
//! trilha que toca é uma função pura das duas ([`efetiva`]).
//!
//! Só a família lê a sua carga: ela diz onde estão as etapas, o que é relógio, como se sintetiza uma
//! espera ([`LeitorDaFamilia`]). Uma espera é tempo a CORRER (a água escorre, o vento sopra), nunca
//! câmara lenta — a câmara lenta é a faixa «Macro Time» da Timeline. Apagar uma etapa FECHA o tempo dela
//! (decisão do dono, 15/10): o que vem depois recua a duração da etapa.

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
    /// **O relógio do quadro cujo gesto foi apagado** (o do mesmo instante do fim da etapa): o Motion
    /// diz que o quadro já não sujou a simulação. `None` = o mesmo.
    fn sem_o_gesto(&self, _carga: &[u8]) -> Option<Vec<u8>> {
        None
    }
    /// **A carga de um evento da etapa deslocada que toca na faixa `n ≥ 1`** — o Painter abre um traço por
    /// faixa (duas pinceladas sobrepostas pintam ao mesmo tempo). `None` = a família não tem faixas.
    fn na_faixa(&self, _carga: &[u8], _n: u32) -> Option<Vec<u8>> {
        None
    }
    /// O evento age sobre «o último» que se fez (o Desfazer do Painter): deslocar uma etapa não pode mudar
    /// qual é a última que acaba antes dele.
    fn e_barreira(&self, _carga: &[u8]) -> bool {
        false
    }
    /// **O passo que, no instante GRAVADO de uma etapa deslocada, guarda o pincel de ali para a faixa
    /// `n`** — a etapa leva o seu pincel para onde for. `None` = a família não precisa.
    fn marca_da_faixa(&self, _n: u32) -> Option<Vec<u8>> {
        None
    }
}

/// As edições de UMA etapa.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EdicaoDeEtapa {
    /// O 1.º evento da etapa na gravação (`Etapa::inicio`).
    pub ancora: usize,
    /// O nome que o artista escreveu (o derivado não se grava).
    pub nome: Option<String>,
    /// O texto que o autor escreveu para o balão do tutorial «fazer» nesta etapa (conteúdo; não muda o
    /// que toca).
    pub dica: Option<String>,
    /// Tempo acrescentado (+) ou tirado (−) ANTES da etapa, µs; tudo o que vem depois anda com ela.
    pub espera_us: i64,
    /// **A etapa anda SOZINHA** Δ µs (o losango arrastado na Timeline, W16): as outras ficam, e ela pode
    /// sobrepor-se a outra ou passar-lhe à frente. O relógio fica onde está (um relógio por quadro).
    pub desloca_us: i64,
    /// Cargas trocadas: `(evento da gravação, carga nova)`.
    pub trocas: Vec<(usize, Vec<u8>)>,
    /// Cargas acrescentadas ANTES de um evento da gravação, no instante dele: `(evento, carga)`.
    pub insercoes: Vec<(usize, Vec<u8>)>,
    /// A trilha sai da gravação nesta etapa mesmo sem mudar o tempo (a conferência de uma edição de
    /// valor divergiu: a água do Painter passa ao relógio fixo).
    pub sai_da_gravacao: bool,
    /// A etapa saiu: os eventos dela (o relógio de dentro também) não tocam, o que vem depois recua a
    /// duração dela, e a trilha sai da gravação ali. As outras edições desta etapa deixam de valer.
    pub apagada: bool,
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
    /// A etapa deslocada começaria antes do instante 0.
    ComecaAntesDoZero(usize),
    /// A etapa deslocada mudaria a etapa que um evento de «o último» desfaz
    /// ([`LeitorDaFamilia::e_barreira`]).
    MudaOQueODesfazerDesfaz(usize),
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
    // As etapas apagadas: início → fim.
    let mut apagadas: BTreeMap<usize, usize> = BTreeMap::new();
    let mut moveis: Vec<Movel> = Vec::new();
    for ed in &track.edicoes {
        let k = etapas
            .iter()
            .position(|e| e.inicio == ed.ancora)
            .ok_or(ErroDeEdicao::AncoraSemEtapa(ed.ancora))?;
        if por_ancora.insert(ed.ancora, ed).is_some() {
            return Err(ErroDeEdicao::AncoraRepetida(ed.ancora));
        }
        if ed.apagada {
            apagadas.insert(ed.ancora, etapas[k].fim);
            continue;
        }
        if ed.desloca_us != 0 {
            moveis.push(Movel {
                inicio: ed.ancora,
                fim: etapas[k].fim,
                desloca_us: ed.desloca_us,
                faixa: u32::try_from(k + 1).unwrap_or(u32::MAX),
            });
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

    let mut out = Saida::default();
    let mut andou: i64 = 0;
    let mut saiu = false;
    let marca = leitor.sai_da_gravacao();
    let sai = |out: &mut Saida, t: u64, saiu: &mut bool| {
        if !*saiu {
            *saiu = true;
            if let Some(m) = &marca {
                out.push(None, t, m.clone());
            }
        }
    };
    let em = |t: MacroTime, d: i64| t.0.saturating_add_signed(d);
    // A etapa apagada que se está a saltar: (fim, a duração dela em µs, o instante onde fecha).
    let mut fecha: Option<(usize, i64, u64)> = None;
    // O instante (gravado) do quadro que perdeu o gesto apagado, até ao 1.º evento que não é relógio.
    let mut sem_gesto: Option<MacroTime> = None;
    for (i, e) in ev.iter().enumerate() {
        if sem_gesto.is_some_and(|t| t != e.t || !leitor.e_relogio(&e.payload)) {
            sem_gesto = None;
        }
        if fecha.is_none()
            && let Some(&fim) = apagadas.get(&i)
        {
            let aqui = em(e.t, andou);
            sai(&mut out, aqui, &mut saiu);
            let dur = i64::try_from(ev[fim].t.0 - e.t.0).unwrap_or(i64::MAX);
            fecha = Some((fim, dur, aqui));
        }
        if let Some((fim, dur, aqui)) = fecha {
            // As inserções de OUTRAS etapas antes de um evento apagado tocam onde ele fechou.
            for c in insercoes.get(&i).into_iter().flatten() {
                let carga = if andou == 0 {
                    c.to_vec()
                } else {
                    leitor.desloca(c, andou).unwrap_or_else(|| c.to_vec())
                };
                out.push(Some(i), aqui, carga);
            }
            if i == fim {
                andou -= dur;
                fecha = None;
                sem_gesto = Some(e.t);
            }
            continue;
        }
        if sem_gesto.is_some()
            && !trocas.contains_key(&i)
            && let Some(c) = leitor.sem_o_gesto(&e.payload)
        {
            let ev_limpo = Event { t: e.t, payload: c };
            empurra(
                &mut out,
                leitor,
                &ev_limpo,
                i,
                em(e.t, andou),
                andou,
                &trocas,
                &insercoes,
            );
            continue;
        }
        if let Some(&ancora) = janela.get(&i) {
            let ed = por_ancora[&ancora];
            let t_ancora = em(ev[ancora].t, andou + ed.espera_us);
            if janela
                .range(..i)
                .next_back()
                .is_none_or(|(_, &a)| a != ancora)
            {
                sai(&mut out, t_ancora.min(em(e.t, andou)), &mut saiu);
            }
            if leitor.e_relogio(&e.payload)
                && !insercoes.contains_key(&i)
                && !trocas.contains_key(&i)
            {
                continue;
            }
            empurra(
                &mut out,
                leitor,
                e,
                i,
                t_ancora.min(em(e.t, andou)),
                andou,
                &trocas,
                &insercoes,
            );
            continue;
        }
        if let Some(ed) = por_ancora.get(&i) {
            let aqui = em(e.t, andou);
            if ed.espera_us != 0 || ed.sai_da_gravacao {
                sai(&mut out, aqui, &mut saiu);
            }
            if ed.espera_us > 0 {
                for q in leitor.espera(ev, i, MacroTime(aqui), ed.espera_us.unsigned_abs(), andou) {
                    out.push(None, q.t.0, q.payload);
                }
            }
            andou += ed.espera_us;
        }
        empurra(
            &mut out,
            leitor,
            e,
            i,
            em(e.t, andou),
            andou,
            &trocas,
            &insercoes,
        );
    }
    if moveis.is_empty() {
        return Ok(out.ev);
    }
    desloca_as_etapas(out, ev, &etapas, &moveis, leitor, saiu)
}

/// Os eventos da trilha que toca e o evento da gravação de que cada um veio (`None` = sintetizado).
#[derive(Default)]
struct Saida {
    ev: Vec<Event>,
    orig: Vec<Option<usize>>,
}

impl Saida {
    fn push(&mut self, orig: Option<usize>, t: u64, payload: Vec<u8>) {
        self.ev.push(Event {
            t: MacroTime(t),
            payload,
        });
        self.orig.push(orig);
    }
}

/// Uma etapa que anda sozinha: os eventos `inicio..=fim` da gravação, Δ, a faixa.
struct Movel {
    inicio: usize,
    fim: usize,
    desloca_us: i64,
    faixa: u32,
}

/// **As etapas que andam sozinhas** (W16): os eventos NÃO-relógio delas somam Δ e tocam na faixa delas; o
/// relógio fica (um relógio por quadro); a fusão é estável (empate = a ordem de antes). Uma etapa levada
/// para lá do último evento estende o relógio ([`LeitorDaFamilia::espera`]) e o fim da gravação anda com
/// ela. A marca de saída da gravação cai no menor instante que mudou.
fn desloca_as_etapas(
    out: Saida,
    ev: &[Event],
    etapas: &[Etapa],
    moveis: &[Movel],
    leitor: &dyn LeitorDaFamilia,
    saiu: bool,
) -> Result<Vec<Event>, ErroDeEdicao> {
    let Saida { ev: mut sai, orig } = out;
    let de = |i: usize| moveis.iter().find(|m| (m.inicio..=m.fim).contains(&i));
    let em_etapa = |i: usize| etapas.iter().position(|g| (g.inicio..=g.fim).contains(&i));
    let mut vaos = vec![Vao::default(); etapas.len()];
    let mut corte = u64::MAX;
    for (e, o) in sai.iter().zip(&orig) {
        let Some(i) = *o else { continue };
        let Some(k) = em_etapa(i) else { continue };
        let d = de(i).map_or(0, |m| m.desloca_us);
        let Some(novo) = e.t.0.checked_add_signed(d) else {
            return Err(ErroDeEdicao::ComecaAntesDoZero(etapas[k].inicio));
        };
        if d != 0 {
            corte = corte.min(e.t.0.min(novo));
        }
        vaos[k].junta(e.t.0, novo);
    }
    for (e, o) in sai.iter().zip(&orig) {
        if !o.is_some_and(|i| em_etapa(i).is_some()) && leitor.e_barreira(&e.payload) {
            barreira(e.t.0, &vaos, etapas, moveis)?;
        }
    }
    let mut marcas = Vec::new();
    for m in moveis {
        let primeiro = sai
            .iter()
            .zip(&orig)
            .find(|(_, o)| o.is_some_and(|i| (m.inicio..=m.fim).contains(&i)));
        if let (Some((e, _)), Some(c)) = (primeiro, leitor.marca_da_faixa(m.faixa)) {
            marcas.push(Event {
                t: e.t,
                payload: c,
            });
        }
    }
    let fim_antes = sai.iter().map(|e| e.t.0).max().unwrap_or(0);
    for (e, o) in sai.iter_mut().zip(&orig) {
        let Some(m) = o.and_then(de) else { continue };
        if leitor.e_relogio(&e.payload) {
            continue;
        }
        e.t = MacroTime(e.t.0.saturating_add_signed(m.desloca_us));
        if let Some(c) = leitor.na_faixa(&e.payload, m.faixa) {
            e.payload = c;
        }
    }
    // A CAUDA (do fim da última etapa no tempo ao fim da gravação: a água a assentar até ao Parar) anda
    // com a última etapa: o fim da trilha, e o clipe com ela, segue-a para lá e para cá.
    let ultima = |depois: bool| vaos.iter().filter_map(|v| v.de(depois)).map(|v| v.1).max();
    let d_fim = match (ultima(false), ultima(true)) {
        (Some(a), Some(d)) => i64::try_from(d).unwrap_or(i64::MAX) - i64::try_from(a).unwrap_or(0),
        _ => 0,
    };
    let fim_novo = fim_antes.saturating_add_signed(d_fim);
    if fim_novo != fim_antes {
        let mut fica = Vec::with_capacity(sai.len());
        for (mut e, o) in sai.into_iter().zip(&orig) {
            let relogio = leitor.e_relogio(&e.payload);
            let da_etapa = !relogio && o.is_some_and(|i| em_etapa(i).is_some());
            if da_etapa || (e.t.0 < fim_antes && e.t.0 <= fim_novo) {
                fica.push(e);
            } else if !relogio {
                // O que fecha a gravação (o Parar) vai para o fim novo.
                e.t = MacroTime(fim_novo);
                fica.push(e);
            } else if e.t.0 <= fim_novo {
                fica.push(e);
            }
            // O relógio da cauda que o fim novo deixou para trás sai.
        }
        sai = fica;
        if fim_novo > fim_antes {
            sai.extend(leitor.espera(ev, ev.len(), MacroTime(fim_antes), fim_novo - fim_antes, 0));
        }
    }
    // A marca da faixa fica no instante GRAVADO da etapa, depois do que lá estava (a cor escolhida antes
    // do toque, no mesmo instante): é o pincel dali que a etapa leva.
    // A cauda que encurtou leva-a consigo: o pincel de ali é o do fim novo.
    sai.extend(marcas.into_iter().map(|mut m| {
        m.t = m.t.min(MacroTime(fim_novo));
        m
    }));
    sai.sort_by_key(|e| e.t);
    if !saiu
        && corte != u64::MAX
        && let Some(m) = leitor.sai_da_gravacao()
    {
        let k = sai.partition_point(|e| e.t.0 < corte);
        sai.insert(
            k,
            Event {
                t: MacroTime(corte),
                payload: m,
            },
        );
    }
    Ok(sai)
}

/// O vão de uma etapa na trilha que toca: o primeiro e o último instante, antes e depois de deslocar.
#[derive(Clone, Copy, Default)]
struct Vao {
    antes: Option<(u64, u64)>,
    depois: Option<(u64, u64)>,
}

impl Vao {
    fn junta(&mut self, antes: u64, depois: u64) {
        let alarga = |v: &mut Option<(u64, u64)>, t: u64| {
            let (a, b) = v.get_or_insert((t, t));
            *a = (*a).min(t);
            *b = (*b).max(t);
        };
        alarga(&mut self.antes, antes);
        alarga(&mut self.depois, depois);
    }

    fn de(&self, depois: bool) -> Option<(u64, u64)> {
        if depois { self.depois } else { self.antes }
    }
}

/// **Um evento de «o último» (o Desfazer) no instante `tb`**: a última etapa a acabar antes dele tem de
/// ser a mesma depois de deslocar, sem empate no instante dele, e nenhuma deslocada passa a tocar ao
/// mesmo tempo que ela (ele desfaria as duas).
fn barreira(tb: u64, vaos: &[Vao], etapas: &[Etapa], moveis: &[Movel]) -> Result<(), ErroDeEdicao> {
    let ultima = |depois: bool| {
        vaos.iter()
            .enumerate()
            .filter_map(|(k, v)| v.de(depois).map(|(_, f)| (f, k)))
            .filter(|&(f, _)| f <= tb)
            .max()
            .map(|(_, k)| k)
    };
    let (antes, depois) = (ultima(false), ultima(true));
    let movida = |k: usize| moveis.iter().any(|m| m.inicio == etapas[k].inicio);
    let empata = vaos
        .iter()
        .enumerate()
        .any(|(k, v)| movida(k) && v.depois.is_some_and(|(_, f)| f == tb) && depois != Some(k));
    let cruza = |a: (u64, u64), b: (u64, u64)| a.0 <= b.1 && b.0 <= a.1;
    let sobrepoe = antes.is_some_and(|l| {
        vaos.iter().enumerate().any(|(k, v)| {
            k != l
                && movida(k)
                && match (v.antes, v.depois, vaos[l].antes, vaos[l].depois) {
                    (Some(a), Some(d), Some(la), Some(ld)) => cruza(d, ld) && !cruza(a, la),
                    _ => false,
                }
        })
    });
    if antes == depois && !empata && !sobrepoe {
        return Ok(());
    }
    let culpada = moveis
        .iter()
        .find(|m| {
            [antes, depois]
                .iter()
                .flatten()
                .any(|&k| etapas[k].inicio == m.inicio)
        })
        .or_else(|| {
            moveis.iter().find(|m| {
                etapas.iter().position(|g| g.inicio == m.inicio).is_some_and(|k| {
                    vaos[k].depois.is_some_and(|(_, f)| f == tb)
                        || antes.is_some_and(|l| {
                            matches!((vaos[k].depois, vaos[l].depois), (Some(d), Some(ld)) if cruza(d, ld))
                        })
                })
            })
        })
        .unwrap_or(&moveis[0]);
    Err(ErroDeEdicao::MudaOQueODesfazerDesfaz(culpada.inicio))
}

/// **Quantos quadros de `quadro_us` cabem numa espera de `us`**, e o resto (µs) que não enche um. A
/// cadência é a EXACTA do `dt` gravado (60 Hz = 16 666,67 µs): arredondar o quadro a µs inteiros
/// acumula — 60 quadros davam 1 000 020 µs e o último caía fora do instante em que a espera acaba.
#[must_use]
pub fn quadros_da_espera(us: u64, quadro_us: f64) -> (u64, u64) {
    let q = quadro_us.max(1.0);
    let n = ((us as f64 + 0.5) / q).floor() as u64;
    (n, us.saturating_sub(no_quadro(n, q)))
}

/// O instante (µs desde o início da espera) do quadro `j`.
#[must_use]
pub fn no_quadro(j: u64, quadro_us: f64) -> u64 {
    (j as f64 * quadro_us).round() as u64
}

#[expect(
    clippy::too_many_arguments,
    reason = "o passo de saída e as duas tabelas da edição"
)]
fn empurra(
    out: &mut Saida,
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
        out.push(Some(i), t, anda(c));
    }
    let carga = trocas.get(&i).copied().unwrap_or(&e.payload);
    out.push(Some(i), t, anda(carga));
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

    /// **Apaga a etapa `k`** (a contar da gravação): o tempo dela fecha-se. `false` = não há etapa `k`.
    pub fn apaga_etapa(&mut self, leitor: &dyn LeitorDaFamilia, k: usize) -> bool {
        let Some(e) = leitor.etapas(&self.events).get(k).copied() else {
            return false;
        };
        self.edicao_mut(e.inicio).apagada = true;
        true
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
