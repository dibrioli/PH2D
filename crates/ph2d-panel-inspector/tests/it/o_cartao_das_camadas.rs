//! ⭐ **O cartão «Collision Layers»** (ficha f1, onda 7): existe nos tipos certos, dobra, resume e leva a fila.
//! A edição continua a de sempre: `os_quadros_da_colisao` e `o_cartao_da_fisica` provam-na por clique real.

use crate::o_cartao_da_fisica::{achado, cena, clica, pinta, sai};
use crate::seam_physics::{as_part, with_body};
use ph2d_a11y::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids::grip_de;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::screens::hero::InspectorPhysicsInfo;
use ph2d_editor_core::widget::card_censo;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_inspector::camadas_fisica::set_current;
use ph2d_panel_inspector::ids::*;
use ph2d_panel_inspector::resumo_do_cartao_de_fisica as resumo_do_cartao;
use ph2d_panel_inspector::set_current_inspector_physics;

/// A tabela da cena 120.
fn mundo() {
    let mut nomes: [String; 32] = Default::default();
    for (i, n) in ["Ground", "Player", "Props", "Zones"]
        .into_iter()
        .enumerate()
    {
        nomes[i] = n.into();
    }
    let mut linhas = [0u32; 32];
    linhas[0] = 1 << 1 | 1 << 2;
    linhas[1] = 1;
    linhas[2] = 1 | 1 << 2 | 1 << 3;
    linhas[3] = 1 << 2;
    set_current(nomes, linhas, 0b1111);
}

fn fim() {
    set_current(Default::default(), [u32::MAX; 32], 0xFF);
    sai();
}

fn na(layer: u8, mut info: InspectorPhysicsInfo) -> InspectorPhysicsInfo {
    info.layer = layer;
    info
}

fn zona() -> InspectorPhysicsInfo {
    InspectorPhysicsInfo {
        is_sensor: true,
        ..with_body()
    }
}

fn solta() -> InspectorPhysicsInfo {
    InspectorPhysicsInfo {
        part_owner: String::new(),
        ..as_part()
    }
}

fn so_com_corpo_acima() -> Vec<NodeId> {
    INSP_PHYS_COLLIDES
        .iter()
        .copied()
        .chain([INSP_PHYS_COLLIDES_ADD, INSP_PHYS_EDIT_MATRIX])
        .collect()
}

/// O corpo, a Zona e a peça têm o cartão inteiro, por baixo do cartão da forma.
#[test]
fn corpo_zona_e_peca_tem_o_cartao_inteiro_logo_depois_da_forma() {
    mundo();
    for (nome, info, depois_de) in [
        ("corpo", with_body(), CARD_PHYS_SHAPE),
        ("zona", zona(), CARD_PHYS_SHAPE),
        (
            "peça",
            as_part(),
            ph2d_editor_core::ids::INSP_LIVE_PHYSICS_SECTION,
        ),
    ] {
        let (mut host, mut st) = cena(info, [1.0, 1.0]);
        let r = pinta(&mut host, &mut st);
        let (cima, cartao) = (
            achado(&r, grip_de(depois_de)).expect("o cartão de cima"),
            achado(&r, grip_de(CARD_PHYS_LAYERS)).unwrap_or_else(|| panic!("{nome}: sem o cartão")),
        );
        assert!(cartao.y > cima.y, "{nome}: fora de ordem");
        for id in INSP_PHYS_COLLIDES[..4].iter().copied().chain([
            INSP_PHYS_COLLIDES_ADD,
            INSP_PHYS_EDIT_MATRIX,
            INSP_PHYS_LAYER_PICK,
            INSP_PHYS_LAYER_RENAME,
        ]) {
            let caixa = achado(&r, id).unwrap_or_else(|| panic!("{nome}: {id:?} sumiu"));
            assert!(caixa.y > cartao.y, "{nome}: {id:?} acima do cartão");
        }
        sai();
    }
    fim();
}

/// A forma solta: só Layer e Rename, e o resumo só o nome, mesmo com camadas que tocariam.
#[test]
fn a_forma_solta_tem_o_cartao_so_com_layer_e_rename() {
    mundo();
    set_current_inspector_physics(None);
    let (mut host, mut st) = cena(na(2, solta()), [1.0, 1.0]);
    let r = pinta(&mut host, &mut st);
    assert!(achado(&r, grip_de(CARD_PHYS_LAYERS)).is_some());
    assert!(achado(&r, INSP_PHYS_LAYER_PICK).is_some());
    assert!(achado(&r, INSP_PHYS_LAYER_RENAME).is_some());
    for id in so_com_corpo_acima() {
        assert!(achado(&r, id).is_none(), "{id:?} na solta");
    }
    assert_eq!(resumo_do_cartao(CARD_PHYS_LAYERS).as_deref(), Some("Props"));
    fim();
}

