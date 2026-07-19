//! Behavioral SEAM test for the Flip panel ↔ tool (blindagem Fase 1.2).
//!
//! Unit tests in `ph2d-tool-flip` exercise `handle_panel_event` directly, and
//! `populate.rs` registers widgets — but NEITHER proves the wire between them is
//! intact. A forgotten `event.rs` arm or a wrong id would leave a slider painted,
//! draggable and SILENTLY DEAD while every unit test + contract gate stays green.
//!
//! These run the full path the desktop shell runs, headless:
//!   populate → set value / click → apply_event → bus → handle_panel_event
//!   → assert the tool's state actually changed.

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids::FlipLayerWidget;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::EventOutcome;
use ph2d_editor_core::tool::Tool; // brings `handle_panel_event` into scope
use ph2d_panel_flip::state::FlipPanelState;
use ph2d_panel_flip::{FlipLayerRow, FlipLayersSnapshot, FlipPanel, LayerRename, ids};
use ph2d_tool_flip::{FlipMode, FlipTool, WIDTH_MAX_PX};
use ph2d_ui_testkit::MockPanelHost;

/// Drag the Size slider to its full end and prove the width reaches the tool —
/// exercising every site from `populate` to `width_px()`.
#[test]
fn size_slider_drag_reaches_tool() {
    let mut host = MockPanelHost::with_panel::<FlipPanel>();
    let mut panel_state = FlipPanelState::default();
    let mut tool = FlipTool::default();

    host.set_slider_value(ids::FLIP_SIZE, 1.0);
    let outcome = host.apply_panel_event::<FlipPanel>(
        &mut panel_state,
        WidgetEvent::ValueChanged(ids::FLIP_SIZE),
    );
    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "panel ignored a real slider edit — `event.rs` arm for FLIP_SIZE is missing"
    );

    let mut forwarded = false;
    for action in host.drained_actions() {
        if let EditorAction::ToolPanelEvent(pe) = action {
            tool.handle_panel_event(pe);
            forwarded = true;
        }
    }
    assert!(
        forwarded,
        "slider edit never reached the bus as a ToolPanelEvent — the seam is dead"
    );
    assert_eq!(
        tool.width_px(),
        WIDTH_MAX_PX,
        "slider→tool seam delivered the wrong px for Size"
    );
}

