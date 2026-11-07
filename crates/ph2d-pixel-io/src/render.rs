//! **A pergunta de todo exportador: o que sai em cada quadro?** PNG, GIF, APNG, WebP e a folha pedem
//! aqui; nenhum compõe por conta própria.

use ph2d_pixel_core::{FrameId, Layer, LayerId, SliceId, Sprite, compose_picked};

use crate::sheet::fatias;
use crate::sheet::recorte;

/// Que camadas entram.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Camadas {
    /// As visíveis, menos as de referência.
    #[default]
    Visiveis,
    /// Todas, até as escondidas (`--all-layers`), menos as de referência.
    Todas,
    /// Só esta (ou este grupo inteiro), mesmo escondida (`--layer`, medido).
    Uma(LayerId),
}

/// Que quadros entram.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Quadros {
    #[default]
    Todos,
    /// O índice da tag em `Sprite::tags`.
    Tag(usize),
    /// Índices da linha do tempo, inclusive nos dois lados.
    Intervalo { de: usize, ate: usize },
}

/// A ordem de reprodução. O vai-e-volta não repete as pontas (a mesma lei do `Play`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Direcao {
    #[default]
    Forward,
    Reverse,
    PingPong,
    PingPongReverse,
}

/// A área que sai (`Area:` do Export File).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Area {
    #[default]
    Canvas,
    /// Só esta slice; o quadro em que ela não aparece não sai.
    Slice(SliceId),
}

/// O pedido de um export.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Pedido {
    pub camadas: Camadas,
    pub quadros: Quadros,
    pub direcao: Direcao,
    /// 1 = sem escala; 0 vale 1.
    pub escala: f64,
    pub area: Area,
}

/// Um quadro pronto.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuadroSaida {
    /// A posição na linha do tempo do documento.
    pub indice: usize,
    pub frame: FrameId,
    pub duration_ms: u32,
    pub w: u32,
    pub h: u32,
    /// RGBA de alfa reto.
    pub rgba: Vec<u8>,
}

/// `id` ou um dos seus pais é `alvo`?
fn dentro_de(s: &Sprite, l: &Layer, alvo: LayerId) -> bool {
    let mut at = Some(l.id);
    while let Some(id) = at {
        if id == alvo {
            return true;
        }
        at = s.layer(id).and_then(|x| x.parent);
    }
    false
}

/// A resposta de «esta camada entra?».
#[must_use]
pub fn entra(s: &Sprite, l: &Layer, c: Camadas) -> bool {
    match c {
        Camadas::Visiveis => !l.reference && s.is_shown(l.id),
        Camadas::Todas => !l.reference,
        Camadas::Uma(alvo) => dentro_de(s, l, alvo),
    }
}

/// Os índices da linha do tempo pela ordem de saída.
#[must_use]
pub fn ordem(s: &Sprite, q: Quadros, d: Direcao) -> Vec<usize> {
    let ultimo = s.frames().len().saturating_sub(1);
    let (de, ate) = match q {
        Quadros::Todos => (0, ultimo),
        Quadros::Tag(i) => s
            .tags()
            .get(i)
            .map_or((0, ultimo), |t| (t.from as usize, t.to as usize)),
        Quadros::Intervalo { de, ate } => (de.min(ate), de.max(ate)),
    };
    let (de, ate) = (de.min(ultimo), ate.min(ultimo));
    let ida: Vec<usize> = (de..=ate).collect();
    let volta: Vec<usize> = ida.iter().rev().copied().collect();
    let miolo = |v: &[usize]| v.get(1..v.len().saturating_sub(1)).unwrap_or(&[]).to_vec();
    match d {
        Direcao::Forward => ida,
        Direcao::Reverse => volta,
        Direcao::PingPong => [ida.clone(), miolo(&volta)].concat(),
        Direcao::PingPongReverse => [volta.clone(), miolo(&ida)].concat(),
    }
}

/// O vizinho mais próximo em escala `f` (medido contra `--scale`): a célula de destino `x` lê
/// `⌊x / f⌋`, e o lado é `⌊lado · f⌋`, no mínimo 1.
#[must_use]
pub fn escalado(rgba: &[u8], w: u32, h: u32, f: f64) -> (u32, u32, Vec<u8>) {
    let nw = ((f64::from(w) * f).floor() as u32).max(1);
    let nh = ((f64::from(h) * f).floor() as u32).max(1);
    let mut out = Vec::with_capacity(nw as usize * nh as usize * 4);
    for y in 0..nh {
        let sy = ((f64::from(y) / f).floor() as u32).min(h - 1) as usize;
        for x in 0..nw {
            let sx = ((f64::from(x) / f).floor() as u32).min(w - 1) as usize;
            let i = (sy * w as usize + sx) * 4;
            out.extend_from_slice(&rgba[i..i + 4]);
        }
    }
    (nw, nh, out)
}

/// O composto de um frame com as camadas pedidas.
#[must_use]
pub fn composto(s: &Sprite, frame: FrameId, c: Camadas) -> Vec<u8> {
    compose_picked(s, frame, &|l| entra(s, l, c))
}

/// **Os quadros do pedido.**
#[must_use]
pub fn quadros(s: &Sprite, p: &Pedido) -> Vec<QuadroSaida> {
    quadros_com(s, p, &|_| {})
}

/// [`quadros`], dizendo a fração feita a cada quadro (a barra de um export longo).
#[must_use]
pub fn quadros_com(s: &Sprite, p: &Pedido, feito: &dyn Fn(f32)) -> Vec<QuadroSaida> {
    quadros_da_area(s, p, false, feito)
}

/// [`quadros_com`] com a escolha da caixa da slice: a de cada quadro, ou (`uma_caixa`) a que cobre
/// todos, como pede um formato animado.
pub(crate) fn quadros_da_area(
    s: &Sprite,
    p: &Pedido,
    uma_caixa: bool,
    feito: &dyn Fn(f32),
) -> Vec<QuadroSaida> {
    let f = if p.escala > 0.0 { p.escala } else { 1.0 };
    let lista = ordem(s, p.quadros, p.direcao);
    let total = lista.len().max(1) as f32;
    let comum = match p.area {
        Area::Slice(id) if uma_caixa => Some(fatias::caixa_comum(s, id, &lista)),
        _ => None,
    };
    lista
        .iter()
        .copied()
        .enumerate()
        .filter_map(|(n, indice)| {
            feito(n as f32 / total);
            let caixa = match p.area {
                Area::Canvas => None,
                Area::Slice(id) => {
                    let daqui = fatias::caixa_no_quadro(s, id, indice)?;
                    Some(comum.unwrap_or(daqui))
                }
            };
            let fr = s.frames()[indice];
            let px = composto(s, fr.id, p.camadas);
            let (cw, ch, px) = match caixa {
                None => (s.width(), s.height(), px),
                Some(r) => {
                    let corte = [r.x as u32, r.y as u32, r.w, r.h];
                    (r.w, r.h, recorte(&px, s.width(), corte))
                }
            };
            let (w, h, rgba) = if (f - 1.0).abs() < f64::EPSILON {
                (cw, ch, px)
            } else {
                escalado(&px, cw, ch, f)
            };
            Some(QuadroSaida {
                indice,
                frame: fr.id,
                duration_ms: fr.duration_ms,
                w,
                h,
                rgba,
            })
        })
        .collect()
}
