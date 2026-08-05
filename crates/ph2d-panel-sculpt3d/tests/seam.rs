//! **A costura de comportamento do painel da cena 3D.**
//!
//! Verde de compilação vale zero aqui: toda falha que este painel pode ter é um
//! controle que pinta e não faz nada. Então a varredura não escolhe uma row
//! representativa — ela percorre [`rows`], a MESMA tabela que o `paint`, o
//! `populate` e o `event` percorrem, e pergunta a cada uma
//! ([[feedback_the_fullest_card_premise_rots]]).
//!
//! Duas metades, porque falham de forma independente:
//!
//! * **Registrado → despacha**: dirige `ValueChanged`/`Click` e afirma o intent.
//! * **Pintado → clicável**: roda o `paint` REAL e clica o rect que ele
//!   registrou, pelo dispatcher real. Um widget não está pronto quando pinta;
//!   ele está pronto quando um teste o clica
//!   ([[feedback_widget_is_done_when_a_test_clicks_it]]).

use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::EventOutcome;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_sculpt3d::{
    Sculpt3dIntent, Sculpt3dPanel, Sculpt3dPanelState, Sculpt3dSnapshot, Sculpt3dUi, drain_intents,
    ids, rows, set_current_sculpt3d,
};
use ph2d_sculpt3d::{Falloff, Verb};
use ph2d_ui_testkit::MockPanelHost;

/// Um viewport do tamanho do dock. ALTO, porque o painel tem seis seções e um
/// paint que ficasse sem espaço não registraria nada e passaria calado.
const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 2400.0,
};

/// Põe um estado conhecido na frente do painel e limpa o que estiver na fila.
fn arrange(ui: Sculpt3dUi) -> (MockPanelHost, Sculpt3dPanelState) {
    set_current_sculpt3d(Some(Sculpt3dSnapshot {
        ui,
        dyntopo: false,
        level: 0,
        level_count: 1,
        pieces: 1,
        isolated: false,
        verts: 6050,
    }));
    let _ = drain_intents();
    (
        MockPanelHost::with_panel::<Sculpt3dPanel>(),
        Sculpt3dPanelState,
    )
}

/// O único intent enfileirado, ou um pânico que diz o que faltou.
fn only_intent(what: &str) -> Sculpt3dIntent {
    let intents = drain_intents();
    assert_eq!(
        intents.len(),
        1,
        "`{what}` devia enfileirar UM intent e enfileirou {intents:?}"
    );
    intents[0]
}

/// **Toda row despacha, e carrega o valor que a pista significa.**
///
/// O oráculo não é *"saiu um intent"*: é *"ESTE campo foi para o valor que esta
/// pista mapeia, e nenhum outro se mexeu"*. Uma row ligada ao setter errado
/// também emitiria um intent.
#[test]
fn every_row_reaches_the_authored_state() {
    for row in rows::rows() {
        let base = Sculpt3dUi::default();
        let (mut host, mut state) = arrange(base);

        let track = 0.75_f32;
        host.set_slider_value(row.slider, track);
        let outcome = host
            .apply_panel_event::<Sculpt3dPanel>(&mut state, WidgetEvent::ValueChanged(row.slider));
        assert_eq!(
            outcome,
            EventOutcome::Consumed,
            "a row `{}` ignorou um arrasto REAL de slider — falta o braço dela no event.rs",
            row.label
        );

        let Sculpt3dIntent::SetUi(got) = only_intent(row.label) else {
            panic!("a row `{}` enfileirou o tipo errado de intent", row.label);
        };

        let mut want = base;
        (row.set)(&mut want, row.value_of(track));
        assert!(
            ((row.get)(&got) - (row.get)(&want)).abs() < 1e-4,
            "a row `{}` levou o campo dela a {} e a pista {track} significa {}",
            row.label,
            (row.get)(&got),
            (row.get)(&want)
        );
        // E nada mais se moveu. É isto que pega uma row ligada ao setter do
        // vizinho — um copy-paste que um oráculo de *"saiu um intent"* não vê.
        assert_eq!(
            got, want,
            "a row `{}` mexeu num campo que não é dela",
            row.label
        );
    }
}