/// Clicking the Draw mode button must switch the tool's canvas mode through the
/// seam (Select → Draw).
#[test]
fn draw_mode_button_switches_the_tool_mode() {
    let mut host = MockPanelHost::with_panel::<FlipPanel>();
    let mut panel_state = FlipPanelState::default();
    let mut tool = FlipTool::default();
    assert_eq!(tool.mode(), FlipMode::Select, "fresh tool starts in Select");

    let outcome = host
        .apply_panel_event::<FlipPanel>(&mut panel_state, WidgetEvent::Click(ids::FLIP_MODE_DRAW));
    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "Draw button click ignored — `event.rs` arm for FLIP_MODE_DRAW is missing"
    );

    for action in host.drained_actions() {
        if let EditorAction::ToolPanelEvent(pe) = action {
            tool.handle_panel_event(pe);
        }
    }
    assert_eq!(
        tool.mode(),
        FlipMode::Draw,
        "mode button never switched the tool mode through the seam"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// PAINTED ≠ POPULATED (a auditoria do "não existe botão fill")
//
// Tudo acima prova que o clique CHEGA na tool. Nada acima prova que o botão está
// NA TELA. O gate `architecture_panel_wiring_parity` lê o TEXTO-FONTE; os seams
// leem o barramento. Nenhum dos dois roda `paint`. Então um widget pode estar
// registrado, wirado, unit-testado e contract-limpo enquanto a chamada de pintura
// dele mora atrás de um `return` — e o relato do usuário é "o botão não existe",
// com todos os gates verdes.
//
// O que o usuário pode clicar é o que a PINTURA registrou. É isso que se lê aqui.
// ─────────────────────────────────────────────────────────────────────────────

/// Um viewport de desktop plausível — o painel ancora no dock direito do layout.
fn viewport() -> ph2d_editor_core::zones::Rect {
    ph2d_editor_core::zones::Rect::new(0.0, 0.0, 1600.0, 900.0)
}

/// **Todo botão de MODO é pintado e clicável** — inclusive o Fill (W4).
///
/// Mutação que sangra: apague a entrada do Fill do `mode_row` e este teste fica
/// vermelho, enquanto TODOS os outros gates do projeto seguem verdes.
#[test]
fn every_mode_button_is_painted_and_clickable() {
    let mut host = MockPanelHost::with_panel::<FlipPanel>();
    let mut st = FlipPanelState::default();
    let painted = host.paint::<FlipPanel>(&mut st, viewport());

    for (id, name) in [
        (ids::FLIP_MODE_SELECT, "Select"),
        (ids::FLIP_MODE_DRAW, "Draw"),
        (ids::FLIP_MODE_ERASE, "Erase"),
        (ids::FLIP_MODE_FILL, "Fill"),
        (ids::FLIP_MODE_RESHAPE, "Sculpt"),
        (ids::FLIP_MODE_EDIT, "Edit"),
    ] {
        let hit = painted.iter().find(|(w, _)| *w == id);
        let Some((_, r)) = hit else {
            panic!("o botao de modo {name} NAO e pintado: nao existe na tela");
        };
        assert!(
            r.w > 0.0 && r.h > 0.0,
            "o botao de modo {name} foi pintado com area ZERO: invisivel e inclicavel ({r:?})"
        );
    }
}

/// A seção do balde é **modal**: só aparece no modo Fill. Fora dele, os widgets do
/// balde não podem estar clicáveis (um widget clicável e invisível é uma armadilha).
///
/// O snapshot vive num global (o shell publica; o painel lê), então o teste o
/// escreve como o `flip_bridge` escreve.
#[test]
fn the_bucket_widgets_appear_only_in_fill_mode() {
    let mut host = MockPanelHost::with_panel::<FlipPanel>();
    let mut st = FlipPanelState::default();
    // O **swatch de Fill NÃO está aqui**: ele é compartilhado com o modo Edit (W6), onde
    // recolore o miolo do traço selecionado — a cor da linha e a do miolo são dois
    // atributos, e fundi-las num controle só foi o defeito que o smoke derrubou. O que é
    // exclusivo do balde são os MODOS dele e os três knobs.
    let bucket = [
        ids::FLIP_FILL_PAINT,
        ids::FLIP_FILL_BEHIND,
        ids::FLIP_FILL_UNPAINT,
        ids::FLIP_GAP,
        ids::FLIP_GROW,
        ids::FLIP_TRAP,
        ids::FLIP_PRECISION,
    ];

    // Modo Draw: o balde não existe na tela.
    let mut snap = ph2d_tool_flip::FlipStyleSnapshot {
        mode: FlipMode::Draw,
        ..Default::default()
    };
    ph2d_panel_flip::set_current_flip_style(Some(snap));
    let painted = host.paint::<FlipPanel>(&mut st, viewport());
    for id in bucket {
        assert!(
            !painted.iter().any(|(w, _)| *w == id),
            "widget do balde {id:?} pintado FORA do modo Fill"
        );
    }

    // Modo Fill: todos existem, com área.
    snap.mode = FlipMode::Fill;
    ph2d_panel_flip::set_current_flip_style(Some(snap));
    let painted = host.paint::<FlipPanel>(&mut st, viewport());
    for id in bucket {
        let hit = painted
            .iter()
            .find(|(w, r)| *w == id && r.w > 0.0 && r.h > 0.0);
        assert!(
            hit.is_some(),
            "widget do balde {id:?} NAO e pintado no modo Fill: a secao esta morta"
        );
    }
}

/// A seção do Colorize (C2) é **modal**: swatch + Apply + Clear só no modo Colorize.
/// Fora dele, clicáveis e invisíveis seriam a armadilha que a doutrina proíbe — o par
/// ausência (Draw) + presença-com-área (Colorize) que a DIRETIVA pede.
#[test]
fn the_colorize_widgets_appear_only_in_colorize_mode() {
    let mut host = MockPanelHost::with_panel::<FlipPanel>();
    let mut st = FlipPanelState::default();
    let colorize = [
        ids::FLIP_COLORIZE_SWATCH,
        ids::FLIP_COLORIZE_APPLY,
        ids::FLIP_COLORIZE_CLEAR,
    ];

    // Modo Draw: o Colorize não existe na tela.
    let mut snap = ph2d_tool_flip::FlipStyleSnapshot {
        mode: FlipMode::Draw,
        ..Default::default()
    };
    ph2d_panel_flip::set_current_flip_style(Some(snap));
    let painted = host.paint::<FlipPanel>(&mut st, viewport());
    for id in colorize {
        assert!(
            !painted.iter().any(|(w, _)| *w == id),
            "widget do Colorize {id:?} pintado FORA do modo Colorize"
        );
    }

    // Modo Colorize: todos existem, com área.
    snap.mode = FlipMode::Colorize;
    ph2d_panel_flip::set_current_flip_style(Some(snap));
    let painted = host.paint::<FlipPanel>(&mut st, viewport());
    for id in colorize {
        let hit = painted
            .iter()
            .find(|(w, r)| *w == id && r.w > 0.0 && r.h > 0.0);
        assert!(
            hit.is_some(),
            "widget do Colorize {id:?} NAO e pintado no modo Colorize: a secao esta morta"
        );
    }
}

/// **Toda caixa numérica registra o seu RANGE.**
///
/// Sem `set_number_range`, a caixa continua pintando, aceitando digitação e passando no
/// gate de wiring — mas o ARRASTO deriva o passo do texto do buffer e anda ~50 unidades
/// por pixel: um pixel de gesto vai do mínimo ao máximo. O widget parece vivo e é
/// inutilizável. Nenhum teste via isso, porque todos digitavam o valor.
///
/// Mutação que sangra: tire o `set_number_range` do `slider_chip` e este teste cai.
#[test]
fn every_number_box_has_a_registered_range() {
    use ph2d_editor_core::interaction::InteractiveState;
    let host = MockPanelHost::with_panel::<FlipPanel>();
    let store = host.store();

    let boxes = [
        ("Size", ids::FLIP_SIZE_NUM),
        ("Hardness", ids::FLIP_HARDNESS_NUM),
        ("Opacity", ids::FLIP_OPACITY_NUM),
        ("Smoothing", ids::FLIP_SMOOTHING_NUM),
        ("Gap", ids::FLIP_GAP_NUM),
        ("Trap", ids::FLIP_TRAP_NUM),
        ("Grow", ids::FLIP_GROW_NUM),
        ("Precision", ids::FLIP_PRECISION_NUM),
        // §4.C — as caixas dos valores PRÓPRIOS da borracha (link desligado).
        ("Eraser size", ids::FLIP_ERASE_SIZE_NUM),
        ("Eraser strength", ids::FLIP_ERASE_STRENGTH_NUM),
    ];
    for (name, id) in boxes {
        assert!(
            matches!(store.get(id), Some(InteractiveState::NumberInput { .. })),
            "a caixa {name} nem esta registrada"
        );
        let range = store.number_range(id);
        let Some((min, max, step)) = range else {
            panic!(
                "a caixa {name} nao registrou range: o arrasto vai andar ~50 unidades por pixel"
            );
        };
        assert!(max > min, "range invertido em {name}: [{min}, {max}]");
        assert!(step > 0.0, "step nao-positivo em {name}: {step}");
    }
}

/// **O painel de cada ferramenta não exibe os atributos das outras** (Enio 2026-07-12).
///
/// Um controle que não faz nada é pior que a ausência dele: o usuário mexe, nada muda, e
/// conclui que o app está quebrado. O Hardness do pincel no modo balde era exatamente isso.
///
/// A tabela é a especificação viva. Mutação que sangra: pinte o Brush no modo Fill e este
/// teste cai.
#[test]
fn each_mode_shows_only_its_own_attributes() {
    let stroke_only = [
        ("Hardness", ids::FLIP_HARDNESS),
        ("Smoothing", ids::FLIP_SMOOTHING),
        ("Stroke color", ids::FLIP_STROKE_SWATCH),
    ];
    let eraser_only = [
        ("Erase soft", ids::FLIP_ERASE_SOFT),
        ("Erase hard", ids::FLIP_ERASE_HARD),
        ("Erase stroke", ids::FLIP_ERASE_STROKE),
    ];
    // Os knobs do balde — exclusivos DELE. (O swatch de Fill saiu daqui: ele é
    // compartilhado com o Edit, ver `fill_swatch`.)
    let bucket_only = [
        ("Gap", ids::FLIP_GAP),
        ("Trap", ids::FLIP_TRAP),
        ("Grow", ids::FLIP_GROW),
        ("Precision", ids::FLIP_PRECISION),
    ];
    // A cor do MIOLO: do balde (que a deposita) e do Edit (que a reescreve na seleção).
    let fill_swatch = [("Fill color", ids::FLIP_FILL_SWATCH)];
    let bucket_expected = [
        ("Fill color", ids::FLIP_FILL_SWATCH),
        ("Gap", ids::FLIP_GAP),
        ("Trap", ids::FLIP_TRAP),
        ("Grow", ids::FLIP_GROW),
        ("Precision", ids::FLIP_PRECISION),
    ];
    // W6 — o Edit Mode. As ops de seleção são SÓ dele; e o Smoothing é o único
    // atributo do Brush que o Edit NÃO pode mostrar (é uma op de GEOMETRIA sobre as
    // amostras cruas da caneta, que um traço já desenhado não guarda — um slider que não
    // pode agir é o controle morto que a doutrina modal proíbe).
    let edit_only = [
        ("Select all", ids::FLIP_EDIT_SELECT_ALL),
        ("Deselect", ids::FLIP_EDIT_DESELECT),
        ("Delete selection", ids::FLIP_EDIT_DELETE),
    ];
    let smoothing_only = [("Smoothing", ids::FLIP_SMOOTHING)];
    // O que o Edit mostra ALÉM das ops: os atributos do traço que ele reescreve na
    // seleção (a cor e a dureza; o Size e a Opacity são compartilhados com outros modos).
    let edit_expected = [
        ("Hardness", ids::FLIP_HARDNESS),
        ("Stroke color", ids::FLIP_STROKE_SWATCH),
        // A cor do MIOLO do traço selecionado — atributo À PARTE da cor da linha (o smoke
        // do Enio derrubou o 1º corte, em que o swatch do traço recoloria os dois).
        ("Fill color", ids::FLIP_FILL_SWATCH),
        ("Select all", ids::FLIP_EDIT_SELECT_ALL),
        ("Deselect", ids::FLIP_EDIT_DESELECT),
        ("Delete selection", ids::FLIP_EDIT_DELETE),
    ];
    // Os oito pincéis de escultura (W5) — atributo SÓ do modo Sculpt.
    let sculpt_only = [
        ("Smooth", ids::FLIP_RS_SMOOTH),
        ("Push", ids::FLIP_RS_PUSH),
        ("Grab", ids::FLIP_RS_GRAB),
        ("Pinch", ids::FLIP_RS_PINCH),
        ("Twist", ids::FLIP_RS_TWIST),
        ("Thickness", ids::FLIP_RS_THICKNESS),
        ("Strength", ids::FLIP_RS_STRENGTH),
        ("Randomize", ids::FLIP_RS_RANDOMIZE),
    ];
    // Colorize (C2): a cor do rabisco + as ações do gesto — atributos SÓ do modo Colorize.
    let colorize_only = [
        ("Colorize color", ids::FLIP_COLORIZE_SWATCH),
        ("Colorize apply", ids::FLIP_COLORIZE_APPLY),
        ("Colorize clear", ids::FLIP_COLORIZE_CLEAR),
    ];

    // (modo, o que TEM de aparecer, o que NÃO pode aparecer)
    #[allow(clippy::type_complexity)] // (modo, o que aparece, o que NAO pode aparecer)
    let cases: [(
        FlipMode,
        &[(&str, ph2d_a11y::NodeId)],
        &[&[(&str, ph2d_a11y::NodeId)]],
    ); 7] = [
        (
            FlipMode::Draw,
            &stroke_only,
            &[
                &eraser_only,
                &bucket_only,
                &fill_swatch,
                &sculpt_only,
                &edit_only,
            ],
        ),
        (
            FlipMode::Erase,
            &eraser_only,
            &[
                &stroke_only,
                &bucket_only,
                &fill_swatch,
                &sculpt_only,
                &edit_only,
            ],
        ),
        (
            FlipMode::Fill,
            &bucket_expected,
            &[&stroke_only, &eraser_only, &sculpt_only, &edit_only],
        ),
        // Sculpt: os oito pincéis — e nada de dureza/alisamento/cor/balde.
        (
            FlipMode::Reshape,
            &sculpt_only,
            &[
                &stroke_only,
                &eraser_only,
                &bucket_only,
                &fill_swatch,
                &edit_only,
            ],
        ),
        // Select move/gira o objeto: não tem atributo de pintura nenhum.
        (
            FlipMode::Select,
            &[],
            &[
                &stroke_only,
                &eraser_only,
                &bucket_only,
                &fill_swatch,
                &sculpt_only,
                &edit_only,
            ],
        ),
        // Edit (W6): as ops de seleção + os atributos do traço que ele reescreve (as DUAS
        // cores — linha e miolo). O Smoothing fica de fora (ver `smoothing_only`), e os
        // KNOBS do balde também: Gap/Grow/Precision são do gesto de preencher, não da
        // seleção.
        (
            FlipMode::Edit,
            &edit_expected,
            &[&eraser_only, &bucket_only, &sculpt_only, &smoothing_only],
        ),
        // Colorize (C2): a cor do rabisco + Apply/Clear, e NADA de traço/borracha/balde/
        // sculpt/edit (só o `mode_row` e o `colorize_section` pintam neste modo).
        (
            FlipMode::Colorize,
            &colorize_only,
            &[
                &stroke_only,
                &eraser_only,
                &bucket_only,
                &fill_swatch,
                &sculpt_only,
                &edit_only,
            ],
        ),
    ];

    // A tabela tem de cobrir TODOS os modos: um modo novo que entrasse sem um caso aqui
    // não seria testado por ninguém — e o `Edit` (W6) fez exatamente isso enquanto as
    // listas eram escritas à mão.
    assert_eq!(
        cases.len(),
        FlipMode::ALL.len(),
        "um modo NOVO nao tem caso nesta tabela — o gate modal esta cego para ele"
    );
    for (mode, expected, forbidden) in cases {
        let mut host = MockPanelHost::with_panel::<FlipPanel>();
        let mut st = FlipPanelState::default();
        ph2d_panel_flip::set_current_flip_style(Some(ph2d_tool_flip::FlipStyleSnapshot {
            mode,
            ..Default::default()
        }));
        let painted = host.paint::<FlipPanel>(&mut st, viewport());
        let on_screen = |id: ph2d_a11y::NodeId| painted.iter().any(|(w, r)| *w == id && r.w > 0.0);

        for (name, id) in expected {
            assert!(
                on_screen(*id),
                "modo {mode:?}: o proprio controle '{name}' NAO aparece"
            );
        }
        for group in forbidden {
            for (name, id) in *group {
                assert!(
                    !on_screen(*id),
                    "modo {mode:?}: aparece '{name}', que e atributo de OUTRA ferramenta"
                );
            }
        }
    }
}

/// O **Size** é o atributo compartilhado **POR DEFAULT**: a espessura do pincel, o raio da
/// borracha, o raio do pincel de escultura (W5 — de propósito: um 2º par de sliders para
/// raio e força seria estado duplicado, e trocar de modo obrigaria a re-ajustar tudo) e,
/// no **Edit** (W6), a espessura dos traços SELECIONADOS. Ele some no balde e no Select.
///
/// ⚠️ **§4.C emendou isto para a BORRACHA, sem revogar o default:** a linha do Size ganhou
/// um toggle de LINK (Unified Paint Settings do Blender) e ele nasce LIGADO — então a
/// borracha continua pintando o `FLIP_SIZE` do pincel, que é o que esta varredura afirma.
/// Deslinkar é opt-in e troca o widget (ver o gate irmão `an_unlinked_eraser_paints_...`).
/// O Sculpt e o Edit seguem compartilhando incondicionalmente.
///
/// A varredura cobre `FlipMode::ALL` (e afirma isso): um modo novo que mostrasse o Size
/// sem passar por aqui escaparia do gate — foi exatamente o que o Edit fez quando a lista
/// era escrita à mão.
#[test]
fn size_is_shared_by_brush_eraser_and_sculpt_and_absent_elsewhere() {
    let cases = [
        (FlipMode::Draw, true),
        (FlipMode::Erase, true),
        (FlipMode::Reshape, true),
        (FlipMode::Edit, true),
        (FlipMode::Fill, false),
        (FlipMode::Select, false),
        // Colorize COMPARTILHA o Size (regra do Erase/Sculpt): ele é a espessura do rabisco,
        // e como o rabisco semeia pela CÁPSULA, é o Size que decide se um toque curto pega a
        // região. A `colorize_section` pinta os MESMOS ids `FLIP_SIZE`/`_NUM`.
        (FlipMode::Colorize, true),
    ];
    assert_eq!(
        cases.len(),
        FlipMode::ALL.len(),
        "um modo NOVO nao entrou nesta varredura — o gate ficaria cego para ele"
    );
    for (mode, want) in cases {
        let mut host = MockPanelHost::with_panel::<FlipPanel>();
        let mut st = FlipPanelState::default();
        ph2d_panel_flip::set_current_flip_style(Some(ph2d_tool_flip::FlipStyleSnapshot {
            mode,
            ..Default::default()
        }));
        let painted = host.paint::<FlipPanel>(&mut st, viewport());
        let shown = painted
            .iter()
            .any(|(w, r)| *w == ids::FLIP_SIZE && r.w > 0.0);
        assert_eq!(shown, want, "modo {mode:?}: Size deveria aparecer? {want}");
    }
}

/// **Os oito pincéis de escultura chegam à tool — cada um no SEU.**
///
/// Duas listas têm de andar juntas: `FLIP_RESHAPE_KIND_IDS` (a ordem do painel) e
/// `ReshapeKind::ALL` (o vocabulário). O decodificador é o zip das duas — e um zip
/// entre listas *desalinhadas* compila perfeitamente e dá o pincel errado: o usuário
/// clica em Twist e o traço engrossa. Nenhum outro gate pega isso.
///
/// Aqui cada um dos oito ids é DIRIGIDO pelo seam real (painel → barramento → tool) e
/// a tool tem de acabar exatamente no pincel daquele botão.
#[test]
fn every_sculpt_brush_button_selects_its_own_brush() {
    use ph2d_tool_flip::ReshapeKind;

    for (id, kind) in ids::FLIP_RESHAPE_KIND_IDS.iter().zip(ReshapeKind::ALL) {
        let mut host = MockPanelHost::with_panel::<FlipPanel>();
        let mut panel_state = FlipPanelState::default();
        let mut tool = FlipTool::default();

        let outcome =
            host.apply_panel_event::<FlipPanel>(&mut panel_state, WidgetEvent::Click(*id));
        assert_eq!(
            outcome,
            EventOutcome::Consumed,
            "o clique no pincel {kind:?} foi IGNORADO — falta o arm em `event.rs`"
        );
        for action in host.drained_actions() {
            if let EditorAction::ToolPanelEvent(pe) = action {
                tool.handle_panel_event(pe);
            }
        }
        assert_eq!(
            tool.reshape_kind(),
            kind,
            "o botao {kind:?} selecionou OUTRO pincel (as duas listas desalinharam)"
        );
    }
}

/// E os oito **existem na tela** no modo Sculpt, com área clicável (o gate de PINTURA
/// — o botão que "não existe" com todos os outros gates verdes, BUGS #8).
#[test]
fn the_eight_sculpt_brushes_are_painted_in_reshape_mode() {
    let mut host = MockPanelHost::with_panel::<FlipPanel>();
    let mut st = FlipPanelState::default();
    ph2d_panel_flip::set_current_flip_style(Some(ph2d_tool_flip::FlipStyleSnapshot {
        mode: FlipMode::Reshape,
        ..Default::default()
    }));
    let painted = host.paint::<FlipPanel>(&mut st, viewport());

    for (id, kind) in ids::FLIP_RESHAPE_KIND_IDS
        .iter()
        .zip(ph2d_tool_flip::ReshapeKind::ALL)
    {
        let hit = painted.iter().find(|(w, _)| w == id);
        let Some((_, r)) = hit else {
            panic!("o pincel {kind:?} NAO e pintado: nao existe na tela");
        };
        assert!(
            r.w > 0.0 && r.h > 0.0,
            "o pincel {kind:?} foi pintado com area ZERO: invisivel e inclicavel ({r:?})"
        );
    }
    // E as duas fileiras não se sobrepõem (4 + 4, não 8 em cima de 4).
    let ys: Vec<f32> = ids::FLIP_RESHAPE_KIND_IDS
        .iter()
        .filter_map(|id| painted.iter().find(|(w, _)| w == id).map(|(_, r)| r.y))
        .collect();
    assert_eq!(ys.len(), 8);
    assert!(
        ys[0] == ys[3] && ys[4] == ys[7] && ys[4] > ys[0],
        "os oito nao sairam em DUAS fileiras de quatro: {ys:?}"
    );
}

/// **O traço preenchido (W5.1) chega à tool** — e a linha Shape só existe no Draw.
///
/// É o material `stroke + fill` do Grease Pencil (como o Suzanne é desenhado): o fill é
/// a triangulação dos pontos do PRÓPRIO traço, então linha e cor são uma geometria só.
#[test]
fn the_shape_row_toggles_the_filled_stroke_and_lives_only_in_draw_mode() {
    // (a) o seam: o clique chega e a tool muda.
    let mut host = MockPanelHost::with_panel::<FlipPanel>();
    let mut st = FlipPanelState::default();
    let mut tool = FlipTool::default();
    assert!(!tool.draw_filled(), "o default e a linha simples");

    let outcome =
        host.apply_panel_event::<FlipPanel>(&mut st, WidgetEvent::Click(ids::FLIP_SHAPE_FILLED));
    assert_eq!(outcome, EventOutcome::Consumed, "o clique foi IGNORADO");
    for action in host.drained_actions() {
        if let EditorAction::ToolPanelEvent(pe) = action {
            tool.handle_panel_event(pe);
        }
    }
    assert!(tool.draw_filled(), "o Filled nao chegou na tool");

    host.apply_panel_event::<FlipPanel>(&mut st, WidgetEvent::Click(ids::FLIP_SHAPE_LINE));
    for action in host.drained_actions() {
        if let EditorAction::ToolPanelEvent(pe) = action {
            tool.handle_panel_event(pe);
        }
    }
    assert!(!tool.draw_filled(), "o Line nao desligou o preenchimento");

    // (b) a PINTURA: a linha Shape existe no Draw e em nenhum outro modo.
    for (mode, want) in [
        (FlipMode::Draw, true),
        (FlipMode::Erase, false),
        (FlipMode::Fill, false),
        (FlipMode::Reshape, false),
        (FlipMode::Select, false),
    ] {
        let mut host = MockPanelHost::with_panel::<FlipPanel>();
        let mut st = FlipPanelState::default();
        ph2d_panel_flip::set_current_flip_style(Some(ph2d_tool_flip::FlipStyleSnapshot {
            mode,
            ..Default::default()
        }));
        let painted = host.paint::<FlipPanel>(&mut st, viewport());
        let shown = painted
            .iter()
            .any(|(w, r)| *w == ids::FLIP_SHAPE_FILLED && r.w > 0.0);
        assert_eq!(
            shown, want,
            "modo {mode:?}: a linha Shape deveria aparecer? {want}"
        );
    }
}

/// 🔴 **Os toggles de DOMÍNIO (W8 + §4.B) chegam à tool — e a linha Select só existe no
/// Edit.**
///
/// É o seam completo dos pills: pintado → hit → Click → bus → `handle_panel_event` →
/// `edit_domain` muda. Mutação que sangra: tirar um id do arm de eventos do painel (o
/// clique é engolido e a tool nunca vê o domínio) ou tirar o braço do
/// `handle_panel_event` (o clique chega e não faz nada).
///
/// A tabela é conferida contra o [`EditDomain::ALL`]: um domínio novo sem caso aqui é uma
/// pill pintada e inerte, e o `assert_eq!` de comprimento o barra na hora.
#[test]
fn the_domain_toggles_reach_the_tool_and_live_only_in_edit_mode() {
    use ph2d_tool_flip::EditDomain;

    let cases = [
        (ids::FLIP_EDIT_DOM_POINT, EditDomain::Point),
        (ids::FLIP_EDIT_DOM_SEGMENT, EditDomain::Segment),
        (ids::FLIP_EDIT_DOM_STROKE, EditDomain::Stroke),
    ];
    assert_eq!(
        cases.len(),
        EditDomain::ALL.len(),
        "dominio novo sem seam test: pill pintada e inerte e o bug no 1 do projeto"
    );

    // (a) o seam: cada clique chega e a tool muda de domínio.
    let mut host = MockPanelHost::with_panel::<FlipPanel>();
    let mut st = FlipPanelState::default();
    let mut tool = FlipTool::default();
    assert_eq!(tool.edit_domain(), EditDomain::Stroke, "o default e Stroke");
    for (id, want) in cases {
        let outcome = host.apply_panel_event::<FlipPanel>(&mut st, WidgetEvent::Click(id));
        assert_eq!(
            outcome,
            EventOutcome::Consumed,
            "o clique {want:?} foi IGNORADO"
        );
        for action in host.drained_actions() {
            if let EditorAction::ToolPanelEvent(pe) = action {
                tool.handle_panel_event(pe);
            }
        }
        assert_eq!(tool.edit_domain(), want, "o {want:?} nao chegou na tool");
    }

    // (b) a PINTURA: a linha do domínio existe no Edit e em nenhum outro modo — e os TRÊS
    // pills são pintados (um id sem pintura é um domínio inalcançável pelo mouse).
    for mode in ph2d_tool_flip::FlipMode::ALL {
        let want = mode == FlipMode::Edit;
        let mut host = MockPanelHost::with_panel::<FlipPanel>();
        let mut st = FlipPanelState::default();
        ph2d_panel_flip::set_current_flip_style(Some(ph2d_tool_flip::FlipStyleSnapshot {
            mode,
            ..Default::default()
        }));
        let painted = host.paint::<FlipPanel>(&mut st, viewport());
        for (id, dom) in cases {
            let shown = painted.iter().any(|(w, r)| *w == id && r.w > 0.0);
            assert_eq!(
                shown, want,
                "modo {mode:?}: o pill {dom:?} deveria aparecer? {want}"
            );
        }
    }
}

/// 🔴 **O botão Duplicate Layer CHEGA ao barramento** (§4.C) — o forward do painel
/// (`event.rs`). O apply real (duplicar no doc) mora no shell (`flip_layers`), mas se o
/// painel ENGOLIR o clique, nada chega lá. Este gate dirige o Click e prova o forward.
///
/// Mutação que sangra: tirar o `FLIP_LAYER_DUPLICATE` do arm de eventos do painel — o
/// clique deixa de ser Consumed / de virar um `ToolPanelEvent::Click(FLIP_LAYER_DUPLICATE)`.
#[test]
fn duplicate_layer_button_forwards_to_the_bus() {
    use ph2d_editor_core::tool::PanelEvent;

    let mut host = MockPanelHost::with_panel::<FlipPanel>();
    let mut st = FlipPanelState::default();
    let outcome =
        host.apply_panel_event::<FlipPanel>(&mut st, WidgetEvent::Click(ids::FLIP_LAYER_DUPLICATE));
    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "o clique no Duplicate foi IGNORADO — falta o arm em `event.rs`"
    );
    let forwarded = host.drained_actions().into_iter().any(|a| {
        matches!(
            a,
            EditorAction::ToolPanelEvent(PanelEvent::Click(id)) if id == ids::FLIP_LAYER_DUPLICATE
        )
    });
    assert!(
        forwarded,
        "o Duplicate nao chegou ao barramento — o shell (flip_layers) nunca duplica"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Inline layer rename (§4.C) — double-click a name → edit → Enter commits.
// The commit travels on the layer's Row id via `SelectOption`; the shell drain
// (`flip_layers`) decodes it and renames. Three halves: OPEN (double-click),
// COMMIT (Enter forwards the new name), PAINT (the field owns the name strip).
// ─────────────────────────────────────────────────────────────────────────────

use ph2d_editor_core::tool::PanelEvent;

/// One published layer, named `name`, active.
fn one_layer(id: u64, name: &str) {
    ph2d_panel_flip::set_current_flip_layers(FlipLayersSnapshot {
        rows: vec![FlipLayerRow {
            id,
            name: name.to_string(),
            blend: 0,
            opacity: 1.0,
            visible: true,
            locked: false,
        }],
        active: Some(id),
    });
}

/// 🔴 **Double-clicking a layer NAME opens the inline rename** (§4.C).
///
/// A single click still selects (that seam is covered elsewhere); the SECOND click of
/// the pair upgrades to `DoubleClick(row_id)` and opens the field on that layer.
///
/// Mutação que sangra: apagar o arm `WidgetEvent::DoubleClick` do `event.rs` — o
/// double-click deixa de ser Consumed e `layer_rename` fica `None`.
#[test]
fn double_clicking_a_layer_name_opens_the_rename() {
    let a = 7u64;
    one_layer(a, "Rough");
    let mut host = MockPanelHost::with_panel::<FlipPanel>();
    let mut st = FlipPanelState::default();

    let row_id = ids::flip_layer_widget_id(a, FlipLayerWidget::Row);
    let outcome = host.apply_panel_event::<FlipPanel>(&mut st, WidgetEvent::DoubleClick(row_id));
    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "o double-click no nome foi IGNORADO — falta o arm em `event.rs`"
    );
    assert_eq!(
        st.layer_rename.map(|lr| lr.layer),
        Some(a),
        "o rename nao abriu na camada do double-click"
    );
}

/// 🔴 **Enter no campo aberto COMMITA o novo nome pela Row id** (§4.C) — o canal que o
/// shell (`flip_layers`) decodifica para renomear no doc.
///
/// O campo é semeado (via `paint`) com o nome ATUAL da camada; o commit forwarda esse
/// texto. Mutação que sangra: o commit forwardar um id/nome hardcoded, ou ler o slot
/// errado do store — o `matches!` de `SelectOption(row_id, "Rough")` cai. E não fechar
/// o `layer_rename` deixa o `is_none()` vermelho.
#[test]
fn committing_the_rename_forwards_the_new_name_on_the_row_id() {
    let a = 11u64;
    one_layer(a, "Rough");
    let mut host = MockPanelHost::with_panel::<FlipPanel>();
    let mut st = FlipPanelState::default();
    let row_id = ids::flip_layer_widget_id(a, FlipLayerWidget::Row);

    // Open + paint (seeds the field with the current name, focused).
    host.apply_panel_event::<FlipPanel>(&mut st, WidgetEvent::DoubleClick(row_id));
    host.paint::<FlipPanel>(&mut st, viewport());

    // Enter → Submit → commit forwards SelectOption(row_id, "Rough").
    let outcome = host
        .apply_panel_event::<FlipPanel>(&mut st, WidgetEvent::Submit(ids::FLIP_LAYER_RENAME_INPUT));
    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "o Enter no campo foi ignorado"
    );
    let forwarded = host.drained_actions().into_iter().any(|act| {
        matches!(
            act,
            EditorAction::ToolPanelEvent(PanelEvent::SelectOption(id, name))
                if id == row_id && name == "Rough"
        )
    });
    assert!(
        forwarded,
        "o commit nao forwardou SelectOption(row_id, name) — o shell nunca renomeia"
    );
    assert!(
        st.layer_rename.is_none(),
        "o campo de rename fica FECHADO apos o commit"
    );
}

