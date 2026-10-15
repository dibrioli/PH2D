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
    /// O meio em vigor no toque dela (a gravação pode trocá-lo entre pinceladas).
    pub meio: PaintMedia,
}

/// Uma pincelada da trilha QUE TOCA: o endereço, o pincel no toque e os passos do ponteiro.
struct Pinc {
    k: usize,
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

/// O critério de uma pincelada: a região e a parte dela que é NOVA na gravação.
#[derive(Clone, Debug, PartialEq)]
pub struct CriterioDaPincelada {
    pub regiao: RegiaoDaPincelada,
    /// Os píxeis da região que nenhuma pincelada anterior cobre (nunca vazio: a pincelada toda
    /// coberta pelas anteriores não tem critério).
    pub novos: Vec<u32>,
}

/// O que o tutorial do Painter precisa de uma trilha: os critérios na tela da gravação e as camadas
/// do estado inicial. Calcula-se uma vez por versão da `Macro`.
pub struct EnsinoDoPainter {
    /// Só as pinceladas com parte NOVA — a lei da W6a: etapa com Δ vazio não tem critério (não espera,
    /// não mostra balão).
    pub criterios: Vec<CriterioDaPincelada>,
    /// As camadas do estado inicial (id, RGBA) — a régua do «pintado».
    inicial: Vec<(u64, Vec<u8>)>,
    /// A tela da gravação: uma imagem de outro tamanho recusa-se ([`Preparo::OutroTamanho`]).
    pub w: u32,
    pub h: u32,
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
                criterios.push(CriterioDaPincelada { novos, regiao });
            }
        }
        Ok(Self {
            criterios,
            inicial,
            w,
            h,
        })
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
}

/// ⭐ **A fracção da parte nova que cumpre uma pincelada** — MEDIDA (W6b, `03_plano_W6.md` §6.1,
/// a sonda deste módulo, o Splash em Wet Paint a 512² e 1024²): a pincelada certa e o «Mostra-me»
/// pintam ≥ 93 %, torta meio raio ≥ 75 %; com o pincel fino 46–52 %, deslocada dois raios ≤ 27 %, nada
/// ≤ 14,6 % (a água das anteriores). 70 % aceita a mão torta e pede ao pincel fino que acabe de encher.
/// ⚠️ Nos outros três meios a mão torta meio raio cai a 61–73 % (a 2.ª sonda, §6.1): ABERTO.
pub const FRACAO_QUE_CUMPRE: f32 = 0.70;

impl EnsinoDoPainter {
    /// A 1.ª pincelada por cumprir no tool `t` (índice em [`Self::criterios`]); `None` = todas. Pára
    /// nela: as seguintes não se avaliam (o balão mostra a 1.ª). As camadas leem-se uma vez.
    #[must_use]
    pub fn por_cumprir(&self, t: &PainterTool) -> Option<usize> {
        let Some(cs) = self.camadas(t) else {
            return (!self.criterios.is_empty()).then_some(0);
        };
        self.criterios
            .iter()
            .position(|c| Self::fracao(&cs, c) < FRACAO_QUE_CUMPRE)
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
