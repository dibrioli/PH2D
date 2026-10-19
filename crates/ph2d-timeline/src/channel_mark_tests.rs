//! Gates do [`super::channel_mark`] (plano 34 §6).

use super::*;
use ph2d_anim::{AnimValue, Interp, RationalTime};

const E: u64 = 7;

/// Opacidade keyada `0,2 @ 0 s → 1,0 @ 1 s` (linear), a 24 fps.
fn doc() -> TimelineDoc {
    let mut d = TimelineDoc::new();
    d.fps_display = 24.0;
    for (t, v) in [(0.0, 0.2_f32), (1.0, 1.0)] {
        d.insert_key(
            E,
            PropKind::Opacity,
            RationalTime::from_seconds(t),
            AnimValue::Float(v),
            Interp::Linear,
        );
    }
    d
}

#[test]
fn sem_curva_e_livre() {
    let d = doc();
    assert_eq!(
        channel_mark(&d, E, PropKind::Rotation, 0.5, true, Some(0.0)),
        ChannelMark::Livre
    );
    assert_eq!(
        channel_mark(&d, 99, PropKind::Opacity, 0.5, true, Some(1.0)),
        ChannelMark::Livre
    );
}

#[test]
fn entre_chaves_na_curva_e_animado() {
    let d = doc();
    assert_eq!(
        channel_mark(&d, E, PropKind::Opacity, 0.5, true, Some(0.6)),
        ChannelMark::Animado
    );
}

/// A chave «deste quadro» aceita meio quadro de exibição para cada lado, e só isso.
#[test]
fn a_chave_deste_quadro_vale_meio_quadro() {
    let d = doc();
    let quase = 0.4 / 24.0;
    assert!(matches!(
        channel_mark(&d, E, PropKind::Opacity, quase, true, None),
        ChannelMark::Chave(_)
    ));
    assert_eq!(
        channel_mark(&d, E, PropKind::Opacity, 0.6 / 24.0, true, None),
        ChannelMark::Animado,
        "a mais de meio quadro ja nao e' a chave deste quadro"
    );
}

/// O clique apaga a chave que o losango aponta: a composição `SelectSingle` + `DeleteSelection`.
#[test]
fn a_chave_apontada_e_a_que_se_apaga() {
    let mut st = crate::TimelineState::new();
    st.doc = doc();
    let mut ph = ph2d_core::Playhead::new(1.0 / 60.0);
    let ChannelMark::Chave(k) = channel_mark(&st.doc, E, PropKind::Opacity, 1.0, true, None) else {
        panic!("ha' chave em 1 s");
    };
    crate::apply_intent(&mut st, &mut ph, crate::TimelineIntent::SelectSingle(k));
    crate::apply_intent(&mut st, &mut ph, crate::TimelineIntent::DeleteSelection);
    let b = st.doc.binding_for(E, PropKind::Opacity).unwrap();
    let keys = st.doc.active_clip().track(b.target).unwrap().keys();
    assert_eq!(keys.len(), 1);
    assert_eq!(keys[0].t.to_seconds(), 0.0, "apagou a chave errada");
}

/// O mundo fora da curva e por gravar ganha de tudo — até da chave do quadro.
#[test]
fn fora_da_curva_e_mudado() {
    let d = doc();
    assert_eq!(
        channel_mark(&d, E, PropKind::Opacity, 0.5, true, Some(0.3)),
        ChannelMark::Mudado
    );
    assert_eq!(
        channel_mark(&d, E, PropKind::Opacity, 1.0, true, Some(0.3)),
        ChannelMark::Mudado
    );
}
