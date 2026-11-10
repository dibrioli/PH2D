//! **As camadas do Pixel Art no vocabulário do chrome**: o modo de mistura, as linhas da lista e o
//! irmão de uma camada. Saiu de `ph2d-app-pixel` (P12): nada aqui vê o estado da família.

use ph2d_editor_core::pixel_panel::{PixelBlend, PixelLayerEffect, PixelLayerRow, PixelOutline};
use ph2d_pixel_core::blend::BlendMode;
use ph2d_pixel_core::filter::outline::{Background, CIRCLE, Outline, Place, SQUARE};
use ph2d_pixel_core::{DocOp, EfeitoDeCamada, Layer, LayerId, LayerKind, Op, Sprite};

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
            normal: l.normal,
            reference: l.reference,
            background: l.background,
            opacity: l.opacity,
            blend: blend_of(l.blend),
            outline: outline_of(l),
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

/// P13: o Outline da camada no vocabulário do chrome (o primeiro efeito; hoje só há este).
#[must_use]
pub fn outline_of(l: &Layer) -> Option<PixelOutline> {
    l.efeitos.iter().find_map(|EfeitoDeCamada::Outline(o)| {
        Some(PixelOutline {
            color: o.color,
            inside: o.place == Place::Inside,
            square: o.matrix == SQUARE,
        })
    })
}

/// P13: Layer Properties ▸ Effects ⇒ a Op de UM passo (`SetLayerEffects`, ou o `Batch` do Apply).
/// `None` = nada muda (camada inexistente, grupo, tilemap, ou o pedido já é o estado).
#[must_use]
pub fn efeito_op(s: &Sprite, id: LayerId, e: PixelLayerEffect) -> Option<Op> {
    let l = s
        .layer(id)
        .filter(|l| !l.is_group() && l.tileset().is_none())?;
    if e == PixelLayerEffect::Apply {
        return ph2d_pixel_core::aplicar_efeitos(s, id);
    }
    let atual = outline_of(l);
    let novo = match e {
        PixelLayerEffect::On(color) => atual.or(Some(PixelOutline {
            color,
            ..Default::default()
        })),
        PixelLayerEffect::Off | PixelLayerEffect::Apply => None,
        PixelLayerEffect::Inside(v) => atual.map(|o| PixelOutline { inside: v, ..o }),
        PixelLayerEffect::Square(v) => atual.map(|o| PixelOutline { square: v, ..o }),
        PixelLayerEffect::Color(c) => atual.map(|o| PixelOutline { color: c, ..o }),
    };
    let efeitos: Vec<EfeitoDeCamada> = novo
        .map(|o| {
            EfeitoDeCamada::Outline(Outline {
                color: o.color,
                background: Background::Mask,
                matrix: if o.square { SQUARE } else { CIRCLE },
                place: if o.inside {
                    Place::Inside
                } else {
                    Place::Outside
                },
                tiled: false,
            })
        })
        .into_iter()
        .collect();
    (efeitos != l.efeitos).then(|| Op::Doc(Box::new(DocOp::SetLayerEffects { layer: id, efeitos })))
}
