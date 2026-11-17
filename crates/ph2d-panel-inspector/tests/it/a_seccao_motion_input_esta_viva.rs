//! **A secção MOTION INPUT é PINTADA e está VIVA sob o dedo** (ciclo 19 do Motion).
//!
//! O gesto é REAL (`Down` + `Up` no rectângulo que a pintura registou): um `WidgetEvent::Click`
//! sintético passaria sobre um controlo ausente do `populate`.

use ph2d_editor_core::action_bus::{ComponentEdit, EditorAction};
use ph2d_editor_core::motion_input_edits::{
    InspectorMotionInputInfo, MotionInputAoVivo as V, MotionInputFieldEdit as E,
    MotionInputOutro as O, SOURCE_COLOR, SOURCE_DAMAGE, SOURCE_HEALTH, SOURCE_VALUE,
};
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_inspector::{InspectorPanel, InspectorState, set_current_inspector_motion_input};
use ph2d_panel_inspector::{censo_dos_avisos, ids};
use ph2d_ui_testkit::MockPanelHost;

pub(super) const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 2600.0,
};
pub(super) const SEC: u128 = 1_000_000_000;
pub(super) const BITS: u64 = 0x000A_11CE;

pub(super) fn pointer(kind: PointerKind, x: f32, y: f32, t: u128) -> PointerEvent {
    PointerEvent {
        kind,
        x,
        y,
        button: PointerButton::Primary,
        source: PointerSource::Mouse,
        pressure: 1.0,
        timestamp_ns: t,
    }
}

pub(super) fn info(source: u8) -> InspectorMotionInputInfo {
    InspectorMotionInputInfo {
        entity_bits: BITS,
        name: "energy".into(),
        source,
        value: 0.5,
        color: [1.0; 4],
        nomes_do_grafo: vec!["energy".into(), "speed".into()],
        tem_vida: true,
        from: String::new(),
        alvo_achado: true,
        tem_corpo: false,
        action: String::new(),
        acoes_do_mapa: Vec::new(),
        property: String::new(),
        campos_do_alvo: Vec::new(),
        campos_par: Vec::new(),
        pai: String::new(),
        alvo_bits: None,
        remap: false,
        remap_from: 0.0,
        remap_to: 1.0,
        curve: String::new(),
        smooth: 0.0,
        from_pick_armed: false,
        selected_count: 1,
        a_correr: false,
        ao_vivo: None,
        tapado_por: None,
    }
}

pub(super) fn pinta(
    i: InspectorMotionInputInfo,
) -> (
    MockPanelHost,
    InspectorState,
    Vec<(ph2d_a11y::NodeId, Rect)>,
) {
    let mut h = MockPanelHost::with_panel::<InspectorPanel>();
    set_current_inspector_motion_input(Some(i));
    let mut st = InspectorState::default();
    let rects = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    (h, st, rects)
}

/// As frases da secção pintadas por `i` (o censo dos avisos, só desta secção).
pub(super) fn frases(i: InspectorMotionInputInfo) -> Vec<String> {
    let mut h = MockPanelHost::with_panel::<InspectorPanel>();
    set_current_inspector_motion_input(Some(i));
    let mut st = InspectorState::default();
    let (_, avisos) = censo_dos_avisos::medindo(|| h.paint::<InspectorPanel>(&mut st, VIEWPORT));
    avisos
        .into_iter()
        .filter(|a| a.seccao() == "motion_input")
        .map(|a| a.texto)
        .collect()
}

pub(super) fn rect_de(rects: &[(ph2d_a11y::NodeId, Rect)], id: ph2d_a11y::NodeId) -> Option<Rect> {
    rects.iter().find(|(n, _)| *n == id).map(|(_, r)| *r)
}

