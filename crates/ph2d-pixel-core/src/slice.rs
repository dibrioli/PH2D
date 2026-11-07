//! **As slices** (P11): retângulos com nome, animados por keys. Medido: `docs/PixelArt/P11_oraculos_ferramenta.md`.
//!
//! A key vale até à seguinte; antes da primeira vale a primeira; caixa vazia esconde até à seguinte.
//! Nada se corta ao ler: fora do canvas, centro e pivot fora da caixa e nomes gémeos sobrevivem.
//! Ao mexer em quadros e na geometria a slice segue a arte (decisão do dono, `P11_plano.md` §4.4 e §9.1).

use std::collections::BTreeSet;

use crate::edit::Rotation;
use crate::op::{Op, OpError};
use crate::sprite::{Frame, FrameId, Rect, Sprite};

/// A cor das slices novas e das que vêm sem cor no ficheiro (Trial: `#0000FF`, T1).
pub const DEFAULT_SLICE_COLOR: [u8; 4] = [0, 0, 255, 255];

/// A identidade de uma slice: os nomes podem repetir-se (F3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SliceId(pub u32);

/// O valor de uma slice num quadro, com centro e pivot relativos à caixa.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SliceShape {
    /// Vazia esconde a slice.
    pub bounds: Rect,
    pub center: Option<Rect>,
    /// O píxel `bounds + pivot` (T4).
    pub pivot: Option<(i32, i32)>,
}

/// Uma key: `shape` vale de `frame` até à key seguinte.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SliceKey {
    /// Posição na linha do tempo. Pode passar do fim: um ficheiro assim abre (F3).
    pub frame: u32,
    pub shape: SliceShape,
}

/// Uma slice: `keys` em ordem de quadro, sem quadros repetidos.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Slice {
    pub id: SliceId,
    pub name: String,
    /// `None` só em ficheiro alheio: desenha-se a [`DEFAULT_SLICE_COLOR`].
    pub color: Option<[u8; 4]>,
    pub data: String,
    pub keys: Vec<SliceKey>,
}

impl Slice {
    /// O valor no quadro `frame`, escondido ou não.
    #[must_use]
    pub fn shape_at(&self, frame: u32) -> Option<SliceShape> {
        let i = self.keys.partition_point(|k| k.frame <= frame);
        self.keys.get(i.saturating_sub(1)).map(|k| k.shape)
    }

    /// O valor no quadro `frame`, se lá aparece.
    #[must_use]
    pub fn visible_at(&self, frame: u32) -> Option<SliceShape> {
        self.shape_at(frame).filter(|k| !k.bounds.is_empty())
    }

    #[must_use]
    pub fn has_key_at(&self, frame: u32) -> bool {
        self.keys.iter().any(|k| k.frame == frame)
    }
}

impl Sprite {
    #[must_use]
    pub fn slices(&self) -> &[Slice] {
        &self.slices
    }

    #[must_use]
    pub fn slice(&self, id: SliceId) -> Option<&Slice> {
        self.slices.iter().find(|s| s.id == id)
    }

    /// As slices por nome, sem distinguir maiúsculas; as gémeas ficam todas, pela ordem de criação (T1, F4).
    #[must_use]
    pub fn slices_by_name(&self) -> Vec<&Slice> {
        let mut v: Vec<&Slice> = self.slices.iter().collect();
        v.sort_by_cached_key(|c| (c.name.to_lowercase(), c.id));
        v
    }
}

/// As leis de uma lista de slices: ids únicos e já emitidos, keys em ordem estrita.
pub(crate) fn check_slices(slices: &[Slice], next_id: u32) -> Result<(), OpError> {
    let bad = OpError::BadSlices;
    let mut seen = BTreeSet::new();
    for s in slices {
        if s.id.0 == 0 || s.id.0 >= next_id || !seen.insert(s.id) {
            return Err(bad("slice.id"));
        }
        if s.keys.windows(2).any(|k| k[0].frame >= k[1].frame) {
            return Err(bad("slice.keys"));
        }
    }
    Ok(())
}

