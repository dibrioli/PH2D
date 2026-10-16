//! **O tutorial «fazer» do Painter** (W6b, `docs/Macros/03_plano_W6.md` §2 e §6): cada pincelada
//! gravada é uma REGIÃO da tela — o caminho do pincel com o raio dele — e cumpre-se quando o aluno
//! pintou ali. A lei é a da W6a: o progresso é função do ESTADO (as camadas do aluno contra as do
//! estado inicial), por isso o Ctrl+Z volta o passo sem gancho nenhum.
//!
//! - **A região conta-se pela parte NOVA**: os píxeis que nenhuma pincelada anterior da gravação
//!   cobre. Uma pincelada por cima de outra cumpria-se com a tinta da anterior.
//! - **«Pintado»** = algum píxel de alguma camada difere do estado inicial (uma camada que o aluno
//!   criou conta pela tinta que tem).
//! - **A régua é a GRAVADA** (W6c) **sem a água** (W6d): cumpre-se quando o aluno acrescenta à água
//!   [`FRACAO_DA_GRAVADA_QUE_CUMPRE`] do que a pincelada gravada acrescenta da parte nova — a base é a
//!   gravação SEM a pincelada (a aguada da anterior pinta sozinha uma fatia nova).
//! - **«Mostra-me»** = o caminho gravado pelas portas do tool vivo, com a cor e o tamanho da
//!   pincelada (os do aluno voltam depois): um passo de undo.

use super::etapas::{self, LeitorDoPainter, TipoDaEtapa};
use super::inicial;
use super::passo::{Passo, abs};
use super::tocar::{ErroDaMacro, TocadorDoPainter};
use crate::PaintMedia;
use crate::tool::PainterTool;
use ph2d_macro::{LeitorDaFamilia, Track};
use std::sync::atomic::{AtomicBool, Ordering};

/// A pincelada `k` (o endereço da GRAVAÇÃO) numa tela `w × h`.
#[derive(Clone, Debug, PartialEq)]
pub struct RegiaoDaPincelada {
    pub k: usize,
    /// O caminho do pincel, em píxeis da tela.
    pub pontos: Vec<[f32; 2]>,
    pub raio: f32,
    pub cor: [u8; 3],
    /// O meio em vigor no toque dela (a gravação pode trocá-lo entre pinceladas).
    pub meio: PaintMedia,
}

/// Uma pincelada da trilha QUE TOCA: o endereço, o evento do toque, o pincel no toque e os passos do
/// ponteiro.
struct Pinc {
    k: usize,
    /// O índice do `Down` nos eventos da trilha que toca.
    inicio: usize,
    cor: [u8; 3],
    raio: f32,
    meio: PaintMedia,
    passos: Vec<Passo>,
}

/// Cada pincelada da trilha QUE TOCA (com as edições).
fn pinceladas(track: &Track) -> Result<Vec<Pinc>, ErroDaMacro> {
    let tocavel = track
        .tocavel(&LeitorDoPainter)
        .map_err(ErroDaMacro::Edicao)?;
    pinceladas_em(track, &tocavel)
}

/// As pinceladas de `track` nos eventos da trilha que toca `tocavel` (a dela).
fn pinceladas_em(track: &Track, tocavel: &Track) -> Result<Vec<Pinc>, ErroDaMacro> {
    let tocadas = LeitorDoPainter.etapas(&tocavel.events);
    let mut out = Vec::new();
    // `etapas` lê uma tocada por etapa viva, pela ordem: as duas listas andam juntas.
    for (e, t) in etapas::etapas(track)?.into_iter().zip(tocadas) {
        if !matches!(e.tipo, TipoDaEtapa::Pincelada(_)) {
            continue;
        }
        let mut passos = Vec::new();
        for ev in &tocavel.events[t.inicio..=t.fim] {
            let p = Passo::descodifica(&ev.payload).map_err(ErroDaMacro::Passo)?;
            if matches!(p, Passo::Ponteiro { .. }) {
                passos.push(p);
            }
        }
        out.push(Pinc {
            k: e.k,
            inicio: t.inicio,
            cor: e.cor,
            raio: e.tamanho_px,
            meio: e.meio,
            passos,
        });
    }
    Ok(out)
}

