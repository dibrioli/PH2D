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
pub mod relogio;
pub mod smoke;
pub mod smoke_antena;
pub mod smoke_pega;
pub mod smoke_caixa;
pub mod smoke_regua;

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
/// depois drena o que a curva mandou ([`canais::aplicar`]), prepara o que lê o tempo (`relogio`: os
/// clips do Action, as molas) e avalia a pilha — objectos e ossos, a IK
/// incluída. Corre DEPOIS do osso inteligente (a pose de base). Devolve quantas entidades a pilha moveu.
pub fn quadro(
    sim: &mut SimWorld,
    cena: &mut ph2d_vec_scene::VecScene,
    preview: &mut PreviewDrive,
    relogio: Option<relogio::Relogio<'_>>,
) -> usize {
    let w = sim.world_mut();
    if !w.contains_resource::<Encenado>() {
        w.insert_resource(Encenado);
        if let Some(n) = smoke::nivel_armado() {
            smoke::encenar(w, n);
        }
    }
    smoke_regua::semear_se_pedida(sim, cena);
    smoke_caixa::semear_se_pedida(sim, cena);
    // Um `IkGoal` de antes da W2 (ficheiro, prefab, colagem) vira o item IK, e a IK de 1 osso o Look
    // At, antes de avaliar.
    ph2d_constraints_ecs::converter_ik_goals(sim.world_mut());
    ph2d_constraints_ecs::converter_ik_de_um_osso(sim.world_mut());
    canais::aplicar(sim, preview);
    caminhos(sim, cena);
    if let Some(r) = relogio {
        ph2d_constraints_ecs::com_autorado(sim, preview, |sim| relogio::preparar(sim, r));
    }
    ph2d_constraints_ecs::evaluate(sim, preview)
}

/// A poligonal LOCAL das formas que algum Follow Path ou Spline IK segue — a porta do `motion.path`
/// ([`ph2d_vec_scene::world_polyline`]) com o afim identidade — e os vértices AUTORADOS das que um
/// ponto de forma prende.
fn caminhos(sim: &mut SimWorld, cena: &ph2d_vec_scene::VecScene) {
    use ph2d_constraints::Kind;
    use ph2d_constraints_ecs::Constraints;
    let w = sim.world();
    let mut linhas = std::collections::BTreeMap::new();
    let mut pontos = std::collections::BTreeMap::new();
    for c in w.iter_entities().filter_map(|e| e.get::<Constraints>()) {
        for it in &c.items {
            let Some(s) = it.target else { continue };
            let Some(forma) = ph2d_ecs::entity_with_stable_id(w, s)
                .and_then(|e| w.get::<ph2d_ecs::VecPathRef>(e))
            else {
                continue;
            };
            match it.kind {
                Kind::FollowPath(_) | Kind::SplineIk(_) => {
                    linhas.entry(s).or_insert_with(|| {
                        ph2d_vec_scene::world_polyline(
                            cena,
                            forma.0,
                            &ph2d_vec_scene::Xform::IDENTITY,
                        )
                    });
                }
                Kind::PontoDeForma(_) => {
                    if let Some(p) = cena.paths().iter().find(|p| p.id == forma.0) {
                        let todos = p
                            .verts
                            .iter()
                            .chain(p.subpaths.iter().flat_map(|c| &c.verts));
                        pontos.insert(s, todos.map(|v| (v.id, v.anchor)).collect());
                    }
                }
                _ => {}
            }
        }
    }
    let w = sim.world_mut();
    w.insert_resource(ph2d_constraints_ecs::Caminhos(linhas));
    w.insert_resource(ph2d_constraints_ecs::Pontos(pontos));
}
