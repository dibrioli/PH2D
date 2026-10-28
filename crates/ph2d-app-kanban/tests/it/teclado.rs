//! Os atalhos do Trello pela porta de teclado do hero: o foco por clique, `j` desce, `,`/`.` levam o
//! cartão à lista vizinha, `c` arquiva.

use crate::{Mesa, pega};
use ph2d_editor_core::screens::hero::board_keys::BoardKey;
use ph2d_host::Modifiers;

#[test]
fn foco_por_clique_j_desce_virgula_e_ponto_mudam_de_lista_e_c_arquiva() {
    let (mut m, id) = Mesa::cena1();
    let backlog = m.lista(id, "Backlog");
    let todo = m.lista(id, "To Do");
    let cs = m.cartoes(id, backlog);
    let r = m.rect_cartao(id, cs[0]);
    m.clica(pega(r));
    // O clique abre o verso (W3); o Esc fecha-o e o foco fica no cartão.
    assert!(m.tecla(BoardKey::Escape, Modifiers::default(), None));
    m.quadro();
    assert_eq!(
        m.docs().probe_focus(id.0),
        Some(cs[0]),
        "o clique dá o foco"
    );
    let sem = Modifiers::default();
    assert!(m.tecla(BoardKey::Char('j'), sem, Some("j")));
    m.quadro();
    assert_eq!(
        m.docs().probe_focus(id.0),
        Some(cs[1]),
        "j: o cartão de baixo"
    );

    assert!(m.tecla(BoardKey::Other, sem, Some(".")));
    m.assenta(id);
    assert_eq!(
        m.kanban(id).cards[&cs[1]].list,
        todo,
        ".: para a lista da direita"
    );
    assert_eq!(m.cartoes(id, todo)[1], cs[1], "no mesmo índice");
    assert_eq!(
        m.docs().probe_focus(id.0),
        Some(cs[1]),
        "o foco vai com ele"
    );

    assert!(m.tecla(BoardKey::Other, sem, Some(",")));
    m.assenta(id);
    assert_eq!(
        m.kanban(id).cards[&cs[1]].list,
        backlog,
        ",: de volta à da esquerda"
    );
    assert_eq!(m.cartoes(id, backlog), cs);

    assert!(m.tecla(BoardKey::Char('c'), sem, Some("c")));
    m.quadro();
    assert!(m.kanban(id).cards[&cs[1]].archived, "c: arquivado");
    assert_eq!(
        m.docs().probe_focus(id.0),
        Some(cs[2]),
        "o foco passa ao de baixo"
    );
}
