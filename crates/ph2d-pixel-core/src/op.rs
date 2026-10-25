//! **A porta única de mutação**: [`apply`] muda o sprite e devolve o inverso exato.

use std::collections::BTreeMap;

use crate::palette::{MAX_COLORS, invert_permutation, is_permutation};
use crate::sprite::{Cel, CelKey, ColorMode, FrameId, Layer, LayerId, Rect, Sprite};

/// Uma mutação do documento. O catálogo cresce por onda; cada variante tem inverso exato.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Op {
    /// Células de um cel (traço, borracha, preenchimento…).
    Patch(Patch),
    /// A paleta e o índice transparente, sem tocar as células (editar uma cor, Palette Size, Load
    /// Palette). Em Indexed, mudar uma cor muda o desenho — é a lei do modo.
    Palette {
        colors: Vec<[u8; 4]>,
        transparent: u8,
    },
    /// **Reordenar a paleta sem mudar a imagem**: `perm[novo] = antigo`; o transparente e, em
    /// Indexed, cada célula seguem a cor (MVP «a lei do remap»).
    Remap { perm: Vec<usize> },
    /// O documento inteiro de uma vez: modo, paleta, transparente e cels (Color Mode, New Palette
    /// from Sprite com remapeamento). O inverso é o estado de antes.
    Whole(Box<Whole>),
    /// A lista de camadas inteira e os cels que mudam com ela (nova, apagar, mover, propriedades,
    /// duplicar, Merge Down, Flatten…). O inverso é a lista de antes e esses cels como estavam.
    Layers(Box<LayerSet>),
}

/// O que [`Op::Layers`] põe no sprite: `layers` substitui a lista; cada `(chave, cel)` de `cels`
/// põe (`Some`) ou tira (`None`) esse cel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayerSet {
    pub layers: Vec<Layer>,
    pub cels: Vec<(CelKey, Option<Cel>)>,
}

/// O que [`Op::Whole`] põe no sprite.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Whole {
    pub mode: ColorMode,
    pub palette: Vec<[u8; 4]>,
    pub transparent: u8,
    pub cels: BTreeMap<CelKey, Cel>,
}

/// «Põe o cel `(layer, frame)` com a caixa `bbox` (`None` = sem cel), guardando o que já estava
/// dentro dela, e escreve `pixels` em `rect` (cortado à caixa).»
///
/// O inverso é o mesmo op com a caixa e os bytes de antes — por isso desfazer devolve o cel ao
/// byte, inclusive a caixa que o traço tinha alargado.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Patch {
    pub layer: LayerId,
    pub frame: FrameId,
    pub bbox: Option<Rect>,
    pub rect: Rect,
    /// `rect.area() * bpp` bytes, linha a linha.
    pub pixels: Vec<u8>,
}

/// Op recusado; o documento fica intacto.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpError {
    UnknownLayer(LayerId),
    UnknownFrame(FrameId),
    /// O tamanho de `pixels` não bate com `rect` × bpp.
    PixelCount {
        expected: usize,
        got: usize,
    },
    /// Paleta vazia, com mais de [`MAX_COLORS`] cores ou transparente fora dela.
    BadPalette,
    /// `perm` não é uma permutação do tamanho da paleta.
    BadPermutation,
    /// Um cel do [`Whole`] não bate com o modo (bytes ≠ área × bpp) ou nomeia camada/frame que
    /// não existe.
    BadCel(CelKey),
    /// A lista de camadas viola uma lei da árvore (o campo diz qual).
    BadLayers(&'static str),
}

impl Op {
    /// Os bytes que o op segura — a régua do tecto do histórico.
    #[must_use]
    pub fn bytes(&self) -> usize {
        match self {
            Op::Patch(p) => p.pixels.len() + std::mem::size_of::<Patch>(),
            Op::Palette { colors, .. } => colors.len() * 4 + std::mem::size_of::<Op>(),
            Op::Remap { perm } => perm.len() * std::mem::size_of::<usize>(),
            Op::Layers(l) => {
                l.layers.len() * std::mem::size_of::<Layer>()
                    + l.cels
                        .iter()
                        .map(|(_, c)| c.as_ref().map_or(0, |c| c.data.len()) + 32)
                        .sum::<usize>()
            }
            Op::Whole(w) => {
                w.palette.len() * 4
                    + w.cels.values().map(|c| c.data.len() + 32).sum::<usize>()
                    + std::mem::size_of::<Whole>()
            }
        }
    }
}

