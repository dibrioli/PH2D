//! **O menu Frame e as tags como construtores puros**: cada acção devolve UM [`Op::Frames`] (um
//! passo de desfazer) ou `None` quando não se aplica.
//!
//! Leis: inserir na posição `i` empurra a tag que começa em `i` ou depois e faz crescer a que a
//! contém; apagar é o inverso, e a tag de um só frame apagado sai; mover não mexe nas tags (a tag
//! nomeia o TRECHO). Um cel dono que sai deixa a imagem ao primeiro ligado que fica.

use crate::op::{FrameSet, Op};
use crate::sprite::{Cel, CelKey, Direction, Frame, FrameId, LayerId, Sprite, Tag};

/// O teto da duração de um frame: o `WORD` do cabeçalho de frame do `.ase`.
pub const MAX_FRAME_MS: u32 = u16::MAX as u32;

/// O que um frame novo leva das células do frame de origem.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NewFrame {
    /// Frame ▸ New Frame: cópia de cada cel (ligado nas camadas *continuous*).
    Copy,
    /// Frame ▸ New Empty Frame.
    Empty,
    /// Frame ▸ Duplicate Linked Cels: cada cel ligado ao de origem.
    Linked,
}

fn op(frames: Vec<Frame>, cels: Vec<(CelKey, Option<Cel>)>, tags: Vec<Tag>) -> Op {
    Op::Frames(Box::new(FrameSet { frames, cels, tags }))
}

/// As tags depois de um frame novo na posição `i`.
#[must_use]
pub fn tags_after_insert(tags: &[Tag], i: u32) -> Vec<Tag> {
    tags.iter()
        .cloned()
        .map(|mut t| {
            if i <= t.from {
                t.from += 1;
                t.to += 1;
            } else if i <= t.to {
                t.to += 1;
            }
            t
        })
        .collect()
}

/// As tags depois de apagar o frame da posição `i` (a de um só frame, esse, sai).
#[must_use]
pub fn tags_after_delete(tags: &[Tag], i: u32) -> Vec<Tag> {
    tags.iter()
        .filter(|t| !(t.from == i && t.to == i))
        .cloned()
        .map(|mut t| {
            if i < t.from {
                t.from -= 1;
                t.to -= 1;
            } else if i <= t.to {
                t.to -= 1;
            }
            t
        })
        .collect()
}

/// Um frame novo logo depois de `after`, com a duração dele.
pub fn new_frame(s: &mut Sprite, after: FrameId, content: NewFrame) -> Option<(Op, FrameId)> {
    let at = s.frame_index(after)?;
    let ms = s.frames()[at].duration_ms;
    insert(s, at + 1, after, content, ms)
}

/// Um frame novo na posição `at` com a duração `ms`, levando de `src` o que `content` pede.
pub fn insert(
    s: &mut Sprite,
    at: usize,
    src: FrameId,
    content: NewFrame,
    ms: u32,
) -> Option<(Op, FrameId)> {
    if at > s.frames().len() || s.frame_index(src).is_none() {
        return None;
    }
    let id = FrameId(s.reserve_id());
    let mut frames = s.frames().to_vec();
    frames.insert(
        at,
        Frame {
            id,
            duration_ms: ms.clamp(1, MAX_FRAME_MS),
        },
    );
    let after = src;
    let mut cels = Vec::new();
    if content != NewFrame::Empty {
        for l in s.layers().iter().filter(|l| !l.is_group()) {
            let src = (l.id, after);
            let Some(e) = s.entry(src) else { continue };
            let cel = if content == NewFrame::Linked || l.continuous {
                Cel::linked(s.owner(src).1, e.z)
            } else {
                Cel {
                    z: e.z,
                    link: None,
                    ..s.cel(src)?.clone()
                }
            };
            cels.push(((l.id, id), Some(cel)));
        }
    }
    let tags = tags_after_insert(s.tags(), at as u32);
    Some((op(frames, cels, tags), id))
}

