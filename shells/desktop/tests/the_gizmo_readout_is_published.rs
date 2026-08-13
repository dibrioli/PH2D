//! **Arch-gate: o número do arrasto é PUBLICADO, e sai do `Transform` vivo.**
//!
//! ⚠️ Os quinze gates de `editor-core` medem a lei (o número, o formato, a ficha, o pouso) e
//! ficariam todos VERDES com uma shell que nunca publica — a feature existiria e não alcançaria
//! nada que o artista toca. A shell é um **binário**, então `shells/desktop/tests/` não a pode
//! importar: estes gates lêem o FONTE, cada um com **controle positivo**, para um ficheiro
//! renomeado dar falha alta em vez de varredura vazia.

const SNAPSHOTS: &str = include_str!("../src/render_loop/snapshots.rs");
const PUBLISH: &str = include_str!("../src/render_loop/gizmo_readout.rs");

/// ⚠️ **CONTROLE POSITIVO.**
#[test]
fn the_scanned_files_are_the_real_ones() {
    assert!(
        SNAPSHOTS.contains("hero.gizmo.view = hero"),
        "o `snapshots.rs` deixou de publicar a view do gizmo: os gates abaixo medem outra coisa"
    );
    assert!(
        PUBLISH.contains("pub(super) fn publish"),
        "o publicador do número mudou de dono"
    );
}

/// **O número é publicado ao lado da view**, no mesmo passe que reconstrói o gesto em curso.
///
/// *Mutação que sangra:* apagar a chamada — a ficha nunca aparece, e nenhum gate de `editor-core`
/// repara (todos eles armam o `HeroScreen` à mão).
#[test]
fn the_publish_runs_beside_the_view() {
    let view = SNAPSHOTS
        .find("hero.gizmo.view = hero")
        .expect("o controle positivo já teria falhado");
    let pubs = SNAPSHOTS
        .find("gizmo_readout::publish(")
        .expect("o `snapshots.rs` não publica o número do arrasto: a ficha nunca aparece");
    assert!(
        pubs > view,
        "o número é publicado ANTES da view ({pubs} < {view}): ele descreveria o gesto do quadro \
         anterior"
    );
}

/// ⭐ **O número sai do `Transform` VIVO — nunca de uma segunda derivação a partir do cursor.**
///
/// O encaixe (Ctrl na posição, Shift no ângulo) vive dentro do que foi escrito; um número
/// re-derivado do cursor diria `12,03` com a forma pousada em `12,00`.
///
/// *Mutação que sangra:* trocar a leitura do mundo por `compute_gizmo_transform` sobre o arrasto.
#[test]
fn the_number_comes_from_the_live_transform_not_from_the_cursor() {
    assert!(
        PUBLISH.contains("sim.world().get::<Transform>(entity)"),
        "o publicador não lê o `Transform` vivo: o número deixou de ser o que o produto escreveu"
    );
    assert!(
        !PUBLISH.contains("compute_gizmo_transform") && !PUBLISH.contains("screen_to_world"),
        "o publicador re-deriva o número do cursor: ele passa a discordar do encaixe"
    );
}

/// **Só os alvos cujo resultado É o `Transform` da entidade recebem ficha.**
///
/// `FlipPose`, `FlipSelection` e `MotionField` escrevem noutro sítio, então para eles o `Transform`
/// não se move — e a ficha diria `+0,0` com a mão a arrastar. ⚠️ Um número errado apresentado como
/// certo é pior que número nenhum.
///
/// *Mutação que sangra:* apagar o `matches!` — os três passam a publicar um zero teimoso.
#[test]
fn only_entity_targets_get_a_readout() {
    let guard = PUBLISH
        .find("GizmoTarget::PrimaryIndividual")
        .expect("o publicador não filtra por alvo");
    let read = PUBLISH
        .find("sim.world().get::<Transform>")
        .expect("o controle positivo já teria falhado");
    assert!(
        guard < read,
        "o filtro de alvo corre DEPOIS da leitura do mundo: os alvos não-entidade recebem número"
    );
    for wrong in ["FlipPose", "FlipSelection", "MotionField"] {
        assert!(
            !PUBLISH.contains(&format!("GizmoTarget::{wrong}")),
            "`{wrong}` aparece na lista de alvos com ficha, e o `Transform` dele não se move"
        );
    }
}

/// **O silêncio de um gesto parado é honrado aqui** — sem isto a ficha pisca a cada clique de
/// selecção, porque um pick de canvas abre um arrasto de Translate.
///
/// *Mutação que sangra:* apagar o `is_idle`.
#[test]
fn a_gesture_that_did_nothing_publishes_nothing() {
    assert!(
        PUBLISH.contains("r.is_idle()"),
        "o publicador não pergunta se o gesto fez alguma coisa: a ficha pisca a cada clique"
    );
}
