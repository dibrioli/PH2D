//! **O DESENHO É O GLIFO** — a porta única do ícone de um `IconButton` autorado.
//!
//! O porquê e a história: `docs/Vector Module/50_cabecalhos_do_app_vec.md` § `widget_icon.rs`.

use ph2d_vec_render::build_bezpath;
use ph2d_vec_scene::VecPath;
use ph2d_vector::{Affine, BezPath, Shape};

/// A viewbox que o pintor de ícone do catálogo assume (`paint_icon_path::VIEWBOX`).
///
/// ⚠️ Ela **não é um limite** e nem um número escolhido aqui: é a moldura em que todo glifo do
/// editor é autorado, e o pintor divide por ela. Normalizar noutra escala desenharia o ícone com
/// o tamanho errado dentro do botão.
const ICON_VIEWBOX: f64 = 24.0;

/// **O glifo de uma forma vestida**, na caixa de 24×24 — ou `None` quando não há figura.
///
/// `None` para um caminho vazio e para uma bbox de área nula em AMBOS os eixos (um ponto): não há
/// escala que o faça caber, e um glifo de dimensão zero desenha nada. ⚠️ Uma **reta** passa: a
/// bbox dela é degenerada num eixo só, o lado maior dá a escala, e o resultado é uma barra
/// centrada — que é o desenho.
#[must_use]
pub fn icon_face(path: &VecPath) -> Option<BezPath> {
    let mut bp = build_bezpath(&path.cooked());
    if bp.elements().is_empty() {
        return None;
    }
    let bb = bp.bounding_box();
    let (w, h) = (bb.width(), bb.height());
    let span = w.max(h);
    if !span.is_finite() || span <= 0.0 {
        return None;
    }
    let s = ICON_VIEWBOX / span;
    // Centra a figura escalada na caixa: o lado maior encosta nas bordas, o menor sobra em partes
    // iguais. O `−bb.x0` leva a origem da bbox ao zero ANTES da escala, e é por isso que a
    // translação vem multiplicada por `s`.
    //
    // ⚠️ O `-s` no eixo Y é a conversão de convenção (ver o cabeçalho), e é ele que decide qual
    // ponta da bbox ancora: sob a inversão o TOPO do mundo (`bb.y1`) é quem tem de pousar no
    // menor Y da caixa, então o termo é `+ bb.y1 * s` e não `- bb.y0 * s`.
    let tx = (ICON_VIEWBOX - w * s) * 0.5 - bb.x0 * s;
    let ty = (ICON_VIEWBOX - h * s) * 0.5 + bb.y1 * s;
    bp.apply_affine(Affine::translate((tx, ty)) * Affine::scale_non_uniform(s, -s));
    Some(bp)
}

#[cfg(test)]
#[path = "widget_icon_tests.rs"]
mod tests;