/// Aplica `op` e devolve o inverso. Falha sem mudar nada.
pub fn apply(sprite: &mut Sprite, op: Op) -> Result<Op, OpError> {
    match op {
        Op::Patch(p) => apply_patch(sprite, p).map(Op::Patch),
        Op::Palette {
            colors,
            transparent,
        } => {
            if colors.is_empty()
                || colors.len() > MAX_COLORS
                || usize::from(transparent) >= colors.len()
            {
                return Err(OpError::BadPalette);
            }
            let colors = std::mem::replace(&mut sprite.palette, colors);
            let transparent = std::mem::replace(&mut sprite.transparent, transparent);
            Ok(Op::Palette {
                colors,
                transparent,
            })
        }
        Op::Remap { perm } => apply_remap(sprite, &perm).map(|()| Op::Remap {
            perm: invert_permutation(&perm),
        }),
        Op::Whole(w) => apply_whole(sprite, *w).map(|w| Op::Whole(Box::new(w))),
        Op::Layers(l) => apply_layers(sprite, *l).map(|l| Op::Layers(Box::new(l))),
    }
}

/// As leis da árvore de camadas: ids únicos e já emitidos; pai existe, é grupo e vem de uma
/// cadeia sem ciclo; a Background é no máximo uma, na raiz, a primeira e de imagem; há pelo
/// menos uma camada de imagem.
pub(crate) fn check_layers(layers: &[Layer], next_id: u32) -> Result<(), OpError> {
    let bad = OpError::BadLayers;
    if !layers.iter().any(|l| !l.is_group()) {
        return Err(bad("no image layer"));
    }
    let mut seen = std::collections::BTreeSet::new();
    for l in layers {
        if l.id.0 == 0 || l.id.0 >= next_id || !seen.insert(l.id) {
            return Err(bad("layer.id"));
        }
    }
    for (i, l) in layers.iter().enumerate() {
        let mut p = l.parent;
        let mut hops = 0;
        while let Some(id) = p {
            let g = layers.iter().find(|x| x.id == id).ok_or(bad("layer.parent"))?;
            if !g.is_group() || hops > layers.len() {
                return Err(bad("layer.parent"));
            }
            p = g.parent;
            hops += 1;
        }
        if l.background && (i != 0 || l.parent.is_some() || l.is_group()) {
            return Err(bad("layer.background"));
        }
    }
    Ok(())
}

fn apply_layers(s: &mut Sprite, set: LayerSet) -> Result<LayerSet, OpError> {
    check_layers(&set.layers, s.next_id)?;
    let bpp = s.mode.bpp();
    let image = |id: LayerId| set.layers.iter().any(|l| l.id == id && !l.is_group());
    for (key, cel) in &set.cels {
        if let Some(c) = cel
            && (!image(key.0)
                || !s.frames.iter().any(|f| f.id == key.1)
                || c.data.len() != c.bbox.area() * bpp)
        {
            return Err(OpError::BadCel(*key));
        }
    }
    // Nenhum cel pode ficar órfão: o de uma camada que sai (ou vira grupo) tem de vir em `cels`.
    let touched: std::collections::BTreeSet<CelKey> = set.cels.iter().map(|(k, _)| *k).collect();
    if let Some(k) = s.cels.keys().find(|k| !touched.contains(k) && !image(k.0)) {
        return Err(OpError::BadCel(*k));
    }
    let mut before = Vec::with_capacity(set.cels.len());
    for (key, cel) in set.cels {
        let old = match cel {
            Some(c) => s.cels.insert(key, c),
            None => s.cels.remove(&key),
        };
        before.push((key, old));
    }
    before.reverse();
    Ok(LayerSet {
        layers: std::mem::replace(&mut s.layers, set.layers),
        cels: before,
    })
}

