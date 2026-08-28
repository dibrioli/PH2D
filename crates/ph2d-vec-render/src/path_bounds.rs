//! **AS CAIXAS de um caminho, em px de TELA** — módulo irmão de [`super`] pelo teto de LOC.
//!
//! O corte é por RESPONSABILIDADE: aqui ninguém DESENHA, só se pergunta *onde, na tela, esta forma
//! vive*. É a pergunta que o produtor de FX raster (plano 24) faz para dimensionar o scratch, e a
//! que o assador de ladrilho de um `PatternSource::Shape` (plano 33 W7) faz para enquadrar a arte.
//!
//! ⚠️ **A lei do transbordo do traço vive numa PORTA só** ([`path_bounds_under`]): o contorno
//! transborda o preenchimento, e duas transcrições dessa conta divergiriam na ponta CEIFADA que o
//! doc dela nomeia — num caminho e não no outro.

use ph2d_vec_scene::{VecPath, VecPathId, VecScene, VecXforms};
use ph2d_vector::{Affine, Rect, Shape};

use crate::{LiveGeometry, build_bezpath, path_to_screen, standalone};

/// **O bbox em TELA de um caminho, como o [`dispatch`] o desenharia** — honrando a geometria
/// DERIVADA (`live`) e a pose (`xforms`). É a pergunta que o produtor de FX raster (plano 24) faz
/// para dimensionar o scratch e posicionar a imagem: onde, na tela, esta forma vive?
///
/// Inclui a metade da espessura do traço (o contorno transborda o fill), escalada pelo afim.
/// `None` se o caminho não existe ou não tem geometria que desenhe algo.
#[must_use]
pub fn path_screen_bounds(
    scene: &VecScene,
    xforms: &VecXforms,
    live: &LiveGeometry,
    id: VecPathId,
    camera: Affine,
) -> Option<(f64, f64, f64, f64)> {
    let mut acc: Option<Rect> = None;
    let mut eat = |path: &VecPath, xf: Affine| {
        if let Some(r) = path_bounds_under(path, xf) {
            acc = Some(acc.map_or(r, |cur| cur.union(r)));
        }
    };
    if let Some(items) = live.get(&id) {
        // Derivada já em MUNDO ⇒ sobe pela câmera (como no `dispatch`).
        for item in items {
            eat(item, camera);
        }
    } else {
        let path = scene.paths().iter().find(|p| p.id == id)?;
        eat(path, path_to_screen(xforms, id, camera));
    }
    let r = acc?;
    Some((r.x0, r.y0, r.x1, r.y1))
}

/// **A caixa de UM caminho AVULSO sob um afim** — a lei de limites desta crate, numa
/// porta só.
///
/// ⚠️ **Ela era o corpo do fecho `eat` do [`path_screen_bounds`]**, e saiu de lá
/// quando um segundo chamador apareceu: o bake de tile de um `source.shape`, cujo
/// `VecPath` vive num store e **não** na cena vetorial (bug do Enio, 2026-08-20 —
/// *"tudo deve brilhar"*). Duas transcrições desta conta divergiriam no
/// transbordo do traço, e o sintoma seria a ponta CEIFADA que o parágrafo abaixo
/// nomeia — num caminho e não no outro.
///
/// Inclui a metade da espessura do traço (o contorno transborda o fill), escalada
/// pelo afim. `None` se o caminho não tem geometria que desenhe algo.
#[must_use]
pub fn path_bounds_under(path: &VecPath, xf: Affine) -> Option<Rect> {
    let mut bp = build_bezpath(path);
    if bp.elements().is_empty() {
        return None;
    }
    bp.apply_affine(xf);
    let r = bp.bounding_box();
    Some(standalone::inflate_for_stroke(path, xf, r))
}

/// A caixa avulsa em coordenadas de tela, como o [`path_screen_bounds`] a devolve.
#[must_use]
pub fn standalone_path_screen_bounds(path: &VecPath, xf: Affine) -> Option<(f64, f64, f64, f64)> {
    let r = path_bounds_under(path, xf)?;
    Some((r.x0, r.y0, r.x1, r.y1))
}
