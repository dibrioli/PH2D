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
use ph2d_sculpt3d::{
    Alpha, ClothFilterKind, Falloff, FilterKind, FilterLaw, RefMode, TransformKind, Verb,
    kelvinlet::Scales,
};
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
        // ⚠️ CHAVES, como o produto entrega — e de propósito chaves que a tabela NÃO conhece:
        // este arnês mede a FIAÇÃO (quantos chips, qual despacha) e não o vocabulário, e uma
        // chave desconhecida volta crua, que é o texto que estes gates comparam.
        matcap_keys: &["Clay", "Pearl", "Skin", "Jade", "Metal", "Wax"],
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
        // ⚠️ **ASSADO e na lei de FÁBRICA** (o índice `1`, a `Forma`): a fileira da
        // lei só existe quando o sprite escolhido já tem canais, e o sweep de
        // costura tem de encontrar os dois chips dela — com `None` aqui eles
        // ficavam vivos na tela e nunca clicados por gate nenhum.
        lei_do_alvo: Some(1),
        lei_rotulos: &[
            "panel.sculpt3d.bake_law.paint",
            "panel.sculpt3d.bake_law.form",
        ],
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
///
/// ⛔⛔⛔ **DUAS CURAS PARA O MESMO DEFEITO ENCONTRARAM-SE NA INTEGRAÇÃO DE 2026-09-20, e a medição
/// desempatou-as.** O `main` e a `line/sculpt3d` curaram, cada um à sua maneira, o mesmo
/// `SIGABRT — has overflowed its stack` que este teste dava no perfil de DESENVOLVIMENTO (e **só**
/// nele: no `ci-test` ele passava, logo o CI e o `ship.sh` nunca o viam, e quem corria o caminho
/// documentado da corrida dirigida — `cargo-test-narrow.sh`, que é `dev` — via o binário inteiro
/// morrer com o script a imprimir **«0 falharam»**. *Um teste que aborta e se lê como zero falhas é
/// pior que um vermelho.*)
///
/// - o `main` pôs o corpo numa **thread com 8 MiB de pilha**, e atribuiu a causa à FRAME de
///   `arrange` → `with_panel` → `populate`, com o array de 24 casos medido a `~230 KB` —
///   *«um oitavo do que estoura»*;
/// - a linha trocou a lista de **valores** por uma de **CONSTRUTORES** (`fn() -> Sculpt3dIntent`),
///   com a causa atribuída às CÓPIAS daquele array (o literal · o `IntoIterator` · a
///   desestruturação por iteração).
///
/// ⭐⭐⭐ **A atribuição do `main` está REFUTADA, com controlo positivo:** a lista de VALORES **sem**
/// thread aborta (`SIGABRT`, na pilha de omissão de `2 MB`), e a lista de CONSTRUTORES **sem** thread
/// passa `2` de `2` no MESMO perfil ⇒ *a frame sozinha cabe, e o dominante era o array*. Medido na
/// integração, nesta árvore, com o controlo a reproduzir o defeito antes da cura.
///
/// ⚠️ **As duas FICAM, e os papéis são diferentes:** os construtores tiram a CAUSA (`24 × 16` bytes,
/// e um intent vivo de cada vez); a thread fica como **folga DECLARADA** para a frame deste painel,
/// que é o maior do app e não tem tecto medido. ⛔ *Nenhuma das duas é falsificável pela suíte de
/// hoje* — tirar qualquer uma delas deixa este teste verde —, e é por isso que a tabela acima está
/// escrita aqui: quem a quiser remover repete aquele par de corridas e escreve o número.
///
/// ⛔ E a saída que as duas recusam pelo mesmo motivo: `RUST_MIN_STACK=16777216` no ambiente cura
/// isto e **cala** o mesmo defeito em todo o resto da suíte.
///
/// ⭐ O **piso de população** (`casos.len() == 24`) vem do lado da linha: sem ele, uma lista que
/// encolhesse mediria menos comandos e lia-se como aprovação.
#[test]
fn every_command_reaches_the_shell() {
    std::thread::Builder::new()
        .stack_size(8 * 1024 * 1024)
        .spawn(corpo_de_every_command_reaches_the_shell)
        .expect("a thread do gate nasce")
        .join()
        .expect("o gate corre ate' ao fim");
}

