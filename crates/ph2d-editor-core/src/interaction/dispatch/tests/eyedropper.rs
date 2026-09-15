//! ⭐⭐⭐ **O CONTA-GOTAS: o botão faz o trabalho no PRESSIONAR, e o `Click` do soltar é redundante.**
//!
//! ⛔⛔ **Este ficheiro nasceu de uma linha de LOG do dono** (2026-09-15, na corrida do
//! `PH2D_PICK_LOG`): `[hero] unhandled event: Click(NodeId(3001329642747827011))`, logo antes de uma
//! amostragem que **funcionou**. O id foi invertido contra os `3 431` literais de `hash_node_id` do
//! repo e é o `blender_eyedropper` — o botão do próprio conta-gotas que ele estava a testar.
//!
//! ⚠️⚠️ **E o achado é que NÃO é um controlo morto: é o DETECTOR a gritar lobo.** O botão arma o modo
//! no `Down` (tem de ser ali — o `Down` SEGUINTE é a colheita), então o `Click` que o `Up` produz
//! chega depois de tudo já feito e ninguém o consome. O `expected_unhandled` da shell passa a
//! conhecê-lo, e **este ficheiro é o que torna isso honesto**: silenciar um diagnóstico sem prova por
//! trás é armengo, e a prova é medir a rota que o silêncio deixa de vigiar.
//!
//! ⚠️ **O custo da falsa alarme já está medido:** aquela linha ficou *«nomeada e por medir»* durante
//! duas waves precisamente porque ninguém conseguia separá-la de uma costura morta a sério.

use super::*;
use crate::interaction::BlenderHitKind;

/// O botão do conta-gotas e um alvo qualquer ao lado dele.
fn conta_gotas() -> (WidgetStore, HitIndex) {
    let mut store = WidgetStore::with_capacity(4);
    store.register(
        BOTAO,
        InteractiveState::BlenderHit {
            parent: PAI,
            kind: BlenderHitKind::Eyedropper,
        },
    );
    store.register(
        ALVO,
        InteractiveState::Button {
            state: ButtonState::Normal,
        },
    );
    let mut hits = HitIndex::new();
    hits.register(BOTAO, Rect::new(0.0, 0.0, 20.0, 20.0));
    hits.register(ALVO, Rect::new(100.0, 100.0, 40.0, 40.0));
    (store, hits)
}

const PAI: NodeId = NodeId(900);
const BOTAO: NodeId = NodeId(901);
const ALVO: NodeId = NodeId(902);

/// ⭐⭐⭐ **O BOTÃO ARMA NO PRESSIONAR, E A COLHEITA É O PRESSIONAR SEGUINTE.**
///
/// É esta ordem que obriga o trabalho a ser feito no `Down`: se ele esperasse pelo `Up`, o gesto do
/// artista — carregar no botão e depois carregar na cor — teria o modo armado tarde demais.
#[test]
fn the_eyedropper_arms_on_down_and_the_next_down_harvests() {
    let (mut store, hits) = conta_gotas();
    let arena = Bump::new();
    let _ = dispatch_pointer(
        &mut store,
        &hits,
        pointer(PointerKind::Down, 10.0, 10.0),
        &arena,
    );
    assert_eq!(
        store.eyedropper_pending(),
        Some(PAI),
        "o `Down` no botão não armou o modo — o clique seguinte não colheria cor nenhuma"
    );
    let arena = Bump::new();
    let evts = dispatch_pointer(
        &mut store,
        &hits,
        pointer(PointerKind::Down, 120.0, 120.0),
        &arena,
    );
    assert!(
        evts.iter()
            .any(|e| matches!(e, WidgetEvent::EyedropperPick { parent, .. } if *parent == PAI)),
        "o `Down` seguinte não emitiu a colheita: {evts:?}"
    );
    assert_eq!(
        store.eyedropper_pending(),
        None,
        "o modo tem de desarmar-se depois de colher, senão o clique a seguir colhe outra vez"
    );
}

/// ⭐⭐ **CARREGAR NO BOTÃO OUTRA VEZ CANCELA** — a saída do modo sem colher nada.
#[test]
fn clicking_the_eyedropper_twice_cancels_the_pick() {
    let (mut store, hits) = conta_gotas();
    for _ in 0..2 {
        let arena = Bump::new();
        let _ = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Down, 10.0, 10.0),
            &arena,
        );
    }
    assert_eq!(
        store.eyedropper_pending(),
        None,
        "o segundo `Down` no botão tinha de cancelar — sem isso o modo só sai colhendo"
    );
}

/// ⭐⭐⭐ **O `Click` DO SOLTAR CHEGA, E ELE É REDUNDANTE POR CONSTRUÇÃO.**
///
/// ⚠️ **Este gate afirma o FACTO que a lista de excepções da shell (`expected_unhandled`) usa como
/// premissa.** Sem ele, aquela lista seria um silêncio sobre uma coisa que ninguém mediu — e a
/// mutação que apagasse o braço do `Down` deixaria o detector calado e o botão morto.
#[test]
fn the_up_still_emits_a_click_that_nobody_needs() {
    let (mut store, hits) = conta_gotas();
    let arena = Bump::new();
    let _ = dispatch_pointer(
        &mut store,
        &hits,
        pointer(PointerKind::Down, 10.0, 10.0),
        &arena,
    );
    // O trabalho JÁ ESTÁ FEITO aqui.
    assert_eq!(store.eyedropper_pending(), Some(PAI));
    let arena = Bump::new();
    let evts = dispatch_pointer(
        &mut store,
        &hits,
        pointer(PointerKind::Up, 10.0, 10.0),
        &arena,
    );
    assert!(
        evts.iter()
            .any(|e| matches!(e, WidgetEvent::Click(id) if *id == BOTAO)),
        "o `Up` deixou de emitir o `Click` — a premissa da lista de excepções da shell mudou: {evts:?}"
    );
    // ⛔ E o `Up` NÃO pode desarmar o modo: entre o `Up` do botão e o `Down` da colheita o artista
    // move o rato, e um modo que morresse aqui nunca colheria nada.
    assert_eq!(
        store.eyedropper_pending(),
        Some(PAI),
        "o `Up` desarmou o modo — o conta-gotas passaria a nunca colher"
    );
}

