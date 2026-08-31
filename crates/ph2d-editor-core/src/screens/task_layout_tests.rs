//! A tabela dos layouts, conferida contra si mesma.
//!
//! ⚠️ **O que ela promete sobre PAINÉIS e FERRAMENTAS é medido noutro sítio** — aqui não há
//! registry nenhum instalado, e uma varredura sobre zero painéis passaria sobre nada. Os gates que
//! confrontam a tabela com o que existe vivem em `ph2d-panel-registry-init/tests/`.

use super::*;

#[test]
fn every_layout_survives_a_round_trip_through_its_wire_name() {
    let mut seen = std::collections::BTreeSet::new();
    for l in TaskLayout::ALL {
        let w = l.spec().wire;
        assert_eq!(TaskLayout::from_wire(w), Some(l), "{l:?} não voltou");
        assert!(
            seen.insert(w),
            "dois layouts partilham a chave {w:?} — a arrumação gravada de um leria a do outro"
        );
        assert!(
            w.chars()
                .all(|c| c.is_ascii_lowercase() || c == '_' || c.is_ascii_digit()),
            "{l:?} tem uma chave de ficheiro que não é snake_case ASCII: {w:?}"
        );
    }
    assert_eq!(TaskLayout::from_wire("um_layout_de_2030"), None);
}

/// ⭐ **Nenhum layout abre o vazio**, e nenhum repete um painel.
#[test]
fn every_layout_opens_something_and_never_twice() {
    for l in TaskLayout::ALL {
        let s = l.spec();
        assert!(
            !s.open.is_empty(),
            "{l:?} não abre painel nenhum — a aba dele é uma tela em branco"
        );
        let mut seen = std::collections::BTreeSet::new();
        for id in s.open {
            assert!(seen.insert(*id), "{l:?} abre `{id}` duas vezes");
        }
    }
}

/// ⚠️ **Os títulos das abas são distintos e curtos** — elas partilham uma barra com cinco menus.
#[test]
fn the_tab_titles_are_distinct_and_short() {
    let mut seen = std::collections::BTreeSet::new();
    for l in TaskLayout::ALL {
        let t = l.spec().title;
        assert!(seen.insert(t), "dois layouts chamam-se {t:?}");
        assert!(
            !t.is_empty() && t.len() <= 10,
            "{t:?} não cabe numa aba ao lado dos cinco menus ({} caracteres)",
            t.len()
        );
    }
}

/// ⛔ **Dois layouts não podem entregar o canvas ao mesmo dono.**
///
/// ⚠️ Não é um erro de tipo, é um erro de desenho: dois layouts com a mesma ferramenta deixam de
/// se distinguir no eixo que a D3 diz ser o do MODO.
///
/// ⚠️ **A excepção é a ferramenta NEUTRA** (`move`, a de omissão do registry): entregar-lhe o
/// canvas é dizer *«esta tarefa não é sobre o canvas»* — o *Animate* é sobre o tempo —, e duas
/// tarefas podem legitimamente dizer isso. O que ela **não** pode ser é ausência: ver
/// `CanvasOwner`.
#[test]
fn no_two_layouts_hand_the_canvas_to_the_same_owner() {
    let mut seen = std::collections::BTreeSet::new();
    let mut modal = 0usize;
    for l in TaskLayout::ALL {
        match l.spec().canvas {
            CanvasOwner::Tool(NEUTRAL_TOOL) => {}
            CanvasOwner::Tool(t) => {
                modal += 1;
                assert!(seen.insert(t), "dois layouts pegam na ferramenta {t:?}");
            }
            CanvasOwner::Model3d => {
                modal += 1;
                assert!(
                    seen.insert("<model3d>"),
                    "dois layouts entregam o canvas ao modelador"
                );
            }
        }
    }
    assert!(
        modal >= 4,
        "só {modal} layouts têm um dono próprio para o canvas — a costura com o Modo evaporou"
    );
}

/// A ferramenta de omissão do registry — o que este app tem em vez de «nenhuma».
const NEUTRAL_TOOL: &str = "move";

/// ⭐⭐ **Sair de uma tarefa MODAL solta o modo dela** — a régua do report do Enio (2026-08-31):
/// *«se abro Nodes e depois Model, o grafo de Nodes persiste»*.
///
/// ⛔ A causa não estava no grafo: o `canvas` era `Option`, o *Model* e o *Animate* diziam `None`,
/// e o `None` significava **herda** — com os painéis da ferramenta anterior atrás, porque quem os
/// abre é a ponte DELA e não a lista de abertos. ⚠️ O gate que aqui estava
/// (`…_one_that_does_not_leaves_the_hand_alone`) **afirmava o defeito**: ele media a decisão em vez
/// da consequência, e por isso ficou verde durante o report inteiro.
#[test]
fn leaving_a_modal_layout_never_leaves_its_mode_behind() {
    for from in TaskLayout::ALL {
        let CanvasOwner::Tool(modal) = from.spec().canvas else {
            continue; // o *Model* não é uma ferramenta; a saída dele é a lei do `field3d_mode`
        };
        if modal == NEUTRAL_TOOL {
            continue;
        }
        for to in TaskLayout::ALL.into_iter().filter(|l| *l != from) {
            assert_ne!(
                to.spec().canvas,
                CanvasOwner::Tool(modal),
                "sair de {from:?} para {to:?} deixa `{modal}` em mãos — e com ela os painéis dela"
            );
        }
    }
}

/// ⚠️ **O de omissão está na lista** — senão o app abre num layout que a barra não mostra.
#[test]
fn the_default_layout_is_one_of_the_tabs() {
    assert!(TaskLayout::ALL.contains(&TaskLayout::default()));
}