/// **Cada row possui exatamente um campo, e o getter e o setter dela concordam
/// sobre qual.**
///
/// ⚠️ Isto existe porque a versão óbvia — dentro da varredura acima, comparando
/// contra `(row.set)(&mut want, …)` — é **circular**: ela computa a expectativa
/// com a própria função em que o bug moraria, então ligar a row do `pinch` ao
/// setter do `strength` moveria os dois lados igual e o gate ficaria VERDE sob
/// exatamente essa mutação.
#[test]
fn each_row_owns_exactly_one_field() {
    for row in rows::rows() {
        let probe = row.value_of(0.375);
        let mut ui = Sculpt3dUi::default();
        (row.set)(&mut ui, probe);
        let read_back = (row.get)(&ui);
        let tolerance = if row.decimals == 0 { 0.5 } else { 1e-3 };
        assert!(
            (read_back - probe).abs() <= tolerance,
            "row `{}`: escreveu {probe} pelo setter e leu {read_back} pelo getter \
             — os dois nomeiam campos diferentes",
            row.label
        );
        for other in rows::rows() {
            if std::ptr::eq(other, row) {
                continue;
            }
            let before = (other.get)(&Sculpt3dUi::default());
            let after = (other.get)(&ui);
            assert!(
                (before - after).abs() < 1e-6,
                "a row `{}` mexeu no campo da row `{}` ({before} -> {after}) — \
                 duas rows estão ligadas a um campo só",
                row.label,
                other.label
            );
        }
    }
}

/// **A edição do chip não notifica em dobro.** Ele está ligado à pista, que já
/// espelhou o valor e disparou o próprio `ValueChanged`.
#[test]
fn a_chip_edit_is_swallowed_because_its_slider_already_spoke() {
    for row in rows::rows() {
        let (mut host, mut state) = arrange(Sculpt3dUi::default());
        let outcome = host
            .apply_panel_event::<Sculpt3dPanel>(&mut state, WidgetEvent::ValueChanged(row.chip));
        assert_eq!(outcome, EventOutcome::Consumed);
        assert!(
            drain_intents().is_empty(),
            "o chip de `{}` enfileirou um intent por cima do da pista — uma \
             edição aplicada duas vezes",
            row.label
        );
    }
}

/// **Todo chip carrega a faixa registrada**, que é o que o torna arrastável em
/// vez de um interruptor de duas posições.
#[test]
fn every_chip_is_draggable_because_its_range_is_registered() {
    let (host, _state) = arrange(Sculpt3dUi::default());
    for row in rows::rows() {
        let (min, max, step) = host.store().number_range(row.chip).unwrap_or_else(|| {
            panic!(
                "o chip de `{}` não tem faixa registrada — o arrasto dele \
                 percorreria ~50 unidades por pixel e ele viraria um min/max",
                row.label
            )
        });
        assert!((min - f64::from(row.min)).abs() < 1e-9, "`{}`", row.label);
        assert!((max - f64::from(row.max)).abs() < 1e-9, "`{}`", row.label);
        assert!((step - row.step).abs() < 1e-9, "`{}`", row.label);
    }
}