/// As pinceladas da trilha QUE TOCA, com o caminho na tela `w × h`.
///
/// # Errors
/// [`ErroDaMacro`].
pub fn regioes(track: &Track, w: u32, h: u32) -> Result<Vec<RegiaoDaPincelada>, ErroDaMacro> {
    let (fw, fh) = (f64::from(w), f64::from(h));
    Ok(pinceladas(track)?
        .into_iter()
        .map(|p| RegiaoDaPincelada {
            k: p.k,
            pontos: p
                .passos
                .iter()
                .filter_map(|p| match p {
                    Passo::Ponteiro { pos, .. } => Some(abs(*pos, fw, fh)),
                    _ => None,
                })
                .collect(),
            raio: p.raio,
            cor: p.cor,
            meio: p.meio,
        })
        .collect())
}

/// Os índices (`y · w + x`) dos píxeis cujo centro está a no máximo `raio` do caminho.
fn rasteriza(r: &RegiaoDaPincelada, w: u32, h: u32) -> Vec<u32> {
    let Some(&p0) = r.pontos.first() else {
        return Vec::new();
    };
    let (mut lo, mut hi) = (p0, p0);
    for p in &r.pontos {
        lo = [lo[0].min(p[0]), lo[1].min(p[1])];
        hi = [hi[0].max(p[0]), hi[1].max(p[1])];
    }
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "coordenadas de tela, recortadas à tela"
    )]
    let faixa = |a: f32, b: f32, n: u32| {
        let a = (a - r.raio).floor().max(0.0) as u32;
        let b = ((b + r.raio).ceil().max(0.0) as u32).min(n);
        a..b
    };
    let r2 = r.raio * r.raio;
    let mut out = Vec::new();
    for y in faixa(lo[1], hi[1], h) {
        for x in faixa(lo[0], hi[0], w) {
            #[expect(clippy::cast_precision_loss, reason = "coordenadas de tela")]
            let c = [x as f32 + 0.5, y as f32 + 0.5];
            let perto = r.pontos.len() == 1 && d2_ponto(c, p0) <= r2
                || r.pontos
                    .windows(2)
                    .any(|s| d2_segmento(c, s[0], s[1]) <= r2);
            if perto {
                out.push(y * w + x);
            }
        }
    }
    out
}

fn d2_ponto(a: [f32; 2], b: [f32; 2]) -> f32 {
    (a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)
}

fn d2_segmento(c: [f32; 2], a: [f32; 2], b: [f32; 2]) -> f32 {
    let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
    let l2 = dx * dx + dy * dy;
    if l2 == 0.0 {
        return d2_ponto(c, a);
    }
    let t = (((c[0] - a[0]) * dx + (c[1] - a[1]) * dy) / l2).clamp(0.0, 1.0); // CLAMP-OK: projecção no segmento
    d2_ponto(c, [a[0] + t * dx, a[1] + t * dy])
}

/// O critério de uma pincelada: a região, a parte dela que é NOVA na gravação e quanto dessa parte a
/// GRAVADA pinta.
#[derive(Clone, Debug, PartialEq)]
pub struct CriterioDaPincelada {
    pub regiao: RegiaoDaPincelada,
    /// Os píxeis da região que nenhuma pincelada anterior cobre (nunca vazio: a pincelada toda
    /// coberta pelas anteriores não tem critério).
    pub novos: Vec<u32>,
    /// Medida na reprodução ([`EnsinoDoPainter::mede`]); `None` = ainda por medir — a pincelada não
    /// se cumpre.
    pub referencia: Option<Referencia>,
    /// O texto que o autor escreveu para o balão nesta etapa.
    pub dica: Option<String>,
}

