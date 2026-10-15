//! **As etapas da trilha do Painter** (W5, `docs/Macros/02_plano_W5.md` §1/§3): uma etapa é uma
//! pincelada (`Ponteiro Down..Up`; a cor e o tamanho dela são o pincel no toque, venham de onde vierem)
//! ou uma porta recusada. O tique, as respostas do worker e os drenos da
//! pré-visualização são o relógio entre elas.
//!
//! Editar uma pincelada toca só nela: a cor e o tamanho novos entram antes do toque e o que ela tinha
//! volta depois do levantar. Re-cronometrar sai da gravação (a água larga a fita, `Passo::SaiDaGravacao`);
//! uma edição de valor só sai se a conferência divergir ([`assenta`]).

use super::inicial;
use super::passo::Passo;
use super::tocar::{ErroDaMacro, TocadorDoPainter};
use crate::PaintMedia;
use crate::tool::PainterTool;
use ph2d_editor_core::tool::PointerPhase;
use ph2d_macro::{Etapa, Event, LeitorDaFamilia, MacroTime, Track};
use std::sync::atomic::{AtomicBool, Ordering};

pub struct LeitorDoPainter;

fn passo(c: &[u8]) -> Option<Passo> {
    Passo::descodifica(c).ok()
}

impl LeitorDaFamilia for LeitorDoPainter {
    fn etapas(&self, eventos: &[Event]) -> Vec<Etapa> {
        let mut out: Vec<Etapa> = Vec::new();
        let mut toque = None;
        for (i, e) in eventos.iter().enumerate() {
            match passo(&e.payload) {
                Some(Passo::Ponteiro {
                    fase: PointerPhase::Down,
                    ..
                }) => toque = Some(i),
                Some(Passo::Ponteiro {
                    fase: PointerPhase::Up,
                    ..
                }) => {
                    if let Some(d) = toque.take() {
                        out.push(Etapa { inicio: d, fim: i });
                    }
                }
                Some(Passo::Recusada(_)) if toque.is_none() => {
                    out.push(Etapa { inicio: i, fim: i });
                }
                _ => {}
            }
        }
        out
    }

    fn e_relogio(&self, carga: &[u8]) -> bool {
        matches!(
            passo(carga),
            Some(
                Passo::Tique(_)
                    | Passo::Agua(_)
                    | Passo::Hover(_)
                    | Passo::Preview
                    | Passo::PreviewBbox
                    | Passo::PreviewSujo
                    | Passo::PreviewAtual
            )
        )
    }

    /// Quadros de editor — a pré-visualização que a ponte drena e o tique — na cadência do último tique
    /// gravado; o resto que não enche um quadro é um tique mais curto (a água anda o tempo inteiro).
    fn espera(&self, ev: &[Event], antes_de: usize, t0: MacroTime, us: u64, _: i64) -> Vec<Event> {
        let dt_ms = ev[..antes_de]
            .iter()
            .rev()
            .find_map(|e| match passo(&e.payload) {
                Some(Passo::Tique(dt)) if dt > 0.0 => Some(dt),
                _ => None,
            })
            .unwrap_or(1000.0 / 60.0);
        let q = f64::from(dt_ms) * 1000.0;
        let mut out = Vec::new();
        let mut quadro = |t: u64, dt: f32| {
            for p in [Passo::Preview, Passo::Tique(dt)] {
                out.push(Event {
                    t: MacroTime(t),
                    payload: p.codifica(),
                });
            }
        };
        let (n, resto) = ph2d_macro::etapa::quadros_da_espera(us, q);
        for j in 0..n {
            quadro(t0.0 + ph2d_macro::etapa::no_quadro(j, q), dt_ms);
        }
        if resto > 0 {
            quadro(t0.0 + us - resto, resto as f32 / 1000.0);
        }
        out
    }

    fn desloca(&self, _: &[u8], _: i64) -> Option<Vec<u8>> {
        None
    }