/// **Os dezesseis verbos são alcançáveis, e cada chip pega o SEU.**
///
/// O `Magnify` existia no enum, tinha alvo e era varrido por todo gate — e o
/// artista **não conseguia pegá-lo**, porque os dez dígitos já estavam tomados.
/// Uma lista de chips derivada de `Verb::ALL` torna essa classe impossível, e
/// este gate é o que prova que o índice do chip e o do enum são o mesmo.
#[test]
fn every_verb_has_a_chip_that_selects_it() {
    assert_eq!(
        ids::SCULPT3D_VERB.len(),
        Verb::ALL.len(),
        "a lista de chips e a lista de verbos têm tamanhos diferentes — algum \
         verbo é inalcançável, ou algum chip nomeia um verbo que não existe"
    );
    for (i, want) in Verb::ALL.into_iter().enumerate() {
        let (mut host, mut state) = arrange(Sculpt3dUi::default());
        let outcome = host.apply_panel_event::<Sculpt3dPanel>(
            &mut state,
            WidgetEvent::Click(ids::SCULPT3D_VERB[i]),
        );
        assert_eq!(outcome, EventOutcome::Consumed, "chip {i} não despacha");
        let Sculpt3dIntent::SetUi(got) = only_intent(want.label()) else {
            panic!("o chip de `{}` enfileirou o intent errado", want.label());
        };
        assert_eq!(
            got.brush.verb,
            want,
            "o chip {i} devia escolher `{}` e escolheu `{}`",
            want.label(),
            got.brush.verb.label()
        );
    }
}

/// **Escolher um verbo arma o default DELE, e nunca apaga uma escolha
/// deliberada.** A mesma lei do teclado, e o precedente é o
/// `arm_inflate_defaults` do Painter.
#[test]
fn picking_a_verb_arms_its_default_but_never_overwrites_the_artist() {
    // 1. Do default do verbo saindo → o default do verbo entrando.
    let base = Sculpt3dUi::default();
    assert!(
        (base.brush.strength - base.brush.verb.default_strength()).abs() < 1e-6,
        "a fixture tem de começar NO default, senão ela testa o outro ramo"
    );
    let mask = Verb::ALL
        .iter()
        .position(|v| *v == Verb::Mask)
        .expect("Mask está no ALL");
    let (mut host, mut state) = arrange(base);
    host.apply_panel_event::<Sculpt3dPanel>(
        &mut state,
        WidgetEvent::Click(ids::SCULPT3D_VERB[mask]),
    );
    let Sculpt3dIntent::SetUi(got) = only_intent("Mask") else {
        panic!("intent errado")
    };
    assert!(
        (got.brush.strength - Verb::Mask.default_strength()).abs() < 1e-6,
        "pegar a máscara não armou a força cheia dela — ela protegeria pela \
         metade e o barro se moveria por baixo"
    );

    // 2. Com uma força AUTORADA, o verbo novo não a toca.
    let authored = Sculpt3dUi {
        brush: ph2d_sculpt3d::Brush {
            strength: 0.123,
            ..base.brush
        },
        ..base
    };
    let (mut host, mut state) = arrange(authored);
    host.apply_panel_event::<Sculpt3dPanel>(
        &mut state,
        WidgetEvent::Click(ids::SCULPT3D_VERB[mask]),
    );
    let Sculpt3dIntent::SetUi(got) = only_intent("Mask") else {
        panic!("intent errado")
    };
    assert!(
        (got.brush.strength - 0.123).abs() < 1e-6,
        "pegar um verbo APAGOU a força que o artista tinha escolhido"
    );
}

/// **As cinco curvas são alcançáveis, e cada chip pega a SUA.**
#[test]
fn every_falloff_has_a_chip_that_selects_it() {
    assert_eq!(ids::SCULPT3D_FALLOFF.len(), Falloff::ALL.len());
    for (i, want) in Falloff::ALL.into_iter().enumerate() {
        let (mut host, mut state) = arrange(Sculpt3dUi::default());
        host.apply_panel_event::<Sculpt3dPanel>(
            &mut state,
            WidgetEvent::Click(ids::SCULPT3D_FALLOFF[i]),
        );
        let Sculpt3dIntent::SetUi(got) = only_intent(want.label()) else {
            panic!("intent errado")
        };
        assert_eq!(got.brush.falloff, want);
    }
}