/// Os cels a pôr para que nenhum ligado fique a apontar para um dono que SAI: o primeiro ligado
/// que fica (na ordem da linha do tempo) recebe a imagem e os outros passam a apontar para ele.
fn rehome(
    s: &Sprite,
    gone: &dyn Fn(CelKey) -> bool,
    order: &[FrameId],
) -> Vec<(CelKey, Option<Cel>)> {
    let mut out = Vec::new();
    for (&key, c) in s.entries().filter(|(k, c)| c.link.is_none() && gone(**k)) {
        let heirs: Vec<FrameId> = order
            .iter()
            .copied()
            .filter(|&f| {
                let k = (key.0, f);
                !gone(k) && s.entry(k).is_some_and(|e| e.link == Some(key.1))
            })
            .collect();
        let Some((&first, rest)) = heirs.split_first() else {
            continue;
        };
        let z = s.entry((key.0, first)).map_or(0, |e| e.z);
        out.push((
            (key.0, first),
            Some(Cel {
                z,
                link: None,
                ..c.clone()
            }),
        ));
        for &f in rest {
            let z = s.entry((key.0, f)).map_or(0, |e| e.z);
            out.push(((key.0, f), Some(Cel::linked(first, z))));
        }
    }
    out
}

/// Frame ▸ Delete Frame (os `ids`, nunca todos).
#[must_use]
pub fn delete(s: &Sprite, ids: &[FrameId]) -> Option<Op> {
    let gone_f = |f: FrameId| ids.contains(&f);
    if !s.frames().iter().any(|f| gone_f(f.id)) || s.frames().iter().all(|f| gone_f(f.id)) {
        return None;
    }
    let frames: Vec<Frame> = s
        .frames()
        .iter()
        .filter(|f| !gone_f(f.id))
        .copied()
        .collect();
    let order: Vec<FrameId> = frames.iter().map(|f| f.id).collect();
    let mut cels: Vec<(CelKey, Option<Cel>)> = s
        .entries()
        .filter(|(k, _)| gone_f(k.1))
        .map(|(k, _)| (*k, None))
        .collect();
    cels.extend(rehome(s, &|k: CelKey| gone_f(k.1), &order));
    let mut tags = s.tags().to_vec();
    let mut positions: Vec<u32> = ids
        .iter()
        .filter_map(|&f| s.frame_index(f))
        .map(|i| i as u32)
        .collect();
    positions.sort_unstable();
    for &i in positions.iter().rev() {
        tags = tags_after_delete(&tags, i);
    }
    Some(op(frames, cels, tags))
}

/// Arrastar o frame `id` para a posição `to` (0 = primeiro).
#[must_use]
pub fn move_to(s: &Sprite, id: FrameId, to: usize) -> Option<Op> {
    let from = s.frame_index(id)?;
    let mut frames = s.frames().to_vec();
    let to = to.min(frames.len() - 1);
    if from == to {
        return None;
    }
    let f = frames.remove(from);
    frames.insert(to, f);
    Some(op(frames, Vec::new(), s.tags().to_vec()))
}

/// Frame ▸ Reverse Frames: as posições `from..=to` ao contrário.
#[must_use]
pub fn reverse(s: &Sprite, from: usize, to: usize) -> Option<Op> {
    if from >= to || to >= s.frames().len() {
        return None;
    }
    let mut frames = s.frames().to_vec();
    frames[from..=to].reverse();
    Some(op(frames, Vec::new(), s.tags().to_vec()))
}

/// Frame Properties ▸ Duration (e *Constant Frame Rate* com todos os ids): `ms` cravado a
/// `1..=`[`MAX_FRAME_MS`].
#[must_use]
pub fn set_duration(s: &Sprite, ids: &[FrameId], ms: u32) -> Option<Op> {
    let ms = ms.clamp(1, MAX_FRAME_MS);
    let mut frames = s.frames().to_vec();
    let mut changed = false;
    for f in frames.iter_mut().filter(|f| ids.contains(&f.id)) {
        changed |= f.duration_ms != ms;
        f.duration_ms = ms;
    }
    changed.then(|| op(frames, Vec::new(), s.tags().to_vec()))
}

