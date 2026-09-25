//! ⭐⭐⭐ **A secção LIVE MESH (o catavento) é PINTADA, está VIVA sob o dedo, mostra os números DO
//! OBJECTO e a escrita chega ao barramento** (`docs/3D/02.2`, rota B).
//!
//! Irmã do [`super::a_seccao_parallax_esta_viva`], no mesmo molde. ⚠️ Nasceu na integração de
//! 2026-09-25 (rodada 03), pelo censo `toda_seccao_viva_chega_a_pixel`: a `line/3DModeling` trouxe
//! a secção e o semeador `set_current_inspector_mesh3d` e nenhum gate a pintava — *a presença era
//! gateada, a pintura não*. A cura que o censo prescreve é escrever o gate, nunca uma linha na
//! catraca `SEM_GATE_QUE_PINTA`.

use ph2d_editor_core::mesh3d_edits::InspectorMesh3dInfo;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_inspector::{InspectorPanel, InspectorState, ids, set_current_inspector_mesh3d};
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 2400.0,
};

/// ⭐ Um catavento que **NÃO** está no valor de fábrica em nenhum dos três campos — *um corpus no
/// NEUTRO de um knob não testa esse knob* (um `Mesh3D` nasce parado, a olhar de frente).
fn catavento() -> InspectorMesh3dInfo {
    InspectorMesh3dInfo {
        entity_bits: 0x00C0_FFEE,
        yaw: 30.0_f32.to_radians(),
        pitch: (-15.0_f32).to_radians(),
        spin: 0.125,
        assado: true,
    }
}

fn host(i: InspectorMesh3dInfo) -> (MockPanelHost, InspectorState) {
    let h = MockPanelHost::with_panel::<InspectorPanel>();
    set_current_inspector_mesh3d(Some(i));
    (h, InspectorState::default())
}

/// Os três números, com o valor que a fixtura tem — em GRAUS, que é como o artista os lê.
const NUMEROS: [(ph2d_a11y::NodeId, f64); 3] = [
    (ids::INSP_MESH3D_YAW, 30.0),
    (ids::INSP_MESH3D_PITCH, -15.0),
    (ids::INSP_MESH3D_SPIN, 0.125),
];

/// ⭐⭐ **TODO campo é pintado com área clicável.**
///
/// **Mutação que deve sangrar:** tirar uma fileira do pintor da secção.
#[test]
fn todo_campo_e_pintado() {
    let (mut h, mut st) = host(catavento());
    let r: Vec<_> = h
        .paint::<InspectorPanel>(&mut st, VIEWPORT)
        .iter()
        .filter(|(_, r)| r.w > 0.0 && r.h > 0.0)
        .map(|(n, _)| *n)
        .collect();
    for (id, _) in NUMEROS {
        assert!(
            r.contains(&id),
            "o campo {id:?} nao foi PINTADO com area clicavel"
        );
    }
    set_current_inspector_mesh3d(None);
}

/// ⭐⭐⭐ **TODO campo está VIVO SOB O DEDO** — o gesto REAL, e não um `WidgetEvent` sintético.
///
/// **Mutação que deve sangrar:** tirar qualquer id do `populate_mesh3d`.
#[test]
fn todo_campo_esta_vivo_sob_o_dedo() {
    for (id, _) in NUMEROS {
        let (mut h, mut st) = host(catavento());
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
            "clicar no meio de {id:?} nao lhe deu o foco — falta o registo no `populate_mesh3d`, e \
             ele esta' MORTO SOB O DEDO"
        );
        set_current_inspector_mesh3d(None);
    }
}

/// ⭐⭐⭐ **OS CAMPOS MOSTRAM O QUE O OBJECTO TEM, e não os valores de fábrica.**
///
/// **Mutação que deve sangrar:** tirar a chamada do `sync_mesh3d` do `sync_sections`.
#[test]
fn os_campos_mostram_o_que_o_objecto_tem() {
    let (mut h, mut st) = host(catavento());
    let _ = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    for (id, esperado) in NUMEROS {
        let lido = h
            .store()
            .number_value(id)
            .unwrap_or_else(|| panic!("o campo {id:?} nem sequer esta' registado"));
        assert!(
            (lido - esperado).abs() < 1.0e-4,
            "o campo {id:?} mostra {lido} e o objecto tem {esperado}"
        );
    }
    set_current_inspector_mesh3d(None);
}

/// ⭐⭐⭐ **O que se ESCREVE num campo chega ao BARRAMENTO, com a variante certa.**
///
/// **Mutações que devem sangrar:** trocar dois braços do `match` do `event_mesh3d` · apagar um.
#[test]
fn escrever_num_campo_chega_ao_barramento_com_a_variante_certa() {
    use ph2d_editor_core::action_bus::{ComponentEdit, EditorAction};
    use ph2d_editor_core::interaction::WidgetEvent;
    use ph2d_editor_core::mesh3d_edits::Mesh3dFieldEdit as E;
    use ph2d_editor_core::panel::PanelHostInternal as _;

    const V: f64 = 1.25;
    let esperado = |id: ph2d_a11y::NodeId| -> E {
        match id {
            i if i == ids::INSP_MESH3D_YAW => E::YawGraus(V),
            i if i == ids::INSP_MESH3D_PITCH => E::PitchGraus(V),
            i if i == ids::INSP_MESH3D_SPIN => E::Spin(V),
            _ => panic!("{id:?} nao esta' na tabela deste gate"),
        }
    };
    for (id, _) in NUMEROS {
        let (mut h, mut st) = host(catavento());
        let _ = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
        h.store_mut().set_number_value(id, V);
        let _ = h.apply_panel_event::<InspectorPanel>(&mut st, WidgetEvent::ValueChanged(id));
        let edits: Vec<_> = h
            .drained_actions()
            .into_iter()
            .filter_map(|a| match a {
                EditorAction::InspectorComponentEdit {
                    edit: ComponentEdit::Mesh3d(e),
                    ..
                } => Some(e),
                _ => None,
            })
            .collect();
        assert_eq!(
            edits,
            vec![esperado(id)],
            "escrever em {id:?} nao chegou ao barramento como a variante certa"
        );
        set_current_inspector_mesh3d(None);
    }
}