fn apply_remap(s: &mut Sprite, perm: &[usize]) -> Result<(), OpError> {
    if perm.len() != s.palette.len() || !is_permutation(perm) {
        return Err(OpError::BadPermutation);
    }
    let inv = invert_permutation(perm);
    s.palette = perm.iter().map(|&old| s.palette[old]).collect();
    s.transparent = inv[usize::from(s.transparent)] as u8;
    if s.mode == ColorMode::Indexed {
        for cel in s.cels.values_mut() {
            for v in &mut cel.data {
                if let Some(&n) = inv.get(usize::from(*v)) {
                    *v = n as u8;
                }
            }
        }
    }
    Ok(())
}

fn apply_whole(s: &mut Sprite, w: Whole) -> Result<Whole, OpError> {
    if w.palette.is_empty()
        || w.palette.len() > MAX_COLORS
        || usize::from(w.transparent) >= w.palette.len()
    {
        return Err(OpError::BadPalette);
    }
    for (&key, cel) in &w.cels {
        let known = s.layer(key.0).is_some() && s.frames.iter().any(|f| f.id == key.1);
        if !known || cel.data.len() != cel.bbox.area() * w.mode.bpp() {
            return Err(OpError::BadCel(key));
        }
    }
    Ok(Whole {
        mode: std::mem::replace(&mut s.mode, w.mode),
        palette: std::mem::replace(&mut s.palette, w.palette),
        transparent: std::mem::replace(&mut s.transparent, w.transparent),
        cels: std::mem::replace(&mut s.cels, w.cels),
    })
}

fn apply_patch(s: &mut Sprite, p: Patch) -> Result<Patch, OpError> {
    if s.layer(p.layer).is_none() {
        return Err(OpError::UnknownLayer(p.layer));
    }
    if !s.frames.iter().any(|f| f.id == p.frame) {
        return Err(OpError::UnknownFrame(p.frame));
    }
    let bpp = s.mode.bpp();
    let expected = p.rect.area() * bpp;
    if p.pixels.len() != expected {
        return Err(OpError::PixelCount {
            expected,
            got: p.pixels.len(),
        });
    }
    let key: CelKey = (p.layer, p.frame);
    let inverse = Patch {
        layer: p.layer,
        frame: p.frame,
        bbox: s.cels.get(&key).map(|c| c.bbox),
        rect: p.rect,
        pixels: s.read_rect(key, p.rect),
    };
    let Some(bbox) = p.bbox.filter(|b| !b.is_empty()) else {
        s.cels.remove(&key);
        return Ok(inverse);
    };
    let transparent = s.transparent_value().to_le_bytes();
    let old = s.cels.remove(&key);
    let mut data = Vec::with_capacity(bbox.area() * bpp);
    for y in 0..bbox.h as i32 {
        for x in 0..bbox.w as i32 {
            let (cx, cy) = (bbox.x + x, bbox.y + y);
            match old.as_ref().filter(|c| c.bbox.contains(cx, cy)) {
                Some(c) => {
                    let i = ((cy - c.bbox.y) as usize * c.bbox.w as usize
                        + (cx - c.bbox.x) as usize)
                        * bpp;
                    data.extend_from_slice(&c.data[i..i + bpp]);
                }
                None => data.extend_from_slice(&transparent[..bpp]),
            }
        }
    }
    let clip = p.rect.intersect(bbox);
    for y in 0..clip.h as i32 {
        for x in 0..clip.w as i32 {
            let (cx, cy) = (clip.x + x, clip.y + y);
            let src =
                ((cy - p.rect.y) as usize * p.rect.w as usize + (cx - p.rect.x) as usize) * bpp;
            let dst = ((cy - bbox.y) as usize * bbox.w as usize + (cx - bbox.x) as usize) * bpp;
            data[dst..dst + bpp].copy_from_slice(&p.pixels[src..src + bpp]);
        }
    }
    s.cels.insert(key, Cel { bbox, data });
    Ok(inverse)
}