/// Sem `Collider` não há cartão; e o Remove e o Undo levam-no e devolvem-no sem sobra.
#[test]
fn sem_collider_nao_aparece_e_o_remove_o_leva_sem_sobra() {
    mundo();
    let vazia = InspectorPhysicsInfo {
        has_body: false,
        has_collider: false,
        ..with_body()
    };
    let (mut host, mut st) = cena(with_body(), [1.0, 1.0]);
    let r = pinta(&mut host, &mut st);
    assert!(achado(&r, grip_de(CARD_PHYS_LAYERS)).is_some(), "controlo");
    let _ = clica(&mut host, &mut st, achado(&r, INSP_PHYS_REMOVE).unwrap());
    let sem_sobra = |r: &[(NodeId, _)]| {
        for id in so_com_corpo_acima().into_iter().chain([
            grip_de(CARD_PHYS_LAYERS),
            INSP_PHYS_LAYER_PICK,
            INSP_PHYS_LAYER_RENAME,
        ]) {
            assert!(achado(r, id).is_none(), "{id:?} sobrou");
        }
    };
    // O app devolve a face vazia (`has_collider: false`) ou nada, conforme o objecto.
    set_current_inspector_physics(Some(vazia));
    sem_sobra(&pinta(&mut host, &mut st));
    set_current_inspector_physics(None);
    let r = pinta(&mut host, &mut st);
    sem_sobra(&r);
    assert_eq!(
        clica(
            &mut host,
            &mut st,
            achado(&r, INSP_PHYS_REMOVED_UNDO).expect("o recibo")
        ),
        [EditorAction::UndoStep { redo: false }]
    );
    set_current_inspector_physics(Some(with_body()));
    assert!(
        achado(&pinta(&mut host, &mut st), grip_de(CARD_PHYS_LAYERS)).is_some(),
        "o Undo devolve-o"
    );
    fim();
}

/// A camada saiu do Shape: Shape dobrado não a leva; dobrar o cartão leva-a, e Touch e One-Way ficam no Shape.
#[test]
fn dobrar_cada_cartao_leva_so_o_que_e_dele() {
    mundo();
    let (mut host, mut st) = cena(with_body(), [1.0, 1.0]);
    host.store_mut().set_collapsed(CARD_PHYS_SHAPE, true);
    let r = pinta(&mut host, &mut st);
    assert!(achado(&r, INSP_PHYS_SENSOR[0]).is_none());
    assert!(achado(&r, INSP_PHYS_ONEWAY[0]).is_none());
    assert!(achado(&r, INSP_PHYS_LAYER_PICK).is_some());
    assert!(achado(&r, INSP_PHYS_COLLIDES[0]).is_some());
    host.store_mut().set_collapsed(CARD_PHYS_SHAPE, false);
    host.store_mut().set_collapsed(CARD_PHYS_LAYERS, true);
    let r = pinta(&mut host, &mut st);
    assert!(achado(&r, INSP_PHYS_SENSOR[0]).is_some());
    assert!(achado(&r, INSP_PHYS_ONEWAY[0]).is_some());
    for id in [
        INSP_PHYS_LAYER_PICK,
        INSP_PHYS_COLLIDES[0],
        INSP_PHYS_LAYER_RENAME,
        INSP_PHYS_EDIT_MATRIX,
    ] {
        assert!(achado(&r, id).is_none(), "{id:?} no cartão dobrado");
    }
    fim();
}

/// O começo do título do cartão: o resto da faixa pode ser ação.
fn no_titulo(r: &[(NodeId, Rect)]) -> Rect {
    let t = achado(r, grip_de(CARD_PHYS_LAYERS)).expect("o título");
    Rect::new(t.x, t.y, t.h, t.h)
}

