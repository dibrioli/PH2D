//! Os gates do auto-key — extraídos de `autokey.rs` sob o teto de 700 LOC (HR-18).
//!
//! Segue módulo FILHO (`#[path]`), então `use super::*` alcança os privados exatamente
//! como antes: o corte é de TAMANHO, não de visibilidade.

use super::*;
use crate::state::TimelineState;
use crate::{TimelineIntent as I, apply_intent};
use ph2d_anim::RationalTime;
use ph2d_anim::{AnimValue, Interp};
use ph2d_core::Playhead;

const E: u64 = 1;
/// PoseSample index for each PropKind we probe.
const TX: usize = 0;
const TY: usize = 1;
const ROT: usize = 2;

fn s(t: f64) -> RationalTime {
    RationalTime::from_seconds(t)
}

/// A doc with a TranslationX track keyed 0→10 over 0..1 s. The entity is in
/// **Separate** mode (it has an X binding), so `default_path` is moot for it —
/// which is exactly the point of the tests that use it.
fn doc_with_tx_track() -> TimelineState {
    let mut st = TimelineState::new();
    let mut ph = Playhead::new(1.0 / 60.0);
    for (t, v) in [(0.0, 0.0), (1.0, 10.0)] {
        apply_intent(
            &mut st,
            &mut ph,
            I::AddKey {
                entity: E,
                prop: PropKind::TranslationX,
                t: s(t),
                value: AnimValue::Float(v),
                interp: Interp::Linear,
            },
        );
    }
    st
}

/// A doc with a straight motion path 0→10 in x over 0..1 s — the entity is in
/// **Path** mode (it has a Position binding with a trajectory).
fn doc_with_path() -> TimelineState {
    let mut st = TimelineState::new();
    let mut ph = Playhead::new(1.0 / 60.0);
    for (t, at) in [(0.0, [0.0_f32, 0.0]), (1.0, [10.0, 0.0])] {
        apply_intent(
            &mut st,
            &mut ph,
            I::AddPathKey {
                entity: E,
                t: s(t),
                at,
            },
        );
    }
    st
}

fn pose(vals: &[(usize, f32)]) -> PoseSample {
    let mut p: PoseSample = [None; 7];
    for &(i, v) in vals {
        p[i] = Some(v);
    }
    p
}

/// `autokey_props` with the separate-axes default. Every fixture here that
/// exercises the *scalar* diff is separate-bound (or non-position), so the
/// default is moot; naming it keeps these calls readable. The mode-specific
/// tests below call `autokey_props` directly with the default they mean.
fn ak(
    st: &TimelineState,
    t: f64,
    world: &PoseSample,
    base: &PoseSample,
    allow: bool,
) -> AutokeyPlan {
    autokey_props(&st.doc, E, t, world, base, allow, false)
}

#[test]
fn a_bound_prop_off_its_curve_is_keyed() {
    // At t = 0.5 the curve says x = 5. The world is at 7 (the user dragged it):
    // key it. The other props are None → never keyed.
    let st = doc_with_tx_track();
    let got = ak(&st, 0.5, &pose(&[(TX, 7.0)]), &pose(&[]), true);
    assert_eq!(got.keys, vec![(PropKind::TranslationX, 7.0)]);
}

#[test]
fn a_bound_prop_sitting_on_its_curve_is_not_keyed() {
    // THE anti-feedback case: after an undo/paste/scrub the apply pass writes
    // the curve value to the world, so world == curve — auto-key must be silent
    // or it would re-key what the document just produced, fighting the undo.
    let st = doc_with_tx_track();
    let got = ak(&st, 0.5, &pose(&[(TX, 5.0)]), &pose(&[]), true);
    assert!(got.is_empty(), "on-curve poses key nothing: {got:?}");
}

#[test]
fn an_unbound_prop_that_moved_since_last_frame_auto_creates() {
    // Rotation has no track. It moved from 0 (last frame) to 0.5 (now), and
    // creation is allowed → key it (the shell will upsert, which binds+creates).
    let st = doc_with_tx_track();
    let got = ak(&st, 0.5, &pose(&[(ROT, 0.5)]), &pose(&[(ROT, 0.0)]), true);
    assert_eq!(got.keys, vec![(PropKind::Rotation, 0.5)]);
}

#[test]
fn an_unbound_prop_that_did_not_move_creates_nothing() {
    let st = doc_with_tx_track();
    let got = ak(&st, 0.5, &pose(&[(ROT, 0.5)]), &pose(&[(ROT, 0.5)]), true);
    assert!(
        got.is_empty(),
        "an unchanged unbound prop must not spray a track"
    );
}

