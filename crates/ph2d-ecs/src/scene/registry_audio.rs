//! **O registo da família do SOM** — irmão do [`super::registry`] por CAP de LOC, como o
//! [`super::registry_camera`]: o irmão bateu `702` de `700` com a SFX Lab W1. O bloco saiu verbatim e na
//! mesma posição.

use super::ComponentRegistry;

/// Os três tipos da família.
pub(super) fn register_audio(reg: &mut ComponentRegistry) {
    reg.register_default::<crate::AudioSource2D>("ph2d::ecs::AudioSource2D");
    // Os sons de um objeto (SFX Lab W1). Irmão e não campo, pelo postcard posicional: arquivo antigo não tem
    // o blob e toca o `sound` de sempre.
    reg.register_default::<crate::SoundSet>("ph2d::ecs::SoundSet");
    // ⚠️ **Um MARCADOR ainda tem de se registar**: a presença É o valor, e sem ele um Ctrl+Z ou um
    // reabrir devolveriam a cena SEM orelhas — que é a única maneira de tudo ficar mudo de uma vez.
    reg.register_default::<crate::AudioListener2D>("ph2d::ecs::AudioListener2D");
}
