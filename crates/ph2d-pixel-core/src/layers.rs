//! As ações do menu Layer do Aseprite, cada uma UM [`Op::Layers`] (um gesto = um passo de undo).
//!
//! Devolvem `None` quando a ação não se aplica (o Aseprite desliga o item): apagar a última camada
//! de imagem, Merge Down sem vizinha de imagem por baixo, etc.

use crate::blend::{BlendMode, blend};
use crate::compose::{Purpose, compose_window};
use crate::op::{LayerSet, Op};
use crate::sprite::{Cel, CelKey, ColorMode, FrameId, Layer, LayerId, LayerKind, Rect, Sprite};

/// Onde uma camada vai parar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Place {
    /// Logo acima de `LayerId`, no mesmo grupo.
    Above(LayerId),
    /// Logo abaixo, no mesmo grupo.
    Below(LayerId),
    /// No topo do grupo.
    Into(LayerId),
}

/// Uma imagem copiada (Edit ▸ Copy): os valores no modo e na paleta de onde saiu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Clip {
    pub mode: ColorMode,
    pub palette: Vec<[u8; 4]>,
    pub transparent: u8,
    /// Onde estava no sprite de origem (o Paste volta a pô-lo aí).
    pub rect: Rect,
    pub data: Vec<u8>,
}

fn set(layers: Vec<Layer>, cels: Vec<(CelKey, Option<Cel>)>) -> Op {
    Op::Layers(Box::new(LayerSet { layers, cels }))
}

/// O id e os descendentes de `id`, na ordem do `Vec`.
fn subtree(layers: &[Layer], id: LayerId) -> Vec<LayerId> {
    let mut out = vec![id];
    let mut i = 0;
    while i < out.len() {
        let p = out[i];
        out.extend(layers.iter().filter(|l| l.parent == Some(p)).map(|l| l.id));
        i += 1;
    }
    layers
        .iter()
        .map(|l| l.id)
        .filter(|x| out.contains(x))
        .collect()
}

fn index(layers: &[Layer], id: LayerId) -> Option<usize> {
    layers.iter().position(|l| l.id == id)
}

/// O primeiro número livre de «`prefix` N» (Aseprite: Layer 2, Layer 3…).
fn fresh_name(s: &Sprite, prefix: &str) -> String {
    let n = s
        .layers()
        .iter()
        .filter_map(|l| l.name.strip_prefix(prefix)?.trim().parse::<u32>().ok())
        .max()
        .unwrap_or(0)
        .max(s.layers().len() as u32);
    format!("{prefix} {}", n + 1)
}

/// Insere `new` em `place`; `None` se o alvo não existir ou não for grupo (para `Into`).
fn insert_at(mut layers: Vec<Layer>, mut new: Vec<Layer>, place: Place) -> Option<Vec<Layer>> {
    let (at, parent) = match place {
        Place::Above(id) => {
            let i = index(&layers, id)?;
            // acima da camada E dos seus descendentes (um grupo leva-os consigo)
            let end = subtree(&layers, id)
                .iter()
                .filter_map(|x| index(&layers, *x))
                .max()?;
            (end.max(i) + 1, layers[i].parent)
        }
        Place::Below(id) => {
            let i = index(&layers, id)?;
            let start = subtree(&layers, id)
                .iter()
                .filter_map(|x| index(&layers, *x))
                .min()?;
            (start.min(i), layers[i].parent)
        }
        Place::Into(g) => {
            let i = index(&layers, g)?;
            if !layers[i].is_group() {
                return None;
            }
            let end = subtree(&layers, g)
                .iter()
                .filter_map(|x| index(&layers, *x))
                .max()?;
            (end + 1, Some(g))
        }
    };
    let root = new.first()?.id;
    for l in &mut new {
        if l.id == root {
            l.parent = parent;
        }
    }
    // a Background é sempre a primeira: ninguém entra por baixo dela
    let at = if layers.first().is_some_and(|l| l.background) {
        at.max(1)
    } else {
        at
    };
    layers.splice(at..at, new);
    Some(layers)
}

/// Layer ▸ New ▸ New Layer: imagem vazia acima de `above`, no mesmo grupo.
pub fn new_layer(s: &mut Sprite, above: LayerId) -> Option<(Op, LayerId)> {
    let id = LayerId(s.reserve_id());
    let layer = Layer::image(id, fresh_name(s, "Layer"));
    let layers = insert_at(s.layers().to_vec(), vec![layer], Place::Above(above))?;
    Some((set(layers, Vec::new()), id))
}