/// O passo que troca a lista inteira, ou `None` se nada muda.
fn set(s: &Sprite, slices: Vec<Slice>) -> Option<Op> {
    (slices != s.slices).then_some(Op::Slices(slices))
}

/// `Slice N`: o maior N em uso mais um (depois de apagar a 2 de 1,3,4 vem a 5, T1).
fn next_name(s: &Sprite) -> String {
    let n = s
        .slices
        .iter()
        .filter_map(|c| c.name.strip_prefix("Slice ")?.parse::<u32>().ok())
        .max()
        .unwrap_or(0);
    format!("Slice {}", n.saturating_add(1))
}

/// **Criar** com o arrasto `r`, recortado ao canvas; `None` se nada sobra (o clique, T1).
pub fn create(s: &mut Sprite, frame: u32, r: Rect) -> Option<(Op, SliceId)> {
    let bounds = r.intersect(s.bounds());
    if bounds.is_empty() {
        return None;
    }
    let id = SliceId(s.reserve_id());
    let mut slices = s.slices.clone();
    slices.push(Slice {
        id,
        name: next_name(s),
        color: Some(DEFAULT_SLICE_COLOR),
        data: String::new(),
        keys: vec![SliceKey {
            frame,
            shape: SliceShape {
                bounds,
                ..SliceShape::default()
            },
        }],
    });
    Some((Op::Slices(slices), id))
}

/// **Editar a slice no quadro `frame`**: a key desse quadro, criada se faltar; sem mudança, nenhum
/// passo e nenhuma key (T5).
#[must_use]
pub fn set_shape(s: &Sprite, id: SliceId, frame: u32, shape: SliceShape) -> Option<Op> {
    let mut slices = s.slices.clone();
    let c = slices.iter_mut().find(|c| c.id == id)?;
    if c.shape_at(frame) == Some(shape) {
        return None;
    }
    match c.keys.binary_search_by_key(&frame, |k| k.frame) {
        Ok(i) => c.keys[i].shape = shape,
        Err(i) => c.keys.insert(i, SliceKey { frame, shape }),
    }
    set(s, slices)
}

/// Editar várias de uma vez, num só passo.
#[must_use]
pub fn set_shapes(s: &Sprite, frame: u32, shapes: &[(SliceId, SliceShape)]) -> Option<Op> {
    let mut slices = s.slices.clone();
    for (id, shape) in shapes {
        let c = slices.iter_mut().find(|c| c.id == *id)?;
        if c.shape_at(frame) == Some(*shape) {
            continue;
        }
        match c.keys.binary_search_by_key(&frame, |k| k.frame) {
            Ok(i) => c.keys[i].shape = *shape,
            Err(i) => c.keys.insert(
                i,
                SliceKey {
                    frame,
                    shape: *shape,
                },
            ),
        }
    }
    set(s, slices)
}

/// O nome, a cor e o texto valem para a slice inteira.
#[must_use]
pub fn set_info(s: &Sprite, id: SliceId, name: &str, color: [u8; 4], data: &str) -> Option<Op> {
    let mut slices = s.slices.clone();
    let c = slices.iter_mut().find(|c| c.id == id)?;
    name.clone_into(&mut c.name);
    c.color = Some(color);
    data.clone_into(&mut c.data);
    set(s, slices)
}

/// **Apagar** as slices inteiras.
#[must_use]
pub fn delete(s: &Sprite, ids: &[SliceId]) -> Option<Op> {
    let slices = s
        .slices
        .iter()
        .filter(|c| !ids.contains(&c.id))
        .cloned()
        .collect();
    set(s, slices)
}

/// **Remover a mudança deste quadro** (além do Aseprite, §9.2): a key sai e o valor anterior volta
/// a valer. A última key não sai: a slice não pode ficar sem caixa.
#[must_use]
pub fn remove_key(s: &Sprite, id: SliceId, frame: u32) -> Option<Op> {
    let mut slices = s.slices.clone();
    let c = slices.iter_mut().find(|c| c.id == id)?;
    if c.keys.len() < 2 {
        return None;
    }
    let i = c.keys.iter().position(|k| k.frame == frame)?;
    c.keys.remove(i);
    set(s, slices)
}

