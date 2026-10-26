//! **A porta única de mutação**: [`apply`] muda o sprite e devolve o inverso exato.

use std::collections::BTreeMap;

use crate::palette::{MAX_COLORS, invert_permutation, is_permutation};
use crate::sprite::{Cel, CelKey, ColorMode, Frame, FrameId, Layer, LayerId, Rect, Sprite, Tag};

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
    /// A lista de frames inteira, os cels que mudam com ela e as tags (novo, apagar, mover,
    /// duração, propriedades e ligação de cels, tags). O inverso é o estado de antes.
    Frames(Box<FrameSet>),
}

/// O que [`Op::Frames`] põe no sprite: `frames` e `tags` substituem as listas; cada `(chave, cel)`
/// de `cels` põe (`Some`) ou tira (`None`) esse cel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrameSet {
    pub frames: Vec<Frame>,
    pub cels: Vec<(CelKey, Option<Cel>)>,
    pub tags: Vec<Tag>,
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
    /// A lista de frames ou de tags viola uma lei (o campo diz qual).
    BadFrames(&'static str),
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
            Op::Frames(f) => {
                f.frames.len() * std::mem::size_of::<Frame>()
                    + f.tags.len() * std::mem::size_of::<Tag>()
                    + f.cels
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
        Op::Frames(f) => apply_frames(sprite, *f).map(|f| Op::Frames(Box::new(f))),
    }
}

/// As leis dos cels no estado FINAL de um op (`changes` por cima de `s.cels`, ou no lugar dele se
/// `replace`): camada de imagem e frame existentes; o dono tem os bytes da caixa e só fica vazio
/// se houver ligado a ele; o ligado não tem bytes e aponta para um dono da mesma camada.
pub(crate) fn check_cels(
    s: &Sprite,
    layers: &[Layer],
    frames: &[Frame],
    changes: &[(CelKey, Option<Cel>)],
    replace: bool,
    bpp: usize,
) -> Result<(), OpError> {
    let mut fin: BTreeMap<CelKey, &Cel> = if replace {
        BTreeMap::new()
    } else {
        s.cels.iter().map(|(k, c)| (*k, c)).collect()
    };
    for (k, c) in changes {
        match c {
            Some(c) => fin.insert(*k, c),
            None => fin.remove(k),
        };
    }
    let image = |id: LayerId| layers.iter().any(|l| l.id == id && !l.is_group());
    for (&k, c) in &fin {
        if !image(k.0) || !frames.iter().any(|f| f.id == k.1) {
            return Err(OpError::BadCel(k));
        }
        match c.link {
            Some(f) => {
                let ok = f != k.1
                    && c.data.is_empty()
                    && c.bbox.is_empty()
                    && fin.get(&(k.0, f)).is_some_and(|o| o.link.is_none());
                if !ok {
                    return Err(OpError::BadCel(k));
                }
            }
            None => {
                let empty_ok =
                    c.bbox.is_empty() && fin.iter().any(|(o, x)| o.0 == k.0 && x.link == Some(k.1));
                if c.data.len() != c.bbox.area() * bpp || (c.bbox.is_empty() && !empty_ok) {
                    return Err(OpError::BadCel(k));
                }
            }
        }
    }
    Ok(())
}

/// As leis da linha do tempo: pelo menos um frame; ids únicos e já emitidos; cada tag dentro dela.
pub(crate) fn check_frames(frames: &[Frame], tags: &[Tag], next_id: u32) -> Result<(), OpError> {
    let bad = OpError::BadFrames;
    if frames.is_empty() {
        return Err(bad("no frame"));
    }
    let mut seen = std::collections::BTreeSet::new();
    for f in frames {
        if f.id.0 == 0 || f.id.0 >= next_id || !seen.insert(f.id) {
            return Err(bad("frame.id"));
        }
    }
    let n = frames.len() as u32;
    if tags.iter().any(|t| t.from > t.to || t.to >= n) {
        return Err(bad("tag.range"));
    }
    Ok(())
}

fn put_cels(s: &mut Sprite, cels: Vec<(CelKey, Option<Cel>)>) -> Vec<(CelKey, Option<Cel>)> {
    let mut before = Vec::with_capacity(cels.len());
    for (key, cel) in cels {
        let old = match cel {
            Some(c) => s.cels.insert(key, c),
            None => s.cels.remove(&key),
        };
        before.push((key, old));
    }
    before.reverse();
    before
}

fn apply_frames(s: &mut Sprite, set: FrameSet) -> Result<FrameSet, OpError> {
    check_frames(&set.frames, &set.tags, s.next_id)?;
    check_cels(s, &s.layers, &set.frames, &set.cels, false, s.mode.bpp())?;
    let cels = put_cels(s, set.cels);
    Ok(FrameSet {
        frames: std::mem::replace(&mut s.frames, set.frames),
        cels,
        tags: std::mem::replace(&mut s.tags, set.tags),
    })
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
            let g = layers
                .iter()
                .find(|x| x.id == id)
                .ok_or(bad("layer.parent"))?;
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
    // Nenhum cel pode ficar órfão: o de uma camada que sai (ou vira grupo) tem de vir em `cels`.
    check_cels(s, &set.layers, &s.frames, &set.cels, false, s.mode.bpp())?;
    let cels = put_cels(s, set.cels);
    Ok(LayerSet {
        layers: std::mem::replace(&mut s.layers, set.layers),
        cels,
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
    let all: Vec<(CelKey, Option<Cel>)> =
        w.cels.iter().map(|(k, c)| (*k, Some(c.clone()))).collect();
    check_cels(s, &s.layers, &s.frames, &all, true, w.mode.bpp())?;
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
    // O traço num cel ligado pinta a imagem do dono: todos os ligados a vêem.
    let key: CelKey = s.owner((p.layer, p.frame));
    let inverse = Patch {
        layer: p.layer,
        frame: p.frame,
        bbox: s.cels.get(&key).map(|c| c.bbox),
        rect: p.rect,
        pixels: s.read_rect(key, p.rect),
    };
    let held = s
        .cels
        .iter()
        .any(|(k, c)| k.0 == key.0 && c.link == Some(key.1));
    let Some(bbox) = p.bbox.filter(|b| !b.is_empty()) else {
        match s.cels.get_mut(&key) {
            // Com ligados, a imagem fica vazia em vez de sair (eles continuam a apontar para ela).
            Some(c) if held => {
                c.bbox = Rect::default();
                c.data.clear();
            }
            _ => {
                s.cels.remove(&key);
            }
        }
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
    let mut cel = Cel::image(bbox, data);
    if let Some(o) = &old {
        (cel.opacity, cel.z) = (o.opacity, o.z);
    }
    s.cels.insert(key, cel);
    Ok(inverse)
}
