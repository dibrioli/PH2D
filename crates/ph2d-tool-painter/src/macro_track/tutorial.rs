//! **O tutorial «fazer» do Painter** (W6b, `docs/Macros/03_plano_W6.md` §2 e §6): cada pincelada
//! gravada é uma REGIÃO da tela — o caminho do pincel com o raio dele — e cumpre-se quando o aluno
//! pintou ali. A lei é a da W6a: o progresso é função do ESTADO (as camadas do aluno contra as do
//! estado inicial), por isso o Ctrl+Z volta o passo sem gancho nenhum.
//!
//! - **A região conta-se pela parte NOVA**: os píxeis que nenhuma pincelada anterior da gravação
//!   cobre. Uma pincelada por cima de outra cumpria-se com a tinta da anterior.
//! - **«Pintado»** = algum píxel de alguma camada difere do estado inicial (uma camada que o aluno
//!   criou conta pela tinta que tem).
//! - **«Mostra-me»** = o caminho gravado pelas portas do tool vivo, com a cor e o tamanho da
//!   pincelada (os do aluno voltam depois): um passo de undo.

use super::etapas::{self, LeitorDoPainter, TipoDaEtapa};
use super::inicial;
use super::passo::{Passo, abs};
use super::tocar::{ErroDaMacro, TocadorDoPainter};
use crate::PaintMedia;
use crate::tool::PainterTool;
use ph2d_macro::{LeitorDaFamilia, Track};

/// A pincelada `k` (o endereço da GRAVAÇÃO) numa tela `w × h`.
#[derive(Clone, Debug, PartialEq)]
pub struct RegiaoDaPincelada {
    pub k: usize,
    /// O caminho do pincel, em píxeis da tela.
    pub pontos: Vec<[f32; 2]>,
    pub raio: f32,
    pub cor: [u8; 3],
}

