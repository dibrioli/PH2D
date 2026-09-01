//! ⭐⭐⭐ **SOLDAR na shell** (plano 39) — a costura entre a selecção e a lei
//! ([`ph2d_vec_scene::weld`]).
//!
//! Segue a convenção das outras operações destrutivas do módulo (`apply_vec_boolean`): os
//! operandos podem ter poses diferentes, então cada um é **assado no MUNDO** e o que nasce é
//! world-space na identidade — a rede aparece exactamente onde os traços estavam.
//!
//! ⚠️ **Decisão do Enio (2026-08-31): soldar CONSOME os traços originais.** Não há vínculo vivo,
//! não há original escondido: o que sobra é a rede, e o caminho de volta é o desfazer.

use ph2d_vec_scene::{Contour, VecPath, VecScene, VecVertex, VecXforms, trim_tool, weld};

/// Um contorno como o motor de corte o vê: os vértices e se ele fecha.
type Arco = (Vec<VecVertex>, bool);

/// Os contornos de um caminho, já no MUNDO.
fn contornos_mundo(scene: &VecScene, xforms: &VecXforms, id: u64) -> Vec<Contour> {
    let Some(p) = scene.path(id) else {
        return Vec::new();
    };
    let mut cozido = p.cooked().into_owned();
    ph2d_vec_scene::bake_xform(&mut cozido, &ph2d_vec_scene::xform_of(xforms, id));
    trim_tool::contours_of(&cozido).collect()
}

/// **SOLDA a selecção**: cada contorno parte-se em arcos nos pontos onde encontra os outros, e as
/// pontas vizinhas caem no mesmo sítio.
///
/// ⚠️ **Um caminho que não encontra ninguém NÃO é tocado** — nem o objecto, nem o id, nem os
/// subpaths. É o que faz soldar uma selecção grande não dissolver os compostos que estavam só a
/// passar por lá.
///
/// ⛔ **Um caminho que É cortado dissolve-se por inteiro**, e um composto perde o buraco: depois da
/// solda os contornos são arcos de uma rede, e um arco não tem dentro. É o preço declarado de
/// *"consome os originais"*.
pub(crate) fn apply_vec_weld(
    scene: &mut VecScene,
    history: &mut ph2d_vec_edit::History,
    pen: &mut ph2d_vec_edit::PenTool,
    xforms: &VecXforms,
) {
    let sel: Vec<u64> = pen.selected_paths().to_vec();
    if sel.len() < 2 && !sel.is_empty() {
        // Um caminho sozinho ainda pode ter AUTO-cruzamento — vale a pena tentar.
    }
    if sel.is_empty() {
        eprintln!("[ph2d-vec] soldar: selecione os tracos a soldar (Shift+clique)");
        return;
    }
    // Todos os contornos de todos os seleccionados, no mundo — o universo que se cruza.
    let por_caminho: Vec<(u64, Vec<Contour>)> = sel
        .iter()
        .map(|&id| (id, contornos_mundo(scene, xforms, id)))
        .collect();
    let escala = escala_da_selecao(&por_caminho);

    // Por caminho: os arcos de cada contorno dele. `None` = ninguém o cortou.
    let mut cortados: Vec<(u64, Vec<Arco>)> = Vec::new();
    for (id, meus) in &por_caminho {
        let mut arcos = Vec::new();
        let mut mexeu = false;
        for (k, c) in meus.iter().enumerate() {
            // Os OUTROS: todo contorno que não seja este. ⚠️ Os do MESMO caminho entram — dois
            // subpaths que se cruzam são uma rede tanto quanto dois caminhos.
            let mut outros: Vec<Arco> = Vec::new();
            for (oid, cs) in &por_caminho {
                for (ok, o) in cs.iter().enumerate() {
                    if oid == id && ok == k {
                        continue; // ele próprio: o auto-cruzamento já sai do `crossings_against`
                    }
                    outros.push((o.verts.clone(), o.closed));
                }
            }
            let xings = trim_tool::crossings_against(&c.verts, c.closed, &outros, escala);
            let pedacos = weld::split_at(&c.verts, c.closed, &xings);
            mexeu |= pedacos.len() > 1 || (c.closed && pedacos.first().is_some_and(|a| !a.1));
            arcos.extend(pedacos);
        }
        if mexeu {
            cortados.push((*id, arcos));
        }
    }
    if cortados.is_empty() {
        eprintln!("[ph2d-vec] soldar: nada se cruza na selecao — nada a soldar");
        return;
    }

    let pre = scene.clone();
    // A fatia de z da base é a do caminho cortado mais ao fundo: a rede não salta para o topo.
    let at = cortados
        .iter()
        .filter_map(|(id, _)| scene.paths().iter().position(|p| p.id == *id))
        .min()
        .unwrap_or(0);
    let mut novos: Vec<u64> = Vec::new();
    let mut k = 0usize;
    for (id, arcos) in cortados {
        let molde = scene.path(id).cloned();
        scene.remove_path(id);
        for (verts, closed) in arcos {
            let base = molde.clone().unwrap_or_default();
            // ⚠️ O estilo VIAJA para cada arco (a cor, a largura, o tracejado); a pilha de efeitos
            // e os subpaths NÃO — eles descrevem o caminho que deixou de existir.
            let arco = VecPath {
                verts,
                closed,
                subpaths: Vec::new(),
                effects: Vec::new(),
                ..base
            };
            novos.push(scene.insert_path(at + k, arco));
            k += 1;
        }
    }
    history.push_undo(pre);
    pen.select_many(&novos);
    eprintln!("[ph2d-vec] soldar: ok ({} arco[s])", novos.len());
}

/// A diagonal da caixa de tudo o que entra — a régua com que duas travessias quase-coincidentes
/// são a mesma. ⛔ Um número fixo trataria uma peça de 5 unidades e uma de 5 000 igual.
fn escala_da_selecao(por_caminho: &[(u64, Vec<Contour>)]) -> f64 {
    let (mut lo, mut hi) = ([f64::INFINITY; 2], [f64::NEG_INFINITY; 2]);
    for (_, cs) in por_caminho {
        for c in cs {
            for v in &c.verts {
                lo = [lo[0].min(v.anchor[0]), lo[1].min(v.anchor[1])];
                hi = [hi[0].max(v.anchor[0]), hi[1].max(v.anchor[1])];
            }
        }
    }
    if !lo[0].is_finite() {
        return 1.0;
    }
    ((hi[0] - lo[0]).powi(2) + (hi[1] - lo[1]).powi(2))
        .sqrt()
        .max(1e-6)
}

#[cfg(test)]
#[path = "vec_weld_tests.rs"]
mod tests;