/// `Duplicate Slice`: `<nome> Copy`, deslocada +2,+2 (T2). Devolve a cópia para a selecionar.
pub fn duplicate(s: &mut Sprite, id: SliceId) -> Option<(Op, SliceId)> {
    let mut copy = s.slice(id)?.clone();
    let new = SliceId(s.reserve_id());
    copy.id = new;
    copy.name.push_str(" Copy");
    for k in &mut copy.keys {
        k.shape.bounds.x = k.shape.bounds.x.saturating_add(2);
        k.shape.bounds.y = k.shape.bounds.y.saturating_add(2);
    }
    let mut slices = s.slices.clone();
    slices.push(copy);
    Some((Op::Slices(slices), new))
}

/// O centro ao ligar `9-Slices`: margem de 1 em todos os lados (T3), sem nunca ficar sem miolo.
#[must_use]
pub fn default_center(bounds: Rect) -> Rect {
    if bounds.w < 3 || bounds.h < 3 {
        return Rect::new(0, 0, bounds.w, bounds.h);
    }
    Rect::new(1, 1, bounds.w - 2, bounds.h - 2)
}

// --- Os quadros: a slice segue a arte ---

/// As slices depois de a linha do tempo passar de `old` a `new`: cada quadro que fica leva o valor
/// que tinha; o quadro novo herda o do anterior (ou o do seguinte, se for o primeiro). Mudança que
/// não toca na ordem deixa as keys como estão.
pub(crate) fn after_frames(slices: &[Slice], old: &[Frame], new: &[Frame]) -> Vec<Slice> {
    let ids = |f: &[Frame]| f.iter().map(|f| f.id).collect::<Vec<FrameId>>();
    let (old, new) = (ids(old), ids(new));
    if old == new {
        return slices.to_vec();
    }
    let mut out = slices.to_vec();
    for c in out.iter_mut().filter(|c| !c.keys.is_empty()) {
        let shape_of = |id: FrameId| {
            let i = old.iter().position(|o| *o == id)?;
            c.shape_at(i as u32)
        };
        let shapes: Vec<Option<SliceShape>> = new.iter().map(|id| shape_of(*id)).collect();
        let fallback = shapes.iter().flatten().next().copied();
        let mut keys: Vec<SliceKey> = Vec::new();
        for (i, shape) in shapes.iter().enumerate() {
            let shape = shape.or(keys.last().map(|k| k.shape)).or(fallback);
            let Some(shape) = shape else { continue };
            if keys.last().is_none_or(|k| k.shape != shape) {
                keys.push(SliceKey {
                    frame: i as u32,
                    shape,
                });
            }
        }
        if !keys.is_empty() {
            c.keys = keys;
        }
    }
    out
}

// --- A geometria: a slice segue a arte ---

fn map_shapes(slices: &[Slice], f: impl Fn(SliceShape) -> SliceShape) -> Vec<Slice> {
    let mut out = slices.to_vec();
    for k in out.iter_mut().flat_map(|c| c.keys.iter_mut()) {
        if !k.shape.bounds.is_empty() {
            k.shape = f(k.shape);
        }
    }
    out
}

/// Canvas Size, Crop e Trim: a caixa anda; centro e pivot, relativos a ela, ficam (T7).
pub(crate) fn translated(slices: &[Slice], dx: i32, dy: i32) -> Vec<Slice> {
    map_shapes(slices, |mut k| {
        k.bounds.x = k.bounds.x.saturating_add(dx);
        k.bounds.y = k.bounds.y.saturating_add(dy);
        k
    })
}

/// O espelho horizontal de `r` dentro de uma largura `outer`.
fn mirror_x(r: Rect, outer: u32) -> Rect {
    let x = i64::from(outer) - i64::from(r.x) - i64::from(r.w);
    Rect { x: x as i32, ..r }
}

