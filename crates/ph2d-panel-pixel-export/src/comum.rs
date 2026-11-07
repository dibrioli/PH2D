//! O que as três janelas partilham: as listas suspensas e a imagem num cartão.

use ph2d_a11y::NodeId;
use ph2d_editor_core::paint::{fill_rounded_rect, rect_to_vello};
use ph2d_editor_core::panel::cards::{CardCtx, Cell, Seg, Widget};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_pixel_io::abrir::Grelha;
use ph2d_pixel_io::render::{Area, Camadas, Quadros};
use ph2d_pixel_io::saida::Formato;
use ph2d_tokens::ColorToken;
use std::sync::{Arc, Mutex};

use crate::canal::Contexto;
use crate::ids;

pub(crate) const FORMATOS: [Formato; 5] = [
    Formato::Png,
    Formato::Gif,
    Formato::Apng,
    Formato::Webp,
    Formato::Ase,
];

/// A altura da caixa da imagem, em larguras do cartão: paisagem, para a folha caber sem empurrar os
/// outros cartões para fora da janela.
const ALTURA_DA_IMAGEM: f32 = 0.625; // LITERAL-PX-OK: razão altura/largura, não píxeis

/// As linhas de uma lista: `(id, texto, acesa)`.
pub(crate) type Linhas = Vec<(NodeId, String, bool)>;

/// A linha acesa de uma lista.
pub(crate) fn texto_aceso(l: &Linhas) -> &str {
    l.iter()
        .find(|x| x.2)
        .or(l.first())
        .map_or("", |x| x.1.as_str())
}

pub(crate) fn segs(l: &Linhas) -> Vec<Seg<'_>> {
    l.iter()
        .map(|(id, t, on)| Seg {
            id: *id,
            icon: None,
            text: t,
            on: *on,
        })
        .collect()
}

/// Uma ou duas listas na mesma linha.
pub(crate) fn listas(
    c: &mut CardCtx<'_>,
    a: (NodeId, &str, &Linhas),
    b: Option<(NodeId, &str, &Linhas)>,
    y: f32,
) -> f32 {
    let (sa, sb) = (segs(a.2), b.map(|b| segs(b.2)));
    let ca = Cell::new(Widget::Dropdown {
        id: a.0,
        text: texto_aceso(a.2),
        options: &sa,
    })
    .label(a.1);
    match (b, &sb) {
        (Some(b), Some(s)) => {
            let cb = Cell::new(Widget::Dropdown {
                id: b.0,
                text: texto_aceso(b.2),
                options: s,
            })
            .label(b.1);
            c.row(&[ca, cb], y)
        }
        _ => c.row(&[ca], y),
    }
}

pub(crate) fn linhas_de_camadas(x: &Contexto, c: Camadas) -> Linhas {
    let mut v = vec![
        (
            ids::LAYERS_VISIBLE,
            tr("panel.pixel.export.layers_visible").to_owned(),
            c == Camadas::Visiveis,
        ),
        (
            ids::LAYERS_ALL,
            tr("panel.pixel.export.layers_all").to_owned(),
            c == Camadas::Todas,
        ),
    ];
    for (i, (id, nome)) in x.camadas.iter().enumerate() {
        v.push((ids::layer_option(i), nome.clone(), c == Camadas::Uma(*id)));
    }
    v
}

pub(crate) fn linhas_de_quadros(x: &Contexto, q: Quadros) -> Linhas {
    let mut v = vec![(
        ids::FRAMES_ALL,
        tr("panel.pixel.export.frames_all").to_owned(),
        q == Quadros::Todos,
    )];
    for (i, nome) in x.tags.iter().enumerate() {
        v.push((ids::tag_option(i), nome.clone(), q == Quadros::Tag(i)));
    }
    v.push((
        ids::FRAMES_RANGE,
        tr("panel.pixel.export.frames_range").to_owned(),
        matches!(q, Quadros::Intervalo { .. }),
    ));
    v
}

/// A lista Area: o canvas e as slices por nome, gémeas incluídas (F4).
pub(crate) fn linhas_de_area(x: &Contexto, a: Area) -> Linhas {
    let mut v = vec![(
        ids::AREA_CANVAS,
        tr("panel.pixel.export.area_canvas").to_owned(),
        a == Area::Canvas,
    )];
    for (i, (id, nome)) in x.slices.iter().enumerate() {
        v.push((ids::area_option(i), nome.clone(), a == Area::Slice(*id)));
    }
    v
}

