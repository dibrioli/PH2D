//! **A base de um critério** do tutorial do Painter (W6d, `docs/Macros/03_plano_W6.md` §6.3): a
//! trilha que toca SEM a pincelada, e a curva de cada medida ao longo do assentar (W7, [`assentada`]).

use super::mede::Paragem;
use super::{Curva, EnsinoDoPainter, idade_de, toca};
use crate::macro_track::etapas::{self, LeitorDoPainter};
use crate::macro_track::tocar::{ErroDaMacro, Recomeco, TocadorDoPainter};
use crate::tool::PainterTool;
use ph2d_macro::{LeitorDaFamilia, Track};

/// **A trilha que toca sem a pincelada `k`**: apagada pela edição do produto, e o tempo dela devolvido
/// à etapa viva seguinte — o toque seguinte cai no mesmo instante que na trilha que toca.
fn sem_a_pincelada(track: &Track, k: usize) -> Result<Track, ErroDaMacro> {
    let erro = |e| match e {
        etapas::ErroDeEdicaoDoPainter::Macro(x) => x,
        _ => ErroDaMacro::PortaRecusada(format!("etapa {k}")),
    };
    // O estado inicial não muda: não se copia.
    let mut t = Track {
        family: track.family.clone(),
        target: track.target,
        initial: Vec::new(),
        events: track.events.clone(),
        edicoes: track.edicoes.clone(),
    };
    let gravadas = LeitorDoPainter.etapas(&track.events);
    let e = gravadas[k];
    let dur = track.events[e.fim].t.0 - track.events[e.inicio].t.0;
    etapas::apaga(&mut t, k).map_err(erro)?;
    let viva = |g: &ph2d_macro::Etapa| {
        !t.edicoes
            .iter()
            .any(|ed| ed.ancora == g.inicio && ed.apagada)
    };
    if let Some(s) = (k + 1..gravadas.len()).find(|&s| viva(&gravadas[s])) {
        let ja = t
            .edicoes
            .iter()
            .find(|ed| ed.ancora == gravadas[s].inicio)
            .map_or(0, |ed| ed.espera_us);
        let us = ja.saturating_add(i64::try_from(dur).unwrap_or(i64::MAX));
        etapas::espera(&mut t, s, us).map_err(erro)?;
    }
    t.tocavel(&LeitorDoPainter).map_err(ErroDaMacro::Edicao)
}

/// A base de um critério por medir: os eventos da trilha sem a pincelada desde o 1.º que difere da
/// trilha que toca até ao instante da medida.
pub(super) struct SemAPincelada {
    k: usize,
    pub(super) c: usize,
    /// O 1.º evento em que as duas trilhas diferem (o mesmo índice nas duas).
    pub(super) desde: usize,
    /// Os eventos da trilha sem a pincelada `desde..` o instante da medida.
    eventos: Vec<Vec<u8>>,
    /// As respostas do worker e os pincéis das faixas da trilha sem a pincelada (o instantâneo passa a
    /// tocá-la).
    respostas: Recomeco,
}

impl SemAPincelada {
    pub(super) fn de(
        track: &Track,
        tocavel: &Track,
        k: usize,
        c: usize,
        fronteira: usize,
    ) -> Result<Self, ErroDaMacro> {
        let sem = sem_a_pincelada(track, k)?;
        let desde = tocavel
            .events
            .iter()
            .zip(&sem.events)
            .position(|(a, b)| a != b)
            .unwrap_or(tocavel.events.len().min(sem.events.len()))
            .min(fronteira);
        // O toque seguinte na trilha sem ela: as etapas são as mesmas menos a dela, antes dele.
        let ate = if fronteira < tocavel.events.len() {
            let st = LeitorDoPainter.etapas(&tocavel.events);
            let ss = LeitorDoPainter.etapas(&sem.events);
            st.iter()
                .position(|e| e.inicio == fronteira)
                .filter(|&s| s > 0 && ss.len() + 1 == st.len())
                .map(|s| ss[s - 1].inicio)
                .ok_or_else(|| ErroDaMacro::PortaRecusada(format!("etapa {k}")))?
        } else {
            sem.events.len()
        }
        .max(desde);
        let respostas = Recomeco::de(
            &sem.initial,
            sem.events.iter().map(|e| e.payload.as_slice()),
        )?;
        Ok(Self {
            k,
            c,
            desde,
            eventos: sem.events[desde..ate]
                .iter()
                .map(|e| e.payload.clone())
                .collect(),
            respostas,
        })
    }

    /// O recomeço da reprodução da trilha que toca parada em [`Self::desde`] (`None` = do inicial).
    pub(super) fn recomeco(&self, toc: &TocadorDoPainter) -> Option<TocadorDoPainter> {
        toc.recomeca(&self.respostas)
    }

    /// A curva dos `novos` que a trilha sem a pincelada pinta, a partir de `recomeco`
    /// ([`Self::recomeco`]) e das amostras da reprodução desde o último `Up` até lá (`antes`). `None` =
    /// parou.
    pub(super) fn mede(
        &self,
        e: &EnsinoDoPainter,
        track: &Track,
        recomeco: Option<TocadorDoPainter>,
        mut antes: Amostras,
        passo: u32,
        parar: Paragem<'_>,
    ) -> Result<Option<Curva>, ErroDaMacro> {
        let mut b = match recomeco {
            Some(b) => b,
            // O instantâneo não é um estado da trilha sem ela (a fita já consumiu uma resposta da
            // pincelada): do estado inicial.
            None => {
                let sem = sem_a_pincelada(track, self.k)?;
                let mut b = TocadorDoPainter::novo(
                    &track.initial,
                    sem.events.iter().map(|e| e.payload.as_slice()),
                )?;
                antes = Amostras::default();
                for ev in &sem.events[..self.desde] {
                    if parar.pediu() {
                        return Ok(None);
                    }
                    toca(&mut b, &ev.payload)?;
                    antes.le(e, b.tool(), self.c);
                }
                b
            }
        };
        for ev in &self.eventos {
            if parar.pediu() {
                return Ok(None);
            }
            toca(&mut b, ev)?;
            antes.le(e, b.tool(), self.c);
        }
        Ok(assentada(e, b, self.c, antes, passo, parar))
    }
}

