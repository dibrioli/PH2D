//! A vista e os menus: recolher é da VISTA (o documento não muda), a cor e o arquivo da lista pelo
//! menu «…», e o «Archived items» do quadro devolve-a.

use crate::{Mesa, centro};
use ph2d_editor_core::zones::Rect;
use ph2d_kanban_model::Hue;

fn linha(rows: &[(String, Rect)], nome: &str) -> Rect {
    rows.iter()
        .find(|(n, _)| n.starts_with(nome))
        .map(|(_, r)| *r)
        .unwrap_or_else(|| {
            panic!(
                "a linha {nome} não está em {:?}",
                rows.iter().map(|r| &r.0).collect::<Vec<_>>()
            )
        })
}

#[test]
fn recolher_uma_lista_muda_a_vista_e_nao_o_documento() {
    let (mut m, id) = Mesa::cena1();
    let todo = m.lista(id, "To Do");
    let antes = m.bytes();
    let (_, recolher, _) = m.docs().probe_list_buttons(id.0, todo).expect("botões");
    m.clica(centro(recolher));
    let v = m.docs().probe_view(id.0).expect("vista");
    assert!(v.collapsed.contains(&todo), "a To Do recolheu");
    assert_eq!(m.bytes(), antes, "o documento ficou igual, byte a byte");
}

#[test]
fn o_menu_da_lista_pinta_arquiva_e_o_arquivo_do_quadro_devolve() {
    let (mut m, id) = Mesa::cena1();
    let todo = m.lista(id, "To Do");
    let menu = |m: &mut Mesa| m.docs().probe_list_buttons(id.0, todo).expect("botões").2;
    let b = menu(&mut m);
    m.clica(centro(b));
    let rows = m.docs().probe_menu_rows(id.0);
    m.clica(centro(linha(&rows, "Color(Some(Blue))")));
    assert_eq!(m.kanban(id).lists[&todo].color, Some(Hue::Blue));
    assert_eq!(
        m.docs().probe_overlays(id.0),
        (false, false),
        "escolher a cor fecha o menu"
    );

    let b = menu(&mut m);
    m.clica(centro(b));
    let rows = m.docs().probe_menu_rows(id.0);
    m.clica(centro(linha(&rows, "ArchiveList")));
    assert!(
        m.kanban(id).lists[&todo].archived,
        "a lista foi para o arquivo"
    );
    assert!(
        m.docs().probe_list_rect(id.0, todo).is_none(),
        "e saiu do quadro"
    );

    let qm = m.docs().probe_rect(id.0, "menu").expect("menu do quadro");
    m.clica(centro(qm));
    let rows = m.docs().probe_menu_rows(id.0);
    m.clica(centro(linha(&rows, "Archived")));
    assert_eq!(
        m.docs().probe_overlays(id.0),
        (false, true),
        "o painel do arquivo abriu"
    );
    let rows = m.painel(id);
    m.clica(centro(linha(&rows, "ShowLists")));
    let rows = m.painel(id);
    m.clica(centro(linha(&rows, "Restore(List(")));
    assert!(!m.kanban(id).lists[&todo].archived, "a lista voltou");
    assert!(
        m.docs().probe_list_rect(id.0, todo).is_some(),
        "e está no quadro"
    );
}