#[test]
fn an_unbound_prop_never_auto_creates_when_creation_is_off() {
    // Panel closed → the shell passes allow_create = false → casual editing
    // never sprays new tracks, however far the object moved.
    let st = doc_with_tx_track();
    let got = ak(&st, 0.5, &pose(&[(ROT, 9.0)]), &pose(&[(ROT, 0.0)]), false);
    assert!(got.is_empty());
    // But a BOUND prop still auto-keys with creation off — updating an
    // existing channel is always allowed.
    let got = ak(&st, 0.5, &pose(&[(TX, 7.0)]), &pose(&[]), false);
    assert_eq!(got.keys, vec![(PropKind::TranslationX, 7.0)]);
}

#[test]
fn an_unbound_prop_with_no_baseline_yet_creates_nothing() {
    // First frame an entity is selected: no baseline → nothing to compare, so
    // its mere selection never mints a key.
    let st = doc_with_tx_track();
    let got = ak(&st, 0.5, &pose(&[(ROT, 9.0)]), &pose(&[]), true);
    assert!(got.is_empty());
}

#[test]
fn an_empty_track_counts_as_unbound() {
    // A binding with no keys has no curve to compare to — treat it as unbound
    // so the first edit still creates a key rather than being lost.
    let mut st = TimelineState::new();
    let mut ph = Playhead::new(1.0 / 60.0);
    apply_intent(
        &mut st,
        &mut ph,
        I::Bind {
            entity: E,
            prop: PropKind::TranslationX,
        },
    );
    let got = ak(&st, 0.5, &pose(&[(TX, 3.0)]), &pose(&[(TX, 0.0)]), true);
    assert_eq!(got.keys, vec![(PropKind::TranslationX, 3.0)]);
}

#[test]
fn upserting_the_returned_props_leaves_the_pose_on_its_curve() {
    // End to end: key what autokey_props returns, and next frame the same pose
    // is ON the curve → nothing more to key. This is the loop that must close.
    let mut st = doc_with_tx_track();
    let t = s(0.5);
    let got = ak(&st, t.to_seconds(), &pose(&[(TX, 7.0)]), &pose(&[]), true);
    for (prop, v) in got.keys {
        st.doc
            .upsert_key(E, prop, t, AnimValue::Float(v), Interp::Linear);
    }
    let again = ak(&st, t.to_seconds(), &pose(&[(TX, 7.0)]), &pose(&[]), true);
    assert!(
        again.is_empty(),
        "the keyed pose is now on its own curve: {again:?}"
    );
}

// ── position mode (ADR-0141) ─────────────────────────────────────────────

#[test]
fn an_entity_with_a_path_keys_the_path_not_the_axes() {
    // THE reported conflict: with a motion path, moving the object must author a
    // path ANCHOR, never separate X/Y. At t=0.5 the straight path draws x=5; the
    // user dragged it to (5, 3) — off the path. ⚠️ The baseline MOVES y (0→3):
    // without the mode skip, the unbound TranslationY would first-touch-create
    // here, so the empty-baseline version was green even with the skip deleted.
    // ⚠️ Pela porta SOLO: uma âncora é geometria do clip e só a aba Keys tem um clip
    // escolhido (`the_anchor_is_refused_under_a_stack`). A premissa fica declarada aqui —
    // uma fixture que chega ao estado pelo default do outro door inverte de sentido no dia
    // em que esse default se move, e segue verde testando o oposto.
    let st = doc_with_path();
    let got = autokey_props_solo(
        &st.doc,
        E,
        0.5,
        &pose(&[(TX, 5.0), (TY, 3.0)]),
        &pose(&[(TX, 5.0), (TY, 0.0)]),
        true,
        true,
    );
    assert!(
        got.keys.is_empty(),
        "Path mode must never key the separate axes: {:?}",
        got.keys
    );
    assert_eq!(
        got.path_key,
        Some([5.0, 3.0]),
        "the anchor lands where the object now is"
    );
}

#[test]
fn a_pose_on_its_trajectory_authors_no_anchor() {
    // The anti-feedback guarantee for Path mode: the pose the apply just wrote
    // (path.at(distance)) is byte-equal to what `position_shown` recomputes, so
    // an untouched object mints no anchor per frame. At t=0.5 the path IS (5,0).
    let st = doc_with_path();
    let got = autokey_props(
        &st.doc,
        E,
        0.5,
        &pose(&[(TX, 5.0), (TY, 0.0)]),
        &pose(&[]),
        true,
        true,
    );
    // ⚠️ `is_empty()` inclui o `path_refused`, e é isso que prende o anti-spam: sob uma
    // pilha (esta porta) um objeto PARADO não pode nem keyar nem RECLAMAR, senão a aba
    // Arrange com AutoKey armado cospe um toast por frame.
    assert!(got.is_empty(), "on-trajectory poses key nothing: {got:?}");
}

