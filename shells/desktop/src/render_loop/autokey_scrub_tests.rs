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

/// ⛔⛔⛔ **O ARRASTO COM *SNAP*, que é a condição da FOTO do dono** (2.º report de 2026-09-17:
/// *«criando keys de todo modo ao arrastar o tempo da timeline»*, com quatro tracks inundadas).
///
/// ⚠️⚠️ **Esta é a metade que a 1.ª cura não apanhava.** Com o Snap ligado o instante salta de
/// quadro em quadro e fica **PARADO** entre saltos, logo os quadros de ecrã do meio liam *«o
/// relógio não andou»* e voltavam a capturar — e uma pose que ainda não assentou cunhava ali.
///
/// ⇒ o guarda passou a ser o **GESTO** (`Scrub`/`SeekFrame` no dreno), não a consequência dele.
///
/// (Red-first: sem o `scrub_now`, este gate reprova enquanto o irmão sem snap passa — que é
/// exactamente a diferença entre os dois reports.)
#[test]
fn arrastar_o_tempo_com_snap_tambem_nao_cunha_nada() {
    let (mut st, mut ph) = state_with_tx_track();
    let antes = n_keys(&st, E, PropKind::TranslationX);
    let mut ak = AutokeyState::default();
    // ⚠️ A pose NÃO segue a curva de propósito: é o caso em que o mundo ainda não assentou (um
    // osso que um motor reposiciona, a malha a recozer). Sem o guarda, cada quadro destes cunha.
    let atrasada = pose(&[(TX, 5.0)]);
    for i in 0..40 {
        let passo = i / 4; // o SNAP: o instante salta so' a cada quatro quadros de ecra
        ph.seek(0.5 + f64::from(passo) * (1.0 / 60.0));
        // A mão está na régua em TODOS os quadros do arrasto — é isso que o dreno regista.
        ak.scrub_now = true;
        frame(&mut st, &ph, &[(E, atrasada)], false, true, &mut ak);
    }
    let depois = n_keys(&st, E, PropKind::TranslationX);
    assert_eq!(
        depois,
        antes,
        "o arrasto COM SNAP cunhou {} chave(s) — o relogio fica parado entre saltos, e e' por isso          que o guarda tem de ser o GESTO",
        depois - antes
    );
}

/// ⏱️ **SONDA — o arrasto REAL, com o mundo escrito pelo `apply`**.
///
/// ⚠️ A sonda anterior calculava a pose a' mao e media o proprio arredondamento dela. Aqui o
/// `apply_from_doc` escreve o mundo (como no produto) e a pose LE-SE de la': se aparecerem chaves,
/// elas sao do passe e nao da fixtura.
#[test]
fn sonda_arrasto_real() {
    use ph2d_ecs::{Entity, Transform, World};
    let (mut st, mut ph) = state_with_tx_track();
    let mut w = World::new();
    let ent = w.spawn(Transform::default()).id();
    // A track do arnes fala da entidade `E`; a do mundo tem de ser a MESMA.
    let bits = ent.to_bits();
    let mut doc2 = st.doc.clone();
    for (t, v) in [(0.0, 0.0_f32), (1.0, 10.0)] {
        doc2.insert_key(
            bits,
            PropKind::TranslationX,
            RationalTime::from_seconds(t),
            AnimValue::Float(v),
            ph2d_anim::Interp::Linear,
        );
    }
    st.doc = doc2;
    let mut ak = AutokeyState::default();
    let antes = n_keys(&st, bits, PropKind::TranslationX);
    for i in 0..40 {
        let passo = i / 4; // o SNAP: o tempo salta so' a cada 4 quadros de ecra
        let t = 0.5 + f64::from(passo) * (1.0 / 60.0);
        ph.seek(t);
        ph2d_timeline::apply_from_doc(&mut w, &mut st.doc, t);
        let x = w
            .get::<Transform>(Entity::from_bits(bits))
            .unwrap()
            .translation
            .x;
        frame(
            &mut st,
            &ph,
            &[(bits, pose(&[(TX, x)]))],
            false,
            true,
            &mut ak,
        );
    }
    println!(
        "ARRASTO REAL: {} chave(s) novas em 40 quadros",
        n_keys(&st, bits, PropKind::TranslationX) - antes
    );
}
