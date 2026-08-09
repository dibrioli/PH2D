//! **O DESENHO É O GLIFO** — a porta única do ícone de um `IconButton` autorado.
//!
//! Um `IconButton` é o segundo (e último) tipo do catálogo cujo parâmetro nem o retângulo, nem o
//! rótulo, nem os tokens, nem o estado vivo determinam: *qual* ícone? Num editor VETORIAL a
//! resposta nativa é a que não inventa canal nenhum — **a forma que veste o botão É o glifo**. O
//! artista desenha a engrenagem, marca-a `IconButton`, e o app põe a moldura à volta dela.
//!
//! # Por que isto é UMA função e não duas
//!
//! As duas metades do W8b precisam do mesmo glifo por motivos diferentes — o **canvas** para o
//! pintar ao vivo, o **codegen** para o escrever no painel gerado como texto — e é exactamente aí
//! que nasce a divergência que só uma screenshot revela: um ícone no canvas e outro no painel.
//! Elas percorrem esta função, e o que muda depois dela é só a saída
//! ([`BezPath`] × [`BezPath::to_svg`]).
//!
//! # A geometria é a AUTORADA, não a projetada — e a consequência está nomeada
//!
//! [`build_bezpath`] sobre o [`cooked`](ph2d_vec_scene::VecPath::cooked) do path: a quina viva
//! (ADR-0121) e a pilha de Live Path Effects entram, a **pose não**. Duas razões, nesta ordem:
//!
//! - **as duas metades alcançam-na com ZERO argumento novo** — o `ui_panel_spec` tem só `scene`, e
//!   pedir-lhe `xforms`/`camera` para orientar um glifo que vai ser re-normalizado numa caixa de
//!   24 px seria churn a troco de nada. Sem argumento em comum, as duas produzem os MESMOS bytes
//!   por construção, e não meramente por gate;
//! - **a moldura já ignora a pose**: o chip é axis-aligned na tela. Honrar a rotação só no glifo
//!   seria honrar meia regra.
//!
//! ⚠️ **O preço, dito em vez de descoberto:** girar a forma pelo gizmo **não gira o glifo** — ele
//! é o desenho autorado, endireitado. Se um smoke pedir o contrário, é um argumento de distância.
//!
//! # A normalização é UNIFORME
//!
//! Um glifo é uma figura, e esticá-la para encher a caixa mudaria o desenho do artista. O lado
//! maior encosta nos 24, o outro centra-se. É a mesma lei que o [`paint_icon_path`] aplica depois
//! ao encaixar a viewbox no retângulo do botão — a de lá é sobre a CAIXA, a daqui sobre a FIGURA.
//!
//! # ⚠️ E ela INVERTE Y, porque os dois espaços discordam
//!
//! O documento é **Y para CIMA** (é a câmera quem inverte: `world_to_screen_affine` multiplica por
//! `scale_non_uniform(k, -k)`), e a caixa de 24×24 do ícone é **Y para BAIXO** — ela é a viewbox do
//! SVG, que é de onde todo glifo do catálogo nasce. O [`paint_icon_path`] mapeia essa caixa para o
//! retângulo do botão **sem inverter nada**, e está certo: para um `IconId` as duas pontas já
//! falam Y-down.
//!
//! Esta função é o ÚNICO ponto em que uma geometria de documento entra naquela caixa, logo é aqui
//! que a conversão pertence. Sem ela o desenho do artista chega ao botão **de cabeça para baixo**
//! (report do Enio, 2026-08-09) — e chegava nas DUAS metades, porque as duas passam por aqui.
//!
//! [`paint_icon_path`]: ph2d_editor::paint::paint_icon_path

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
pub(crate) fn icon_face(path: &VecPath) -> Option<BezPath> {
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
