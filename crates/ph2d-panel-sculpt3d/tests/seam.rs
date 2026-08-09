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
use ph2d_sculpt3d::{Alpha, Falloff, TransformKind, Verb};
use ph2d_ui_testkit::MockPanelHost;

/// A escala que a fixture finge que o modelo comporta.
const ALPHA_SEED: f32 = 0.0375;

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
    arrange_with(snapshot(ui, true))
}

/// A fixture com um retrato JÁ montado — a porta que o gate do botão de assar
/// usa para variar UM campo sem copiar os outros doze.
fn arrange_with(snap: Sculpt3dSnapshot) -> (MockPanelHost, Sculpt3dPanelState) {
    set_current_sculpt3d(Some(snap));
    let _ = drain_intents();
    (
        MockPanelHost::with_panel::<Sculpt3dPanel>(),
        Sculpt3dPanelState,
    )
}

/// O retrato da fixture. **Uma fixture, dois consumidores** — um gate que
/// montasse o seu próprio `Sculpt3dSnapshot` continuaria passando depois de o
/// desta função ficar torto.
fn snapshot(ui: Sculpt3dUi, has_bake_target: bool) -> Sculpt3dSnapshot {
    Sculpt3dSnapshot {
        // ⚠️ **DESARMADO e' o caso comum**, e a fixture o declara em vez de o
        // herdar: um gate que chegasse ao estado armado por toggle inverteria de
        // sentido no dia em que o default se movesse, e seguiria verde testando
        // o oposto. E o gate do transform arma o outro.
        transform: None,
        // O AO fresco e' o caso comum; o gate do aviso arma o outro.
        ao_stale: false,
        ui,
        dyntopo: false,
        level: 0,
        level_count: 1,
        pieces: 1,
        isolated: false,
        // ⚠️ A fixture publica os SEIS nomes do renderizador, e não uma lista
        // curta: a varredura de costura tem de encontrar TODO chip que o produto
        // pinta, e um retrato com dois materiais deixaria quatro ids fora do
        // sweep — vivos na tela e nunca clicados aqui.
        matcaps: &["Clay", "Pearl", "Skin", "Jade", "Metal", "Wax"],
        verts: 6050,
        // ⚠️ **Um seed DIFERENTE do default de fábrica**, senão o gate do
        // semeamento ficaria verde sem provar nada: a asserção é *"a escala foi
        // para a do modelo"*, e com os dois iguais ela não distingue semear de
        // não fazer coisa nenhuma.
        alpha_seed: ALPHA_SEED,
        // ⚠️ **O `arrange` publica COM alvo**, que é o estado em que o artista
        // de fato aperta o botão — e o que mantém a DICA fora do caminho de
        // todo sweep de layout. Quem varia este campo é o gate do botão.
        has_bake_target,
        // ⚠️ Um modelo de tamanho 2 — a esfera unitária que este módulo abre. Um
        // zero aqui faria o preview cair no piso do `span_of` e a fixture mediria
        // o degenerado em vez do caso normal.
        model_span: 2.0,
    }
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
        (ids::SCULPT3D_BAKE_AO, Sculpt3dIntent::BakeAo),
        (ids::SCULPT3D_BAKE_SPRITE, Sculpt3dIntent::BakeToSprite),
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
/// **A QUARTA condição de UI: a sequência LEVA a algum lugar.**
///
/// ⚠️ **A varredura logo abaixo prova a TERCEIRA** — que um ponteiro REAL
/// alcança os três retângulos. Ela não diz *o quê* eles empurram, e um id
/// registrado sem braço no `event` pinta, responde ao mouse e enfileira nada.
/// As duas metades são o par que este arquivo usa em todo grupo de chips.
#[test]
fn each_transform_chip_arms_its_own_kind() {
    assert_eq!(
        ids::SCULPT3D_TRANSFORM.len(),
        TransformKind::ALL.len(),
        "a lista de chips e a de espécies têm tamanhos diferentes -- uma delas é \
         inalcançável, ou um chip nomeia uma que não existe"
    );
    for (i, want) in TransformKind::ALL.into_iter().enumerate() {
        let (mut host, mut state) = arrange(Sculpt3dUi::default());
        let outcome = host.apply_panel_event::<Sculpt3dPanel>(
            &mut state,
            WidgetEvent::Click(ids::SCULPT3D_TRANSFORM[i]),
        );
        assert_eq!(outcome, EventOutcome::Consumed, "o chip {i} nao despacha");
        assert_eq!(
            only_intent(want.label()),
            Sculpt3dIntent::ArmTransform(want),
            "o chip de {} armou outra coisa",
            want.label()
        );
    }
}

#[test]
fn every_painted_control_is_clickable_where_it_is_drawn() {
    // ⚠️ **O Crease em mãos**, e a premissa é declarada em vez de herdada: as
    // rows `plane_offset` e `pinch` só são PINTADAS para os verbos que as leem,
    // então uma fixture no verbo default varreria doze das quatorze rows e
    // passaria — a forma exata do sweep que perde a premissa.
    let mut ui = Sculpt3dUi::default();
    ui.brush.verb = Verb::Crease;
    // ⚠️ **E um padrão DIRECIONAL armado, pela mesma razão, uma wave depois.**
    // As três rows do alpha (a escala e os dois ângulos do eixo) só existem com
    // um padrão em mãos, e as duas do EIXO só com um dos direcionais: com o
    // `alpha` no `None` de fábrica esta varredura passaria por cima delas e
    // ficaria verde sobre três controles que nunca foram clicados. É a terceira
    // vez que este arquivo escreve a mesma frase — *a fixture tem de conter o
    // fenômeno*.
    ui.brush.alpha = Some(Alpha::Strata);
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
    // ⚠️ **A opção `0` é o pincel LISO e não um padrão**, então o laço é sobre
    // `Alpha::ALL` deslocado de um — a mesma aritmética do pintor e do roteador.
    want.push(("alpha none".to_string(), ids::SCULPT3D_ALPHA[0]));
    for (i, a) in Alpha::ALL.into_iter().enumerate() {
        want.push((format!("alpha {}", a.label()), ids::SCULPT3D_ALPHA[i + 1]));
    }
    for (i, id) in ids::SCULPT3D_MASK_OP.into_iter().enumerate() {
        want.push((format!("mask op {i}"), id));
    }
    for (i, k) in TransformKind::ALL.into_iter().enumerate() {
        want.push((
            format!("transform {}", k.label()),
            ids::SCULPT3D_TRANSFORM[i],
        ));
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
        ("bake ao", ids::SCULPT3D_BAKE_AO),
        ("bake sprite", ids::SCULPT3D_BAKE_SPRITE),
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
    //
    // ⚠️ **Ela NÃO tem lista, e a ausência é a correção.** A versão anterior
    // enumerava os grupos à mão, e a enumeração já tinha apodrecido: os chips de
    // matcap, o Accumulate e o Wireframe **nunca foram varridos** — pintados,
    // hit-registrados, e nenhum gate perguntando se respondiam. Descoberto por
    // uma mutação que tirou uma fileira inteira do `populate` e deixou **os vinte
    // gates verdes**. Agora o conjunto é o que o PAINT registrou, então um
    // controle novo nasce coberto: é impossível esquecer de o acrescentar aqui,
    // porque não há aqui onde acrescentar.
    let by_id: std::collections::BTreeMap<_, _> = want
        .iter()
        .skip(sliders)
        .map(|(n, id)| (*id, n.clone()))
        .collect();
    // ⚠️ **A exclusão é por GESTO, e a polaridade é o ponto:** o default é *tem
    // de responder a um clique*, e sai da varredura só o que é dirigido por
    // ARRASTO — as pistas, os chips numéricos e o puxador do próprio painel
    // (chrome do host, não deste painel). Exigir um `Click` de qualquer um dos
    // três seria afirmar um gesto que eles não têm; e como a lista é de EXCEÇÕES,
    // esquecer um controle novo nela o deixa **coberto**, não fora.
    let mut dragged: Vec<ph2d_a11y::NodeId> =
        rows::rows().flat_map(|r| [r.slider, r.chip]).collect();
    dragged.extend([
        ph2d_editor_core::ids::INSP_DRAG_HANDLE,
        ph2d_editor_core::ids::INSP_RESIZE_HANDLE,
        ph2d_editor_core::ids::INSP_RESIZE_HANDLE_BL,
        ph2d_editor_core::widget::SCULPT3D_SCROLLBAR_ID,
    ]);
    let mut seen: Vec<ph2d_a11y::NodeId> = Vec::new();
    for &(id, rect) in &painted {
        if dragged.contains(&id) || seen.contains(&id) {
            continue;
        }
        seen.push(id);
        let name = by_id
            .get(&id)
            .cloned()
            .unwrap_or_else(|| format!("{id:?} (fora da tabela de presença)"));
        // O rect que VALE é o último registrado — um controle repintado numa
        // segunda passada é dono dos próprios pixels pela posição final.
        let rect = painted
            .iter()
            .rev()
            .find(|(pid, _)| *pid == id)
            .map_or(rect, |(_, r)| *r);
        let (cx, cy) = (rect.x + rect.w * 0.5, rect.y + rect.h * 0.5);
        assert_eq!(
            host.hit_at(cx, cy),
            Some(id),
            "`{name}` é pintado mas outra coisa é dona dos pixels no centro dele"
        );
        let events = host.click_at(cx, cy);
        assert!(
            events
                .iter()
                .any(|e| matches!(e, WidgetEvent::Click(c) if *c == id)),
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

/// **Cada chip de material arma o SEU material — e o primeiro arma o rig.**
///
/// ⚠️ O oráculo por-chip, e não *"saiu um intent"*: a lista tem um deslocamento
/// (a opção `0` é o rig, que **não** é um matcap), e um `- 1` mal posto ligaria
/// todo chip ao material anterior. Um gate que só olhasse o primeiro e o último
/// ficaria verde sobre isso.
#[test]
fn every_matcap_chip_arms_its_own_material() {
    for (i, &id) in ids::SCULPT3D_MATCAP.iter().enumerate() {
        let base = Sculpt3dUi::default();
        let (mut host, mut state) = arrange(base);
        let outcome = host.apply_panel_event::<Sculpt3dPanel>(&mut state, WidgetEvent::Click(id));
        assert_eq!(outcome, EventOutcome::Consumed, "o chip {i} não despacha");
        let Sculpt3dIntent::SetUi(got) = only_intent("matcap") else {
            panic!("o chip {i} enfileirou o tipo errado de intent");
        };
        let want = match i {
            0 => None,
            k => Some(u8::try_from(k - 1).expect("cabe")),
        };
        assert_eq!(got.matcap, want, "o chip {i} armou {:?}", got.matcap);
        assert_eq!(
            got,
            Sculpt3dUi {
                matcap: want,
                ..base
            },
            "o chip {i} mexeu num campo que não é dele"
        );
    }
}

/// **Cada chip de padrão arma o SEU padrão, e o primeiro arma o pincel LISO.**
///
/// ⚠️ **A contagem é afirmada aqui, e não em prosa:** a fileira tem
/// `Alpha::ALL.len() + 1` ids, e o `+ 1` é o *None* — que **não** é um padrão. Um
/// chip a mais pinta uma opção que o motor não tem (o `event` a prende no
/// último, e o artista vê o padrão errado ao pedir um que não existe); um a menos
/// deixa um padrão **inalcançável**. As duas falhas são silenciosas.
#[test]
fn every_alpha_chip_arms_its_own_pattern() {
    assert_eq!(
        ids::SCULPT3D_ALPHA.len(),
        Alpha::ALL.len() + 1,
        "{} chips para {} padrões + o pincel liso",
        ids::SCULPT3D_ALPHA.len(),
        Alpha::ALL.len()
    );
    for (i, &id) in ids::SCULPT3D_ALPHA.iter().enumerate() {
        let base = Sculpt3dUi::default();
        let (mut host, mut state) = arrange(base);
        let outcome = host.apply_panel_event::<Sculpt3dPanel>(&mut state, WidgetEvent::Click(id));
        assert_eq!(outcome, EventOutcome::Consumed, "o chip {i} não despacha");
        let Sculpt3dIntent::SetUi(got) = only_intent("alpha") else {
            panic!("o chip {i} enfileirou o tipo errado de intent");
        };
        let want = i.checked_sub(1).map(|k| Alpha::ALL[k]);
        assert_eq!(
            got.brush.alpha, want,
            "o chip {i} armou {:?}",
            got.brush.alpha
        );
        let mut expected = base;
        expected.brush.alpha = want;
        // ⚠️ **Armar um padrão SEMEIA a escala do modelo** — e o chip `None` não,
        // porque não há padrão cujo tamanho medir. As duas metades no mesmo gate
        // de propósito: um seed que disparasse sempre poria um número de escala
        // num pincel liso, e um que nunca disparasse é o defeito que o smoke
        // reprovou (*"os poros são gigantescos"*).
        if want.is_some() {
            expected.brush.alpha_scale = ALPHA_SEED;
        }
        assert_eq!(got, expected, "o chip {i} mexeu num campo que não é dele");
    }
}

/// **O seed é um DEFAULT, não uma política: ele não pisa na escolha do artista.**
///
/// A mesma lei do `arm_inflate_defaults` do Painter e do default de força por
/// verbo — e sem esta metade o artista perderia o número dele toda vez que
/// trocasse de padrão, que é pior que não semear.
#[test]
fn seeding_the_alpha_scale_never_overwrites_a_chosen_one() {
    let mut ui = Sculpt3dUi::default();
    ui.brush.alpha_scale = 0.123;
    let (mut host, mut state) = arrange(ui);
    let _ = host
        .apply_panel_event::<Sculpt3dPanel>(&mut state, WidgetEvent::Click(ids::SCULPT3D_ALPHA[2]));
    let Sculpt3dIntent::SetUi(got) = only_intent("alpha") else {
        panic!("tipo errado de intent");
    };
    assert!(
        (got.brush.alpha_scale - 0.123).abs() < 1e-6,
        "o seed pisou na escala escolhida: {}",
        got.brush.alpha_scale
    );
}

/// **A pista de escala do alpha SOME sem padrão armado.**
///
/// Ela mede o tamanho de uma feature, e sem padrão não há feature — é a mesma
/// lei das duas pistas de lâmpada sob um matcap, e a mesma razão: uma row
/// condicional é **pulada**, nunca pintada apagada, porque um controle que
/// desenha e não responde mente sobre o que o pincel faz.
#[test]
fn the_alpha_scale_row_is_absent_without_a_pattern() {
    for (alpha, want) in [
        (None, false),
        (Some(Alpha::Noise), true),
        (Some(Alpha::Cracks), true),
    ] {
        let mut ui = Sculpt3dUi::default();
        ui.brush.alpha = alpha;
        let (mut host, mut state) = arrange(ui);
        let painted = host.paint::<Sculpt3dPanel>(&mut state, VIEWPORT);
        for id in [ids::SCULPT3D_ALPHA_SCALE, ids::SCULPT3D_ALPHA_SCALE_NUM] {
            assert_eq!(
                painted.iter().any(|(pid, _)| *pid == id),
                want,
                "com alpha {alpha:?} a pista de escala devia {}",
                if want { "estar lá" } else { "sumir" }
            );
        }
    }
}

/// **AS DUAS PISTAS DO EIXO SÓ EXISTEM COM UM PADRÃO DIRECIONAL.**
///
/// Três estados e não dois, e é o do meio que carrega o gate: **sem padrão**
/// nenhum eixo faz sentido · com um **isotrópico** o eixo não move um bit (há
/// gate no motor provando: os seis nem olham o frame) · com um **direcional** ele
/// é o controle da wave.
///
/// ⚠️ **O caso isotrópico é o que separa este gate de um `alpha.is_some()`.** Um
/// predicado que só perguntasse *"há padrão?"* pintaria duas pistas mortas sob o
/// Pores — e mortas do jeito pior, porque elas RESPONDEM ao arrasto e não mudam
/// um pixel, que é indistinguível de *"o eixo está quebrado"*.
#[test]
fn the_axis_rows_are_absent_unless_the_pattern_has_a_direction() {
    for (alpha, want) in [
        (None, false),
        (Some(Alpha::Pores), false),
        (Some(Alpha::Noise), false),
        (Some(Alpha::Strata), true),
        (Some(Alpha::Scratches), true),
        (Some(Alpha::Weave), true),
    ] {
        let mut ui = Sculpt3dUi::default();
        ui.brush.alpha = alpha;
        let (mut host, mut state) = arrange(ui);
        let painted = host.paint::<Sculpt3dPanel>(&mut state, VIEWPORT);
        for id in [
            ids::SCULPT3D_ALPHA_AZ,
            ids::SCULPT3D_ALPHA_AZ_NUM,
            ids::SCULPT3D_ALPHA_ELEV,
            ids::SCULPT3D_ALPHA_ELEV_NUM,
        ] {
            assert_eq!(
                painted.iter().any(|(pid, _)| *pid == id),
                want,
                "com alpha {alpha:?} as pistas de eixo deviam {}",
                if want { "estar lá" } else { "sumir" }
            );
        }
    }
}

/// **O wireframe alterna, e não toca em mais nada.**
#[test]
fn the_wireframe_toggle_flips_only_the_view() {
    for before in [false, true] {
        let base = Sculpt3dUi {
            wireframe: before,
            ..Sculpt3dUi::default()
        };
        let (mut host, mut state) = arrange(base);
        host.apply_panel_event::<Sculpt3dPanel>(
            &mut state,
            WidgetEvent::Click(ids::SCULPT3D_WIREFRAME),
        );
        let Sculpt3dIntent::SetUi(got) = only_intent("wireframe") else {
            panic!("o wireframe enfileirou o tipo errado de intent");
        };
        assert_eq!(
            got,
            Sculpt3dUi {
                wireframe: !before,
                ..base
            },
            "o wireframe não alternou, ou levou um vizinho junto"
        );
    }
}

/// **As duas pistas de LÂMPADA somem sob um matcap** — e estão lá sob o rig.
///
/// ⚠️ Um matcap é sombreamento função apenas da normal de vista: ele não lê o
/// rig, por definição. As duas metades são um gate só de propósito — a de
/// presença sozinha ficaria verde com o `show` cravado em `true`, e a de
/// ausência sozinha ficaria verde com ele cravado em `false`.
#[test]
fn the_lamp_rows_are_absent_under_a_matcap_and_present_under_the_rig() {
    let lamps = [ids::SCULPT3D_LIGHT_AZ, ids::SCULPT3D_LIGHT_ELEV];
    for (matcap, want) in [(None, true), (Some(0), false), (Some(3), false)] {
        let (mut host, mut state) = arrange(Sculpt3dUi {
            matcap,
            ..Sculpt3dUi::default()
        });
        let painted = host.paint::<Sculpt3dPanel>(&mut state, VIEWPORT);
        for id in lamps {
            assert_eq!(
                painted.iter().any(|(pid, _)| *pid == id),
                want,
                "com matcap {matcap:?} a pista de lâmpada devia {}",
                if want { "estar lá" } else { "sumir" }
            );
        }
    }
}

/// **O INTERRUPTOR DO PREVIEW NO BARRO só existe com padrão armado, e ALTERNA.**
///
/// ⚠️ As duas metades num gate só, e a de AUSÊNCIA é a que carrega peso: sem
/// padrão ele seria um interruptor de coisa nenhuma — o mesmo mecanismo da pista
/// de escala, e a mesma lei do módulo (*uma row condicional é PULADA, nunca
/// pintada apagada*).
#[test]
fn the_model_preview_switch_exists_only_with_a_pattern_and_flips_it() {
    // SEM padrão: não é pintado.
    let (mut host, mut state) = arrange(Sculpt3dUi::default());
    let painted = host.paint::<Sculpt3dPanel>(&mut state, VIEWPORT);
    assert!(
        !painted
            .iter()
            .any(|(pid, _)| *pid == ids::SCULPT3D_ALPHA_PREVIEW),
        "o interruptor apareceu sem padrão armado"
    );

    // COM padrão: pintado, e o clique alterna só ele.
    for before in [false, true] {
        let mut ui = Sculpt3dUi::default();
        ui.brush.alpha = Some(ph2d_sculpt3d::Alpha::ALL[0]);
        ui.alpha_preview = before;
        let (mut host, mut state) = arrange(ui);
        let painted = host.paint::<Sculpt3dPanel>(&mut state, VIEWPORT);
        assert!(
            painted
                .iter()
                .any(|(pid, _)| *pid == ids::SCULPT3D_ALPHA_PREVIEW),
            "o interruptor sumiu com um padrão armado"
        );
        host.apply_panel_event::<Sculpt3dPanel>(
            &mut state,
            WidgetEvent::Click(ids::SCULPT3D_ALPHA_PREVIEW),
        );
        let Sculpt3dIntent::SetUi(got) = only_intent("preview no barro") else {
            panic!("o preview enfileirou o tipo errado de intent");
        };
        let mut want = ui;
        want.alpha_preview = !before;
        assert_eq!(got, want, "o interruptor não alternou, ou levou um vizinho");
    }
}

/// **O ACCUMULATE é oferecido — e SÓ — onde ele faz alguma coisa.**
///
/// ⚠️ As duas metades num gate só, e a seleção é pelo GRIP e não por
/// `accumulates()`: filtrar pela função sob teste esvaziaria o laço no dia em
/// que ela mentisse, e o gate passaria sobre nada. Quem tem âncora carrega o
/// gesto TOTAL desde o pen-down — um interruptor de somar ali seria um controle
/// que aparece e não muda um vértice.
#[test]
fn the_accumulate_switch_is_offered_only_where_it_does_something() {
    for verb in Verb::ALL {
        let mut ui = Sculpt3dUi::default();
        ui.brush.verb = verb;
        let (mut host, mut state) = arrange(ui);
        let painted = host.paint::<Sculpt3dPanel>(&mut state, VIEWPORT);
        let stamps = matches!(verb.grip(), ph2d_sculpt3d::Grip::Stamp);
        assert_eq!(
            painted
                .iter()
                .any(|(pid, _)| *pid == ids::SCULPT3D_ACCUMULATE),
            stamps,
            "com {verb:?} o interruptor devia {}",
            if stamps { "estar lá" } else { "sumir" }
        );
    }
}

/// **E ele alterna o campo do PINCEL, não um estado paralelo.**
#[test]
fn the_accumulate_switch_flips_the_brush_field() {
    for before in [false, true] {
        let mut ui = Sculpt3dUi::default();
        ui.brush.accumulate = before;
        let (mut host, mut state) = arrange(ui);
        host.apply_panel_event::<Sculpt3dPanel>(
            &mut state,
            WidgetEvent::Click(ids::SCULPT3D_ACCUMULATE),
        );
        let Sculpt3dIntent::SetUi(got) = only_intent("accumulate") else {
            panic!("o accumulate enfileirou o tipo errado de intent");
        };
        let mut want = ui;
        want.brush.accumulate = !before;
        assert_eq!(got, want, "o accumulate não alternou, ou levou um vizinho");
    }
}

/// **AS DUAS PISTAS DO SSS EXISTEM** — por ID, e não por iteração da tabela.
///
/// ⚠️ **Este gate nasceu de uma mutação SOBREVIVENTE, e ela expôs um oráculo
/// AUTO-REFERENTE.** Toda a varredura deste arquivo faz `for row in rows()` — o
/// que é certo para *"cada row registrada é clicável"* e **cego** para *"esta row
/// existe"*: apagar a linha do `Scatter` da tabela encolhe a lista que o gate
/// percorre, e os 21 testes ficam VERDES sobre um slider que sumiu da tela. É a
/// mesma forma que a `line/Painter` documentou (*"encolher o array encolhe a
/// lista que ele itera"*).
///
/// A cura é perguntar pelo ID, que é um nome que a tabela não pode encolher.
#[test]
fn the_two_subsurface_tracks_are_on_the_table() {
    let by_id = |id| rows::rows().any(|r| r.slider == id);
    assert!(
        by_id(ids::SCULPT3D_SSS),
        "a pista da FORCA do espalhamento sumiu da tabela"
    );
    assert!(
        by_id(ids::SCULPT3D_SSS_SCATTER),
        "a pista do ALCANCE sumiu da tabela — e ela e' o numero que decide o LOOK"
    );
}

/// **A pista do ALCANCE só existe com o espalhamento LIGADO.**
///
/// ⚠️ Com a força em zero a tabela do SSS nem é consultada, então este slider não
/// moveria um pixel — e um controle que não faz nada é o que esta casa varre a
/// cada wave. A metade oposta importa igual: com o canal ligado ele **tem** de
/// aparecer, senão o artista fica sem o número que decide o look.
#[test]
fn the_scatter_track_follows_the_channel_it_belongs_to() {
    let row = rows::rows()
        .find(|r| r.slider == ids::SCULPT3D_SSS_SCATTER)
        .expect("a pista do alcance esta' na tabela");
    // ⚠️ O canal DESLIGADO é o default, e a fixture o declara em vez de o herdar:
    // uma fixture que chega ao estado por omissão inverte de sentido no dia em
    // que o default se move, e segue verde testando o oposto.
    let mut ui = Sculpt3dUi {
        sss: 0.0,
        ..Sculpt3dUi::default()
    };
    assert!(
        !(row.show)(&ui),
        "com o espalhamento DESLIGADO o alcance seria um slider inerte"
    );
    ui.sss = 0.5;
    assert!(
        (row.show)(&ui),
        "com o espalhamento LIGADO o alcance TEM de estar a' mao"
    );
}

/// **O BOTÃO DE ASSAR NO SPRITE existe COM e SEM alvo — o que some é a dica.**
///
/// ⚠️ **É a decisão inteira desta wave, e ela vai contra o reflexo desta casa.**
/// A regra local é *oferecer só quando o gesto leva a algum lugar* (o "Join
/// Selected Bodies" da física; o Filter Layer do Painter), e aqui ela seria
/// exatamente errada: o gesto tinha uma porta só — o atalho `Shift+B` — e a
/// queixa que o botão veio resolver é que **ninguém sabia que ele existia**. Um
/// botão que só aparece para quem já preparou a cena é invisível para quem ainda
/// não sabe que precisa preparar.
///
/// O que responde ao artista é a DICA, no molde do `ao_stale`: a condição é
/// DITA, e a linha só existe quando há o que avisar.
///
/// ⚠️ **A dica em si não tem oráculo de unidade, e isto está declarado em vez de
/// disfarçado:** ela é um `readout` — texto sem `NodeId` —, e a única grandeza
/// que o harness expõe são os rects dos widgets REGISTRADOS. Um proxy (a altura
/// do painel, a posição do vizinho de baixo) expiraria na primeira linha nova, e
/// esta casa já pagou por âncoras assim. O que este gate prova é a metade que
/// decide: **o botão não desaparece**. O texto é do smoke.
#[test]
fn the_bake_button_survives_having_no_sprite_selected() {
    for has_target in [true, false] {
        let (mut host, mut state) = arrange_with(snapshot(Sculpt3dUi::default(), has_target));
        let painted = host.paint::<Sculpt3dPanel>(&mut state, VIEWPORT);
        let rect = painted
            .iter()
            .find(|(id, _)| *id == ids::SCULPT3D_BAKE_SPRITE)
            .map(|(_, r)| *r)
            .unwrap_or_else(|| {
                panic!("com alvo={has_target} o botao de assar no sprite nao foi pintado")
            });
        // E vivo sob o mouse no PRÓPRIO centro — a metade que separa
        // *hit-registrado* de *responde*.
        let evs = host.click_at(rect.x + rect.w * 0.5, rect.y + rect.h * 0.5);
        assert!(
            evs.iter()
                .any(|e| matches!(e, WidgetEvent::Click(id) if *id == ids::SCULPT3D_BAKE_SPRITE)),
            "com alvo={has_target} o botao esta pintado e morto sob o mouse"
        );
    }
}

/// **A dica tem tradução.** Um `tr` de chave desconhecida devolve a própria
/// chave, então um rótulo esquecido chega à tela como `panel.sculpt3d.…` —
/// pintado, legível, e errado.
#[test]
fn the_bake_labels_are_translated() {
    for key in [
        "panel.sculpt3d.section.bake",
        "panel.sculpt3d.bake_sprite",
        "panel.sculpt3d.bake_sprite.hint",
    ] {
        assert_ne!(
            ph2d_i18n::tr(key),
            key,
            "`{key}` nao tem traducao e chegaria a tela como a propria chave"
        );
    }
}