fn mirror_y(r: Rect, outer: u32) -> Rect {
    let y = i64::from(outer) - i64::from(r.y) - i64::from(r.h);
    Rect { y: y as i32, ..r }
}

/// Flip Canvas: o pivot é um píxel, por isso espelha em `w − 1`.
pub(crate) fn flipped(slices: &[Slice], w: u32, h: u32, horizontal: bool) -> Vec<Slice> {
    map_shapes(slices, |k| {
        let b = k.bounds;
        if horizontal {
            SliceShape {
                bounds: mirror_x(b, w),
                center: k.center.map(|c| mirror_x(c, b.w)),
                pivot: k.pivot.map(|(x, y)| (b.w as i32 - 1 - x, y)),
            }
        } else {
            SliceShape {
                bounds: mirror_y(b, h),
                center: k.center.map(|c| mirror_y(c, b.h)),
                pivot: k.pivot.map(|(x, y)| (x, b.h as i32 - 1 - y)),
            }
        }
    })
}

/// A rotação de `r` dentro de uma área `outer`, pela lei dos píxeis de `rotate_canvas`.
fn turn(r: Rect, outer: (u32, u32), rot: Rotation) -> Rect {
    let (ow, oh) = (i64::from(outer.0), i64::from(outer.1));
    let (x, y, w, h) = (
        i64::from(r.x),
        i64::from(r.y),
        i64::from(r.w),
        i64::from(r.h),
    );
    match rot {
        Rotation::Cw90 => Rect::new((oh - y - h) as i32, r.x, r.h, r.w),
        Rotation::Ccw90 => Rect::new(r.y, (ow - x - w) as i32, r.h, r.w),
        Rotation::R180 => Rect::new((ow - x - w) as i32, (oh - y - h) as i32, r.w, r.h),
    }
}

/// Rotate Canvas.
pub(crate) fn rotated(slices: &[Slice], w: u32, h: u32, rot: Rotation) -> Vec<Slice> {
    map_shapes(slices, |k| {
        let b = k.bounds;
        let inner = (b.w, b.h);
        SliceShape {
            bounds: turn(b, (w, h), rot),
            center: k.center.map(|c| turn(c, inner, rot)),
            pivot: k.pivot.map(|p| {
                let px = turn(Rect::new(p.0, p.1, 1, 1), inner, rot);
                (px.x, px.y)
            }),
        }
    })
}

/// Sprite Size: posição e pivot truncam para zero; cada BORDA trunca, e o tamanho é a diferença
/// (G1: `4,15` a 50 % dá largura 7). Uma caixa que existia não some: mínimo 1 (sem medida).
fn scale_coord(v: i32, to: u32, from: u32) -> i32 {
    (i64::from(v) * i64::from(to) / i64::from(from.max(1))) as i32
}

fn scale_span(start: i32, len: u32, to: u32, from: u32) -> (i32, u32) {
    let a = scale_coord(start, to, from);
    let end = i64::from(start) + i64::from(len);
    let b = end * i64::from(to) / i64::from(from.max(1));
    (a, (b - i64::from(a)).max(1) as u32)
}

fn scale_rect(r: Rect, sx: (u32, u32), sy: (u32, u32)) -> Rect {
    let (x, w) = scale_span(r.x, r.w, sx.0, sx.1);
    let (y, h) = scale_span(r.y, r.h, sy.0, sy.1);
    Rect::new(x, y, w, h)
}

/// Sprite Size de `from` para `to`, em caixa, centro e pivot.
pub(crate) fn scaled(slices: &[Slice], from: (u32, u32), to: (u32, u32)) -> Vec<Slice> {
    let (sx, sy) = ((to.0, from.0), (to.1, from.1));
    map_shapes(slices, |k| SliceShape {
        bounds: scale_rect(k.bounds, sx, sy),
        center: k.center.map(|c| scale_rect(c, sx, sy)),
        pivot: k
            .pivot
            .map(|(x, y)| (scale_coord(x, sx.0, sx.1), scale_coord(y, sy.0, sy.1))),
    })
}
