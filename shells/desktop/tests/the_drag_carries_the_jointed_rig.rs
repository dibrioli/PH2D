//! **O Down semeia o rig pela porta única, com as três condições** — arch-gate
//! sobre a costura que nenhum unit test alcança (W-JG).
//!
//! A LEI é gateada onde ela mora: `joint_rig_drag`'s own tests drive a real
//! `SimWorld` headless and prove which bodies a rig carries. O que eles não
//! podem tocar é o pointer-Down, que precisa de `App` + `HeroScreen` + janela.
//! Duas coisas se decidem lá e só lá:
//!
//! 1. **Os DOIS sítios de Down chamam a MESMA porta.** Um gizmo abre por dois
//!    caminhos — a alça (`is_specific_handle`) e o pick de canvas — e antes
//!    desta wave cada um carregava a sua cópia da semeadura de grupo. Duas
//!    cópias é como arrastar pela alça passaria a carregar a corrente e
//!    arrastar pelo corpo, não.
//! 2. **As condições do `carry_rig`.** Cada uma some em silêncio: sem o
//!    `!is_playing()` o arrasto brigaria com o solver enquanto ele reimpõe a
//!    restrição; sem o `alt_key()` o rig deixaria de ser **opt-in** e todo
//!    arrasto passaria a levar a corrente; e sem o `Translate` um scale de grupo
//!    passaria a arrastar rig. ⚠️ As duas primeiras valem para os DOIS sítios; a
//!    terceira é só do sítio que pode abrir qualquer gesto — no pick de canvas
//!    ela não poderia ser falsa, e um gate incapaz de falhar pelo motivo que
//!    alega é pior que nenhum.
//!
//! ⚠️ **O sinal do Alt é afirmado, não só a presença dele** (Enio inverteu a
//! polaridade em 2026-07-26): `alt_key()` casa tanto com `!alt` quanto com `alt`,
//! então um gate que só procura o nome do modificador passa por cima da
//! inversão inteira.
//!
//! Nada aqui afirma distância em bytes ou vizinhança de linhas: a lição de
//! `the_dispatch_is_handed_the_live_geometry` (2026-07-23) é que um proxy
//! posicional expira na wave seguinte. O que se afirma é *quem é chamado* e
//! *com que argumentos*.

use std::fs;

fn source() -> String {
    fs::read_to_string("src/input_dispatch.rs").expect("input_dispatch.rs")
}

/// Todas as chamadas da porta, uma string por chamada (do nome até o `);`).
fn seed_calls(src: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = src;
    while let Some(i) = rest.find("crate::joint_rig_drag::seed_group_drag_starts(") {
        let after = &rest[i..];
        let end = after
            .find(");")
            .expect("chamada de seed_group_drag_starts sem fechamento");
        out.push(after[..end].to_string());
        rest = &after[end..];
    }
    out
}

/// **Os dois Downs que abrem um drag semeiam pela mesma porta.**
///
/// Mutação: devolver qualquer um dos sítios à cópia inline (ou apagá-lo) deixa
/// a contagem em 1 — e o rig passa a depender de POR ONDE se pegou o corpo.
#[test]
fn both_pointer_down_sites_seed_through_the_one_door() {
    let src = source();
    let calls = seed_calls(&src);
    assert_eq!(
        calls.len(),
        2,
        "esperados exatamente 2 chamadores (alça do gizmo + pick de canvas), achados {}",
        calls.len()
    );
    // E a cópia antiga não voltou por baixo: ninguém mais empurra um
    // `GroupDragSnapshot` à mão no despacho de ponteiro.
    assert!(
        !src.contains("GroupDragSnapshot {"),
        "o Down voltou a construir snapshots de grupo à mão — a porta única morreu"
    );
}

/// Cada expressão `let carry_rig = …;` do despacho de ponteiro.
///
/// ⚠️ Perguntada por CONTEÚDO e não por posição: qual das duas é a alça e qual
/// é o pick de canvas é exatamente o tipo de fato que a próxima wave reordena
/// (`the_dispatch_is_handed_the_live_geometry`, 2026-07-23).
fn carry_rig_exprs(src: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = src;
    while let Some(i) = rest.find("let carry_rig =") {
        let after = &rest[i..];
        let end = after.find(';').expect("expressão `carry_rig` sem `;`");
        out.push(after[..end].to_string());
        rest = &after[end..];
    }
    out
}

