//! ⭐⭐ **O ROTEADOR DAS CENAS DO SMOKE responde por todas elas** (W97).
//!
//! # ⚠️ O buraco que isto fecha
//!
//! Até aqui **nenhum gate construía uma cena do smoke**. O `scene()` termina em
//! `.expect("as cenas do smoke são documentos válidos")`, então uma cena com um raio que não cabe,
//! um perfil que cruza o eixo ou uma referência para a frente **entra em pânico ao arrancar** — e o
//! primeiro a descobrir seria o Enio, com a janela a fechar-se e uma mensagem que não é para ele.
//!
//! ⚠️ E o segundo lado é mais silencioso: um `n` sem braço no `match` **cai no `_`** e desenha a
//! cena 1. O artista pede a cena nova, vê a de sempre, e conclui que a feature não foi feita. É a
//! família do `no_two_smoke_scenes_claim_the_same_level` dos outros módulos, aqui pela primeira vez.

use ph2d_field::{NodeKind, Op};

/// Quantas cenas o roteador promete. ⚠️ **Sobe com o `match`**, e a nota do `main.rs` e a do
/// `field3d_smoke.rs` sobem com ela — as três dizem a mesma coisa e o gate abaixo prende-as.
const CENAS: u32 = 7;

/// ⭐ **Todas as cenas do roteador CONSTROEM**, e nenhuma é a cena 1 disfarçada.
///
/// ⚠️ A comparação é entre **documentos**, e não entre contagens de nós: duas cenas podem ter o
/// mesmo número de nós e formas diferentes, e um braço em falta produz o documento **igual** ao da
/// cena 1 — que é exactamente o que se quer apanhar.
#[test]
fn every_smoke_scene_builds_and_is_its_own() {
    let docs: Vec<_> = (1..=CENAS).map(crate::field3d_smoke::scene).collect();
    for (i, doc) in docs.iter().enumerate() {
        assert!(
            !doc.nodes().is_empty(),
            "a cena {} saiu sem nós",
            i as u32 + 1
        );
    }
    for i in 1..docs.len() {
        assert_ne!(
            docs[i],
            docs[0],
            "a cena {} é a cena 1 disfarçada — falta o braço dela no `match` do roteador",
            i + 1
        );
    }
}

/// ⭐⭐⭐ **A cena 7 é a que o Enio vê para julgar o verbo por forma** — e ela tem de ter a **forma**
/// que a wave promete, não só um número.
///
/// ⚠️ Um gate que só a construísse passaria com os quatro nós todos calados, que é a peça de sempre.
/// O que se afirma aqui é a **receita**: quatro irmãos num grupo só (⛔ zero aninhamento — é a queixa
/// que abriu a wave), um calado (a herança), e **dois cortes com raios de junção diferentes** — o par
/// que era inexprimível sem dois grupos.
#[test]
fn the_verb_scene_shows_two_cuts_with_different_radii_and_no_nesting() {
    let doc = crate::field3d_smoke::scene(7);
    let raiz = doc.node(doc.root()).expect("a raiz");
    let NodeKind::Combine { children, .. } = &raiz.kind else {
        panic!("a raiz da cena 7 tem de ser o grupo único");
    };
    assert_eq!(children.len(), 4, "quatro irmãos, e num grupo só");
    // ⛔ **Zero aninhamento**: nenhum filho pode ser ele próprio uma combinação.
    for c in children {
        assert!(
            !matches!(
                doc.node(*c).map(|n| &n.kind),
                Some(NodeKind::Combine { .. })
            ),
            "a cena 7 aninhou um grupo — ela existe precisamente para mostrar que não é preciso"
        );
    }
    let verbos: Vec<_> = children
        .iter()
        .map(|c| doc.node(*c).expect("filho").verb)
        .collect();
    assert!(
        verbos[1].is_none(),
        "a 2.ª forma tem de ficar CALADA — é ela que mostra a herança ao lado das que se pronunciam"
    );
    let cortes: Vec<f32> = verbos
        .iter()
        .filter_map(|v| match v {
            Some(Op::Difference(b)) => Some(b.amount()),
            _ => None,
        })
        .collect();
    assert_eq!(cortes.len(), 2, "a cena tem de trazer DOIS cortes");
    assert!(
        (cortes[0] - cortes[1]).abs() > 1e-4,
        "os dois cortes têm o mesmo raio de junção ({cortes:?}) — assim a cena não mostra nada que \
         um grupo só já não fizesse"
    );
}

/// ⚠️ **As duas notas que dizem o alcance do smoke concordam com o roteador.**
///
/// Elas já tinham envelhecido uma vez: o `main.rs` dizia `1..3` com **seis** cenas construídas. *Uma
/// nota que diz o alcance de um roteador é a primeira coisa que alguém lê e a última que alguém
/// actualiza.*
#[test]
fn the_notes_agree_with_the_router() {
    let alcance = format!("PH2D_FIELD_SMOKE=1..{CENAS}");
    for (nome, fonte) in [
        ("field3d_smoke.rs", include_str!("field3d_smoke.rs")),
        ("main.rs", include_str!("main.rs")),
    ] {
        assert!(
            fonte.contains(&alcance),
            "o `{nome}` não diz `{alcance}` — o roteador tem {CENAS} cenas e a nota dele envelheceu"
        );
    }
}