fn corpo_de_every_command_reaches_the_shell() {
    let casos: [(ph2d_a11y::NodeId, fn() -> Sculpt3dIntent); 27] = [
        (ids::SCULPT3D_DYNTOPO, || Sculpt3dIntent::ToggleDyntopo),
        (ids::SCULPT3D_LEVEL_DOWN, || {
            Sculpt3dIntent::ChangeLevel(false)
        }),
        (ids::SCULPT3D_LEVEL_UP, || Sculpt3dIntent::ChangeLevel(true)),
        (ids::SCULPT3D_SUBDIVIDE, || Sculpt3dIntent::Subdivide),
        (ids::SCULPT3D_REVERSE, || Sculpt3dIntent::ReverseLevel),
        (ids::SCULPT3D_FLATTEN, || Sculpt3dIntent::Flatten),
        (ids::SCULPT3D_REMESH, || Sculpt3dIntent::Remesh),
        (ids::SCULPT3D_QUAD_REMESH, || Sculpt3dIntent::QuadRemesh),
        (ids::SCULPT3D_CLOSE_HOLES, || Sculpt3dIntent::CloseHoles),
        (ids::SCULPT3D_DUPLICATE, || Sculpt3dIntent::Duplicate),
        (ids::SCULPT3D_DELETE, || Sculpt3dIntent::Delete),
        (ids::SCULPT3D_ISOLATE, || Sculpt3dIntent::ToggleIsolate),
        (ids::SCULPT3D_MERGE, || Sculpt3dIntent::Merge),
        (ids::SCULPT3D_BAKE_AO, || Sculpt3dIntent::BakeAo),
        (ids::SCULPT3D_BAKE_SPRITE, || Sculpt3dIntent::BakeToSprite),
        (ids::SCULPT3D_ALPHA_SPRITE, || {
            Sculpt3dIntent::AlphaFromSprite
        }),
        (ids::SCULPT3D_ADD[0], || Sculpt3dIntent::AddSphere),
        (ids::SCULPT3D_ADD[1], || Sculpt3dIntent::AddCube),
        (ids::SCULPT3D_ADD[2], || Sculpt3dIntent::AddCylinder),
        (ids::SCULPT3D_ADD[3], || Sculpt3dIntent::AddTorus),
        (ids::SCULPT3D_MASK_OP[0], || Sculpt3dIntent::MaskClear),
        (ids::SCULPT3D_MASK_OP[1], || Sculpt3dIntent::MaskInvert),
        (ids::SCULPT3D_MASK_OP[2], || Sculpt3dIntent::MaskBlur),
        (ids::SCULPT3D_MASK_OP[3], || Sculpt3dIntent::MaskSharpen),
        (ids::SCULPT3D_COLOR_FILL, || Sculpt3dIntent::ColorFill),
        // ⭐ Os dois chips da LEI do objecto assado: **um PEDIDO por chip**, e a
        // POSIÇÃO é a tag — por isso o índice aparece aqui à mão, e não derivado.
        (ids::SCULPT3D_BAKE_LAW[0], || Sculpt3dIntent::LeiDoAlvo(0)),
        (ids::SCULPT3D_BAKE_LAW[1], || Sculpt3dIntent::LeiDoAlvo(1)),
    ];
    assert_eq!(
        casos.len(),
        27,
        "o piso de população: a lista dos comandos de um toque encolheu"
    );
    for (id, faz) in casos {
        let want = faz();
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

/// **TODO cabeçalho PINTADO dobra — e dobrar é local do painel**, nunca alcança
/// a cena.
///
/// ⚠️ **A lista é DERIVADA ([`rows::section_headers`]), e a versão anterior era
/// escrita à mão com SEIS ids.** O painel pinta SETE cabeçalhos, e o que faltava
/// na lista era exatamente o que faltava no `is_section_header` do `event.rs`: o
/// `SCULPT3D_SEC_BAKE`. Um gate que enumera à mão o mesmo conjunto que o produto
/// enumera à mão fica verde sobre a linha que os dois esqueceram — *duas listas
/// e um gate são três respostas à mesma pergunta, e a que o artista toca é a que
/// envelhece*.
///
/// ⚠️ **A metade da PINTURA é a que torna a derivação honesta.** Sem ela, um id
/// acrescentado ao `section_headers()` e nunca desenhado passaria (registrado e
/// despachado, mas invisível); com ela, o gate exige que cada cabeçalho da porta
/// única seja de facto um chevron na tela.
#[test]
fn every_painted_section_header_folds_and_never_touches_the_scene() {
    let (mut host, mut state) = arrange(Sculpt3dUi::default());
    let painted = host.paint::<Sculpt3dPanel>(&mut state, VIEWPORT);
    let heads: Vec<_> = rows::section_headers().collect();
    assert!(
        heads.len() >= 7,
        "a fixture perdeu o fenômeno: só {} cabeçalho(s) na porta única",
        heads.len()
    );
    for id in heads {
        assert!(
            painted.iter().any(|(pid, _)| *pid == id),
            "o cabeçalho {id:?} está na porta única e ninguém o pinta"
        );
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
    //
    // ⛔⛔ **OS 38 VERBOS SAÍRAM DAQUI EM 2026-09-20, e não é uma tolerância** (ordem do dono): eles
    // deixaram de ser pintados no painel e vivem na paleta
    // ([`ph2d_panel_sculpt3d::brush_palette`]). Esta varredura pergunta *«o que o painel pinta é
    // clicável onde está desenhado?»*, e um controlo que ele já não pinta não é sujeito dela.
    //
    // ⚠️⚠️ **Quem responde por eles agora são TRÊS gates, e a divisão é de propósito:** a paleta
    // contém os 38 com os ids do painel (`todo_pincel_chega_a_paleta_com_o_id_do_painel`), o id
    // volta a ser o verbo certo (`o_id_da_paleta_resolve_para_o_verbo_certo`), e o **botão que a
    // abre** é pintado e clicável — que é o que a linha abaixo põe nesta mesma varredura.
    // *Apagar uma linha de uma tabela de cobertura sem dizer quem passou a cobrir é como uma
    // catraca vira licença.*
    want.push(("open brushes".to_string(), ids::SCULPT3D_OPEN_BRUSHES));
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
        // ⭐⭐ **Os dois chips da LEI do objecto assado.** ⚠️ Eles só são pintados
        // com `lei_do_alvo: Some(..)` no retrato — e é por isso que a fixture o
        // declara: *a fixture tem de conter o fenómeno*, a quarta vez que este
        // ficheiro escreve a frase. Sem esta entrada, uma fileira que deixasse de
        // ser pintada continuava a despachar por `Click` sintético e o sweep dos
        // comandos ficava VERDE — que é a diferença entre *nunca pintado* e
        // *morto sob o dedo*, e esta crate já a pagou sete vezes.
        ("bake law paint", ids::SCULPT3D_BAKE_LAW[0]),
        ("bake law form", ids::SCULPT3D_BAKE_LAW[1]),
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
    cada_botao_responde_no_proprio_centro(&mut host, &painted, &by_id);
}

/// **A metade SEM LISTA do gate acima**, extraída para ter DOIS chamadores.
///
/// ⚠️⚠️ **A extracção é de 07/09 e a razão é medida:** o gate irmão arma o
/// **Crease**, e com outro pincel na mão as fileiras do TECIDO nem são
/// desenhadas — os oito modos e as três áreas ficaram **mortos sob o ponteiro**
/// desde 06/09 sem um único gate a acusá-lo. *A fixtura tem de conter o
/// fenómeno*, e uma fixtura só nunca contém dois pincéis.
///
/// ⛔ Ela continua sem lista: o conjunto é o que o PAINT registou, então um
/// controlo novo nasce coberto — não há aqui onde o esquecer.
fn cada_botao_responde_no_proprio_centro(
    host: &mut MockPanelHost,
    painted: &[(ph2d_a11y::NodeId, ph2d_editor_core::zones::Rect)],
    by_id: &std::collections::BTreeMap<ph2d_a11y::NodeId, String>,
) {
    // ⚠️ **A exclusão é por GESTO, e a polaridade é o ponto:** o default é *tem
    // de responder a um clique*, e sai da varredura só o que é dirigido por
    // ARRASTO — as pistas, os chips numéricos e o puxador do próprio painel
    // (chrome do host, não deste painel). Exigir um `Click` de qualquer um dos
    // três seria afirmar um gesto que eles não têm; e como a lista é de EXCEÇÕES,
    // esquecer um controle novo nela o deixa **coberto**, não fora.
    let mut dragged: Vec<ph2d_a11y::NodeId> =
        rows::rows().flat_map(|r| [r.slider, r.chip]).collect();
    dragged.extend([
        ph2d_panel_model3d::ids::INSP_DRAG_HANDLE,
        ph2d_panel_sculpt3d::ids::INSP_RESIZE_HANDLE,
        ph2d_panel_sculpt3d::ids::INSP_RESIZE_HANDLE_BL,
        ph2d_editor_core::widget::SCULPT3D_SCROLLBAR_ID,
    ]);
    let mut seen: Vec<ph2d_a11y::NodeId> = Vec::new();
    for &(id, rect) in painted {
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

/// ⭐⭐⭐ **GATE — com o PINCEL DE TECIDO na mão, tudo o que o painel desenha
/// responde ao ponteiro.**
///
/// ⛔⛔ **Ele nasce vermelho sobre o que shipou em 06/09:** os oito chips de
/// *Deformation* e os três de *Simulation Area* estavam pintados, hit-indexados
/// e **fora do `populate`** ⇒ o clique era descartado em silêncio. O gate irmão
/// não os via porque arma o **Crease**, e a fileira do tecido só existe com o
/// tecido na mão. *A fixtura tem de conter o fenómeno* — a sexta vez que este
/// ficheiro o escreve.
///
/// ⚠️ **A área é a *Local* de propósito:** é a única que oferece o *Pin
/// Simulation Boundary* (espec §2.3), e com a omissão (*Dynamic*) aquela caixa
/// não seria varrida.
#[test]
fn every_cloth_control_is_clickable_where_it_is_drawn() {
    let mut ui = Sculpt3dUi::default();
    ph2d_panel_sculpt3d::state::switch_verb(&mut ui, Verb::Cloth);
    ui.brush.cloth_area = ph2d_sculpt3d::ClothArea::Local;
    ui.ui_level = UiLevel::Pro;
    let (mut host, mut state) = arrange(ui.clone());
    let painted = host.paint::<Sculpt3dPanel>(&mut state, VIEWPORT);

    // ⚠️ **Anti-vácuo com NOME:** os cinco knobs do tecido, as três fileiras e a
    // caixa do pino têm de estar entre o que foi pintado — senão a varredura
    // abaixo passaria sobre um painel que não desenhou nada de tecido.
    let mut want: Vec<(String, ph2d_a11y::NodeId)> = Vec::new();
    for row in rows::rows() {
        if row.visible(&ui) {
            want.push((row.label.to_string(), row.slider));
            want.push((row.label.to_string(), row.chip));
        }
    }
    let sliders = want.len();
    for (i, m) in ph2d_sculpt3d::ClothMode::ALL.into_iter().enumerate() {
        want.push((
            format!("cloth mode {}", m.label()),
            ids::SCULPT3D_CLOTH_MODE[i],
        ));
    }
    for (i, a) in ph2d_sculpt3d::ClothArea::ALL.into_iter().enumerate() {
        want.push((
            format!("cloth area {}", a.label()),
            ids::SCULPT3D_CLOTH_AREA[i],
        ));
    }
    for (i, f) in ph2d_sculpt3d::ClothForceFalloff::ALL
        .into_iter()
        .enumerate()
    {
        want.push((
            format!("cloth force falloff {}", f.label()),
            ids::SCULPT3D_CLOTH_FORCE_FALLOFF[i],
        ));
    }
    want.push(("cloth pin".to_string(), ids::SCULPT3D_CLOTH_PIN));
    assert!(
        want.len() > sliders + 12,
        "a fixtura de tecido varre {} controlos -- ela nao contem o fenomeno",
        want.len()
    );
    for (name, id) in &want {
        assert!(
            painted.iter().any(|(pid, _)| pid == id),
            "`{name}` ({id:?}) devia estar pintado com o tecido na mao"
        );
    }
    let by_id: std::collections::BTreeMap<_, _> = want
        .iter()
        .skip(sliders)
        .map(|(n, id)| (*id, n.clone()))
        .collect();
    cada_botao_responde_no_proprio_centro(&mut host, &painted, &by_id);
}

/// ⭐⭐ **GATE — com o ESFREGÃO na mão, a fileira dele é PINTADA e responde ao
/// ponteiro.**
///
/// ⛔⛔ **As duas metades são defeitos DIFERENTES que dão o MESMO report** (a
/// lei que a `line/Vector` pagou duas vezes): uma fileira nunca pintada e uma
/// fileira **morta sob o dedo** leem-se igual para quem usa. O `Click`
/// sintético só apanha a segunda porque o gesto é REAL — carregar no centro do
/// rect que o painel registou.
///
/// ⚠️ **A fixtura tem de conter o fenómeno:** os gates irmãos armam o `Crease` e
/// o `Cloth`, e a fileira deste pincel só existe com **ele** na mão.
#[test]
fn every_smear_control_is_clickable_where_it_is_drawn() {
    let mut ui = Sculpt3dUi::default();
    ph2d_panel_sculpt3d::state::switch_verb(&mut ui, Verb::SmearMultires);
    ui.ui_level = UiLevel::Pro;
    let (mut host, mut state) = arrange(ui);
    let painted = host.paint::<Sculpt3dPanel>(&mut state, VIEWPORT);

    let want: Vec<(String, ph2d_a11y::NodeId)> = ph2d_sculpt3d::SmearMode::ALL
        .into_iter()
        .enumerate()
        .map(|(i, m)| {
            (
                format!("smear mode {}", m.label()),
                ids::SCULPT3D_SMEAR_MODE[i],
            )
        })
        .collect();
    assert_eq!(
        want.len(),
        3,
        "a espec §5.3 conta TRÊS direcções — a fixtura deixou de conter o fenómeno"
    );
    for (name, id) in &want {
        assert!(
            painted.iter().any(|(pid, _)| pid == id),
            "`{name}` ({id:?}) devia estar pintado com o esfregão na mão"
        );
    }
    let by_id: std::collections::BTreeMap<_, _> =
        want.iter().map(|(n, id)| (*id, n.clone())).collect();
    for (name, id) in &want {
        let rect = painted
            .iter()
            .rev()
            .find(|(pid, _)| pid == id)
            .map(|(_, r)| *r)
            .expect("pintado logo acima");
        let (cx, cy) = (rect.x + rect.w * 0.5, rect.y + rect.h * 0.5);
        let events = host.click_at(cx, cy);
        assert!(
            events
                .iter()
                .any(|e| matches!(e, WidgetEvent::Click(c) if *c == *id)),
            "clicar `{name}` no centro pintado não produziu Click — ele está no \
             índice de hit e morto sob o dedo"
        );
    }
    let _ = by_id;
}

/// ⭐⭐ **GATE — com o PROJECTAR na mão, as TRÊS superfícies dele são pintadas e
/// respondem ao ponteiro.**
///
/// ⛔⛔ **Irmão exacto do gate do esfregão, e ele existe porque aquele defeito
/// aconteceu:** os três chips do esfregão nasceram pintados, hit-indexados e
/// **mortos sob o dedo** — faltava a fileira no `populate`. *Um controlo nunca
/// pintado e um morto sob o dedo dão o MESMO report*, e só o gesto REAL os
/// separa.
///
/// ⚠️ **A caixa dos dois sentidos entra com os chips**, e não é zelo: ela é o
/// único caminho para um alvo do lado errado ser alcançado, logo sem ela metade
/// da espec §6.3 é inexprimível pelo artista.
#[test]
fn every_project_control_is_clickable_where_it_is_drawn() {
    let mut ui = Sculpt3dUi::default();
    ph2d_panel_sculpt3d::state::switch_verb(&mut ui, Verb::SceneProject);
    ui.ui_level = UiLevel::Pro;
    let (mut host, mut state) = arrange(ui);
    let painted = host.paint::<Sculpt3dPanel>(&mut state, VIEWPORT);

    let mut want: Vec<(String, ph2d_a11y::NodeId)> = ph2d_sculpt3d::ProjectMode::ALL
        .into_iter()
        .enumerate()
        .map(|(i, m)| {
            (
                format!("project mode {}", m.label()),
                ids::SCULPT3D_PROJECT_MODE[i],
            )
        })
        .collect();
    assert_eq!(
        want.len(),
        2,
        "a espec §6.2 conta DUAS direcções — a fixtura deixou de conter o fenómeno"
    );
    want.push(("search both ways".to_owned(), ids::SCULPT3D_PROJECT_BIDIR));
    for (name, id) in &want {
        assert!(
            painted.iter().any(|(pid, _)| pid == id),
            "`{name}` ({id:?}) devia estar pintado com o projectar na mão"
        );
        let rect = painted
            .iter()
            .rev()
            .find(|(pid, _)| pid == id)
            .map(|(_, r)| *r)
            .expect("pintado logo acima");
        let (cx, cy) = (rect.x + rect.w * 0.5, rect.y + rect.h * 0.5);
        let events = host.click_at(cx, cy);
        assert!(
            events
                .iter()
                .any(|e| matches!(e, WidgetEvent::Click(c) if *c == *id)),
            "clicar `{name}` no centro pintado não produziu Click — ele está \
             pintado e morto sob o dedo"
        );
    }
}

/// ⛔⛔⛔ **GATE — com a POSE na mão, as CINCO deformações são pintadas e
/// respondem ao ponteiro.**
///
/// # Ele reproduz um report do dono, à letra
///
/// 2026-09-15: *«os outros 2 botões ainda não funcionam»*. Os três chips do
/// `Deformation` estavam pintados, hit-indexados, **com braço no `event.rs`** e
/// **mortos sob o ponteiro** — faltava a fileira no `populate`. Da mão do
/// artista o sintoma é o pior possível: clicar em `Scale / Translate` não muda
/// nada, o pincel FICA no modo de omissão, e as outras duas deformações leem-se
/// como **leis partidas** em vez de um clique descartado.
///
/// ⚠️⚠️ **É a SÉTIMA vez que esta crate paga isto**, e a razão de nenhum gate o
/// ver é sempre a mesma: os irmãos armam **outro** pincel, e com ele na mão esta
/// fileira nem chega a ser desenhada. *Uma fixtura que não contém o fenómeno não
/// afirma nada sobre ele.* ⇒ a cura ESTRUTURAL é o censo derivado
/// (`populate_censo_tests`), que compara o despacho com o registo sem armar
/// pincel nenhum; este gate é a **reprodução**, e vale por medir o gesto real.
///
/// ⚠️ **As duas caixas entram com os chips:** `Pin far end` é o que separa uma
/// rotação em torno do pivô de um arrasto rígido, e sem ela metade do §5.1 é
/// inexprimível pelo artista.
#[test]
fn every_pose_control_is_clickable_where_it_is_drawn() {
    let mut ui = Sculpt3dUi::default();
    ph2d_panel_sculpt3d::state::switch_verb(&mut ui, Verb::Pose);
    ui.ui_level = UiLevel::Pro;
    let (mut host, mut state) = arrange(ui);
    let painted = host.paint::<Sculpt3dPanel>(&mut state, VIEWPORT);

    let mut want: Vec<(String, ph2d_a11y::NodeId)> = ph2d_sculpt3d::PoseDeformacao::ALL
        .into_iter()
        .enumerate()
        .map(|(i, d)| {
            (
                format!("deformation {}", d.label_key()),
                ids::SCULPT3D_POSE_MODE[i],
            )
        })
        .collect();
    assert_eq!(
        want.len(),
        5,
        "os três modos da espec dão CINCO deformações — a fixtura deixou de \
         conter o fenómeno"
    );
    want.push(("pin far end".to_owned(), ids::SCULPT3D_POSE_ANCHORED));
    for (name, id) in &want {
        assert!(
            painted.iter().any(|(pid, _)| pid == id),
            "`{name}` ({id:?}) devia estar pintado com a pose na mão"
        );
        let rect = painted
            .iter()
            .rev()
            .find(|(pid, _)| pid == id)
            .map(|(_, r)| *r)
            .expect("pintado logo acima");
        let (cx, cy) = (rect.x + rect.w * 0.5, rect.y + rect.h * 0.5);
        let events = host.click_at(cx, cy);
        assert!(
            events
                .iter()
                .any(|e| matches!(e, WidgetEvent::Click(c) if *c == *id)),
            "clicar `{name}` no centro pintado não produziu Click — ele está \
             pintado e morto sob o dedo, que é o report do dono à letra"
        );
    }

    // ⛔⛔ **A SEGUNDA METADE, e ela existe porque a primeira era CEGA:** o
    // `Scale without rotating` só é pintado com a deformação `Scale` escolhida
    // (é a única em que ele tem o que travar — espec §5.4), e este gate corria
    // com o valor de fábrica, que é `Rotate`. *Uma fixtura que não contém o
    // fenómeno não afirma nada sobre ele*, e foi exactamente essa cegueira que
    // deixou os treze chips passarem sete vezes.
    //
    // ⚠️ Ele é o controlo da pergunta do dono (2026-09-15: *«Em Scale o osso
    // escalona e rotaciona ao mesmo tempo. Isso é o esperado?»*) — **é**, e esta
    // caixa é a resposta dele. Se ela nascer morta, a resposta é inalcançável.
    let mut ui = Sculpt3dUi::default();
    ph2d_panel_sculpt3d::state::switch_verb(&mut ui, Verb::Pose);
    ui.ui_level = UiLevel::Pro;
    ui.brush.pose.deformacao = ph2d_sculpt3d::PoseDeformacao::Escalar;
    let (mut host, mut state) = arrange(ui);
    let painted = host.paint::<Sculpt3dPanel>(&mut state, VIEWPORT);
    let id = ids::SCULPT3D_POSE_ROT_LOCK;
    let rect = painted
        .iter()
        .rev()
        .find(|(pid, _)| *pid == id)
        .map(|(_, r)| *r)
        .expect(
            "`Scale without rotating` devia estar pintado com a deformação \
             `Scale` escolhida — é a única em que ele tem o que travar",
        );
    let (cx, cy) = (rect.x + rect.w * 0.5, rect.y + rect.h * 0.5);
    let events = host.click_at(cx, cy);
    assert!(
        events
            .iter()
            .any(|e| matches!(e, WidgetEvent::Click(c) if *c == id)),
        "clicar `Scale without rotating` no centro pintado não produziu Click — \
         a caixa que responde à pergunta do dono está morta sob o dedo"
    );
}

/// ⛔⛔ **O INTERRUPTOR `Accumulate` NÃO É OFERECIDO A QUEM NÃO O LÊ.**
///
/// ⚠️⚠️ **Este gate nasceu de uma mutação SOBREVIVENTE, e o que ela expôs é
/// ONDE a lei é observável.** Pôr o [`ph2d_sculpt3d::Verb::EraseMultires`] de
/// volta na família do `accumulates()` não mudava **geometria nenhuma** — no
/// `Grip::Stamp` a coluna aditiva do `GripLaw` é `false` de qualquer maneira, e
/// o alvo do apagador lê a posição viva nos dois casos. ⇒ *o predicado governa
/// uma ROW, não um pixel de barro*, e um gate de geometria nunca o apanharia.
///
/// ⭐ E a row importa: um interruptor que aparece e não faz nada é o **controlo
/// morto** que o §5.0 do roteador descreve — a espécie que *todo gate de registo
/// atravessa verde*, porque ele está pintado, hit-indexado e vivo sob o dedo.
///
/// ⚠️ **As duas metades:** o apagador **não** o vê, e o `Draw` **vê** — senão um
/// `false` cravado passaria.
#[test]
fn o_acumular_nao_e_oferecido_a_quem_nao_o_le() {
    let com = |verb: Verb| {
        let mut ui = Sculpt3dUi::default();
        ph2d_panel_sculpt3d::state::switch_verb(&mut ui, verb);
        ui.ui_level = UiLevel::Pro;
        let (mut host, mut state) = arrange(ui);
        let painted = host.paint::<Sculpt3dPanel>(&mut state, VIEWPORT);
        painted
            .iter()
            .any(|(pid, _)| *pid == ids::SCULPT3D_ACCUMULATE)
    };
    assert!(
        !com(Verb::EraseMultires),
        "o `Accumulate` foi pintado com o apagador em mãos — o alvo dele é \
         ABSOLUTO (a superfície de referência) e acumular não nomeia nada"
    );
    // ⛔⛔ **E o ESFREGÃO, por uma razão IRMÃ e não a mesma** (2026-09-14): o
    // alvo dele muda a cada dab (o campo `D` é relido), mas é ancorado na
    // superfície de referência — e o `Accumulate` desta casa é o `from_live`,
    // *de onde a curva de queda mede a distância*. ⚠️ **Ele está VIVO**
    // (medido em `o_acumular_do_esfregao_e_uma_lei_que_ninguem_declara`), e é
    // por isso que ele não pode ser oferecido: a lei que ele instalaria não
    // está em espec nenhuma. *Um knob vivo escondido e um knob morto escondido
    // leem-se igual — o que os separa é a medição escrita ao lado.*
    assert!(
        !com(Verb::SmearMultires),
        "o `Accumulate` foi pintado com o esfregão em mãos — a lei que ele \
         instalaria não é declarada por referência nenhuma"
    );
    // ⭐ **O controlo:** um verbo que o LÊ continua a vê-lo.
    assert!(
        com(Verb::Draw),
        "o `Accumulate` sumiu do `Draw` — sem este lado, um `false` cravado no \
         predicado passaria a metade de cima"
    );
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
        // ⚠️ **A escada vem da PORTA** (`LightMode::from_option_index`) e não de
        // um `match` escrito aqui: uma segunda cópia da aritmética divergiria no
        // dia do quarto modo — e o quarto modo foi 2026-09-20, quando o PLANO
        // entrou à frente do rig.
        let want = ph2d_panel_sculpt3d::state::LightMode::from_option_index(i);
        assert_eq!(got.lighting, want, "o chip {i} armou {:?}", got.lighting);
        assert_eq!(
            got,
            Sculpt3dUi {
                lighting: want,
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
/// A mesma lei do `arm_tool_falloff_defaults` do Painter e do default de força por
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

/// **SÓ O ESTÚDIO SOME SOB UM MATCAP — as duas pistas da LÂMPADA ficam.**
///
/// ⛔⛔ **Este gate chamava-se `the_rows_that_read_the_rig_vanish_under_a_matcap`
/// e a premissa dele MORREU em 2026-09-20, por report do dono** (*«não encontrei
/// no painel de Sculpt os parâmetros de iluminação»*). Ele afirmava que as TRÊS
/// rows somem, e a que foi medida é outra:
///
/// * o **ambiente do estúdio** e as duas metades da **subsuperfície** são
///   consumidos **só** pelo `mesh.wgsl` ⇒ sob um matcap são de facto inertes, e
///   a razão de sempre continua de pé (*um matcap já É um ambiente*);
/// * o **azimute** e a **elevação** escrevem o **RIG DO DOCUMENTO**
///   (`ph2d_app_sculpt3d::panel::apply_ui`, uma atribuição sem `if`), e o rig
///   tem um consumidor que o matcap **não desliga** — a fase
///   `fase_relight_baked_forms` re-acende **todo objecto 3D assado** cujo
///   carimbo de rig ficou velho, uma vez por quadro e **fora de toda `feature`**
///   (o gate `a_baked_object_outlives_the_3d_module` do shell já o afirma).
///
/// ⇒ escondê-las era esconder **dois controlos VIVOS**, e como o painel nasce em
/// `Matcap(0)` o artista não tinha por onde descobrir que este app tem um rig.
///
/// ⚠️ **As duas metades num gate só, de propósito:** a de ausência sozinha
/// ficaria verde com o `show` cravado em `false` (o defeito do report), e a de
/// presença sozinha ficaria verde com ele cravado em `true` — o que apagaria a
/// lei que ainda protege os três do estúdio.
#[test]
fn so_o_estudio_some_sob_um_matcap_as_lampadas_ficam() {
    use ph2d_panel_sculpt3d::state::LightMode;

    // O que o matcap de facto torna inerte: o `mesh.wgsl` nem chega a estes
    // termos. A subsuperfície entra aqui desde 2026-08-30, pela mesma lei.
    let so_do_rig = [ids::SCULPT3D_ENV, ids::SCULPT3D_SSS];
    // O que escreve o rig do DOCUMENTO — ver o doc acima.
    let lampadas = [ids::SCULPT3D_LIGHT_AZ, ids::SCULPT3D_LIGHT_ELEV];

    // ⚠️ **O estado de FÁBRICA é um matcap**, e é ali que o dono não as achou.
    // Se ele deixar de o ser, é esta asserção que muda primeiro.
    assert_eq!(Sculpt3dUi::default().lighting, LightMode::Matcap(0));

    for (lighting, quer_o_estudio) in [
        (LightMode::Rig, true),
        // ⚠️ O modo PLANO esconde o estúdio pela razão levada ao extremo: ali
        // não há luz nenhuma a ler. Ele entrou na tabela em 2026-09-20.
        (LightMode::Flat, false),
        (LightMode::Matcap(0), false),
        (LightMode::Matcap(3), false),
    ] {
        let (mut host, mut state) = arrange(Sculpt3dUi {
            lighting,
            ..Sculpt3dUi::default()
        });
        let painted = host.paint::<Sculpt3dPanel>(&mut state, VIEWPORT);
        let pintada = |id| painted.iter().any(|(pid, _)| *pid == id);
        for id in so_do_rig {
            assert_eq!(
                pintada(id),
                quer_o_estudio,
                "com a luz {lighting:?} a row {id:?} devia {}",
                if quer_o_estudio { "estar lá" } else { "sumir" }
            );
        }
        for id in lampadas {
            assert!(
                pintada(id),
                "com a luz {lighting:?} a pista {id:?} tem de ser PINTADA: ela \
                 escreve o rig do documento, e o rig acende os objectos assados \
                 a cada quadro — esconde^-la e' esconder um controlo VIVO"
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
        // porta, e a demão é um carimbo que **não** o lê (o verbo *Layer* mede
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
/// Blender tem as duas metades pela mesma razão (o factor de frente-de-face é
/// propriedade da lei e existe sempre; uma opção do pincel, *Front Faces Only*,
/// decide se ele corre).
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

/// ⭐⭐⭐ **O `Connected Only` existe em todo verbo MENOS o Cloth.**
///
/// ⚠️ **A varredura genérica deste ficheiro é cega a ele** — ele não é uma `Row`
/// (é um toggle, não um número) —, então sem este gate apagar a metade que o
/// pinta deixa todos os outros verdes.
#[test]
fn the_connected_only_switch_exists_for_every_verb_but_the_cloth() {
    let mut seen = (false, false);
    for verb in Verb::ALL {
        let mut ui = Sculpt3dUi::default();
        ui.brush.verb = verb;
        let offers = ui.brush.offers_surface_only();
        let (mut host, mut state) = arrange(ui.clone());
        let painted = host.paint::<Sculpt3dPanel>(&mut state, VIEWPORT);
        assert_eq!(
            painted
                .iter()
                .any(|(pid, _)| *pid == ids::SCULPT3D_SURFACE_ONLY),
            offers,
            "com {verb:?} a caixa `Connected Only` devia {}",
            if offers { "estar lá" } else { "sumir" }
        );
        if offers {
            seen.0 = true;
        } else {
            seen.1 = true;
        }
    }
    // ⚠️ **O CONTROLE das duas pontas:** um `offers` constante deixaria o laço
    // acima verde afirmando nada. ⛔ E o `false` só existe porque o `Verb::Cloth`
    // desvia antes do `dab_core` — se algum dia ele passar a lá entrar, este
    // controle reprova e é a pergunta certa a fazer.
    assert!(
        seen.0 && seen.1,
        "a varredura não achou os dois casos: {seen:?} -- o Cloth e' o unico verbo \
         que NAO oferece a caixa, e sem ele o laco nao discrimina nada"
    );
}

/// ⭐⭐⭐ **E O CLIQUE CHEGA AO CAMPO DO PINCEL** — a metade que nenhum gate de
/// registo vê.
///
/// ⛔⛔ **Este gate nasceu de uma mutação SOBREVIVENTE** (2026-09-10): apagar a
/// entrada do `Connected Only` da tabela `TOGGLES` — ou seja, a caixa pintada,
/// registada, e o clique a não virar nada — deixava **toda** a suíte verde. É a
/// família do *«dreno de um braço só»* que o `CLAUDE.md` §5.0 nomeia: *nenhum
/// instrumento desta casa pergunta se o VALOR chega a um consumidor*, e o
/// `every_painted_control_is_clickable_where_it_is_drawn` prova que o clique
/// **chega ao painel**, nunca que a escrita dele chega ao pincel.
#[test]
fn the_connected_only_switch_flips_the_brush_field() {
    for before in [false, true] {
        let mut ui = Sculpt3dUi::default();
        ui.brush.surface_only = before;
        assert!(
            ui.brush.offers_surface_only(),
            "a fixture perdeu a premissa: o verbo de omissao tem de oferecer a caixa"
        );
        let (mut host, mut state) = arrange(ui.clone());
        host.apply_panel_event::<Sculpt3dPanel>(
            &mut state,
            WidgetEvent::Click(ids::SCULPT3D_SURFACE_ONLY),
        );
        let Sculpt3dIntent::SetUi(got) = only_intent("surface_only") else {
            panic!("o `Connected Only` enfileirou o tipo errado de intent");
        };
        let mut want = ui;
        want.brush.surface_only = !before;
        assert_eq!(
            got, want,
            "o `Connected Only` nao alternou, ou levou um vizinho -- e uma caixa que \
             nao vira nada le-se exactamente como uma que funciona"
        );
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
    // ⚠️ **O `matcap` tambem e' DECLARADO, e pela mesma razao que o `sss`.** Ate' 2026-08-30 esta
    // fixtura herdava-o do default — que e' `Some(0)`, ou seja **ligado** — e sob um matcap a
    // subsuperficie inteira e' inalcancavel: o `mesh.wgsl` devolve na linha 879 e as duas funcoes
    // de SSS so' sao chamadas na 898 e na 904. ⇒ a pista passou a seguir tambem o `under_the_rig`,
    // como as pistas de lampada vizinhas ja' seguiam, e esta fixtura tinha de dizer em que estado
    // a pergunta faz sentido.
    // ⭐ E' a disciplina que o comentario acima ja' pregava, aplicada ao segundo campo: *uma
    // fixtura que chega ao estado por omissao inverte de sentido no dia em que o default se move.*
    let mut ui = Sculpt3dUi {
        sss: 0.0,
        lighting: ph2d_panel_sculpt3d::state::LightMode::Rig,
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
///
/// ⛔⛔⛔ **E a premissa deste gate MORREU em 2026-09-15, medida: nem todo pincel
/// tem força.** O censo dos knobs lê o `Density` a arrastar o `Strength` de
/// `0,1` a `1,0` com desvio **`0,000e0`** no barro — o efeito dele é sobre a
/// TOPOLOGIA, e o dab sai antes de a cadeia de peso existir. *Esconder os dois
/// que todo pincel tem é amputação; pintar um que o barro não sente é uma
/// promessa que a ferramenta não cumpre*, e o `Density` tem o `Detail` no lugar.
///
/// ⭐⭐ **A excepção é DERIVADA e não uma lista à mão** — ela sai da mesma porta
/// que o painel consulta ([`ph2d_sculpt3d::Verb::a_forca_chega_ao_barro`]). Uma
/// lista aqui e um predicado lá seriam duas respostas à mesma pergunta, e a que
/// o artista vê é a que envelhece.
///
/// ⛔⛔⛔ **E ELA MORREU OUTRA VEZ em 2026-09-15, no mesmo dia e do outro lado:
/// nem todo verbo é um PINCEL.** O [`ph2d_sculpt3d::Verb::BoxTrim`] não tem
/// raio nenhum — o que delimita o efeito dele é a FORMA que a mão desenha —, e
/// quem o apanhou foi o censo dos knobs mortos na primeira corrida com o verbo
/// novo (`("Box Trim", "panel.sculpt3d.radius")`). ⇒ as **duas** pistas passam a
/// ser afirmadas pela porta do motor, e a frase *«o raio é de TODOS»* fica aqui
/// como contraste. *Uma asserção incondicional é uma premissa à espera do
/// primeiro membro que não couber nela.*
///
/// ⚠️ **As DUAS metades da população estão afirmadas**, senão um predicado que
/// respondesse `false` sempre (ou `true` sempre) passaria este gate: cada pista
/// esconde-se em **pelo menos um** verbo e aparece na **grande maioria**.
#[test]
fn the_basic_level_never_hides_the_two_knobs_every_brush_has() {
    let (mut com_forca, mut sem_forca) = (0usize, 0usize);
    for v in Verb::ALL {
        let mut ui = Sculpt3dUi::default();
        ui.brush.verb = v;
        ui.ui_level = UiLevel::Basic;
        let raio = rows::rows()
            .find(|r| r.slider == ids::SCULPT3D_RADIUS)
            .expect("na tabela");
        assert_eq!(
            raio.visible(&ui),
            v.o_raio_chega_ao_barro(),
            "`{}` e o {} discordam sobre o raio — a pista é pintada exactamente \
             onde a lei a lê",
            raio.label,
            v.label()
        );
        let forca = rows::rows()
            .find(|r| r.slider == ids::SCULPT3D_STRENGTH)
            .expect("na tabela");
        if v.a_forca_chega_ao_barro() {
            com_forca += 1;
            assert!(
                forca.visible(&ui),
                "`{}` sumiu em Basic com o {} em mãos, e a lei dele LÊ a força",
                forca.label,
                v.label()
            );
        } else {
            sem_forca += 1;
            assert!(
                !forca.visible(&ui),
                "o {} não sente a força (medido `0,000e0` no barro) e o painel \
                 pinta-a na mesma — a promessa que a ferramenta não cumpre",
                v.label()
            );
        }
    }
    // ⭐ **E a população do RAIO também**, pela mesma razão: sem esta metade um
    // `o_raio_chega_ao_barro` constante passaria o `assert_eq!` de cima.
    let sem_raio = Verb::ALL
        .into_iter()
        .filter(|v| !v.o_raio_chega_ao_barro())
        .count();
    assert!(
        sem_forca >= 1 && com_forca >= Verb::ALL.len() - 3 && (1..=3).contains(&sem_raio),
        "a população está torta: {sem_forca} sem força, {com_forca} com ela e \
         {sem_raio} sem raio — um predicado constante passaria este gate"
    );
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

/// **A DUREZA É OFERECIDA ONDE O DAB LÊ A DISTÂNCIA, E EM LUGAR NENHUM MAIS.**
///
/// ⚠️ **A metade negativa é a wave, e ela é o INVERSO da fileira abaixo.** A
/// largura do campo existe *onde o campo corre*; a dureza existe *onde ele NÃO
/// corre* — porque com um campo armado a curva do dab é o
/// `kelvinlet::rim_landing` e o `shaped_distance` não é chamado. As duas leem a
/// MESMA porta do motor (`RefMode::field`), e é isso que impede a segunda de
/// nascer desalinhada da primeira.
///
/// ⚠️ **A inércia foi MEDIDA pela porta do produto antes de a fileira sumir** —
/// `ph2d-sculpt3d/tests/it/measure_where_the_curve_knobs_reach.rs`, gate
/// `neither_curve_knob_reaches_an_elastic_field`: dois valores de dureza dão o
/// mesmo barro **ao bit** sob campo, e o MESMO verbo no `s-mode` os separa. Sem
/// essa medição isto seria esconder um controle por palpite, que é exatamente o
/// que a cerca do Falloff (`the_basic_level_never_hides_the_curve_that_shapes_the_dab`)
/// já cobrou uma vez nesta mesma seção.
///
/// ⚠️ **PRO nas duas metades**, pela razão que o `a_conditional_row_is_absent_with_the_wrong_tool`
/// já documenta: a dureza é uma row de Pro, então em Basic a metade negativa
/// passaria pelo motivo ERRADO.
#[test]
fn the_hardness_row_is_offered_only_where_the_dab_reads_the_distance() {
    let armed = |mode: RefMode| {
        let mut ui = Sculpt3dUi {
            ui_level: UiLevel::Pro,
            ..Default::default()
        };
        ui.set_mode_of(Verb::Move, mode);
        ph2d_panel_sculpt3d::state::switch_verb(&mut ui, Verb::Move);
        ui
    };

    // A fixture tem de conter o fenômeno, e a premissa é DECLARADA.
    let with_field = armed(RefMode::L);
    assert!(
        with_field.brush.mode.field(with_field.brush.verb).is_some(),
        "a fixture perdeu a premissa: o Move em L tem de declarar campo"
    );
    let (mut host, mut state) = arrange(with_field);
    let painted = host.paint::<Sculpt3dPanel>(&mut state, VIEWPORT);
    for id in [ids::SCULPT3D_HARDNESS, ids::SCULPT3D_HARDNESS_NUM] {
        assert!(
            !painted.iter().any(|(pid, _)| *pid == id),
            "{id:?} foi pintado sob um campo elástico — ali a curva é o \
             `rim_landing` e o `shaped_distance` nem é chamado"
        );
    }

    // CONTROLE 1 — o MESMO verbo no `s-mode`: o dab lê a distância, a row volta.
    let without = armed(RefMode::S);
    assert!(
        without.brush.mode.field(without.brush.verb).is_none(),
        "a fixture perdeu o controle: o Move em S não pode declarar campo"
    );
    let (mut host, mut state) = arrange(without);
    let painted = host.paint::<Sculpt3dPanel>(&mut state, VIEWPORT);
    assert!(
        painted
            .iter()
            .any(|(pid, _)| *pid == ids::SCULPT3D_HARDNESS),
        "o `s-mode` do MESMO verbo lê a dureza e a fileira não foi pintada"
    );

    // CONTROLE 2 — o verbo que PINTA O CANAL a lê (o `shaped_distance` roda
    // antes da curva própria da máscara), e é a assimetria que separa esta row
    // do Falloff ao lado dela.
    let mut ui = Sculpt3dUi {
        ui_level: UiLevel::Pro,
        ..Default::default()
    };
    ph2d_panel_sculpt3d::state::switch_verb(&mut ui, Verb::Mask);
    let (mut host, mut state) = arrange(ui);
    let painted = host.paint::<Sculpt3dPanel>(&mut state, VIEWPORT);
    assert!(
        painted
            .iter()
            .any(|(pid, _)| *pid == ids::SCULPT3D_HARDNESS),
        "a máscara LÊ a dureza (ela remapeia a distância que a curva do canal \
         consome) e a fileira sumiu"
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
/// ⚠️ **A régua é a REFERÊNCIA, não o gosto.** Medido nos painéis comuns de
/// pintura do Blender: o *Falloff* **não** é desenhado dentro das definições
/// avançadas do pincel — ele é painel de primeira classe, e no
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
    assert_eq!(
        ids::SCULPT3D_CLOTH_FILTER_KIND.len(),
        ClothFilterKind::ALL.len(),
        "a fileira de TECIDO e a lista de tipos têm tamanhos diferentes — algum tipo é \
         inalcançável, ou algum chip nomeia um que não existe"
    );
    // ⭐ **As DUAS famílias no mesmo laço** (espec §7): a lei do filtro é uma
    // união, e um gate que só varresse uma delas deixaria cinco chips sem régua.
    let todas: Vec<(ph2d_a11y::NodeId, FilterLaw)> = FilterKind::ALL
        .into_iter()
        .enumerate()
        .map(|(i, k)| (ids::SCULPT3D_FILTER_KIND[i], FilterLaw::Mesh(k)))
        .chain(
            ClothFilterKind::ALL
                .into_iter()
                .enumerate()
                .map(|(i, k)| (ids::SCULPT3D_CLOTH_FILTER_KIND[i], FilterLaw::Cloth(k))),
        )
        .collect();
    for (n, &(chip, kind)) in todas.iter().enumerate() {
        let mut ui = Sculpt3dUi::default();
        // Um verbo SEM lei própria de propósito: as leis sem verbo só são
        // alcançáveis se o selector não depender de quem está em mãos.
        ui.brush.verb = Verb::Draw;
        // ⚠️ **A fixture começa numa lei DIFERENTE da que o chip escreve**,
        // senão a iteração `i = 0` é verde por vácuo: o `default()` já vale
        // `ALL[0]`, e um chip que não escrevesse nada passaria a asserção.
        ui.filter_law = todas[(n + 1) % todas.len()].1;
        // ⚠️ **O retrato chega ARMADO**, e o gate acima é que prova que o
        // interruptor leva até aqui: o `filter_armed` é campo do SNAPSHOT (o
        // que a cena responde), não do `Sculpt3dUi` que o painel devolve, então
        // clicar o toggle nesta fixture não o move -- ele enfileira o intent e
        // a cena é quem decide.
        let mut snap = snapshot(ui, true);
        snap.filter_armed = true;
        let (mut host, mut state) = arrange_with(snap);

        let painted = host.paint::<Sculpt3dPanel>(&mut state, VIEWPORT);
        let onde = |ps: &[(ph2d_a11y::NodeId, Rect)], id| {
            ps.iter()
                .find(|(i, _)| *i == id)
                .map(|(_, r)| *r)
                .unwrap_or_else(|| panic!("{kind:?}: o chip não foi pintado com o filtro armado"))
        };
        // ⭐⭐ **ROLAR FAZ PARTE DA PERGUNTA.** O corpo do painel é RECORTADO
        // (`push_clip`), então um chip pintado abaixo da dobra existe e não é
        // clicável — e a fileira do tecido caiu exactamente aí quando nasceu.
        // *Um gate que só clicasse no que já estava à vista responderia «morto»
        // sobre um controlo vivo a uma volta da roda do rato.*
        //
        // ⚠️ **O alvo é DERIVADO, e não um número escolhido:** é a linha do
        // PRIMEIRO chip da fileira de malha, que as iterações de cima já provaram
        // ser clicável nesta fixture.
        let alvo = onde(&painted, ids::SCULPT3D_FILTER_KIND[0]).y;
        let scroll = (onde(&painted, chip).y - alvo).max(0.0);
        host.set_panel_scroll(ph2d_editor_core::ids::SCULPT3D_PANEL, scroll);
        let painted = host.paint::<Sculpt3dPanel>(&mut state, VIEWPORT);
        let rect = onde(&painted, chip);
        let evs = host.click_at(rect.x + rect.w * 0.5, rect.y + rect.h * 0.5);
        assert!(
            evs.iter()
                .any(|e| matches!(e, WidgetEvent::Click(id) if *id == chip)),
            "{kind:?}: o chip está pintado e morto sob o mouse"
        );
        for e in evs {
            let _ = host.apply_panel_event::<Sculpt3dPanel>(&mut state, e);
        }
        let Sculpt3dIntent::SetUi(got) = only_intent("chip de lei") else {
            panic!("{kind:?}: o chip enfileirou o intent errado");
        };
        assert_eq!(
            got.filter_law, kind,
            "{kind:?}: o chip escreveu OUTRA lei -- o artista escolhe uma e recebe outra"
        );
    }
}

/// ⭐⭐⭐ **A RAZÃO DE A CURVA SER INERTE CHEGA A PIXEL** — e a régua são os
/// GLIFOS, não a banda reservada.
///
/// ⛔⛔ **É o achado §4.2 da auditoria do `source.lsystem` aplicado aqui:** um
/// gate que prometia medir *«a queixa chega a PIXEL»* media o `y +=`, que não é
/// a pintura — *apagar o texto inteiro deixava-o verde*. O que conta texto é
/// `resources.glyphs`: o Vello encaminha texto por `draw_glyphs`, e **nenhum
/// glifo entra na contagem de caminhos**.
///
/// ⚠️ **A fixtura é o MESMO verbo com o MESMO número de fileiras**, e só o
/// `Segments` muda: com `1` a curva é inerte (medido `0,000e0` no barro em
/// todas as doze curvas) e com `2` ela chega (`2,755e-1`). *Comparar dois
/// verbos diferentes mediria as fileiras próprias de cada um, e não a nota.*
#[test]
fn a_razao_da_curva_inerte_chega_a_pixel() {
    let com_segmentos = |segmentos: u32| {
        let mut ui = Sculpt3dUi::default();
        ui.brush.verb = Verb::Pose;
        ui.brush.pose.deformacao = ph2d_sculpt3d::PoseDeformacao::Torcer;
        ui.brush.pose.segmentos = segmentos;
        ui.ui_level = UiLevel::Pro;
        let (mut host, mut state) = arrange(ui);
        host.paint_and_count_geometry::<Sculpt3dPanel>(&mut state, VIEWPORT)
            .0
    };
    let (inerte, viva) = (com_segmentos(1), com_segmentos(2));
    assert!(
        inerte > viva,
        "com UM segmento a curva não faz nada e o painel tem de o dizer: \
         {inerte} glifos contra {viva} — a nota não chegou a pixel"
    );
    // ⭐ E o CONTROLO de que o balde se enche pelo texto certo e não por ruído:
    // a diferença tem de ter o tamanho de uma frase, não de um dígito.
    assert!(
        inerte - viva > 20,
        "a diferença é de {} glifo(s) — isso é um número a mudar de largura, \
         não uma frase",
        inerte - viva
    );
}

/// ⭐⭐ **GATE — com o BOX TRIM na mão, a fileira das FORMAS é pintada e responde
/// ao ponteiro, e a pista do traço aparece SÓ no laço.**
///
/// ⛔⛔ **Irmão exacto dos gates do esfregão e do projectar, e ele existe porque
/// aquele defeito aconteceu SETE vezes nesta crate** — a última custou o report
/// *«os outros 2 botões ainda não funcionam»*. Um controlo nunca pintado e um
/// morto sob o dedo dão o MESMO report, e só o gesto REAL os separa.
///
/// ⚠️ **A segunda metade é a que o dono pediu por escrito** (*«Em laço um
/// parâmetro para suavizar o traço»*): a pista tem de aparecer com o laço e
/// **não** com a caixa — suavizar dois pontos é o controlo morto que esta casa
/// varre a cada wave, e pintá-lo seria uma promessa que a ferramenta não cumpre.
#[test]
fn every_trim_control_is_clickable_where_it_is_drawn() {
    use ph2d_sculpt3d::TrimForma;

    let mut ui = Sculpt3dUi::default();
    ph2d_panel_sculpt3d::state::switch_verb(&mut ui, Verb::BoxTrim);
    ui.ui_level = UiLevel::Pro;
    let (mut host, mut state) = arrange(ui);
    let painted = host.paint::<Sculpt3dPanel>(&mut state, VIEWPORT);

    let want: Vec<(String, ph2d_a11y::NodeId)> = TrimForma::ALL
        .into_iter()
        .enumerate()
        .map(|(i, f)| {
            (
                format!("trim shape {}", f.label()),
                ids::SCULPT3D_TRIM_FORMA[i],
            )
        })
        .collect();
    assert_eq!(
        want.len(),
        3,
        "o dono pediu TRÊS botões (box, circle, laço) — a fixtura deixou de \
         conter o fenómeno"
    );
    for (name, id) in &want {
        assert!(
            painted.iter().any(|(pid, _)| pid == id),
            "`{name}` ({id:?}) devia estar pintado com o Box Trim na mão"
        );
        let rect = painted
            .iter()
            .rev()
            .find(|(pid, _)| pid == id)
            .map(|(_, r)| *r)
            .expect("pintado logo acima");
        let (cx, cy) = (rect.x + rect.w * 0.5, rect.y + rect.h * 0.5);
        assert!(
            host.click_at(cx, cy)
                .iter()
                .any(|e| matches!(e, WidgetEvent::Click(c) if *c == *id)),
            "clicar `{name}` no centro pintado não produziu Click — ele está no \
             índice de hit e morto sob o dedo"
        );
    }

    // ⭐ **A pista do traço segue a FORMA**, e as duas metades são obrigatórias:
    // sem a negativa, uma pista pintada sempre passaria.
    let pintada = |forma: TrimForma| {
        let mut u = Sculpt3dUi::default();
        ph2d_panel_sculpt3d::state::switch_verb(&mut u, Verb::BoxTrim);
        u.brush.trim_forma = forma;
        u.ui_level = UiLevel::Pro;
        rows::rows()
            .find(|r| r.slider == ids::SCULPT3D_TRIM_SMOOTH)
            .expect("na tabela")
            .visible(&u)
    };
    assert!(
        pintada(TrimForma::Laco),
        "a pista da suavização sumiu com o LAÇO na mão, que é o único que a lê"
    );
    for muda in [TrimForma::Caixa, TrimForma::Circulo] {
        assert!(
            !pintada(muda),
            "a pista da suavização é pintada com o {} na mão — ele guarda DOIS \
             pontos, e não há traço a suavizar",
            muda.label()
        );
    }
    // ⛔ E com um PINCEL na mão ela não existe de todo.
    let mut outro = Sculpt3dUi::default();
    ph2d_panel_sculpt3d::state::switch_verb(&mut outro, Verb::Draw);
    outro.ui_level = UiLevel::Pro;
    assert!(
        !rows::rows()
            .find(|r| r.slider == ids::SCULPT3D_TRIM_SMOOTH)
            .expect("na tabela")
            .visible(&outro),
        "a pista da suavização do corte aparece com um PINCEL na mão"
    );
}

/// ⭐⭐⭐ **GATE — os controlos PRÓPRIOS de um pincel cabem no ENCAIXE.**
///
/// # Ele reproduz um report do dono, à letra
///
/// 2026-09-17: *«não vejo os botões mas deveriam ficar na secção detail»*. Os
/// dois chips novos estavam **pintados, hit-indexados e vivos sob o dedo** — o
/// gate de costura provava as três coisas — e caíam em `y = 1049` e `y = 858`,
/// **abaixo da dobra de um encaixe real**. O `Gap` que o primeiro acompanha está
/// em `y = 636`: **413 px** de distância.
///
/// ⚠️ **O selector que provocou a medição (`Gap Law`) foi APAGADO no mesmo dia**,
/// por veredito do dono sobre o smoke — e este gate **fica**, porque o que ele
/// mede não era aquele chip: era *os controlos próprios de um pincel caberem no
/// encaixe*, e o `Ray Direction` e o `Search Both Ways` nasciam em `967` e `1001`
/// muito antes de ele existir. *Um gate cujo sujeito era um exemplo sobrevive ao
/// exemplo; um cujo sujeito era o exemplo teria de morrer com ele.*
///
/// ⛔⛔ **E NENHUM gate desta crate o podia ver, por construção:** os de costura
/// pintam numa [`VIEWPORT`] de **`2400`** px de altura, escolhida para caber
/// tudo. *Uma régua calibrada num tamanho que o artista não tem é cega
/// exactamente onde a escolha acontece* — a mesma família do gate da `line/UIUX`
/// que media a largura de omissão enquanto o dono tinha outra.
///
/// # A altura, e de onde ela vem
///
/// `880` px é o encaixe medido desta casa — o número que o `CLAUDE.md` §5 já
/// publica ao dizer que *«o nó desenha 1083 px num dock de 880»*. ⛔ Não é um
/// número escolhido aqui.
///
/// # ⭐⭐⭐ A CATRACA ESTÁ A **ZERO** desde 2026-09-20, e quem a desceu foi o CENSO DE OBSOLESCÊNCIA
///
/// Ela nasceu em 2026-09-17 com **cinco** pincéis cujos controlos próprios caíam
/// abaixo da dobra, e o doc dela dizia: *«curá-los é mexer na disposição de cinco
/// ferramentas que o dono já aprovou em smoke, e isso é wave dele»*, com o preço
/// nomeado — *«cada `+23` é mais rolagem entre o artista e um controlo que ele tem
/// de alcançar, e a "etapa de arrumação" que o dono anunciou passa a ter um número»*.
///
/// ⭐ **A etapa de arrumação chegou** (ordem do dono, 2026-09-20): o selector de
/// pincéis saiu do painel para a paleta, e a secção `Tool` encolheu `276 px`. Os
/// **cinco** passaram a caber, e a lista está **vazia**.
///
/// ⚠️⚠️ **Não fui eu que medi a descida — foi a METADE DA OBSOLESCÊNCIA deste
/// gate**, na primeira corrida a seguir à cirurgia:
///
/// ```text
/// Cloth: ele passou a caber no encaixe (y = 854 <= 880) — APAGUE a linha dele
/// da catraca, senao ela vira licenca
/// ```
///
/// ⭐⭐ E a lista vazia é a régua mais apertada que existe: com ela, **o `else`
/// abaixo exige que TODOS os sete caibam**, e nenhuma linha sobra onde registar
/// um que volte a não caber. *Uma catraca sem censo de obsolescência não desce:
/// ela vira licença* — esta desceu porque o tinha.
#[test]
fn os_controlos_proprios_de_um_pincel_cabem_no_encaixe() {
    /// O encaixe MEDIDO desta casa — ver o doc.
    const ALTURA_DO_ENCAIXE_PX: f32 = 880.0;
    /// A folga do ratchet: a disposição é aritmética de `f32` sobre tokens, e
    /// um pixel de deriva não é uma regressão.
    const FOLGA_PX: f32 = 1.0;
    /// ⛔ **SÓ ENCOLHE** — os cinco que passam a dobra hoje, com o número medido
    /// em 2026-09-17. Curar um deles é apagar a linha dele daqui.
    ///
    /// ⛔⛔⛔ **E ela SUBIU `+23 px` em 2026-09-19, por ORDEM DO DONO — a única
    /// espécie de subida que esta casa admite.** Ele mandou *«implementar o
    /// pincel de pintura, de Blur e Smear»*, e um verbo novo precisa de um
    /// CHIP: a grelha de verbos é adaptativa à LARGURA dos rótulos, o 36.º
    /// rebentou a linha, e uma fileira de chips a mais na secção *Tool* empurra
    /// **todos** os pincéis para baixo — os cinco pela mesma quantidade.
    ///
    /// ⚠️⚠️ **O preço é REAL e não some por estar registado:** o encaixe mede
    /// `880` e o pior destes já estava em `1 107`. Cada `+23` é mais rolagem
    /// entre o artista e um controlo que ele tem de alcançar, e a «etapa de
    /// arrumação» que o dono anunciou passa a ter um número. *Registar uma
    /// subida não é curá-la; é impedir que a seguinte passe calada.*
    ///
    /// ⚠️ **Blur e Smear ainda não estão no catálogo** — se os rótulos deles
    /// rebentarem outra linha, esta tabela volta a acusar, e é isso que se quer.
    // ⭐⭐⭐ **VAZIA desde 2026-09-20** — ver o cabeçalho. ⛔ Ela só ENCOLHE: uma
    // entrada nova aqui é um pincel que deixou de caber, e a cura é a disposição,
    // nunca a linha.
    const ACIMA_DA_DOBRA: [(&str, f32); 0] = [];

    let proprios: [&[ph2d_a11y::NodeId]; 7] = [
        &ids::SCULPT3D_CLOTH_MODE[..],
        &ids::SCULPT3D_BOUNDARY_FALLOFF[..],
        &ids::SCULPT3D_SMEAR_MODE[..],
        &ids::SCULPT3D_TRIM_FORMA[..],
        &ids::SCULPT3D_PLANO_INVERSAO[..],
        &[ids::SCULPT3D_PROJECT_BIDIR][..],
        &[ids::SCULPT3D_POSE_ROT_LOCK][..],
    ];
    let verbos = [
        Verb::Cloth,
        Verb::Boundary,
        Verb::SmearMultires,
        Verb::BoxTrim,
        Verb::Plane,
        Verb::SceneProject,
        Verb::Pose,
    ];
    let mut vistos = 0usize;
    for verbo in verbos {
        let mut ui = Sculpt3dUi::default();
        ph2d_panel_sculpt3d::state::switch_verb(&mut ui, verbo);
        // ⚠️ A escala é a deformação em que a pose mostra MAIS controlos —
        // medir a de fábrica esconderia dois deles.
        ui.brush.pose.deformacao = ph2d_sculpt3d::PoseDeformacao::Escalar;
        let (mut host, mut state) = arrange(ui);
        let painted = host.paint::<Sculpt3dPanel>(&mut state, VIEWPORT);
        let mut fundo = 0.0f32;
        for lista in proprios {
            for id in lista {
                if let Some((_, r)) = painted.iter().rev().find(|(p, _)| p == id) {
                    fundo = fundo.max(r.y + r.h);
                }
            }
        }
        assert!(
            fundo > 0.0,
            "{verbo:?}: nenhum controlo proprio foi pintado — a fixtura deixou \
             de conter o fenomeno"
        );
        vistos += 1;
        let nome = format!("{verbo:?}");
        if let Some((_, tecto)) = ACIMA_DA_DOBRA.iter().find(|(n, _)| *n == nome) {
            assert!(
                fundo <= tecto + FOLGA_PX,
                "{nome}: o ultimo controlo proprio desceu para y = {fundo:.0}, e a \
                 catraca registou {tecto:.0} — ela SO' ENCOLHE"
            );
            assert!(
                fundo > ALTURA_DO_ENCAIXE_PX,
                "{nome}: ele passou a caber no encaixe (y = {fundo:.0} <= \
                 {ALTURA_DO_ENCAIXE_PX:.0}) — APAGUE a linha dele da catraca, \
                 senao ela vira licenca"
            );
        } else {
            assert!(
                fundo <= ALTURA_DO_ENCAIXE_PX,
                "{nome}: o ultimo controlo proprio cai em y = {fundo:.0}, abaixo da \
                 dobra de um encaixe de {ALTURA_DO_ENCAIXE_PX:.0} px — o artista \
                 nao o ve'. Ou o sobe para junto do knob que ele governa, ou o \
                 declara na catraca com o numero MEDIDO"
            );
        }
    }
    assert_eq!(
        vistos, 7,
        "o censo correu {vistos} pinceis e a populacao com controlos proprios e' 7"
    );
}

/// ⭐⭐⭐ **GATE — o PENTE é ALCANÇÁVEL, SOME para quem o ignora, e DIZ porque
/// dorme.**
///
/// # Porque são TRÊS metades, e nenhuma basta sozinha
///
/// ⛔⛔ **Esta crate pagou SETE vezes o mesmo report** (*«os outros 2 botões
/// ainda não funcionam»*): *um controlo nunca pintado e um morto sob o dedo dão
/// o MESMO relato*, e só o gesto REAL os separa. A primeira metade pinta o
/// painel de verdade e depois **arrasta** a pista pelo despachante.
///
/// ⚠️ **A segunda é DERIVADA do motor, nunca de uma lista escrita à mão:** a
/// população de quem ignora o pente é um facto **medido no oráculo** (cinco
/// verbos, saída byte-idêntica nos dois lados do controlo), e uma cópia dela
/// aqui divergiria no dia em que alguém a re-medisse. O censo compara, verbo a
/// verbo, o que o painel MOSTRA com o que o motor RESPONDE — ⛔ e o piso de
/// população é o que impede que ele fique verde sobre um `Verb::ALL` que
/// encolheu.
///
/// ⚠️⚠️ **A terceira é a metade da cerca que o `show` não consegue exprimir.** A
/// topologia dinâmica ARMADA é a única pré-condição de estado do pente (espec
/// §2.1: desarmada, os dois lados do controlo dão a MESMA malha byte a byte), e
/// o `dyntopo` é um FACTO do [`Sculpt3dSnapshot`] — o `show` de uma
/// [`rows::Row`] só vê o [`Sculpt3dUi`]. ⇒ a pista fica e o painel **diz
/// porquê**, e a régua é a contagem de GLIFOS: *a banda reservada não é a
/// pintura*, e um gate que medisse o `y +=` ficaria verde com o texto apagado.
///
/// ⛔ **E a metade NEGATIVA da terceira é metade do valor:** com o interruptor
/// armado o painel cala-se, e com um verbo que ignora o pente ele cala-se
/// **também com o interruptor desarmado** — senão o app explicaria a inércia de
/// um controlo que nem sequer está na tela.
#[test]
fn o_pente_e_alcancavel_some_para_quem_o_ignora_e_diz_porque_dorme() {
    let retrato = |verb: Verb, dyntopo: bool| -> Sculpt3dSnapshot {
        let mut ui = Sculpt3dUi::default();
        // ⚠️ Pela PORTA do produto (`switch_verb`), nunca escrevendo
        // `ui.brush.verb` à mão: a troca de ferramenta é que traz o pincel do
        // slot, e um arranjo montado à mão mede outro programa.
        ph2d_panel_sculpt3d::state::switch_verb(&mut ui, verb);
        Sculpt3dSnapshot {
            dyntopo,
            ..snapshot(ui, true)
        }
    };

    // ── (a) PINTADO, e a pista escreve o campo DELA ──────────────────────────
    let (mut host, mut state) = arrange_with(retrato(Verb::Crease, true));
    let painted = host.paint::<Sculpt3dPanel>(&mut state, VIEWPORT);
    for (nome, id) in [
        ("a pista do pente", ids::SCULPT3D_PENTE),
        ("o chip do pente", ids::SCULPT3D_PENTE_NUM),
    ] {
        assert!(
            painted.iter().any(|(pid, _)| *pid == id),
            "{nome} nao foi pintado com a topologia dinamica armada"
        );
    }
    host.set_slider_value(ids::SCULPT3D_PENTE, 0.5);
    assert_eq!(
        host.apply_panel_event::<Sculpt3dPanel>(
            &mut state,
            WidgetEvent::ValueChanged(ids::SCULPT3D_PENTE),
        ),
        EventOutcome::Consumed,
        "a pista do pente ignorou um arrasto REAL — falta o braco dela no event.rs"
    );
    let Sculpt3dIntent::SetUi(got) = only_intent("a pista do pente") else {
        panic!("a pista do pente enfileirou o tipo errado de intent");
    };
    assert!(
        (got.brush.pente - 0.5).abs() < 1e-4,
        "a pista levou o pente a {} e meio curso significa 0,5",
        got.brush.pente
    );

    // ── (b) O CENSO: o painel mostra exactamente quem o motor diz que honra ──
    let linha = rows::rows()
        .find(|r| r.slider == ids::SCULPT3D_PENTE)
        .expect("o pente esta' na tabela");
    // ⛔ **A PREMISSA que faz a razao do `paint/body.rs` poder perguntar pela
    // PORTA (`rows::penteia`) em vez de por `Row::visible`:** com a fileira em
    // `Basic` as duas coincidem sempre, porque todo nivel a mostra. No dia em
    // que ela subir para `Pro` isto reprova, e a linha do pintor tem de passar a
    // perguntar `visible` — a premissa morre a' vista no diff.
    assert_eq!(
        linha.level,
        UiLevel::Basic,
        "o pente subiu de nivel: a razao pintada no `paint/body.rs` pergunta \
         `rows::penteia` e passaria a prometer uma fileira que o nivel esconde"
    );
    let mut ignoram = Vec::new();
    for verb in Verb::ALL {
        let mut ui = Sculpt3dUi::default();
        ph2d_panel_sculpt3d::state::switch_verb(&mut ui, verb);
        let honra = verb.honra_o_pente();
        assert_eq!(
            linha.visible(&ui),
            honra,
            "{verb:?}: o painel {} a pista do pente e o motor diz que ele {} honra",
            if honra { "esconde" } else { "pinta" },
            if honra { "" } else { "nao" }
        );
        if !honra {
            ignoram.push(verb);
        }
    }
    // ⛔ O piso de populacao: sem ele, um `Verb::ALL` vazio deixaria o laco
    // acima trivialmente verde — a forma de censo que este repo ja' pagou.
    assert_eq!(
        ignoram.len(),
        5,
        "o oraculo mediu CINCO verbos que ignoram o pente e o censo achou {} ({:?}) \
         — ou a medicao mudou (e a espec §6.3 tem de mudar com ela) ou o motor \
         desalinhou-se dela",
        ignoram.len(),
        ignoram
    );

    // ── (c) A RAZÃO chega a PIXEL, e cala-se onde deve ──────────────────────
    let glifos = |verb: Verb, dyntopo: bool| -> u32 {
        let (mut host, mut state) = arrange_with(retrato(verb, dyntopo));
        host.paint_and_count_geometry::<Sculpt3dPanel>(&mut state, VIEWPORT)
            .0
    };
    let (dormente, acordado) = (glifos(Verb::Crease, false), glifos(Verb::Crease, true));
    assert!(
        dormente > acordado + 20,
        "com a topologia dinamica desarmada o painel tem de DIZER que o pente \
         dorme: {dormente} glifos contra {acordado} — isso e' um numero a mudar \
         de largura, nao uma frase"
    );
    let (mudo_off, mudo_on) = (glifos(Verb::Mask, false), glifos(Verb::Mask, true));
    assert!(
        mudo_off.abs_diff(mudo_on) < 5,
        "com um verbo que IGNORA o pente o painel explicou a inercia de um \
         controlo que nem esta' na tela: {mudo_off} glifos contra {mudo_on}"
    );
}

/// ⭐⭐⭐⭐ **GATE — O ROTEIRO DA `=49` DIZ ONDE O `Edge Flow` ESTÁ, e as QUATRO
/// coisas que ele afirma sobre a tela são medidas aqui.**
///
/// Decisão do dono (2026-09-19), depois de ver o preço medido de mover a
/// fileira: **deixar a disposição como está e escrever no roteiro onde ela
/// fica**. ⇒ o passo (1) passou a fazer quatro afirmações sobre a tela, e *uma
/// afirmação sobre a tela mede-se na tela* — eu já disse ao dono que esta
/// fileira vivia no `Pro` (é `Basic`) e que ela estava atrás do sombreado (o
/// sombreado está `286 px` **abaixo** dela).
///
/// # As quatro metades, e porque nenhuma basta
///
/// 1. **o NOME** — o roteiro imprime os rótulos que o painel pinta (se um deles
///    for renomeado, o dono procura uma palavra que não está na tela);
/// 2. **o NÍVEL** — a fileira é `Basic`, logo o roteiro **não** manda trocar de
///    nível; se ela passasse a `Pro`, o passo ficaria impossível em silêncio;
/// 3. **a DOBRA** — ela cai em `1271` contra os `880` do encaixe, e é isso que
///    torna a frase *«role a roda»* necessária em vez de ruído;
/// 4. **a ORDEM** — o que o dono rola até lá é `Tool` → `Brush` → `Symmetry` →
///    `Topology` → `Edge Flow` → `Shading`, e ela lê-se do `y`, ⛔ **nunca** da
///    tabela `SECTIONS`, que está noutra ordem (foi daí que veio o meu erro).
///
/// ⛔⛔ **A metade (3) é uma CATRACA AO CONTRÁRIO, de propósito:** no dia em que
/// a arrumação que o dono anunciou trouxer esta fileira para cima da dobra, ela
/// **reprova** — e a cura é apagar a frase da rolagem do roteiro, nunca afrouxar
/// o número. Por isso ela é uma EQUIVALÊNCIA e não uma desigualdade: hoje
/// apanha quem apague a frase com o botão ainda escondido, e amanhã apanha quem
/// deixe a frase depois de o pôr à vista. *Uma cena que manda rolar à procura de
/// um controlo que já está à vista é a espécie que o `CLAUDE.md` §5.0 chama de
/// pior que uma cena ausente.*
#[test]
fn o_roteiro_da_49_diz_onde_o_edge_flow_esta() {
    /// O encaixe MEDIDO desta casa — o mesmo número do gate irmão
    /// [`os_controlos_proprios_de_um_pincel_cabem_no_encaixe`].
    const ALTURA_DO_ENCAIXE_PX: f32 = 880.0;
    let roteiro = include_str!("../../../ph2d-app-sculpt3d/src/scenes_pente.rs");

    // ── (1) os nomes ───────────────────────────────────────────────────────
    for chave in [
        "panel.sculpt3d.pente",
        "panel.sculpt3d.dyntopo",
        "panel.sculpt3d.section.tool",
        "panel.sculpt3d.section.brush",
        "panel.sculpt3d.section.symmetry",
        "panel.sculpt3d.section.topology",
    ] {
        let rotulo = ph2d_i18n::tr(chave);
        assert!(
            roteiro.contains(&format!("`{rotulo}`")),
            "o roteiro da =49 manda o dono procurar `{rotulo}` e nao o nomeia — \
             ou o rotulo mudou e o roteiro ficou para tras"
        );
    }

    // ── (2) o nível ────────────────────────────────────────────────────────
    let mut ui = Sculpt3dUi::default();
    ph2d_panel_sculpt3d::state::switch_verb(&mut ui, Verb::Draw);
    ui.ui_level = UiLevel::Basic;
    let fileira = rows::rows()
        .find(|r| r.label == "panel.sculpt3d.pente")
        .expect("o painel perdeu a fileira do pente e o roteiro ainda a nomeia");
    assert!(
        fileira.visible(&ui),
        "o roteiro da =49 nao manda trocar de nivel nenhum, e a fileira do pente \
         nao e' pintada no `Basic` com o carimbo na mao — o dono procura e nao acha"
    );

    // ── (3) a dobra e (4) a ordem ──────────────────────────────────────────
    // ⚠️ Com a topologia ARMADA, que é como a `=49` abre.
    let mut snap = snapshot(ui, true);
    snap.dyntopo = true;
    let (mut host, mut state) = arrange_with(snap);
    let painted = host.paint::<Sculpt3dPanel>(&mut state, VIEWPORT);
    let onde = |id: ph2d_a11y::NodeId, nome: &str| -> f32 {
        painted
            .iter()
            .rev()
            .find(|(p, _)| *p == id)
            .map(|(_, r)| r.y)
            .unwrap_or_else(|| panic!("o painel nao pintou `{nome}`"))
    };
    let y_pente = onde(ids::SCULPT3D_PENTE, "Edge Flow");
    assert_eq!(
        y_pente > ALTURA_DO_ENCAIXE_PX,
        roteiro.contains("role a roda"),
        "o passo (1) e a tela discordam: o `Edge Flow` cai em y = {y_pente:.0} \
         contra um encaixe de {ALTURA_DO_ENCAIXE_PX:.0} px. Se ele subiu para \
         cima da dobra, APAGUE a frase da rolagem do roteiro; se a frase saiu \
         com ele ainda escondido, o dono vai procura-lo e nao o achar"
    );
    let escada = [
        (ids::SCULPT3D_SEC_TOOL, "Tool"),
        (ids::SCULPT3D_SEC_BRUSH, "Brush"),
        (ids::SCULPT3D_SEC_SYMMETRY, "Symmetry"),
        (ids::SCULPT3D_SEC_TOPOLOGY, "Topology"),
        (ids::SCULPT3D_PENTE, "Edge Flow"),
        (ids::SCULPT3D_SEC_SHADING, "Shading"),
    ];
    for par in escada.windows(2) {
        let (a, b) = (par[0], par[1]);
        assert!(
            onde(a.0, a.1) < onde(b.0, b.1),
            "o roteiro manda passar `{}` antes de `{}` e a tela pinta-os ao \
             contrario ({:.0} contra {:.0}) — o dono rola para o lado errado",
            a.1,
            b.1,
            onde(a.0, a.1),
            onde(b.0, b.1)
        );
    }
}

/// ⭐⭐⭐⭐ **SONDA — ONDE O `Edge Flow` CAI NO PAINEL, com o pincel de fábrica.**
///
/// Decisão do dono (2026-09-19): *«deixar desligado e tornar o botão achável»*.
/// ⚠️ **Antes de mover nada, a pergunta é a do dock:** a dobra desta casa é
/// `880 px` (o mesmo número do gate irmão), e a pista do pente vive na TERCEIRA
/// secção contínua, atrás do `TOOL`, do `BRUSH` e do `SYMMETRY` — ⛔ e **não**
/// do `SHADING`, que fica `286 px` abaixo dela.
///
/// ⛔ *Uma afirmação sobre a tela mede-se na tela* — eu já disse ao dono que ela
/// estava em `Pro` e ela é `Basic`.
#[test]
#[ignore = "sonda: imprime a posicao, nao afirma nada"]
fn diag_onde_cai_a_pista_do_pente() {
    const ALTURA_DO_ENCAIXE_PX: f32 = 880.0;
    for (nome, dyntopo) in [("dyntopo DESARMADO", false), ("dyntopo armado", true)] {
        let mut ui = Sculpt3dUi::default();
        ph2d_panel_sculpt3d::state::switch_verb(&mut ui, Verb::Draw);
        let mut snap = snapshot(ui, true);
        snap.dyntopo = dyntopo;
        let (mut host, mut state) = arrange_with(snap);
        let painted = host.paint::<Sculpt3dPanel>(&mut state, VIEWPORT);
        let acha = |id: ph2d_a11y::NodeId| -> Option<f32> {
            painted
                .iter()
                .rev()
                .find(|(p, _)| *p == id)
                .map(|(_, r)| r.y)
        };
        println!("\n== {nome} ==");
        // ⚠️ **Em ORDEM DE TELA, e não na ordem da tabela** — as duas discordam,
        // e escrevê-la de memória foi o que me fez dizer ao dono que o `Edge
        // Flow` está atrás do sombreado: ele está *à frente* dele (`1271` contra
        // `1557`). *A ordem de pintura lê-se do `y`, nunca da `SECTIONS`.*
        let mut linhas: Vec<(f32, &str)> = [
            ("secção TOOL", ids::SCULPT3D_SEC_TOOL),
            ("secção BRUSH", ids::SCULPT3D_SEC_BRUSH),
            ("Radius", ids::SCULPT3D_RADIUS),
            ("Connected Only", ids::SCULPT3D_SURFACE_ONLY),
            ("secção SYMMETRY", ids::SCULPT3D_SEC_SYMMETRY),
            ("secção TOPOLOGY", ids::SCULPT3D_SEC_TOPOLOGY),
            ("⭐ Edge Flow", ids::SCULPT3D_PENTE),
            ("secção SHADING", ids::SCULPT3D_SEC_SHADING),
            ("secção SCENE", ids::SCULPT3D_SEC_SCENE),
            ("secção BAKE", ids::SCULPT3D_SEC_BAKE),
        ]
        .into_iter()
        .filter_map(|(rotulo, id)| acha(id).map(|y| (y, rotulo)))
        .collect();
        linhas.sort_by(|a, b| a.0.total_cmp(&b.0));
        for (y, rotulo) in linhas {
            println!(
                "  {rotulo:<18} y = {y:>7.0}   {}",
                if y > ALTURA_DO_ENCAIXE_PX {
                    "⛔ ABAIXO DA DOBRA"
                } else {
                    "visível"
                }
            );
        }
    }
}

/// ⭐⭐⭐ **O BOTÃO QUE SUBSTITUIU AS 38 FICHAS PEDE A PALETA** — a metade de costura do gesto.
///
/// ⛔⛔ Sem isto, o selector podia estar pintado, hit-indexado, **e mudo sob o dedo** — a espécie
/// que este painel pagou treze vezes num dia só (os chips de `Deformation`, 2026-09-14: pintados,
/// com braço no `event.rs`, e **ausentes do `populate`**). ⭐ Aqui ele entra pela tabela
/// `COMMANDS`, que é a lista ÚNICA que o `populate` e o `event` percorrem — e é isso que torna
/// este gate uma confirmação e não uma esperança.
///
/// *Mutação que sangra:* apagar a linha dele da `COMMANDS`.
#[test]
fn o_botao_dos_pinceis_pede_a_paleta() {
    let (mut host, mut state) = arrange(Sculpt3dUi::default());
    host.apply_panel_event::<Sculpt3dPanel>(
        &mut state,
        WidgetEvent::Click(ids::SCULPT3D_OPEN_BRUSHES),
    );
    assert!(
        matches!(
            only_intent("open brushes"),
            Sculpt3dIntent::OpenBrushPalette
        ),
        "o botão do selector enfileirou o intent errado",
    );
}

/// ⭐⭐⭐ **O *PICK* DA PALETA FAZ A MESMA COISA QUE A FICHA FAZIA** — a prova de que há UMA lei.
///
/// ⛔⛔ O *pick* volta pela shell (`take_command_pick_if`) e não como evento de painel, logo era o
/// sítio natural para nascer uma **segunda resposta** a *«o que é trocar de pincel?»*. A porta
/// [`ph2d_panel_sculpt3d::intent_for_palette_pick`] delega no mesmo `group_chip_ui`, e este gate
/// mede-o **contra o caminho da ficha**: os dois têm de produzir o MESMO `SetUi`, ao bit.
///
/// ⚠️ **O CONTROLO está dentro**: a comparação é com o que o clique na ficha produz, não com um
/// valor escrito à mão. *Um `assert_eq!` contra uma constante minha provaria a minha aritmética,
/// não a igualdade das duas rotas.*
///
/// *Mutação que sangra:* o `intent_for_palette_pick` construir o `Sculpt3dUi` ele próprio.
#[test]
fn o_pick_da_paleta_e_a_ficha_produzem_a_mesma_lei() {
    for verbo in [Verb::Clay, Verb::Smooth, Verb::Pose] {
        let i = Verb::ALL.iter().position(|&v| v == verbo).unwrap();

        // (a) o caminho da FICHA — o que o painel fazia até 2026-09-20.
        let (mut host, mut state) = arrange(Sculpt3dUi::default());
        host.apply_panel_event::<Sculpt3dPanel>(
            &mut state,
            WidgetEvent::Click(ids::SCULPT3D_VERB[i]),
        );
        let Sculpt3dIntent::SetUi(pela_ficha) = only_intent("ficha do verbo") else {
            panic!("a ficha do verbo enfileirou o tipo errado de intent");
        };

        // (b) o caminho da PALETA — o mesmo id, pela porta nova.
        let (_host, _state) = arrange(Sculpt3dUi::default());
        let Some(Sculpt3dIntent::SetUi(pelo_pick)) =
            ph2d_panel_sculpt3d::intent_for_palette_pick(ids::SCULPT3D_VERB[i])
        else {
            panic!("o pick da paleta não resolveu para um SetUi");
        };

        assert_eq!(
            pelo_pick, pela_ficha,
            "{verbo:?}: as duas rotas divergiram — há uma segunda lei algures",
        );
    }
    // ⚠️ **E o controlo negativo**: um id que não é de verbo não produz intent nenhum. Sem ele,
    //    um `intent_for_palette_pick` que devolvesse sempre `Some` passaria o laço de cima.
    let (_host, _state) = arrange(Sculpt3dUi::default());
    assert!(
        ph2d_panel_sculpt3d::intent_for_palette_pick(ids::SCULPT3D_OPEN_BRUSHES).is_none(),
        "o botão que ABRE a paleta não é um pincel",
    );
}
