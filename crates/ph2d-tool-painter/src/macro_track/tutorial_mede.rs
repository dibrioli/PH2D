//! **Medir as curvas de cada critério** (W7, `docs/Macros/04_plano_W7.md` item 2): a reprodução da
//! trilha que toca corre num fio; cada medida (a gravada e a base de uma pincelada, cada uma
//! [`base::assentada`]) corre num de poucos fios trabalhadores, com uma fila curta — nunca dezenas de
//! instantâneos vivos de uma vez.

use super::base::{self, Amostras, SemAPincelada};
use super::{Curva, EnsinoDoPainter, LeitorDoPainter, Medida, Referencia, pinceladas_em, toca};
use crate::macro_track::passo::Passo;
use crate::macro_track::tocar::{ErroDaMacro, TocadorDoPainter};
use ph2d_editor_core::tool::PointerPhase;
use ph2d_macro::Track;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender, SyncSender, TrySendError};
use std::sync::{Mutex, PoisonError};

/// ⭐ **Quadros entre dois pontos de uma curva** (depois do corte; antes dele, um por evento). MEDIDO
/// (W7, sondas `sonda_quanto_espera_o_aluno` e `sonda_o_custo_do_passo`, smoke): o aluno que pula a
/// pincelada e espera chega, nas idades amostradas, a (máx., % da régua; 30 pinceladas Wet Paint 512²)
///
/// | passo | 1 | 4 | 8 | 16 | 32 |
/// |---|---|---|---|---|---|
/// | Nada nas curvas | **0,0** | 1,8 | 2,2 | 2,9 | 6,4 |
///
/// e ler a cada quadro não custa nada que se meça (`mede` do Splash 512², 4 fios: 29,1 s com 1 ×
/// 30,0 s com 8, load 20 → 12): a simulação da água (~7 ms por quadro) é tudo.
pub(super) const PASSO_DA_CURVA: u32 = 1;

/// Os fios trabalhadores de uma medição: um quarto dos núcleos (o editor e as outras famílias ficam
/// com o resto), pelo menos um.
pub(super) fn fios() -> usize {
    std::thread::available_parallelism().map_or(1, |n| n.get() / 4).max(1)
}

/// Pararam a medição: quem a pediu (a `Macro` mudou) ou ela própria (um erro a meio).
#[derive(Clone, Copy)]
pub(super) struct Paragem<'a> {
    de_fora: &'a AtomicBool,
    daqui: &'a AtomicBool,
}

impl Paragem<'_> {
    pub(super) fn pediu(self) -> bool {
        self.de_fora.load(Ordering::Relaxed) || self.daqui.load(Ordering::Relaxed)
    }
}

/// Uma medida por fazer: a `i`-ésima pincelada medida, a gravada ou a base.
enum Trabalho<'a> {
    Gravada {
        i: usize,
        c: usize,
        toc: TocadorDoPainter,
        antes: Amostras,
    },
    Base {
        i: usize,
        s: &'a SemAPincelada,
        recomeco: Option<TocadorDoPainter>,
        antes: Amostras,
    },
}

/// As leituras de uma medida na reprodução: depois de cada evento `desde..ate` (do último `Up` antes
/// do corte até ao corte), do critério `c`.
struct Janela {
    desde: usize,
    ate: usize,
    c: usize,
    amostras: Amostras,
}

/// O que um fio devolve: a `i`-ésima, se é a base, e a curva (`None` = parou).
type Feita = (usize, bool, Result<Option<Curva>, ErroDaMacro>);

impl EnsinoDoPainter {
    /// **Mede cada critério** de `track` — a curva da gravada e a da base ao longo do assentar
    /// ([`base::assentada`]), com a régua do aluno (o «pintado»), a partir do toque da pincelada
    /// SEGUINTE (ou do fim):
    ///
    /// - `gravada`: a trilha que toca, reproduzida do estado inicial — a idade desde o `Up` da
    ///   pincelada;
    /// - `base`: a MESMA trilha com a pincelada apagada ([`super::etapas::apaga`]) e o tempo dela
    ///   devolvido à etapa seguinte ([`super::etapas::espera`]) — a idade desde o `Up` da anterior.
    ///   Recomeça do instantâneo da reprodução no 1.º evento em que as duas diferem (W5c).
    ///
    /// Uma porta recusada salta-se (o tutorial não a ensina). `chega` recebe cada medida pela ordem
    /// das pinceladas, assim que as duas curvas dela acabam. `Ok(false)` = `parar` pediu-o a meio.
    ///
    /// # Errors
    /// [`ErroDaMacro`] — a reprodução divergiu.
    pub fn mede(
        &self,
        track: &Track,
        parar: &AtomicBool,
        chega: impl FnMut(Medida),
    ) -> Result<bool, ErroDaMacro> {
        self.mede_com(track, parar, PASSO_DA_CURVA, fios(), chega)
    }

