//! **As camadas** (P3) do lado da família: a lista que o painel PIXEL mostra, a camada activa, e o
//! menu Layer do Aseprite (nova, grupo, apagar, mover, duplicar, Merge Down, Flatten, Layer via
//! Copy/Cut, colar como camada). Tudo o que muda o documento é UM passo do desfazer do sprite
//! (`ph2d_pixel_core::layers`).

use ph2d_editor_core::pixel_panel::{PixelBlend, PixelLayerAction, PixelLayerRow, PixelRequest};
use ph2d_pixel_core::blend::BlendMode;
use ph2d_pixel_core::layers::{self, Place};
use ph2d_pixel_core::{LayerId, LayerKind, Op, Sprite};

use crate::{Doc, PixelApp};

/// O modo do núcleo no vocabulário do chrome (os mesmos 19, na mesma ordem).
#[must_use]
pub fn blend_of(m: BlendMode) -> PixelBlend {
    PixelBlend::ALL[usize::from(m.code())]
}

#[must_use]
pub fn blend_to(b: PixelBlend) -> BlendMode {
    let i = PixelBlend::ALL.iter().position(|&x| x == b).unwrap_or(0);
    BlendMode::ALL[i]
}

/// Porque não se pinta na camada activa (a chave i18n do aviso), ou `None`.
#[must_use]
pub fn recusa_pintar(doc: &Doc) -> Option<&'static str> {
    let l = doc.sprite.layer(doc.layer)?;
    if l.is_group() {
        Some("pixel.layer.refuse.group")
    } else if l.locked {
        Some("pixel.layer.refuse.locked")
    } else if !doc.sprite.is_shown(l.id) {
        Some("pixel.layer.refuse.hidden")
    } else {
        None
    }
}

/// As linhas da lista, de CIMA para baixo; os filhos de um grupo recolhido não aparecem.
#[must_use]
pub fn rows(s: &Sprite) -> Vec<PixelLayerRow> {
    let mut out = Vec::new();
    push_children(s, None, 0, &mut out);
    out
}

fn push_children(s: &Sprite, parent: Option<LayerId>, depth: u8, out: &mut Vec<PixelLayerRow>) {
    for l in s.layers().iter().rev().filter(|l| l.parent == parent) {
        let collapsed = matches!(l.kind, LayerKind::Group { collapsed: true });
        out.push(PixelLayerRow {
            id: l.id.0,
            name: l.name.clone(),
            depth,
            group: l.is_group(),
            collapsed,
            visible: l.visible,
            locked: l.locked,
            reference: l.reference,
            background: l.background,
            opacity: l.opacity,
            blend: blend_of(l.blend),
        });
        if l.is_group() && !collapsed {
            push_children(s, Some(l.id), depth.saturating_add(1), out);
        }
    }
}

/// A camada activa tem de existir: depois de um desfazer que a tirou, a de imagem mais alta.
pub(crate) fn sane_active(doc: &mut Doc) {
    if doc.sprite.layer(doc.layer).is_none() {
        doc.layer = doc
            .sprite
            .layers()
            .iter()
            .rev()
            .find(|l| !l.is_group())
            .map_or(doc.layer, |l| l.id);
    }
}

/// O irmão logo acima/abaixo de `id` (o degrau do ↑/↓).
fn sibling(s: &Sprite, id: LayerId, up: bool) -> Option<LayerId> {
    let me = s.layer(id)?;
    let irmaos: Vec<LayerId> = s
        .layers()
        .iter()
        .filter(|l| l.parent == me.parent)
        .map(|l| l.id)
        .collect();
    let i = irmaos.iter().position(|&x| x == id)?;
    if up {
        irmaos.get(i + 1).copied()
    } else {
        i.checked_sub(1).and_then(|j| irmaos.get(j).copied())
    }
}

