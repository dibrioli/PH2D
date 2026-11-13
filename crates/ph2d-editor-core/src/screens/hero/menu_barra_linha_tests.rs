use super::*;
use crate::interaction::ContextMenuKind as K;

/// ⭐ **Toda linha estática dos menus da barra tem ícone** (onda 8, dono 13/11: «sem ícones … ainda não
/// estão tão bonitos»). A população é a tabela de cada menu da barra e a seta do Play; as linhas que
/// abrem uma lista de VALORES (as das Preferências) também — os valores mostram a escolha.
#[test]
fn toda_folha_dos_menus_da_barra_tem_icone() {
    let mut kinds: Vec<K> = super::super::menu_bar::CASCADES
        .iter()
        .map(|(_, k)| *k)
        .collect();
    kinds.extend([K::MenuBarRoot, K::MenuBarPlay, K::SettingsMenu]);
    let mut sem = Vec::new();
    let mut total = 0;
    for kind in kinds {
        for (id, key, _) in super::super::menu_rows::menu_rows(kind) {
            total += 1;
            if super::super::menu_icones::icone(*id).is_none() {
                sem.push(format!("{kind:?}: {key:?}"));
            }
        }
    }
    assert!(total > 60, "a população encolheu: {total}");
    assert!(sem.is_empty(), "linhas sem ícone: {sem:#?}");
}

/// ⭐ **A grelha da maquete**: o glifo de 16 depois do respiro, o rótulo depois do vão, a mesma coluna
/// em toda linha (os rótulos alinham haja glifo ou não), e o glifo ao meio da altura.
#[test]
fn o_glifo_e_o_rotulo_tem_uma_coluna_so() {
    let r = Rect::new(10.0, 40.0, 232.0, ph2d_tokens::MENU_ROW_H_PX);
    let g = caixa_do_glifo(r);
    assert_eq!(g.w, BAR_ICON_PX);
    assert!(((g.y + g.h * 0.5) - (r.y + r.h * 0.5)).abs() < 1e-4);
    assert_eq!(g.x, r.x + Spacing::Sm.px());
    assert_eq!(x_do_rotulo(r), g.x + g.w + Spacing::Md.px());
    let outra = Rect::new(10.0, 64.0, 232.0, ph2d_tokens::MENU_ROW_H_PX);
    assert_eq!(x_do_rotulo(outra), x_do_rotulo(r));
}
