//! **As etapas da trilha do Painter** (W5, `docs/Macros/02_plano_W5.md` §1/§3): uma etapa é uma
//! pincelada (`Ponteiro Down..Up`, a começar no 1.º evento do quadro do toque: a cor e o tamanho
//! escolhidos para ela estão lá) ou uma porta recusada. O tique, as respostas do worker e os drenos da
//! pré-visualização são o relógio entre elas.
//!
//! Editar uma pincelada toca só nela: a cor e o tamanho novos entram antes do toque e o que ela tinha
//! volta depois do levantar. Re-cronometrar sai da gravação (a água larga a fita, `Passo::SaiDaGravacao`);
//! uma edição de valor só sai se a conferência divergir ([`assenta`]).

use super::inicial;
use super::passo::Passo;
use super::tocar::{ErroDaMacro, TocadorDoPainter};
use crate::tool::PainterTool;
use ph2d_editor_core::tool::PointerPhase;
use ph2d_macro::{Etapa, Event, LeitorDaFamilia, MacroTime, Track};

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
                        let livre = out.last().map_or(0, |a| a.fim + 1);
                        let inicio = (livre..=d)
                            .find(|&j| eventos[j].t == eventos[d].t)
                            .unwrap_or(d);
                        out.push(Etapa { inicio, fim: i });
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
    pub etapa: Etapa,
    pub tipo: TipoDaEtapa,
    /// Início e fim na trilha QUE TOCA (com as esperas editadas).
    pub t0: MacroTime,
    pub t1: MacroTime,
    pub cor: [u8; 3],
    pub tamanho_px: f32,
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

/// O pincel no toque de cada pincelada da trilha QUE TOCA, lido por um tool no estado inicial que só
/// recebe os passos de configuração.
fn pinceis(tocavel: &Track) -> Result<Vec<([u8; 3], f32)>, ErroDaMacro> {
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
            out.push((t.brush_color_srgb8(), t.brush_size_px()));
        } else if e_de_configuracao(&p) {
            TocadorDoPainter::aplica_ao_tool(&mut t, p)?;
        }
    }
    Ok(out)
}

/// **As etapas de uma trilha do Painter**, com as edições dela aplicadas.
///
/// # Errors
/// [`ErroDaMacro`] (uma edição que não cabe, um estado inicial que não se lê).
pub fn etapas(track: &Track) -> Result<Vec<EtapaDoPainter>, ErroDaMacro> {
    let tocavel = track.tocavel(&LeitorDoPainter).map_err(ErroDaMacro::Edicao)?;
    let gravadas = LeitorDoPainter.etapas(&track.events);
    let tocadas = LeitorDoPainter.etapas(&tocavel.events);
    let pinceis = pinceis(&tocavel)?;
    let mut n = 0;
    let mut out = Vec::with_capacity(gravadas.len());
    for (g, t) in gravadas.iter().zip(&tocadas) {
        let tipo = match passo(&track.events[g.fim].payload) {
            Some(Passo::Recusada(porta)) => TipoDaEtapa::Recusada(porta),
            _ => {
                n += 1;
                TipoDaEtapa::Pincelada(n)
            }
        };
        let (cor, tamanho_px) = match tipo {
            TipoDaEtapa::Pincelada(i) => pinceis.get(i - 1).copied().unwrap_or(([0; 3], 0.0)),
            TipoDaEtapa::Recusada(_) => ([0; 3], 0.0),
        };
        out.push(EtapaDoPainter {
            etapa: *g,
            tipo,
            t0: tocavel.events[t.inicio].t,
            t1: tocavel.events[t.fim].t,
            cor,
            tamanho_px,
            nome: track
                .edicoes
                .iter()
                .find(|e| e.ancora == g.inicio)
                .and_then(|e| e.nome.clone()),
        });
    }
    Ok(out)
}

/// Uma edição pedida a uma etapa que não a aceita.
#[derive(Clone, Debug, PartialEq)]
pub enum ErroDeEdicaoDoPainter {
    SemEtapa(usize),
    NaoEPincelada(usize),
    Macro(ErroDaMacro),
}

fn pincelada(track: &Track, k: usize) -> Result<(Etapa, usize, EtapaDoPainter), ErroDeEdicaoDoPainter> {
    let todas = etapas(track).map_err(ErroDeEdicaoDoPainter::Macro)?;
    let e = todas.get(k).cloned().ok_or(ErroDeEdicaoDoPainter::SemEtapa(k))?;
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
    ed.insercoes.retain(|(i, c)| {
        !((*i == down || *i == depois) && passo(c).is_some_and(|p| mesma(&p)))
    });
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
    match toca_tudo(track) {
        Err(ErroDaMacro::Divergiu(_)) => {
            let Some(primeira) = track.edicoes.iter().map(|e| e.ancora).min() else {
                return toca_tudo(track);
            };
            track.edicao_mut(primeira).sai_da_gravacao = true;
            toca_tudo(track)
        }
        r => r,
    }
}

fn toca_tudo(track: &Track) -> Result<(), ErroDaMacro> {
    let t = track.tocavel(&LeitorDoPainter).map_err(ErroDaMacro::Edicao)?;
    let mut toc = TocadorDoPainter::novo(&t.initial, t.events.iter().map(|e| e.payload.as_slice()))?;
    for e in &t.events {
        toc.aplica(&e.payload)?;
    }
    Ok(())
}

#[cfg(test)]
pub(crate) fn tela_no_fim(track: &Track) -> Result<Vec<u8>, ErroDaMacro> {
    let t = track.tocavel(&LeitorDoPainter).map_err(ErroDaMacro::Edicao)?;
    let mut toc = TocadorDoPainter::novo(&t.initial, t.events.iter().map(|e| e.payload.as_slice()))?;
    for e in &t.events {
        toc.aplica(&e.payload)?;
    }
    Ok(ph2d_editor_core::tool::RasterEditTool::run_full(toc.tool_mut()).0)
}
