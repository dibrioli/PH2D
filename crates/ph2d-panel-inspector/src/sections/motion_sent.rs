//! **O cartão MOTION** (GEF 08 MI-8, maquete 2A): em Play, uma linha por nó «Send signal» do objecto —
//! «Sent low_health 2 times». A contagem zera a cada Play (a ponte, `Envios::corrida`).

use super::*;
use ph2d_editor_core::motion_sent::{InspectorMotionSentInfo, MotionSent};
use ph2d_i18n::{tr, tr_with};

/// A frase de um envio: «not sent yet» no zero (como «Not fired yet»), «once» no singular.
pub(crate) fn frase(s: &MotionSent) -> String {
    let n = s.times.to_string();
    match s.times {
        0 => tr_with("panel.inspector.motion.not_sent", &[("s", &s.signal)]),
        1 => tr_with("panel.inspector.motion.sent_once", &[("s", &s.signal)]),
        _ => tr_with(
            "panel.inspector.motion.sent",
            &[("s", &s.signal), ("n", &n)],
        ),
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_motion_sent_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    i: &InspectorMotionSentInfo,
) -> f32 {
    let (fold, mut cur_y) = match super::nav::cabecalho(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        y,
        ph2d_editor_core::ids::INSP_LIVE_MOTION_SECTION,
        tr("component.object_kind.motion"),
    ) {
        Ok(v) => v,
        Err(y) => return y,
    };
    for s in &i.sent {
        cur_y = super::rows::aviso(
            scene,
            text_system,
            theme,
            x,
            w,
            cur_y,
            &frase(s),
            ColorToken::Text1,
        );
    }
    fold.finish(store, scene, hit_index, cur_y)
}