#[test]
fn separate_axes_win_over_the_path_default() {
    // The default is only for FRESH entities. One already animating X/Y keeps
    // keying X/Y even with the Motion Path toggle on — the two modes never mix.
    let st = doc_with_tx_track();
    let got = autokey_props(&st.doc, E, 0.5, &pose(&[(TX, 7.0)]), &pose(&[]), true, true);
    assert_eq!(got.keys, vec![(PropKind::TranslationX, 7.0)]);
    assert_eq!(
        got.path_key, None,
        "an X/Y entity never grows a path from the default"
    );
}

#[test]
fn a_fresh_entity_takes_the_toggle_default() {
    // No position animation yet → the toggle decides. Same motion, two modes.
    let st = TimelineState::new();
    let world = pose(&[(TX, 4.0), (TY, 2.0)]);
    let base = pose(&[(TX, 0.0), (TY, 0.0)]);

    // Os dois lados pela MESMA porta (solo): o que se compara aqui é o MODO, e trocar a
    // vista junto compararia duas coisas de uma vez.
    let path = autokey_props_solo(&st.doc, E, 0.5, &world, &base, true, true);
    assert_eq!(
        path.path_key,
        Some([4.0, 2.0]),
        "default Path → the first anchor"
    );
    assert!(
        path.keys.is_empty(),
        "Path mode keys no axes: {:?}",
        path.keys
    );

    let sep = autokey_props_solo(&st.doc, E, 0.5, &world, &base, true, false);
    assert_eq!(sep.path_key, None, "default Separate → no path");
    assert_eq!(
        sep.keys,
        vec![(PropKind::TranslationX, 4.0), (PropKind::TranslationY, 2.0)]
    );
}

/// **A âncora é geometria do CLIP, e só a aba Keys tem um clip escolhido** (Enio,
/// 2026-07-31: *"Path editável apenas em Keys: Clips"*).
///
/// A metade de AUTORIA da mesma lei que o overlay aplica ao desenho e às alças. Sob uma
/// pilha a pose na tela é a composição das strips: plantar a âncora ali editaria a
/// geometria de um clip que a aba nem nomeia e reescreveria a distância de TODAS as keys
/// dele — de um gesto que parece local.
///
/// ⚠️ **Presença E ausência sobre o MESMO movimento**, e é o que impede o gate de ser
/// vácuo: a única coisa que muda entre as duas metades é a PORTA. E a recusa é um VALOR —
/// `path_key: None` sozinho seria indistinguível de *"nada aconteceu"*, que é a forma
/// exata de um gesto silenciosamente inerte.
///
/// **Mutação que deve sangrar:** o ramo do `path_key` ignorar o `solo`.
#[test]
fn the_anchor_is_refused_under_a_stack() {
    let st = doc_with_path();
    // O MESMO gesto nas duas: em t=0.5 a trajetória reta desenha x=5, e o objeto foi
    // arrastado para (5, 3) — fora dela.
    let (world, base) = (pose(&[(TX, 5.0), (TY, 3.0)]), pose(&[(TX, 5.0), (TY, 0.0)]));

    let keys_tab = autokey_props_solo(&st.doc, E, 0.5, &world, &base, true, true);
    assert_eq!(
        keys_tab.path_key,
        Some([5.0, 3.0]),
        "na aba Keys a âncora é autorada onde o objeto está"
    );
    assert_eq!(keys_tab.path_refused, None, "e não há o que recusar");

    let stacked = autokey_props(&st.doc, E, 0.5, &world, &base, true, true);
    assert_eq!(
        stacked.path_key, None,
        "fora da aba Keys nenhuma âncora é plantada"
    );
    assert_eq!(
        stacked.path_refused,
        Some(KeyRefusal::PathNeedsKeysTab),
        "e o artista é dono de um motivo: um gesto que não faz nada e não diz nada é \
         indistinguível de uma ferramenta quebrada"
    );
    assert!(
        stacked.keys.is_empty(),
        "recusar a âncora não vira uma segunda porta pelos eixos: {:?}",
        stacked.keys
    );
}