/// Um clique real no centro de `id`; devolve as edições da secção que chegaram ao barramento.
pub(super) fn clica(
    h: &mut MockPanelHost,
    st: &mut InspectorState,
    rects: &[(ph2d_a11y::NodeId, Rect)],
    id: ph2d_a11y::NodeId,
) -> Vec<E> {
    let r = rect_de(rects, id).unwrap_or_else(|| panic!("{id:?} não foi pintado"));
    let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    let _ = h.drained_actions();
    let mut evs = h.dispatch_pointer_event(pointer(PointerKind::Down, cx, cy, SEC));
    evs.extend(h.dispatch_pointer_event(pointer(PointerKind::Up, cx, cy, SEC + SEC / 100)));
    assert!(
        !evs.is_empty(),
        "o ponteiro sobre {id:?} não virou evento — pintado e ausente do `populate`"
    );
    for ev in evs {
        h.apply_panel_event::<InspectorPanel>(st, ev);
    }
    h.drained_actions()
        .iter()
        .filter_map(|a| match a {
            EditorAction::InspectorComponentEdit {
                entity_bits,
                edit: ComponentEdit::MotionInput(e),
            } => {
                assert_eq!(*entity_bits, BITS);
                Some(e.clone())
            }
            _ => None,
        })
        .collect()
}

/// As linhas `Value` e `Color` só existem com a fonte que as lê; nome, chips e fonte sempre.
///
/// **Mutação que deve sangrar:** pintar a linha do número sem o `if i.le_valor()`.
#[test]
fn so_a_fonte_que_le_um_campo_o_mostra() {
    for (fonte, valor, cor) in [
        (SOURCE_VALUE, true, false),
        (SOURCE_COLOR, false, true),
        (SOURCE_HEALTH, false, false),
    ] {
        let (_h, _st, rects) = pinta(info(fonte));
        let pintado = |id| rect_de(&rects, id).is_some();
        assert!(
            !pintado(ids::INSP_MI_NAME),
            "B: com o nome lido pelo grafo, a escolha são os chips — sem caixa (fonte {fonte})"
        );
        assert!(pintado(ids::INSP_MI_NEW), "«New…» no fim dos chips");
        for id in ids::INSP_MI_SOURCE {
            assert!(pintado(id), "a fonte pinta-se (fonte {fonte})");
        }
        for id in &ids::INSP_MI_GRAFO[..2] {
            assert!(pintado(*id), "um chip por nome do grafo (fonte {fonte})");
        }
        assert!(
            !pintado(ids::INSP_MI_GRAFO[2]),
            "só há dois nomes no grafo — o terceiro chip não existe"
        );
        assert_eq!(pintado(ids::INSP_MI_VALUE), valor, "número (fonte {fonte})");
        assert_eq!(pintado(ids::INSP_MI_COLOR), cor, "cor (fonte {fonte})");
    }
}

/// O clique numa FONTE envia o índice dela; o clique num CHIP do grafo escreve aquele nome.
#[test]
fn a_fonte_e_o_chip_do_grafo_editam_o_componente() {
    let (mut h, mut st, rects) = pinta(info(SOURCE_VALUE));
    assert_eq!(
        clica(&mut h, &mut st, &rects, ids::INSP_MI_SOURCE[3]),
        vec![E::Source(3)]
    );
    assert_eq!(
        clica(&mut h, &mut st, &rects, ids::INSP_MI_GRAFO[1]),
        vec![E::Name("speed".into())],
        "o chip escreve o nome que o grafo lê"
    );
}

/// Sem instantâneo, nada da secção se pinta.
#[test]
fn sem_componente_nao_ha_seccao() {
    let mut h = MockPanelHost::with_panel::<InspectorPanel>();
    set_current_inspector_motion_input(None);
    let mut st = InspectorState::default();
    let rects = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    assert!(rect_de(&rects, ids::INSP_MI_NAME).is_none());
}

/// ⭐ B: a caixa do nome aparece só para um nome que o grafo não lê (ou sem grafo, como a fileira
/// «Input»); «New…» esvazia o nome e foca a caixa.
///
/// **Mutação que deve sangrar:** pintar a caixa sempre, ou «New…» sem esvaziar.
#[test]
fn a_caixa_do_nome_so_aparece_para_um_nome_novo() {
    let mut fora = info(SOURCE_VALUE);
    fora.name = "wind".into();
    let (_h, _st, rects) = pinta(fora);
    assert!(
        rect_de(&rects, ids::INSP_MI_NAME).is_some(),
        "um nome que o grafo não lê"
    );
    let mut sem = info(SOURCE_VALUE);
    sem.nomes_do_grafo.clear();
    let (_h, _st, rects) = pinta(sem);
    assert!(
        rect_de(&rects, ids::INSP_MI_NAME).is_some(),
        "sem grafo, a caixa é a fileira"
    );
    assert!(
        rect_de(&rects, ids::INSP_MI_NEW).is_none(),
        "sem chips, sem «New…»"
    );

    let (mut h, mut st, rects) = pinta(info(SOURCE_VALUE));
    assert_eq!(
        clica(&mut h, &mut st, &rects, ids::INSP_MI_NEW),
        vec![E::Name(String::new())]
    );
    assert_eq!(
        h.store().focus_id(),
        Some(ids::INSP_MI_NAME),
        "a caixa fica a escrever"
    );
}

