//! ⛔⛔ **OS DOIS GESTOS DOS ÓRFÃOS TÊM BRAÇO NO DRENO** (ADR-0164 / F5.3 e F5.3-ter).
//!
//! # Porque este gate é textual, e porque ele é preciso
//!
//! O `match` que consome o `EditorAction` na `render_loop` termina num `_ => {}`: uma acção **nova
//! sem braço compila, corre e não faz nada**. É a primeira das duas espécies de controlo morto que
//! a caça de 2026-08-30 nomeou, e **nenhum gate de registo a apanha** — um seam de painel prova
//! que o clique chega ao *barramento*, nunca que alguém do outro lado o lê. O irmão deste ficheiro
//! é o `the_apply_ladder_has_one_door`, e ele existe pela mesma razão.
//!
//! # ⚠️ Os dois juntos, porque a doença é a CONFUSÃO entre eles
//!
//! *Limpar todas* e *largar uma* são gestos vizinhos com o mesmo sujeito, e o modo de falha barato
//! é o segundo braço chamar a porta do primeiro — um `✕` de linha que apaga a lista inteira. ⇒ o
//! gate exige que cada acção nomeie a **sua** porta.
//!
//! ⚠️ **Ele descasca comentários antes de varrer.** Um censo textual que não separa prosa de código
//! mente nos dois sentidos: acusa a prosa que descreve a cura, e absolve o código quando um
//! comentário vizinho nomeia a porta.

use std::path::Path;

fn code_of(rel: &str) -> String {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("src").join(rel);
    let body = std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{}: {e}", p.display()));
    body.lines()
        .map(|l| match l.find("//") {
            Some(i) => &l[..i],
            None => l,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// ⭐⭐⭐ **Cada gesto tem braço, e cada braço chama a PORTA DELE.**
///
/// **Mutação que deve sangrar:** apagar o braço do `InspectorDropUnusedOverride`, ou fazê-lo chamar
/// o `clear_orphans`.
#[test]
fn each_unused_override_gesture_has_its_own_arm_and_its_own_door() {
    let body = code_of("render_loop/mod.rs");
    for (action, door) in [
        (
            "EditorAction::InspectorClearUnusedOverrides",
            "inspector_instance::clear_orphans(",
        ),
        (
            "EditorAction::InspectorDropUnusedOverride",
            "inspector_instance::drop_orphan(",
        ),
    ] {
        assert!(
            body.contains(action),
            "{action} nao tem braco no dreno — o `_ => {{}}` do fim do match come-a em silencio"
        );
        assert!(
            body.contains(door),
            "o braco de {action} nao chama {door} — o clique morre a um passo do efeito, ou \
             chama a porta do gesto VIZINHO (que apaga a lista inteira)"
        );
    }
}

/// ⚠️ **As duas portas são IRMÃS e não metades uma da outra** — `drop_orphan` não pode ser escrito
/// à custa do `clear_orphans`, senão o `✕` de uma linha herda o alcance do botão de baixo.
///
/// ⛔ E as duas mexem **só** nos órfãos: uma excepção com alvo é o que o artista está a ver e a
/// usar, e apagá-la seria *Revert to Master* com outro nome.
#[test]
fn dropping_one_never_reaches_the_live_overrides() {
    let body = code_of("render_loop/inspector_instance.rs");
    let drop_fn = body
        .split_once("pub(super) fn drop_orphan(")
        .expect("a porta de largar uma")
        .1;
    assert!(
        !drop_fn.contains("overrides"),
        "o `drop_orphan` toca nas excepcoes VIVAS — o `x` de uma linha de orfao nao pode apagar o \
         que o artista esta' a usar"
    );
    assert!(
        !drop_fn.contains("orphans.clear()"),
        "o `drop_orphan` limpa o mapa inteiro — e' o botao de baixo com outro icone"
    );
}