impl PixelApp {
    /// Os pedidos das camadas; `false` = não é de camada (segue para a paleta).
    pub(crate) fn aplica_camada(&mut self, r: &PixelRequest) -> bool {
        let layer_request = matches!(
            r,
            PixelRequest::LayerSelect(_)
                | PixelRequest::LayerVisible(_)
                | PixelRequest::LayerLocked(_)
                | PixelRequest::LayerCollapsed(_)
                | PixelRequest::LayerSolo(_)
                | PixelRequest::LayerOpacity(_)
                | PixelRequest::LayerBlend(_)
                | PixelRequest::LayerAction(_)
                | PixelRequest::LayerRename { .. }
        );
        if !layer_request {
            return false;
        }
        let Some(key) = self.chave else {
            return true;
        };
        if let PixelRequest::LayerSolo(id) = r {
            if let Some(op) = self.solo_op(key, LayerId(*id)) {
                let _ = self.commit(key, op);
            }
            return true;
        }
        let Some(doc) = self.docs.get_mut(&key) else {
            return true;
        };
        let active = doc.layer;
        let frame = doc.frame;
        let s = &mut doc.sprite;
        let mut novo_activo = None;
        let op: Option<Op> = match r {
            PixelRequest::LayerSelect(id) => {
                if s.layer(LayerId(*id)).is_some() {
                    doc.layer = LayerId(*id);
                    self.solo = self.solo.take().filter(|(k, ..)| *k == key);
                }
                None
            }
            PixelRequest::LayerVisible(id) => {
                layers::edit(s, LayerId(*id), |l| l.visible = !l.visible)
            }
            PixelRequest::LayerLocked(id) => {
                layers::edit(s, LayerId(*id), |l| l.locked = !l.locked)
            }
            PixelRequest::LayerCollapsed(id) => layers::edit(s, LayerId(*id), |l| {
                if let LayerKind::Group { collapsed } = &mut l.kind {
                    *collapsed = !*collapsed;
                }
            }),
            PixelRequest::LayerOpacity(o) => layers::edit(s, active, |l| l.opacity = *o),
            PixelRequest::LayerBlend(b) => layers::edit(s, active, |l| l.blend = blend_to(*b)),
            PixelRequest::LayerRename { id, name } => {
                let name = name.trim();
                (!name.is_empty())
                    .then(|| layers::edit(s, LayerId(*id), |l| name.clone_into(&mut l.name)))
                    .flatten()
            }
            PixelRequest::LayerAction(PixelLayerAction::ViaCopy | PixelLayerAction::ViaCut)
                if s.mask().is_none() =>
            {
                self.avisos.push((
                    true,
                    ph2d_i18n::tr("pixel.layer.refuse.selection").to_owned(),
                ));
                None
            }
            PixelRequest::LayerAction(a) => {
                let bg = s.value_of_rgba(self.bg);
                let clip = self.clip.as_ref().map(|p| &p.clip);
                let (op, nova) = acao(s, *a, active, frame, clip, bg);
                novo_activo = nova;
                if op.is_none() {
                    self.avisos
                        .push((true, ph2d_i18n::tr("pixel.layer.refuse.action").to_owned()));
                }
                op
            }
            _ => None,
        };
        if let Some(op) = op
            && self.commit(key, op).is_ok()
            && let Some(doc) = self.docs.get_mut(&key)
        {
            if let Some(n) = novo_activo {
                doc.layer = n;
            }
            sane_active(doc);
        }
        true
    }

    /// *Solo*: só `id` (e os grupos que a contêm) se vêem; o segundo Solo na mesma camada devolve a
    /// visibilidade que havia. Um passo do desfazer, como qualquer olho.
    fn solo_op(&mut self, key: u32, id: LayerId) -> Option<Op> {
        let s = &self.docs.get(&key)?.sprite;
        if let Some((k, alvo, antes)) = self.solo.take()
            && k == key
            && alvo == id
        {
            let mut layers = s.layers().to_vec();
            for l in &mut layers {
                if let Some((_, v)) = antes.iter().find(|(x, _)| *x == l.id) {
                    l.visible = *v;
                }
            }
            return Some(Op::Layers(Box::new(ph2d_pixel_core::LayerSet {
                layers,
                cels: Vec::new(),
            })));
        }
        let antes: Vec<(LayerId, bool)> = s.layers().iter().map(|l| (l.id, l.visible)).collect();
        let mut keep = vec![id];
        let mut p = s.layer(id)?.parent;
        while let Some(g) = p {
            keep.push(g);
            p = s.layer(g).and_then(|l| l.parent);
        }
        // os descendentes de um grupo em solo também se vêem
        let mut i = 0;
        while i < keep.len() {
            let g = keep[i];
            keep.extend(
                s.layers()
                    .iter()
                    .filter(|l| l.parent == Some(g) && !keep.contains(&l.id))
                    .map(|l| l.id)
                    .collect::<Vec<_>>(),
            );
            i += 1;
        }
        let mut layers = s.layers().to_vec();
        for l in &mut layers {
            l.visible = keep.contains(&l.id);
        }
        self.solo = Some((key, id, antes));
        Some(Op::Layers(Box::new(ph2d_pixel_core::LayerSet {
            layers,
            cels: Vec::new(),
        })))
    }
}