/// Quanto dos `novos` de uma pincelada a GRAVAÇÃO pinta, com ela e sem ela, no mesmo instante.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Referencia {
    /// A trilha que toca.
    pub gravada: f32,
    /// A trilha que toca SEM esta pincelada: o que a água das outras pinta sozinha.
    pub base: f32,
}

impl Referencia {
    /// O que a pincelada acrescenta à água.
    #[must_use]
    pub fn delta(self) -> f32 {
        self.gravada - self.base
    }
}

/// A medida da pincelada `k` (o endereço da gravação).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Medida {
    pub k: usize,
    pub referencia: Referencia,
}

/// O que o tutorial do Painter precisa de uma trilha: os critérios na tela da gravação e as camadas
/// do estado inicial. Calcula-se uma vez por versão da `Macro`, fora do quadro; as medidas chegam
/// depois, uma a uma ([`EnsinoDoPainter::com_medida`]).
#[derive(Clone)]
pub struct EnsinoDoPainter {
    /// Só as pinceladas com parte NOVA que a gravada pinta — a lei da W6a: etapa com Δ vazio não tem
    /// critério (não espera, não mostra balão).
    pub criterios: Vec<CriterioDaPincelada>,
    /// As camadas do estado inicial (id, RGBA) — a régua do «pintado».
    inicial: Vec<(u64, Vec<u8>)>,
    /// A tela da gravação: uma imagem de outro tamanho recusa-se ([`Preparo::OutroTamanho`]).
    pub w: u32,
    pub h: u32,
    /// Quantas medidas chegaram (a chave do progresso de quem avalia).
    chegadas: usize,
}

/// As camadas do aluno, cada uma com a MESMA camada do estado inicial (`None` = criada pelo aluno).
type Camadas<'a> = Vec<(&'a [u8], Option<&'a [u8]>)>;

impl EnsinoDoPainter {
    /// # Errors
    /// [`ErroDaMacro`].
    pub fn de(track: &Track) -> Result<Self, ErroDaMacro> {
        let t0 = inicial::tool_no_inicial(&track.initial).map_err(ErroDaMacro::Inicial)?;
        let (w, h) = t0.canvas_size();
        let inicial = t0
            .camadas_rgba()
            .into_iter()
            .map(|(id, px)| (id, px.to_vec()))
            .collect();
        let mut coberto = vec![false; w as usize * h as usize];
        let mut criterios = Vec::new();
        let gravadas = LeitorDoPainter.etapas(&track.events);
        let dica = |k: usize| {
            let ancora = gravadas.get(k)?.inicio;
            track
                .edicoes
                .iter()
                .find(|e| e.ancora == ancora)?
                .dica
                .clone()
        };
        for regiao in regioes(track, w, h)? {
            let dentro = rasteriza(&regiao, w, h);
            let novos: Vec<u32> = dentro
                .iter()
                .copied()
                .filter(|&i| !coberto[i as usize])
                .collect();
            for &i in &dentro {
                coberto[i as usize] = true;
            }
            if !novos.is_empty() {
                criterios.push(CriterioDaPincelada {
                    novos,
                    dica: dica(regiao.k),
                    regiao,
                    referencia: None,
                });
            }
        }
        Ok(Self {
            criterios,
            inicial,
            w,
            h,
            chegadas: 0,
        })
    }

    /// [`Self::de`] com as medidas já feitas (no fio de quem chama).
    ///
    /// # Errors
    /// [`ErroDaMacro`].
    pub fn medido(track: &Track) -> Result<Self, ErroDaMacro> {
        let mut e = Self::de(track)?;
        let mut ms = Vec::new();
        e.mede(track, &AtomicBool::new(false), |m| ms.push(m))?;
        for m in ms {
            e.com_medida(m);
        }
        Ok(e)
    }