/// Cel Properties: a opacidade (da imagem — muda em todos os ligados) e o z (só deste cel).
#[must_use]
pub fn set_cel(s: &Sprite, key: CelKey, opacity: Option<u8>, z: Option<i16>) -> Option<Op> {
    let mut cels = Vec::new();
    if let Some(o) = opacity {
        let ok = s.owner(key);
        let img = s.entry(ok)?;
        if img.opacity != o {
            cels.push((
                ok,
                Some(Cel {
                    opacity: o,
                    ..img.clone()
                }),
            ));
        }
    }
    if let Some(z) = z {
        let me = cels
            .iter()
            .find(|(k, _)| *k == key)
            .and_then(|(_, c)| c.clone())
            .or_else(|| s.entry(key).cloned())?;
        if me.z != z {
            cels.retain(|(k, _)| *k != key);
            cels.push((key, Some(Cel { z, ..me })));
        }
    }
    (!cels.is_empty()).then(|| op(s.frames().to_vec(), cels, s.tags().to_vec()))
}

/// Cel ▸ Links Cels: os cels da camada nos `frames` passam a partilhar a imagem do primeiro deles
/// (na linha do tempo); os ligados de fora aos que deixam de ser donos seguem para o novo dono.
#[must_use]
pub fn link(s: &Sprite, layer: LayerId, frames: &[FrameId]) -> Option<Op> {
    let order: Vec<FrameId> = s
        .frames()
        .iter()
        .map(|f| f.id)
        .filter(|f| frames.contains(f) && s.entry((layer, *f)).is_some())
        .collect();
    let (&first, rest) = order.split_first()?;
    let owner = s.owner((layer, first));
    let mut cels = Vec::new();
    for (&k, e) in s.entries().filter(|(k, _)| k.0 == layer && k.1 != owner.1) {
        let joins =
            rest.contains(&k.1) || e.link.is_some_and(|o| o != owner.1 && rest.contains(&o));
        if joins && e.link != Some(owner.1) {
            cels.push((k, Some(Cel::linked(owner.1, e.z))));
        }
    }
    (!cels.is_empty()).then(|| op(s.frames().to_vec(), cels, s.tags().to_vec()))
}

/// Cel ▸ Unlink Cel: o cel `key` fica com uma cópia própria da imagem.
#[must_use]
pub fn unlink(s: &Sprite, key: CelKey) -> Option<Op> {
    let e = s.entry(key)?;
    let order: Vec<FrameId> = s.frames().iter().map(|f| f.id).collect();
    let cels = if e.link.is_some() {
        vec![(
            key,
            Some(Cel {
                z: e.z,
                link: None,
                ..s.cel(key)?.clone()
            }),
        )]
    } else {
        let mut v = rehome(s, &|k: CelKey| k == key, &order);
        if v.is_empty() {
            return None;
        }
        v.push((key, Some(e.clone())));
        v
    };
    Some(op(s.frames().to_vec(), cels, s.tags().to_vec()))
}

/// Frame ▸ Tags ▸ New Tag sobre `from..=to` (posições).
#[must_use]
pub fn new_tag(s: &Sprite, name: &str, from: u32, to: u32) -> Option<Op> {
    let (from, to) = (from.min(to), from.max(to));
    if to as usize >= s.frames().len() {
        return None;
    }
    let mut tags = s.tags().to_vec();
    tags.push(Tag {
        name: name.to_owned(),
        from,
        to,
        direction: Direction::Forward,
        repeat: 0,
        color: [0, 0, 0, 255],
    });
    Some(op(s.frames().to_vec(), Vec::new(), tags))
}

/// Tag Properties: muda a tag `i` (o intervalo crava-se à linha do tempo).
#[must_use]
pub fn edit_tag(s: &Sprite, i: usize, f: impl FnOnce(&mut Tag)) -> Option<Op> {
    let mut tags = s.tags().to_vec();
    let before = tags.get(i)?.clone();
    let t = &mut tags[i];
    f(t);
    let last = s.frames().len() as u32 - 1;
    (t.from, t.to) = (t.from.min(t.to).min(last), t.from.max(t.to).min(last));
    (*t != before).then(|| op(s.frames().to_vec(), Vec::new(), tags))
}

/// Frame ▸ Tags ▸ Delete Tag.
#[must_use]
pub fn delete_tag(s: &Sprite, i: usize) -> Option<Op> {
    let mut tags = s.tags().to_vec();
    (i < tags.len()).then(|| {
        tags.remove(i);
        op(s.frames().to_vec(), Vec::new(), tags)
    })
}
