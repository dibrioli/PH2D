//! J1 pela costura do hero: o SELETOR de vistas do cabeçalho (o quadro e as quatro da
//! `ph2d-kanban-views`, na mesma aba) e o painel do FILTRO com o cabeçalho fixo.

use crate::w6::{alvo, tecla, toca};
use crate::{AREA, Mesa, centro};
use ph2d_kanban_model::{
    FieldDef, FieldId, FieldKind, FieldValue, FracKey, KanbanId, KanbanOp, Meta, Record,
};
use ph2d_kanban_views::ViewKind;
use ph2d_kanban_views::table::Column;

fn vista(m: &mut Mesa, id: KanbanId, i: u8) {
    toca(m, id, "ViewsButton");
    assert_eq!(m.docs().probe_w6(id.0).0.as_deref(), Some("Views"));
    toca(m, id, &format!("View({i})"));
    m.quadro();
}

#[test]
fn j1_o_seletor_troca_para_a_tabela_e_o_cartao_dela_abre_o_verso_e_volta_ao_quadro() {
    let (mut m, id) = Mesa::cena1();
    vista(&mut m, id, 1);
    assert_eq!(m.docs().probe_view_kind(id.0), Some(ViewKind::Table));
    let (card, r) = m
        .docs()
        .probe_table_cell(id.0, 0, Column::Card)
        .expect("a tabela pintou");
    m.clica(centro(r));
    assert_eq!(
        m.docs().probe_back(id.0),
        Some(card),
        "o clique no cartão da tabela abre o verso"
    );
    // Cada vista pinta-se na aba.
    for (i, k) in [
        (2, ViewKind::Calendar),
        (3, ViewKind::Timeline),
        (4, ViewKind::Dashboard),
    ] {
        m.tecla(
            ph2d_editor_core::screens::hero::board_keys::BoardKey::Escape,
            crate::verso::sem(),
            None,
        );
        m.quadro();
        assert_eq!(m.docs().probe_back(id.0), None, "o Esc fecha o verso");
        m.quadro();
        vista(&mut m, id, i);
        assert_eq!(m.docs().probe_view_kind(id.0), Some(k));
    }
    vista(&mut m, id, 0);
    assert_eq!(m.docs().probe_view_kind(id.0), None, "de volta ao quadro");
    let backlog = m.lista(id, "Backlog");
    let c = m.cartoes(id, backlog)[0];
    let _ = m.rect_cartao(id, c);
}

#[test]
fn j1_a_tabela_escreve_pelo_plano_na_porta_do_kanban_e_um_ctrl_z_desfaz() {
    let (mut m, id) = Mesa::cena1();
    let f = {
        let shared = m.hero.documents.kanbans_shared();
        let mut set = shared.borrow_mut();
        let k = set.doc_mut(id).expect("kanban");
        let f = FieldId(k.mint());
        let def = FieldDef {
            id: f,
            name: "Approved".into(),
            kind: FieldKind::Checkbox,
            on_front: false,
            pos: FracKey::between(None, None),
            meta: Meta::default(),
        };
        let _ = KanbanOp::put(Record::Field(def)).apply(k);
        f
    };
    m.quadro();
    vista(&mut m, id, 1);
    let (card, r) = m
        .docs()
        .probe_table_cell(id.0, 0, Column::Field(f))
        .expect("a coluna do campo");
    m.clica(centro(r));
    assert_eq!(
        m.kanban(id).cards[&card].fields.get(&f),
        Some(&FieldValue::Checkbox(true))
    );
    assert_eq!(m.desfaz(), Some(true));
    assert_eq!(
        m.kanban(id).cards[&card].fields.get(&f),
        None,
        "UM passo de desfazer do Kanban"
    );
}

#[test]
fn j1_o_filtro_tem_cabecalho_fixo_a_palavra_sempre_a_vista_e_cabe_na_janela() {
    let (mut m, id) = Mesa::cena1();
    {
        // Etiquetas bastantes para o painel ser mais alto que a janela.
        let shared = m.hero.documents.kanbans_shared();
        let mut set = shared.borrow_mut();
        let k = set.doc_mut(id).expect("kanban");
        for i in 0..24 {
            let l = ph2d_kanban_model::Label {
                id: ph2d_kanban_model::LabelId(k.mint()),
                name: format!("Extra {i}"),
                color: None,
                meta: Meta::default(),
            };
            let _ = KanbanOp::put(Record::Label(l)).apply(k);
        }
    }
    m.quadro();
    assert!(tecla(&mut m, 'f'));
    let botao = alvo(&mut m, id, "FilterButton");
    let folha = alvo(&mut m, id, "Inside");
    let fechar = alvo(&mut m, id, "Close");
    let palavra = alvo(&mut m, id, "Keyword");
    assert!(
        folha.y >= botao.y + botao.h,
        "ancorada por baixo do «Filters»"
    );
    assert!(
        folha.y + folha.h <= AREA.y + AREA.h,
        "a altura cabe na janela"
    );
    for _ in 0..30 {
        let c = centro(folha);
        m.roda(c.0, c.1 + folha.h / 3.0, -48.0);
    }
    assert!(
        m.docs()
            .probe_w6_hits(id.0)
            .iter()
            .any(|(n, _)| n.starts_with("NoLabels") || n.starts_with("Label(")),
        "o corpo rolou até às etiquetas"
    );
    assert_eq!(alvo(&mut m, id, "Close"), fechar, "o «×» não rola");
    assert_eq!(
        alvo(&mut m, id, "Keyword"),
        palavra,
        "a palavra-chave não rola"
    );
    // Nenhum alvo do corpo invade o cabeçalho fixo.
    let topo = palavra.y + palavra.h;
    for (n, r) in m.docs().probe_w6_hits(id.0) {
        if n.starts_with("Label(") || n.starts_with("Member(") || n.starts_with("Due(") {
            assert!(r.y >= topo, "{n} por baixo do cabeçalho: {r:?}");
        }
    }
}