/// Os pontos de uma curva a crescer: um `Up` (a idade volta atrás) recomeça-a; a mesma idade fica com
/// a última leitura.
#[derive(Default)]
pub(super) struct Amostras(Vec<(u32, f32)>);

impl Amostras {
    /// Lê a fracção do critério `c` em `t`, na idade dele (sem `Up` nenhum, nada).
    pub(super) fn le(&mut self, e: &EnsinoDoPainter, t: &PainterTool, c: usize) {
        let Some(a) = idade_de(t) else {
            return;
        };
        let f = e.fracao_pintada(t, &e.criterios[c]);
        match self.0.last_mut() {
            Some(p) if p.0 == a => p.1 = f,
            Some(p) if p.0 > a => {
                self.0.clear();
                self.0.push((a, f));
            }
            _ => self.0.push((a, f)),
        }
    }
}

/// **A curva dos `novos` do critério `c` desde o último `Up`** (W7, `docs/Macros/04_plano_W7.md`):
/// `antes` = as leituras da reprodução desde lá (a cada evento); depois `toc` continua sem eventos, no
/// relógio fixo, até o motor parar ([`PainterTool::wet_assentada`]) ou [`QUADROS_ATE_ASSENTAR`], lido a
/// cada `passo` quadros e no fim. A idade é a do tool ([`idade_de`]); uma trilha sem `Up` não a tem: a
/// curva é a fracção assentada. `None` = parou.
pub(super) fn assentada(
    e: &EnsinoDoPainter,
    toc: TocadorDoPainter,
    c: usize,
    pontos: Amostras,
    passo: u32,
    parar: Paragem<'_>,
) -> Option<Curva> {
    assentada_por(e, toc, c, pontos, passo, parar, QUADROS_ATE_ASSENTAR)
}

/// [`assentada`] com no máximo `quadros` quadros depois do corte.
pub(super) fn assentada_por(
    e: &EnsinoDoPainter,
    mut toc: TocadorDoPainter,
    c: usize,
    mut pontos: Amostras,
    passo: u32,
    parar: Paragem<'_>,
    quadros: u32,
) -> Option<Curva> {
    toc.passa_ao_relogio_fixo();
    let t = toc.tool_mut();
    let mut q = 0;
    loop {
        let parou = q >= quadros || t.wet_assentada();
        if parou || q % passo.max(1) == 0 {
            pontos.le(e, t, c);
        }
        if parou {
            break;
        }
        if parar.pediu() {
            return None;
        }
        drop(t.take_preview_arc());
        ph2d_editor_core::tool::Tool::on_tick(t, crate::macro_track::roteiro::DT_MS);
        q += 1;
    }
    Some(if pontos.0.is_empty() {
        Curva::constante(e.fracao_pintada(t, &e.criterios[c]))
    } else {
        Curva(pontos.0)
    })
}

/// ⭐ **Quadros (60 Hz) que a curva de uma medida corre depois do corte** (W7, `docs/Macros/04_plano_W7.md`
/// item 2) — ou menos, se a água parar; depois do último ponto a curva vale o último. Esperar a água PARAR custa 2 645–3 562 quadros por medida em Wet Paint
/// (a fracção muda até ~2 600, com pausas de até 413), e as medidas são duas por pincelada. Medida num
/// instante T, o aluno que pula e espera PARA SEMPRE chega a `(Nada_final − Nada_T) / régua_T`; a gravada
/// ainda cresce `(Exacta_final − Exacta_T) / régua_T` (sonda `sonda_quando_a_fracao_assenta`, o máximo;
/// Splash 512² / 1024² / 8 das 30 pinceladas):
///
/// | T | 60 | 120 | 240 | 480 | **960** | 1920 |
/// |---|---|---|---|---|---|---|
/// | erro do Nada (%) | 25,4 / 12,7 / 73,4 | 25,4 / 12,6 / 73,2 | 25,2 / 12,6 / 58,8 | 20,4 / 11,1 / 24,4 | **9,0 / 5,3 / 7,9** | 0,6 / 0,3 / 0,6 |
/// | a gravada ainda cresce (%) | 5,1 / 1,6 / 89,4 | 5,0 / 1,6 / 89,4 | 4,8 / 1,4 / 65,7 | 0,5 / 0,7 / 10,9 | **0,03 / 0,1 / 0,6** | 0 |
///
/// 960: o Nada fica a ≤ 9 % contra a fracção que cumpre (~67 %); a régua a ≤ 0,6 % da final; 3,4× menos
/// que esperar a água parar, e um custo que não depende de ela secar (a «água viva»). Com as curvas
/// (`sonda_quanto_espera_o_aluno`): o Nada que espera além da última idade chega a ≤ 8,3 / 4,9 / 7,3 %
/// (Splash 512² / 1024² / 30 pinceladas).
pub(super) const QUADROS_ATE_ASSENTAR: u32 = 960;
