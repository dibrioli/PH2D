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

/// ⛔ **Um layout que declara ferramenta declara-a por id, e dois não podem declarar a mesma.**
///
/// ⚠️ Dois layouts com a mesma ferramenta não são um erro de tipo, mas são um erro de desenho:
/// eles deixam de se distinguir no eixo que a D3 diz ser o do MODO.
#[test]
fn no_two_layouts_claim_the_same_tool() {
    let mut seen = std::collections::BTreeSet::new();
    let mut with_tool = 0usize;
    for l in TaskLayout::ALL {
        if let Some(t) = l.spec().tool {
            with_tool += 1;
            assert!(seen.insert(t), "dois layouts pegam na ferramenta {t:?}");
        }
    }
    assert!(
        with_tool >= 3,
        "só {with_tool} layouts pegam numa ferramenta — a costura opcional com o Modo evaporou"
    );
}

/// ⚠️ **O de omissão está na lista** — senão o app abre num layout que a barra não mostra.
#[test]
fn the_default_layout_is_one_of_the_tabs() {
    assert!(TaskLayout::ALL.contains(&TaskLayout::default()));
}
