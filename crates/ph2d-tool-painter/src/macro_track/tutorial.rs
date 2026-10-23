//! **O tutorial «fazer» do Painter** (W6b, `docs/Macros/03_plano_W6.md` §2 e §6): cada pincelada
//! gravada é uma REGIÃO da tela — o caminho do pincel com o raio dele — e cumpre-se quando o aluno
//! pintou ali. A lei é a da W6a: o progresso é função do ESTADO (as camadas do aluno contra as do
//! estado inicial), por isso o Ctrl+Z volta o passo sem gancho nenhum.
//!
//! - **A região conta-se pela parte NOVA**: os píxeis que nenhuma pincelada anterior da gravação
//!   cobre. Uma pincelada por cima de outra cumpria-se com a tinta da anterior.
//! - **«Pintado»** = algum píxel de alguma camada difere do estado inicial (uma camada que o aluno
//!   criou conta pela tinta que tem).
//! - **A régua é a GRAVADA** (W6c) **sem a água** (W6d) **na MESMA idade** (W7): cumpre-se quando o
//!   aluno acrescenta à água [`FRACAO_DA_GRAVADA_QUE_CUMPRE`] do que a pincelada gravada acrescenta da
//!   parte nova — a base é a gravação SEM a pincelada (a aguada da anterior pinta sozinha uma fatia
//!   nova); as duas são curvas, lidas na idade do aluno desde o último `Up` ([`idade`]).
//! - **«Mostra-me»** = o caminho gravado pelas portas do tool vivo, com a cor e o tamanho da
//!   pincelada (os do aluno voltam depois): um passo de undo.
//! - **O Fill** (W13, [`fill`]): o ColorDrop é uma etapa cuja região é o que ele pintou na reprodução,
//!   cumprida com a cor dele; uma pincelada sobre um Fill conta onde difere da cor dele.

use super::etapas::{self, LeitorDoPainter, TipoDaEtapa};
use super::inicial;
use super::passo::{Passo, abs};
use super::tocar::{ErroDaMacro, TocadorDoPainter};
use crate::PaintMedia;
use crate::tool::PainterTool;
use ph2d_macro::{LeitorDaFamilia, Track};
use std::sync::atomic::AtomicBool;

#[path = "tutorial_base.rs"]
mod base;
#[path = "tutorial_curva.rs"]
mod curva;
#[path = "tutorial_fill.rs"]
pub mod fill;
pub use curva::{Curva, Referencia};
#[path = "tutorial_mede.rs"]
mod mede;

/// O que uma etapa do tutorial pede.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TipoDoCriterio {
    /// Tinta na área, de qualquer cor.
    Pincelada,
    /// O ColorDrop do C&F: a região na cor dele ([`fill::parecida`]).
    Preenchimento,
}

/// A pincelada `k` (o endereço da GRAVAÇÃO) numa tela `w × h`.
#[derive(Clone, Debug, PartialEq)]
pub struct RegiaoDaPincelada {
    pub k: usize,
    pub tipo: TipoDoCriterio,
    /// O caminho do pincel, em píxeis da tela; num Fill, o sítio onde a cor caiu.
    pub pontos: Vec<[f32; 2]>,
    pub raio: f32,
    pub cor: [u8; 3],
    /// O meio em vigor no toque dela (a gravação pode trocá-lo entre pinceladas).
    pub meio: PaintMedia,
}

