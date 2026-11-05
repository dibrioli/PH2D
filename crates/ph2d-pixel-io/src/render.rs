//! **A pergunta de todo exportador: o que sai em cada quadro?** PNG, GIF, APNG, WebP e a folha pedem
//! aqui; nenhum compõe por conta própria.

use ph2d_pixel_core::{FrameId, Layer, LayerId, Sprite, compose_picked};

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

/// O pedido de um export.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Pedido {
    pub camadas: Camadas,
    pub quadros: Quadros,
    pub direcao: Direcao,
    /// 1 = sem escala; 0 vale 1.
    pub escala: f64,
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
    let f = if p.escala > 0.0 { p.escala } else { 1.0 };
    ordem(s, p.quadros, p.direcao)
        .into_iter()
        .map(|indice| {
            let fr = s.frames()[indice];
            let px = composto(s, fr.id, p.camadas);
            let (w, h, rgba) = if (f - 1.0).abs() < f64::EPSILON {
                (s.width(), s.height(), px)
            } else {
                escalado(&px, s.width(), s.height(), f)
            };
            QuadroSaida {
                indice,
                frame: fr.id,
                duration_ms: fr.duration_ms,
                w,
                h,
                rgba,
            }
        })
        .collect()
}