/// **Os três eixos do espelho são INDEPENDENTES.** Um rádio faria X apagar Y, e
/// o ZBrush espelha em dois eixos ao mesmo tempo.
#[test]
fn each_mirror_axis_toggles_only_itself() {
    for (id, name) in [
        (ids::SCULPT3D_SYM_X, "X"),
        (ids::SCULPT3D_SYM_Y, "Y"),
        (ids::SCULPT3D_SYM_Z, "Z"),
    ] {
        let (mut host, mut state) = arrange(Sculpt3dUi::default());
        host.apply_panel_event::<Sculpt3dPanel>(&mut state, WidgetEvent::Click(id));
        let Sculpt3dIntent::SetUi(got) = only_intent(name) else {
            panic!("intent errado")
        };
        let on = [got.symmetry.x, got.symmetry.y, got.symmetry.z];
        assert_eq!(
            on.iter().filter(|b| **b).count(),
            1,
            "clicar o eixo {name} ligou {on:?} — os três eixos não são independentes"
        );
    }
}

/// **Todo comando de um toque chega ao shell**, e o certo.
#[test]
fn every_command_reaches_the_shell() {
    for (id, want) in [
        (ids::SCULPT3D_DYNTOPO, Sculpt3dIntent::ToggleDyntopo),
        (ids::SCULPT3D_LEVEL_DOWN, Sculpt3dIntent::ChangeLevel(false)),
        (ids::SCULPT3D_LEVEL_UP, Sculpt3dIntent::ChangeLevel(true)),
        (ids::SCULPT3D_SUBDIVIDE, Sculpt3dIntent::Subdivide),
        (ids::SCULPT3D_REVERSE, Sculpt3dIntent::ReverseLevel),
        (ids::SCULPT3D_REMESH, Sculpt3dIntent::Remesh),
        (ids::SCULPT3D_CLOSE_HOLES, Sculpt3dIntent::CloseHoles),
        (ids::SCULPT3D_DUPLICATE, Sculpt3dIntent::Duplicate),
        (ids::SCULPT3D_DELETE, Sculpt3dIntent::Delete),
        (ids::SCULPT3D_ISOLATE, Sculpt3dIntent::ToggleIsolate),
        (ids::SCULPT3D_MERGE, Sculpt3dIntent::Merge),
        (ids::SCULPT3D_ADD[0], Sculpt3dIntent::AddSphere),
        (ids::SCULPT3D_ADD[1], Sculpt3dIntent::AddCube),
        (ids::SCULPT3D_ADD[2], Sculpt3dIntent::AddCylinder),
        (ids::SCULPT3D_ADD[3], Sculpt3dIntent::AddTorus),
        (ids::SCULPT3D_MASK_OP[0], Sculpt3dIntent::MaskClear),
        (ids::SCULPT3D_MASK_OP[1], Sculpt3dIntent::MaskInvert),
        (ids::SCULPT3D_MASK_OP[2], Sculpt3dIntent::MaskBlur),
        (ids::SCULPT3D_MASK_OP[3], Sculpt3dIntent::MaskSharpen),
    ] {
        let (mut host, mut state) = arrange(Sculpt3dUi::default());
        let outcome = host.apply_panel_event::<Sculpt3dPanel>(&mut state, WidgetEvent::Click(id));
        assert_eq!(
            outcome,
            EventOutcome::Consumed,
            "{want:?} é pintado e não tem braço no event.rs"
        );
        assert_eq!(only_intent(&format!("{want:?}")), want);
    }
}

