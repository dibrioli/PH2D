//! ⛔⛔⛔ **OS DOIS RELATOS DO DONO SOBRE O AUTO-KEY** (2026-09-17), reproduzidos no caminho do
//! passe — *«ao arrastar o tempo na timeline cria keyframes em todos os quadros»* e *«mover
//! handles não cria keys»*.
//!
//! ⚠️ **Eles são as DUAS metades da mesma pergunta** — *o que conta como uma edição do artista?* —
//! e é por isso que vivem no mesmo ficheiro: uma cura que só olhasse a metade que dispara a mais
//! poderia calar a que dispara a menos, e ao contrário.

use super::test_helpers::*;
use super::*;
use ph2d_timeline::{TimelineIntent as I, apply_intent};

/// Quantas chaves a track de `prop` tem agora (0 se ela nem existe).
fn n_keys(st: &TimelineState, entity: u64, prop: PropKind) -> usize {
    let Some(b) = st.doc.binding_for(entity, prop) else {
        return 0;
    };
    st.doc.active_clip().track(b.target).map_or(0, |t| t.len())
}

/// ⛔⛔⛔ **ARRASTAR O TEMPO NÃO É UMA EDIÇÃO** — com o AutoKey armado e a mão parada, mover o
/// cursor de tempo não pode cunhar uma chave por quadro.
///
/// Report do dono: *«ao arrastar o tempo na timeline cria keyframes em todos os quadros»*.
///
/// ⚠️ **A pose amostrada é a MESMA em todos os quadros** de propósito: é o que o mundo entrega a
/// um objecto que ninguém tocou. Se o passe cunhar aqui, o gatilho é o RELÓGIO e não a mão — que é
/// exactamente o defeito reportado.
#[test]
fn arrastar_o_tempo_com_a_mao_parada_nao_cunha_nada() {
    let (mut st, mut ph) = state_with_tx_track();
    let antes = n_keys(&st, E, PropKind::TranslationX);
    let mut ak = AutokeyState::default();

    // A pose que o apply deixou no mundo em `t = 0,5` (a curva diz `x = 5`).
    let parada = pose(&[(TX, 5.0)]);
    // O artista arrasta o cursor por trinta quadros. `drag_now = false`: a mão está no CURSOR,
    // nunca num gizmo.
    for i in 0..30 {
        ph.seek(0.5 + f64::from(i) * (1.0 / 60.0));
        frame(&mut st, &ph, &[(E, parada)], false, true, &mut ak);
    }

    let depois = n_keys(&st, E, PropKind::TranslationX);
    assert_eq!(
        depois,
        antes,
        "arrastar o tempo cunhou {} chave(s) — o gatilho do auto-key tem de ser a MAO, nunca o \
         relogio",
        depois - antes
    );
}

/// ⛔⛔⛔ **MOVER UMA ALÇA CRIA CHAVE** — com o AutoKey armado, arquear um osso é uma edição como
/// mover um objecto é.
///
/// Report do dono: *«mover handles não cria keys»*. ⚠️ **E ele tem razão por construção:** o
/// [`PoseSample`] tem a forma do `PropKind::AUTOKEYED`, e as quatro alças ficaram de fora quando o
/// canal nasceu — *um canal que o auto-key não amostra é invisível a ele, por mais correcta que a
/// lei dele seja*.
#[test]
fn mover_uma_alca_com_autokey_armado_cunha_a_chave() {
    let mut st = TimelineState::new();
    let mut ph = Playhead::new(1.0 / 60.0);
    // O osso tem uma track de alça — o artista criou-a pelo `+ Track`.
    apply_intent(
        &mut st,
        &mut ph,
        I::AddKey {
            entity: E,
            prop: PropKind::BoneBendInX,
            t: RationalTime::from_seconds(0.0),
            value: AnimValue::Float(0.0),
            interp: ph2d_anim::Interp::Linear,
        },
    );
    ph.seek(1.0);
    ph.pause();
    let antes = n_keys(&st, E, PropKind::BoneBendInX);

    let mut ak = AutokeyState::default();
    // O artista arrasta a alça: a pose deste quadro traz a alça noutro sítio.
    let arrastada = pose_com_alca(1.25);
    frame(&mut st, &ph, &[(E, arrastada)], true, true, &mut ak);

    let depois = n_keys(&st, E, PropKind::BoneBendInX);
    assert!(
        depois > antes,
        "arrastar a alça nao cunhou chave nenhuma ({antes} -> {depois}) — o auto-key nao a AMOSTRA"
    );
}

/// A pose de um osso cuja alça da raiz está em `x`. ⚠️ Ela usa o índice que o `PropKind::AUTOKEYED`
/// dá à `BoneBendInX` — *se ele não estiver lá, isto não compila*, que é o aviso certo.
fn pose_com_alca(x: f32) -> PoseSample {
    let i = PropKind::AUTOKEYED
        .iter()
        .position(|p| *p == PropKind::BoneBendInX)
        .expect("a alça tem de ser um canal que o auto-key amostra");
    let mut p: PoseSample = [None; PropKind::AUTOKEYED.len()];
    p[i] = Some(x);
    p
}
