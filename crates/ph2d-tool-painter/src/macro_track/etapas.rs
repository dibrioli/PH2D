//! **As etapas da trilha do Painter** (W5, `docs/Macros/02_plano_W5.md` §1/§3): uma etapa é uma
//! pincelada (`Ponteiro Down..Up`; a cor e o tamanho dela são o pincel no toque, venham de onde vierem)
//! ou uma porta recusada. O tique, as respostas do worker e os drenos da
//! pré-visualização são o relógio entre elas.
//!
//! Editar uma pincelada toca só nela: a cor e o tamanho novos entram antes do toque e o que ela tinha
//! volta depois do levantar. Re-cronometrar sai da gravação (a água larga a fita, `Passo::SaiDaGravacao`);
//! uma edição de valor só sai se a conferência divergir ([`assenta`]).

use super::passo::Passo;
use super::tocar::{ErroDaMacro, TocadorDoPainter};
use crate::PaintMedia;
use ph2d_editor_core::tool::PointerPhase;
use ph2d_macro::{Etapa, Event, LeitorDaFamilia, MacroTime, Track};
use std::sync::atomic::{AtomicBool, Ordering};
use std::collections::BTreeMap;

pub struct LeitorDoPainter;

pub(super) fn passo(c: &[u8]) -> Option<Passo> {
    Passo::descodifica(c).ok()
}

/// O passo que fecha um ColorDrop: o Done do gesto parado, o Done ou o Cancel do modal, a volta.
fn fecha_o_drop(p: &Passo) -> bool {
    use ph2d_editor_core::ids::{PAINTER_FILL_MODAL_CANCEL, PAINTER_FILL_MODAL_DONE};
    use ph2d_editor_core::tool::PanelEvent;
    match p {
        Passo::FillCommit | Passo::ColorDropRestaura => true,
        Passo::Painel(PanelEvent::Click(id)) => {
            *id == PAINTER_FILL_MODAL_DONE || *id == PAINTER_FILL_MODAL_CANCEL
        }
        _ => false,
    }
}

/// A etapa que começa no último de `eventos` é um ColorDrop: o último marco até lá é o C&F a entrar em
/// Fill.
pub(super) fn e_um_colordrop(eventos: &[Event]) -> bool {
    eventos
        .iter()
        .rev()
        .find_map(|e| match passo(&e.payload)? {
            Passo::ColorDropComeca(_) => Some(true),
            Passo::Ponteiro {
                fase: PointerPhase::Down | PointerPhase::Up,
                ..
            } => Some(false),
            p if fecha_o_drop(&p) => Some(false),
            _ => None,
        })
        .unwrap_or(false)
}

/// O que está aberto numa faixa enquanto se leem as etapas.
#[derive(Default)]
struct Aberta {
    toque: Option<usize>,
    drop: bool,
    marco: Option<usize>,
    solto: Option<(usize, usize)>,
}

impl Aberta {
    fn fecha_solto(&mut self, out: &mut Vec<Etapa>) {
        if let Some((inicio, fim)) = self.solto.take() {
            out.push(Etapa { inicio, fim });
        }
    }

    /// Uma pincelada é `Down..Up`. Um ColorDrop (W11) vai da entrada em Fill (`ColorDropComeca`, W13:
    /// apagá-lo leva-a, senão a pincelada seguinte tocava em Fill) ao passo que o fecha — o limiar
    /// afinado depois de soltar é dele —, e o do gesto parado não tem `Up`.
    fn le(&mut self, i: usize, p: &Passo, out: &mut Vec<Etapa>) {
        match p {
            Passo::Ponteiro {
                fase: PointerPhase::Down,
                ..
            } => {
                self.fecha_solto(out);
                self.toque = Some(if self.drop { self.marco.unwrap_or(i) } else { i });
            }
            Passo::Ponteiro {
                fase: PointerPhase::Up,
                ..
            } => {
                if let Some(d) = self.toque.take() {
                    if self.drop {
                        self.solto = Some((d, i));
                    } else {
                        out.push(Etapa { inicio: d, fim: i });
                    }
                }
            }
            Passo::ColorDropComeca(_) => {
                if !self.drop {
                    self.marco = Some(i);
                }
                self.drop = true;
            }
            p if self.drop && fecha_o_drop(p) => {
                self.drop = false;
                self.marco = None;
                if let Some(d) = self.solto.take().map(|(d, _)| d).or_else(|| self.toque.take()) {
                    out.push(Etapa { inicio: d, fim: i });
                }
            }
            Passo::Recusada(_) if self.toque.is_none() => {
                self.fecha_solto(out);
                out.push(Etapa { inicio: i, fim: i });
            }
            _ => {}
        }
    }
}

