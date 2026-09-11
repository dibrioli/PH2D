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
//! 2. **As condições do alcance.** Cada uma some em silêncio: sem o
//!    `!is_playing()` o arrasto brigaria com o solver enquanto ele reimpõe a
//!    restrição, e sem o `Translate` um scale de grupo passaria a arrastar rig.
//!    ⚠️ A primeira vale para os DOIS sítios; a segunda é só do sítio que pode
//!    abrir qualquer gesto — no pick de canvas ela não poderia ser falsa, e um
//!    gate incapaz de falhar pelo motivo que alega é pior que nenhum.
//!
//! ⚠️ **A POLÍTICA saiu daqui** (W-JointTools): quanto da cadeia um arrasto
//! carrega passou a ser uma escolha do artista (o rádio da seção Joints) mais a
//! lei do Alt, e as duas moram em `JointTool::drag_reach`, com gate próprio ao
//! lado. O que a shell tem de provar é que **pergunta àquela porta** em vez de
//! decidir aqui — o gate do sinal do Alt viajou para junto da lei.
//!
//! ⚠️ **Três gates deste arquivo já expiraram uma vez, e é a lição dele mesmo:**
//! eles liam a expressão `let carry_rig = …`, um PROXY que a wave seguinte
//! renomeou para `reach`. As asserções continuavam certas e o produto também —
//! o que envelheceu foi o endereço. Reescritas sobre a propriedade.
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

/// Cada expressão `let carry_reach = …;` do despacho de ponteiro.
///
/// ⚠️ Perguntada por CONTEÚDO e não por posição: qual das duas é a alça e qual
/// é o pick de canvas é exatamente o tipo de fato que a próxima wave reordena
/// (`the_dispatch_is_handed_the_live_geometry`, 2026-07-23).
///
/// ⚠️ E o nome é `carry_reach` e não `reach` porque a 1ª versão desta wave usou
/// o curto e o scanner colheu **três** — o `input_dispatch` já tinha um `reach`
/// geométrico, sem relação nenhuma. Um scanner por nome herda toda homonímia do
/// arquivo, e o preço de escolher um nome específico é zero.
fn reach_exprs(src: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = src;
    while let Some(i) = rest.find("let carry_reach =") {
        let after = &rest[i..];
        let end = after.find("};").expect("expressão `reach` sem fechamento");
        out.push(after[..end].to_string());
        rest = &after[end..];
    }
    out
}

/// **Todo sítio pergunta o alcance à porta única, com o relógio e o Alt CRU.**
///
/// Três afirmações, cada uma com o seu modo de falha:
/// - `is_playing()` — em play a pose é do solver, que reimpõe a restrição no
///   tick seguinte de qualquer forma;
/// - `drag_reach(...)` — a política é do modelo, e decidi-la aqui é a segunda
///   cópia da regra (foi assim que o `Links` sumiu quando o Alt virou `Whole`);
/// - o Alt entra **sem negação** — `!alt_key()` reintroduziria a polaridade que
///   o Enio inverteu em 2026-07-26, e o gate do SIGNIFICADO dela mora agora na
///   `JointTool` (`alt_always_means_the_whole_rig_and_never_a_pose`).
#[test]
fn every_site_asks_the_reach_door_at_rest_with_the_raw_alt() {
    let src = source();
    let exprs = reach_exprs(&src);
    assert_eq!(
        exprs.len(),
        2,
        "esperados 2 `carry_reach`, achados {}",
        exprs.len()
    );
    for expr in &exprs {
        assert!(
            expr.contains("self.playhead.is_playing()"),
            "a condição do relógio sumiu de um `carry_reach`:\n{expr}"
        );
        assert!(
            expr.contains("self.interaction.joint.drag_reach(self.modifiers.alt_key())"),
            "o alcance tem de vir da porta única, com o Alt cru:\n{expr}"
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
    let exprs = reach_exprs(&src);
    assert!(
        exprs.iter().any(|e| e.contains("GizmoDragKind::Translate")),
        "nenhum `carry_reach` gateia no tipo do gesto; um scale de grupo arrastaria rig"
    );
}

/// **O arrasto expande pela política que RECEBEU, nunca por uma cravada.**
///
/// Existem três portas com a MESMA assinatura — `jointed_rig` (todo tipo
/// conduz), `jointed_group` (só Dynamic, a política do BAKE) e `jointed_by` (a
/// que pergunta) — então trocar uma pela outra compila em silêncio. Cravar
/// qualquer uma das duas primeiras aqui apaga metade do rádio: o modo `Links`
/// passaria a carregar o gancho, ou o `Rig` a deixá-lo para trás, e o artista
/// veria dois chips fazendo a mesma coisa.
#[test]
fn the_drag_expands_by_the_policy_it_was_handed() {
    let door = fs::read_to_string("src/joint_rig_drag.rs").expect("joint_rig_drag.rs");
    assert!(
        door.contains("ph2d_physics_ecs::jointed_by(sim.world_mut(), &seed, reach)"),
        "o arrasto tem de expandir por `jointed_by` com o alcance que recebeu"
    );
    for hard in ["jointed_rig(", "jointed_group("] {
        assert!(
            !door.contains(hard),
            "`{hard}` cravado no arrasto apaga metade do rádio de modos"
        );
    }
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
