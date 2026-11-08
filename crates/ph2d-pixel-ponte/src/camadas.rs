//! **As camadas do Pixel Art no vocabulário do chrome**: o modo de mistura, as linhas da lista e o
//! irmão de uma camada. Saiu de `ph2d-app-pixel` (P12): nada aqui vê o estado da família.

use ph2d_editor_core::pixel_panel::{PixelBlend, PixelLayerRow};
use ph2d_pixel_core::blend::BlendMode;
use ph2d_pixel_core::{LayerId, LayerKind, Sprite};

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
            continuous: l.continuous,
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

/// O irmão logo acima/abaixo de `id` (o degrau do ↑/↓).
pub fn sibling(s: &Sprite, id: LayerId, up: bool) -> Option<LayerId> {
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