/// **Dobrar uma seção é local do painel** — nunca alcança a cena.
#[test]
fn folding_a_section_never_touches_the_scene() {
    for id in [
        ids::SCULPT3D_SEC_TOOL,
        ids::SCULPT3D_SEC_BRUSH,
        ids::SCULPT3D_SEC_SYMMETRY,
        ids::SCULPT3D_SEC_TOPOLOGY,
        ids::SCULPT3D_SEC_SHADING,
        ids::SCULPT3D_SEC_SCENE,
    ] {
        let (mut host, mut state) = arrange(Sculpt3dUi::default());
        let outcome = host.apply_panel_event::<Sculpt3dPanel>(&mut state, WidgetEvent::Click(id));
        assert_eq!(
            outcome,
            EventOutcome::Consumed,
            "o cabeçalho {id:?} não é clicável, mas o chevron dele é pintado"
        );
        assert!(host.store().is_collapsed(id), "a seção {id:?} não dobrou");
        assert!(
            drain_intents().is_empty(),
            "dobrar {id:?} publicou uma mudança de cena"
        );
    }
}

/// **Sem cena, o painel é INERTE.**
///
/// ⚠️ Não é higiene: com o retrato em `None` um clique que ainda enfileirasse
/// seria aplicado à primeira escultura que aparecesse — um gesto que o artista
/// fez sobre outra coisa, ressuscitado.
#[test]
fn with_no_scene_nothing_dispatches() {
    set_current_sculpt3d(None);
    let _ = drain_intents();
    let mut host = MockPanelHost::with_panel::<Sculpt3dPanel>();
    let mut state = Sculpt3dPanelState;
    let outcome = host.apply_panel_event::<Sculpt3dPanel>(
        &mut state,
        WidgetEvent::Click(ids::SCULPT3D_SUBDIVIDE),
    );
    assert_eq!(outcome, EventOutcome::Ignored);
    assert!(drain_intents().is_empty());
}

