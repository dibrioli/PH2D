#![forbid(unsafe_code)]
//! **A família das constraints** (plano
//! [`docs/BlenderConstraints/01_plano.md`](https://github.com/dibrioli/PH2D/blob/main/docs/BlenderConstraints/01_plano.md)
//! §2.1). A shell chama [`quadro`] uma vez por quadro, entre o apply da timeline e o esqueleto.

pub mod canais;
pub mod inspector;
pub mod marcas;
pub mod smoke;

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
/// depois drena o que a curva mandou ([`canais::aplicar`]) e avalia a pilha. Devolve quantas entidades a pilha moveu.
pub fn quadro(sim: &mut SimWorld, preview: &mut PreviewDrive) -> usize {
    let w = sim.world_mut();
    if !w.contains_resource::<Encenado>() {
        w.insert_resource(Encenado);
        if let Some(n) = smoke::nivel_armado() {
            smoke::encenar(w, n);
        }
    }
    canais::aplicar(sim, preview);
    ph2d_constraints_ecs::evaluate(sim, preview)
}
