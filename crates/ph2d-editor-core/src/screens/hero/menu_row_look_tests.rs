//! A cara das linhas dos menus da timeline: cada linha tem ícone, o destrutivo fecha cada menu.

use super::*;

/// Toda linha de todo menu da timeline tem ícone — exceto as dez famílias de easing (o submenu é
/// uma lista de NOMES de curva; um ícone por família seria um desenho que ninguém lê).
#[test]
fn every_timeline_menu_row_has_an_icon_except_the_easing_families() {
    for table in c::ALL_TIMELINE_MENUS {
        if core::ptr::eq(*table, &c::TIMELINE_EASE_MENU[..]) {
            continue;
        }
        for (id, key, _) in table.iter() {
            assert!(icone(*id).is_some(), "a linha {:?} nao tem icone", key.tr());
        }
    }
}

/// O destrutivo vem POR ÚLTIMO em cada menu (dono 13/11) e nenhum rótulo leva «▶» — a seta é ícone.
#[test]
fn the_destructive_row_ends_every_timeline_menu_and_no_label_carries_an_arrow() {
    for table in c::ALL_TIMELINE_MENUS {
        let n = table.len();
        for (i, (id, key, _)) in table.iter().enumerate() {
            let label = key.tr();
            assert!(
                !label.contains('\u{25b6}'),
                "{label:?} ainda leva o triangulo colado"
            );
            if destrutiva(*id) {
                assert_eq!(i + 1, n, "{label:?} destroi e nao e a ultima linha do menu");
            }
        }
    }
}