/// **Pintado ⟹ clicável.** O `paint` REAL, e depois o dispatcher REAL contra o
/// rect que ele registrou.
///
/// Esta é a metade que um `WidgetEvent` empurrado à mão pula: um controle pode
/// pintar, hit-registrar e encaminhar — todo outro gate verde — e continuar
/// morto de pedra sob o mouse porque o `populate` nunca o registrou.
#[test]
fn every_painted_control_is_clickable_where_it_is_drawn() {
    // ⚠️ **O Crease em mãos**, e a premissa é declarada em vez de herdada: as
    // rows `plane_offset` e `pinch` só são PINTADAS para os verbos que as leem,
    // então uma fixture no verbo default varreria doze das quatorze rows e
    // passaria — a forma exata do sweep que perde a premissa.
    let mut ui = Sculpt3dUi::default();
    ui.brush.verb = Verb::Crease;
    let (mut host, mut state) = arrange(ui);
    let painted = host.paint::<Sculpt3dPanel>(&mut state, VIEWPORT);

    // Duas listas, porque as duas metades do gate são diferentes: TUDO tem de
    // estar pintado, e só os BOTÕES respondem a um clique no centro (uma pista
    // de slider é dirigida por ARRASTO — exigir um Click dela seria afirmar um
    // gesto que ela não tem).
    let mut want: Vec<(String, ph2d_a11y::NodeId)> = Vec::new();
    for row in rows::rows() {
        // ⚠️ **A condição é `show` VERDADEIRO.** A primeira versão deste laço
        // tinha o `!` invertido: ela exigia as rows que o Crease NÃO pinta e
        // passava sobre um conjunto quase vazio — verde sobre nada, a forma exata
        // de gate que este arquivo existe para não ter.
        if (row.show)(&ui) {
            want.push((row.label.to_string(), row.slider));
            want.push((row.label.to_string(), row.chip));
        }
    }
    let sliders = want.len();
    // Os grupos de chips, os toggles e os comandos.
    for (i, v) in Verb::ALL.into_iter().enumerate() {
        want.push((format!("verb {}", v.label()), ids::SCULPT3D_VERB[i]));
    }
    for (i, f) in Falloff::ALL.into_iter().enumerate() {
        want.push((format!("falloff {}", f.label()), ids::SCULPT3D_FALLOFF[i]));
    }
    for (i, id) in ids::SCULPT3D_MASK_OP.into_iter().enumerate() {
        want.push((format!("mask op {i}"), id));
    }
    for (i, id) in ids::SCULPT3D_ADD.into_iter().enumerate() {
        want.push((format!("add {i}"), id));
    }
    for (i, id) in ids::SCULPT3D_DETAIL.into_iter().enumerate() {
        want.push((format!("detail {i}"), id));
    }
    for (name, id) in [
        ("sym x", ids::SCULPT3D_SYM_X),
        ("sym y", ids::SCULPT3D_SYM_Y),
        ("sym z", ids::SCULPT3D_SYM_Z),
        ("dyntopo", ids::SCULPT3D_DYNTOPO),
        ("level -", ids::SCULPT3D_LEVEL_DOWN),
        ("level +", ids::SCULPT3D_LEVEL_UP),
        ("subdivide", ids::SCULPT3D_SUBDIVIDE),
        ("reverse", ids::SCULPT3D_REVERSE),
        ("remesh", ids::SCULPT3D_REMESH),
        ("close holes", ids::SCULPT3D_CLOSE_HOLES),
        ("duplicate", ids::SCULPT3D_DUPLICATE),
        ("delete", ids::SCULPT3D_DELETE),
        ("isolate", ids::SCULPT3D_ISOLATE),
        ("merge", ids::SCULPT3D_MERGE),
    ] {
        want.push((name.to_string(), id));
    }

    for (name, id) in &want {
        assert!(
            painted.iter().any(|(pid, _)| pid == id),
            "`{name}` ({id:?}) está na tabela mas o paint nunca o registrou"
        );
    }

    // E cada BOTÃO responde de fato a um ponteiro no PRÓPRIO centro. É esta
    // metade que separa *hit-registrado* de *vivo sob o mouse* — a falha que o
    // gate de dez ferramentas do Impasto pegou nascendo verde.
    for (name, id) in want.iter().skip(sliders) {
        let rect = painted
            .iter()
            .rev()
            .find(|(pid, _)| pid == id)
            .map(|(_, r)| *r)
            .expect("registrado acima");
        let (cx, cy) = (rect.x + rect.w * 0.5, rect.y + rect.h * 0.5);
        assert_eq!(
            host.hit_at(cx, cy),
            Some(*id),
            "`{name}` é pintado mas outra coisa é dona dos pixels no centro dele"
        );
        let events = host.click_at(cx, cy);
        assert!(
            events
                .iter()
                .any(|e| matches!(e, WidgetEvent::Click(c) if c == id)),
            "clicar `{name}` no centro pintado não produziu Click — ele está no \
             índice de hit mas não é focável no store"
        );
    }
}

/// **As rows condicionais não são pintadas com a ferramenta errada.**
///
/// A metade oposta do gate acima, e ela falha sozinha: um `show` sempre-verdade
/// deixaria dois knobs mortos em doze das dezesseis ferramentas.
#[test]
fn a_conditional_row_is_absent_with_the_wrong_tool() {
    let mut ui = Sculpt3dUi::default();
    ui.brush.verb = Verb::Smooth; // nem plano nem crease
    let (mut host, mut state) = arrange(ui);
    let painted = host.paint::<Sculpt3dPanel>(&mut state, VIEWPORT);
    for id in [ids::SCULPT3D_PLANE_OFFSET, ids::SCULPT3D_PINCH] {
        assert!(
            !painted.iter().any(|(pid, _)| *pid == id),
            "{id:?} foi pintado com o Smooth em mãos — um knob que o verbo não lê"
        );
    }
    // E o controle: com o verbo que os lê, eles aparecem.
    ui.brush.verb = Verb::Clay;
    let (mut host, mut state) = arrange(ui);
    let painted = host.paint::<Sculpt3dPanel>(&mut state, VIEWPORT);
    assert!(
        painted
            .iter()
            .any(|(pid, _)| *pid == ids::SCULPT3D_PLANE_OFFSET),
        "o Clay é um verbo de PLANO e o Plane Offset não foi pintado"
    );
}