/// Uma acção do menu Layer: o op e a camada que fica activa.
fn acao(
    s: &mut Sprite,
    a: PixelLayerAction,
    active: LayerId,
    frame: ph2d_pixel_core::FrameId,
    clip: Option<&layers::Clip>,
    bg: u32,
) -> (Option<Op>, Option<LayerId>) {
    let par = |r: Option<(Op, LayerId)>| match r {
        Some((op, id)) => (Some(op), Some(id)),
        None => (None, None),
    };
    match a {
        PixelLayerAction::NewLayer => par(layers::new_layer(s, active)),
        PixelLayerAction::NewGroup => par(layers::new_group(s, active)),
        PixelLayerAction::Delete => {
            // a activa passa à vizinha de baixo (ou de cima), como no Aseprite
            let next = sibling(s, active, false).or_else(|| sibling(s, active, true));
            (layers::delete(s, active), next)
        }
        PixelLayerAction::Duplicate => par(layers::duplicate(s, active)),
        PixelLayerAction::MoveUp => {
            let op =
                sibling(s, active, true).and_then(|t| layers::move_to(s, active, Place::Above(t)));
            (op, None)
        }
        PixelLayerAction::MoveDown => {
            let op =
                sibling(s, active, false).and_then(|t| layers::move_to(s, active, Place::Below(t)));
            (op, None)
        }
        PixelLayerAction::MergeDown => {
            let below = sibling(s, active, false);
            (layers::merge_down(s, active), below)
        }
        PixelLayerAction::Flatten | PixelLayerAction::FlattenVisible => {
            let op = layers::flatten(s, a == PixelLayerAction::FlattenVisible);
            let flat = op.as_ref().and_then(|op| match op {
                Op::Layers(set) => set
                    .layers
                    .iter()
                    .find(|l| !s.layers().iter().any(|o| o.id == l.id) || l.background)
                    .map(|l| l.id),
                _ => None,
            });
            (op, flat)
        }
        PixelLayerAction::ViaCopy | PixelLayerAction::ViaCut => par(via_selection(
            s,
            active,
            frame,
            a == PixelLayerAction::ViaCut,
            bg,
        )),
        PixelLayerAction::PasteAsLayer | PixelLayerAction::PasteAsReference => {
            let reference = a == PixelLayerAction::PasteAsReference;
            match clip {
                Some(c) => par(layers::paste_as_new(s, active, frame, c, reference)),
                None => (None, None),
            }
        }
    }
}

/// *Layer via Copy/Cut* com a selecção de VERDADE (P6): só as células seleccionadas vão para a
/// camada nova (as outras ficam transparentes nela); no Cut o original perde-as (Background: a cor
/// de fundo). UM passo — o Cut é um lote (camada nova + buraco), os dois contra o estado de antes.
fn via_selection(
    s: &mut Sprite,
    active: LayerId,
    frame: ph2d_pixel_core::FrameId,
    cut: bool,
    bg: u32,
) -> Option<(Op, LayerId)> {
    use ph2d_pixel_core::transform::{Floating, Image, lift, stamp};
    if s.layer(active)?.is_group() {
        return None;
    }
    let key = (active, frame);
    let m = s.mask()?.clone();
    let f = lift(s, key, Some(&m))?;
    let clip = layers::Clip {
        mode: s.mode(),
        palette: s.palette().to_vec(),
        transparent: s.transparent_index(),
        rect: f.rect(),
        data: f.image.data,
    };
    let buraco = if cut {
        let nada = Floating {
            x: 0,
            y: 0,
            image: Image::new(0, 0, s.mode().bpp(), s.transparent_value()),
            keep: Vec::new(),
        };
        stamp(s, key, &nada, Some(&m), None, bg)
    } else {
        None
    };
    let (op, id) = layers::paste_as_new(s, active, frame, &clip, false)?;
    let op = match buraco {
        Some(b) => Op::Batch(vec![b, op]),
        None => op,
    };
    Some((op, id))
}
