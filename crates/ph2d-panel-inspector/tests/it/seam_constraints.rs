//! **A costura do cartão CONSTRAINTS** (`docs/BlenderConstraints/01_plano.md` §4): todo controlo
//! pintado chega ao barramento com a SUA edição, pelo gesto REAL (`click_at`, `drag_at`) — e a
//! edição leva o `Item::id` da linha, nunca a posição dela (por isso os ids da fixtura são `7` e
//! `42`, e não `0` e `1`). O lado do mundo está em `ph2d-app-constraints`.

use ph2d_editor_core::action_bus::{ComponentEdit, EditorAction};
use ph2d_editor_core::constraint_edits::{
    ConstraintFieldEdit as E, EstadoDaConstraint, InspectorConstraintRow, InspectorConstraintsInfo,
};
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_inspector::{
    InspectorPanel, InspectorState, ids, set_current_inspector_constraints,
};
use ph2d_ui_testkit::MockPanelHost;

const ENTITY: u64 = 0x1_0000_00C5;
const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 320.0,
    h: 2400.0,
};

/// Duas linhas: um Child Of COM alvo (id `7`) e um Limit Rotation, que não pede alvo (id `42`).
fn pilha() -> InspectorConstraintsInfo {
    InspectorConstraintsInfo {
        entity_bits: ENTITY,
        rows: vec![
            InspectorConstraintRow {
                id: 7,
                tipo: 0,
                alvo: Some("Mão".into()),
                precisa_alvo: true,
                forca: 1.0,
                ligada: true,
                estado: EstadoDaConstraint::Activa,
                escolhendo: false,
            },
            InspectorConstraintRow {
                id: 42,
                tipo: 6,
                alvo: None,
                precisa_alvo: false,
                forca: 0.5,
                ligada: true,
                estado: EstadoDaConstraint::Ciclo,
                escolhendo: false,
            },
        ],
        selected_count: 1,
    }
}

fn pinta(host: &mut MockPanelHost, state: &mut InspectorState) -> Vec<(ph2d_a11y::NodeId, Rect)> {
    host.paint::<InspectorPanel>(state, VIEWPORT)
}

fn rect_de(rects: &[(ph2d_a11y::NodeId, Rect)], id: ph2d_a11y::NodeId) -> Option<Rect> {
    rects.iter().find(|(n, _)| *n == id).map(|(_, r)| *r)
}

fn aplica(host: &mut MockPanelHost, state: &mut InspectorState, evs: Vec<WidgetEvent>) {
    for ev in evs {
        let _ = host.apply_panel_event::<InspectorPanel>(state, ev);
    }
}

fn edicoes(acoes: Vec<EditorAction>) -> Vec<E> {
    acoes
        .into_iter()
        .filter_map(|a| match a {
            EditorAction::InspectorComponentEdit {
                entity_bits,
                edit: ComponentEdit::Constraint(e),
            } if entity_bits == ENTITY => Some(e),
            _ => None,
        })
        .collect()
}

/// Clica no CENTRO do widget `id` e devolve as edições publicadas.
fn clica_em(host: &mut MockPanelHost, state: &mut InspectorState, id: ph2d_a11y::NodeId) -> Vec<E> {
    let rects = pinta(host, state);
    let r = rect_de(&rects, id).unwrap_or_else(|| panic!("o cartão não pintou {id:?}"));
    let evs = host.click_at(r.x + r.w * 0.5, r.y + r.h * 0.5);
    assert!(!evs.is_empty(), "{id:?} está MORTO sob o dedo");
    aplica(host, state, evs);
    edicoes(host.drained_actions())
}

fn clica(id: ph2d_a11y::NodeId) -> Vec<E> {
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut state = InspectorState::default();
    set_current_inspector_constraints(Some(pilha()));
    let out = clica_em(&mut host, &mut state, id);
    set_current_inspector_constraints(None);
    out
}

/// ⭐⭐ Os botões de cada linha mandam o `Item::id` DELA — o `42` da 2.ª linha, nunca `1`.
///
/// **Mutações que devem sangrar:** publicar a posição em vez do `row.id` · trocar SUBIR/DESCER ·
/// tirar uma tabela do `populate_constraint`.
#[test]
fn os_botoes_de_cada_linha_mandam_o_id_dela() {
    assert_eq!(clica(ids::INSP_CONSTRAINT_APAGAR[1]), vec![E::Apagar(42)]);
    assert_eq!(clica(ids::INSP_CONSTRAINT_SUBIR[1]), vec![E::Mover(42, -1)]);
    assert_eq!(clica(ids::INSP_CONSTRAINT_DESCER[0]), vec![E::Mover(7, 1)]);
    assert_eq!(
        clica(ids::INSP_CONSTRAINT_ALVO[0]),
        vec![E::EscolherAlvo(7)]
    );
}