/// O clique no título dobra de verdade; dobrado o resumo aparece, aberto não, e o clique volta a abrir.
#[test]
fn o_clique_no_titulo_dobra_e_o_resumo_aparece_no_titulo() {
    mundo();
    let (mut host, mut st) = cena(na(2, with_body()), [1.0, 1.0]);
    let mede = |host: &mut _, st: &mut _| {
        let (_, censo) = card_censo::grava(|| pinta(host, st));
        censo
            .titulos
            .iter()
            .rev()
            .find(|t| t.titulo == "Collision Layers")
            .cloned()
            .expect("o cartão se chama Collision Layers")
    };
    assert!(
        mede(&mut host, &mut st).resumo_x.is_none(),
        "aberto, sem resumo"
    );
    assert!(!host.store().is_collapsed(CARD_PHYS_LAYERS), "nasce aberto");
    let r = pinta(&mut host, &mut st);
    let _ = clica(&mut host, &mut st, no_titulo(&r));
    assert!(
        host.store().is_collapsed(CARD_PHYS_LAYERS),
        "o clique dobra"
    );
    let medido = mede(&mut host, &mut st);
    assert!(medido.resumo_x.is_some(), "{medido:?}");
    let r = pinta(&mut host, &mut st);
    let _ = clica(&mut host, &mut st, no_titulo(&r));
    assert!(!host.store().is_collapsed(CARD_PHYS_LAYERS), "e reabre");
    fim();
}

/// O título dobrado nunca fica mudo: a fila inteira onde cabe; onde não cabe, o nome inteiro, nunca a meio.
#[test]
fn a_fila_longa_cede_ao_nome_da_camada_sem_cortar() {
    mundo();
    let x_em = |info: InspectorPhysicsInfo, largura: f32| {
        let (mut host, mut st) = cena(info, [1.0, 1.0]);
        host.store_mut().set_collapsed(CARD_PHYS_LAYERS, true);
        let (_, censo) = card_censo::grava(|| {
            crate::o_cartao_da_sprite::pinta_na_janela(&mut host, &mut st, largura, 3200.0)
        });
        let t = censo
            .titulos
            .iter()
            .rev()
            .find(|t| t.titulo == "Collision Layers")
            .cloned()
            .expect("o cartão");
        sai();
        t
    };
    let piso = ph2d_editor_core::interaction::WidgetStore::DOCK_W_MIN;
    for largura in [piso, 230.0, 250.0, 270.0, 294.0, 340.0, 420.0, 600.0] {
        let t = x_em(na(2, with_body()), largura);
        let x = t
            .resumo_x
            .unwrap_or_else(|| panic!("{largura}: mudo: {t:?}"));
        assert!(x >= t.fim, "{largura}: {t:?}");
    }
    let (fila, nome) = (x_em(na(2, with_body()), 600.0), x_em(na(2, solta()), 600.0));
    assert!(
        fila.resumo_x.unwrap() + 40.0 < nome.resumo_x.unwrap(),
        "largo, a fila inteira: {fila:?} × {nome:?}"
    );
    let (fila, nome) = (x_em(na(2, with_body()), piso), x_em(na(2, solta()), piso));
    assert_eq!(fila.resumo_x, nome.resumo_x, "estreito, só o nome");
    fim();
}

/// Sem nada escolhido nasce aberto; depois manda o artista: dobrado continua dobrado.
#[test]
fn nasce_aberto_uma_vez_e_depois_manda_o_artista() {
    mundo();
    ph2d_panel_inspector::set_cards_on(true);
    ph2d_panel_inspector::set_current_inspector_sprite(None);
    set_current_inspector_physics(Some(with_body()));
    let mut host = crate::host_em_metros();
    let mut st = ph2d_panel_inspector::InspectorState::default();
    let r = pinta(&mut host, &mut st);
    assert_eq!(host.store().collapsed_choice(CARD_PHYS_LAYERS), Some(false));
    assert!(achado(&r, grip_de(CARD_PHYS_LAYERS)).is_some());
    host.store_mut().set_collapsed(CARD_PHYS_LAYERS, true);
    let _ = pinta(&mut host, &mut st);
    assert!(host.store().is_collapsed(CARD_PHYS_LAYERS));
    fim();
}

/// O exemplo da decisão do dono; cada camada lê a SUA linha, e nomes vazios usam os números.
#[test]
fn o_resumo_diz_a_camada_e_os_quadros_acesos_dela() {
    mundo();
    let caso = |info| {
        set_current_inspector_physics(Some(info));
        resumo_do_cartao(CARD_PHYS_LAYERS)
    };
    let props = caso(na(2, with_body()));
    assert_eq!(props.as_deref(), Some("Props \u{b7} Ground, Props, Zones"));
    let player = caso(na(1, with_body()));
    assert_eq!(player.as_deref(), Some("Player \u{b7} Ground"));
    let zonas = caso(na(3, zona()));
    assert_eq!(zonas.as_deref(), Some("Zones \u{b7} Props"));
    let peca = caso(na(1, as_part()));
    assert_eq!(peca.as_deref(), Some("Player \u{b7} Ground"));
    let mut linhas = [0u32; 32];
    linhas[1] = 1 << 3;
    set_current(Default::default(), linhas, 0b1011);
    assert_eq!(
        caso(na(1, with_body())).as_deref(),
        Some("Layer 1 \u{b7} Layer 3")
    );
    assert_eq!(
        caso(na(0, with_body())).as_deref(),
        Some("Default \u{b7} None")
    );
    fim();
}

