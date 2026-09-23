//! ⭐⭐⭐ **A secção PARALLAX é PINTADA, está VIVA sob o dedo, e mostra os números DO OBJECTO**
//! (plano 24, W7).
//!
//! Irmã do [`super::a_seccao_weapon_esta_viva`], no mesmo molde e pelas mesmas razões — e com UMA
//! metade que as irmãs não têm: **os três blocos opcionais só aparecem com o componente deles**, e
//! um bloco que aparecesse sempre pintaria seis campos mortos em toda camada que não repete.

use ph2d_editor_core::parallax_edits::InspectorParallaxInfo;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_inspector::{InspectorPanel, InspectorState, ids, set_current_inspector_parallax};
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 2400.0,
};

/// ⭐ Uma camada que **NÃO** está no valor de fábrica em nenhum dos dez campos — *um corpus no NEUTRO
/// de um knob não testa esse knob*.
fn camada(completa: bool) -> InspectorParallaxInfo {
    InspectorParallaxInfo {
        entity_bits: 0x00AB_CDEF,
        factor: [0.35, 0.6],
        repeat: completa.then_some([7.0, 3.0]),
        motion: completa.then_some([0.35, -0.2]),
        limits: completa.then_some(([-30.0, -12.0], [30.0, 14.0])),
        camera: ph2d_editor_core::parallax_edits::CameraDoJogo::Activa,
        atravessa: false,
        pre_visualizacao: true,
        outro_motor: false,
        e_neutra: false,
        selected_count: 1,
    }
}

fn host(i: InspectorParallaxInfo) -> (MockPanelHost, InspectorState) {
    let h = MockPanelHost::with_panel::<InspectorPanel>();
    set_current_inspector_parallax(Some(i));
    (h, InspectorState::default())
}

/// Os dez números, com o valor que a fixtura completa tem.
const NUMEROS: [(ph2d_a11y::NodeId, f64); 10] = [
    (ids::INSP_PARALLAX_K_X, 0.35),
    (ids::INSP_PARALLAX_K_Y, 0.6),
    (ids::INSP_PARALLAX_TILE_X, 7.0),
    (ids::INSP_PARALLAX_TILE_Y, 3.0),
    (ids::INSP_PARALLAX_VEL_X, 0.35),
    (ids::INSP_PARALLAX_VEL_Y, -0.2),
    (ids::INSP_PARALLAX_MIN_X, -30.0),
    (ids::INSP_PARALLAX_MIN_Y, -12.0),
    (ids::INSP_PARALLAX_MAX_X, 30.0),
    (ids::INSP_PARALLAX_MAX_Y, 14.0),
];

fn pintados(h: &mut MockPanelHost, st: &mut InspectorState) -> Vec<ph2d_a11y::NodeId> {
    h.paint::<InspectorPanel>(st, VIEWPORT)
        .iter()
        .filter(|(_, r)| r.w > 0.0 && r.h > 0.0)
        .map(|(n, _)| *n)
        .collect()
}

/// ⭐⭐ **TODO campo é pintado com área clicável** — e, com só o `ScrollFactor`, SÓ os dois dele.
///
/// **Mutações que devem sangrar:** tirar uma fileira do pintor · pintar um bloco sem o componente.
#[test]
fn os_blocos_aparecem_so_com_o_componente_deles() {
    let (mut h, mut st) = host(camada(true));
    let r = pintados(&mut h, &mut st);
    for (id, _) in NUMEROS {
        assert!(
            r.contains(&id),
            "o campo {id:?} nao foi PINTADO com area clicavel"
        );
    }
    let (mut h, mut st) = host(camada(false));
    let r = pintados(&mut h, &mut st);
    for (i, (id, _)) in NUMEROS.iter().enumerate() {
        assert_eq!(
            r.contains(id),
            i < 2,
            "{id:?}: so' o factor aparece numa camada sem repeticao, deriva nem cerca"
        );
    }
    set_current_inspector_parallax(None);
}

/// ⭐⭐⭐ **TODO campo está VIVO SOB O DEDO** — o gesto REAL, e não um `WidgetEvent` sintético.
///
/// **Mutação que deve sangrar:** tirar qualquer id do `populate_parallax`.
#[test]
fn todo_campo_esta_vivo_sob_o_dedo() {
    for (id, _) in NUMEROS {
        let (mut h, mut st) = host(camada(true));
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
            "clicar no meio de {id:?} nao lhe deu o foco — falta o registo no `populate_parallax`, \
             e ele esta' MORTO SOB O DEDO"
        );
        set_current_inspector_parallax(None);
    }
}

