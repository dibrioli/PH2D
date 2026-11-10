use crate::{TimelineIntent as I, TimelineState, apply_intent, arm_view_loop};
use ph2d_anim::RationalTime;
use ph2d_core::Playhead;

const DT: f64 = 1.0 / 24.0;

/// Um clipe e três marcadores (takeoff 0 · top 0.5 · land 1), na aba Keys sem pilha (o relógio do editor).
fn pulo(loop_do_clipe: Option<(f64, f64)>) -> (TimelineState, Playhead, Playhead) {
    let mut st = TimelineState::new();
    for (t, n) in [(0.0, "takeoff"), (0.5, "top"), (1.0, "land")] {
        st.doc.add_marker(RationalTime::from_seconds(t), n);
    }
    st.doc.set_active_loop_for(true, loop_do_clipe);
    st.keys_mode = true;
    let (mut ed, mut clip) = (Playhead::new(DT), Playhead::new(DT));
    arm_view_loop(&mut st, &mut ed, &mut clip);
    (st, ed, clip)
}

fn quadro(st: &mut TimelineState, ed: &mut Playhead, clip: &mut Playhead) {
    arm_view_loop(st, ed, clip);
}

/// ⭐ Escolher takeoff → top arma `[0, 0.5)` e o Play dá a volta nele; ✕ devolve o loop do clipe — com
/// e sem loop autorado. O documento fica byte-igual e o undo sem passo. *Mutação: o ✕ não devolver.*
#[test]
fn o_trecho_toca_em_loop_e_o_x_devolve_o_loop_do_clipe() {
    for autorado in [None, Some((0.25, 0.75))] {
        let (mut st, mut ed, mut clip) = pulo(autorado);
        let antes = serde_json::to_string(&st.doc).expect("doc");
        assert_eq!(ed.loop_range(), autorado);
        apply_intent(&mut st, &mut ed, I::PlayStretch(Some((0, 1))));
        quadro(&mut st, &mut ed, &mut clip);
        assert_eq!(ed.loop_range(), Some((0.0, 0.5)));
        assert!(ed.is_playing());
        for _ in 0..30 {
            ed.advance();
            assert!(ed.time() < 0.5, "deu a volta no trecho: {}", ed.time());
        }
        // Outra ação qualquer que re-sincroniza o loop não ganha ao trecho no quadro seguinte.
        apply_intent(&mut st, &mut ed, I::SetActiveClip { index: 0 });
        quadro(&mut st, &mut ed, &mut clip);
        assert_eq!(ed.loop_range(), Some((0.0, 0.5)));
        apply_intent(&mut st, &mut ed, I::PlayStretch(None));
        quadro(&mut st, &mut ed, &mut clip);
        assert_eq!(ed.loop_range(), autorado, "o ✕ devolve o loop do clipe");
        assert_eq!(st.doc.active_loop_for(true), autorado);
        assert_eq!(serde_json::to_string(&st.doc).expect("doc"), antes);
        assert!(!st.history.can_undo(), "o trecho não é passo de undo");
    }
}

/// Trocar de clipe, trocar de aba e um marcador que se move largam o trecho.
#[test]
fn trocar_de_clipe_de_aba_ou_mexer_no_marcador_larga_o_trecho() {
    let (mut st, mut ed, mut clip) = pulo(None);
    st.doc.add_clip("walk".into());
    st.doc.set_active(0);
    let arma = |st: &mut TimelineState, ed: &mut Playhead, clip: &mut Playhead| {
        apply_intent(st, ed, I::PlayStretch(Some((1, 2))));
        quadro(st, ed, clip);
        assert_eq!(ed.loop_range(), Some((0.5, 1.0)));
    };
    arma(&mut st, &mut ed, &mut clip);
    st.doc.set_active(1);
    quadro(&mut st, &mut ed, &mut clip);
    assert_eq!((st.trecho, ed.loop_range()), (None, None), "outro clipe");
    arma(&mut st, &mut ed, &mut clip);
    st.keys_mode = false;
    quadro(&mut st, &mut ed, &mut clip);
    assert_eq!((st.trecho, ed.loop_range()), (None, None), "outra aba");
    st.keys_mode = true;
    quadro(&mut st, &mut ed, &mut clip);
    arma(&mut st, &mut ed, &mut clip);
    st.doc.move_marker(2, RationalTime::from_seconds(0.75));
    quadro(&mut st, &mut ed, &mut clip);
    assert_eq!(
        (st.trecho, ed.loop_range()),
        (None, None),
        "o marcador mudou"
    );
    apply_intent(&mut st, &mut ed, I::PlayStretch(Some((1, 0))));
    assert_eq!(st.trecho, None, "fim antes do início não é trecho");
}