/// Layer ▸ New ▸ New Group.
pub fn new_group(s: &mut Sprite, above: LayerId) -> Option<(Op, LayerId)> {
    let id = LayerId(s.reserve_id());
    let mut g = Layer::image(id, fresh_name(s, "Group"));
    g.kind = LayerKind::Group { collapsed: false };
    let layers = insert_at(s.layers().to_vec(), vec![g], Place::Above(above))?;
    Some((set(layers, Vec::new()), id))
}

fn cels_of(s: &Sprite, ids: &[LayerId]) -> Vec<(CelKey, Option<Cel>)> {
    s.frames()
        .iter()
        .flat_map(|f| ids.iter().map(move |&l| (l, f.id)))
        .filter(|k| s.entry(*k).is_some())
        .map(|k| (k, None))
        .collect()
}

/// Layer ▸ Delete Layer (com os descendentes de um grupo e os cels de todos os frames).
#[must_use]
pub fn delete(s: &Sprite, id: LayerId) -> Option<Op> {
    let gone = subtree(s.layers(), id);
    let layers: Vec<Layer> = s
        .layers()
        .iter()
        .filter(|l| !gone.contains(&l.id))
        .cloned()
        .collect();
    if !layers.iter().any(|l| !l.is_group()) {
        return None;
    }
    Some(set(layers, cels_of(s, &gone)))
}

/// Uma propriedade (nome, visível, trancada, opacidade, modo, recolhido…). `None` se nada mudar
/// ou se a mudança violar a lei (o id não muda por aqui; a Background não vira grupo).
#[must_use]
pub fn edit(s: &Sprite, id: LayerId, f: impl FnOnce(&mut Layer)) -> Option<Op> {
    let mut layers = s.layers().to_vec();
    let i = index(&layers, id)?;
    let before = layers[i].clone();
    f(&mut layers[i]);
    layers[i].id = before.id;
    layers[i].parent = before.parent;
    layers[i].kind = match (before.kind, layers[i].kind) {
        (LayerKind::Group { .. }, k @ LayerKind::Group { .. }) => k,
        (k, _) => k,
    };
    (layers[i] != before).then(|| set(layers, Vec::new()))
}

/// Mover (arrastar na lista). `None` se o destino estiver dentro do próprio grupo que se move, ou
/// se a Background saísse do fundo.
#[must_use]
pub fn move_to(s: &Sprite, id: LayerId, place: Place) -> Option<Op> {
    let moving = subtree(s.layers(), id);
    let target = match place {
        Place::Above(t) | Place::Below(t) | Place::Into(t) => t,
    };
    if moving.contains(&target) || s.layer(id)?.background {
        return None;
    }
    let (kept, block): (Vec<Layer>, Vec<Layer>) = s
        .layers()
        .iter()
        .cloned()
        .partition(|l| !moving.contains(&l.id));
    let layers = insert_at(kept, block, place)?;
    (layers.as_slice() != s.layers()).then(|| set(layers, Vec::new()))
}

/// Layer ▸ Duplicate: a cópia (com os descendentes e os cels) logo acima, «nome Copy».
pub fn duplicate(s: &mut Sprite, id: LayerId) -> Option<(Op, LayerId)> {
    let tree = subtree(s.layers(), id);
    let mut map = Vec::new();
    for &old in &tree {
        map.push((old, LayerId(s.reserve_id())));
    }
    let new_of = |x: LayerId| map.iter().find(|(o, _)| *o == x).map(|(_, n)| *n);
    let mut copies = Vec::new();
    let mut cels = Vec::new();
    for &old in &tree {
        let mut l = s.layer(old)?.clone();
        l.id = new_of(old)?;
        l.parent = l.parent.and_then(new_of).or(l.parent);
        l.background = false;
        if old == id {
            l.name = format!("{} Copy", l.name);
        }
        for f in s.frames() {
            if let Some(c) = s.entry((old, f.id)) {
                cels.push(((l.id, f.id), Some(c.clone())));
            }
        }
        copies.push(l);
    }
    let root = new_of(id)?;
    let layers = insert_at(s.layers().to_vec(), copies, Place::Above(id))?;
    Some((set(layers, cels), root))
}

/// O cel `(layer, frame)` como RGBA de alfa reto no retângulo `r`.
fn rgba_rect(s: &Sprite, key: CelKey, r: Rect) -> Vec<u8> {
    let mut out = Vec::with_capacity(r.area() * 4);
    for y in r.y..r.y + r.h as i32 {
        for x in r.x..r.x + r.w as i32 {
            out.extend_from_slice(&s.rgba_of_value(s.value_at(key, x, y)));
        }
    }
    out
}

/// RGBA → um cel no modo do sprite.
fn cel_from_rgba(s: &Sprite, r: Rect, rgba: &[u8]) -> Cel {
    let mut data = Vec::with_capacity(r.area() * s.mode().bpp());
    for px in rgba.chunks_exact(4) {
        let v = s.value_of_rgba([px[0], px[1], px[2], px[3]]);
        data.extend_from_slice(&v.to_le_bytes()[..s.mode().bpp()]);
    }
    Cel::image(r, data)
}