/// Uma etapa da trilha QUE TOCA: o endereço, os eventos dela, o pincel no toque e os passos do gesto
/// (o ponteiro; num Fill, também as portas do C&F e o modal).
struct Pinc {
    k: usize,
    tipo: TipoDoCriterio,
    /// O 1.º e o último evento da etapa na trilha que toca.
    inicio: usize,
    fim: usize,
    cor: [u8; 3],
    raio: f32,
    meio: PaintMedia,
    limiar: f32,
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
    let mut out = Vec::new();
    for e in etapas::tocadas(track, tocavel)? {
        let tipo = match e.tipo {
            TipoDaEtapa::Pincelada(_) => TipoDoCriterio::Pincelada,
            TipoDaEtapa::Preenchimento(_) => TipoDoCriterio::Preenchimento,
            TipoDaEtapa::Recusada(_) => continue,
        };
        let t = e.tocada;
        let mut passos = Vec::new();
        // Só os passos da faixa dela: uma deslocada (W16) toca entrelaçada com outra.
        for ev in tocavel.events[t.inicio..=t.fim]
            .iter()
            .filter(|ev| super::passo::faixa_de(&ev.payload) == e.faixa)
        {
            let p = Passo::descodifica(&ev.payload).map_err(ErroDaMacro::Passo)?;
            let entra = match tipo {
                TipoDoCriterio::Pincelada => matches!(p, Passo::Ponteiro { .. }),
                TipoDoCriterio::Preenchimento => fill::e_do_colordrop(&p),
            };
            if entra {
                passos.push(p);
            }
        }
        let (cor, raio, meio, limiar) = e
            .no_toque
            .map_or(([0; 3], 0.0, PaintMedia::default(), 0.0), |p| {
                (p.cor, p.tamanho_px, p.meio, p.limiar)
            });
        out.push(Pinc {
            k: e.k,
            tipo,
            inicio: t.inicio,
            fim: t.fim,
            cor,
            raio,
            meio,
            limiar,
            passos,
        });
    }
    Ok(out)
}

/// As etapas da trilha QUE TOCA, com o caminho na tela `w × h`.
///
/// # Errors
/// [`ErroDaMacro`].
pub fn regioes(track: &Track, w: u32, h: u32) -> Result<Vec<RegiaoDaPincelada>, ErroDaMacro> {
    Ok(pinceladas(track)?
        .iter()
        .map(|p| regiao_de(p, w, h))
        .collect())
}

/// A região de `p` numa tela `w × h`: o caminho; num Fill, o último sítio do ponteiro (onde a cor
/// caiu), sem raio.
fn regiao_de(p: &Pinc, w: u32, h: u32) -> RegiaoDaPincelada {
    let (fw, fh) = (f64::from(w), f64::from(h));
    let mut pontos: Vec<[f32; 2]> = p
        .passos
        .iter()
        .filter_map(|p| match p {
            Passo::Ponteiro { pos, .. } => Some(abs(*pos, fw, fh)),
            _ => None,
        })
        .collect();
    let raio = match p.tipo {
        TipoDoCriterio::Pincelada => p.raio,
        TipoDoCriterio::Preenchimento => {
            pontos = pontos.last().copied().into_iter().collect();
            0.0
        }
    };
    RegiaoDaPincelada {
        k: p.k,
        tipo: p.tipo,
        pontos,
        raio,
        cor: p.cor,
        meio: p.meio,
    }
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
    /// Os píxeis da região que ela julga (nunca vazio: sem nenhum, não tem critério) — numa pincelada,
    /// os que nenhuma anterior cobre e nenhum Fill seguinte repinta; num Fill, os que nada seguinte
    /// repinta.
    pub novos: Vec<u32>,
    /// Numa pincelada, a cor do Fill anterior sob cada um dos `novos` (`None` = o estado inicial): ali
    /// conta a tinta que não é a dele. Vazio num Fill.
    pub fundos: Vec<Option<[u8; 3]>>,
    /// Medida na reprodução ([`EnsinoDoPainter::mede`]); `None` = ainda por medir — a pincelada não
    /// se cumpre.
    pub referencia: Option<Referencia>,
    /// O texto que o autor escreveu para o balão nesta etapa.
    pub dica: Option<String>,
}

/// A idade de `t` em quadros (60 Hz) desde o último `Up` na tela; `None` = nenhum.
#[must_use]
pub fn idade_de(t: &PainterTool) -> Option<u32> {
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "segundos ≥ 0, saturado"
    )]
    t.segundos_desde_o_ultimo_up()
        .map(|s| (s * 60.0).round().min(f64::from(u32::MAX)) as u32)
}

/// A idade do aluno `t` ([`idade_de`]); sem `Up` nenhum, depois de toda curva.
#[must_use]
pub fn idade(t: &PainterTool) -> u32 {
    idade_de(t).unwrap_or(u32::MAX)
}

