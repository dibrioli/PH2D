//! ⭐⭐⭐ **As secções NAV REGION e NAV AGENT são PINTADAS, estão VIVAS sob o dedo, e mostram os
//! números DO OBJECTO** (plano 30, W4).
//!
//! Irmã do [`super::a_seccao_vida_esta_viva`], e pelo mesmo motivo: *a semente é a única metade de
//! uma secção cujo sujeito é o WIDGET* — os gates da lei e do dreno entram **abaixo** do
//! `WidgetStore`.
//!
//! # ⭐ E a linha do ALVO muda com o MODO
//!
//! `Object` pinta o nome, `Point` pinta o ponto, `None` nenhum dos dois — ⇒ o gate tem as duas
//! metades: a linha do modo é pintada, e a do outro modo **não** (um controlo morto em cada modo
//! passaria só na primeira).

use ph2d_editor_core::action_bus::{ComponentEdit, EditorAction};
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::nav_edits::{
    InspectorNavAgent, InspectorNavInfo, InspectorNavRegion, NavAlvoModo, NavFieldEdit as E,
};
use ph2d_editor_core::zones::Rect;
use ph2d_panel_inspector::{InspectorPanel, InspectorState, ids, set_current_inspector_nav};
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 3200.0,
};

/// Uma região fora do neutro em todo campo (`NavRegion::default()` é `10 × 10` e todas as camadas).
fn regiao() -> InspectorNavRegion {
    InspectorNavRegion {
        half_w: 6.5,
        half_h: 3.25,
        obstacle_layers: 0b0000_0101,
    }
}

/// Um agente fora do neutro em todo campo — *um corpus no NEUTRO de um knob não testa esse knob*.
fn agente(modo: NavAlvoModo) -> InspectorNavAgent {
    InspectorNavAgent {
        alvo_modo: modo,
        alvo_nome: "Hero".into(),
        alvo_perdido: false,
        alvo_tag: 0,
        alvo_ponto: [4.5, -2.0],
        radius: 0.65,
        arrive: 0.9,
        repath: 1.25,
        stuck_after: 2.5,
        active: true,
        avoidance: true,
        avoid_harm: true,
        has_health: true,
        on_arrived: "chegou".into(),
        on_no_path: "longe".into(),
        on_stuck: "preso".into(),
        has_body: true,
        has_mover: true,
        mover_reads_keys: false,
        has_platformer: false,
        in_region: true,
        agora: None,
    }
}

fn info(region: Option<InspectorNavRegion>, agent: Option<InspectorNavAgent>) -> InspectorNavInfo {
    InspectorNavInfo {
        entity_bits: 0x00CD_5678,
        region,
        agent,
        cost_area: None,
        link: None,
        clock_playing: true,
        selected_count: 1,
    }
}

fn host(i: InspectorNavInfo) -> (MockPanelHost, InspectorState) {
    let h = MockPanelHost::with_panel::<InspectorPanel>();
    set_current_inspector_nav(Some(i));
    (h, InspectorState::default())
}

fn pintado(rects: &[(ph2d_a11y::NodeId, Rect)], id: ph2d_a11y::NodeId) -> bool {
    rects
        .iter()
        .any(|(n, r)| *n == id && r.w > 0.0 && r.h > 0.0)
}

/// Os números de cada secção no modo `Object`, com o valor que a fixtura tem.
const NUMEROS: [(ph2d_a11y::NodeId, f64); 6] = [
    (ids::INSP_NAV_HALF_W, 6.5),
    (ids::INSP_NAV_HALF_H, 3.25),
    (ids::INSP_NAV_RADIUS, 0.65),
    (ids::INSP_NAV_ARRIVE, 0.9),
    (ids::INSP_NAV_REPATH, 1.25),
    (ids::INSP_NAV_STUCK, 2.5),
];

/// Os nomes no modo `Object`.
const NOMES: [(ph2d_a11y::NodeId, &str); 4] = [
    (ids::INSP_NAV_TARGET_NAME, "Hero"),
    (ids::INSP_NAV_ON_ARRIVED, "chegou"),
    (ids::INSP_NAV_ON_NO_PATH, "longe"),
    (ids::INSP_NAV_ON_STUCK, "preso"),
];