/// ⭐ A saída do aviso «sem Health»: o botão só existe com a queixa, e o clique PEDE a `Health`
/// pela porta da paleta (não é uma edição do componente).
#[test]
fn sem_health_o_aviso_tem_a_saida() {
    let (_h, _st, rects) = pinta(info(SOURCE_HEALTH));
    assert!(
        rect_de(&rects, ids::INSP_MI_ADD_HEALTH).is_none(),
        "com Health não há botão"
    );
    let mut outra = info(SOURCE_HEALTH);
    outra.name = "wind".into();
    let (_h, _st, rects) = pinta(outra);
    assert!(
        rect_de(&rects, ids::INSP_MI_ADD_HEALTH).is_none(),
        "outra queixa (nome que o grafo não lê) não tem a saída da Health"
    );
    let mut sem = info(SOURCE_HEALTH);
    sem.tem_vida = false;
    let (mut h, mut st, rects) = pinta(sem);
    let r = rect_de(&rects, ids::INSP_MI_ADD_HEALTH).expect("o botão Add Health");
    let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    let _ = h.drained_actions();
    let mut evs = h.dispatch_pointer_event(pointer(PointerKind::Down, cx, cy, SEC));
    evs.extend(h.dispatch_pointer_event(pointer(PointerKind::Up, cx, cy, SEC + SEC / 100)));
    for ev in evs {
        h.apply_panel_event::<InspectorPanel>(&mut st, ev);
    }
    assert!(
        h.drained_actions().iter().any(|a| matches!(
            a,
            EditorAction::InspectorAddComponentRequested {
                entity_bits: BITS,
                component: Some("ph2d::physics::Health")
            }
        )),
        "o clique pede a Health"
    );
}

/// ⭐ MI-11: o cartão diz o que chega ao grafo — a vida lida (também parado), o que manda em Play,
/// os disparos do dano, e quem tapa o nome.
#[test]
fn o_cartao_diz_o_que_chega_ao_grafo() {
    let mut vida = info(SOURCE_HEALTH);
    vida.ao_vivo = Some(V::Numero(0.75));
    assert!(frases(vida).contains(&"Reads Health here: 0.75".to_string()));

    let mut valor = info(SOURCE_VALUE);
    valor.ao_vivo = Some(V::Numero(0.5));
    assert!(
        !frases(valor.clone()).iter().any(|f| f.starts_with("Sends")),
        "parado, o número é o do campo: sem frase"
    );
    valor.a_correr = true;
    valor.tapado_por = Some((O::Script, V::Numero(0.2)));
    let f = frases(valor);
    assert!(f.contains(&"Sends 0.50".to_string()), "{f:?}");
    assert!(
        f.contains(
            &"A script sets this name first (0.20) \u{2014} the graph reads that.".to_string()
        ),
        "{f:?}"
    );

    let mut dano = info(SOURCE_DAMAGE);
    dano.a_correr = true;
    dano.ao_vivo = Some(V::Disparos(3));
    assert!(frases(dano.clone()).contains(&"Fired 3 times".to_string()));
    dano.ao_vivo = Some(V::Disparos(1));
    assert!(frases(dano).contains(&"Fired once".to_string()));
}

/// Em Play a vida chega ao grafo já moldada (Remap/curva/Smooth): «Reads Health here» mentiria.
#[test]
fn em_play_a_vida_moldada_e_o_que_manda_nao_o_que_le() {
    let mut vida = info(SOURCE_HEALTH);
    vida.ao_vivo = Some(V::Numero(0.75));
    vida.a_correr = true;
    let f = frases(vida);
    assert!(f.contains(&"Sends 0.75".to_string()), "{f:?}");
    assert!(!f.iter().any(|s| s.starts_with("Reads Health")), "{f:?}");
}