/// Layer ▸ Merge Down: a camada mistura-se (com o seu modo e opacidade) sobre a vizinha de baixo,
/// em cada frame; a de baixo fica com o nome e as propriedades dela.
#[must_use]
pub fn merge_down(s: &Sprite, id: LayerId) -> Option<Op> {
    let i = index(s.layers(), id)?;
    let top = &s.layers()[i];
    let below = s.layers()[..i]
        .iter()
        .rev()
        .find(|l| l.parent == top.parent)?;
    if top.is_group() || below.is_group() || top.reference || below.reference {
        return None;
    }
    let mut cels = cels_of(s, &[top.id]);
    for f in s.frames() {
        let (kt, kb) = ((top.id, f.id), (below.id, f.id));
        let area = match (s.cel(kt), s.cel(kb)) {
            (None, _) => {
                // Um ligado de baixo que fica: vira cópia, o dono dele pode ser refeito noutro frame.
                if let Some(e) = s.entry(kb).filter(|e| e.link.is_some())
                    && let Some(img) = s.cel(kb)
                {
                    cels.push((
                        kb,
                        Some(Cel {
                            z: e.z,
                            link: None,
                            ..img.clone()
                        }),
                    ));
                }
                continue;
            }
            (Some(t), None) => t.bbox,
            (Some(t), Some(b)) => t.bbox.union(b.bbox),
        }
        .intersect(s.bounds());
        let src = rgba_rect(s, kt, area);
        let mut dst = rgba_rect(s, kb, area);
        let cel_op = s.cel(kt).map_or(255, |c| c.opacity);
        let opacity = if top.visible {
            crate::blend::mul(u32::from(top.opacity), u32::from(cel_op)) as u8
        } else {
            0
        };
        for (d, t) in dst.chunks_exact_mut(4).zip(src.chunks_exact(4)) {
            let px = blend(
                top.blend,
                [d[0], d[1], d[2], d[3]],
                [t[0], t[1], t[2], t[3]],
                opacity,
            );
            d.copy_from_slice(&px);
        }
        let mut merged = cel_from_rgba(s, area, &dst);
        if let (Some(b), Some(e)) = (s.cel(kb), s.entry(kb)) {
            (merged.opacity, merged.z) = (b.opacity, e.z);
        }
        cels.push((kb, Some(merged)));
    }
    let layers = s
        .layers()
        .iter()
        .filter(|l| l.id != top.id)
        .cloned()
        .collect();
    Some(set(layers, cels))
}

/// Layer ▸ Flatten (`visible_only = false`) / Flatten Visible: as camadas (visíveis) compõem-se
/// numa só; com Background ela fica Background. As referências ficam de fora e mantêm-se.
pub fn flatten(s: &mut Sprite, visible_only: bool) -> Option<Op> {
    let merged: Vec<LayerId> = s
        .layers()
        .iter()
        .filter(|l| !l.reference && (!visible_only || s.is_shown(l.id)))
        .map(|l| l.id)
        .collect();
    if merged.is_empty() {
        return None;
    }
    let background = s
        .layers()
        .first()
        .filter(|l| l.background && merged.contains(&l.id));
    let mut flat = match background {
        Some(b) => b.clone(),
        None => Layer::image(LayerId(s.reserve_id()), "Flattened"),
    };
    flat.opacity = 255;
    flat.blend = BlendMode::Normal;
    flat.visible = true;
    // Flatten (todas) compõe também as escondidas, como se se vissem.
    let mut view = s.clone();
    for l in &mut view.layers {
        if !visible_only {
            l.visible = true;
        }
    }
    let mut cels = cels_of(s, &merged);
    for f in s.frames() {
        let rgba = compose_window(&view, f.id, None, s.bounds(), Purpose::Export);
        if rgba.chunks_exact(4).any(|p| p[3] != 0) {
            cels.push(((flat.id, f.id), Some(cel_from_rgba(s, s.bounds(), &rgba))));
        }
    }
    let mut layers: Vec<Layer> = s
        .layers()
        .iter()
        .filter(|l| !merged.contains(&l.id))
        .cloned()
        .collect();
    // grupos esvaziados saem; quem ficou dentro de um grupo fundido sobe à raiz
    for l in &mut layers {
        if l.parent.is_some_and(|p| merged.contains(&p)) {
            l.parent = None;
        }
    }
    // no fundo; acima de uma Background escondida que o Flatten Visible deixou
    let at = usize::from(layers.first().is_some_and(|l| l.background));
    layers.insert(at, flat);
    Some(set(layers, cels))
}

