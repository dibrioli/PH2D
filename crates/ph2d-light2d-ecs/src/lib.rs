//! **A luz 2D no ECS** — os componentes, a porta de registo e a COLETA.
//!
//! Desenho: [`docs/Luz2D/01_plano.md`](https://github.com/dibrioli/PH2D/blob/main/docs/Luz2D/01_plano.md)
//! §2. A [`coleta::cena_de_luz`] é a ÚNICA porta por onde emissores e oclusores chegam a um solver
//! — o gabarito (`ph2d-light2d-ref`) e a GPU (W2+). Nenhum solver lê o `World` por outro caminho.
//!
//! ⚠️ Componentes NOVOS não pedem degrau de `PROJECT_SCHEMA`: o blob é chaveado por
//! `blake3(nome canónico)` (`ph2d-ecs/src/scene/registry.rs`, comentário do `VecMorphMachine`) e um
//! ficheiro antigo simplesmente não o tem.

pub mod cena;
pub mod coleta;
mod componentes;

pub use cena::{
    ALBEDO_MAX, CenaDeLuz, Emissor, FormaDoSprite, Oclusor, OutroDeLuz, Rgb, SpriteDeLuz,
    VetorDeLuz,
};
pub use componentes::{BloqueiaLuz, Luz, NaoRecebeLuz, TipoDeLuz};
use ph2d_ecs::scene::ComponentRegistry;

/// Os nomes canónicos, na ordem do registo. Fonte única para a porta e para o teste dela.
pub const NOMES_CANONICOS: [&str; 3] = [
    "ph2d::light2d::BloqueiaLuz",
    "ph2d::light2d::Luz",
    "ph2d::light2d::NaoRecebeLuz",
];

/// **A porta de registo** — chamada uma vez no arranque, ao lado do `register_ecs_components`.
/// Sem ela o `WorldSnapshot` (undo + save) descarta a luz em silêncio.
///
/// `Luz` chega pelo menu Add (o catálogo marca-a `intrinsic`: fora da paleta); o `Default` dela é a
/// lâmpada de fábrica. `BloqueiaLuz` e `NaoRecebeLuz` são oferecidos pela paleta.
pub fn register_light2d_components(reg: &mut ComponentRegistry) {
    reg.register_default::<BloqueiaLuz>(NOMES_CANONICOS[0]);
    reg.register_default::<Luz>(NOMES_CANONICOS[1]);
    reg.register_default::<NaoRecebeLuz>(NOMES_CANONICOS[2]);
}
