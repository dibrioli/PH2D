//! O pedido ao Motion: os objectos de que algum ponto de simulação lê partículas, e os `id` de cada
//! um (`docs/BlenderConstraints/07_W6_pontas.md` §5). O Motion arma as tomadas dos sinks deles e
//! devolve só essas partículas em `ph2d_ecs::motion_generator::Particulas`.

use std::collections::{BTreeMap, BTreeSet};

use ph2d_constraints::Kind;
use ph2d_constraints_ecs::Constraints;
use ph2d_ecs::motion_generator::ParticulasPedidas;
use ph2d_ecs::{PresentWorld, SimWorld};
use ph2d_editor_core::GizmoView;
use ph2d_editor_core::screens::hero::GizmoStateGroup;
use ph2d_preview_drive::PreviewDrive;

/// Reescreve o pedido (só quando muda: o Motion arma as tomadas pela diferença).
pub(crate) fn pedir(sim: &mut SimWorld) {
    let w = sim.world();
    let mut ids: BTreeMap<_, BTreeSet<u32>> = BTreeMap::new();
    for it in w
        .iter_entities()
        .filter_map(|e| e.get::<Constraints>())
        .flat_map(|c| &c.items)
    {
        if let (Kind::PontoDeSimulacao(p), Some(s)) = (it.kind, it.target) {
            ids.entry(s).or_default().insert(p.id);
        }
    }
    if w.get_resource::<ParticulasPedidas>().map(|p| &p.0) != Some(&ids) {
        sim.world_mut().insert_resource(ParticulasPedidas(ids));
    }
}

/// ⭐ **O passe DEPOIS do cozimento, inteiro** (§3): a pilha relê os donos presos às partículas deste
/// cozimento ([`ph2d_constraints_ecs::depois_do_motion`]) e quem já os desenhou neste quadro segue-os
/// — o espelho e as instâncias no `present` (o extract correu antes) e a caixa do gizmo da selecção
/// (publicada antes, do espelho). Devolve quantos donos moveu; nenhum ⇒ nada mais se lê.
pub fn depois_do_motion(
    sim: &mut SimWorld,
    present: &mut PresentWorld,
    preview: &mut PreviewDrive,
    gizmo: Option<&mut GizmoStateGroup>,
) -> usize {
    let movidos = ph2d_constraints_ecs::depois_do_motion(sim, preview);
    if movidos.is_empty() {
        return 0;
    }
    let repostos = ph2d_render::repoe_no_presente(sim.world(), present.world_mut(), &movidos);
    if let Some(g) = gizmo {
        for (e, antes, depois) in &repostos {
            // Só translação: uma base que mudou (rodou, escalou) não se repõe por deslocamento.
            if antes.affine()[..4] != depois.affine()[..4] {
                continue;
            }
            let d = depois.translation() - antes.translation();
            let bits = e.to_bits();
            if g.selection == Some(bits)
                && let Some(v) = g.view.as_mut()
            {
                desloca(v, [d.x, d.y]);
            }
            for (b, v) in &mut g.extra_views {
                if *b == bits {
                    desloca(v, [d.x, d.y]);
                }
            }
        }
    }
    movidos.len()
}

/// A caixa inteira por `d` de mundo: pela translação do `model` (sob a qual vivem caixa e pivô).
fn desloca(v: &mut GizmoView, d: [f32; 2]) {
    if v.model == ph2d_editor_core::gizmo::MODEL_IDENTITY {
        for p in [
            &mut v.bbox_min_world,
            &mut v.bbox_max_world,
            &mut v.pivot_world,
        ] {
            p[0] += d[0];
            p[1] += d[1];
        }
    } else {
        v.model[4] += d[0];
        v.model[5] += d[1];
    }
}