/// A medida da pincelada `k` (o endereço da gravação).
#[derive(Clone, Debug, PartialEq)]
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
    /// Um Fill pede a reprodução até ao fecho dele ([`fill::regioes_dos_fills`]).
    ///
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
        let tocavel = track
            .tocavel(&LeitorDoPainter)
            .map_err(ErroDaMacro::Edicao)?;
        let ps = pinceladas_em(track, &tocavel)?;
        let areas: Vec<Vec<u32>> = ps
            .iter()
            .zip(fill::regioes_dos_fills(&tocavel, &ps, w, h)?)
            .map(|(p, f)| f.unwrap_or_else(|| rasteriza(&regiao_de(p, w, h), w, h)))
            .collect();
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
        let n = w as usize * h as usize;
        // Depois: um Fill julga só onde nada o repinta; uma pincelada, só onde nenhum Fill a repinta.
        let (mut depois, mut depois_fill) = (vec![false; n], vec![false; n]);
        let mut julgados = vec![Vec::new(); ps.len()];
        for (j, p) in ps.iter().enumerate().rev() {
            let fora = match p.tipo {
                TipoDoCriterio::Pincelada => &depois_fill,
                TipoDoCriterio::Preenchimento => &depois,
            };
            julgados[j] = areas[j]
                .iter()
                .copied()
                .filter(|&i| !fora[i as usize])
                .collect();
            for &i in &areas[j] {
                depois[i as usize] = true;
                depois_fill[i as usize] |= p.tipo == TipoDoCriterio::Preenchimento;
            }
        }
        // Antes: a parte NOVA de uma pincelada e o Fill debaixo dela.
        let mut coberto = vec![false; n];
        let mut fundo: Vec<Option<[u8; 3]>> = vec![None; n];
        let mut criterios = Vec::new();
        for ((p, area), julgado) in ps.iter().zip(&areas).zip(julgados) {
            let (novos, fundos) = match p.tipo {
                TipoDoCriterio::Pincelada => {
                    let novos: Vec<u32> = julgado
                        .into_iter()
                        .filter(|&i| !coberto[i as usize])
                        .collect();
                    let fundos = novos.iter().map(|&i| fundo[i as usize]).collect();
                    for &i in area {
                        coberto[i as usize] = true;
                    }
                    (novos, fundos)
                }
                TipoDoCriterio::Preenchimento => {
                    for &i in area {
                        coberto[i as usize] = false;
                        fundo[i as usize] = Some(p.cor);
                    }
                    (julgado, Vec::new())
                }
            };
            if !novos.is_empty() {
                criterios.push(CriterioDaPincelada {
                    novos,
                    fundos,
                    dica: dica(p.k),
                    regiao: regiao_de(p, w, h),
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

    /// A fracção dos píxeis NOVOS de `c` feitos nas `camadas`: alguma camada mudou ali — numa
    /// pincelada, para outra cor que a do Fill debaixo; num Fill, para a cor dele.
    fn fracao(camadas: &Camadas<'_>, c: &CriterioDaPincelada) -> f32 {
        let feito = |j: usize, i: u32| {
            let k = i as usize * 4;
            camadas.iter().any(|(px, ini)| {
                let mudou = match ini {
                    Some(ini) => px[k..k + 4] != ini[k..k + 4],
                    None => px[k + 3] > 0,
                };
                mudou
                    && match c.regiao.tipo {
                        TipoDoCriterio::Pincelada => c
                            .fundos
                            .get(j)
                            .copied()
                            .flatten()
                            .is_none_or(|f| !fill::parecida(&px[k..k + 4], f)),
                        TipoDoCriterio::Preenchimento => {
                            fill::parecida(&px[k..k + 4], c.regiao.cor)
                        }
                    }
            })
        };
        let n = c
            .novos
            .iter()
            .enumerate()
            .filter(|&(j, &i)| feito(j, i))
            .count();
        #[expect(clippy::cast_precision_loss, reason = "contagens de píxeis")]
        let f = n as f32 / c.novos.len() as f32;
        f
    }

    /// A fracção dos píxeis NOVOS do critério `c` que o aluno `t` pintou (`0` numa tela de outro
    /// tamanho).
    #[must_use]
    pub fn fracao_pintada(&self, t: &PainterTool, c: &CriterioDaPincelada) -> f32 {
        self.camadas(t).map_or(0.0, |cs| Self::fracao(&cs, c))
    }

    /// O que o aluno `t` acrescentou à água em `c`, sobre o que a gravada acrescenta, na idade dele
    /// ([`idade`]): `(fracção pintada − base(τ)) / (gravada(τ) − base(τ))` (`None` = por medir).
    #[must_use]
    pub fn razao(&self, t: &PainterTool, c: &CriterioDaPincelada) -> Option<f32> {
        let (b, d) = c.referencia.as_ref()?.em(idade(t));
        Some((self.fracao_pintada(t, c) - b) / d)
    }

    /// A idade de `t` na chave de quem guarda a avaliação: depois da última idade amostrada nenhuma
    /// curva muda, e a chave também não.
    #[must_use]
    pub fn idade_na_chave(&self, t: &PainterTool) -> u32 {
        let ate = self
            .criterios
            .iter()
            .filter_map(|c| c.referencia.as_ref().map(Referencia::ate))
            .max()
            .unwrap_or(0);
        idade(t).min(ate)
    }
}

/// ⭐ **A fracção do que a GRAVADA acrescenta à água que cumpre uma pincelada** —
/// `(fracção pintada − base(τ)) / (gravada(τ) − base(τ)) ≥ isto`, na idade τ do aluno. MEDIDA (W7,
/// `04_plano_W7.md` item 2, as sondas deste módulo; smoke; % normalizada, o aluno avaliado na idade dele
/// depois de a água assentar; o Nada também em cada idade):
///
/// | Trilha | Exacta = «Mostra-me» | Torta ½ r (mín.) | Fina (máx.) | Nada (máx.): nas curvas / depois |
/// |---|---|---|---|---|
/// | Splash, Digital / Watercolor / Impasto 512² | 100 | 73,1 / 74,0 / 74,0 | 43,8 / 44,3 / 44,5 | 0 / 0 |
/// | Splash, Wet Paint 512² | 100 | 74,6 | 55,6 | 0 / 8,3 |
/// | Splash, Wet Paint 1024² | 100 | 80,2 | 60,0 | 0 / 4,9 |
/// | 30 pinceladas, os quatro meios 512² | 100 (no `Up`, Wet Paint) | — | — | 0 / 7,3 (Wet Paint) |
///
/// O meio do intervalo 60,0 (a fina, Wet Paint 1024²) – 73,1 (a torta, Digital): 6,55 pontos de margem
/// para cada lado.
pub const FRACAO_DA_GRAVADA_QUE_CUMPRE: f32 = 0.6655;

/// **O que uma pincelada tem de acrescentar à água** (`gravada − base`, no fim das curvas) para ter
/// critério — abaixo, a lei do Δ vazio; e o piso do acréscimo em cada idade (a guarda da divisão).
/// MEDIDO (W6d; W7 com a água assentada): o menor Δ real nas sondas é 1,0 % (Watercolor, uma fatia
/// fina; Wet Paint 19,7 %).
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
        let q = idade(t);
        self.criterios.iter().position(|c| {
            c.referencia.as_ref().is_none_or(|r| {
                let (b, d) = r.em(q);
                Self::fracao(&cs, c) - b < FRACAO_DA_GRAVADA_QUE_CUMPRE * d
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

    /// A cor que falta no pincel para a etapa `atual` (índice em [`Self::criterios`]): a de um Fill,
    /// quando a do pincel não é parecida ([`fill::rgb_parecida`]).
    #[must_use]
    pub fn cor_que_falta(&self, t: &PainterTool, atual: usize) -> Option<[u8; 3]> {
        let c = self.criterios.get(atual)?;
        (c.regiao.tipo == TipoDoCriterio::Preenchimento
            && !fill::rgb_parecida(t.brush_color_srgb8(), c.regiao.cor))
        .then_some(c.regiao.cor)
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

/// **«Mostra-me» da etapa `k`** no tool vivo `t`: o caminho gravado pelas portas do tool, com a cor e
/// o tamanho dela; os do aluno voltam no fim. Um passo de undo (o do Down..Up). Um Fill:
/// [`fill::mostra`].
///
/// # Errors
/// [`ErroDaMacro`]; a etapa `k` não é uma pincelada nem um Fill.
pub fn mostra(t: &mut PainterTool, track: &Track, k: usize) -> Result<(), ErroDaMacro> {
    let Some(p) = pinceladas(track)?.into_iter().find(|p| p.k == k) else {
        return Err(ErroDaMacro::PortaRecusada(format!("etapa {k}")));
    };
    if p.tipo == TipoDoCriterio::Preenchimento {
        return fill::mostra(t, p);
    }
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
