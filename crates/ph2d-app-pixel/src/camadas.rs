//! **As camadas** (P3) do lado da família: a lista que o painel PIXEL mostra, a camada activa, e o
//! menu Layer do Aseprite (nova, grupo, apagar, mover, duplicar, Merge Down, Flatten, Layer via
//! Copy/Cut, colar como camada). Tudo o que muda o documento é UM passo do desfazer do sprite
//! (`ph2d_pixel_core::layers`).

use ph2d_editor_core::pixel_panel::{PixelLayerAction, PixelRequest};
use ph2d_pixel_core::layers::{self, Place};
use ph2d_pixel_core::{LayerId, LayerKind, Op, Sprite};

use crate::{Doc, PixelApp};
use ph2d_pixel_ponte::leis::via_selection;

use ph2d_pixel_ponte::camadas::sibling;
pub use ph2d_pixel_ponte::camadas::{blend_of, blend_to, rows};

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
    } else if l.tileset().is_some() {
        Some("pixel.layer.refuse.tilemap")
    } else {
        None
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
            PixelRequest::LayerAction(PixelLayerAction::PasteAsSprite) => {
                match self.clip.as_ref() {
                    Some(p) => self.nasce = Some(p.clip.clone()),
                    None => self
                        .avisos
                        .push((true, ph2d_i18n::tr("pixel.layer.refuse.action").to_owned())),
                }
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
        // Saem antes, em `crate::tiles::aplica_pedido`.
        PixelLayerAction::NewTilemap
        | PixelLayerAction::ConvertToTilemap
        | PixelLayerAction::ConvertToImage => (None, None),
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
        PixelLayerAction::PasteAsSprite => (None, None),
    }
}

/// O *Auto Select Layer*: a camada activa passa a ser a de cima, visível e de imagem, com uma
/// célula pintada em `at` neste frame; sem nenhuma, fica a que estava.
pub(crate) fn escolhe_sob(app: &mut PixelApp, key: u32, at: (i32, i32)) {
    let Some(doc) = app.docs.get_mut(&key) else {
        return;
    };
    let s = &doc.sprite;
    let vazio = s.transparent_value();
    let pintada = |l: LayerId| {
        let v = s.value_at((l, doc.frame), at.0, at.1);
        match s.mode() {
            ph2d_pixel_core::ColorMode::Indexed => v != vazio,
            _ => s.rgba_of_value(v)[3] != 0,
        }
    };
    let sob = s
        .layers()
        .iter()
        .rev()
        .filter(|l| !l.is_group() && s.is_shown(l.id))
        .map(|l| l.id)
        .find(|&l| s.bounds().contains(at.0, at.1) && pintada(l));
    if let Some(l) = sob {
        doc.layer = l;
    }
}
