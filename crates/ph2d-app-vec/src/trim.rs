//! **O pedaço que o Trim aponta** (plano 38) — o TIPO que o realce desenha e o clique apaga. A lei
//! (o que é uma fronteira, o que sobra) mora em `ph2d_vec_scene::trim_tool`; o gesto na shell
//! (`vec_trim.rs`).
//!
//! ⚠️ **Desceu da shell em 2026-09-12** (`line/render-loop`, A9 da auditoria de arquitectura): o
//! campo `vec_trim_hit` da `App` tinha o TIPO na shell, e um campo assim não pode juntar-se ao
//! [`crate::state::VecState`].

use ph2d_vec_scene::{VecPath, VecPathId, VecScene, VecVertex, VecXforms, trim_tool};

/// **O pedaço que o cursor está a apontar.** É o que o realce desenha e o que o clique apaga — a
/// mesma resposta, pela mesma porta.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TrimHit {
    pub path: VecPathId,
    /// O índice do contorno na ordem canónica (`trim_tool::contours_of`): `0` = primário.
    pub contour: usize,
    pub de: f64,
    pub ate: f64,
}

// ── A COSTURA (desceu da shell `vec_trim.rs` em 2026-10-05) ──────────────────────────────────
//
// ⭐⭐⭐ **A ferramenta TRIM na shell** (plano 38) — a costura entre o ponteiro e a lei pura
// ([`ph2d_vec_scene::trim_tool`]).
//
// # O que mora aqui e o que NÃO mora
//
// Aqui: *quem está sob o cursor*, *em que espaço se mede* e *quem são os outros caminhos que
// cortam*. A **lei** — o que é uma fronteira, qual é o pedaço, o que sobra do corte — mora na
// crate, sem cena e sem ponteiro, e é lá que ela é testada.
//
// # O ESPAÇO é o LOCAL do alvo, e isso não é conveniência
//
// A geometria de um caminho é **local** e a pose vive no `Transform` (ADR-0110/0111). Medir os
// cruzamentos em mundo obrigaria a levar o alvo para lá e a trazer as fracções de volta; medir no
// local do ALVO leva os OUTROS até ele (`xf_outro ∘ xf_alvo⁻¹`) e deixa as fracções já na régua em
// que o corte acontece. ⚠️ Um alvo com escala não-uniforme torna as duas escolhas **diferentes**,
// e a que importa é a do corte.

/// A geometria COZIDA de um caminho, no espaço LOCAL de `alvo`.
fn cozido_em(path: &VecPath, xforms: &VecXforms, alvo: VecPathId) -> Option<VecPath> {
    let mut p = path.cooked().into_owned();
    if path.id != alvo {
        let para_mundo = ph2d_vec_scene::xform_of(xforms, path.id);
        let de_mundo = ph2d_vec_scene::xform_of(xforms, alvo).inverse()?;
        ph2d_vec_scene::bake_xform(&mut p, &para_mundo.then(&de_mundo));
    }
    Some(p)
}

/// **QUAL PEDAÇO está sob `local`** (o cursor no espaço local de `alvo`), ou `None` quando nenhum
/// contorno dele está ao alcance de `tol`.
///
/// ⚠️ **O contorno é escolhido pela DISTÂNCIA**, e não pela ordem: num compound o cursor pode estar
/// mais perto de um subpath do que do primário, e apagar o pedaço errado seria pior que não fazer
/// nada.
pub fn hit(
    scene: &VecScene,
    xforms: &VecXforms,
    alvo: VecPathId,
    local: [f64; 2],
    tol: f64,
) -> Option<TrimHit> {
    let path = scene.path(alvo)?;
    let cozido = cozido_em(path, xforms, alvo)?;
    // Os OUTROS, já no local do alvo — e o alvo NÃO entra nesta lista: o auto-cruzamento dele já
    // sai do `crossings_against`, e pô-lo aqui contaria cada travessia duas vezes.
    let outros: Vec<(Vec<VecVertex>, bool)> = scene
        .paths()
        .iter()
        .filter(|p| p.id != alvo)
        .filter_map(|p| cozido_em(p, xforms, alvo))
        .flat_map(|p| {
            trim_tool::contours_of(&p)
                .map(|c| (c.verts, c.closed))
                .collect::<Vec<_>>()
        })
        .collect();
    let escala = escala_de(&cozido);

    let mut melhor: Option<(usize, f64, f64)> = None; // (contorno, fracção, distância)
    for (i, c) in trim_tool::contours_of(&cozido).enumerate() {
        if let Some((frac, dist)) = trim_tool::nearest_fraction(&c.verts, c.closed, local)
            && dist <= tol
            && melhor.is_none_or(|(_, _, m)| dist < m)
        {
            melhor = Some((i, frac, dist));
        }
    }
    let (contour, frac, _) = melhor?;
    let c = trim_tool::contours_of(&cozido).nth(contour)?;
    let xings = trim_tool::crossings_against(&c.verts, c.closed, &outros, escala);
    let fronteiras = trim_tool::boundaries(&c.verts, c.closed, &xings);
    let (de, ate) = trim_tool::piece_at(&fronteiras, c.closed, frac)?;
    Some(TrimHit {
        path: alvo,
        contour,
        de,
        ate,
    })
}

