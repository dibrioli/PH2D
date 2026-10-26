#![forbid(unsafe_code)]
//! **A família das constraints** (plano
//! [`docs/BlenderConstraints/01_plano.md`](https://github.com/dibrioli/PH2D/blob/main/docs/BlenderConstraints/01_plano.md)
//! §2.1). A shell chama [`quadro`] uma vez por quadro, entre o apply da timeline e o esqueleto.

pub mod bake;
pub mod campos;
pub mod canais;
pub mod chaves;
pub mod compensa;
pub mod gizmo;
pub mod inspector;
pub mod marcas;
pub mod smoke;
pub mod smoke_pega;

use bevy_ecs::prelude::Resource;
use ph2d_ecs::SimWorld;
use ph2d_preview_drive::PreviewDrive;

/// A declaração da família à shell.
pub const FAMILY: ph2d_app_host::AppFamily = ph2d_app_host::AppFamily {
    key: "constraints",
    routers: smoke::ROUTERS,
};

/// A cena de smoke já foi encenada neste mundo (uma vez por processo e mundo).
#[derive(Resource)]
struct Encenado;

/// ⭐ **A porta do quadro.** Na primeira chamada encena a cena armada (`PH2D_CONSTRAINT_SMOKE`);
/// depois drena o que a curva mandou ([`canais::aplicar`]) e avalia a pilha — objectos e ossos, a IK
/// incluída. Corre DEPOIS do osso inteligente (a pose de base). Devolve quantas entidades a pilha moveu.
pub fn quadro(
    sim: &mut SimWorld,
    cena: &ph2d_vec_scene::VecScene,
    preview: &mut PreviewDrive,
) -> usize {
    let w = sim.world_mut();
    if !w.contains_resource::<Encenado>() {
        w.insert_resource(Encenado);
        if let Some(n) = smoke::nivel_armado() {
            smoke::encenar(w, n);
        }
    }
    // Um `IkGoal` de antes da W2 (ficheiro, prefab, colagem) vira o item IK, e a IK de 1 osso o Look
    // At, antes de avaliar.
    ph2d_constraints_ecs::converter_ik_goals(sim.world_mut());
    ph2d_constraints_ecs::converter_ik_de_um_osso(sim.world_mut());
    canais::aplicar(sim, preview);
    caminhos(sim, cena);
    ph2d_constraints_ecs::evaluate(sim, preview)
}

/// A poligonal LOCAL das formas que algum Follow Path segue — a porta do `motion.path`
/// ([`ph2d_vec_scene::world_polyline`]) com o afim identidade.
fn caminhos(sim: &mut SimWorld, cena: &ph2d_vec_scene::VecScene) {
    use ph2d_constraints::Kind;
    use ph2d_constraints_ecs::Constraints;
    let w = sim.world();
    let mut out = std::collections::BTreeMap::new();
    for c in w.iter_entities().filter_map(|e| e.get::<Constraints>()) {
        for it in c
            .items
            .iter()
            .filter(|i| matches!(i.kind, Kind::FollowPath(_)))
        {
            let Some(s) = it.target else { continue };
            let Some(forma) = ph2d_ecs::entity_with_stable_id(w, s)
                .and_then(|e| w.get::<ph2d_ecs::VecPathRef>(e))
            else {
                continue;
            };
            let local =
                ph2d_vec_scene::world_polyline(cena, forma.0, &ph2d_vec_scene::Xform::IDENTITY);
            out.insert(s, local);
        }
    }
    sim.world_mut()
        .insert_resource(ph2d_constraints_ecs::Caminhos(out));
}