/// ⭐⭐ «Enabled» afirma o CONTRÁRIO do instantâneo, com o id da linha.
#[test]
fn a_caixa_enabled_manda_o_contrario_do_instantaneo() {
    assert_eq!(
        clica(ids::INSP_CONSTRAINT_LIGADA[0]),
        vec![E::Ligada(7, false)]
    );
}

/// ⭐ Uma linha cujo tipo não pede alvo não pinta «Pick» (o Limit Rotation).
#[test]
fn uma_linha_sem_alvo_nao_pinta_o_pick() {
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut state = InspectorState::default();
    set_current_inspector_constraints(Some(pilha()));
    let rects = pinta(&mut host, &mut state);
    set_current_inspector_constraints(None);
    assert!(rect_de(&rects, ids::INSP_CONSTRAINT_ALVO[0]).is_some());
    assert!(
        rect_de(&rects, ids::INSP_CONSTRAINT_ALVO[1]).is_none(),
        "o Limit Rotation não tem alvo: «Pick» seria um botão mudo"
    );
    assert!(rect_de(&rects, ids::INSP_CONSTRAINT_FORCA[2]).is_none());
}

/// ⭐⭐⭐ ARRASTAR a força da 2.ª linha publica `Forca(42, v)` com o valor NOVO.
#[test]
fn arrastar_a_forca_publica_o_valor_novo_com_o_id_da_linha() {
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut state = InspectorState::default();
    set_current_inspector_constraints(Some(pilha()));
    let rects = pinta(&mut host, &mut state);
    let r =
        rect_de(&rects, ids::INSP_CONSTRAINT_FORCA[1]).expect("a barra da força não foi pintada");
    let y = r.y + r.h * 0.5;
    let evs = host.drag_at(r.x + r.w * 0.5, y, r.x + r.w * 0.8, y); // LITERAL-PX-OK: 80 % da trilha
    aplica(&mut host, &mut state, evs);
    let e = edicoes(host.drained_actions());
    set_current_inspector_constraints(None);
    let Some(E::Forca(id, v)) = e.last() else {
        panic!("arrastar a força não publicou nada: {e:?}");
    };
    assert_eq!(*id, 42);
    assert!(
        (v - 0.8).abs() < 0.05,
        "a força publicada é {v}, o dedo parou a 0,8"
    ); // LITERAL-PX-OK: tolerância
}

/// ⭐⭐ «Add Constraint» abre a lista; o 6.º tipo manda `Adicionar(5)` e a lista fecha.
#[test]
fn o_mais_abre_a_lista_e_o_tipo_acrescenta() {
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut state = InspectorState::default();
    set_current_inspector_constraints(Some(pilha()));
    let fechada = pinta(&mut host, &mut state);
    assert!(rect_de(&fechada, ids::INSP_CONSTRAINT_TIPO[5]).is_none());
    assert_eq!(
        clica_em(&mut host, &mut state, ids::INSP_CONSTRAINT_ADD),
        vec![]
    );
    assert_eq!(
        clica_em(&mut host, &mut state, ids::INSP_CONSTRAINT_TIPO[5]),
        vec![E::Adicionar(5)]
    );
    let depois = pinta(&mut host, &mut state);
    set_current_inspector_constraints(None);
    assert!(
        rect_de(&depois, ids::INSP_CONSTRAINT_TIPO[5]).is_none(),
        "a lista dos tipos não fechou depois da escolha"
    );
}

/// ⭐ Cheio, o «+» não existe (um botão que aceita o clique e não faz nada é controlo morto).
#[test]
fn cheio_o_mais_desaparece() {
    let mut cheia = pilha();
    let modelo = cheia.rows[1].clone();
    cheia.rows = (0..ids::INSP_CONSTRAINT_FORCA.len())
        .map(|k| InspectorConstraintRow {
            id: u32::try_from(k).unwrap() + 100,
            ..modelo.clone()
        })
        .collect();
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut state = InspectorState::default();
    set_current_inspector_constraints(Some(cheia));
    // A pilha cheia é mais alta que o painel de sempre: uma vista que a leva inteira.
    let alta = Rect {
        h: 20_000.0, // LITERAL-PX-OK: 32 linhas de altura
        ..VIEWPORT
    };
    let rects = host.paint::<InspectorPanel>(&mut state, alta);
    set_current_inspector_constraints(None);
    assert!(rect_de(&rects, ids::INSP_CONSTRAINT_ADD).is_none());
    assert!(rect_de(&rects, *ids::INSP_CONSTRAINT_FORCA.last().unwrap()).is_some());
}