/// Os ids da fila chegam à matriz pelo clique real, com o Shape dobrado: o cartão não depende dele.
#[test]
fn os_quadros_e_o_edit_matrix_chegam_pelo_clique_com_o_shape_dobrado() {
    mundo();
    let (mut host, mut st) = cena(na(2, with_body()), [1.0, 1.0]);
    host.store_mut().set_collapsed(CARD_PHYS_SHAPE, true);
    let r = pinta(&mut host, &mut st);
    assert_eq!(
        clica(
            &mut host,
            &mut st,
            achado(&r, INSP_PHYS_COLLIDES[1]).unwrap()
        ),
        [EditorAction::LayerPairToggle { a: 2, b: 1 }]
    );
    assert_eq!(
        clica(
            &mut host,
            &mut st,
            achado(&r, INSP_PHYS_EDIT_MATRIX).unwrap()
        ),
        [EditorAction::LayerMatrixOpen { layer: 2 }]
    );
    fim();
}

/// ⭐ Onda 8: duplo clique no chip da Layer abre o nome ali, com foco; escrever não grava; Enter grava UM
/// passo; Esc desiste; vazio grava vazio (volta a «Layer n»). O campo vazio mostra «Layer n».
#[test]
fn duplo_clique_no_chip_da_layer_renomeia_num_passo() {
    use ph2d_editor_core::interaction::WidgetEvent;
    mundo();
    let (mut host, mut st) = cena(na(2, with_body()), [1.0, 1.0]);
    let r = pinta(&mut host, &mut st);
    assert!(
        achado(&r, INSP_PHYS_LAYER_PICK).is_some(),
        "o chip da Layer"
    );
    let evento = |host: &mut ph2d_ui_testkit::MockPanelHost,
                  st: &mut ph2d_panel_inspector::InspectorState,
                  ev: WidgetEvent| {
        let _ = host.drained_actions();
        let _ = host.apply_panel_event::<ph2d_panel_inspector::InspectorPanel>(st, ev);
        host.drained_actions()
    };
    let _ = evento(
        &mut host,
        &mut st,
        WidgetEvent::DoubleClick(INSP_PHYS_LAYER_PICK),
    );
    assert_eq!(host.store().focus_id(), Some(INSP_PHYS_LAYER_NAMES[2]));
    assert!(
        !host.store().is_collapsed(INSP_PHYS_LAYER_RENAMING),
        "o campo abriu"
    );
    host.set_text(INSP_PHYS_LAYER_NAMES[2], "Mud");
    assert!(
        evento(
            &mut host,
            &mut st,
            WidgetEvent::TextChanged(INSP_PHYS_LAYER_NAMES[2])
        )
        .is_empty(),
        "escrever não grava"
    );
    assert_eq!(
        evento(
            &mut host,
            &mut st,
            WidgetEvent::Submit(INSP_PHYS_LAYER_NAMES[2])
        ),
        [EditorAction::LayerRename {
            layer: 2,
            name: "Mud".into()
        }]
    );
    assert!(
        !host.store().is_collapsed(INSP_PHYS_LAYER_RENAMING),
        "Enter fica no modo Rename (passa ao campo seguinte)"
    );
    let _ = evento(
        &mut host,
        &mut st,
        WidgetEvent::DoubleClick(INSP_PHYS_LAYER_PICK),
    );
    host.set_text(INSP_PHYS_LAYER_NAMES[2], "Nope");
    assert!(
        evento(
            &mut host,
            &mut st,
            WidgetEvent::Cancel(INSP_PHYS_LAYER_NAMES[2])
        )
        .is_empty(),
        "Esc nunca grava"
    );
    let _ = evento(
        &mut host,
        &mut st,
        WidgetEvent::DoubleClick(INSP_PHYS_LAYER_PICK),
    );
    host.set_text(INSP_PHYS_LAYER_NAMES[2], "  ");
    assert_eq!(
        evento(
            &mut host,
            &mut st,
            WidgetEvent::Submit(INSP_PHYS_LAYER_NAMES[2])
        ),
        [EditorAction::LayerRename {
            layer: 2,
            name: String::new()
        }],
        "vazio volta a «Layer n»"
    );
    fim();
}