impl LeitorDaFamilia for LeitorDoPainter {
    /// Uma pincelada é `Down..Up`. Um ColorDrop (W11) vai da entrada em Fill (`ColorDropComeca`, W13:
    /// apagá-lo leva-a, senão a pincelada seguinte tocava em Fill) ao passo que o fecha — o limiar
    /// afinado depois de soltar é dele —, e o do gesto parado não tem `Up`.
    fn etapas(&self, eventos: &[Event]) -> Vec<Etapa> {
        // Cada faixa tem o seu toque aberto (W16: duas pinceladas deslocadas tocam entrelaçadas); a
        // gravação só tem a faixa 0.
        let mut faixas: BTreeMap<u32, Aberta> = BTreeMap::new();
        let mut out: Vec<Etapa> = Vec::new();
        for (i, e) in eventos.iter().enumerate() {
            let Some(p) = passo(&e.payload) else { continue };
            faixas
                .entry(super::passo::faixa_de(&e.payload))
                .or_default()
                .le(i, &p, &mut out);
        }
        for a in faixas.values_mut() {
            a.fecha_solto(&mut out);
        }
        out.sort_by_key(|e| e.inicio);
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

    /// Cada pincelada deslocada abre o seu traço (W16): duas sobrepostas pintam ao mesmo tempo.
    fn na_faixa(&self, carga: &[u8], n: u32) -> Option<Vec<u8>> {
        super::passo::na_faixa(carga, n)
    }

    fn marca_da_faixa(&self, n: u32) -> Option<Vec<u8>> {
        Some(Passo::PincelDaFaixa(n).codifica())
    }

    /// O Desfazer e o Refazer agem sobre «o último» passo da pilha.
    fn e_barreira(&self, carga: &[u8]) -> bool {
        matches!(passo(carga), Some(Passo::Desfaz | Passo::Refaz))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum TipoDaEtapa {
    /// A n-ésima pincelada (a contar de 1).
    Pincelada(usize),
    /// O n-ésimo ColorDrop do C&F (a cor solta na tela que inunda), a contar de 1.
    Preenchimento(usize),
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
#[path = "etapas_no_toque.rs"]
mod no_toque;
pub(crate) use no_toque::{NoToque, pinceis, pinceis_das_faixas};
#[path = "etapas_tocadas.rs"]
mod tocadas;
pub(crate) use tocadas::{Tocada, tocadas};
#[path = "etapas_desloca.rs"]
mod desloca;
pub use desloca::desloca;

/// **As etapas VIVAS de uma trilha do Painter**, com as edições dela aplicadas (uma apagada sai; a
/// numeração das pinceladas é a da gravação).
///
/// # Errors
/// [`ErroDaMacro`] (uma edição que não cabe, um estado inicial que não se lê).
pub fn etapas(track: &Track) -> Result<Vec<EtapaDoPainter>, ErroDaMacro> {
    let tocavel = track
        .tocavel(&LeitorDoPainter)
        .map_err(ErroDaMacro::Edicao)?;
    let mut out = Vec::new();
    for e in tocadas(track, &tocavel)? {
        let (cor, tamanho_px, meio) = e
            .no_toque
            .map_or(([0; 3], 0.0, PaintMedia::default()), |p| {
                (p.cor, p.tamanho_px, p.meio)
            });
        out.push(EtapaDoPainter {
            k: e.k,
            etapa: e.gravada,
            tipo: e.tipo,
            t0: tocavel.events[e.tocada.inicio].t,
            t1: tocavel.events[e.tocada.fim].t,
            cor,
            tamanho_px,
            meio,
            nome: track
                .edicoes
                .iter()
                .find(|x| x.ancora == e.gravada.inicio)
                .and_then(|x| x.nome.clone()),
        });
    }
    Ok(out)
}

pub(super) fn apagada(track: &Track, g: &Etapa) -> bool {
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
    /// Só duas pinceladas simples tocam ao mesmo tempo (W16): um preenchimento, uma forma, uma seleção
    /// ou uma porta recusada não se sobrepõem a outra etapa.
    SoPinceladasSeSobrepoem(usize),
    /// A etapa tocaria noutro meio (duas sobrepostas de meios diferentes, ou a água do Wet Paint que a
    /// tela tem no instante novo).
    OutroMeio(usize),
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
    // O ColorDrop tem a cor e o sítio onde a cor caiu; o tamanho do pincel não lhe chega.
    if !matches!(
        e.tipo,
        TipoDaEtapa::Pincelada(_) | TipoDaEtapa::Preenchimento(_)
    ) {
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
    let (etapa, down, e) = pincelada(track, k)?;
    if !matches!(e.tipo, TipoDaEtapa::Pincelada(_)) {
        return Err(ErroDeEdicaoDoPainter::NaoEPincelada(k));
    }
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

/// As imagens do início e do fim de uma gravação (RGBA8 straight, `w`×`h`), e o documento do fim.
pub struct ImagensDaGravacao {
    pub inicio: Vec<u8>,
    pub fim: Vec<u8>,
    pub w: u32,
    pub h: u32,
    pub modelo: ModeloDoFim,
}

/// **O documento do fim de uma gravação** (camadas, relevo, cobertura, material) — a régua por texel da
/// reescrita de uma etapa editada ([`PainterTool::reescreve_a_gravacao`], W8 item 2b).
pub struct ModeloDoFim(pub(crate) crate::undo::ModelSnapshot);

/// ⭐ **Quadros (60 Hz) que a água da imagem do fim corre no máximo depois do último evento** (W8 item
/// 2c, `docs/Macros/05_plano_W8.md`): a água do editor continua depois do Parar até parar sozinha, e a
/// imagem do fim é essa — assentada. Medido (W7, `QUADROS_ATE_ASSENTAR` da régua): a água das
/// pinceladas de teste pára em 2 645–3 562 quadros. O tecto é de CPU do fio do fim: 10 min de
/// simulação, ~45 s de um núcleo a 1024² (1,24 ms por passo, `04_plano_W7.md` item 6).
pub const QUADROS_ATE_A_AGUA_DO_FIM_PARAR: u32 = 36_000;

/// **As imagens ao início e ao fim da gravação, como ela TOCA** — com as edições das etapas (W8 item
/// 2b: editar uma etapa muda a imagem) e a água assentada (2c) —, como o Apply as escreve na sprite. A
/// régua de «o artista pintou depois do Parar» (W7 item 1). `Ok(None)` = `parar`.
///
/// # Errors
/// O [`ErroDaMacro`] da reprodução.
pub fn imagens_da_gravacao(
    track: &Track,
    parar: &AtomicBool,
) -> Result<Option<ImagensDaGravacao>, ErroDaMacro> {
    let tocavel = track
        .tocavel(&LeitorDoPainter)
        .map_err(ErroDaMacro::Edicao)?;
    let mut toc = TocadorDoPainter::novo(
        &tocavel.initial,
        tocavel.events.iter().map(|e| e.payload.as_slice()),
    )?;
    // ⛔ Nunca o `run_full`: numa pilha trivial ele LEVA a tela, e o fim saía vazio (W7).
    let vazia = || (Vec::new(), 0, 0);
    let (inicio, wi, hi) = toc.tool().imagem_aplicada().unwrap_or_else(vazia);
    for e in &tocavel.events {
        if parar.load(Ordering::Relaxed) {
            return Ok(None);
        }
        toc.aplica(&e.payload)?;
    }
    toc.passa_ao_relogio_fixo();
    let t = toc.tool_mut();
    for _ in 0..QUADROS_ATE_A_AGUA_DO_FIM_PARAR {
        if t.wet_assentada() {
            break;
        }
        if parar.load(Ordering::Relaxed) {
            return Ok(None);
        }
        t.drena_sem_compor();
        ph2d_editor_core::tool::Tool::on_tick(t, super::roteiro::DT_MS);
    }
    let (fim, w, h) = t.imagem_aplicada().unwrap_or_else(vazia);
    let modelo = ModeloDoFim(t.snapshot_model());
    // Uma gravação que muda o tamanho da tela não tem píxeis do início a comparar.
    let inicio = if (wi, hi) == (w, h) {
        inicio
    } else {
        fim.clone()
    };
    Ok(Some(ImagensDaGravacao {
        inicio,
        fim,
        w,
        h,
        modelo,
    }))
}
