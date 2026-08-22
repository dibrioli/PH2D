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
    RetopoMode, Sculpt3dIntent, Sculpt3dPanel, Sculpt3dPanelState, Sculpt3dSnapshot, Sculpt3dUi,
    UiLevel, drain_intents, ids, rows, set_current_sculpt3d,
};
use ph2d_sculpt3d::{Alpha, Falloff, FilterKind, RefMode, TransformKind, Verb, kelvinlet::Scales};
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
        alpha_image_name: None,
        // ⚠️ **DESARMADO e' o caso comum**, e a fixture o declara em vez de o
        // herdar: um gate que chegasse ao estado armado por toggle inverteria de
        // sentido no dia em que o default se movesse, e seguiria verde testando
        // o oposto. E o gate do transform arma o outro.
        transform: None,
        // ⚠️ **DESARMADO pelo mesmo motivo do vizinho acima**, e o gate do
        // filtro arma o outro.
        filter_armed: false,
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
    intents[0].clone()
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
        let (mut host, mut state) = arrange(base.clone());

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

/// **Escolher um verbo traz o pincel DAQUELE verbo — e a afinação do anterior
/// fica com o anterior.**
///
/// ⚠️ **Este gate afirmava o OPOSTO até 2026-08-17** (*"pegar um verbo APAGOU a
/// força que o artista tinha escolhido"*), e ele estava fiel à lei de então: a
/// troca re-armava campo a campo *"se o artista ainda não mexeu"*, então uma
/// força autorada atravessava para o verbo novo. Era exactamente o report do
/// Enio — *"as configurações de cada tool não devem se propagar para outra"*.
///
/// ⚠️ **E ele roda pelo CLIQUE REAL, que é o que o irmão de unidade não faz:**
/// o `tests/verb_slots.rs` dirige a porta (`switch_verb`) direto; aqui o
/// caminho inteiro é exercitado — o id do chip, o roteador, o intent. Um dos
/// dois pode ficar verde sobre o outro quebrado.
#[test]
fn picking_a_verb_brings_that_verbs_brush_and_leaves_the_previous_tuning_behind() {
    let base = Sculpt3dUi::default();
    assert!(
        (base.brush.strength - base.brush.verb.default_strength()).abs() < 1e-6,
        "a fixture tem de começar NO default, senão ela testa o outro ramo"
    );
    let idx = |v: Verb| {
        Verb::ALL
            .iter()
            .position(|x| *x == v)
            .expect("o verbo está no ALL")
    };
    let click = |ui: Sculpt3dUi, v: Verb| -> Sculpt3dUi {
        let (mut host, mut state) = arrange(ui);
        host.apply_panel_event::<Sculpt3dPanel>(
            &mut state,
            WidgetEvent::Click(ids::SCULPT3D_VERB[idx(v)]),
        );
        let Sculpt3dIntent::SetUi(got) = only_intent("verbo") else {
            panic!("intent errado")
        };
        got
    };

    // 1. Um verbo nunca tocado traz o default DELE.
    let got = click(base.clone(), Verb::Mask);
    assert!(
        (got.brush.strength - Verb::Mask.default_strength()).abs() < 1e-6,
        "pegar a máscara não trouxe a força cheia dela — ela protegeria pela \
         metade e o barro se moveria por baixo"
    );

    // 2. Uma força autorada no Draw NÃO atravessa para a máscara...
    let authored = Sculpt3dUi {
        brush: ph2d_sculpt3d::Brush {
            strength: 0.123,
            ..base.brush.clone()
        },
        ..base
    };
    let got = click(authored, Verb::Mask);
    assert!(
        (got.brush.strength - Verb::Mask.default_strength()).abs() < 1e-6,
        "a força do Draw atravessou para a máscara: veio {}",
        got.brush.strength
    );

    // 3. ...e ela está lá quando o artista VOLTA. Sem esta metade, um `switch`
    // que jogasse o slot fora passaria pelas duas de cima.
    let back = click(got, Verb::Draw);
    assert!(
        (back.brush.strength - 0.123).abs() < 1e-6,
        "o Draw esqueceu a força que o artista lhe deu: veio {}",
        back.brush.strength
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
        (ids::SCULPT3D_FLATTEN, Sculpt3dIntent::Flatten),
        (ids::SCULPT3D_REMESH, Sculpt3dIntent::Remesh),
        (ids::SCULPT3D_QUAD_REMESH, Sculpt3dIntent::QuadRemesh),
        (ids::SCULPT3D_CLOSE_HOLES, Sculpt3dIntent::CloseHoles),
        (ids::SCULPT3D_DUPLICATE, Sculpt3dIntent::Duplicate),
        (ids::SCULPT3D_DELETE, Sculpt3dIntent::Delete),
        (ids::SCULPT3D_ISOLATE, Sculpt3dIntent::ToggleIsolate),
        (ids::SCULPT3D_MERGE, Sculpt3dIntent::Merge),
        (ids::SCULPT3D_BAKE_AO, Sculpt3dIntent::BakeAo),
        (ids::SCULPT3D_BAKE_SPRITE, Sculpt3dIntent::BakeToSprite),
        (ids::SCULPT3D_ALPHA_SPRITE, Sculpt3dIntent::AlphaFromSprite),
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
    // ⚠️ **E em PRO, pela MESMA frase, uma wave depois.** As rows de Pro são
    // puladas em Basic — que é o default —, então a varredura passaria por cima
    // do Falloff, do Plane Offset, do Pinch e da Dureza e ficaria verde sobre
    // quatro controles que nunca foram clicados. *A fixture tem de conter o
    // fenômeno* — a quarta vez que este arquivo escreve isto.
    ui.ui_level = UiLevel::Pro;
    // ⚠️ **E o modo `L` armado, pela MESMA frase, uma wave depois — a QUINTA.**
    // A fileira da largura do campo só é pintada onde o verbo declara um campo
    // elástico (`RefMode::field(verb).is_some()`), e no `S` de fábrica ela não
    // é: a varredura passaria por cima de três chips que nunca foram clicados.
    // O modo é escrito na TABELA DO VERBO e re-resolvido, que é a porta que o
    // roteador usa — pôr `ui.brush.mode` direto seria a segunda porta que este
    // arquivo já documenta no `event.rs`.
    ui.set_mode_of(Verb::Crease, RefMode::L);
    ph2d_panel_sculpt3d::state::switch_verb(&mut ui, Verb::Crease);
    let (mut host, mut state) = arrange(ui.clone());
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
        // ⚠️ **`visible` e não `show`:** as duas perguntas são independentes
        // (*este pincel a lê?* × *este nível a oferece?*) e o pintor faz as duas
        // por uma porta só. Perguntar só a primeira aqui exigiria em PRO rows
        // que o painel não desenha, e em BASIC deixaria as de Pro fora da conta.
        if row.visible(&ui) {
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
    // ⚠️ **Só os OFERECIDOS**, e o id vem da posição no `RefMode::ALL` — a
    // varredura pergunta ao motor exatamente como o pintor pergunta, senão ela
    // exigiria um chip do `L` que o painel não desenha (e o gate ficaria
    // vermelho sobre um produto correto).
    for m in RefMode::offered_for(ui.brush.verb) {
        want.push((
            format!("ref {}", m.label()),
            ids::SCULPT3D_REF_MODE[m as usize],
        ));
    }
    want.push(("ref apply to all".to_string(), ids::SCULPT3D_REF_MODE_ALL));
    // ⚠️ **A MESMA porta que o pintor pergunta**, e não `Verb::Crease` escrito à
    // mão: uma lista de verbos aqui apodrece no dia em que um sexto verbo passar
    // a declarar campo, e o gate ficaria verde sobre uma fileira que ele não
    // varre.
    if ui.brush.mode.field(ui.brush.verb).is_some() {
        for (i, sc) in Scales::ALL.into_iter().enumerate() {
            want.push((
                format!("field width {}", sc.label()),
                ids::SCULPT3D_ELASTIC_SCALES[i],
            ));
        }
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
    // ⚠️ **PRO nas DUAS metades, e é o que torna o gate honesto.** As duas rows
    // são de Pro, então em Basic a metade negativa passaria pelo motivo ERRADO
    // (escondidas pelo NÍVEL, não pelo verbo) — um gate que não pode falhar pela
    // razão que alega. Fixando o nível, o que sobra a variar é o verbo.
    ui.ui_level = UiLevel::Pro;
    let (mut host, mut state) = arrange(ui.clone());
    let painted = host.paint::<Sculpt3dPanel>(&mut state, VIEWPORT);
    for id in [ids::SCULPT3D_PLANE_OFFSET, ids::SCULPT3D_PINCH] {
        assert!(
            !painted.iter().any(|(pid, _)| *pid == id),
            "{id:?} foi pintado com o Smooth em mãos — um knob que o verbo não lê"
        );
    }
    // E o controle: com o verbo que os lê, eles aparecem.
    ui.brush.verb = Verb::Clay;
    ui.ui_level = UiLevel::Pro;
    let (mut host, mut state) = arrange(ui.clone());
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
        let (mut host, mut state) = arrange(base.clone());
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
/// `Alpha::ALL.len() + 2` ids, e os DOIS a mais não são padrões — o primeiro é o
/// *None* (o pincel liso) e o último é o slot de IMAGEM. Um chip a mais pinta
/// uma opção que o motor não tem (o `event` a prende no último, e o artista vê o
/// padrão errado ao pedir um que não existe); um a menos deixa um padrão
/// **inalcançável**. As duas falhas são silenciosas.
///
/// ⚠️ **O laço para ANTES do último de propósito, e o irmão abaixo o cobre:** o
/// chip da imagem não enfileira um `SetUi` — ele não teria o que armar, porque
/// os pixels vivem na CENA e não no retrato. Varrê-lo aqui exigiria um `if` no
/// meio do laço, e um laço com uma exceção é como o décimo-segundo chip nasce
/// sem gate.
#[test]
fn every_alpha_chip_arms_its_own_pattern() {
    assert_eq!(
        ids::SCULPT3D_ALPHA.len(),
        Alpha::ALL.len() + 2,
        "{} chips para {} padrões + o pincel liso + o slot de imagem",
        ids::SCULPT3D_ALPHA.len(),
        Alpha::ALL.len()
    );
    for (i, &id) in ids::SCULPT3D_ALPHA
        .iter()
        .enumerate()
        .take(Alpha::ALL.len() + 1)
    {
        let base = Sculpt3dUi::default();
        let (mut host, mut state) = arrange(base.clone());
        let outcome = host.apply_panel_event::<Sculpt3dPanel>(&mut state, WidgetEvent::Click(id));
        assert_eq!(outcome, EventOutcome::Consumed, "o chip {i} não despacha");
        let Sculpt3dIntent::SetUi(got) = only_intent("alpha") else {
            panic!("o chip {i} enfileirou o tipo errado de intent");
        };
        let want = i.checked_sub(1).map(|k| Alpha::ALL[k].clone());
        assert_eq!(
            got.brush.alpha, want,
            "o chip {i} armou {:?}",
            got.brush.alpha
        );
        let mut expected = base.clone();
        expected.brush.alpha = want.clone();
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
    let (mut host, mut state) = arrange(ui.clone());
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
        ui.brush.alpha = alpha.clone();
        let (mut host, mut state) = arrange(ui.clone());
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
        ui.brush.alpha = alpha.clone();
        let (mut host, mut state) = arrange(ui.clone());
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
        let (mut host, mut state) = arrange(base.clone());
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

/// **AS ROWS QUE LEEM O RIG somem sob um matcap** — e estão lá sob o rig.
///
/// ⚠️ Um matcap é sombreamento função apenas da normal de vista: ele não lê o
/// rig, por definição. As duas metades são um gate só de propósito — a de
/// presença sozinha ficaria verde com o `show` cravado em `true`, e a de
/// ausência sozinha ficaria verde com ele cravado em `false`.
///
/// ⚠️ **O AMBIENTE entrou nesta lista e não ganhou gate próprio**, e a razão é
/// que a lei é *a mesma*: um matcap **já É um ambiente** — uma esfera de
/// iluminação capturada, de onde saem o piso, o céu e o realce de uma vez —,
/// então o termo do estúdio não entra naquele caminho e o slider seria um
/// controle que não faz nada. Um segundo gate aqui seria a segunda cópia de
/// *"esta row lê o rig"*, e ele divergiria no dia em que a terceira row entrasse
/// só numa das duas listas.
#[test]
fn the_rows_that_read_the_rig_vanish_under_a_matcap() {
    let lamps = [
        ids::SCULPT3D_LIGHT_AZ,
        ids::SCULPT3D_LIGHT_ELEV,
        ids::SCULPT3D_ENV,
    ];
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
                "com matcap {matcap:?} a row {id:?} devia {}",
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
        ui.brush.alpha = Some(ph2d_sculpt3d::Alpha::ALL[0].clone());
        ui.alpha_preview = before;
        let (mut host, mut state) = arrange(ui.clone());
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
        let (mut host, mut state) = arrange(ui.clone());
        let painted = host.paint::<Sculpt3dPanel>(&mut state, VIEWPORT);
        // ⚠️ **A porta é `Verb::accumulates()`, e não o GRIP** — a segunda vez
        // que este repo paga a mesma lição na mesma janela (o
        // `stroke_apply_tests` a pagou com o `unit_accum`). `Grip::Stamp` diz
        // *que gesto é este*; quem responde *este verbo lê o interruptor?* é a
        // porta, e a demão é um carimbo que **não** o lê (o `layer.cc` mede
        // contra o `orig` incondicionalmente). Enquanto os dois concordassem, o
        // gate era verde por acidente.
        let offers = verb.accumulates();
        assert_eq!(
            painted
                .iter()
                .any(|(pid, _)| *pid == ids::SCULPT3D_ACCUMULATE),
            offers,
            "com {verb:?} o interruptor devia {}",
            if offers { "estar lá" } else { "sumir" }
        );
    }
}

/// **E ele alterna o campo do PINCEL, não um estado paralelo.**
#[test]
fn the_accumulate_switch_flips_the_brush_field() {
    for before in [false, true] {
        let mut ui = Sculpt3dUi::default();
        ui.brush.accumulate = before;
        let (mut host, mut state) = arrange(ui.clone());
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

/// **O FRONT FACES ONLY é oferecido — e SÓ — onde a LEI existe.**
///
/// ⚠️ **A varredura é por MODO e não por verbo**, e é essa a pergunta: a lei
/// (`FrontFace`) é do modo de referência, o interruptor é do pincel, e o
/// Blender tem as duas metades pela mesma razão (`calc_front_face` existe
/// sempre; o `if (brush.flag & BRUSH_FRONTFACE)` decide se corre).
///
/// ⚠️ **Ele não é uma `Row`** (é um toggle, não um número), então a varredura
/// genérica deste arquivo é cega a ele — sem este gate, apagar a metade que o
/// pinta deixa os outros verdes.
#[test]
fn the_front_face_switch_is_offered_only_where_the_law_exists() {
    let mut seen = (false, false);
    for verb in Verb::ALL {
        for mode in RefMode::ALL {
            let mut ui = Sculpt3dUi::default();
            ui.brush.verb = verb;
            ui.brush.mode = mode;
            let offers = ui.brush.offers_front_faces();
            let (mut host, mut state) = arrange(ui.clone());
            let painted = host.paint::<Sculpt3dPanel>(&mut state, VIEWPORT);
            assert_eq!(
                painted
                    .iter()
                    .any(|(pid, _)| *pid == ids::SCULPT3D_FRONT_FACES),
                offers,
                "com {verb:?} em {mode:?} o interruptor devia {}",
                if offers { "estar lá" } else { "sumir" }
            );
            if offers {
                seen.0 = true;
            } else {
                seen.1 = true;
            }
        }
    }
    // ⚠️ **O CONTROLE das duas pontas:** um `offers` constante deixaria o laço
    // acima verde afirmando nada.
    assert!(
        seen.0 && seen.1,
        "a varredura não achou os dois casos: {seen:?}"
    );
}

/// **E ele alterna o campo do PINCEL, não um estado paralelo.**
#[test]
fn the_front_face_switch_flips_the_brush_field() {
    for before in [false, true] {
        let mut ui = Sculpt3dUi::default();
        // ⚠️ **O modo tem de DECLARAR a lei**, senão o roteador recusa o clique
        // (e com razão) e o gate mediria a recusa em vez do interruptor.
        ui.brush.mode = RefMode::B;
        ui.brush.front_faces_only = before;
        assert!(ui.brush.offers_front_faces(), "a fixture perdeu a premissa");
        let (mut host, mut state) = arrange(ui.clone());
        host.apply_panel_event::<Sculpt3dPanel>(
            &mut state,
            WidgetEvent::Click(ids::SCULPT3D_FRONT_FACES),
        );
        let Sculpt3dIntent::SetUi(got) = only_intent("front_faces") else {
            panic!("o front-face enfileirou o tipo errado de intent");
        };
        let mut want = ui;
        want.brush.front_faces_only = !before;
        assert_eq!(got, want, "o front-face não alternou, ou levou um vizinho");
    }
}

/// **O INTERRUPTOR DA LÂMINA existe, e SÓ com a lâmina em mãos.**
///
/// ⚠️ **Perguntado por ID e não pela tabela** — ele não é uma `Row` (é um
/// toggle, não um número), então a varredura genérica deste arquivo é cega a
/// ele: sem este gate, apagar a metade que o pinta deixa os outros verdes.
#[test]
fn the_dynamic_switch_is_offered_only_with_the_blade_in_hand() {
    for verb in Verb::ALL {
        let mut ui = Sculpt3dUi::default();
        ui.brush.verb = verb;
        let (mut host, mut state) = arrange(ui.clone());
        let painted = host.paint::<Sculpt3dPanel>(&mut state, VIEWPORT);
        assert_eq!(
            painted
                .iter()
                .any(|(pid, _)| *pid == ids::SCULPT3D_SCRAPE_DYNAMIC),
            verb == Verb::MultiplaneScrape,
            "com {verb:?} o interruptor de ler-a-superfície devia {}",
            if verb == Verb::MultiplaneScrape {
                "estar lá"
            } else {
                "sumir"
            }
        );
    }
}

/// **E ele alterna o campo do PINCEL, não um estado paralelo.**
#[test]
fn the_dynamic_switch_flips_the_brush_field() {
    for before in [false, true] {
        let mut ui = Sculpt3dUi::default();
        ui.brush.verb = Verb::MultiplaneScrape;
        ui.brush.scrape_dynamic = before;
        let (mut host, mut state) = arrange(ui.clone());
        host.apply_panel_event::<Sculpt3dPanel>(
            &mut state,
            WidgetEvent::Click(ids::SCULPT3D_SCRAPE_DYNAMIC),
        );
        let Sculpt3dIntent::SetUi(got) = only_intent("scrape_dynamic") else {
            panic!("o modo dinâmico enfileirou o tipo errado de intent");
        };
        let mut want = ui;
        want.brush.scrape_dynamic = !before;
        assert_eq!(
            got, want,
            "o modo dinâmico não alternou, ou levou um vizinho"
        );
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

/// **O botão do alpha por imagem existe SÓ com um sprite selecionado**, e as
/// duas metades estão no mesmo gate.
///
/// ⚠️ **Ausência, não dimming.** Um botão que só pode falhar é como o artista
/// aprende que ele não funciona; é a mesma decisão do "Light the Selected
/// Sprite" logo acima, que troca o botão por uma dica quando não há alvo.
///
/// ⚠️ **E o oráculo do lado presente CLICA**, não só olha: pintar um retângulo
/// e registrá-lo são coisas diferentes de estar vivo sob o mouse — a falha que
/// este arquivo existe para pegar.
#[test]
fn the_pattern_from_sprite_button_needs_a_selected_sprite() {
    for has in [false, true] {
        let (mut host, mut state) = arrange(Sculpt3dUi::default());
        set_current_sculpt3d(Some(Sculpt3dSnapshot {
            ui: Sculpt3dUi::default(),
            has_bake_target: has,
            ..Sculpt3dSnapshot::default()
        }));
        let painted = host.paint::<Sculpt3dPanel>(&mut state, VIEWPORT);
        assert_eq!(
            painted
                .iter()
                .any(|(id, _)| *id == ids::SCULPT3D_ALPHA_SPRITE),
            has,
            "com sprite={has} o botao devia {}",
            if has { "estar la'" } else { "sumir" }
        );
        if has {
            let outcome = host.apply_panel_event::<Sculpt3dPanel>(
                &mut state,
                WidgetEvent::Click(ids::SCULPT3D_ALPHA_SPRITE),
            );
            assert_eq!(
                outcome,
                EventOutcome::Consumed,
                "o botao e' pintado e o clique nao chega ao barramento"
            );
            assert_eq!(only_intent("alpha"), Sculpt3dIntent::AlphaFromSprite);
        }
    }
}

/// **O chip do SLOT DE IMAGEM pede à cena que re-arme o que ela lembra.**
///
/// ⚠️ **Ele é o único chip da fileira que NÃO enfileira um `SetUi`**, e a razão é
/// estrutural: o painel só vê o retrato, e no instante em que o artista escolheu
/// um procedural o `Arc<AlphaImage>` deixou o `Sculpt3dUi`. Sem esta porta o chip
/// seria um controle que só sabe deixar de estar aceso — pintado, hit-registrado
/// e incapaz de voltar.
#[test]
fn the_image_chip_asks_the_scene_to_re_arm_what_it_remembers() {
    let id = *ids::SCULPT3D_ALPHA
        .last()
        .expect("a fileira de padrão não é vazia");
    let (mut host, mut state) = arrange(Sculpt3dUi::default());
    let outcome = host.apply_panel_event::<Sculpt3dPanel>(&mut state, WidgetEvent::Click(id));
    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "o chip da imagem não despacha"
    );
    assert!(
        matches!(only_intent("imagem"), Sculpt3dIntent::ArmStoredImage),
        "o chip da imagem enfileirou o tipo errado de intent — um `SetUi` aqui \
         armaria o padrão de índice errado, ou nada"
    );
}

/// **O chip aceso é o do slot quando uma IMAGEM está armada** — o report
/// *"o painel diz None com um padrão vivo"*.
///
/// ⚠️ **O oráculo é a porta que PINTA** (`alpha_chip_index`), e não uma
/// re-derivação escrita aqui: a aritmética do índice é feita duas vezes no
/// produto (para pintar e para despachar), e um gate com uma terceira cópia
/// concordaria com ele mesmo enquanto as duas do produto divergiam.
#[test]
fn an_armed_image_lights_its_own_chip_not_none() {
    let img = std::sync::Arc::new(
        ph2d_sculpt3d::AlphaImage::from_rgba(2, 2, &[128; 16]).expect("fixture é uma imagem"),
    );
    let mut ui = Sculpt3dUi::default();
    ui.brush.alpha = Some(ph2d_sculpt3d::Alpha::Image(img));
    let mut snap = snapshot(ui, false);
    snap.alpha_image_name = Some(std::sync::Arc::from("Post"));
    assert_eq!(
        ph2d_panel_sculpt3d::alpha_chip_index(&snap),
        Alpha::ALL.len() + 1,
        "uma imagem armada não acende o chip dela — o painel diz «nenhum padrão» \
         com um padrão vivo e um preview desenhado logo abaixo"
    );

    // CONTROLE: sem imagem o chip aceso é o *None*, e continua sendo.
    assert_eq!(
        ph2d_panel_sculpt3d::alpha_chip_index(&snapshot(Sculpt3dUi::default(), false)),
        0,
        "o pincel liso deixou de acender o primeiro chip"
    );
}

/// **O ACHATAR só existe com a pilha MONTADA** — presença E ausência.
///
/// ⚠️ Com um nível ele é um no-op, e um botão que não faz nada é pior que um
/// botão que falta — a mesma lei que esconde as rows de um verbo que não as lê.
/// A metade da AUSÊNCIA é a que carrega o gate: sem ela, um botão morto na
/// pilha de um nível passaria o sweep de clicabilidade e ninguém veria.
#[test]
fn the_flatten_button_exists_only_where_there_is_a_stack() {
    // Com pilha: pintado e clicável onde é desenhado.
    let mut snap = snapshot(Sculpt3dUi::default(), true);
    snap.level_count = 3;
    snap.level = 1;
    let (mut host, mut state) = arrange_with(snap);
    let painted = host.paint::<Sculpt3dPanel>(&mut state, VIEWPORT);
    let rect = painted
        .iter()
        .find(|(id, _)| *id == ids::SCULPT3D_FLATTEN)
        .map(|(_, r)| *r)
        .expect("com a pilha montada o achatar é pintado");
    // ⚠️ **O clique é dirigido pelo PONTEIRO, no próprio centro** — é a metade
    // que separa *hit-registrado* de *responde*, e a que pegaria um id fora do
    // `populate` (o painel de física já pagou esta: 36 células pintadas, com
    // arm, e mortas sob o mouse).
    let evs = host.click_at(rect.x + rect.w * 0.5, rect.y + rect.h * 0.5);
    assert!(
        evs.iter()
            .any(|e| matches!(e, WidgetEvent::Click(id) if *id == ids::SCULPT3D_FLATTEN)),
        "o botão do achatar está pintado e morto sob o mouse"
    );
    for e in evs {
        let _ = host.apply_panel_event::<Sculpt3dPanel>(&mut state, e);
    }
    assert_eq!(
        drain_intents(),
        vec![Sculpt3dIntent::Flatten],
        "o clique no achatar não chegou ao shell"
    );

    // CONTROLE: com um nível só ele não é desenhado.
    let (mut host, mut state) = arrange(Sculpt3dUi::default());
    let painted = host.paint::<Sculpt3dPanel>(&mut state, VIEWPORT);
    assert!(
        !painted.iter().any(|(id, _)| *id == ids::SCULPT3D_FLATTEN),
        "sem pilha o achatar é um no-op, e ele está na tela"
    );
}

/// **CADA MODO OFERECIDO TEM UM CHIP QUE O PEGA — e o chip escreve na tabela do
/// VERBO, não no pincel.**
#[test]
fn every_offered_reference_has_a_chip_that_selects_it_for_that_verb() {
    for verb in [Verb::Draw, Verb::Smooth, Verb::Crease] {
        for want in RefMode::offered_for(verb) {
            let mut ui = Sculpt3dUi::default();
            ph2d_panel_sculpt3d::state::switch_verb(&mut ui, verb);
            let (mut host, mut state) = arrange(ui);
            host.apply_panel_event::<Sculpt3dPanel>(
                &mut state,
                WidgetEvent::Click(ids::SCULPT3D_REF_MODE[want as usize]),
            );
            let Sculpt3dIntent::SetUi(got) = only_intent(want.label()) else {
                panic!("intent errado")
            };
            assert_eq!(got.brush.mode, want, "{} em {}", want.label(), verb.label());
            let i = ph2d_panel_sculpt3d::state::verb_index(verb);
            assert_eq!(got.mode_of(Verb::ALL[i]), want, "a tabela do verbo");
        }
    }
}

/// **A ESCOLHA É POR VERBO, e trocar de ferramenta a TRAZ DE VOLTA.**
///
/// ⚠️ É o gate que separa *"o modo é do pincel"* de *"o modo é da ferramenta"* —
/// sem ele, guardar a escolha só no `Brush` passaria, e o artista a perderia na
/// primeira troca de tool sem nada reclamar.
#[test]
fn the_reference_is_remembered_per_verb_across_a_tool_switch() {
    let mut ui = Sculpt3dUi::default();
    // O Draw vai para `B`; o Smooth fica no default.
    ui.set_mode_of(Verb::Draw, RefMode::B);
    ph2d_panel_sculpt3d::state::switch_verb(&mut ui, Verb::Draw);
    assert_eq!(ui.brush.mode, RefMode::B, "o Draw veste o que a tabela diz");
    ph2d_panel_sculpt3d::state::switch_verb(&mut ui, Verb::Smooth);
    assert_eq!(
        ui.brush.mode,
        RefMode::default(),
        "o Smooth tem escolha PRÓPRIA"
    );
    ph2d_panel_sculpt3d::state::switch_verb(&mut ui, Verb::Draw);
    assert_eq!(ui.brush.mode, RefMode::B, "e a do Draw volta");
}

/// **O CARIMBO leva a referência corrente a TODAS as ferramentas** — um gesto
/// sobre o estado por-verbo, e não um segundo seletor global.
#[test]
fn apply_to_all_stamps_the_current_reference_onto_every_verb() {
    let mut ui = Sculpt3dUi::default();
    ui.set_mode_of(Verb::Draw, RefMode::B);
    ph2d_panel_sculpt3d::state::switch_verb(&mut ui, Verb::Draw);
    let (mut host, mut state) = arrange(ui);
    host.apply_panel_event::<Sculpt3dPanel>(
        &mut state,
        WidgetEvent::Click(ids::SCULPT3D_REF_MODE_ALL),
    );
    let Sculpt3dIntent::SetUi(got) = only_intent("apply to all") else {
        panic!("intent errado")
    };
    assert!(
        Verb::ALL.iter().all(|&v| got.mode_of(v) == RefMode::B),
        "o carimbo tinha de alcançar os dezasseis: {:?}",
        Verb::ALL.map(|v| got.mode_of(v))
    );
}

/// **O CARIMBO SÓ ALCANÇA QUEM DECLARA O MODO — e onde ele não alcança,
/// PRESERVA.**
///
/// ⚠️ **Esta era a única porta capaz de pôr um modo onde ele não tem lei.**
/// Enquanto os três modos respondiam por todo verbo (até a W4) o carimbo era um
/// `fill` e ninguém notava; com o `L` declarando só o Smooth, carimbá-lo em
/// todos deixaria quinze verbos rodando uma [`RefMode::kernel`] de literatura
/// que não fala deles — **e com o chip a mostrar `S`**, porque o painel pinta os
/// OFERECIDOS e o `L` não estaria entre eles. O chip que mente, pela porta de
/// trás.
///
/// ⚠️ **A segunda metade é tão load-bearing quanto a primeira:** onde o carimbo
/// não alcança ele **guarda o que estava lá**, em vez de repor um default. O
/// artista carimbou uma escolha; ele não pediu um reset das que não cabem.
#[test]
fn apply_to_all_only_reaches_the_verbs_that_declare_the_mode() {
    let mut ui = Sculpt3dUi::default();
    // O Draw tem uma escolha DELIBERADA que o carimbo não pode alcançar.
    ui.set_mode_of(Verb::Draw, RefMode::B);
    ui.set_mode_of(Verb::Smooth, RefMode::L);
    ph2d_panel_sculpt3d::state::switch_verb(&mut ui, Verb::Smooth);
    assert_eq!(ui.brush.mode, RefMode::L, "a premissa: o Smooth está no L");
    let (mut host, mut state) = arrange(ui);
    host.apply_panel_event::<Sculpt3dPanel>(
        &mut state,
        WidgetEvent::Click(ids::SCULPT3D_REF_MODE_ALL),
    );
    let Sculpt3dIntent::SetUi(got) = only_intent("apply to all") else {
        panic!("intent errado")
    };
    assert_eq!(got.mode_of(Verb::Smooth), RefMode::L, "onde ele declara");
    assert_eq!(
        got.mode_of(Verb::Draw),
        RefMode::B,
        "onde ele NÃO declara, a escolha do artista fica de pé"
    );
    for (i, m) in Verb::ALL.map(|v| got.mode_of(v)).into_iter().enumerate() {
        assert!(
            RefMode::offered_for(Verb::ALL[i]).any(|o| o == m),
            "{}: ficou com um modo que ele não oferece ({})",
            Verb::ALL[i].label(),
            m.label()
        );
    }
}

// ── Basic × Pro (§2 do plano) ───────────────────────────────────────────────

/// **Cada nível tem um chip que o escolhe.**
#[test]
fn every_ui_level_has_a_chip_that_selects_it() {
    for (i, &want) in UiLevel::ALL.iter().enumerate() {
        // Parte-se sempre do OUTRO, senão o chip do default passaria sem fazer
        // nada e o gate ficaria verde sobre um clique inerte.
        let ui = Sculpt3dUi {
            ui_level: if want == UiLevel::Basic {
                UiLevel::Pro
            } else {
                UiLevel::Basic
            },
            ..Sculpt3dUi::default()
        };
        let (mut host, mut state) = arrange(ui);
        let outcome = host.apply_panel_event::<Sculpt3dPanel>(
            &mut state,
            WidgetEvent::Click(ids::SCULPT3D_UI_LEVEL[i]),
        );
        assert_eq!(outcome, EventOutcome::Consumed, "o chip {i} não despacha");
        let Sculpt3dIntent::SetUi(got) = only_intent("nível") else {
            panic!("intent errado")
        };
        assert_eq!(got.ui_level, want, "o chip {i} armou o nível errado");
    }
}

/// **O chip DIVULGA e nunca DECIDE** — trocar de nível deixa todo o resto do
/// estado autorado byte a byte onde estava.
///
/// ⚠️ **É a propriedade que separa divulgação progressiva de política.** No dia
/// em que alguém fizer o Basic *zerar* um knob que ele esconde (o reflexo
/// natural: *"se não se vê, não deveria agir"*), o artista perderia trabalho
/// autorado ao mudar com que profundidade OLHA — e nenhum outro gate desta
/// suíte veria isso, porque todos fixam o nível.
#[test]
fn the_detail_chip_discloses_and_never_decides() {
    // Um estado com os quatro knobs de Pro LONGE dos defaults: se o nível
    // decidisse alguma coisa, é aqui que apareceria.
    let mut before = Sculpt3dUi::default();
    before.brush.verb = Verb::Crease;
    before.brush.plane_offset = -0.4;
    before.brush.pinch = 0.9;
    before.brush.hardness = 0.6;
    before.brush.falloff = Falloff::Sharper;
    before.ui_level = UiLevel::Basic;

    // ⚠️ **As DUAS direções, e a primeira mutação provou que uma só não basta:**
    // *esconder* é o gesto em que o reflexo de zerar aparece, então um gate que
    // só sobe de Basic para Pro passa sobre um Basic que apaga o que esconde
    // ([[feedback_layered_defenses_need_per_layer_gates]]).
    for (from, to, chip) in [
        (UiLevel::Basic, UiLevel::Pro, 1usize),
        (UiLevel::Pro, UiLevel::Basic, 0usize),
    ] {
        before.ui_level = from;
        let (mut host, mut state) = arrange(before.clone());
        host.apply_panel_event::<Sculpt3dPanel>(
            &mut state,
            WidgetEvent::Click(ids::SCULPT3D_UI_LEVEL[chip]),
        );
        let Sculpt3dIntent::SetUi(after) = only_intent("nível") else {
            panic!("intent errado")
        };
        assert_eq!(
            after.ui_level, to,
            "o nível não mudou de {from:?} para {to:?}"
        );
        // O oráculo é a IGUALDADE do resto: reescrever `after` com o nível de
        // volta tem de devolver exatamente o estado de partida.
        let mut rolled_back = after.clone();
        rolled_back.ui_level = before.ui_level;
        assert_eq!(
            rolled_back, before,
            "ir de {from:?} para {to:?} mexeu em algo que não é o nível"
        );
    }
}

/// **Uma row de Pro é alcançável em Pro e ausente em Basic.**
///
/// ⚠️ **As duas metades, e nenhuma basta:** só a primeira deixaria passar um
/// Basic que não esconde nada (o chip vira decoração); só a segunda deixaria
/// passar uma row que *nunca* é pintada — a affordance morta que esta casa varre
/// a cada wave. E o laço percorre a TABELA, então uma row de Pro nova nasce
/// coberta pelas duas.
#[test]
fn a_pro_row_is_reachable_in_pro_and_absent_in_basic() {
    // ⚠️ **DUAS fixtures, porque as duas rows condicionais de Pro se EXCLUEM
    // por desenho:** o `plane_offset` é dos verbos de plano e o `pinch` é do
    // Crease, então nenhum verbo as tem juntas — uma fixture só varreria duas
    // das três e o `>= 3` seria impossível de satisfazer honestamente.
    let mut seen: Vec<&'static str> = Vec::new();
    for verb in [Verb::Crease, Verb::Clay] {
        let mut ui = Sculpt3dUi::default();
        ui.brush.verb = verb;

        ui.ui_level = UiLevel::Pro;
        let (mut host, mut state) = arrange(ui.clone());
        let in_pro = host.paint::<Sculpt3dPanel>(&mut state, VIEWPORT);
        ui.ui_level = UiLevel::Basic;
        let (mut host, mut state) = arrange(ui.clone());
        let in_basic = host.paint::<Sculpt3dPanel>(&mut state, VIEWPORT);

        for row in rows::rows().filter(|r| r.level == UiLevel::Pro && (r.show)(&ui)) {
            if !seen.contains(&row.label) {
                seen.push(row.label);
            }
            assert!(
                in_pro.iter().any(|(id, _)| *id == row.slider),
                "`{}` é de Pro e o Pro não a pintou com o {} em mãos",
                row.label,
                verb.label()
            );
            assert!(
                !in_basic.iter().any(|(id, _)| *id == row.slider),
                "`{}` é de Pro e o Basic a pintou assim mesmo",
                row.label
            );
        }
    }
    assert!(
        seen.len() >= 3,
        "a fixture tem de conter o fenômeno: só {:?} row(s) de Pro varridas",
        seen
    );
}

/// **O Basic nunca esconde Raio nem Força** — sejam quais forem o verbo e o
/// padrão.
///
/// ⚠️ É a metade da regra que o §2 chama de *amputação*: esconder um knob que
/// alguém armou é divulgação progressiva; esconder os dois que TODO pincel tem
/// deixaria o artista sem ferramenta e sem nada na tela explicando por quê.
#[test]
fn the_basic_level_never_hides_the_two_knobs_every_brush_has() {
    for v in Verb::ALL {
        let mut ui = Sculpt3dUi::default();
        ui.brush.verb = v;
        ui.ui_level = UiLevel::Basic;
        for id in [ids::SCULPT3D_RADIUS, ids::SCULPT3D_STRENGTH] {
            let row = rows::rows().find(|r| r.slider == id).expect("na tabela");
            assert!(
                row.visible(&ui),
                "`{}` sumiu em Basic com o {} em mãos",
                row.label,
                v.label()
            );
        }
    }
}

/// **A DUREZA tem uma row, e ela escreve o campo que o kernel lê.**
///
/// ⚠️ O gate existe porque o knob nasceu no kernel numa wave e ficou **sem
/// porta** — gateado dos dois lados, medido, e inalcançável por qualquer gesto.
/// Um campo sem controle é uma capacidade que ninguém tem.
#[test]
fn the_hardness_row_writes_the_field_the_kernel_reads() {
    let row = rows::rows()
        .find(|r| r.slider == ids::SCULPT3D_HARDNESS)
        .expect("a dureza está na tabela");
    assert_eq!(row.level, UiLevel::Pro, "a dureza é um knob de Pro");
    let mut ui = Sculpt3dUi::default();
    (row.set)(&mut ui, 0.75);
    assert!(
        (ui.brush.hardness - 0.75).abs() < 1e-6,
        "a row da dureza não escreveu `brush.hardness`: {}",
        ui.brush.hardness
    );
    assert!(
        (row.get)(&ui) - 0.75 < 1e-6,
        "e o retrato dela tem de ler o mesmo número"
    );
    // ⚠️ E o TETO é alcançável de propósito: `1` é o disco duro, que o
    // `shaped_distance` trata num braço PRÓPRIO (a fórmula geral divide por
    // `1 − h`). Uma pista que parasse antes o tornaria inexprimível.
    assert!(
        (row.max - 1.0).abs() < 1e-6,
        "a pista da dureza tem de alcançar o disco duro"
    );
}

/// **A LARGURA DO CAMPO É OFERECIDA ONDE O CAMPO CORRE, E EM LUGAR NENHUM
/// MAIS** — e o chip escreve no pincel.
///
/// ⚠️ **As duas metades são portas INDEPENDENTES**, e um gate que só afirmasse a
/// presença ficaria verde com qualquer uma delas removida: a fileira exige que o
/// verbo declare campo (`RefMode::field`) **e** que o nível seja `Pro`. Os dois
/// controles negativos abaixo são o que separa *"a row aparece"* de *"a row
/// aparece pelo motivo certo"*.
///
/// ⚠️ **E a metade do CAMPO é a que importa:** sem ela os três chips existiriam
/// em `S`, onde o `stroke_target` nunca chama o kernel — três botões que não
/// movem um vértice, e o artista descobre isso arrastando.
#[test]
fn the_field_width_row_exists_only_where_the_field_does_and_the_chip_lands() {
    let armed = || {
        let mut ui = Sculpt3dUi {
            ui_level: UiLevel::Pro,
            ..Default::default()
        };
        ui.set_mode_of(Verb::Move, RefMode::L);
        ph2d_panel_sculpt3d::state::switch_verb(&mut ui, Verb::Move);
        assert!(
            ui.brush.mode.field(ui.brush.verb).is_some(),
            "a fixture não contém o fenômeno: o Move em L tem de declarar campo"
        );
        ui
    };

    // Os três chips existem, e cada um pousa a sua família.
    for (i, want) in Scales::ALL.into_iter().enumerate() {
        let (mut host, mut state) = arrange(armed());
        let painted = host.paint::<Sculpt3dPanel>(&mut state, VIEWPORT);
        assert!(
            painted
                .iter()
                .any(|(id, _)| *id == ids::SCULPT3D_ELASTIC_SCALES[i]),
            "o chip {} não está na tela",
            want.label()
        );
        host.apply_panel_event::<Sculpt3dPanel>(
            &mut state,
            WidgetEvent::Click(ids::SCULPT3D_ELASTIC_SCALES[i]),
        );
        let Sculpt3dIntent::SetUi(got) = only_intent(want.label()) else {
            panic!("intent errado")
        };
        assert_eq!(
            got.brush.elastic_scales,
            want,
            "o clique em {} não pousou",
            want.label()
        );
    }

    // CONTROLE 1 — o mesmo verbo em `S`: o campo não corre, a fileira não existe.
    let mut ui = armed();
    ui.set_mode_of(Verb::Move, RefMode::S);
    ph2d_panel_sculpt3d::state::switch_verb(&mut ui, Verb::Move);
    let (mut host, mut state) = arrange(ui);
    let painted = host.paint::<Sculpt3dPanel>(&mut state, VIEWPORT);
    assert!(
        !painted
            .iter()
            .any(|(id, _)| *id == ids::SCULPT3D_ELASTIC_SCALES[0]),
        "sem campo elástico os chips não movem um vértice, e estão na tela"
    );

    // CONTROLE 2 — o campo corre, mas em BASIC a largura é a que o kernel armou.
    let mut ui = armed();
    ui.ui_level = UiLevel::Basic;
    let (mut host, mut state) = arrange(ui);
    let painted = host.paint::<Sculpt3dPanel>(&mut state, VIEWPORT);
    assert!(
        !painted
            .iter()
            .any(|(id, _)| *id == ids::SCULPT3D_ELASTIC_SCALES[0]),
        "a fileira é de Pro e está no Basic"
    );
}

/// **OS DOIS KNOBS DA FAIXA APARECEM COM A FAIXA, E COM MAIS NADA.**
///
/// ⚠️ **Este gate nasceu de DUAS mutações sobreviventes:** tirar as rows da
/// tabela e alargar o `show` para `always` deixavam a suíte do painel inteira
/// VERDE. O primeiro é um motor com knobs que o artista não alcança; o segundo
/// são dois sliders mortos em dezasseis das dezassete ferramentas.
///
/// ⚠️ **Pro nas duas metades**, pela mesma razão que o
/// `a_conditional_row_is_absent_with_the_wrong_tool` já documenta: em Basic a
/// metade negativa passaria pelo motivo ERRADO.
#[test]
fn the_strip_knobs_are_painted_for_the_strip_and_for_nothing_else() {
    let with = |verb: Verb| {
        let mut ui = Sculpt3dUi::default();
        ui.brush.verb = verb;
        ui.ui_level = UiLevel::Pro;
        let (mut host, mut state) = arrange(ui);
        let painted = host.paint::<Sculpt3dPanel>(&mut state, VIEWPORT);
        [ids::SCULPT3D_TIP_ROUNDNESS, ids::SCULPT3D_STRIP_LENGTH]
            .map(|id| painted.iter().any(|(pid, _)| *pid == id))
    };
    assert_eq!(
        with(Verb::ClayStrips),
        [true, true],
        "a faixa tem de oferecer a dureza da ponta e o comprimento"
    );
    // Os CONTROLES: os dois verbos mais próximos — o que deposita pelo mesmo
    // `reach` (Draw) e o que também ergue um plano (Clay).
    for verb in [Verb::Draw, Verb::Clay] {
        assert_eq!(
            with(verb),
            [false, false],
            "{verb:?} não lê a silhueta da faixa e mostrou os knobs dela"
        );
    }
}

/// **OS DOIS KNOBS DO HC existem, e SÓ com o Surface Smooth em mãos.**
///
/// ⚠️ **A varredura genérica deste arquivo é CEGA a isto**, e a mutação prova:
/// apagar as duas rows da tabela deixa os quarenta e cinco gates VERDES. Elas
/// são `Row`s bem-formadas — pintadas, registradas, clicáveis —, e todo gate
/// genérico pergunta *"o que está na tabela funciona?"*; nenhum pergunta *"o
/// que a LEI lê está na tabela?"*. Um verbo cujo kernel consome um número que
/// nenhuma row oferece é uma lei que o artista não alcança — a quarta condição
/// de fechamento, e a única que não é implicada pelas outras três.
///
/// ⚠️ **E as DUAS metades são precisas:** o irmão [`Verb::Smooth`] também lê o
/// anel e **não tem `b` nenhum para devolver**, então oferecer-lhe estes dois
/// seria um par de sliders que não move um vértice. Sem a metade da AUSÊNCIA um
/// `show: always` passaria aqui.
#[test]
fn the_hc_knobs_are_offered_only_with_the_surface_smooth_in_hand() {
    for verb in Verb::ALL {
        let mut ui = Sculpt3dUi::default();
        ui.brush.verb = verb;
        ui.ui_level = UiLevel::Pro;
        let (mut host, mut state) = arrange(ui.clone());
        let painted = host.paint::<Sculpt3dPanel>(&mut state, VIEWPORT);
        let want = verb == Verb::SurfaceSmooth;
        for id in [ids::SCULPT3D_HC_SHAPE, ids::SCULPT3D_HC_VERTEX] {
            assert_eq!(
                painted.iter().any(|(pid, _)| *pid == id),
                want,
                "com {verb:?} o knob do HC devia {}",
                if want { "estar la" } else { "sumir" }
            );
        }
    }
}

/// **E o piso do β vem do MOTOR, nunca de um literal nesta tabela.**
///
/// ⚠️ Abaixo de `0,5` o operador AMPLIFICA em vez de contrair (a forma fechada
/// está em `ph2d_sculpt3d::HC_VERTEX_DEFAULT`), então uma segunda cópia do
/// número aqui divergiria no dia em que a lei o movesse — e o slider passaria a
/// oferecer com o dedo exactamente a faixa que rebenta a malha.
#[test]
fn the_beta_slider_starts_where_the_engine_stops_amplifying() {
    let row = rows::rows()
        .find(|r| r.slider == ids::SCULPT3D_HC_VERTEX)
        .expect("a row do beta sumiu da tabela");
    assert!(
        (row.min - ph2d_sculpt3d::HC_VERTEX_MIN).abs() < f32::EPSILON,
        "o piso da row ({}) não é o do motor ({})",
        row.min,
        ph2d_sculpt3d::HC_VERTEX_MIN
    );
}

/// **O Basic nunca esconde a CURVA que dá forma ao dab** — seja qual for o
/// verbo.
///
/// ⚠️ **A régua é a REFERÊNCIA, não o gosto.** Medido no
/// `properties_paint_common.py` do Blender: o `FalloffPanel` **não** é desenhado
/// por `brush_settings_advanced` — ele é painel de primeira classe, e no
/// cabeçalho de ferramenta ele é um **popover sempre visível**
/// (`layout.popover("VIEW3D_PT_tools_brush_falloff")`). Ele é *dobrado*, nunca
/// *ausente*: o artista SEMPRE vê que existe uma curva.
///
/// ⚠️ **E é por isso que a premissa do Basic estava errada, não a regra dele:**
/// o doc do [`UiLevel::Basic`] diz *"o vocabulário do SculptGL"*, e o SculptGL
/// **não tem** seletor de curva (a dele é fixa) — então herdar aquele
/// vocabulário apagava um controle que a NOSSA malha tem doze vezes e que a
/// outra referência trata como primeiro-classe.
#[test]
fn the_basic_level_never_hides_the_curve_that_shapes_the_dab() {
    for v in Verb::ALL {
        let mut ui = Sculpt3dUi::default();
        ui.brush.verb = v;
        ui.ui_level = UiLevel::Basic;
        let (mut host, mut state) = arrange(ui);
        let painted = host.paint::<Sculpt3dPanel>(&mut state, VIEWPORT);
        assert!(
            painted
                .iter()
                .any(|(id, _)| *id == ids::SCULPT3D_FALLOFF[0]),
            "o Basic escondeu o seletor de curva com o {} em mãos",
            v.label()
        );
    }
}

/// **O FILTRO é oferecido a TODO verbo, e o SELECTOR só com ele armado.**
///
/// ⚠️ **A PREMISSA ANTERIOR foi derrubada pela W9b, e o gate anterior a
/// afirmava** (*"o filtro só existe onde há uma LEI"*): a lei era DERIVADA do
/// verbo em mãos, então oferecer o interruptor ao Draw daria um controle que
/// arma, muda o que o botão esquerdo faz e não move um vértice. Com a lei
/// **escolhida** três delas (`Scale`, `Sphere`, `Random`) não têm verbo nenhum,
/// e o critério antigo as tornava inalcançáveis por gesto — o filtro passa a ser
/// oferecido **sempre**.
///
/// ⚠️ **A metade da AUSÊNCIA mudou de sujeito, não sumiu:** o que não pode
/// aparecer desarmado é o SELECTOR — sete chips a escolher uma lei que nada
/// consome são sete controles mortos.
#[test]
fn the_filter_is_offered_to_every_verb_and_the_picker_only_when_armed() {
    // O CONTROLE de antes vira a asserção principal: um verbo SEM lei própria.
    assert!(
        !Verb::Draw.filters_mesh(),
        "a fixture perdeu a premissa: o Draw passou a ter lei de filtro"
    );
    let mut ui = Sculpt3dUi::default();
    ui.brush.verb = Verb::Draw;
    let (mut host, mut state) = arrange(ui);
    let painted = host.paint::<Sculpt3dPanel>(&mut state, VIEWPORT);
    let rect = painted
        .iter()
        .find(|(id, _)| *id == ids::SCULPT3D_FILTER)
        .map(|(_, r)| *r)
        .expect("o interruptor do filtro não foi oferecido a um verbo sem lei própria");

    // Desarmado: nenhum chip de lei na tela.
    for id in ids::SCULPT3D_FILTER_KIND {
        assert!(
            !painted.iter().any(|(pid, _)| *pid == id),
            "o selector de lei foi pintado com o filtro DESARMADO"
        );
    }

    // E o interruptor despacha.
    let evs = host.click_at(rect.x + rect.w * 0.5, rect.y + rect.h * 0.5);
    assert!(
        evs.iter()
            .any(|e| matches!(e, WidgetEvent::Click(id) if *id == ids::SCULPT3D_FILTER)),
        "o interruptor do filtro está pintado e morto sob o mouse"
    );
    for e in evs {
        let _ = host.apply_panel_event::<Sculpt3dPanel>(&mut state, e);
    }
    assert_eq!(
        drain_intents(),
        vec![Sculpt3dIntent::ArmFilter],
        "o clique no filtro não chegou ao shell"
    );
}

/// ⭐⭐ **OS DOIS MOTORES DE RETOPOLOGIA SÃO ESCOLHÍVEIS** — pintados, vivos sob o
/// mouse, e cada chip escreve o SEU.
///
/// ⛔ **O porte do Instant Meshes viveu a wave inteira do pivô atrás de
/// `PH2D_RETOPO_LEGACY=1`** — alcançável só por quem soubesse o nome da variável.
/// *Um motor que o painel não oferece não existe para o artista*, e o Enio pediu-o
/// pelo nome (2026-08-21).
///
/// ⚠️ **A metade que carrega o gate é a ÚLTIMA:** um selector cujos dois chips
/// despachem o mesmo índice é pintado, clicável e passa em todo sweep de
/// clicabilidade — e o artista escolhe `Fast` para receber a cadeia lenta.
#[test]
fn every_retopo_engine_is_pickable_and_writes_its_own() {
    assert_eq!(
        ids::SCULPT3D_RETOPO_MODE.len(),
        RetopoMode::ALL.len(),
        "a lista de chips e a lista de motores têm tamanhos diferentes — algum motor é \
         inalcançável, ou algum chip nomeia um motor que não existe"
    );
    for (i, mode) in RetopoMode::ALL.into_iter().enumerate() {
        let ui = Sculpt3dUi {
            // ⚠️ **A fixture começa no OUTRO motor**, senão a iteração `i = 0` é
            // verde por vácuo: o `default()` já vale `ALL[0]`, e um chip que não
            // escrevesse nada passaria a asserção.
            retopo_mode: RetopoMode::ALL[(i + 1) % RetopoMode::ALL.len()],
            ..Sculpt3dUi::default()
        };
        let (mut host, mut state) = arrange_with(snapshot(ui, true));

        let painted = host.paint::<Sculpt3dPanel>(&mut state, VIEWPORT);
        let rect = painted
            .iter()
            .find(|(id, _)| *id == ids::SCULPT3D_RETOPO_MODE[i])
            .map(|(_, r)| *r)
            .unwrap_or_else(|| panic!("{mode:?}: o chip do motor não foi pintado"));
        let evs = host.click_at(rect.x + rect.w * 0.5, rect.y + rect.h * 0.5);
        assert!(
            evs.iter().any(
                |e| matches!(e, WidgetEvent::Click(id) if *id == ids::SCULPT3D_RETOPO_MODE[i])
            ),
            "{mode:?}: o chip do motor está pintado e morto sob o mouse"
        );
        for e in evs {
            let _ = host.apply_panel_event::<Sculpt3dPanel>(&mut state, e);
        }
        let Sculpt3dIntent::SetUi(got) = only_intent("chip de motor") else {
            panic!("{mode:?}: o chip enfileirou o intent errado");
        };
        assert_eq!(
            got.retopo_mode, mode,
            "{mode:?}: o chip escreveu OUTRO motor -- o artista escolhe um e recebe outro"
        );
    }
}

/// ⭐ **AS SETE LEIS SÃO ESCOLHÍVEIS** — pintadas, vivas sob o mouse, e cada
/// chip escreve a SUA.
///
/// ⚠️ **A metade que carrega o gate é a ÚLTIMA:** um selector cujos sete chips
/// despachem o mesmo índice é pintado, clicável e passa em todo sweep de
/// clicabilidade — e o artista escolhe `Random` para receber `Smooth`.
#[test]
fn every_filter_law_is_pickable_and_writes_its_own() {
    assert_eq!(
        ids::SCULPT3D_FILTER_KIND.len(),
        FilterKind::ALL.len(),
        "a lista de chips e a lista de leis têm tamanhos diferentes — alguma lei é \
         inalcançável, ou algum chip nomeia uma lei que não existe"
    );
    for (i, kind) in FilterKind::ALL.into_iter().enumerate() {
        let mut ui = Sculpt3dUi::default();
        // Um verbo SEM lei própria de propósito: as três leis sem verbo só são
        // alcançáveis se o selector não depender de quem está em mãos.
        ui.brush.verb = Verb::Draw;
        // ⚠️ **A fixture começa numa lei DIFERENTE da que o chip escreve**,
        // senão a iteração `i = 0` é verde por vácuo: o `default()` já vale
        // `ALL[0]`, e um chip que não escrevesse nada passaria a asserção.
        ui.filter_kind = FilterKind::ALL[(i + 1) % FilterKind::ALL.len()];
        // ⚠️ **O retrato chega ARMADO**, e o gate acima é que prova que o
        // interruptor leva até aqui: o `filter_armed` é campo do SNAPSHOT (o
        // que a cena responde), não do `Sculpt3dUi` que o painel devolve, então
        // clicar o toggle nesta fixture não o move -- ele enfileira o intent e
        // a cena é quem decide.
        let mut snap = snapshot(ui, true);
        snap.filter_armed = true;
        let (mut host, mut state) = arrange_with(snap);

        let painted = host.paint::<Sculpt3dPanel>(&mut state, VIEWPORT);
        let rect = painted
            .iter()
            .find(|(id, _)| *id == ids::SCULPT3D_FILTER_KIND[i])
            .map(|(_, r)| *r)
            .unwrap_or_else(|| panic!("{kind:?}: o chip não foi pintado com o filtro armado"));
        let evs = host.click_at(rect.x + rect.w * 0.5, rect.y + rect.h * 0.5);
        assert!(
            evs.iter().any(
                |e| matches!(e, WidgetEvent::Click(id) if *id == ids::SCULPT3D_FILTER_KIND[i])
            ),
            "{kind:?}: o chip está pintado e morto sob o mouse"
        );
        for e in evs {
            let _ = host.apply_panel_event::<Sculpt3dPanel>(&mut state, e);
        }
        let Sculpt3dIntent::SetUi(got) = only_intent("chip de lei") else {
            panic!("{kind:?}: o chip enfileirou o intent errado");
        };
        assert_eq!(
            got.filter_kind, kind,
            "{kind:?}: o chip escreveu OUTRA lei -- o artista escolhe uma e recebe outra"
        );
    }
}