/// ⭐⭐ **TODO campo das duas secções é pintado com área clicável** — as oito camadas, os três
/// modos, o interruptor, os números e os nomes.
///
/// **Mutação que deve sangrar:** tirar qualquer linha dos dois pintores.
#[test]
fn todo_campo_das_duas_seccoes_e_pintado() {
    let (mut h, mut st) = host(info(Some(regiao()), Some(agente(NavAlvoModo::Objecto))));
    let rects = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    let todos = NUMEROS
        .iter()
        .map(|(i, _)| *i)
        .chain(NOMES.iter().map(|(i, _)| *i))
        .chain(ids::INSP_NAV_LAYERS)
        .chain(ids::INSP_NAV_TARGET_MODE)
        .chain([ids::INSP_NAV_ACTIVE, ids::INSP_NAV_AVOIDANCE]);
    for id in todos {
        assert!(
            pintado(&rects, id),
            "o campo {id:?} nao foi PINTADO com area clicavel"
        );
    }
    set_current_inspector_nav(None);
}

/// ⭐⭐⭐ **A linha do alvo segue o MODO** — e a do outro modo não é pintada.
///
/// **Mutações que devem sangrar:** pintar o nome fora do modo `Object` · pintar o ponto fora do
/// modo `Point`.
#[test]
fn a_linha_do_alvo_segue_o_modo() {
    let casos = [
        (NavAlvoModo::Nenhum, false, false),
        (NavAlvoModo::Objecto, true, false),
        (NavAlvoModo::Ponto, false, true),
    ];
    for (modo, nome, ponto) in casos {
        let (mut h, mut st) = host(info(None, Some(agente(modo))));
        let rects = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
        assert_eq!(
            pintado(&rects, ids::INSP_NAV_TARGET_NAME),
            nome,
            "{modo:?}: o NOME"
        );
        assert_eq!(
            pintado(&rects, ids::INSP_NAV_TARGET_X),
            ponto,
            "{modo:?}: o PONTO x"
        );
        assert_eq!(
            pintado(&rects, ids::INSP_NAV_TARGET_Y),
            ponto,
            "{modo:?}: o PONTO y"
        );
        // O CONTROLO: o resto da secção continua lá em todo modo.
        assert!(
            pintado(&rects, ids::INSP_NAV_RADIUS),
            "{modo:?}: a secção apagou-se"
        );
        set_current_inspector_nav(None);
    }
}

/// ⭐⭐⭐ **TODO campo está VIVO SOB O DEDO** — o gesto REAL, e não um `WidgetEvent` sintético.
///
/// **Mutação que deve sangrar:** tirar qualquer id do `populate_nav`.
#[test]
fn todo_campo_esta_vivo_sob_o_dedo() {
    let ponto = [ids::INSP_NAV_TARGET_X, ids::INSP_NAV_TARGET_Y];
    for id in NUMEROS
        .iter()
        .map(|(i, _)| *i)
        .chain(NOMES.iter().map(|(i, _)| *i))
        .chain(ponto)
    {
        let modo = if ponto.contains(&id) {
            NavAlvoModo::Ponto
        } else {
            NavAlvoModo::Objecto
        };
        let (mut h, mut st) = host(info(Some(regiao()), Some(agente(modo))));
        let rects = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
        let r = rects
            .iter()
            .find(|(n, _)| *n == id)
            .map(|(_, r)| *r)
            .unwrap_or_else(|| panic!("{id:?} nao foi pintado"));
        let _ = h.click_at(r.x + r.w * 0.5, r.y + r.h * 0.5);
        assert_eq!(
            h.store().focus_id(),
            Some(id),
            "clicar no meio de {id:?} nao lhe deu o foco — falta o registo no `populate_nav`, e ele \
             esta' MORTO SOB O DEDO"
        );
        set_current_inspector_nav(None);
    }
}

/// ⭐⭐⭐ **OS CAMPOS MOSTRAM O QUE O OBJECTO TEM, e não os valores de fábrica.**
///
/// **Mutação que deve sangrar:** tirar a chamada do `sync_nav` do `sync_sections`.
#[test]
fn os_campos_mostram_o_que_o_objecto_tem() {
    let (mut h, mut st) = host(info(Some(regiao()), Some(agente(NavAlvoModo::Objecto))));
    let _ = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    for (id, esperado) in NUMEROS.into_iter().chain([
        (ids::INSP_NAV_TARGET_X, 4.5),
        (ids::INSP_NAV_TARGET_Y, -2.0),
    ]) {
        let lido = h
            .store()
            .number_value(id)
            .unwrap_or_else(|| panic!("o campo {id:?} nem sequer esta' registado"));
        assert!(
            (lido - esperado).abs() < 1.0e-5,
            "o campo {id:?} mostra {lido} e o objecto tem {esperado} — o painel mostra os valores \
             de FABRICA do `populate_nav`"
        );
    }
    for (id, esperado) in NOMES {
        assert_eq!(
            h.store().text(id),
            Some(esperado),
            "o campo de texto {id:?}"
        );
    }
    set_current_inspector_nav(None);
}