    /// [`Self::mede`] com `passo` quadros entre os pontos das curvas e `n` fios trabalhadores.
    pub(super) fn mede_com(
        &self,
        track: &Track,
        parar: &AtomicBool,
        passo: u32,
        n: usize,
        chega: impl FnMut(Medida),
    ) -> Result<bool, ErroDaMacro> {
        let tocavel = track
            .tocavel(&LeitorDoPainter)
            .map_err(ErroDaMacro::Edicao)?;
        let ps = pinceladas_em(track, &tocavel)?;
        // A i-ésima medida: (o evento antes do qual se mede a gravada, o critério).
        let mut fronteiras = Vec::new();
        let mut bases = Vec::new();
        for (j, p) in ps.iter().enumerate() {
            let Some(c) = self.criterios.iter().position(|c| c.regiao.k == p.k) else {
                continue;
            };
            let fronteira = ps.get(j + 1).map_or(tocavel.events.len(), |q| q.inicio);
            bases.push((
                fronteiras.len(),
                SemAPincelada::de(track, &tocavel, p.k, c, fronteira)?,
            ));
            fronteiras.push((fronteira, c));
        }
        bases.sort_by_key(|(_, b)| b.desde);
        let daqui = AtomicBool::new(false);
        let p = Paragem {
            de_fora: parar,
            daqui: &daqui,
        };
        let (tx, rx) = mpsc::sync_channel::<Trabalho<'_>>(n.max(1));
        let rx = Mutex::new(rx);
        let (feitas_tx, feitas) = mpsc::channel::<Feita>();
        std::thread::scope(|sc| {
            for _ in 0..n.max(1) {
                let (rx, feitas_tx) = (&rx, feitas_tx.clone());
                sc.spawn(move || trabalha(self, track, passo, p, rx, &feitas_tx));
            }
            drop(feitas_tx);
            let mut r = Reproducao {
                e: self,
                fronteiras: &fronteiras,
                prontas: vec![(None, None); fronteiras.len()],
                proxima: 0,
                feitas,
                chega,
            };
            let fim = r.corre(&tocavel, &bases, tx, p);
            if !matches!(fim, Ok(true)) {
                daqui.store(true, Ordering::Relaxed);
            }
            fim
        })
    }
}

/// Um fio trabalhador: mede até a fila fechar.
fn trabalha<'a>(
    e: &EnsinoDoPainter,
    track: &Track,
    passo: u32,
    p: Paragem<'_>,
    rx: &Mutex<Receiver<Trabalho<'a>>>,
    feitas: &Sender<Feita>,
) {
    loop {
        let Ok(t) = rx.lock().unwrap_or_else(PoisonError::into_inner).recv() else {
            return;
        };
        let feita = match t {
            Trabalho::Gravada { i, c, toc, antes } => (
                i,
                false,
                Ok(base::assentada(e, toc, c, antes, passo, p)),
            ),
            Trabalho::Base {
                i,
                s,
                recomeco,
                antes,
            } => (i, true, s.mede(e, track, recomeco, antes, passo, p)),
        };
        if feitas.send(feita).is_err() {
            return;
        }
    }
}

/// A reprodução que distribui as medidas e junta-as pela ordem das pinceladas.
struct Reproducao<'a, F> {
    e: &'a EnsinoDoPainter,
    fronteiras: &'a [(usize, usize)],
    /// (gravada, base) de cada medida.
    prontas: Vec<(Option<Curva>, Option<Curva>)>,
    /// A 1.ª medida por mandar.
    proxima: usize,
    feitas: Receiver<Feita>,
    chega: F,
}

