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
use ph2d_field::Bound;
use ph2d_panel_model3d::state::{Model3dPanelState, ModelSnapshot, ParamRow};
use ph2d_panel_model3d::{Model3dPanel, ModelIntent, drain_intents, publish, state};
use ph2d_ui_testkit::MockPanelHost;

/// ⚠️ **A entidade NÃO é o número da linha, de propósito.** A união é a entidade `77` e está na
/// **posição 0** da lista — é a separação que os gates abaixo medem: o id do controle vem da
/// posição (o `populate` cunha a família às cegas), e o intent tem de sair com a *entidade*.
const THE_UNION: u64 = 77;

fn scene_with_one_union() {
    publish(ModelSnapshot {
        modes: Vec::new(),
        frames: Vec::new(),
        adds: Vec::new(),
        ops: Vec::new(),
        acts: Vec::new(),
        rows: vec![ParamRow {
            entity: THE_UNION,
            param: ph2d_field::Param::Dim(0),
            key: "field.dim.round",
            value: 0.05,
            // Faixa de 0,4 — o número que o gate abaixo usa para distinguir a escala da linha de
            // uma escala fixa.
            bound: Bound::Soft(0.4),
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
    // O id vem da POSIÇÃO da linha (0), não da entidade (77).
    let slider = ids::model3d_radius_slider(0);

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
        vec![ModelIntent::SetParam {
            entity: THE_UNION,
            param: ph2d_field::Param::Dim(0),
            value: 0.2
        }],
        "meio curso de uma faixa de 0,4 é 0,2 (0,5 = a escala da LINHA não foi aplicada), e a \
         entidade tem de ser a 77 e não a posição 0"
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

/// ⭐ **N linhas ocupam N faixas distintas** — o gate do smoke *"o painel apresenta apenas um
/// slider"* (Enio, 2026-08-19).
///
/// # O mecanismo, para não voltar
///
/// `paint_slider_with_chip_layout_adaptive` devolve a **altura usada**; este arquivo devolve o **y
/// seguinte**. As duas convenções coexistem no repo, e `y = paint_row(...)` misturava-as: a segunda
/// linha ia parar em `y = 28` **absoluto** — dentro do título e fora do recorte — e as três
/// seguintes com ela. O painel mostrava UMA linha, e o artista concluía que o modelo tinha
/// encolhido para um cilindro.
///
/// ⚠️ O gate mede os **retângulos de hit** que a pintura regista, e não a imagem: é onde a diferença
/// entre "pintado" e "alcançável pelo rato" aparece, e as duas quebraram juntas.
#[test]
fn every_row_gets_its_own_band_none_stacked_on_another() {
    let nodes: Vec<ParamRow> = (0..4)
        .map(|n| ParamRow {
            entity: 100,
            param: ph2d_field::Param::Dim(n as u16),
            key: "field.dim.round",
            value: 0.05,
            bound: Bound::Hard(0.22),
        })
        .collect();
    publish(ModelSnapshot {
        modes: Vec::new(),
        frames: Vec::new(),
        adds: Vec::new(),
        ops: Vec::new(),
        acts: Vec::new(),
        rows: nodes,
        node_count: 4,
        last_trace_ms: 0.0,
    });

    let mut host = MockPanelHost::with_panel::<Model3dPanel>();
    host.set_panel_visible(Model3dPanel::ID, true);
    let mut panel_state = Model3dPanelState;
    let viewport = ph2d_editor_core::zones::Rect::new(0.0, 0.0, 1280.0, 800.0);
    let rects = host.paint::<Model3dPanel>(&mut panel_state, viewport);

    let mut tops: Vec<f32> = Vec::new();
    for n in 0..4u32 {
        let id = ids::model3d_radius_slider(n);
        let r = rects
            .iter()
            .find(|(rid, _)| *rid == id)
            .map(|(_, r)| *r)
            .unwrap_or_else(|| panic!("a linha {n} não registou o slider dela — ela foi pintada?"));
        tops.push(r.y);
    }
    for (i, pair) in tops.windows(2).enumerate() {
        assert!(
            pair[1] > pair[0] + 1.0,
            "as linhas {i} e {} estão na mesma faixa ({} e {}) — o avanço do y está a usar a ALTURA \
             como se fosse a posição",
            i + 1,
            pair[0],
            pair[1]
        );
    }

    // E todas caem DENTRO do painel, abaixo do título — uma linha em `y ≈ 28` absoluto é o sintoma
    // exato do bug, e ficaria recortada em vez de visível.
    let panel = rects
        .iter()
        .find(|(id, _)| *id == ids::MODEL3D_PANEL)
        .map(|(_, r)| *r);
    if let Some(panel) = panel {
        for (n, top) in tops.iter().enumerate() {
            assert!(
                *top > panel.y && *top < panel.y + panel.h,
                "a linha {n} caiu em y={top}, fora do corpo do painel ({}..{})",
                panel.y,
                panel.y + panel.h
            );
        }
    }
}

/// ⭐ **Clicar num verbo do seletor chega ao intent, com a POSIÇÃO dele.**
///
/// ⚠️ A posição, e não o nome: o painel não conhece o enum dos modos — ele vive no shell, com o
/// gizmo. Uma cópia dele aqui seria uma segunda contagem de verbos a envelhecer, e o dia em que o
/// shell ganhasse o quarto, o painel mostraria três.
#[test]
fn clicking_a_verb_reaches_the_gizmo_intent() {
    let _ = drain_intents();
    publish(ModelSnapshot {
        modes: vec![
            state::ModeChip {
                key: "panel.model3d.mode.move",
                active: true,
            },
            state::ModeChip {
                key: "panel.model3d.mode.rotate",
                active: false,
            },
            state::ModeChip {
                key: "panel.model3d.mode.scale",
                active: false,
            },
        ],
        frames: Vec::new(),
        adds: Vec::new(),
        ops: Vec::new(),
        acts: Vec::new(),
        rows: Vec::new(),
        node_count: 0,
        last_trace_ms: 0.0,
    });

    let mut host = MockPanelHost::with_panel::<Model3dPanel>();
    let mut panel_state = Model3dPanelState;
    let outcome = host.apply_panel_event::<Model3dPanel>(
        &mut panel_state,
        WidgetEvent::Click(ids::model3d_mode_button(1)),
    );
    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "o painel ignorou um clique REAL num verbo — falta o braço em `event.rs` ou o id saiu da \
         família"
    );
    assert_eq!(drain_intents(), vec![ModelIntent::SetGizmoMode { slot: 1 }]);
}

/// ⚠️ **Um slot da família SEM verbo no retrato não despacha nada.**
///
/// A família tem `MAX_MODES` ids e o shell publica três. Sem esta guarda, um clique num slot vazio
/// (que nada pintou) mandaria o shell trocar para um modo que não existe.
#[test]
fn a_verb_slot_with_no_verb_behind_it_does_nothing() {
    let _ = drain_intents();
    publish(ModelSnapshot {
        modes: vec![state::ModeChip {
            key: "panel.model3d.mode.move",
            active: true,
        }],
        frames: Vec::new(),
        adds: Vec::new(),
        ops: Vec::new(),
        acts: Vec::new(),
        rows: Vec::new(),
        node_count: 0,
        last_trace_ms: 0.0,
    });

    let mut host = MockPanelHost::with_panel::<Model3dPanel>();
    let mut panel_state = Model3dPanelState;
    let outcome = host.apply_panel_event::<Model3dPanel>(
        &mut panel_state,
        WidgetEvent::Click(ids::model3d_mode_button(5)),
    );
    assert_eq!(outcome, EventOutcome::Ignored);
    assert!(drain_intents().is_empty());
}

/// ⭐ **O seletor de EIXOS é uma família de ids própria**, e o clique nele não dispara o verbo.
///
/// ⚠️ Os dois seletores coexistem no painel. Partilhar a família faria um clique em «Local»
/// disparar o verbo da mesma posição — e o sintoma seria *"trocar de eixos troca a ferramenta"*,
/// que ninguém liga a ids partilhados.
#[test]
fn the_axis_selector_is_its_own_family() {
    let _ = drain_intents();
    publish(ModelSnapshot {
        modes: vec![
            state::ModeChip {
                key: "panel.model3d.mode.move",
                active: true,
            },
            state::ModeChip {
                key: "panel.model3d.mode.rotate",
                active: false,
            },
        ],
        frames: vec![
            state::ModeChip {
                key: "panel.model3d.frame.global",
                active: true,
            },
            state::ModeChip {
                key: "panel.model3d.frame.local",
                active: false,
            },
        ],
        adds: Vec::new(),
        ops: Vec::new(),
        acts: Vec::new(),
        rows: Vec::new(),
        node_count: 0,
        last_trace_ms: 0.0,
    });

    let mut host = MockPanelHost::with_panel::<Model3dPanel>();
    let mut panel_state = Model3dPanelState;
    let outcome = host.apply_panel_event::<Model3dPanel>(
        &mut panel_state,
        WidgetEvent::Click(ids::model3d_frame_button(1)),
    );
    assert_eq!(outcome, EventOutcome::Consumed);
    assert_eq!(
        drain_intents(),
        vec![ModelIntent::SetGizmoFrame { slot: 1 }],
        "o clique no eixo tem de pedir o EIXO — se vier `SetGizmoMode`, as famílias colidiram"
    );
}

/// ⭐ **Os cinco seletores são cinco famílias**, e um clique em cada chega ao intent certo.
///
/// ⚠️ São verbo · eixos · criar · combinar · agir, todos no mesmo painel. Uma família partilhada
/// faria um clique em «+ Sphere» pedir a segunda operação booleana — o tipo de defeito que se lê
/// como *"o botão faz outra coisa"* e ninguém liga a ids.
#[test]
fn the_selectors_never_answer_for_each_other() {
    let chip = |key: &'static str| state::ModeChip { key, active: false };
    let _ = drain_intents();
    publish(ModelSnapshot {
        modes: vec![
            chip("panel.model3d.mode.move"),
            chip("panel.model3d.mode.rotate"),
        ],
        frames: vec![
            chip("panel.model3d.frame.global"),
            chip("panel.model3d.frame.local"),
        ],
        adds: vec![
            chip("panel.model3d.add.box"),
            chip("panel.model3d.add.sphere"),
        ],
        ops: vec![
            chip("panel.model3d.op.union"),
            chip("panel.model3d.op.subtract"),
        ],
        acts: vec![
            chip("panel.model3d.act.duplicate"),
            chip("panel.model3d.act.delete"),
        ],
        rows: Vec::new(),
        node_count: 0,
        last_trace_ms: 0.0,
    });

    let mut host = MockPanelHost::with_panel::<Model3dPanel>();
    let mut panel_state = Model3dPanelState;
    let mut click =
        |id| host.apply_panel_event::<Model3dPanel>(&mut panel_state, WidgetEvent::Click(id));
    assert_eq!(click(ids::model3d_mode_button(1)), EventOutcome::Consumed);
    assert_eq!(click(ids::model3d_frame_button(1)), EventOutcome::Consumed);
    assert_eq!(click(ids::model3d_add_button(1)), EventOutcome::Consumed);
    assert_eq!(click(ids::model3d_op_button(1)), EventOutcome::Consumed);
    assert_eq!(click(ids::model3d_act_button(1)), EventOutcome::Consumed);

    assert_eq!(
        drain_intents(),
        vec![
            ModelIntent::SetGizmoMode { slot: 1 },
            ModelIntent::SetGizmoFrame { slot: 1 },
            ModelIntent::AddShape { slot: 1 },
            ModelIntent::ApplyOp { slot: 1 },
            ModelIntent::Act { slot: 1 },
        ],
        "cada família tem de responder por si — se dois intents forem iguais, os ids colidiram"
    );
}

/// ⚠️ **A fileira de operações só é pintada quando pode agir.** Quando o retrato a publica vazia,
/// um clique num id dela não pede nada — um controle que aparece e não faz nada é pior do que um
/// que não aparece.
#[test]
fn an_empty_operation_row_dispatches_nothing() {
    let _ = drain_intents();
    scene_with_one_union();
    let mut host = MockPanelHost::with_panel::<Model3dPanel>();
    let mut panel_state = Model3dPanelState;
    let outcome = host.apply_panel_event::<Model3dPanel>(
        &mut panel_state,
        WidgetEvent::Click(ids::model3d_op_button(0)),
    );
    assert_eq!(outcome, EventOutcome::Ignored);
    assert!(drain_intents().is_empty());
}