/// ⭐⭐⭐ **Cada secção só existe para quem TEM o componente dela** (ADR-0166).
#[test]
fn cada_seccao_so_existe_para_quem_tem_o_componente() {
    let (mut h, mut st) = host(info(Some(regiao()), None));
    let rects = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    assert!(
        pintado(&rects, ids::INSP_NAV_HALF_W),
        "o CONTROLO: a região é pintada"
    );
    assert!(
        !pintado(&rects, ids::INSP_NAV_RADIUS),
        "o agente foi pintado SEM agente"
    );
    set_current_inspector_nav(None);

    let mut h = MockPanelHost::with_panel::<InspectorPanel>();
    let mut st = InspectorState::default();
    let rects = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    assert!(
        !pintado(&rects, ids::INSP_NAV_HALF_W) && !pintado(&rects, ids::INSP_NAV_RADIUS),
        "uma das secções foi pintada sem instantâneo nenhum"
    );
}

fn edicoes(h: &mut MockPanelHost) -> Vec<E> {
    h.drained_actions()
        .into_iter()
        .filter_map(|a| match a {
            EditorAction::InspectorComponentEdit {
                edit: ComponentEdit::Nav(e),
                ..
            } => Some(e),
            _ => None,
        })
        .collect()
}

/// ⭐⭐⭐ **Um clique numa CAMADA troca UM bit da máscara do OBJECTO**, e um clique num MODO pede
/// esse modo — lidos do instantâneo, nunca do store.
///
/// **Mutações que devem sangrar:** `|` em vez de `^` (nunca desliga) · trocar a ordem dos ids dos
/// modos · o interruptor deixar de inverter.
#[test]
fn as_camadas_os_modos_e_o_interruptor_pedem_o_que_o_ecra_promete() {
    for (i, &id) in ids::INSP_NAV_LAYERS.iter().enumerate() {
        let (mut h, mut st) = host(info(Some(regiao()), None));
        let _ = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
        let _ = h.apply_panel_event::<InspectorPanel>(&mut st, WidgetEvent::Click(id));
        assert_eq!(
            edicoes(&mut h),
            vec![E::ObstacleLayers(0b0000_0101 ^ (1 << i))],
            "a camada {i} tem de trocar SÓ o bit dela (a 0 e a 2 estão acesas e DESLIGAM)"
        );
        set_current_inspector_nav(None);
    }
    for (i, &id) in ids::INSP_NAV_TARGET_MODE.iter().enumerate() {
        let (mut h, mut st) = host(info(None, Some(agente(NavAlvoModo::Objecto))));
        let _ = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
        let _ = h.apply_panel_event::<InspectorPanel>(&mut st, WidgetEvent::Click(id));
        assert_eq!(edicoes(&mut h), vec![E::AlvoModo(NavAlvoModo::ALL[i])]);
        set_current_inspector_nav(None);
    }
    let (mut h, mut st) = host(info(None, Some(agente(NavAlvoModo::Objecto))));
    let _ = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    let _ =
        h.apply_panel_event::<InspectorPanel>(&mut st, WidgetEvent::Toggled(ids::INSP_NAV_ACTIVE));
    assert_eq!(
        edicoes(&mut h),
        vec![E::Active(false)],
        "o interruptor pede o CONTRÁRIO"
    );
    set_current_inspector_nav(None);
}

/// ⭐⭐⭐ **(W5) «Avoid Others» sob o DEDO pede o contrário do que o objecto tem** — o clique REAL no
/// meio da caixa pintada, até ao barramento.
///
/// **Mutações que devem sangrar:** tirar o registo do `populate_nav` · tirar o braço do despacho ·
/// o braço pedir o mesmo valor · trocar o id do `Active` pelo do desvio.
#[test]
fn clicar_em_avoid_others_pede_o_contrario() {
    for tem in [true, false] {
        let mut a = agente(NavAlvoModo::Objecto);
        a.avoidance = tem;
        let (mut h, mut st) = host(info(None, Some(a)));
        let rects = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
        let r = rects
            .iter()
            .find(|(n, _)| *n == ids::INSP_NAV_AVOIDANCE)
            .map(|(_, r)| *r)
            .expect("«Avoid Others» não foi pintado");
        let evs = h.click_at(r.x + r.w * 0.5, r.y + r.h * 0.5);
        for ev in evs {
            let _ = h.apply_panel_event::<InspectorPanel>(&mut st, ev);
        }
        assert_eq!(
            edicoes(&mut h),
            vec![E::Avoidance(!tem)],
            "com o desvio a {tem}"
        );
        set_current_inspector_nav(None);
    }
}