/// Layer ▸ New ▸ Layer via Copy / Layer via Cut: o retângulo `r` da camada `id` no frame vai para
/// uma camada nova logo acima; em Cut, o original fica com o buraco (a cor transparente).
pub fn via(
    s: &mut Sprite,
    id: LayerId,
    frame: FrameId,
    r: Rect,
    cut: bool,
) -> Option<(Op, LayerId)> {
    let src = s.layer(id)?;
    if src.is_group() {
        return None;
    }
    let r = r.intersect(s.bounds());
    let key = (id, frame);
    let bbox = s.cel(key)?.bbox.intersect(r);
    if bbox.is_empty() {
        return None;
    }
    let new = LayerId(s.reserve_id());
    let layer = Layer::image(new, fresh_name(s, "Layer"));
    let mut cels = vec![((new, frame), Some(Cel::image(bbox, s.read_rect(key, bbox))))];
    if cut {
        let old = s.cel(key)?;
        let mut hole = old.clone();
        let bpp = s.mode().bpp();
        // Background: o buraco é a cor de fundo do documento (o Aseprite pinta a cor de fundo);
        // aqui, o transparente do modo, menos na Background, que é opaca: fica igual.
        if !s.layer(id)?.background {
            let t = s.transparent_value().to_le_bytes();
            for y in bbox.y..bbox.y + bbox.h as i32 {
                for x in bbox.x..bbox.x + bbox.w as i32 {
                    let i = ((y - old.bbox.y) as usize * old.bbox.w as usize
                        + (x - old.bbox.x) as usize)
                        * bpp;
                    hole.data[i..i + bpp].copy_from_slice(&t[..bpp]);
                }
            }
            cels.push((s.owner(key), Some(hole)));
        }
    }
    let layers = insert_at(s.layers().to_vec(), vec![layer], Place::Above(id))?;
    Some((set(layers, cels), new))
}

/// Edit ▸ Copy: o retângulo `r` do cel (a camada inteira quando não há seleção).
#[must_use]
pub fn copy(s: &Sprite, key: CelKey, r: Rect) -> Option<Clip> {
    let r = s.cel(key)?.bbox.intersect(r);
    (!r.is_empty()).then(|| Clip {
        mode: s.mode(),
        palette: s.palette().to_vec(),
        transparent: s.transparent_index(),
        rect: r,
        data: s.read_rect(key, r),
    })
}

/// Edit ▸ Paste Special ▸ Paste as New Layer / Paste as New Reference Layer: o recorte numa
/// camada nova acima de `above`, no mesmo sítio de onde saiu (cortado ao sprite). Modo e paleta
/// iguais = os mesmos bytes; senão passa por RGBA para o modo do sprite.
pub fn paste_as_new(
    s: &mut Sprite,
    above: LayerId,
    frame: FrameId,
    clip: &Clip,
    reference: bool,
) -> Option<(Op, LayerId)> {
    let bbox = clip.rect.intersect(s.bounds());
    if bbox.is_empty() {
        return None;
    }
    let same = clip.mode == s.mode()
        && (clip.mode != ColorMode::Indexed
            || (clip.palette == s.palette() && clip.transparent == s.transparent_index()));
    let bpp_src = clip.mode.bpp();
    let mut data = Vec::with_capacity(bbox.area() * s.mode().bpp());
    for y in bbox.y..bbox.y + bbox.h as i32 {
        for x in bbox.x..bbox.x + bbox.w as i32 {
            let i = ((y - clip.rect.y) as usize * clip.rect.w as usize
                + (x - clip.rect.x) as usize)
                * bpp_src;
            let b = &clip.data[i..i + bpp_src];
            if same {
                data.extend_from_slice(b);
            } else {
                let rgba = clip_rgba(clip, b);
                let v = s.value_of_rgba(rgba);
                data.extend_from_slice(&v.to_le_bytes()[..s.mode().bpp()]);
            }
        }
    }
    let id = LayerId(s.reserve_id());
    let mut layer = Layer::image(id, fresh_name(s, "Layer"));
    layer.reference = reference;
    let layers = insert_at(s.layers().to_vec(), vec![layer], Place::Above(above))?;
    Some((
        set(layers, vec![((id, frame), Some(Cel::image(bbox, data)))]),
        id,
    ))
}

fn clip_rgba(c: &Clip, b: &[u8]) -> [u8; 4] {
    match c.mode {
        ColorMode::Rgba => [b[0], b[1], b[2], b[3]],
        ColorMode::Gray => [b[0], b[0], b[0], b[1]],
        ColorMode::Indexed => {
            if b[0] == c.transparent {
                [0; 4]
            } else {
                c.palette.get(usize::from(b[0])).copied().unwrap_or([0; 4])
            }
        }
    }
}
