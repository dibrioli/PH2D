//! ⭐ **ENQUADRAR A PEÇA** — para onde a câmera vai quando alguém pede *«mostra-me isto»*.
//!
//! ⚠️ **O corte é por assunto**: o [`super`] responde por *o que a mão agarra e onde* (as alças, o
//! enquadramento em pixels da área, a órbita); este por *a moldura de uma peça nova, o `Home`, e a
//! viagem entre vistas nomeadas*. O ficheiro comum passou as `600` do gate de LOC do shell quando as
//! alças de vértice entraram na W133 — ⛔ *split, nunca allowlist*.
//!
//! ⚠️ **Módulo-filho por `#[path]`**, com re-export no pai: `field3d_input::frame_the_part`,
//! `::frame_into` e `::fly_to_view` continuam a resolver em todos os chamadores.

use super::*;

/// ⭐ **Enquadra a peça que o smoke tem em mãos** — o elo entre a lei pura e o documento.
///
/// ⚠️ **`false` quando não há peça**, e quem chama decide o que fazer com isso: o `Home` já repôs a
/// orientação e fica assim (não há o que enquadrar); o pedido de um load simplesmente não tem efeito
/// e volta a ser feito no quadro seguinte, quando o documento já estiver cozido.
pub(crate) fn frame_the_part(s: &mut Smoke) -> bool {
    let mut to = s.vp().cam;
    if !frame_into(s, &mut to) {
        return false;
    }
    crate::field3d_smoke::fly_to(s, to);
    true
}

/// A mesma conta, escrita num destino em vez de na câmera — é ela que faz o `Home` **compor** o
/// repor com o enquadrar numa viagem só, em vez de duas.
pub(crate) fn frame_into(s: &Smoke, to: &mut ph2d_field_render::Orbit) -> bool {
    let Some(doc) = s.doc.as_ref() else {
        return false;
    };
    let reg = crate::field3d_smoke::sampled_registry();
    let Some(ball) = ph2d_field_eval::bounds::bounding_ball(doc, &reg) else {
        return false;
    };
    law::frame(to, ball);
    true
}

/// ⭐ **Partir para uma vista nomeada**: a orientação dela **e** o enquadramento, numa viagem só.
pub(crate) fn fly_to_view(s: &mut Smoke, view: crate::field3d_views::Standard) {
    let mut to = s.vp().cam;
    to.rotation = view.rotation();
    frame_into(s, &mut to);
    crate::field3d_smoke::fly_to(s, to);
}
