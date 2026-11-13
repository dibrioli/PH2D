//! As fotos do smoke do jogo (`PH2D_GAME_SMOKE_FOTO`) e a mão que as conduz, como folha: o que cada foto
//! pede ao mundo, o roteiro da mão e o que a janela do jogo lê e empurra. A shell só executa.

mod foto;
pub mod janela;
pub mod mao;
pub mod nomes;

pub use foto::{
    CENTRO_DAS_FOTOS_13, CENTRO_DOS_USOS, Foto, FotoDaCamera, ONDE_LARGA, PassoDoDono, PlanoDaFoto,
    QUADROS_POR_USO, foto, plano_da_foto, tempo_da_chave, uso_no_quadro, usos_do_dono,
};

use ph2d_ecs::{Entity, SimWorld};

/// A camada dos montes do nível 2 — nome de objecto da cena (`ph2d_app_game::smoke::CAMADAS_2`).
pub const MONTES: &str = "Hills";

/// O herói das cenas de smoke — nome de objecto da cena, não texto da interface.
#[must_use]
pub fn player(sim: &mut SimWorld) -> Option<Entity> {
    por_nome(sim, "Player")
}

/// O objecto da raiz com este nome.
#[must_use]
pub fn por_nome(sim: &mut SimWorld, nome: &str) -> Option<Entity> {
    let w = sim.world_mut();
    let mut q = w.query::<(Entity, &ph2d_ecs::Name)>();
    q.iter(w).find(|(_, n)| n.0 == nome).map(|(e, _)| e)
}
