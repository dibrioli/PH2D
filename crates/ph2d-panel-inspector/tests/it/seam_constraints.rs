//! **A costura do cartão CONSTRAINTS** (`docs/BlenderConstraints/01_plano.md` §4): todo controlo
//! pintado chega ao barramento com a SUA edição, pelo gesto REAL (`click_at`, `drag_at`) — e a
//! edição leva o `Item::id` da linha, nunca a posição dela (por isso os ids da fixtura são `7` e
//! `42`, e não `0` e `1`). O lado do mundo está em `ph2d-app-constraints`.

use ph2d_editor_core::action_bus::{ComponentEdit, EditorAction};
use ph2d_editor_core::constraint_edits::{
    AjusteEscolha, AjusteNumero, ConstraintFieldEdit as E, EstadoDaConstraint,
    InspectorConstraintRow, InspectorConstraintsInfo, Unidade,
};
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_inspector::{
    InspectorPanel, InspectorState, ids, set_current_inspector_constraints,
};
use ph2d_ui_testkit::MockPanelHost;

// Os campos do canal (`ph2d_ecs::canal_constraint`): o painel não depende da `ph2d-ecs`; o gate
// `o_vocabulario_das_constraints_nao_renumera` prende os números do lado de lá.
const LIMITE_MIN: u16 = 3;
const LIMITE_MAX: u16 = 4;
const ESPACO: u16 = 8;

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
                numeros: vec![],
                escolhas: vec![],
                propriedades: vec![],
                ordem: vec![],
                zera_desvio: true,
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
                numeros: vec![
                    AjusteNumero {
                        campo: LIMITE_MIN,
                        rotulo: "panel.inspector.constraint.min",
                        valor: -45.0,
                        unidade: Unidade::Graus,
                    },
                    AjusteNumero {
                        campo: LIMITE_MAX,
                        rotulo: "panel.inspector.constraint.max",
                        valor: 45.0,
                        unidade: Unidade::Graus,
                    },
                ],
                escolhas: vec![AjusteEscolha {
                    campo: ESPACO,
                    rotulo: "panel.inspector.constraint.space",
                    opcoes: &[
                        "panel.inspector.constraint.space.world",
                        "panel.inspector.constraint.space.local",
                    ],
                    valor: 0,
                }],
                propriedades: vec![],
                ordem: vec![],
                zera_desvio: false,
            },
        ],
        selected_count: 1,
        colas: vec![],
        cola_armada: None,
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

/// ⭐⭐⭐ **Os ajustes de cada tipo** (report do dono 25/10: *«em Limit Rotation, onde estão
/// configuradas as limitações?»*): o Max ESCRITO na caixa publica `Campo(42, LIMITE_MAX, v)`; a
/// opção «Local» publica o índice dela no campo da escolha; «Reset Offset» só existe no Child Of e
/// manda o id dele.
///
/// **Mutações que devem sangrar:** o `j` do número trocado (Min ↔ Max) · a opção pela posição da
/// linha · tirar `INSP_CONSTRAINT_ZERAR` do `populate_constraint`.
#[test]
fn os_ajustes_de_cada_tipo_mandam_o_campo_da_linha() {
    assert_eq!(
        clica(ids::INSP_CONSTRAINT_ZERAR[0]),
        vec![E::ZerarDesvio(7)]
    );
    assert_eq!(
        clica(ids::INSP_CONSTRAINT_OPCAO[0][1][1]),
        vec![E::Campo(42, ESPACO, 1.0)]
    );
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut state = InspectorState::default();
    set_current_inspector_constraints(Some(pilha()));
    let rects = pinta(&mut host, &mut state);
    assert!(
        rect_de(&rects, ids::INSP_CONSTRAINT_ZERAR[1]).is_none(),
        "o Limit não tem desvio"
    );
    assert!(
        rect_de(&rects, ids::INSP_CONSTRAINT_NUMERO[0][0]).is_none(),
        "o Child Of não tem números"
    );
    assert!(
        rect_de(&rects, ids::INSP_CONSTRAINT_OPCAO[0][2][1]).is_none(),
        "o espaço tem 2 opções"
    );
    let evs = host.type_into_number(ids::INSP_CONSTRAINT_NUMERO[1][1], "30");
    aplica(&mut host, &mut state, evs);
    let e = edicoes(host.drained_actions());
    set_current_inspector_constraints(None);
    assert_eq!(e.last(), Some(&E::Campo(42, LIMITE_MAX, 30.0)), "{e:?}");
}