/// O tamanho de referência para fundir travessias quase-coincidentes: a diagonal da caixa das
/// âncoras. ⛔ Um número fixo trataria uma peça de 5 unidades e uma de 5 000 com a mesma tolerância.
fn escala_de(path: &VecPath) -> f64 {
    let mut lo = [f64::INFINITY; 2];
    let mut hi = [f64::NEG_INFINITY; 2];
    for c in trim_tool::contours_of(path) {
        for v in &c.verts {
            lo = [lo[0].min(v.anchor[0]), lo[1].min(v.anchor[1])];
            hi = [hi[0].max(v.anchor[0]), hi[1].max(v.anchor[1])];
        }
    }
    if !lo[0].is_finite() {
        return 1.0;
    }
    ((hi[0] - lo[0]).powi(2) + (hi[1] - lo[1]).powi(2))
        .sqrt()
        .max(1e-6)
}

/// **APLICA o corte.** `true` quando a cena mudou.
///
/// ⚠️ O caminho é **apagado** quando não sobra geometria — é a peça toda, e é a resposta do Fusion
/// para uma reta que não cruza nada.
pub fn apply(scene: &mut VecScene, hit: &TrimHit) -> bool {
    let Some(path) = scene.path(hit.path).cloned() else {
        return false;
    };
    match trim_tool::sever(&path, hit.contour, hit.de, hit.ate) {
        // ⚠️ **Escreve NO LUGAR** (`path_mut`), e não apaga-e-empurra: o id, a ordem em z e a
        // entidade ECS que o representa (ADR-0110) têm de sobreviver ao corte — um `push_path` daria
        // um objecto novo, no topo da pilha, sem o `Transform` nem os componentes que este tinha.
        Some(novo) => {
            let Some(slot) = scene.path_mut(hit.path) else {
                return false;
            };
            *slot = novo;
            true
        }
        None => scene.remove_path(hit.path),
    }
}

/// **O PEDAÇO em coordenadas de MUNDO**, pronto para o realce.
///
/// ⚠️ Sai da **mesma porta** que o corte (`trim_tool::piece_geometry`, o complemento exacto do
/// `sever`), e volta ao mundo pelo afim do alvo — porque foi no local dele que a fracção foi medida.
#[must_use]
pub fn piece_world(scene: &VecScene, xforms: &VecXforms, hit: &TrimHit) -> Vec<VecPath> {
    let Some(path) = scene.path(hit.path) else {
        return Vec::new();
    };
    let Some(cozido) = cozido_em(path, xforms, hit.path) else {
        return Vec::new();
    };
    let Some(c) = trim_tool::contours_of(&cozido).nth(hit.contour) else {
        return Vec::new();
    };
    let Some(verts) = trim_tool::piece_geometry(&c.verts, c.closed, hit.de, hit.ate) else {
        return Vec::new();
    };
    let mut p = VecPath {
        verts,
        closed: false,
        subpaths: Vec::new(),
        effects: Vec::new(),
        ..cozido
    };
    ph2d_vec_scene::bake_xform(&mut p, &ph2d_vec_scene::xform_of(xforms, hit.path));
    vec![p]
}

/// ⭐ **O pedaço no ponto de MUNDO `world`** — a porta única do realce (por quadro) e do clique: o
/// caminho que a caneta agarra ali ([`ph2d_vec_edit::PenTool::path_at`], que respeita o que não se
/// agarra) e o pedaço dele, medido no espaço local do alvo.
#[must_use]
pub fn hit_at(
    pen: &ph2d_vec_edit::PenTool,
    scene: &VecScene,
    xforms: &VecXforms,
    world: [f64; 2],
    tol: f64,
) -> Option<TrimHit> {
    let alvo = pen.path_at(scene, world, tol)?;
    let local = ph2d_vec_scene::xform_of(xforms, alvo)
        .inverse()
        .map_or(world, |inv| inv.apply(world));
    hit(scene, xforms, alvo, local, tol)
}

#[cfg(test)]
#[path = "trim_tests.rs"]
mod tests;