    fn sai_da_gravacao(&self) -> Option<Vec<u8>> {
        Some(Passo::SaiDaGravacao.codifica())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum TipoDaEtapa {
    /// A n-ésima pincelada (a contar de 1).
    Pincelada(usize),
    /// Uma porta que a macro não reproduz: o tocador pára aqui (o porquê).
    Recusada(String),
}

/// Uma etapa como o painel a mostra: os eventos da gravação, os instantes na trilha que toca e as
/// propriedades da pincelada no toque.
#[derive(Clone, Debug, PartialEq)]
pub struct EtapaDoPainter {
    /// A etapa a contar da GRAVAÇÃO — o endereço de toda edição (uma apagada não se mostra, mas não
    /// renumera as outras).
    pub k: usize,
    pub etapa: Etapa,
    pub tipo: TipoDaEtapa,
    /// Início e fim na trilha QUE TOCA (com as esperas editadas).
    pub t0: MacroTime,
    pub t1: MacroTime,
    pub cor: [u8; 3],
    pub tamanho_px: f32,
    /// O meio em vigor no toque (a gravação pode trocá-lo pelo seletor do painel entre pinceladas).
    pub meio: PaintMedia,
    /// O nome que o artista escreveu.
    pub nome: Option<String>,
}

/// Passos que mudam só o estado do pincel — a leitura das propriedades aplica-os pelas portas do tool,
/// sem pintar (o ponteiro, a água e os drenos não entram).
fn e_de_configuracao(p: &Passo) -> bool {
    !matches!(
        p,
        Passo::Ponteiro { .. }
            | Passo::Hover(_)
            | Passo::Tique(_)
            | Passo::Agua(_)
            | Passo::Preview
            | Passo::PreviewBbox
            | Passo::PreviewSujo
            | Passo::PreviewAtual
            | Passo::Completo
            | Passo::Para
            | Passo::Desfaz
            | Passo::Refaz
            | Passo::PedeCommit
            | Passo::TiraCommit
            | Passo::Assado
            | Passo::TiraAssadoAdiado
            | Passo::SaiDaGravacao
            | Passo::Recusada(_)
            | Passo::Desconhecido(_)
    )
}

/// O pincel no toque de cada pincelada da trilha QUE TOCA (cor, tamanho e meio), lido por um tool no
/// estado inicial que só recebe os passos de configuração.
fn pinceis(tocavel: &Track) -> Result<Vec<([u8; 3], f32, PaintMedia)>, ErroDaMacro> {
    let mut t: PainterTool =
        inicial::tool_no_inicial(&tocavel.initial).map_err(ErroDaMacro::Inicial)?;
    let mut out = Vec::new();
    for e in &tocavel.events {
        let p = Passo::descodifica(&e.payload).map_err(ErroDaMacro::Passo)?;
        if let Passo::Ponteiro {
            fase: PointerPhase::Down,
            ..
        } = p
        {
            out.push((t.brush_color_srgb8(), t.brush_size_px(), t.paint_media()));
        } else if e_de_configuracao(&p) {
            TocadorDoPainter::aplica_ao_tool(&mut t, p)?;
        }
    }
    Ok(out)
}

/// **As etapas VIVAS de uma trilha do Painter**, com as edições dela aplicadas (uma apagada sai; a
/// numeração das pinceladas é a da gravação).
///
/// # Errors
/// [`ErroDaMacro`] (uma edição que não cabe, um estado inicial que não se lê).
pub fn etapas(track: &Track) -> Result<Vec<EtapaDoPainter>, ErroDaMacro> {
    let tocavel = track
        .tocavel(&LeitorDoPainter)
        .map_err(ErroDaMacro::Edicao)?;
    let gravadas = LeitorDoPainter.etapas(&track.events);
    let mut tocadas = LeitorDoPainter.etapas(&tocavel.events).into_iter();
    let mut pinceis = pinceis(&tocavel)?.into_iter();
    let mut n = 0;
    let mut out = Vec::with_capacity(gravadas.len());
    for (k, g) in gravadas.iter().enumerate() {
        let tipo = match passo(&track.events[g.fim].payload) {
            Some(Passo::Recusada(porta)) => TipoDaEtapa::Recusada(porta),
            _ => {
                n += 1;
                TipoDaEtapa::Pincelada(n)
            }
        };
        if apagada(track, g) {
            continue;
        }
        let Some(t) = tocadas.next() else { break };
        let (cor, tamanho_px, meio) = match tipo {
            TipoDaEtapa::Pincelada(_) => {
                pinceis
                    .next()
                    .unwrap_or(([0; 3], 0.0, PaintMedia::default()))
            }
            TipoDaEtapa::Recusada(_) => ([0; 3], 0.0, PaintMedia::default()),
        };
        out.push(EtapaDoPainter {
            k,
            etapa: *g,
            tipo,
            t0: tocavel.events[t.inicio].t,
            t1: tocavel.events[t.fim].t,
            cor,
            tamanho_px,
            meio,
            nome: track
                .edicoes
                .iter()
                .find(|e| e.ancora == g.inicio)
                .and_then(|e| e.nome.clone()),
        });
    }
    Ok(out)
}

fn apagada(track: &Track, g: &Etapa) -> bool {
    track
        .edicoes
        .iter()
        .any(|e| e.ancora == g.inicio && e.apagada)
}

/// **Apaga a etapa `k`**: o tempo dela fecha-se e a água passa ao relógio fixo ali.
///
/// # Errors
/// [`ErroDeEdicaoDoPainter::SemEtapa`].
pub fn apaga(track: &mut Track, k: usize) -> Result<(), ErroDeEdicaoDoPainter> {
    if track.apaga_etapa(&LeitorDoPainter, k) {
        Ok(())
    } else {
        Err(ErroDeEdicaoDoPainter::SemEtapa(k))
    }
}

/// Uma edição pedida a uma etapa que não a aceita.
#[derive(Clone, Debug, PartialEq)]
pub enum ErroDeEdicaoDoPainter {
    SemEtapa(usize),
    NaoEPincelada(usize),
    Macro(ErroDaMacro),
}

fn pincelada(
    track: &Track,
    k: usize,
) -> Result<(Etapa, usize, EtapaDoPainter), ErroDeEdicaoDoPainter> {
    let e = etapas(track)
        .map_err(ErroDeEdicaoDoPainter::Macro)?
        .into_iter()
        .find(|e| e.k == k)
        .ok_or(ErroDeEdicaoDoPainter::SemEtapa(k))?;
    if !matches!(e.tipo, TipoDaEtapa::Pincelada(_)) {
        return Err(ErroDeEdicaoDoPainter::NaoEPincelada(k));
    }
    let down = (e.etapa.inicio..=e.etapa.fim)
        .find(|&i| {
            matches!(
                passo(&track.events[i].payload),
                Some(Passo::Ponteiro {
                    fase: PointerPhase::Down,
                    ..
                })
            )
        })
        .expect("uma pincelada tem o seu toque");
    Ok((e.etapa, down, e))
}

/// Põe `novo` antes do toque e o que a pincelada tinha (`antigo`) depois do levantar — as inserções
/// anteriores da MESMA propriedade nesta etapa saem.
fn troca_a_volta(
    track: &mut Track,
    etapa: Etapa,
    down: usize,
    mesma: impl Fn(&Passo) -> bool,
    novo: Passo,
    antigo: Passo,
) {
    let depois = etapa.fim + 1;
    let tem_depois = depois < track.events.len();
    let ed = track.edicao_mut(etapa.inicio);
    ed.insercoes
        .retain(|(i, c)| !((*i == down || *i == depois) && passo(c).is_some_and(|p| mesma(&p))));
    ed.insercoes.push((down, novo.codifica()));
    if tem_depois {
        ed.insercoes.push((depois, antigo.codifica()));
    }
}

/// **A cor da pincelada `k`** (só dela).
///
/// # Errors
/// [`ErroDeEdicaoDoPainter`].
pub fn edita_cor(track: &mut Track, k: usize, cor: [u8; 3]) -> Result<(), ErroDeEdicaoDoPainter> {
    let (etapa, down, _) = pincelada(track, k)?;
    let original = etapas(&Track {
        edicoes: Vec::new(),
        ..track.clone()
    })
    .map_err(ErroDeEdicaoDoPainter::Macro)?[k]
        .cor;
    troca_a_volta(
        track,
        etapa,
        down,
        |p| matches!(p, Passo::Cor(_)),
        Passo::Cor(cor),
        Passo::Cor(original),
    );
    Ok(())
}

/// **O tamanho da pincelada `k`**, em píxeis (só dela).
///
/// # Errors
/// [`ErroDeEdicaoDoPainter`].
pub fn edita_tamanho(track: &mut Track, k: usize, px: f32) -> Result<(), ErroDeEdicaoDoPainter> {
    let (etapa, down, _) = pincelada(track, k)?;
    let original = etapas(&Track {
        edicoes: Vec::new(),
        ..track.clone()
    })
    .map_err(ErroDeEdicaoDoPainter::Macro)?[k]
        .tamanho_px;
    troca_a_volta(
        track,
        etapa,
        down,
        |p| matches!(p, Passo::TamanhoPx(_)),
        Passo::TamanhoPx(px),
        Passo::TamanhoPx(original),
    );
    Ok(())
}

/// **O caminho da pincelada `k` deslocado** `d` (fracções da tela de referência), a contar do
/// GRAVADO — editar outra vez substitui, não acumula.
///
/// # Errors
/// [`ErroDeEdicaoDoPainter`].
pub fn desloca_caminho(
    track: &mut Track,
    k: usize,
    d: [f64; 2],
) -> Result<(), ErroDeEdicaoDoPainter> {
    let (etapa, _, _) = pincelada(track, k)?;
    let novos: Vec<(usize, Vec<u8>)> = (etapa.inicio..=etapa.fim)
        .filter_map(|i| match passo(&track.events[i].payload)? {
            Passo::Ponteiro {
                pos,
                pressao,
                tilt,
                fase,
            } => Some((
                i,
                Passo::Ponteiro {
                    pos: [pos[0] + d[0], pos[1] + d[1]],
                    pressao,
                    tilt,
                    fase,
                }
                .codifica(),
            )),
            _ => None,
        })
        .collect();
    let ed = track.edicao_mut(etapa.inicio);
    ed.trocas
        .retain(|(i, _)| !(etapa.inicio..=etapa.fim).contains(i));
    if d != [0.0, 0.0] {
        ed.trocas.extend(novos);
    }
    Ok(())
}

/// **A espera antes da etapa `k`**, em µs (+ abre, − encurta); a etapa e as seguintes andam com ela.
///
/// # Errors
/// [`ErroDeEdicaoDoPainter::SemEtapa`].
pub fn espera(track: &mut Track, k: usize, us: i64) -> Result<(), ErroDeEdicaoDoPainter> {
    let e = *LeitorDoPainter
        .etapas(&track.events)
        .get(k)
        .ok_or(ErroDeEdicaoDoPainter::SemEtapa(k))?;
    track.edicao_mut(e.inicio).espera_us = us;
    Ok(())
}

/// **Confere a trilha editada tocando-a do zero**; se a água gravada já não responde (uma edição de
/// valor fez a água secar antes de uma marca), a 1.ª etapa editada sai da gravação e confere outra vez.
///
/// # Errors
/// O [`ErroDaMacro`] que sobra (uma porta recusada, um passo desconhecido).
pub fn assenta(track: &mut Track) -> Result<(), ErroDaMacro> {
    if let Conferida::SaiEm(a) = confere(track, None, &AtomicBool::new(false))? {
        track.edicao_mut(a).sai_da_gravacao = true;
    }
    Ok(())
}

/// De onde uma conferência recomeça: um tocador a meio da trilha que toca (com a fita dela) e quantos
/// eventos dela já despachou.
pub struct Partida {
    pub tocador: TocadorDoPainter,
    pub feitos: usize,
}

/// O que a conferência decide.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Conferida {
    /// A água gravada responde: a trilha fica como está.
    Passa,
    /// A água já não responde: a etapa que começa neste evento da gravação sai da gravação.
    SaiEm(usize),
    /// Interrompida antes do fim: sem conclusão.
    Parada,
}

/// **Até onde um instantâneo serve à conferência de `track`** (eventos da trilha que toca): o prefixo
/// que ainda é a gravação. A marca de saída da 2.ª volta cai na 1.ª etapa que muda o toque, e toda
/// edição que muda o toque muda a trilha nesse início ou antes (a janela de uma espera encurtada) — o
/// prefixo nunca o passa.
///
/// # Errors
/// [`ErroDaMacro::Edicao`].
pub fn limite_da_partida(track: &Track) -> Result<usize, ErroDaMacro> {
    let t = track
        .tocavel(&LeitorDoPainter)
        .map_err(ErroDaMacro::Edicao)?;
    Ok(t.events
        .iter()
        .zip(&track.events)
        .take_while(|(a, b)| a == b)
        .count())
}

/// A 1.ª etapa (âncora) cuja edição muda o toque — um nome não muda.
fn primeira_que_toca(track: &Track) -> Option<usize> {
    track
        .edicoes
        .iter()
        .filter(|e| {
            e.espera_us != 0
                || !e.trocas.is_empty()
                || !e.insercoes.is_empty()
                || e.sai_da_gravacao
                || e.apagada
        })
        .map(|e| e.ancora)
        .min()
}

/// **A conferência** de uma trilha editada — a decisão do [`assenta`], sem a escrever. Toca do zero, ou
/// de `partida` (um estado da trilha que toca; uma de depois do [`limite_da_partida`] ignora-se), até ao
/// fim; se a água gravada já não responde, toca outra vez com a 1.ª etapa editada fora da gravação (tem
/// de chegar ao fim). `parar` interrompe entre dois eventos (quem pediu já não quer a resposta).
///
/// # Errors
/// O [`ErroDaMacro`] que sobra (uma porta recusada, um passo desconhecido).
pub fn confere(
    track: &Track,
    partida: Option<Partida>,
    parar: &AtomicBool,
) -> Result<Conferida, ErroDaMacro> {
    let t = track
        .tocavel(&LeitorDoPainter)
        .map_err(ErroDaMacro::Edicao)?;
    // Uma partida de depois do prefixo que ainda é a gravação não serve à 2.ª volta (a marca de saída
    // cai antes dela): do zero.
    let limite = limite_da_partida(track)?;
    let partida = partida.filter(|p| p.feitos <= limite);
    let reserva = partida.as_ref().map(|p| Partida {
        tocador: p
            .tocador
            .instantaneo()
            .expect("o tocador de uma macro não grava nem tem worker nem ponte de dispositivo"),
        feitos: p.feitos,
    });
    match toca_de(&t, partida, parar) {
        Ok(true) => Ok(Conferida::Passa),
        Ok(false) => Ok(Conferida::Parada),
        Err(ErroDaMacro::Divergiu(d)) => {
            let Some(primeira) = primeira_que_toca(track) else {
                return Err(ErroDaMacro::Divergiu(d));
            };
            let mut fora = track.clone();
            fora.edicao_mut(primeira).sai_da_gravacao = true;
            let t = fora
                .tocavel(&LeitorDoPainter)
                .map_err(ErroDaMacro::Edicao)?;
            // A marca de saída não é resposta do worker: a fila da reserva já é a desta volta.
            Ok(if toca_de(&t, reserva, parar)? {
                Conferida::SaiEm(primeira)
            } else {
                Conferida::Parada
            })
        }
        Err(e) => Err(e),
    }
}

/// `true` = chegou ao fim; `false` = `parar`.
fn toca_de(t: &Track, partida: Option<Partida>, parar: &AtomicBool) -> Result<bool, ErroDaMacro> {
    let (mut toc, desde) = match partida {
        Some(p) => (p.tocador, p.feitos),
        None => (
            TocadorDoPainter::novo(&t.initial, t.events.iter().map(|e| e.payload.as_slice()))?,
            0,
        ),
    };
    for e in t.events.iter().skip(desde) {
        if parar.load(Ordering::Relaxed) {
            return Ok(false);
        }
        toc.aplica(&e.payload)?;
    }
    Ok(true)
}

/// **A tela ao fim da trilha que toca**, tocada do zero — a régua de «a edição mudou a tela».
///
/// # Errors
/// O [`ErroDaMacro`] da reprodução.
pub fn tela_no_fim(track: &Track) -> Result<Vec<u8>, ErroDaMacro> {
    let t = track
        .tocavel(&LeitorDoPainter)
        .map_err(ErroDaMacro::Edicao)?;
    let mut toc =
        TocadorDoPainter::novo(&t.initial, t.events.iter().map(|e| e.payload.as_slice()))?;
    for e in &t.events {
        toc.aplica(&e.payload)?;
    }
    Ok(ph2d_editor_core::tool::RasterEditTool::run_full(toc.tool_mut()).0)
}
