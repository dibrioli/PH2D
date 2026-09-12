//! **A metade da FRONTEIRA** dos gates da cena `physics_smoke_player`: os que exercitam o
//! **assador desta shell** (os rótulos que o painel do Inspector pinta) sobre uma cena que vive na
//! [`crate::physics_smoke_player`].
//!
//! ⚠️ Um `#[cfg(test)]` é invisível do outro lado da fronteira de crate (HOWTO §2),
//! e o corte é **por quem o teste EXERCITA, não por quem ele nomeia** — os outros
//! gates da mesma cena ficaram com o sujeito, na crate.

use crate::physics_smoke_player::*;

/// A mensagem NOMEIA os dois knobs que ela manda o artista mexer.
///
/// ⚠️ Um roteiro que cita um controle por um nome que a UI não usa é um roteiro
/// que faz o artista procurar o que não existe e reportar a feature como
/// ausente.
#[test]
fn the_message_names_the_controls_the_panel_paints() {
    for label in ["Weight on Ground", "Push on Ground"] {
        assert!(
            REACTION_SMOKE_MESSAGE.contains(label),
            "a mensagem da cena 85 tem de nomear o controle {label:?}"
        );
        assert!(
            ph2d_panel_inspector::player_row_labels().contains(&label),
            "e a §14 tem de PINTAR uma row com esse rotulo: {label:?}"
        );
    }
}