/// ⭐⭐⭐ **OS CAMPOS MOSTRAM O QUE O OBJECTO TEM, e não os valores de fábrica.**
///
/// **Mutação que deve sangrar:** tirar a chamada do `sync_parallax` do `sync_sections`.
#[test]
fn os_campos_mostram_o_que_o_objecto_tem() {
    let (mut h, mut st) = host(camada(true));
    let _ = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    for (id, esperado) in NUMEROS {
        let lido = h
            .store()
            .number_value(id)
            .unwrap_or_else(|| panic!("o campo {id:?} nem sequer esta' registado"));
        assert!(
            (lido - esperado).abs() < 1.0e-5,
            "o campo {id:?} mostra {lido} e o objecto tem {esperado}"
        );
    }
    set_current_inspector_parallax(None);
}

/// ⭐⭐⭐ **O que se ESCREVE num campo chega ao BARRAMENTO, com a variante e o PAR certos** — o
/// outro eixo vem do INSTANTÂNEO, nunca de um zero.
///
/// **Mutações que devem sangrar:** tirar qualquer braço do `match` do `event_parallax` · trocar os
/// eixos de um par · ir buscar o outro eixo a um zero.
#[test]
fn escrever_num_campo_chega_ao_barramento_com_a_variante_e_o_par() {
    use ph2d_editor_core::action_bus::{ComponentEdit, EditorAction};
    use ph2d_editor_core::interaction::WidgetEvent;
    use ph2d_editor_core::panel::PanelHostInternal as _;
    use ph2d_editor_core::parallax_edits::ParallaxFieldEdit as E;

    const V: f32 = 1.25;
    let esperado = |id: ph2d_a11y::NodeId| -> E {
        match id {
            i if i == ids::INSP_PARALLAX_K_X => E::Factor([V, 0.6]),
            i if i == ids::INSP_PARALLAX_K_Y => E::Factor([0.35, V]),
            i if i == ids::INSP_PARALLAX_TILE_X => E::Repeat([V, 3.0]),
            i if i == ids::INSP_PARALLAX_TILE_Y => E::Repeat([7.0, V]),
            i if i == ids::INSP_PARALLAX_VEL_X => E::Motion([V, -0.2]),
            i if i == ids::INSP_PARALLAX_VEL_Y => E::Motion([0.35, V]),
            i if i == ids::INSP_PARALLAX_MIN_X => E::LimitsMin([V, -12.0]),
            i if i == ids::INSP_PARALLAX_MIN_Y => E::LimitsMin([-30.0, V]),
            i if i == ids::INSP_PARALLAX_MAX_X => E::LimitsMax([V, 14.0]),
            i if i == ids::INSP_PARALLAX_MAX_Y => E::LimitsMax([30.0, V]),
            _ => panic!("{id:?} nao esta' na tabela deste gate"),
        }
    };
    for (id, _) in NUMEROS {
        let (mut h, mut st) = host(camada(true));
        let _ = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
        h.store_mut().set_number_value(id, f64::from(V));
        let _ = h.apply_panel_event::<InspectorPanel>(&mut st, WidgetEvent::ValueChanged(id));
        let edits: Vec<_> = h
            .drained_actions()
            .into_iter()
            .filter_map(|a| match a {
                EditorAction::InspectorComponentEdit {
                    edit: ComponentEdit::Parallax(e),
                    ..
                } => Some(e),
                _ => None,
            })
            .collect();
        set_current_inspector_parallax(None);
        assert_eq!(edits, vec![esperado(id)], "{id:?}");
    }
}

/// ⭐⭐⭐ **A secção só existe para quem TEM o `ScrollFactor`** (ADR-0166).
#[test]
fn sem_o_componente_a_seccao_nao_e_pintada() {
    set_current_inspector_parallax(None);
    let mut h = MockPanelHost::with_panel::<InspectorPanel>();
    let mut st = InspectorState::default();
    let r = pintados(&mut h, &mut st);
    for (id, _) in NUMEROS {
        assert!(
            !r.contains(&id),
            "o campo {id:?} foi pintado num objecto SEM paralaxe"
        );
    }
}