impl<F: FnMut(Medida)> Reproducao<'_, F> {
    /// Reproduz `tocavel`, manda cada medida aos fios e junta-as. A fila `tx` fecha ao sair (os fios
    /// acabam).
    fn corre<'s>(
        &mut self,
        tocavel: &Track,
        bases: &'s [(usize, SemAPincelada)],
        tx: SyncSender<Trabalho<'s>>,
        p: Paragem<'_>,
    ) -> Result<bool, ErroDaMacro> {
        let mut toc = TocadorDoPainter::novo(
            &tocavel.initial,
            tocavel.events.iter().map(|e| e.payload.as_slice()),
        )?;
        let ups: Vec<usize> = tocavel
            .events
            .iter()
            .enumerate()
            .filter(|(_, e)| {
                matches!(
                    Passo::descodifica(&e.payload),
                    Ok(Passo::Ponteiro {
                        fase: PointerPhase::Up,
                        ..
                    })
                )
            })
            .map(|(i, _)| i)
            .collect();
        let janela = |ate: usize, c: usize| Janela {
            desde: ups.iter().rev().find(|&&u| u < ate).copied().unwrap_or(ate),
            ate,
            c,
            amostras: Amostras::default(),
        };
        // As janelas de cada medida: as das bases, depois as das gravadas.
        let mut janelas: Vec<Janela> = bases
            .iter()
            .map(|(_, s)| janela(s.desde, s.c))
            .chain(self.fronteiras.iter().map(|&(x, c)| janela(x, c)))
            .collect();
        let (mut f, mut b) = (
            self.fronteiras.iter().enumerate().peekable(),
            bases.iter().enumerate().peekable(),
        );
        for ev in 0..=tocavel.events.len() {
            while let Some((j, (i, s))) = b.next_if(|(_, (_, s))| s.desde == ev) {
                let t = Trabalho::Base {
                    i: *i,
                    s,
                    recomeco: s.recomeco(&toc),
                    antes: std::mem::take(&mut janelas[j].amostras),
                };
                if !self.manda(&tx, t)? {
                    return Ok(false);
                }
            }
            while let Some((i, &(_, c))) = f.next_if(|(_, (antes_de, _))| *antes_de == ev) {
                let copia = toc
                    .instantaneo()
                    .expect("o tool de um tocador nunca grava nem tem worker: tem instantâneo");
                let antes = std::mem::take(&mut janelas[bases.len() + i].amostras);
                if !self.manda(&tx, Trabalho::Gravada { i, c, toc: copia, antes })? {
                    return Ok(false);
                }
            }
            while let Ok(x) = self.feitas.try_recv() {
                if !self.junta(x)? {
                    return Ok(false);
                }
            }
            let Some(e) = tocavel.events.get(ev) else {
                break;
            };
            if f.peek().is_none() && b.peek().is_none() {
                break; // nenhuma medida depois daqui
            }
            if p.pediu() {
                return Ok(false);
            }
            toca(&mut toc, &e.payload)?;
            for j in janelas.iter_mut().filter(|j| j.desde <= ev && ev < j.ate) {
                j.amostras.le(self.e, toc.tool(), j.c);
            }
        }
        drop(tx);
        while self.proxima < self.prontas.len() {
            let x = self
                .feitas
                .recv()
                .expect("os fios da medição acabaram com medidas por fazer");
            if !self.junta(x)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// Põe `t` na fila; cheia, junta uma medida feita enquanto espera. `false` = parou.
    fn manda<'s>(
        &mut self,
        tx: &SyncSender<Trabalho<'s>>,
        mut t: Trabalho<'s>,
    ) -> Result<bool, ErroDaMacro> {
        loop {
            match tx.try_send(t) {
                Ok(()) => return Ok(true),
                Err(TrySendError::Full(de_volta)) => {
                    t = de_volta;
                    let x = self
                        .feitas
                        .recv()
                        .expect("os fios da medição acabaram com a fila cheia");
                    if !self.junta(x)? {
                        return Ok(false);
                    }
                }
                Err(TrySendError::Disconnected(_)) => {
                    unreachable!("os fios só largam a fila quando ela fecha")
                }
            }
        }
    }

    /// Guarda uma curva feita e manda as medidas completas pela ordem. `false` = parou.
    fn junta(&mut self, (i, e_base, r): Feita) -> Result<bool, ErroDaMacro> {
        let Some(curva) = r? else {
            return Ok(false);
        };
        let lugar = &mut self.prontas[i];
        if e_base {
            lugar.1 = Some(curva);
        } else {
            lugar.0 = Some(curva);
        }
        while let Some((Some(_), Some(_))) = self.prontas.get(self.proxima) {
            let (g, b) = std::mem::take(&mut self.prontas[self.proxima]);
            let c = self.fronteiras[self.proxima].1;
            (self.chega)(Medida {
                k: self.e.criterios[c].regiao.k,
                referencia: Referencia {
                    gravada: g.expect("acima"),
                    base: b.expect("acima"),
                },
            });
            self.proxima += 1;
        }
        Ok(true)
    }
}