/// ⭐⭐ **Os números obedecem aos seletores do projecto** (dono 25/10: *«temos um seletor nas
/// configurações entre px e metros, e precisa ser obedecido»*): com o projecto em px a Distance
/// mostra-se em px (1,5 m a 100 px/m = 150) e o 300 escrito volta como 3 m; com o projecto em
/// radianos o Max mostra-se em radianos e o 1 escrito volta como 57,3°.
#[test]
fn os_numeros_obedecem_a_px_metros_e_graus_radianos_do_projecto() {
    use ph2d_editor_core::project::{DisplayAngle, DisplayUnit};
    const DISTANCIA: u16 = 5;
    let mut info = pilha();
    info.rows[0].numeros = vec![AjusteNumero {
        campo: DISTANCIA,
        rotulo: "panel.inspector.constraint.distance",
        valor: 1.5,
        unidade: Unidade::Metros,
    }];
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut state = InspectorState::default();
    host.project_mut().display_unit = DisplayUnit::Pixels;
    host.project_mut().pixels_per_meter = 100.0;
    host.project_mut().display_angle = DisplayAngle::Radians;
    set_current_inspector_constraints(Some(info));
    pinta(&mut host, &mut state);
    let mostrado = host.store().number_value(ids::INSP_CONSTRAINT_NUMERO[0][0]);
    let max = host.store().number_value(ids::INSP_CONSTRAINT_NUMERO[1][1]);
    let evs = host.type_into_number(ids::INSP_CONSTRAINT_NUMERO[0][0], "300");
    aplica(&mut host, &mut state, evs);
    let evs = host.type_into_number(ids::INSP_CONSTRAINT_NUMERO[1][1], "1");
    aplica(&mut host, &mut state, evs);
    let e = edicoes(host.drained_actions());
    set_current_inspector_constraints(None);
    assert!(
        mostrado.is_some_and(|v| (v - 150.0).abs() < 1e-3),
        "a Distance mostrou {mostrado:?}"
    );
    assert!(
        max.is_some_and(|v| (v - 45f64.to_radians()).abs() < 1e-4),
        "o Max mostrou {max:?}"
    );
    assert!(e.contains(&E::Campo(7, DISTANCIA, 3.0)), "{e:?}");
    assert!(
        e.iter()
            .any(|x| matches!(x, E::Campo(42, LIMITE_MAX, v) if (v - 57.29578).abs() < 1e-3)),
        "{e:?}"
    );
}

/// ⭐ «Bake» pede `Cozer` (a pilha inteira do objecto); sem itens o botão não existe — cozer uma pilha
/// vazia seria um controlo morto.
#[test]
fn o_bake_pede_cozer_e_so_existe_com_itens() {
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut state = InspectorState::default();
    set_current_inspector_constraints(Some(pilha()));
    pinta(&mut host, &mut state);
    assert_eq!(
        clica_em(&mut host, &mut state, ids::INSP_CONSTRAINT_BAKE),
        vec![E::Cozer]
    );
    let mut vazia = pilha();
    vazia.rows.clear();
    set_current_inspector_constraints(Some(vazia));
    let rects = pinta(&mut host, &mut state);
    set_current_inspector_constraints(None);
    assert!(rect_de(&rects, ids::INSP_CONSTRAINT_BAKE).is_none());
    assert!(rect_de(&rects, ids::INSP_CONSTRAINT_ADD).is_some());
}

/// O despacho também recusa: um clique no id do «Bake» com a pilha VAZIA (um evento que chega sem
/// o botão pintado — atalho, macro, evento atrasado) não pede `Cozer`.
///
/// **Mutação que deve sangrar:** tirar `&& !info.rows.is_empty()` do braço do Bake.
#[test]
fn o_bake_com_a_pilha_vazia_nao_pede_nada() {
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut state = InspectorState::default();
    let mut vazia = pilha();
    vazia.rows.clear();
    set_current_inspector_constraints(Some(vazia));
    pinta(&mut host, &mut state);
    aplica(
        &mut host,
        &mut state,
        vec![WidgetEvent::Click(ids::INSP_CONSTRAINT_BAKE)],
    );
    let out = edicoes(host.drained_actions());
    set_current_inspector_constraints(None);
    assert!(out.is_empty(), "{out:?}");
}