/// O que um clique na lista Area escolheu.
pub(crate) fn area_de(x: &Contexto, id: NodeId) -> Option<Area> {
    if id == ids::AREA_CANVAS {
        return Some(Area::Canvas);
    }
    let i = (0..x.slices.len()).find(|&i| ids::area_option(i) == id)?;
    Some(Area::Slice(x.slices[i].0))
}

/// O que um clique numa lista de camadas escolheu.
pub(crate) fn camada_de(x: &Contexto, id: NodeId) -> Option<Camadas> {
    if id == ids::LAYERS_VISIBLE {
        return Some(Camadas::Visiveis);
    }
    if id == ids::LAYERS_ALL {
        return Some(Camadas::Todas);
    }
    let i = (0..x.camadas.len()).find(|&i| ids::layer_option(i) == id)?;
    Some(Camadas::Uma(x.camadas[i].0))
}

/// O que um clique numa lista de quadros escolheu; o intervalo começa no sprite inteiro.
pub(crate) fn quadro_de(x: &Contexto, id: NodeId) -> Option<Quadros> {
    if id == ids::FRAMES_ALL {
        return Some(Quadros::Todos);
    }
    if id == ids::FRAMES_RANGE {
        return Some(Quadros::Intervalo {
            de: 0,
            ate: (x.quadros as usize).saturating_sub(1),
        });
    }
    (0..x.tags.len())
        .find(|&i| ids::tag_option(i) == id)
        .map(Quadros::Tag)
}

/// A imagem pronta para um cartão.
pub(crate) struct ImagemDoCartao {
    pub img: ph2d_vector::StableImage,
    pub lados: (u32, u32),
    pub xadrez: Arc<Mutex<ph2d_pixel_render::Checker>>,
}

/// Pinta a imagem no cartão, inteira, sobre o xadrez, com a grelha por cima; devolve o fundo da caixa.
/// A imagem nítida: vizinho mais próximo, como a Preview.
pub(crate) fn imagem_no_cartao(
    c: &mut CardCtx<'_>,
    i: &ImagemDoCartao,
    grelha: Option<(Grelha, (u32, u32))>,
    y: f32,
) -> f32 {
    let caixa = Rect::new(c.x, y, c.w, c.w * ALTURA_DA_IMAGEM);
    #[expect(clippy::cast_precision_loss, reason = "lados de imagem ≤ 65 535")]
    let (w, h) = (i.lados.0 as f32, i.lados.1 as f32);
    let e = (caixa.w / w.max(1.0)).min(caixa.h / h.max(1.0));
    let (dw, dh) = (w * e, h * e);
    let (x, y0) = (
        caixa.x + (caixa.w - dw) * 0.5,
        caixa.y + (caixa.h - dh) * 0.5,
    );
    c.scene.push_clip(&rect_to_vello(caixa));
    let to_screen = ph2d_vector::Affine::translate((f64::from(x), f64::from(y0)))
        * ph2d_vector::Affine::scale(f64::from(e));
    if let Ok(mut x) = i.xadrez.lock() {
        x.draw_under(c.scene, i.lados.0, i.lados.1, to_screen);
    }
    let dest = (
        f64::from(x),
        f64::from(y0),
        f64::from(x + dw),
        f64::from(y0 + dh),
    );
    c.scene
        .draw_stable_image(&i.img, dest, ph2d_vector::ImageQuality::Low);
    if let Some((g, (iw, ih))) = grelha {
        grade(c, g, (iw, ih), (x, y0, e));
    }
    c.scene.pop_layer();
    caixa.y + caixa.h
}

/// O contorno de cada célula inteira da grelha, por cima da imagem.
fn grade(c: &mut CardCtx<'_>, g: Grelha, (iw, ih): (u32, u32), (x, y, e): (f32, f32, f32)) {
    let (cols, rows) = ph2d_pixel_io::abrir::celulas(&g, iw, ih);
    let cor = c.ink.color(ColorToken::Accent);
    #[expect(clippy::cast_precision_loss, reason = "coordenadas de imagem ≤ 65 535")]
    let px = |n: u32| n as f32 * e;
    let traco = ph2d_tokens::StrokeToken::Thin.px();
    for r in 0..rows {
        for k in 0..cols {
            let cx = x + px(g.x + k * (g.w + g.gap_x));
            let cy = y + px(g.y + r * (g.h + g.gap_y));
            let (cw, ch) = (px(g.w), px(g.h));
            for lado in [
                Rect::new(cx, cy, cw, traco),
                Rect::new(cx, cy + ch - traco, cw, traco),
                Rect::new(cx, cy, traco, ch),
                Rect::new(cx + cw - traco, cy, traco, ch),
            ] {
                fill_rounded_rect(c.scene, lado, 0.0, cor);
            }
        }
    }
}
