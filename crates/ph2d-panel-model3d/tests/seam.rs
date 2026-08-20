//! Costura COMPORTAMENTAL do painel de modelagem 3D — o caminho inteiro, sem app.
//!
//! ⚠️ **Este arquivo existe porque um gate de arquitetura o exigiu, e ele estava certo.** Os testes
//! dentro da crate empurravam intents à mão: mediam a **fila**, não o `apply_event`. Um braço em
//! falta em `event.rs`, um id fora da família ou uma leitura errada do store deixariam o controle
//! pintado, arrastável e **silenciosamente morto**, com todos aqueles testes verdes.
//!
//! Aqui corre-se o que o shell corre: `populate` → escrever a trilha do slider → `apply_event` →
//! drenar o intent → afirmar o número que saiu.

use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::{EventOutcome, Panel, PanelHostInternal};
use ph2d_field::RadiusBound;
use ph2d_panel_model3d::state::{Model3dPanelState, ModelSnapshot, RadiusRow};
use ph2d_panel_model3d::{Model3dPanel, ModelIntent, drain_intents, publish, state};
use ph2d_ui_testkit::MockPanelHost;

fn scene_with_one_union() {
    publish(ModelSnapshot {
        rows: vec![RadiusRow {
            node: 3,
            kind_key: "panel.model3d.kind.union",
            radius: 0.05,
            // Faixa de 0,4 — o número que o gate abaixo usa para distinguir a escala da linha de
            // uma escala fixa.
            bound: RadiusBound::Soft(0.4),
        }],
        node_count: 4,
        last_trace_ms: 9.0,
    });
}

/// ⭐ **Arrastar o slider de um raio chega ao intent, com o valor da faixa DAQUELA linha.**
///
/// É a prova de ponta a ponta da promessa do módulo: *o raio fica editável*. E o número escolhido
/// separa as duas hipóteses — meio curso de uma faixa de 0,4 é **0,2**; se saísse 0,5, a ligação
/// 0..1 que o `populate` instala teria escapado para o valor.
#[test]
fn dragging_a_radius_slider_reaches_the_document_intent() {
    let _ = drain_intents();
    scene_with_one_union();

    let mut host = MockPanelHost::with_panel::<Model3dPanel>();
    let mut panel_state = Model3dPanelState;
    let slider = ids::model3d_radius_slider(3);

    host.set_slider_value(slider, 0.5);
    let outcome =
        host.apply_panel_event::<Model3dPanel>(&mut panel_state, WidgetEvent::ValueChanged(slider));

    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "o painel ignorou uma edição REAL de slider — falta o braço em `event.rs` ou o id saiu da \
         família"
    );
    let intents = drain_intents();
    assert_eq!(
        intents,
        vec![ModelIntent::SetRadius {
            node: 3,
            radius: 0.2
        }],
        "meio curso de uma faixa de 0,4 é 0,2; 0,5 significa que a escala da LINHA não foi aplicada"
    );
}

/// ⚠️ **O campo numérico não notifica duas vezes.**
///
/// Ele está ligado ao slider, então uma digitação espelha-se nele e o slider dispara o seu próprio
/// `ValueChanged`. Se este braço também emitisse, uma edição viraria duas — e a segunda chegaria
/// com o valor da primeira, o que se lê como *"o número volta atrás sozinho"*.
#[test]
fn the_number_field_does_not_emit_a_second_time() {
    let _ = drain_intents();
    scene_with_one_union();

    let mut host = MockPanelHost::with_panel::<Model3dPanel>();
    let mut panel_state = Model3dPanelState;
    let chip = ids::model3d_radius_chip(3);

    let outcome =
        host.apply_panel_event::<Model3dPanel>(&mut panel_state, WidgetEvent::ValueChanged(chip));
    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "o evento é do painel — engoli-lo é diferente de ignorá-lo"
    );
    assert!(
        drain_intents().is_empty(),
        "o campo ligado ao slider não pode emitir a sua própria edição"
    );
}

/// **O X fecha o painel** — a porta que o abre e a que o fecha têm de concordar.
#[test]
fn the_close_button_hides_the_panel() {
    let mut host = MockPanelHost::with_panel::<Model3dPanel>();
    let mut panel_state = Model3dPanelState;
    host.set_panel_visible(Model3dPanel::ID, true);

    let outcome = host.apply_panel_event::<Model3dPanel>(
        &mut panel_state,
        WidgetEvent::Click(ids::MODEL3D_CLOSE),
    );
    assert_eq!(outcome, EventOutcome::Consumed);
    assert!(
        !host.panel_visible(Model3dPanel::ID),
        "o X tem de esconder o painel"
    );
}

/// ⚠️ **Um id que não é deste painel é IGNORADO, não engolido.**
///
/// `Ignored` é o que deixa o evento seguir para quem o quer. Um painel que consumisse tudo o que
/// lhe chega mataria em silêncio os controles do painel de baixo — e o sintoma seria "aquele
/// slider parou de funcionar quando abri o painel 3D".
#[test]
fn an_id_from_another_panel_is_ignored_not_swallowed() {
    let _ = drain_intents();
    scene_with_one_union();
    let mut host = MockPanelHost::with_panel::<Model3dPanel>();
    let mut panel_state = Model3dPanelState;
    let outcome = host.apply_panel_event::<Model3dPanel>(
        &mut panel_state,
        WidgetEvent::ValueChanged(ids::INSP_DRAG_HANDLE),
    );
    assert_eq!(outcome, EventOutcome::Ignored);
    assert!(drain_intents().is_empty());
    let _ = state::current();
}

/// ⚠️ **Um id da família SEM linha no retrato não inventa um nó.**
///
/// O documento pode encolher entre o quadro pintado e o evento que ele gerou. Emitir uma edição
/// para um nó que já não existe seria escrever num índice que passou a ser outra coisa — e um
/// índice fora do fim seria pior ainda.
#[test]
fn a_family_id_without_a_row_does_not_invent_a_node() {
    let _ = drain_intents();
    // Um retrato VAZIO: a cena foi fechada entre o quadro e o evento.
    publish(ModelSnapshot::default());

    let mut host = MockPanelHost::with_panel::<Model3dPanel>();
    let mut panel_state = Model3dPanelState;
    let slider = ids::model3d_radius_slider(7);
    host.set_slider_value(slider, 0.9);
    let outcome =
        host.apply_panel_event::<Model3dPanel>(&mut panel_state, WidgetEvent::ValueChanged(slider));

    assert_eq!(
        outcome,
        EventOutcome::Ignored,
        "sem linha, o evento não é deste painel"
    );
    assert!(
        drain_intents().is_empty(),
        "um nó que não está no retrato não pode receber edição"
    );
}