/// Cada pincelada da trilha QUE TOCA (com as edições): `k`, cor, tamanho e os passos do ponteiro.
fn pinceladas(track: &Track) -> Result<Vec<(usize, [u8; 3], f32, Vec<Passo>)>, ErroDaMacro> {
    let tocavel = track
        .tocavel(&LeitorDoPainter)
        .map_err(ErroDaMacro::Edicao)?;
    let tocadas = LeitorDoPainter.etapas(&tocavel.events);
    let mut out = Vec::new();
    // `etapas` lê uma tocada por etapa viva, pela ordem: as duas listas andam juntas.
    for (e, t) in etapas::etapas(track)?.into_iter().zip(tocadas) {
        if !matches!(e.tipo, TipoDaEtapa::Pincelada(_)) {
            continue;
        }
        let mut ps = Vec::new();
        for ev in &tocavel.events[t.inicio..=t.fim] {
            let p = Passo::descodifica(&ev.payload).map_err(ErroDaMacro::Passo)?;
            if matches!(p, Passo::Ponteiro { .. }) {
                ps.push(p);
            }
        }
        out.push((e.k, e.cor, e.tamanho_px, ps));
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
        .map(|(k, cor, raio, ps)| RegiaoDaPincelada {
            k,
            pontos: ps
                .iter()
                .filter_map(|p| match p {
                    Passo::Ponteiro { pos, .. } => Some(abs(*pos, fw, fh)),
                    _ => None,
                })
                .collect(),
            raio,
            cor,
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
        clippy::cast_precision_loss,
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

/// O critério de uma pincelada: a região e a parte dela que é NOVA na gravação.
#[derive(Clone, Debug, PartialEq)]
pub struct CriterioDaPincelada {
    pub regiao: RegiaoDaPincelada,
    /// Os píxeis da região que nenhuma pincelada anterior cobre; toda a região se não sobra nenhum.
    pub novos: Vec<u32>,
}

/// O que o tutorial do Painter precisa de uma trilha numa tela `w × h`: os critérios, as camadas do
/// estado inicial e o meio. Calcula-se uma vez por versão da `Macro` e tamanho da tela.
pub struct EnsinoDoPainter {
    pub criterios: Vec<CriterioDaPincelada>,
    /// As camadas do estado inicial (id, RGBA) — a régua do «pintado».
    inicial: Vec<(u64, Vec<u8>)>,
    pub meio: PaintMedia,
    pub w: u32,
    pub h: u32,
}

impl EnsinoDoPainter {
    /// `tela` = o tamanho da imagem do aluno; `None` = o da gravação.
    ///
    /// # Errors
    /// [`ErroDaMacro`].
    pub fn de(track: &Track, tela: Option<(u32, u32)>) -> Result<Self, ErroDaMacro> {
        let t0 = inicial::tool_no_inicial(&track.initial).map_err(ErroDaMacro::Inicial)?;
        let (w0, h0) = t0.canvas_size();
        let (w, h) = tela.unwrap_or((w0, h0));
        let inicial = t0
            .camadas_rgba()
            .into_iter()
            .map(|(id, px)| (id, reamostra(px, (w0, h0), (w, h))))
            .collect();
        let mut coberto = vec![false; w as usize * h as usize];
        let mut criterios = Vec::new();
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
            criterios.push(CriterioDaPincelada {
                novos: if novos.is_empty() { dentro } else { novos },
                regiao,
            });
        }
        Ok(Self {
            criterios,
            inicial,
            meio: t0.paint_media(),
            w,
            h,
        })
    }

    /// A fracção dos píxeis NOVOS do critério `c` que o aluno `t` pintou (`0` numa tela de outro
    /// tamanho).
    #[must_use]
    pub fn fracao_pintada(&self, t: &PainterTool, c: &CriterioDaPincelada) -> f32 {
        if t.canvas_size() != (self.w, self.h) || c.novos.is_empty() {
            return 0.0;
        }
        let aluno = t.camadas_rgba();
        let pintado = |i: u32| {
            let k = i as usize * 4;
            aluno.iter().any(
                |(id, px)| match self.inicial.iter().find(|(j, _)| j == id) {
                    Some((_, ini)) => px[k..k + 4] != ini[k..k + 4],
                    None => px[k + 3] > 0,
                },
            )
        };
        #[expect(clippy::cast_precision_loss, reason = "contagens de píxeis")]
        let f = c.novos.iter().filter(|&&i| pintado(i)).count() as f32 / c.novos.len() as f32;
        f
    }
}

/// ⭐ **A fracção da parte nova que cumpre uma pincelada** — MEDIDA (W6b, `03_plano_W6.md` §6.1,
/// a sonda deste módulo, o Splash a 512² e 1024²): a pincelada certa e o «Mostra-me» pintam ≥ 93 %,
/// torta meio raio ≥ 75 %; com o pincel fino 46–52 %, deslocada dois raios ≤ 27 %, nada ≤ 14,6 % (a
/// água das anteriores). 70 % aceita a mão torta e pede ao pincel fino que acabe de encher.
pub const FRACAO_QUE_CUMPRE: f32 = 0.70;

impl EnsinoDoPainter {
    /// A 1.ª pincelada por cumprir no tool `t` (índice em [`Self::criterios`]); `None` = todas. Pára
    /// nela: as seguintes não se avaliam (o balão mostra a 1.ª).
    #[must_use]
    pub fn por_cumprir(&self, t: &PainterTool) -> Option<usize> {
        self.criterios
            .iter()
            .position(|c| self.fracao_pintada(t, c) < FRACAO_QUE_CUMPRE)
    }

    /// O que falta antes de pintar: o tool `t` (o Painter, `na_mao` = é a ferramenta activa) na
    /// imagem `alvo` (bits) e no meio da gravação.
    #[must_use]
    pub fn preparo(&self, t: Option<&PainterTool>, na_mao: bool, alvo: u64) -> Option<Preparo> {
        let Some(t) = t.filter(|t| na_mao && t.bound_document() == Some(alvo)) else {
            return Some(Preparo::AbreParaPintar);
        };
        (t.paint_media() != self.meio).then_some(Preparo::Meio(self.meio))
    }
}

/// O que falta antes das pinceladas.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Preparo {
    AbreParaPintar,
    Meio(PaintMedia),
}

/// A chave i18n do nome de um meio (a do seletor do pincel).
#[must_use]
pub fn chave_do_meio(m: PaintMedia) -> &'static str {
    match m {
        PaintMedia::Digital => "tool.painter.media.digital",
        PaintMedia::Watercolor => "tool.painter.media.watercolor",
        PaintMedia::Impasto => "tool.painter.media.impasto",
        PaintMedia::WetPaint => "tool.painter.media.wet_paint",
    }
}

/// `px` (`de`) no tamanho `para`, pelo vizinho mais perto (a régua do «pintado» numa imagem do aluno
/// de outro tamanho que a gravada; do mesmo tamanho é a cópia).
fn reamostra(px: &[u8], de: (u32, u32), para: (u32, u32)) -> Vec<u8> {
    if de == para {
        return px.to_vec();
    }
    let (w, h) = (para.0 as usize, para.1 as usize);
    let (w0, h0) = (de.0.max(1) as usize, de.1.max(1) as usize);
    let mut out = vec![0u8; w * h * 4];
    for y in 0..h {
        let y0 = y * h0 / h.max(1);
        for x in 0..w {
            let x0 = x * w0 / w.max(1);
            let (a, b) = ((y * w + x) * 4, (y0 * w0 + x0) * 4);
            if let Some(src) = px.get(b..b + 4) {
                out[a..a + 4].copy_from_slice(src);
            }
        }
    }
    out
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
    t.recomeca_com(&t0);
    Ok(())
}

/// **«Mostra-me» da pincelada `k`** no tool vivo `t`: o caminho gravado pelas portas do tool, com a
/// cor e o tamanho dela; os do aluno voltam no fim. Um passo de undo (o do Down..Up).
///
/// # Errors
/// [`ErroDaMacro`]; a etapa `k` não é uma pincelada.
pub fn mostra(t: &mut PainterTool, track: &Track, k: usize) -> Result<(), ErroDaMacro> {
    let Some((_, cor, raio, ps)) = pinceladas(track)?.into_iter().find(|p| p.0 == k) else {
        return Err(ErroDaMacro::PortaRecusada(format!("etapa {k}")));
    };
    let antes = (t.brush_color_srgb8(), t.brush_size_px());
    t.set_brush_color_srgb8(cor);
    t.set_brush_size_px(raio);
    let r = ps
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