/// Uma linha Follow Value (id `9`): Follow com a pose e um campo, Drive com a pose, na ordem do
/// cartão (Follow, faixa, Drive, faixa).
fn follow_value() -> InspectorConstraintsInfo {
    use ph2d_editor_core::constraint_edits::{
        AjustePropriedade, OpcaoPropriedade, Peca, PropriedadeDoValor as P,
    };
    const DE: u16 = 23;
    const PARA: u16 = 24;
    let pose = |n: usize| {
        (0..n)
            .map(|i| OpcaoPropriedade {
                prop: P::Canal(u8::try_from(i).unwrap()),
                rotulo: format!("canal {i}"),
            })
            .collect::<Vec<_>>()
    };
    let mut follow = pose(5);
    follow.push(OpcaoPropriedade {
        prop: P::Campo([2, 11, 0, 0]),
        rotulo: "campo".into(),
    });
    let numero = |campo| AjusteNumero {
        campo,
        rotulo: "panel.inspector.constraint.from_min",
        valor: 0.0,
        unidade: Unidade::Metros,
    };
    let mut i = pilha();
    i.rows = vec![InspectorConstraintRow {
        id: 9,
        tipo: 20,
        alvo: Some("Mão".into()),
        precisa_alvo: true,
        forca: 1.0,
        ligada: true,
        estado: EstadoDaConstraint::Activa,
        escolhendo: false,
        numeros: vec![numero(25), numero(26), numero(27), numero(28)],
        escolhas: vec![],
        propriedades: vec![
            AjustePropriedade {
                campo: DE,
                rotulo: "panel.inspector.constraint.follow",
                opcoes: follow,
                valor: P::Canal(1),
                atual: "canal 1".into(),
            },
            AjustePropriedade {
                campo: PARA,
                rotulo: "panel.inspector.constraint.drive",
                opcoes: pose(5),
                valor: P::Canal(2),
                atual: "canal 2".into(),
            },
        ],
        ordem: vec![
            Peca::Propriedade(0),
            Peca::Numero(0),
            Peca::Numero(1),
            Peca::Propriedade(1),
            Peca::Numero(2),
            Peca::Numero(3),
        ],
        zera_desvio: false,
    }];
    i
}

/// ⭐⭐⭐ **O ▾ Follow do Follow Value**: pinta-se na ordem da família (Follow, faixa, Drive), abre
/// a lista do alvo, e a 6.ª opção (o campo) manda `Propriedade(9, DE, campo)` e fecha o ▾.
///
/// **Mutações que devem sangrar:** tirar o registo das opções do `populate_constraint` · ler a
/// opção de outra lista (o Drive) · apagar o `constraint_prop_popover` do passe diferido.
#[test]
fn the_value_cards_follow_dropdown_writes_the_chosen_property() {
    use ph2d_editor_core::constraint_edits::PropriedadeDoValor as P;
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut state = InspectorState::default();
    set_current_inspector_constraints(Some(follow_value()));
    let rects = pinta(&mut host, &mut state);
    let y = |id| {
        rect_de(&rects, id)
            .unwrap_or_else(|| panic!("{id:?} não pintado"))
            .y
    };
    let follow = ids::INSP_CONSTRAINT_PROP[0][0];
    let drive = ids::INSP_CONSTRAINT_PROP[1][0];
    assert!(y(follow) < y(ids::INSP_CONSTRAINT_NUMERO[0][0]));
    assert!(y(ids::INSP_CONSTRAINT_NUMERO[1][0]) < y(drive));
    assert!(y(drive) < y(ids::INSP_CONSTRAINT_NUMERO[2][0]));
    assert_eq!(host.dropdown_is_open(follow), Some(false));
    host.set_dropdown_open(follow, true);
    let e = clica_em(&mut host, &mut state, ids::INSP_CONSTRAINT_PROP_OPCAO[5]);
    set_current_inspector_constraints(None);
    assert_eq!(e, vec![E::Propriedade(9, 23, P::Campo([2, 11, 0, 0]))]);
    assert_eq!(
        host.dropdown_is_open(follow),
        Some(false),
        "o ▾ ficou aberto"
    );
}
