//! ⭐⭐ **Add Clip acima do tecto DIZ porque não** (onda 4, item 4) — com o clique REAL no `+`, a
//! intenção aplicada ao documento e a recusa drenada para a fila de toasts pela porta que a shell
//! chama (`speak_clip_refusals`).
//!
//! ⛔ Antes: dezasseis clips, um clique, nada — o motor devolvia `None` e o arm engolia-o.

use ph2d_editor_core::ToastQueue;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_timeline::state::{TimelinePanelState, set_current_timeline};
use ph2d_panel_timeline::{TimelinePanel, ids, speak_clip_refusals};
use ph2d_timeline::{
    ClipRefusal, MAX_CLIPS, SCENE, TimelineState, TimelineViewSnapshot, apply_intent,
};
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect::new(0.0, 0.0, 1600.0, 900.0);

fn playhead() -> ph2d_core::Playhead {
    ph2d_core::Playhead::new(1.0 / 60.0)
}

/// A Cena com `n` clips (o «Main» incluído).
fn cena_com(n: usize) -> TimelineState {
    let mut st = TimelineState::new();
    for i in 1..n {
        st.doc.add_clip_for(format!("c{i}"), SCENE).unwrap();
    }
    assert_eq!(st.doc.clips_of(SCENE).count(), n);
    st
}

/// Pinta, clica no meio de `id` e aplica o que o painel pediu ao documento.
fn clica_e_aplica(st: &mut TimelineState, id: ph2d_editor_core::NodeId) {
    let _ = ph2d_panel_timeline::drain_intents();
    let mut snap = TimelineViewSnapshot::default();
    snap.rebuild(st, &playhead(), false);
    set_current_timeline(Some(snap));
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut panel = TimelinePanelState::default();
    let regs = host.paint::<TimelinePanel>(&mut panel, VIEWPORT);
    let r = regs
        .iter()
        .find(|(w, _)| *w == id)
        .map(|(_, r)| *r)
        .unwrap_or_else(|| panic!("{id:?} não foi pintado/registado"));
    for ev in host.click_at(r.x + r.w * 0.5, r.y + r.h * 0.5) {
        host.apply_panel_event::<TimelinePanel>(&mut panel, ev);
    }
    let mut ph = playhead();
    for p in ph2d_panel_timeline::drain_intents() {
        apply_intent(st, &mut ph, p);
    }
}

/// ⭐⭐ Dezasseis clips + clicar Add Clip ⇒ nenhum clip novo E um toast com a frase da chave.
///
/// **Mutações que devem sangrar:** tirar o `state.refusals.push(r)` do arm `AddClip` · o
/// `speak_clip_refusals` não empurrar o toast.
#[test]
fn o_mais_cheio_recusa_em_voz_alta() {
    let mut st = cena_com(MAX_CLIPS);
    let antes = st.doc.clips().len();
    assert!(!st.history.can_undo());
    clica_e_aplica(&mut st, ids::TIMELINE_ADD_CLIP);
    assert_eq!(st.doc.clips().len(), antes, "nasceu um clip acima do tecto");
    assert!(!st.history.can_undo(), "a recusa empilhou um undo vazio");
    assert_eq!(st.refusals, [ClipRefusal::AnimationFull]);

    let mut toasts = ToastQueue::new();
    speak_clip_refusals(&mut st, &mut toasts);
    let frases: Vec<&str> = toasts.iter().map(|t| t.message.as_str()).collect();
    let esperada = ph2d_i18n::tr_with(
        ClipRefusal::AnimationFull.message_key(),
        &[("max", &MAX_CLIPS)],
    );
    assert_eq!(frases, [esperada.as_str()], "o artista não leu porquê");
    assert_ne!(
        esperada,
        ClipRefusal::AnimationFull.message_key(),
        "a chave não está na tabela — o toast mostraria a chave crua"
    );
    assert!(esperada.contains(&MAX_CLIPS.to_string()), "{esperada}");
    assert!(st.refusals.is_empty(), "drenar esvazia");
}

/// O Duplicate no tecto recusa pela MESMA porta.
#[test]
fn o_duplicate_no_tecto_tambem_diz() {
    let mut st = cena_com(MAX_CLIPS);
    let antes = st.doc.clips().len();
    clica_e_aplica(&mut st, ids::TIMELINE_DUP_CLIP);
    assert_eq!(st.doc.clips().len(), antes);
    assert_eq!(st.refusals, [ClipRefusal::AnimationFull]);
}

/// CONTROLO: abaixo do tecto o `+` cria e não recusa nada.
#[test]
fn abaixo_do_tecto_cria_calado() {
    let mut st = cena_com(MAX_CLIPS - 1);
    let antes = st.doc.clips().len();
    clica_e_aplica(&mut st, ids::TIMELINE_ADD_CLIP);
    assert_eq!(st.doc.clips().len(), antes + 1);
    assert!(st.refusals.is_empty());
}
