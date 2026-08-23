//! Os gates da **COSTURA do pie menu** — a sequência inteira, sem janela.
//!
//! ⚠️ O que se prova aqui é o que a lei pura não alcança: *abrir com o que o modo oferece · acender
//! pela direcção · soltar escolher · a zona morta cancelar · o transbordo abrir a outra vista*.

use ph2d_editor::HeroScreen;
use ph2d_editor::NodeId;
use ph2d_editor::screens::hero::radial;
use ph2d_editor::widget::{RadialItem, radial_dead_zone_px, radial_item_offset};

const C: [f32; 2] = [400.0, 300.0];

fn hero_with(n: usize) -> HeroScreen {
    let mut h = HeroScreen::new(NodeId(1));
    let items: Vec<RadialItem> = (0..n)
        .map(|i| RadialItem {
            label: format!("T{i}"),
            #[allow(clippy::cast_possible_truncation)]
            id: NodeId(9000 + i as u64),
        })
        .collect();
    assert!(h.store.open_radial(C, items), "a fixture não abriu o menu");
    h
}

/// ⭐ **APONTAR ACENDE, SOLTAR ESCOLHE — e é o item da DIRECÇÃO.**
#[test]
fn pointing_lights_and_releasing_chooses_the_direction() {
    for i in 0..8 {
        let mut h = hero_with(8);
        let o = radial_item_offset(i, 8);
        h.store.radial_point([C[0] + o[0], C[1] + o[1]]);
        assert_eq!(
            h.store.radial().and_then(|r| r.hot),
            Some(i),
            "apontar na direcção do item {i} acendeu outro"
        );
        let chosen = h.store.close_radial().expect("soltar tinha de escolher");
        assert_eq!(chosen.label, format!("T{i}"));
        assert!(
            h.store.radial().is_none(),
            "o menu ficou aberto depois de escolher"
        );
    }
}

/// ⛔ **A ZONA MORTA CANCELA, e o menu fecha na mesma.**
///
/// ⚠️ As duas metades importam: cancelar tem de **não escolher** e tem de **fechar**. Um menu que
/// cancelasse sem fechar ficaria preso na tela sobre a arte.
#[test]
fn releasing_in_the_dead_zone_cancels_and_still_closes() {
    let mut h = hero_with(8);
    h.store
        .radial_point([C[0] + radial_dead_zone_px() * 0.5, C[1]]);
    assert_eq!(h.store.radial().and_then(|r| r.hot), None);
    assert!(
        h.store.close_radial().is_none(),
        "a zona morta escolheu algo"
    );
    assert!(h.store.radial().is_none(), "cancelar deixou o menu aberto");
}

/// ⛔ **UM MENU SEM ITENS NÃO ABRE.**
///
/// ⚠️ Mesma lei do modo de preview: um menu que não oferece nada é indistinguível de um atalho
/// partido, e o artista não teria como saber que o que falta é o modo em que ele está.
#[test]
fn an_empty_menu_refuses_to_open() {
    let mut h = HeroScreen::new(NodeId(1));
    assert!(!h.store.open_radial(C, Vec::new()));
    assert!(h.store.radial().is_none());
}

/// ⭐ **O SECTOR DE TRANSBORDO É RECONHECIDO, e ele não é um comando.**
///
/// ⚠️ Ele tem de ser distinguível de todo item real — senão escolher *"More…"* rotearia um id que
/// o router da paleta não conhece, e o gesto morreria em silêncio.
#[test]
fn the_overflow_sector_is_not_a_command() {
    let many: Vec<RadialItem> = (0..20)
        .map(|i| RadialItem {
            label: format!("T{i}"),
            #[allow(clippy::cast_possible_truncation)]
            id: NodeId(9000 + i as u64),
        })
        .collect();
    let fitted = radial::fit(many);
    let last = fitted.last().expect("o radial não sai vazio");
    assert_eq!(last.id, radial::RADIAL_MORE);
    assert!(
        !fitted[..fitted.len() - 1]
            .iter()
            .any(|i| i.id == radial::RADIAL_MORE),
        "o id do transbordo colidiu com um item real"
    );
}