    /// Chega a medida de uma pincelada: uma que não acrescenta à água mais que [`DELTA_MINIMO`] (a
    /// água das outras já lhe pinta a parte nova; uma borracha no papel em branco) deixa de ter
    /// critério — a lei do Δ vazio. `false` = não é de um critério por medir.
    pub fn com_medida(&mut self, m: Medida) -> bool {
        let Some(i) = self
            .criterios
            .iter()
            .position(|c| c.regiao.k == m.k && c.referencia.is_none())
        else {
            return false;
        };
        self.chegadas += 1;
        if m.referencia.delta() > DELTA_MINIMO {
            self.criterios[i].referencia = Some(m.referencia);
        } else {
            self.criterios.remove(i);
        }
        true
    }

    /// Alguma medida ainda não chegou.
    #[must_use]
    pub fn a_medir(&self) -> bool {
        self.criterios.iter().any(|c| c.referencia.is_none())
    }

    /// Quantas medidas chegaram.
    #[must_use]
    pub fn chegadas(&self) -> usize {
        self.chegadas
    }

    /// **Mede cada critério** de `track`, com a régua do aluno (o «pintado»), no instante antes do
    /// toque da pincelada SEGUINTE (ou no fim) — a água que corre depois do levantar conta, como conta
    /// para o aluno:
    ///
    /// - `gravada`: a trilha que toca, reproduzida do estado inicial;
    /// - `base`: a MESMA trilha com a pincelada apagada ([`etapas::apaga`]) e o tempo dela devolvido
    ///   à etapa seguinte ([`etapas::espera`]) — o toque seguinte cai no mesmo instante. Recomeça do
    ///   instantâneo da reprodução no 1.º evento em que as duas diferem (W5c).
    ///
    /// Uma porta recusada salta-se (o tutorial não a ensina). `chega` recebe cada medida pela ordem
    /// das pinceladas. `Ok(false)` = `parar` pediu-o a meio.
    ///
    /// # Errors
    /// [`ErroDaMacro`] — a reprodução divergiu.
    pub fn mede(
        &self,
        track: &Track,
        parar: &AtomicBool,
        mut chega: impl FnMut(Medida),
    ) -> Result<bool, ErroDaMacro> {
        let tocavel = track
            .tocavel(&LeitorDoPainter)
            .map_err(ErroDaMacro::Edicao)?;
        let ps = pinceladas_em(track, &tocavel)?;
        // (o evento antes do qual se mede, o critério), pela ordem dos eventos.
        let mut fronteiras = Vec::new();
        let mut bases = Vec::new();
        for (j, p) in ps.iter().enumerate() {
            let Some(c) = self.criterios.iter().position(|c| c.regiao.k == p.k) else {
                continue;
            };
            let fronteira = ps.get(j + 1).map_or(tocavel.events.len(), |q| q.inicio);
            fronteiras.push((fronteira, c));
            bases.push(SemAPincelada::de(track, &tocavel, p.k, c, fronteira)?);
        }
        bases.sort_by_key(|b| b.desde);
        let mut base = vec![None; self.criterios.len()];
        let mut toc = TocadorDoPainter::novo(
            &tocavel.initial,
            tocavel.events.iter().map(|e| e.payload.as_slice()),
        )?;
        let (mut f, mut b) = (fronteiras.iter().peekable(), bases.iter().peekable());
        for i in 0..=tocavel.events.len() {
            while let Some(s) = b.next_if(|s| s.desde == i) {
                match s.mede(self, track, &toc, parar)? {
                    Some(x) => base[s.c] = Some(x),
                    None => return Ok(false),
                }
            }
            while let Some(&(_, c)) = f.next_if(|&&(antes_de, _)| antes_de == i) {
                let crit = &self.criterios[c];
                chega(Medida {
                    k: crit.regiao.k,
                    referencia: Referencia {
                        gravada: self.fracao_pintada(toc.tool(), crit),
                        base: base[c].expect("a base mede-se antes da fronteira"),
                    },
                });
            }
            let Some(ev) = tocavel.events.get(i) else {
                break;
            };
            if parar.load(Ordering::Relaxed) {
                return Ok(false);
            }
            toca(&mut toc, &ev.payload)?;
        }
        Ok(true)
    }