/// 🔴 **O campo de rename OCUPA a faixa do nome — e o hit da row cede a ele** (§4.C).
///
/// Enquanto renomeia, um clique na faixa tem de editar texto, não re-selecionar a
/// camada; então o hit registrado ali é o do CAMPO, não o da Row. Fora do rename é o
/// contrário. Mutação que sangra: pintar o texto do nome (registrando a Row) em vez do
/// campo enquanto renomeia — o campo some do painted e a Row reaparece.
#[test]
fn the_rename_field_owns_the_name_strip_while_renaming() {
    let a = 3u64;
    one_layer(a, "A");
    let mut host = MockPanelHost::with_panel::<FlipPanel>();
    let mut st = FlipPanelState::default();
    let row_id = ids::flip_layer_widget_id(a, FlipLayerWidget::Row);

    // Não renomeando: a faixa do nome é a Row; o campo não existe.
    let painted = host.paint::<FlipPanel>(&mut st, viewport());
    assert!(
        painted.iter().any(|(w, r)| *w == row_id && r.w > 0.0),
        "a faixa do nome (Row) deveria ser pintada e clicável"
    );
    assert!(
        !painted
            .iter()
            .any(|(w, _)| *w == ids::FLIP_LAYER_RENAME_INPUT),
        "nao ha campo de rename sem rename aberto"
    );

    // Renomeando ESTA row: o campo ocupa a faixa; a Row cede o hit.
    st.layer_rename = Some(LayerRename {
        layer: a,
        opened: false,
    });
    let painted = host.paint::<FlipPanel>(&mut st, viewport());
    assert!(
        painted
            .iter()
            .any(|(w, r)| *w == ids::FLIP_LAYER_RENAME_INPUT && r.w > 0.0),
        "o campo de rename nao foi pintado sobre a row renomeada"
    );
    assert!(
        !painted.iter().any(|(w, _)| *w == row_id),
        "o hit da row-select tem de CEDER ao campo enquanto renomeia"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// §4.C — os LINKS da borracha (Unified Paint Settings do Blender).
//
// Um toggle na LINHA da propriedade diz se o Size/Strength da borracha SEGUE o do
// pincel. Ligado é o default (e o comportamento histórico); desligado, a borracha
// passa a ler/escrever widgets PRÓPRIOS. Três coisas a provar: o toggle chega na
// tool, o slider próprio chega na tool, e a tela troca o widget conforme o link.
// ─────────────────────────────────────────────────────────────────────────────

/// Um snapshot no modo Erase com os links no estado dado.
fn erase_snap(link_size: bool, link_strength: bool) -> ph2d_tool_flip::FlipStyleSnapshot {
    ph2d_tool_flip::FlipStyleSnapshot {
        mode: FlipMode::Erase,
        link_size,
        link_strength,
        ..Default::default()
    }
}

/// 🔴 **Os dois toggles de link CHEGAM na tool** e invertem o flag certo.
///
/// Mutação que sangra: tirar `FLIP_LINK_SIZE`/`FLIP_LINK_STRENGTH` do arm de eventos do
/// painel (o clique é engolido) ou o braço do `handle_panel_event` (chega e não faz nada).
#[test]
fn the_link_toggles_reach_the_tool() {
    for (id, name) in [
        (ids::FLIP_LINK_SIZE, "Size"),
        (ids::FLIP_LINK_STRENGTH, "Strength"),
    ] {
        let mut host = MockPanelHost::with_panel::<FlipPanel>();
        let mut st = FlipPanelState::default();
        let mut tool = FlipTool::default();
        assert!(
            tool.link_size() && tool.link_strength(),
            "o default e LINKADO"
        );

        let outcome = host.apply_panel_event::<FlipPanel>(&mut st, WidgetEvent::Click(id));
        assert_eq!(
            outcome,
            EventOutcome::Consumed,
            "o clique no link de {name} foi IGNORADO — falta o arm em `event.rs`"
        );
        for action in host.drained_actions() {
            if let EditorAction::ToolPanelEvent(pe) = action {
                tool.handle_panel_event(pe);
            }
        }
        let (size, strength) = (tool.link_size(), tool.link_strength());
        if id == ids::FLIP_LINK_SIZE {
            assert!(!size && strength, "o toggle de Size mexeu no flag errado");
        } else {
            assert!(
                size && !strength,
                "o toggle de Strength mexeu no flag errado"
            );
        }
    }
}

/// 🔴 **O slider PRÓPRIO da borracha chega na tool** — e move só ela.
///
/// Mutação que sangra: tirar `FLIP_ERASE_SIZE` do arm de `ValueChanged` (o arrasto é
/// engolido) ou do `handle_panel_event` (chega e não escreve `erase_px`).
#[test]
fn the_unlinked_eraser_slider_reaches_the_tool() {
    let mut host = MockPanelHost::with_panel::<FlipPanel>();
    let mut st = FlipPanelState::default();
    let mut tool = FlipTool::default();
    tool.handle_panel_event(ph2d_editor_core::tool::PanelEvent::Click(
        ids::FLIP_LINK_SIZE,
    )); // deslinka

    host.set_slider_value(ids::FLIP_ERASE_SIZE, 1.0);
    let outcome = host
        .apply_panel_event::<FlipPanel>(&mut st, WidgetEvent::ValueChanged(ids::FLIP_ERASE_SIZE));
    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "o arrasto do Size da borracha foi IGNORADO"
    );
    for action in host.drained_actions() {
        if let EditorAction::ToolPanelEvent(pe) = action {
            tool.handle_panel_event(pe);
        }
    }
    assert_eq!(
        tool.eraser_size_px(),
        WIDTH_MAX_PX,
        "o slider proprio nao chegou na borracha"
    );
    assert_eq!(
        tool.width_px(),
        ph2d_tool_flip::DEFAULT_WIDTH_PX,
        "e o PINCEL nao pode ter se mexido"
    );
}

/// 🔴 **O link TROCA o widget que a linha pinta** (§4.C): linkado é o slider do pincel,
/// deslinkado é o da borracha — nunca os dois, nunca nenhum.
///
/// É o gate que impede as duas patologias: um slider deslinkado que ainda escreve no
/// pincel (linkado pintando o id próprio) e um controle morto na tela (o id do pincel
/// pintado enquanto a borracha lê outro número).
///
/// Mutação que sangra: o `brush()` ignorar `snap.link_size` — o par de asserts inverte.
#[test]
fn an_unlinked_eraser_paints_its_own_slider_and_a_linked_one_paints_the_brushs() {
    // (a) LINKADO (o default): a linha é a do PINCEL.
    let mut host = MockPanelHost::with_panel::<FlipPanel>();
    let mut st = FlipPanelState::default();
    ph2d_panel_flip::set_current_flip_style(Some(erase_snap(true, true)));
    let painted = host.paint::<FlipPanel>(&mut st, viewport());
    let on = |p: &Vec<(ph2d_a11y::NodeId, ph2d_editor_core::zones::Rect)>,
              id: ph2d_a11y::NodeId| p.iter().any(|(w, r)| *w == id && r.w > 0.0);
    assert!(on(&painted, ids::FLIP_SIZE), "linkado: o Size do pincel");
    assert!(
        on(&painted, ids::FLIP_OPACITY),
        "linkado: a Strength do pincel"
    );
    assert!(
        !on(&painted, ids::FLIP_ERASE_SIZE) && !on(&painted, ids::FLIP_ERASE_STRENGTH),
        "linkado NAO pode pintar os sliders proprios da borracha"
    );

    // (b) DESLINKADO: a linha vira a da BORRACHA.
    ph2d_panel_flip::set_current_flip_style(Some(erase_snap(false, false)));
    let painted = host.paint::<FlipPanel>(&mut st, viewport());
    assert!(
        on(&painted, ids::FLIP_ERASE_SIZE) && on(&painted, ids::FLIP_ERASE_STRENGTH),
        "deslinkado: os sliders PROPRIOS da borracha"
    );
    assert!(
        !on(&painted, ids::FLIP_SIZE) && !on(&painted, ids::FLIP_OPACITY),
        "deslinkado, o slider do PINCEL nao pode continuar na tela (ele escreveria no \
         pincel enquanto a borracha le outro numero)"
    );

    // (c) Os TOGGLES existem nos dois estados (é por eles que se volta atrás).
    for linked in [true, false] {
        ph2d_panel_flip::set_current_flip_style(Some(erase_snap(linked, linked)));
        let painted = host.paint::<FlipPanel>(&mut st, viewport());
        assert!(
            on(&painted, ids::FLIP_LINK_SIZE) && on(&painted, ids::FLIP_LINK_STRENGTH),
            "os toggles de link somem com link={linked} — nao da pra desfazer"
        );
    }
}

/// 🔴 **Os toggles de link só existem na BORRACHA.** Eles governam pintura↔borracha; num
/// modo sem borracha seriam um controle que não decide nada — a doutrina modal do painel.
///
/// A varredura cobre `FlipMode::ALL`: um modo novo que os pintasse escaparia do gate.
#[test]
fn the_link_toggles_live_only_in_erase_mode() {
    for mode in ph2d_tool_flip::FlipMode::ALL {
        let want = mode == FlipMode::Erase;
        let mut host = MockPanelHost::with_panel::<FlipPanel>();
        let mut st = FlipPanelState::default();
        ph2d_panel_flip::set_current_flip_style(Some(ph2d_tool_flip::FlipStyleSnapshot {
            mode,
            ..Default::default()
        }));
        let painted = host.paint::<FlipPanel>(&mut st, viewport());
        for (id, name) in [
            (ids::FLIP_LINK_SIZE, "link Size"),
            (ids::FLIP_LINK_STRENGTH, "link Strength"),
        ] {
            let shown = painted.iter().any(|(w, r)| *w == id && r.w > 0.0);
            assert_eq!(
                shown, want,
                "modo {mode:?}: o '{name}' deveria aparecer? {want}"
            );
        }
    }
}

/// 🔴 **A Strength é SOFT-only** (Enio 2026-07-17: *"borracha hard não obedece a
/// strength"* — não obedecia mesmo).
///
/// Hard CORTA o ponto e Stroke apaga o traço inteiro: as duas são binárias e não têm o que
/// dosar — o `erase_at` sempre documentou o parâmetro como *"(Soft only)"*. O slider ficava
/// pintado e INERTE, que é o controle morto proibido pela doutrina modal deste painel
/// (*"o usuário mexe, nada muda, e conclui que o app está quebrado"*).
///
/// O toggle de LINK da Strength vai junto: ele governa um número que, ali, não existe.
///
/// Mutação que sangra: pintar a linha da Strength em qualquer sub-modo (tirar o
/// `strength_applies` do `brush()`) — os casos Hard/Stroke caem.
#[test]
fn the_strength_row_lives_only_in_the_soft_eraser() {
    use ph2d_tool_flip::EraseMode;

    let cases = [
        (EraseMode::Soft, true),
        (EraseMode::Hard, false),
        (EraseMode::Stroke, false),
    ];
    for (erase, want) in cases {
        let mut host = MockPanelHost::with_panel::<FlipPanel>();
        let mut st = FlipPanelState::default();
        ph2d_panel_flip::set_current_flip_style(Some(ph2d_tool_flip::FlipStyleSnapshot {
            mode: FlipMode::Erase,
            erase,
            ..Default::default()
        }));
        let painted = host.paint::<FlipPanel>(&mut st, viewport());
        let on = |id: ph2d_a11y::NodeId| painted.iter().any(|(w, r)| *w == id && r.w > 0.0);

        assert_eq!(
            on(ids::FLIP_OPACITY),
            want,
            "borracha {erase:?}: a Strength deveria aparecer? {want} \
             (Hard/Stroke sao binarias — o slider ali nao faz NADA)"
        );
        assert_eq!(
            on(ids::FLIP_LINK_STRENGTH),
            want,
            "borracha {erase:?}: o link da Strength segue a propria Strength"
        );
        // O Size (e o link dele) existem nos TRÊS: raio toda borracha tem.
        assert!(on(ids::FLIP_SIZE), "borracha {erase:?}: o Size sumiu");
        assert!(
            on(ids::FLIP_LINK_SIZE),
            "borracha {erase:?}: o link do Size sumiu"
        );
    }
}

/// **A costura do Trap, dirigida de ponta a ponta** (COLORIZE C1).
///
/// Arrasta o slider REAL e prova que o valor chega ao tool — exercitando os sete
/// sítios (`ids` → `populate` → `paint`/hit → `event` → bus → `handle_panel_event` →
/// snapshot). É o que a DIRETIVA §2 exige: um widget pintado sem arm é um clique
/// dropado **em silêncio**, e nenhum `cargo check` o pega.
///
/// A asserção é o VALOR projetado, não "algo mudou": o mapa `track → px` vive em dois
/// lugares (o painel desenha, o tool projeta) e um mapa divergente é um slider que
/// mostra um número e aplica outro.
#[test]
fn the_trap_slider_reaches_the_tool() {
    let mut host = MockPanelHost::with_panel::<FlipPanel>();
    let mut panel_state = FlipPanelState::default();
    let mut tool = FlipTool::default();

    assert_eq!(
        tool.ui_snapshot().trap,
        0.0,
        "o Trap TEM de nascer desligado — a wave Colorize e opt-in, e um default \
         diferente de 0 reescreveria o balde que o Enio ja aprovou"
    );

    host.set_slider_value(ids::FLIP_TRAP, 1.0);
    let outcome = host.apply_panel_event::<FlipPanel>(
        &mut panel_state,
        WidgetEvent::ValueChanged(ids::FLIP_TRAP),
    );
    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "o painel ignorou uma edicao real do slider — falta o arm de FLIP_TRAP em event.rs"
    );

    let mut forwarded = false;
    for action in host.drained_actions() {
        if let EditorAction::ToolPanelEvent(pe) = action {
            tool.handle_panel_event(pe);
            forwarded = true;
        }
    }
    assert!(
        forwarded,
        "a edicao do Trap nunca chegou ao bus como ToolPanelEvent — a costura esta morta"
    );
    assert_eq!(
        tool.ui_snapshot().trap,
        ph2d_tool_flip::TRAP_MAX_PX,
        "a costura slider→tool entregou o px errado para o Trap"
    );
}