/// ⭐⭐⭐ **O que se ESCREVE num nome chega ao BARRAMENTO, e com a variante DELE.**
///
/// **Mutações que devem sangrar:** tirar qualquer braço do `match` do texto · trocar dois.
#[test]
fn escrever_num_nome_chega_ao_barramento_com_a_variante_dele() {
    const ESCRITO: &str = "escrito-pelo-gate";
    let t = || ESCRITO.to_owned();
    let esperado = [
        (ids::INSP_NAV_TARGET_NAME, E::AlvoNome(t())),
        (ids::INSP_NAV_ON_ARRIVED, E::OnArrived(t())),
        (ids::INSP_NAV_ON_NO_PATH, E::OnNoPath(t())),
        (ids::INSP_NAV_ON_STUCK, E::OnStuck(t())),
    ];
    for (id, e) in esperado {
        let (mut h, mut st) = host(info(None, Some(agente(NavAlvoModo::Objecto))));
        let _ = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
        h.set_text(id, ESCRITO);
        let _ = h.apply_panel_event::<InspectorPanel>(&mut st, WidgetEvent::TextChanged(id));
        assert_eq!(edicoes(&mut h), vec![e], "escrever em {id:?}");
        set_current_inspector_nav(None);
    }
}

/// ⭐⭐ **(W6) Cada modo novo pinta a SUA linha e não a dos outros** — `Tag` o chip da tag,
/// `Patrol` o campo do nome (o da forma), e nenhum dos dois o ponto.
///
/// **Mutações que devem sangrar:** tirar um braço novo do pintor · pintar o chip em `Patrol`.
#[test]
fn os_modos_da_w6_pintam_a_sua_linha() {
    for (modo, sim, nao) in [
        (
            NavAlvoModo::Tag,
            ids::INSP_NAV_TAG_PICK,
            [ids::INSP_NAV_TARGET_NAME, ids::INSP_NAV_TARGET_X],
        ),
        (
            NavAlvoModo::Patrulha,
            ids::INSP_NAV_TARGET_NAME,
            [ids::INSP_NAV_TAG_PICK, ids::INSP_NAV_TARGET_X],
        ),
    ] {
        let (mut h, mut st) = host(info(None, Some(agente(modo))));
        let rects = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
        assert!(
            pintado(&rects, sim),
            "{modo:?}: a linha dele não foi pintada"
        );
        for id in nao {
            assert!(
                !pintado(&rects, id),
                "{modo:?}: pintou a linha de outro modo"
            );
        }
        set_current_inspector_nav(None);
    }
}

/// ⭐⭐⭐ **(W6) Escolher uma tag com o ponteiro REAL pede ESSA tag** — o chip abre a lista da árvore
/// do projecto, e a opção clicada chega ao barramento com o id dela.
///
/// **Mutações que devem sangrar:** tirar as opções do `populate_nav` · tirar o braço do despacho ·
/// o despacho ler a opção ao lado.
#[test]
fn escolher_uma_tag_com_o_ponteiro_pede_essa_tag() {
    use ph2d_editor_core::screens::hero::InspectorTagRow;
    let linha = |id: u64, path: &str, depth| InspectorTagRow {
        id,
        path: path.into(),
        label: path.rsplit('/').next().unwrap_or(path).into(),
        depth,
    };
    ph2d_panel_inspector::set_current_tag_tree(vec![
        linha(3, "Enemy", 0),
        linha(8, "Enemy/Flying", 1),
        linha(5, "Coin", 0),
    ]);
    let mut a = agente(NavAlvoModo::Tag);
    a.alvo_tag = 3;
    let (mut h, mut st) = host(info(None, Some(a)));
    let _ = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    h.set_dropdown_open(ids::INSP_NAV_TAG_PICK, true);
    let rects = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    let r = rects
        .iter()
        .find(|(n, _)| *n == ids::INSP_NAV_TAG_OPT[1])
        .map(|(_, r)| *r)
        .expect("a lista aberta não pintou a 2.ª tag");
    for ev in h.click_at(r.x + r.w * 0.5, r.y + r.h * 0.5) {
        let _ = h.apply_panel_event::<InspectorPanel>(&mut st, ev);
    }
    assert_eq!(
        edicoes(&mut h),
        vec![E::AlvoTag(8)],
        "a 2.ª opção é «Enemy/Flying»"
    );
    set_current_inspector_nav(None);
    ph2d_panel_inspector::set_current_tag_tree(Vec::new());
}
