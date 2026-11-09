//! **O cursor dos pontos do «Outline»**: a ÚNICA tabela estado → [`Figura`] (o shell só a pede ao sistema).
//!
//! | estado | figura |
//! |---|---|
//! | nada sob o ponteiro | `Seta` (seleção directa) |
//! | sobre um ponto | `SetaPonto` · com `Alt` `PenaMenos` · com `Shift` `Alterna` |
//! | sobre uma aresta | `PenaMais` |
//! | a arrastar (armado) | `Agarrando` · sobre outro ponto do anel `Funde` |

use crate::pontos as p;
use crate::{Teclas, editando};
use bevy_ecs::prelude::Resource;
use bevy_ecs::world::World;
use ph2d_cursor_art::Figura;
use ph2d_host::WindowSize;
use ph2d_render::Camera2d;

/// As teclas do último gesto sobre os pontos: a tinta mostra o que o `Alt`/`Shift` farão.
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TeclasNoCanvas(pub Teclas);

/// A figura do cursor em `s`, ou `None` fora da edição de pontos.
#[must_use]
pub fn cursor_dos_pontos(
    world: &World,
    cam: &Camera2d,
    win: WindowSize,
    s: (f32, f32),
    teclas: Teclas,
) -> Option<Figura> {
    let alvo = editando(world)?;
    let (pose, m, o) = p::do_alvo(world, alvo)?;
    let ecra = p::no_ecra(&pose, &m, &o, cam, win);
    let arrasto = world
        .get_resource::<p::PontosNoCanvas>()
        .and_then(|x| x.arrasto.clone());
    if let Some(a) = arrasto.filter(|a| a.armado) {
        return Some(if p::alvo_da_fusao(&ecra, a.ponto, s).is_some() {
            Figura::Funde
        } else {
            Figura::Agarrando
        });
    }
    if p::ponto_em(&ecra, s).is_some() {
        return Some(if teclas.alt {
            Figura::PenaMenos
        } else if teclas.shift {
            Figura::Alterna
        } else {
            Figura::SetaPonto
        });
    }
    if p::aresta_em(&ecra, s).is_some() {
        return Some(Figura::PenaMais);
    }
    Some(Figura::Seta)
}