    /// As camadas do aluno `t` ao lado das do inicial, emparelhadas UMA vez por avaliação; `None` numa
    /// tela de outro tamanho.
    fn camadas<'a>(&'a self, t: &'a PainterTool) -> Option<Camadas<'a>> {
        (t.canvas_size() == (self.w, self.h)).then(|| {
            t.camadas_rgba()
                .into_iter()
                .map(|(id, px)| {
                    let ini = self.inicial.iter().find(|(j, _)| *j == id);
                    (px, ini.map(|(_, v)| v.as_slice()))
                })
                .collect()
        })
    }

    /// A fracção dos píxeis NOVOS de `c` pintados nas `camadas`.
    fn fracao(camadas: &Camadas<'_>, c: &CriterioDaPincelada) -> f32 {
        let pintado = |i: u32| {
            let k = i as usize * 4;
            camadas.iter().any(|(px, ini)| match ini {
                Some(ini) => px[k..k + 4] != ini[k..k + 4],
                None => px[k + 3] > 0,
            })
        };
        #[expect(clippy::cast_precision_loss, reason = "contagens de píxeis")]
        let f = c.novos.iter().filter(|&&i| pintado(i)).count() as f32 / c.novos.len() as f32;
        f
    }

    /// A fracção dos píxeis NOVOS do critério `c` que o aluno `t` pintou (`0` numa tela de outro
    /// tamanho).
    #[must_use]
    pub fn fracao_pintada(&self, t: &PainterTool, c: &CriterioDaPincelada) -> f32 {
        self.camadas(t).map_or(0.0, |cs| Self::fracao(&cs, c))
    }

    /// O que o aluno `t` acrescentou à água em `c`, sobre o que a gravada acrescenta:
    /// `(fracção pintada − base) / (gravada − base)` (`None` = por medir).
    #[must_use]
    pub fn razao(&self, t: &PainterTool, c: &CriterioDaPincelada) -> Option<f32> {
        let r = c.referencia?;
        Some((self.fracao_pintada(t, c) - r.base) / r.delta())
    }
}

/// ⭐ **A fracção do que a GRAVADA acrescenta à água que cumpre uma pincelada** —
/// `(fracção pintada − base) / (gravada − base) ≥ isto`. MEDIDA (W6d, `03_plano_W6.md` §6.3, as sondas
/// deste módulo; mín./máx. das pinceladas, % normalizada):
///
/// | Trilha | Exacta = «Mostra-me» | Torta ½ r (mín.) | Fina (máx.) | Nada = pular e esperar (máx.) |
/// |---|---|---|---|---|
/// | Splash, Digital 512² | 100 | 73,1 | 43,8 | 0 |
/// | Splash, Watercolor 512² | 100 | 74,0 | 44,3 | 0 |
/// | Splash, Impasto 512² | 100 | 74,0 | 44,5 | 0 |
/// | Splash, Wet Paint 512² | 100–102,3 | 80,3 | 49,1 | 0,1 |
/// | Splash, Wet Paint 1024² | 100–101,1 | 81,9 | 57,5 | 0 |
/// | 30 pinceladas, Wet Paint 512² | 100–172,8 | — | — | 67,5 |
///
/// O meio do intervalo 67,5–73,1: 2,8 pontos de margem para cada lado.
pub const FRACAO_DA_GRAVADA_QUE_CUMPRE: f32 = 0.703;

/// **O que uma pincelada tem de acrescentar à água** (`gravada − base`) para ter critério — abaixo,
/// a lei do Δ vazio. MEDIDO (W6d): o menor Δ real nas sondas é 1,0 % (Watercolor, uma fatia fina; Wet
/// Paint 15,3 %); o aluno que pula a pincelada fica a ≤ 0,1 ponto da base (Splash, Wet Paint).
pub const DELTA_MINIMO: f32 = 0.005;