/// ⭐⭐⭐ **A LEI DA ISENÇÃO É DA FAMÍLIA, E SÓ DELA** — o gate das DUAS metades.
///
/// ⚠️ Sem a segunda metade, uma isenção alargada ao TIPO `Click` deixaria o detector de costura
/// morta da shell cego para **todo botão do app** — que é exactamente a classe de defeito que ele
/// existe para apanhar, e a que o `CLAUDE.md` §5.0 diz que nenhuma outra sonda vê.
#[test]
fn only_the_colour_picker_family_is_exempt_from_the_dead_seam_detector() {
    let (mut store, _) = conta_gotas();
    assert!(
        crate::interaction::work_already_done_on_down(&store, BOTAO),
        "o botão do selector deixou de ser isento: o log do dono enche-se de lobo"
    );
    assert!(
        !crate::interaction::work_already_done_on_down(&store, ALVO),
        "a isenção alargou para o TIPO: o detector ficou cego a todo botão do app"
    );
    // ⛔ E um id que a loja NÃO conhece grita também — é a forma de um widget pintado e nunca
    // registado, que é metade do defeito que este detector procura.
    assert!(
        !crate::interaction::work_already_done_on_down(&store, NodeId(123_456)),
        "um id desconhecido foi isento: um widget nunca registado passaria calado"
    );
    // E o controlo do próprio gate: a loja de facto conhece os dois.
    assert!(store.get(BOTAO).is_some() && store.get(ALVO).is_some());
    let _ = &mut store;
}

// ── As excepções que já existiam, mudadas de casa com a lei ──────────────────────────────────
/// Uma loja com um widget do selector de cor e um botão qualquer ao lado.
fn loja() -> WidgetStore {
    let mut store = WidgetStore::with_capacity(4);
    store.register(
        crate::ids::BLENDER_EYEDROPPER,
        InteractiveState::BlenderHit {
            parent: crate::ids::INSP_BLENDER_PICKER,
            kind: BlenderHitKind::Eyedropper,
        },
    );
    store
}

/// **O picker é isento, e QUALQUER outro `ValueChanged` continua a gritar.**
///
/// ⚠️ As duas metades são o gate. Sem a segunda, calar `WidgetEvent::ValueChanged` inteiro
/// passaria — e o detector de seam morto ficaria cego para todo slider e chip do app, que é
/// a classe de bug que ele existe para apanhar (um widget pintado, registado e MUDO).
#[test]
fn the_picker_is_exempt_but_every_other_value_changed_still_reports() {
    let picker = WidgetEvent::ValueChanged(crate::ids::INSP_BLENDER_PICKER);
    assert!(
        crate::interaction::unhandled_is_expected(&picker, &loja()),
        "o picker nao foi isento: o log volta a uma linha por frame de arrasto"
    );

    // Um id qualquer que NÃO é o picker — o controle.
    let other = WidgetEvent::ValueChanged(NodeId(777));
    assert!(
        !crate::interaction::unhandled_is_expected(&other, &loja()),
        "a isencao alargou para o TIPO: o detector de seam morto ficou cego"
    );
}

/// **Focus/Blur seguem isentos** — a isenção que já existia não pode cair na reescrita.
#[test]
fn focus_and_blur_stay_exempt() {
    let id = crate::ids::INSP_BLENDER_PICKER;
    assert!(crate::interaction::unhandled_is_expected(
        &WidgetEvent::Focus(id),
        &loja()
    ));
    assert!(crate::interaction::unhandled_is_expected(
        &WidgetEvent::Blur(id),
        &loja()
    ));
}

/// ⭐⭐⭐ **O `Click` ATRAVESSA a lista de excepções — e só o da família.**
///
/// ⚠️⚠️ **Este gate nasceu de DUAS mutações sobreviventes na mudança de casa:** os gates que mediam
/// o braço do `Click` ficaram a medir a PORTA de baixo (`work_already_done_on_down`) e ninguém
/// exercitava o `unhandled_is_expected` com um `Click` — logo alargar a isenção ao TIPO inteiro, ou
/// apagá-la, deixava a suíte verde. *Mover uma lei sem mover a régua que a atravessa deixa um gate
/// a medir a metade de baixo.*
#[test]
fn a_click_crosses_the_exemption_list_and_only_the_family_is_let_through() {
    let (store, _) = conta_gotas();
    assert!(
        crate::interaction::unhandled_is_expected(&WidgetEvent::Click(BOTAO), &store),
        "o `Click` do selector voltou a gritar: o log do dono enche-se de lobo"
    );
    assert!(
        !crate::interaction::unhandled_is_expected(&WidgetEvent::Click(ALVO), &store),
        "a isenção alargou para o TIPO `Click`: o detector ficou cego a todo botão do app"
    );
}