/// ⭐ Onda 9c (dono 07/11): «Rename layers…» pelo clique REAL troca as caixinhas pela lista de TODAS as camadas
/// visíveis, cada uma com o seu campo e o nome INTEIRO; Enter num campo grava ESSA camada (não a do objecto) e
/// passa ao seguinte; Esc sai sem gravar; «Done» sai e as caixinhas voltam. O botão diz «Rename layers…».
#[test]
fn o_modo_rename_lista_todas_as_camadas_e_grava_a_do_campo() {
    use ph2d_editor_core::interaction::WidgetEvent;
    mundo();
    let mut nomes: [String; 32] = Default::default();
    for (i, n) in [
        "Ground",
        "Player Shots From The Big Boss Ship",
        "Props",
        "Zones",
    ]
    .into_iter()
    .enumerate()
    {
        nomes[i] = n.into();
    }
    let linhas = [u32::MAX; 32];
    set_current(nomes, linhas, 0b1111);
    let (mut host, mut st) = cena(na(2, with_body()), [1.0, 1.0]);
    let r = pinta(&mut host, &mut st);
    assert!(
        achado(&r, INSP_PHYS_COLLIDES[1]).is_some(),
        "as caixinhas antes"
    );
    assert!(
        achado(&r, INSP_PHYS_LAYER_NAMES[0]).is_none(),
        "sem campos antes"
    );
    assert_eq!(
        ph2d_i18n::tr("card.insp.phys.layer_rename"),
        "Rename layers…"
    );
    let _ = clica(
        &mut host,
        &mut st,
        achado(&r, INSP_PHYS_LAYER_RENAME).unwrap(),
    );
    let r = pinta(&mut host, &mut st);
    for j in 0..4 {
        assert!(
            achado(&r, INSP_PHYS_LAYER_NAMES[j]).is_some(),
            "o campo da camada {j}"
        );
        assert!(
            achado(&r, INSP_PHYS_COLLIDES[j]).is_none(),
            "a caixinha {j} sumiu"
        );
    }
    assert!(
        achado(&r, INSP_PHYS_LAYER_NAMES[4]).is_none(),
        "só as visíveis"
    );
    assert_eq!(
        host.store().focus_id(),
        Some(INSP_PHYS_LAYER_NAMES[2]),
        "foco na do objecto"
    );
    assert_eq!(
        host.store().text(INSP_PHYS_LAYER_NAMES[1]),
        Some("Player Shots From The Big Boss Ship"),
        "o campo mostra o nome inteiro"
    );
    let evento = |host: &mut ph2d_ui_testkit::MockPanelHost,
                  st: &mut ph2d_panel_inspector::InspectorState,
                  ev: WidgetEvent| {
        let _ = host.drained_actions();
        let _ = host.apply_panel_event::<ph2d_panel_inspector::InspectorPanel>(st, ev);
        host.drained_actions()
    };
    host.set_text(INSP_PHYS_LAYER_NAMES[1], "Rock");
    assert_eq!(
        evento(
            &mut host,
            &mut st,
            WidgetEvent::Submit(INSP_PHYS_LAYER_NAMES[1])
        ),
        [EditorAction::LayerRename {
            layer: 1,
            name: "Rock".into()
        }],
        "grava a camada do CAMPO, um passo"
    );
    assert_eq!(
        host.store().focus_id(),
        Some(INSP_PHYS_LAYER_NAMES[2]),
        "passa ao seguinte"
    );
    host.set_text(INSP_PHYS_LAYER_NAMES[3], "Nope");
    assert!(
        evento(
            &mut host,
            &mut st,
            WidgetEvent::Cancel(INSP_PHYS_LAYER_NAMES[3])
        )
        .is_empty(),
        "Esc nunca grava"
    );
    assert!(
        host.store().is_collapsed(INSP_PHYS_LAYER_RENAMING),
        "Esc sai"
    );
    // O 2.º clique no MESMO botão com o relógio do mock parado seria um duplo clique: vai pelo evento.
    let _ = evento(
        &mut host,
        &mut st,
        WidgetEvent::Click(INSP_PHYS_LAYER_RENAME),
    );
    let r = pinta(&mut host, &mut st);
    let _ = clica(
        &mut host,
        &mut st,
        achado(&r, INSP_PHYS_LAYER_RENAME_DONE).unwrap(),
    );
    let r = pinta(&mut host, &mut st);
    assert!(
        achado(&r, INSP_PHYS_COLLIDES[1]).is_some(),
        "Done: as caixinhas voltam"
    );
    assert!(achado(&r, INSP_PHYS_LAYER_NAMES[1]).is_none());
    fim();
}