impl EnsinoDoPainter {
    /// A 1.ª pincelada por cumprir no tool `t` (índice em [`Self::criterios`]); `None` = todas. Pára
    /// nela: as seguintes não se avaliam (o balão mostra a 1.ª). As camadas leem-se uma vez. Uma
    /// referência por medir não se cumpre.
    #[must_use]
    pub fn por_cumprir(&self, t: &PainterTool) -> Option<usize> {
        let Some(cs) = self.camadas(t) else {
            return (!self.criterios.is_empty()).then_some(0);
        };
        self.criterios.iter().position(|c| {
            c.referencia.is_none_or(|r| {
                Self::fracao(&cs, c) - r.base < FRACAO_DA_GRAVADA_QUE_CUMPRE * r.delta()
            })
        })
    }

    /// O que falta antes de AVALIAR: o tool `t` (o Painter, `na_mao` = é a ferramenta activa) na
    /// imagem `alvo` (bits), da tela da gravação. `Ok` = o tool a avaliar.
    ///
    /// # Errors
    /// [`Preparo::AbreParaPintar`] ou [`Preparo::OutroTamanho`].
    pub fn aberto<'a>(
        &self,
        t: Option<&'a PainterTool>,
        na_mao: bool,
        alvo: u64,
    ) -> Result<&'a PainterTool, Preparo> {
        let t = t
            .filter(|t| na_mao && t.bound_document() == Some(alvo))
            .ok_or(Preparo::AbreParaPintar)?;
        if t.canvas_size() == (self.w, self.h) {
            Ok(t)
        } else {
            Err(Preparo::OutroTamanho {
                w: self.w,
                h: self.h,
            })
        }
    }

    /// O meio que falta para a pincelada `atual` (índice em [`Self::criterios`]): o dela, não o do
    /// início da gravação.
    #[must_use]
    pub fn meio_que_falta(&self, t: &PainterTool, atual: usize) -> Option<PaintMedia> {
        let m = self.criterios.get(atual)?.regiao.meio;
        (t.paint_media() != m).then_some(m)
    }

    /// O que falta antes da pincelada corrente: [`Self::aberto`] e depois [`Self::meio_que_falta`].
    #[must_use]
    pub fn preparo(&self, t: Option<&PainterTool>, na_mao: bool, alvo: u64) -> Option<Preparo> {
        match self.aberto(t, na_mao, alvo) {
            Err(p) => Some(p),
            Ok(t) => self
                .meio_que_falta(t, self.por_cumprir(t)?)
                .map(Preparo::Meio),
        }
    }
}

/// Um evento na reprodução de uma medida: uma porta recusada salta-se.
fn toca(toc: &mut TocadorDoPainter, carga: &[u8]) -> Result<(), ErroDaMacro> {
    match toc.aplica(carga) {
        Err(ErroDaMacro::PortaRecusada(_)) | Ok(()) => Ok(()),
        Err(x) => Err(x),
    }
}

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
struct SemAPincelada {
    k: usize,
    c: usize,
    /// O 1.º evento em que as duas trilhas diferem (o mesmo índice nas duas).
    desde: usize,
    /// Os eventos da trilha sem a pincelada `desde..` o instante da medida.
    eventos: Vec<Vec<u8>>,
    /// As respostas do worker da trilha sem a pincelada (a fita do instantâneo passa a ser esta).
    respostas: std::sync::Arc<[crate::tool::paint::fita::RespostaDoWorker]>,
}

impl SemAPincelada {
    fn de(
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
        let respostas =
            super::tocar::respostas_de(sem.events.iter().map(|e| e.payload.as_slice()))?;
        Ok(Self {
            k,
            c,
            desde,
            eventos: sem.events[desde..ate]
                .iter()
                .map(|e| e.payload.clone())
                .collect(),
            respostas: respostas.into(),
        })
    }