/// **Todo sítio que carrega o rig pergunta pelo relógio E pelo Alt — e o Alt
/// ARMA, não escapa.**
///
/// As duas condições que valem para os DOIS caminhos: em play a pose é do
/// solver (que reimpõe a restrição no tick seguinte de qualquer forma), e o rig
/// inteiro é **opt-in por gesto**.
///
/// ⚠️ A metade do SINAL é o que torna este gate capaz de pegar a inversão:
/// procurar só `alt_key()` casaria com a polaridade antiga (`!alt`, o Alt como
/// escape) e o gate ficaria verde sobre o comportamento oposto ao pedido.
#[test]
fn every_site_carries_the_rig_only_at_rest_and_with_alt_held() {
    let src = source();
    let exprs = carry_rig_exprs(&src);
    assert_eq!(
        exprs.len(),
        2,
        "esperados 2 `carry_rig`, achados {}",
        exprs.len()
    );
    for expr in &exprs {
        assert!(
            expr.contains("!self.playhead.is_playing()"),
            "a condição do relógio sumiu de um carry_rig:\n{expr}"
        );
        assert!(
            expr.contains("&& self.modifiers.alt_key()"),
            "o Alt tem de ARMAR o rig (`&& self.modifiers.alt_key()`):\n{expr}"
        );
        assert!(
            !expr.contains("!self.modifiers.alt_key()"),
            "o Alt voltou a ser ESCAPE (`!alt`) — a polaridade foi invertida em \
             2026-07-26 por ordem do Enio:\n{expr}"
        );
    }
}

/// **O sítio que pode abrir qualquer gesto pergunta TAMBÉM o tipo.**
///
/// A alça abre scale / rotate / translate conforme o chip agarrado, então sem
/// esta condição um scale de grupo passaria a arrastar rig — e a semântica de
/// rotação/escala de um rig é uma decisão de pivô que esta wave não tomou.
///
/// ⚠️ O pick de canvas NÃO é cobrado disto de propósito: o `GizmoDragState`
/// dele traz `GizmoDragKind::Translate` literal, então a condição não poderia
/// ser falsa ali — seria um gate incapaz de falhar pelo motivo que alega.
#[test]
fn the_site_that_can_open_any_gesture_also_asks_for_a_translate() {
    let src = source();
    let exprs = carry_rig_exprs(&src);
    assert!(
        exprs.iter().any(|e| e.contains("GizmoDragKind::Translate")),
        "nenhum carry_rig gateia no tipo do gesto; um scale de grupo arrastaria rig"
    );
}

/// **O arrasto usa a porta do RIG, não a do BAKE.**
///
/// As duas existem e respondem perguntas diferentes (`jointed_group` = *quem
/// congela?*, só Dynamic; `jointed_rig` = *quem tem de andar junto?*, todo
/// corpo). Trocar uma pela outra aqui é o modo de falha que o Enio reportou —
/// a corrente que não arrasta o gancho — e ele compila em silêncio, porque as
/// duas têm a MESMA assinatura.
#[test]
fn the_drag_asks_the_rig_door_not_the_bake_door() {
    let door = fs::read_to_string("src/joint_rig_drag.rs").expect("joint_rig_drag.rs");
    let call = door
        .find("ph2d_physics_ecs::jointed_rig(")
        .map(|_| "rig")
        .or_else(|| {
            door.find("ph2d_physics_ecs::jointed_group(")
                .map(|_| "group")
        });
    assert_eq!(
        call,
        Some("rig"),
        "o arrasto tem de expandir por `jointed_rig` (todo tipo conduz); \
         `jointed_group` é a política do BAKE e deixaria o gancho para trás"
    );
}

/// **`Alt` não tem um segundo dono neste gesto.**
///
/// O `GizmoModifiers.alt` que o Down monta alimenta o
/// `compute_gizmo_transform`, e lá o Translate NÃO o lê — é por isso que Alt
/// pôde virar o escape do rig sem disputar significado. Se alguém der um
/// sentido de Translate ao Alt na matemática do gizmo, este gate cai e a
/// escolha volta à mesa.
#[test]
fn alt_is_inert_in_the_translate_math_it_shares_the_gesture_with() {
    let math = fs::read_to_string("../../crates/ph2d-editor-core/src/gizmo/transform.rs")
        .expect("gizmo/transform.rs");
    let i = math
        .find("GizmoDragKind::Translate => {")
        .expect("o braço Translate do compute_gizmo_transform sumiu");
    let arm = &math[i..];
    let end = arm
        .find("GizmoDragKind::ScaleCorner")
        .expect("o braço seguinte sumiu");
    assert!(
        !arm[..end].contains("alt"),
        "o Translate passou a ler `alt` — o escape do rig (W-JG) precisa de outro modificador"
    );
}