    /// A fracção dos `novos` que a trilha sem a pincelada pinta, a partir de `toc` (a reprodução da
    /// trilha que toca parada em [`Self::desde`]). `None` = `parar`.
    fn mede(
        &self,
        e: &EnsinoDoPainter,
        track: &Track,
        toc: &TocadorDoPainter,
        parar: &AtomicBool,
    ) -> Result<Option<f32>, ErroDaMacro> {
        let mut b = match toc.recomeca(&self.respostas) {
            Some(b) => b,
            // O instantâneo não é um estado da trilha sem ela (a fita já consumiu uma resposta da
            // pincelada): do estado inicial.
            None => {
                let sem = sem_a_pincelada(track, self.k)?;
                let mut b = TocadorDoPainter::novo(
                    &track.initial,
                    sem.events.iter().map(|e| e.payload.as_slice()),
                )?;
                for ev in &sem.events[..self.desde] {
                    if parar.load(Ordering::Relaxed) {
                        return Ok(None);
                    }
                    toca(&mut b, &ev.payload)?;
                }
                b
            }
        };
        for ev in &self.eventos {
            if parar.load(Ordering::Relaxed) {
                return Ok(None);
            }
            toca(&mut b, ev)?;
        }
        Ok(Some(e.fracao_pintada(b.tool(), &e.criterios[self.c])))
    }
}

/// O que falta antes das pinceladas.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Preparo {
    AbreParaPintar,
    /// A imagem do aluno não é do tamanho da gravação (`w × h`): o tutorial não a avalia.
    OutroTamanho {
        w: u32,
        h: u32,
    },
    Meio(PaintMedia),
}

/// **«Recomeçar»**: o documento do tool vivo `t` volta ao estado inicial da trilha — um passo de
/// undo. A água em curso assenta antes (é a tinta do aluno, e o passo desfaz-se com ela).
///
/// # Errors
/// [`ErroDaMacro::Inicial`]; uma tela de outro tamanho que a gravada recusa-se.
pub fn recomeca(t: &mut PainterTool, track: &Track) -> Result<(), ErroDaMacro> {
    let t0 = inicial::tool_no_inicial(&track.initial).map_err(ErroDaMacro::Inicial)?;
    if t0.canvas_size() != t.canvas_size() {
        return Err(ErroDaMacro::PortaRecusada("recomeca".into()));
    }
    // Uma macro a gravar não reproduz o «Recomeçar» (não é uma porta do tool): a reprodução pára aqui.
    t.macro_recusa("tutorial_recomeca");
    t.recomeca_com(&t0);
    Ok(())
}

/// **«Mostra-me» da pincelada `k`** no tool vivo `t`: o caminho gravado pelas portas do tool, com a
/// cor e o tamanho dela; os do aluno voltam no fim. Um passo de undo (o do Down..Up).
///
/// # Errors
/// [`ErroDaMacro`]; a etapa `k` não é uma pincelada.
pub fn mostra(t: &mut PainterTool, track: &Track, k: usize) -> Result<(), ErroDaMacro> {
    let Some(p) = pinceladas(track)?.into_iter().find(|p| p.k == k) else {
        return Err(ErroDaMacro::PortaRecusada(format!("etapa {k}")));
    };
    let antes = (t.brush_color_srgb8(), t.brush_size_px());
    t.set_brush_color_srgb8(p.cor);
    t.set_brush_size_px(p.raio);
    let r = p
        .passos
        .into_iter()
        .try_for_each(|p| TocadorDoPainter::aplica_ao_tool(t, p).map(drop));
    t.set_brush_color_srgb8(antes.0);
    t.set_brush_size_px(antes.1);
    r
}

#[cfg(test)]
#[path = "tutorial_sonda_tests.rs"]
mod sonda;
#[cfg(test)]
#[path = "tutorial_tests.rs"]
mod tests;
