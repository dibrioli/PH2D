//! ⭐⭐⭐ **SEAM DO ESQUELETO** (estudo 42 item 5) — o gesto REAL sobre cada controlo, e o que ele
//! faz chegar ao barramento.
//!
//! ⚠️ **Ele nasce com a wave, e não depois de um report**, que é a lição do [bug #29]: a pilha de
//! aparência shipou com o modelo, o transform, o renderer, o exportador e o painel gateados — e
//! **zero** gates no caminho do clique. Os controlos pintavam, hit-indexavam, registavam (logo
//! ACENDIAM sob o rato) e o `Click` morria no `apply_event` do painel.
//!
//! ⛔ **Nenhum gate existente vê isto**: o `hit_indexed_ids_are_registered` mede FOCABILIDADE, e os
//! testes da shell medem o que ela faz **depois** de receber o verbo. Entre os dois há um passo que
//! nenhum atravessa — *o clique sai do painel?*
//!
//! [bug #29]: ../../../docs/Vector%20Module/BUGS_vector.md

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::tool::PanelEvent;
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_skeleton::state::SkeletonPanelState;
use ph2d_panel_skeleton::{SkeletonPanel, state};
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 900.0,
};
const SEC: u128 = 1_000_000_000;

fn pointer(kind: PointerKind, x: f32, y: f32, t: u128) -> PointerEvent {
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

/// A cena tem esqueleto, a selecção está presa e há um osso em foco — o estado em que **todos** os
/// controlos da seção são oferecidos.
fn publica_tudo() {
    state::set_current_skinned(state::Skinned {
        vector: true,
        imagem: false,
    });
    state::set_current_bone(Some(ph2d_skeleton::bend::BoneSpec::straight(20.0, 1.0)));
}

fn limpa() {
    state::set_current_bone_ik_auto_side(None);
    state::set_current_skin_law_envelope(false);
    // ⚠️ O envelope repõe-se a `true` (o default conservador): sem isto o teste que o desliga
    // contamina todos os seguintes, e eles ficam verdes sobre um painel sem aquele campo.
    state::set_current_envelope_manda(true);
    state::set_current_skinned(state::Skinned::default());
    state::set_current_bone(None);
    state::set_current_bone_handles(None);
    state::set_current_bone_tip(None);
    state::set_current_bone_ik(None);
    // ⚠️ **A lente do APONTAR também se limpa**: com ela por repor, o teste seguinte pintaria o
    // desvio e esconderia a *Softness* sobre uma âncora que ALCANÇA — e ficaria verde a medir
    // outro painel.
    state::set_current_bone_aim(None);
    state::set_current_bone_smart(None);
    state::set_current_bone_actions(Vec::new());
}

/// **O gesto REAL sobre um retângulo pintado**, e o que ele deixa no barramento.
///
/// Down+Up de verdade, e não um `WidgetEvent::Click` sintético: o sintético prova a allowlist do
/// painel mas **pula a checagem de focabilidade no store** — as duas metades falham de maneiras
/// diferentes, e um gate que só faz uma fica verde sobre a outra.
fn clica(id: ph2d_a11y::NodeId, what: &str) -> Vec<EditorAction> {
    let mut host = MockPanelHost::with_panel::<SkeletonPanel>();
    let mut st = SkeletonPanelState;
    let r = host
        .painted_rect::<SkeletonPanel>(&mut st, VIEWPORT, id)
        .unwrap_or_else(|| panic!("{what} nao foi PINTADO com area clicavel"));
    let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    host.dispatch_pointer_event(pointer(PointerKind::Down, cx, cy, SEC));
    let evs = host.dispatch_pointer_event(pointer(PointerKind::Up, cx, cy, SEC + SEC / 100));
    assert!(
        evs.iter()
            .any(|e| matches!(e, WidgetEvent::Click(c) if *c == id)),
        "{what}: o ponteiro sobre o retangulo nao virou Click — ele esta' desenhado e nao existe \
         para o dispatcher (falta o `register` no populate)"
    );
    for ev in evs {
        host.apply_panel_event::<SkeletonPanel>(&mut st, ev);
    }
    host.drained_actions()
}

/// ⭐⭐⭐ **TODO VERBO DO ESQUELETO ATRAVESSA O BARRAMENTO** — a população sai da TABELA.
///
/// ⚠️ **O oráculo é o `EditorAction`, nunca o `WidgetEvent`.** Um controlo que produz `Click` e não
/// produz `ToolPanelEvent` acende sob o rato, consome o gesto e não faz nada — que é literalmente o
/// report do bug #29 (*"o olho não funciona"*).
///
/// ⛔⛔ **E este gate estava VERDE sobre um verbo morto**, porque a população dele era uma lista de
/// TRÊS escrita à mão. Em 2026-09-07 a âncora acrescentou o *Add IK* e o *Remove IK* à mesma seção;
/// eles pintavam, acendiam e o clique morria no painel — report do dono: *«Add IK não funciona»*.
/// *Um gate que existe para apanhar uma lista esquecida não pode ter a própria população escrita à
/// mão.*
///
/// ⇒ a população é [`ids::VECTOR_BONE_VERBS`], a mesma tabela que o registo e o encaminhamento
/// lêem. Um verbo novo entra aqui **sozinho** — e se ele não for pintado no estado que a `estado_de`
/// declara, o gate reprova a dizer *«não foi PINTADO»*, que é a pergunta certa a fazer ao autor.
#[test]
fn every_verb_of_the_skeleton_reaches_the_bus() {
    // ⚠️ **As DUAS tabelas**, encadeadas em vez de copiadas: a dos verbos e a da fileira do lado da
    // dobra. Uma fileira nova que ganhe tabela própria e não venha a este `chain` fica sem régua de
    // costura — e é exactamente o buraco por onde o *Add IK* passou.
    for id in ids::VECTOR_BONE_VERBS
        .into_iter()
        .chain(ids::VECTOR_BONE_BEND_IDS)
        .chain(ids::VECTOR_BONE_HANDLES_IDS)
    {
        estado_de(id);
        let acoes = clica(id, "um verbo do esqueleto");
        assert!(
            acoes.iter().any(|a| matches!(
                a,
                EditorAction::ToolPanelEvent(PanelEvent::Click(c)) if *c == id
            )),
            "o Click de {id:?} nao chegou ao barramento — ele acende sob o rato e nao faz nada \
             (falta a tabela `VECTOR_BONE_VERBS` na allowlist do `event_clicks`)"
        );
    }
    limpa();
}

/// ⭐⭐⭐ **TODO CAMPO NUMÉRICO DO ESQUELETO ATRAVESSA O BARRAMENTO** — o irmão do de cima, e o mesmo
/// modo de falha com outra cara: fora da lista o campo **aceita teclas e não fala com ninguém**.
///
/// ⚠️ O oráculo é o `ToolPanelEvent(SetValue)`, e o gesto é o de um campo: focar e escrever.
#[test]
fn every_number_of_the_skeleton_reaches_the_bus() {
    for id in ids::VECTOR_BONE_FIELDS {
        estado_de(id);
        let mut host = MockPanelHost::with_panel::<SkeletonPanel>();
        let mut st = SkeletonPanelState;
        assert!(
            host.painted_rect::<SkeletonPanel>(&mut st, VIEWPORT, id)
                .is_some(),
            "{id:?} nao foi PINTADO — declare em `estado_de` sob que estado ele existe"
        );
        // ⚠️ O valor entra pelo STORE e o evento só diz *«este mudou»* — é o contrato do
        // `ValueChanged`, e escrever o número no evento mediria outro programa.
        host.store_mut().set_number_value(id, 0.5);
        host.apply_panel_event::<SkeletonPanel>(&mut st, WidgetEvent::ValueChanged(id));
        let acoes = host.drained_actions();
        assert!(
            acoes.iter().any(|a| matches!(
                a,
                EditorAction::ToolPanelEvent(PanelEvent::SetValue(c, _)) if *c == id
            )),
            "o valor de {id:?} nao chegou ao barramento — o campo aceita teclas e nao fala com \
             ninguem (falta a tabela `VECTOR_BONE_FIELDS` no `event::is_shell_number_field`)"
        );
    }
    limpa();
}

/// **Sob que estado cada controlo da seção é PINTADO.**
///
/// ⚠️ É a única metade que não se deriva: o *Add IK* e o *Remove IK* excluem-se por construção (um
/// osso tem âncora ou não tem), e é por isso que a tabela sozinha não basta para os clicar. ⭐ Mas
/// ela basta para os **descobrir**: um verbo novo que não venha aqui reprova a dizer *«não foi
/// PINTADO»*, e essa é exactamente a pergunta que o autor tem de responder.
fn estado_de(id: ph2d_a11y::NodeId) {
    publica_tudo();
    // ⚠️ **A CURVATURA tem a MESMA forma de exclusão que a âncora e o limite**: as quatro alças só
    // existem num osso com mais de um segmento, porque num osso rígido elas são **provadamente
    // inertes** (gate `one_segment_never_bends_whatever_the_handles_say`, em `ph2d-skeleton`).
    // ⛔ Pintá-las sempre seria o painel a prometer quatro números que não mudam um pixel.
    //
    // ⭐ **E a fileira de ONDE vêm as alças tem a MESMA exclusão, pela mesma razão** — ela decide
    // quem escreve aqueles quatro números, logo não tem sujeito onde eles não existem.
    let precisa_de_segmentos = [
        ids::VECTOR_BONE_CURVE_IN_X,
        ids::VECTOR_BONE_CURVE_IN_Y,
        ids::VECTOR_BONE_CURVE_OUT_X,
        ids::VECTOR_BONE_CURVE_OUT_Y,
    ]
    .contains(&id)
        || ids::VECTOR_BONE_HANDLES_IDS.contains(&id);
    if precisa_de_segmentos {
        state::set_current_bone(Some(ph2d_skeleton::bend::BoneSpec {
            segments: 4,
            ..ph2d_skeleton::bend::BoneSpec::straight(20.0, 1.0)
        }));
    }
    // ⚠️ Os três segmentos do LADO vivem com os números da âncora — eles só têm sujeito quando ela
    // existe. A lista é derivada da tabela dos segmentos, não escrita à mão: é a mesma lição que
    // pôs a população deste ficheiro em `VECTOR_BONE_VERBS`.
    // ⭐⭐⭐ **E o DESVIO tem uma exclusão a MAIS: ele só existe numa âncora que APONTA.**
    //
    // ⚠️ A *Softness* e o *Bend* têm a exclusão **oposta** (medidos inertes com a corrente em UM),
    // logo os dois estados não podem ser o mesmo — e é por isso que a lente é um segundo eixo aqui
    // e não mais um nome na lista de cima.
    let precisa_de_apontar = id == ids::VECTOR_BONE_IK_OFFSET;
    let precisa_de_ancora = precisa_de_apontar
        || id == ids::VECTOR_BONE_IK_REMOVE
        || id == ids::VECTOR_BONE_IK_MIX
        || id == ids::VECTOR_BONE_IK_SOFTNESS
        || id == ids::VECTOR_BONE_IK_CHAIN
        || ids::VECTOR_BONE_BEND_IDS.contains(&id);
    state::set_current_bone_ik(precisa_de_ancora.then_some((
        1.0,
        0.0,
        2.0,
        ph2d_skeleton::BendSide::Keep,
    )));
    state::set_current_bone_aim(precisa_de_apontar.then_some(0.0));
    // ⚠️ O limite tem a MESMA forma de exclusão que a âncora: *Add* só existe sem ele, *Remove* e
    // os dois extremos só com ele. Um controlo fora deste `if` reprova a dizer «não foi PINTADO»,
    // que é a pergunta certa a fazer a quem o acrescentou.
    let precisa_de_limite = id == ids::VECTOR_BONE_LIMIT_REMOVE
        || id == ids::VECTOR_BONE_LIMIT_MIN
        || id == ids::VECTOR_BONE_LIMIT_MAX;
    state::set_current_bone_limit(precisa_de_limite.then_some((-45.0, 45.0)));
    // ⚠️ O osso inteligente tem a MESMA forma de exclusão: *Add* só sem acção, *Remove*, o selector
    // e os dois ângulos só com ela.
    let precisa_de_accao = id == ids::VECTOR_BONE_SMART_REMOVE
        || id == ph2d_panel_skeleton::ids::VECTOR_BONE_SMART_CLIP
        || id == ids::VECTOR_BONE_SMART_PICK
        || id == ids::VECTOR_BONE_SMART_FROM
        || id == ids::VECTOR_BONE_SMART_TO
        || ids::VECTOR_BONE_SMART_CLIP_IDS.contains(&id);
    state::set_current_bone_smart(precisa_de_accao.then(|| state::SmartBoneView {
        from: 0.0,
        to: 90.0,
        clip: "Walk".to_string(),
        target: "Leaf".to_string(),
        picking: false,
    }));
    state::set_current_bone_actions(if precisa_de_accao {
        vec!["Main".to_string(), "Walk".to_string()]
    } else {
        Vec::new()
    });
    // ⚠️ **O selector de PONTA tem a mesma forma de exclusão**: ele só existe onde a shell publica
    // a lista, isto é, num osso cujas alças vêm da corrente.
    let precisa_de_ponta =
        id == ph2d_panel_skeleton::ids::VECTOR_BONE_TIP || ids::VECTOR_BONE_TIP_IDS.contains(&id);
    state::set_current_bone_tip(precisa_de_ponta.then(|| state::TipView {
        rotulos: vec![
            "From Chain".to_string(),
            "Nobody".to_string(),
            "Bone 21".to_string(),
            "Bone 22".to_string(),
        ],
        ligado: 0,
        escondidos: 0,
    }));
}

/// ⭐⭐⭐ **O SELECTOR DE ACÇÃO existe, lista o documento, e a escolha CHEGA AO BARRAMENTO.**
///
/// ⚠️⚠️ **Ele nasce de um report do dono** (2026-09-08: *«não há meios de selecionar nem o objeto
/// alvo nem a animação»*). Até esse dia a seção pintava *Remove* mais dois campos de graus e nada
/// que dissesse a que animação o osso ficara preso — e o gesto amarrava-o, calado, à única acção que
/// um documento novo tem (`"Main"`). *Um controlo cujo sujeito é invisível lê-se exactamente como um
/// controlo morto.*
///
/// ⚠️ **O gesto é REAL, e as DUAS metades importam**: o chip tem de ABRIR (ele é `Dropdown` no
/// store e botão na tela — registá-lo como `Button` fá-lo-ia acender e nunca abrir lista nenhuma) e
/// a opção de dentro tem de virar `Click` que ATRAVESSA. Um `Click` sintético passa com o chip
/// morto sob o ponteiro, que é a cicatriz dos quatro chips da booleana.
#[test]
fn the_action_picker_lists_the_document_and_the_choice_reaches_the_bus() {
    estado_de(ph2d_panel_skeleton::ids::VECTOR_BONE_SMART_CLIP);
    let mut host = MockPanelHost::with_panel::<SkeletonPanel>();
    let mut st = SkeletonPanelState;
    let chip = host
        .painted_rect::<SkeletonPanel>(
            &mut st,
            VIEWPORT,
            ph2d_panel_skeleton::ids::VECTOR_BONE_SMART_CLIP,
        )
        .expect("o chip da acção não foi PINTADO — o osso inteligente volta a não ter sujeito");
    // Abrir a lista.
    host.dispatch_pointer_event(pointer(PointerKind::Down, chip.x + 2.0, chip.y + 2.0, SEC));
    let evs = host.dispatch_pointer_event(pointer(
        PointerKind::Up,
        chip.x + 2.0,
        chip.y + 2.0,
        SEC + SEC / 100,
    ));
    for ev in evs {
        host.apply_panel_event::<SkeletonPanel>(&mut st, ev);
    }
    // A 2.ª opção é a acção do osso — escolher a 1.ª (`Main`) é o gesto que o report pedia.
    let opt = ids::VECTOR_BONE_SMART_CLIP_IDS[0];
    let r = host
        .painted_rect::<SkeletonPanel>(&mut st, VIEWPORT, opt)
        .expect("com a lista ABERTA a acção tem de ser pintada — senão não é escolhível");
    let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    host.dispatch_pointer_event(pointer(PointerKind::Down, cx, cy, 2 * SEC));
    let evs = host.dispatch_pointer_event(pointer(PointerKind::Up, cx, cy, 2 * SEC + SEC / 100));
    assert!(
        evs.iter()
            .any(|e| matches!(e, WidgetEvent::Click(c) if *c == opt)),
        "a opção da lista está desenhada e MORTA sob o ponteiro"
    );
    for ev in evs {
        host.apply_panel_event::<SkeletonPanel>(&mut st, ev);
    }
    assert!(
        host.drained_actions().into_iter().any(|a| matches!(
            a,
            EditorAction::ToolPanelEvent(PanelEvent::Click(c)) if c == opt
        )),
        "o Click da opção não chegou ao barramento — escolher a animação não escreveria no mundo \
         (falta `VECTOR_BONE_SMART_CLIP_IDS` na allowlist do `event_clicks`)"
    );
    limpa();
}

/// ⭐⭐⭐ **A OPÇÃO QUE SE CARREGA É A QUE SE VÊ** — a `n`-ésima linha da lista devolve o `n`-ésimo id.
///
/// ⛔⛔ **Report do dono (2026-09-08):** *«ao selecionar na lista de actions … não consegue
/// selecionar o clip desejado»*. ⚠️ **Este gate mede a metade do PAINEL** — que a linha `i` da lista
/// é o id `i` do pool, ordenada de cima para baixo. A outra metade (a shell resolver essa posição
/// contra a lista **filtrada**, e não contra o documento inteiro) vive em `skeleton_smart`, e as
/// duas falham exactamente igual de fora: *escolhe-se uma linha e liga-se outra acção*.
///
/// ⚠️ **A 2.ª opção, e não a 1.ª:** um erro de deslocamento de um (ou uma lista invertida) passa
/// despercebido sobre o índice `0`, que é o mesmo em quase toda ordenação errada.
#[test]
fn the_option_you_press_is_the_one_you_see() {
    estado_de(ph2d_panel_skeleton::ids::VECTOR_BONE_SMART_CLIP);
    let mut host = MockPanelHost::with_panel::<SkeletonPanel>();
    let mut st = SkeletonPanelState;
    let chip = host
        .painted_rect::<SkeletonPanel>(
            &mut st,
            VIEWPORT,
            ph2d_panel_skeleton::ids::VECTOR_BONE_SMART_CLIP,
        )
        .expect("o chip da acção");
    host.dispatch_pointer_event(pointer(PointerKind::Down, chip.x + 2.0, chip.y + 2.0, SEC));
    let evs = host.dispatch_pointer_event(pointer(
        PointerKind::Up,
        chip.x + 2.0,
        chip.y + 2.0,
        SEC + SEC / 100,
    ));
    for ev in evs {
        host.apply_panel_event::<SkeletonPanel>(&mut st, ev);
    }
    // As duas opções publicadas (`Main`, `Walk`) têm de sair NESTA ordem, de cima para baixo.
    let r0 = host
        .painted_rect::<SkeletonPanel>(&mut st, VIEWPORT, ids::VECTOR_BONE_SMART_CLIP_IDS[0])
        .expect("a 1.ª opção");
    let r1 = host
        .painted_rect::<SkeletonPanel>(&mut st, VIEWPORT, ids::VECTOR_BONE_SMART_CLIP_IDS[1])
        .expect("a 2.ª opção");
    assert!(
        r1.y > r0.y,
        "a 2.ª opção não está abaixo da 1.ª ({} contra {}) — a lista pintada e o pool de ids estão          em ordens diferentes, e escolher uma linha ligaria outra acção",
        r1.y,
        r0.y
    );
    // E o gesto REAL sobre a 2.ª devolve o id da 2.ª.
    let (cx, cy) = (r1.x + r1.w * 0.5, r1.y + r1.h * 0.5);
    host.dispatch_pointer_event(pointer(PointerKind::Down, cx, cy, 2 * SEC));
    let evs = host.dispatch_pointer_event(pointer(PointerKind::Up, cx, cy, 2 * SEC + SEC / 100));
    assert!(
        evs.iter().any(
            |e| matches!(e, WidgetEvent::Click(c) if *c == ids::VECTOR_BONE_SMART_CLIP_IDS[1])
        ),
        "carregar na 2.ª linha não devolveu o id da 2.ª opção: {evs:?}"
    );
    limpa();
}

/// ⭐ **SEM ACÇÃO não há selector** — a outra metade da lei do controlo morto.
///
/// ⚠️ Um chip que liste as animações do documento sobre um osso que não percorre nenhuma é um
/// controlo que só sabe escrever num componente que não existe.
#[test]
fn a_bone_without_an_action_is_offered_no_picker() {
    publica_tudo();
    state::set_current_bone_smart(None);
    state::set_current_bone_actions(Vec::new());
    let mut host = MockPanelHost::with_panel::<SkeletonPanel>();
    let mut st = SkeletonPanelState;
    assert!(
        host.painted_rect::<SkeletonPanel>(
            &mut st,
            VIEWPORT,
            ph2d_panel_skeleton::ids::VECTOR_BONE_SMART_CLIP
        )
        .is_none(),
        "o selector de acção foi pintado num osso sem acção — ele só saberia escrever num \
         componente ausente"
    );
    limpa();
}

/// ⭐⭐⭐ **TODO SEGMENTO DO LADO DA DOBRA É TAMBÉM UM VERBO DA SHELL** — o censo que liga as duas
/// tabelas.
///
/// ⚠️ Elas respondem a perguntas diferentes sobre os mesmos ids ([`ids::VECTOR_BONE_BEND_IDS`] diz
/// *qual segmento é qual variante*; [`ids::VECTOR_BONE_VERBS`] diz *o que atravessa para a shell*),
/// e é exactamente por serem duas que uma pode envelhecer sem a outra. Um segmento que caia fora da
/// segunda **pinta, acende sob o rato e o clique morre dentro do painel** — o modo de falha que
/// esta seção já pagou quatro vezes.
///
/// ⭐ E ele afirma a outra metade também: que a fileira tem um id por variante da LEI. Uma variante
/// nova em `BendSide::ALL` sem id ao lado deixaria um estado do documento **inalcançável pela UI**.
#[test]
fn the_bend_row_has_exactly_one_segment_per_variant_of_the_law() {
    assert_eq!(
        ids::VECTOR_BONE_BEND_IDS.len(),
        ph2d_skeleton::BendSide::ALL.len(),
        "a fileira do lado da dobra e o vocabulário da lei têm tamanhos diferentes — uma variante \
         ficou sem segmento (**inalcançável pela UI**) ou um segmento ficou sem variante (morto)"
    );
}

/// ⛔ **AS DUAS SAÍDAS SÓ EXISTEM COM UMA FORMA PRESA** — e o `Bind` existe sempre que a seção
/// existe. *Um botão que só sabe recusar é pior que um botão ausente.*
///
/// ⚠️ **As duas metades**, porque um gate que só afirmasse a presença ficaria verde sobre um painel
/// que as mostra sempre.
#[test]
fn the_two_exits_appear_only_when_something_is_bound() {
    let sem = |id| {
        let mut host = MockPanelHost::with_panel::<SkeletonPanel>();
        let mut st = SkeletonPanelState;
        host.painted_rect::<SkeletonPanel>(&mut st, VIEWPORT, id)
            .is_some()
    };
    state::set_current_skinned(state::Skinned::default());
    state::set_current_bone(None);
    assert!(sem(ids::VECTOR_BONE_BIND), "o Bind tem de estar la' sempre");
    assert!(
        !sem(ids::VECTOR_BONE_EXPAND) && !sem(ids::VECTOR_BONE_RELEASE),
        "as saidas foram pintadas sem nada preso"
    );
    state::set_current_skinned(state::Skinned {
        vector: true,
        imagem: false,
    });
    assert!(
        sem(ids::VECTOR_BONE_EXPAND) && sem(ids::VECTOR_BONE_RELEASE),
        "as saidas sumiram com uma forma presa"
    );
    limpa();
}

/// ⭐⭐⭐ **COM UMA IMAGEM PRESA, O *RELEASE* APARECE E O *EXPAND* NÃO** — o report do dono
/// (2026-09-18: *«ainda não temos a opção de desconectar a malha do osso»*).
///
/// ⛔⛔ **O defeito era MUDO e vinha de mais atrás:** o facto publicado era um `bool` que só conhecia
/// formas vectoriais, logo com uma imagem presa ele lia `false` e as **duas** saídas nem eram
/// pintadas. *O artista não via um botão morto — via a ausência de um botão*, que é o report que
/// ele escreveu.
///
/// ⚠️ **E o *Expand* fica de fora por LEI da mídia, não por fiação:** ele troca o desenho autorado
/// pela geometria deformada de agora, e uma imagem **não tem geometria autorada** (a malha é
/// derivada da tinta, por quadro). Assar a deformação nos pixels é outra operação, e ela não existe
/// — pintá-lo aqui seria um controlo morto sob o dedo.
#[test]
fn a_skinned_image_offers_release_but_not_expand() {
    let sem = |id| {
        let mut host = MockPanelHost::with_panel::<SkeletonPanel>();
        let mut st = SkeletonPanelState;
        host.painted_rect::<SkeletonPanel>(&mut st, VIEWPORT, id)
            .is_some()
    };
    state::set_current_skinned(state::Skinned {
        vector: false,
        imagem: true,
    });
    state::set_current_bone(None);
    assert!(
        sem(ids::VECTOR_BONE_RELEASE),
        "com uma IMAGEM presa o Release nao foi pintado — e' o report do dono a' letra: a opcao \
         de desconectar a malha do osso nao existe para uma imagem"
    );
    assert!(
        !sem(ids::VECTOR_BONE_EXPAND),
        "o Expand foi pintado com uma imagem presa: ele troca o desenho autorado pela geometria \
         deformada, e uma imagem nao tem geometria autorada — seria um controlo morto sob o dedo"
    );
    // ⚠️ E o CONTROLO: com as duas mídias presas o *Expand* volta, senão este gate leria como
    // «o Expand nunca aparece».
    state::set_current_skinned(state::Skinned {
        vector: true,
        imagem: true,
    });
    assert!(
        sem(ids::VECTOR_BONE_EXPAND),
        "com uma FORMA presa ao lado da imagem o Expand tem de voltar — ele e' da forma"
    );
    limpa();
}

// ⛔⛔ **A AUSÊNCIA do painel numa cena sem ossos MUDOU DE DONO** (2026-09-09).
//
// Enquanto isto era uma secção do painel de vetor, ela decidia sozinha se se pintava. Com painel
// próprio, quem decide é a **shell** (`panel_visible`) — a mesma porta de todos os painéis —, e o
// gate que mede a lei vive lá: `the_skeleton_panel_only_opens_where_it_has_a_subject`.
//
// ⚠️ *Uma lei que muda de dono e deixa o gate para trás é uma lei sem prova* — por isso ela sai
// daqui com o endereço do sítio novo, e não em silêncio.

/// ⭐⭐ **OS DOIS NÚMEROS DO OSSO SÓ EXISTEM COM UM OSSO EM FOCO** — sem sujeito, um campo é a
/// espécie de controlo morto que o `CLAUDE.md` §5.0 nomeia.
#[test]
fn the_two_bone_numbers_need_a_bone_in_focus() {
    state::set_current_skinned(state::Skinned::default());
    state::set_current_bone(None);
    let pintado = |id| {
        let mut host = MockPanelHost::with_panel::<SkeletonPanel>();
        let mut st = SkeletonPanelState;
        host.painted_rect::<SkeletonPanel>(&mut st, VIEWPORT, id)
            .is_some()
    };
    assert!(
        !pintado(ids::VECTOR_BONE_LENGTH) && !pintado(ids::VECTOR_BONE_STRENGTH),
        "os campos foram pintados sem osso nenhum em foco"
    );
    state::set_current_bone(Some(ph2d_skeleton::bend::BoneSpec::straight(20.0, 1.0)));
    assert!(
        pintado(ids::VECTOR_BONE_LENGTH) && pintado(ids::VECTOR_BONE_STRENGTH),
        "os campos sumiram com um osso em foco"
    );
    limpa();
}

/// Põe a ferramenta OSSO na mão com o verbo pedido — o estado em que o grupo alternável existe.
///
/// ⚠️ O que atravessa é o **ÍNDICE** em `BoneAction::ALL`, não o enum: é o que mantém este painel
/// sem depender da crate da ferramenta de vector.
fn modo_osso(verbo: usize) {
    state::set_current_bone_tool(Some(verbo));
}

/// ⭐⭐⭐ **OS DOIS SEGMENTOS DE CRIAR × TRANSFORMAR CHEGAM À FERRAMENTA** (Enio, 2026-09-07).
///
/// ⚠️ **O oráculo é o `EditorAction`, nunca o `WidgetEvent`** — a lição do bug #29: um controlo que
/// produz `Click` e não produz `ToolPanelEvent` acende sob o rato, consome o gesto e **não faz
/// nada**. Um grupo alternável que não troca o verbo é exactamente o defeito que ele existe para
/// curar, com uma camada de confusão a mais.
#[test]
fn both_segments_of_create_and_transform_reach_the_tool() {
    publica_tudo();
    modo_osso(0);
    for (id, nome) in [
        (ph2d_tool_vector::ids::VECTOR_BONE_ACT_CREATE, "Create"),
        (
            ph2d_tool_vector::ids::VECTOR_BONE_ACT_TRANSFORM,
            "Transform",
        ),
    ] {
        let acoes = clica(id, nome);
        assert!(
            acoes.iter().any(|a| matches!(
                a,
                EditorAction::ToolPanelEvent(PanelEvent::Click(c)) if *c == id
            )),
            "o segmento {nome} nao chegou a' ferramenta"
        );
    }
    limpa();
    state::set_current_bone_tool(None);
}

/// ⭐⭐⭐ **A FILEIRA TEM UM SEGMENTO POR VERBO — nem mais, nem menos.**
///
/// ⛔⛔ **Ela e a [`ph2d_tool_vector::BoneAction::ALL`] são alinhadas por POSIÇÃO**, e é isso que o
/// doc da `VECTOR_BONE_ACTION_IDS` promete por escrito. Um verbo novo sem id é um gesto que o
/// artista **não alcança**; um id a mais é um segmento que acende e não troca nada. *As duas
/// falhas são mudas, e as curas são opostas.*
///
/// ⚠️ **E o ÍNDICE de cada verbo tem de ser o dele** — sem esta metade, um `indice()` que
/// devolvesse `0` para todos passaria a contagem e acenderia sempre o primeiro segmento, que é
/// exactamente o defeito que o `usize::from(acao == Transform)` tinha com três verbos.
///
/// (Mutação: `indice()` a devolver `0` ⇒ RED na 2.ª metade. Um id a mais ⇒ RED na 1.ª.)
#[test]
fn a_fileira_de_verbos_do_osso_tem_um_segmento_por_accao() {
    use ph2d_tool_vector::BoneAction;
    assert_eq!(
        ph2d_panel_skeleton::ids::VECTOR_BONE_ACTION_IDS.len(),
        BoneAction::ALL.len(),
        "a fileira do painel e a tabela do vocabulario tem tamanhos diferentes"
    );
    for (i, a) in BoneAction::ALL.iter().enumerate() {
        assert_eq!(a.indice(), i, "o verbo {a:?} acende o segmento errado");
    }
}

/// ⭐⭐⭐ **O SEGMENTO DO PESO CHEGA À FERRAMENTA, e os dois números dele também.**
///
/// ⚠️ **É a lei do bug #29 aplicada ao terceiro verbo**: um controlo que produz `Click` e não
/// produz `ToolPanelEvent` acende sob o rato, consome o gesto e **não faz nada** — e aqui ele
/// pareceria um pincel partido, que é a queixa que esta casa já pagou sete vezes.
///
/// ⚠️ **Os dois números viajam por `ValueChanged` e não por `Click`**, e o gate mede-os pela porta
/// que o painel de facto usa (`apply_event`): sem eles na condição, o artista digita um raio e o
/// pincel continua com o de fábrica, **sem um erro**.
#[test]
fn o_verbo_do_peso_e_os_dois_numeros_dele_chegam_a_ferramenta() {
    publica_tudo();
    modo_osso(2);
    let acoes = clica(ph2d_tool_vector::ids::VECTOR_BONE_ACT_WEIGHT, "Weight");
    assert!(
        acoes.iter().any(|a| matches!(
            a,
            EditorAction::ToolPanelEvent(PanelEvent::Click(c))
                if *c == ph2d_tool_vector::ids::VECTOR_BONE_ACT_WEIGHT
        )),
        "o segmento Weight nao chegou a' ferramenta"
    );
    for (id, nome) in [
        (
            ph2d_tool_vector::ids::VECTOR_BONE_WEIGHT_RADIUS,
            "Brush Radius",
        ),
        (
            ph2d_tool_vector::ids::VECTOR_BONE_WEIGHT_AMOUNT,
            "Brush Strength",
        ),
    ] {
        let mut host = MockPanelHost::with_panel::<SkeletonPanel>();
        let mut st = SkeletonPanelState;
        host.store_mut().set_number_value(id, 0.5);
        host.apply_panel_event::<SkeletonPanel>(&mut st, WidgetEvent::ValueChanged(id));
        assert!(
            host.drained_actions().iter().any(|a| matches!(
                a,
                EditorAction::ToolPanelEvent(PanelEvent::SetValue(c, _)) if *c == id
            )),
            "o campo {nome} aceita teclas e nao fala com ninguem"
        );
    }
    limpa();
    state::set_current_bone_tool(None);
}

/// ⭐⭐⭐ **A FILEIRA DA DIRECÇÃO TEM UM SEGMENTO POR LADO** — ordem do dono, 2026-09-19.
///
/// ⚠️ **A mesma lei da fileira dos verbos, e pelo mesmo motivo:** o painel acende por ÍNDICE, e uma
/// lista com outro tamanho (ou noutra ordem) acende o segmento errado **em silêncio**.
#[test]
fn a_fileira_da_direccao_do_peso_tem_um_segmento_por_lado() {
    use ph2d_tool_vector::WeightDirection;
    assert_eq!(
        ph2d_panel_skeleton::ids::VECTOR_BONE_WEIGHT_DIR_IDS.len(),
        WeightDirection::ALL.len(),
        "a fileira do painel e a tabela do vocabulario tem tamanhos diferentes"
    );
    for (i, lado) in WeightDirection::ALL.iter().enumerate() {
        assert_eq!(lado.indice(), i, "o lado {lado:?} acende o segmento errado");
    }
    // ⭐⭐⭐ **E o id na posição de um lado É o id DAQUELE lado** — a metade que nasceu de uma
    // mutação SOBREVIVENTE: sem ela, trocar a ordem da lista passava, e o segmento rotulado *Add*
    // ficava a mandar `Subtract`. *«Índice-alinhadas» era uma afirmação que nada verificava.*
    assert_eq!(
        ph2d_panel_skeleton::ids::VECTOR_BONE_WEIGHT_DIR_IDS[WeightDirection::Add.indice()],
        ph2d_tool_vector::ids::VECTOR_BONE_WEIGHT_ADD,
        "a posicao do Add na fileira carrega outro id"
    );
    assert_eq!(
        ph2d_panel_skeleton::ids::VECTOR_BONE_WEIGHT_DIR_IDS[WeightDirection::Subtract.indice()],
        ph2d_tool_vector::ids::VECTOR_BONE_WEIGHT_SUB,
        "a posicao do Subtract na fileira carrega outro id"
    );
}

/// ⭐⭐⭐ **OS DOIS BOTÕES DA DIRECÇÃO RESPONDEM AO DEDO** (ordem do dono, 2026-09-19).
///
/// ⛔⛔ **É a SÉTIMA vez que esta casa paga a lição, e é por isso que o gate usa o ponteiro REAL:**
/// um chip pintado, hit-indexado e **ausente do `populate`** fica morto sob o dedo — o clique morre
/// no `is_focusable`, e *um controlo nunca pintado e um morto sob o dedo dão o MESMO report*. Um
/// `WidgetEvent::Click` sintético passa com o chip morto; o `clica` aqui faz *down* + *up* sobre o
/// rectângulo que o painel de facto pintou.
///
/// ⚠️ **E o oráculo é o `EditorAction`, nunca o `WidgetEvent`** (a lição do bug #29): um controlo
/// que acende e não fala com a ferramenta consome o gesto e não faz nada.
#[test]
fn os_dois_botoes_da_direccao_do_peso_respondem_ao_dedo() {
    publica_tudo();
    modo_osso(2);
    for (id, nome) in [
        (ph2d_tool_vector::ids::VECTOR_BONE_WEIGHT_ADD, "Add"),
        (ph2d_tool_vector::ids::VECTOR_BONE_WEIGHT_SUB, "Subtract"),
    ] {
        let acoes = clica(id, nome);
        assert!(
            acoes.iter().any(|a| matches!(
                a,
                EditorAction::ToolPanelEvent(PanelEvent::Click(c)) if *c == id
            )),
            "o botao {nome} nao chegou a' ferramenta"
        );
    }
    limpa();
    state::set_current_bone_tool(None);
}

/// ⭐⭐⭐ **A FILEIRA DO MODO TEM UM SEGMENTO POR MODO** (F29) — a mesma lei da irmã acima, e pelo
/// mesmo motivo medido: o painel acende por ÍNDICE, e uma lista noutra ordem acende o segmento
/// errado **em silêncio**.
#[test]
fn a_fileira_do_modo_do_peso_tem_um_segmento_por_modo() {
    use ph2d_tool_vector::WeightMode;
    assert_eq!(
        ph2d_panel_skeleton::ids::VECTOR_BONE_WEIGHT_MODE_IDS.len(),
        WeightMode::ALL.len(),
        "a fileira do painel e a tabela do vocabulario tem tamanhos diferentes"
    );
    assert_eq!(
        ph2d_panel_skeleton::ids::VECTOR_BONE_WEIGHT_MODE_IDS[WeightMode::Cumulative.indice()],
        ph2d_tool_vector::ids::VECTOR_BONE_WEIGHT_CUMUL,
        "a posicao do Cumulative na fileira carrega outro id"
    );
    assert_eq!(
        ph2d_panel_skeleton::ids::VECTOR_BONE_WEIGHT_MODE_IDS[WeightMode::Absolute.indice()],
        ph2d_tool_vector::ids::VECTOR_BONE_WEIGHT_ABS,
        "a posicao do Absolute na fileira carrega outro id"
    );
}

/// ⭐⭐⭐ **OS DOIS BOTÕES DO MODO RESPONDEM AO DEDO** (F29).
///
/// ⛔⛔ **A OITAVA vez que esta casa escreve este gate, e a razão não mudou:** um chip pintado,
/// hit-indexado e ausente do `populate` fica **morto sob o dedo** — e *um controlo nunca pintado e
/// um morto sob o dedo dão o MESMO report*. Um `WidgetEvent::Click` sintético passa com o chip
/// morto; o `clica` faz *down* + *up* sobre o rectângulo que o painel de facto pintou.
#[test]
fn os_dois_botoes_do_modo_do_peso_respondem_ao_dedo() {
    publica_tudo();
    modo_osso(2);
    for (id, nome) in [
        (
            ph2d_tool_vector::ids::VECTOR_BONE_WEIGHT_CUMUL,
            "Cumulative",
        ),
        (ph2d_tool_vector::ids::VECTOR_BONE_WEIGHT_ABS, "Absolute"),
    ] {
        let acoes = clica(id, nome);
        assert!(
            acoes.iter().any(|a| matches!(
                a,
                EditorAction::ToolPanelEvent(PanelEvent::Click(c)) if *c == id
            )),
            "o botao {nome} nao chegou a' ferramenta"
        );
    }
    limpa();
    state::set_current_bone_tool(None);
}

/// ⭐⭐⭐ **NO MODO ABSOLUTO A DIRECÇÃO NÃO CHEGA A SER PINTADA** — ordem do dono: *«neste modo os
/// botões Add e Subtract ficam inactivos»*, e aqui *inactivo* é **ausente**.
///
/// ⚠️⚠️ **O oráculo é o PIXEL e não a função que decide:** o gate irmão da crate
/// (`a_fileira_da_direccao_some_no_modo_absoluto`) mede a lei, e este mede que ela chega à tela —
/// *o terceiro elo do §5.0, que é sempre o que falta*.
///
/// ⛔ **As duas metades, porque as curas são opostas:** a fileira do MODO continua pintada ali
/// (senão o artista não consegue voltar), e a da direcção volta no modo cumulativo (senão ele
/// perde o lado).
#[test]
fn no_modo_absoluto_a_direccao_nao_e_pintada() {
    publica_tudo();
    modo_osso(2);
    let pintado = |modo: usize, id: ph2d_a11y::NodeId| {
        state::set_current_bone_weight(0.4, 0.15, 0, modo);
        let mut host = MockPanelHost::with_panel::<SkeletonPanel>();
        let mut st = SkeletonPanelState;
        host.painted_rect::<SkeletonPanel>(&mut st, VIEWPORT, id)
            .is_some()
    };
    let abs = ph2d_tool_vector::WeightMode::Absolute.indice();
    let cum = ph2d_tool_vector::WeightMode::Cumulative.indice();
    assert!(
        !pintado(abs, ph2d_tool_vector::ids::VECTOR_BONE_WEIGHT_ADD),
        "no modo ABSOLUTO o botao Add continua na tela — ele nao tem sujeito ali"
    );
    assert!(
        pintado(cum, ph2d_tool_vector::ids::VECTOR_BONE_WEIGHT_ADD),
        "no modo CUMULATIVO o botao Add sumiu — o artista perdeu o lado"
    );
    assert!(
        pintado(abs, ph2d_tool_vector::ids::VECTOR_BONE_WEIGHT_CUMUL),
        "a fileira do MODO sumiu no absoluto — o artista nao consegue voltar"
    );
    state::set_current_bone_weight(0.0, 0.0, 0, 0);
    limpa();
    state::set_current_bone_tool(None);
}

/// ⭐⭐ **O SEGMENTO ACESO SEGUE O QUE A SHELL PUBLICA** — e os dois botões seguem o VERBO.
///
/// ⚠️ **As duas metades, porque as curas são opostas:** pintados nos outros verbos eles seriam
/// controlos que não fazem nada; ausentes no verbo deles, a direcção é inalcançável e o artista
/// volta a não ter como tirar peso.
#[test]
fn os_botoes_da_direccao_seguem_o_verbo_e_o_que_a_shell_publica() {
    let pintado = |verbo: Option<usize>, id| {
        let mut host = MockPanelHost::with_panel::<SkeletonPanel>();
        let mut st = SkeletonPanelState;
        publica_tudo();
        state::set_current_bone_tool(verbo);
        let v = host
            .painted_rect::<SkeletonPanel>(&mut st, VIEWPORT, id)
            .is_some();
        limpa();
        state::set_current_bone_tool(None);
        v
    };
    for id in ph2d_panel_skeleton::ids::VECTOR_BONE_WEIGHT_DIR_IDS {
        assert!(
            pintado(Some(2), id),
            "o lado do pincel nao e' pintado com o verbo Weight armado"
        );
        assert!(
            !pintado(Some(1), id),
            "o lado do pincel e' pintado em Transformar"
        );
        assert!(
            !pintado(None, id),
            "o lado do pincel e' pintado fora da ferramenta Osso"
        );
    }
    // ⚠️ **Qual dos dois está ACESO não é observável daqui, e dizê-lo é o que torna este gate
    // honesto:** a `segmented` recebe o booleano e pinta-o — ela não o guarda no `WidgetStore`,
    // logo de fora só se vê *«foi pintado»*. A metade que falta é uma conta, e ela é julgada onde
    // pode ser CHAMADA: `section::tests::o_indice_publicado_acende_o_lado_do_pincel`.
    limpa();
    state::set_current_bone_tool(None);
}

/// ⭐⭐ **OS DOIS NÚMEROS DO PINCEL SÓ EXISTEM COM O PINCEL NA MÃO.**
///
/// ⚠️ **As DUAS metades:** pintados nos outros dois verbos, eles seriam controlos que não fazem
/// nada — a espécie de morto que o `CLAUDE.md` §5.0 nomeia; ausentes no verbo deles, o pincel é
/// inalcançável. *Uma metade sozinha fica verde sobre o defeito oposto.*
#[test]
fn os_numeros_do_pincel_seguem_o_verbo() {
    let pintado = |verbo: Option<usize>, id| {
        let mut host = MockPanelHost::with_panel::<SkeletonPanel>();
        let mut st = SkeletonPanelState;
        publica_tudo();
        state::set_current_bone_tool(verbo);
        let v = host
            .painted_rect::<SkeletonPanel>(&mut st, VIEWPORT, id)
            .is_some();
        limpa();
        state::set_current_bone_tool(None);
        v
    };
    for id in [
        ph2d_tool_vector::ids::VECTOR_BONE_WEIGHT_RADIUS,
        ph2d_tool_vector::ids::VECTOR_BONE_WEIGHT_AMOUNT,
    ] {
        assert!(
            pintado(Some(2), id),
            "o numero do pincel nao e' pintado com o verbo Weight armado"
        );
        assert!(
            !pintado(Some(0), id),
            "o numero do pincel e' pintado em Criar"
        );
        assert!(
            !pintado(None, id),
            "o numero do pincel e' pintado fora da ferramenta Osso"
        );
    }
}

/// ⭐⭐⭐ **O GRUPO É A PORTA, e por isso é pintado SEMPRE — mas nada acende sem estar armado.**
///
/// ⛔⛔ **Ordem do dono, 2026-09-09**, e ela INVERTE a lei anterior deste gate (*«o grupo só existe
/// no modo Osso»*). O pill `Bone` saiu da fileira de modos do painel de vector, e com ele foi-se a
/// única porta para o modo ⇒ estes dois segmentos passaram a **ser** a entrada. *Um controlo que só
/// aparece depois de já se estar no modo que ele liga não é uma porta.*
///
/// A regra, nas palavras dele:
///
/// | estado | o que acende |
/// |---|---|
/// | *«se não há ossos no mundo»* | **nenhum**, até ele carregar em *Create* |
/// | *«ao seleccionar o osso»* | **Transform** |
///
/// ⚠️ **As duas metades**, senão o gate fica verde sobre um painel que acende sempre um deles.
#[test]
fn the_create_transform_group_is_the_door_and_starts_with_nothing_lit() {
    let pintado = |id| {
        let mut host = MockPanelHost::with_panel::<SkeletonPanel>();
        let mut st = SkeletonPanelState;
        host.painted_rect::<SkeletonPanel>(&mut st, VIEWPORT, id)
            .is_some()
    };
    limpa();
    state::set_current_bone_tool(None);

    // ⚠️ **Uma cena SEM ossos**: o painel abre-se pelo menu, e a porta tem de estar lá.
    assert!(
        pintado(ph2d_tool_vector::ids::VECTOR_BONE_ACT_CREATE)
            && pintado(ph2d_tool_vector::ids::VECTOR_BONE_ACT_TRANSFORM),
        "a porta do modo Osso não é pintada numa cena sem ossos — o artista abre o painel pelo menu \
         e não tem gesto nenhum que crie o primeiro"
    );
    // ⚠️ **Qual deles ACENDE mede-se na porta** (`section::aceso`, com gate próprio): o estado
    // *aceso* de um segmento não vive no `WidgetStore` — ele é passado ao pintor a cada quadro, e
    // este arnês só vê o retângulo. *Um gate que afirmasse a cor aqui mediria o instrumento.*
    state::set_current_bone_tool(Some(1));
    assert!(
        pintado(ph2d_tool_vector::ids::VECTOR_BONE_ACT_CREATE)
            && pintado(ph2d_tool_vector::ids::VECTOR_BONE_ACT_TRANSFORM),
        "com um osso escolhido os dois segmentos continuam a ser oferecidos"
    );
    state::set_current_bone_tool(None);
}

/// ⭐⭐⭐ **OS NÚMEROS DO PAINEL SÃO OS DO DOCUMENTO** (report do dono, 2026-09-14: *«IK chain mostra
/// 0 ao inserir IK»*).
///
/// ⛔⛔ **Os cinco campos nasciam com `0` e nada lá escrevia** — o `populate` regista-os com
/// `value: 0.0` e ninguém chamava `set_number_value`. O publicador (`set_current_bone_ik`) existe,
/// a shell chama-o todo quadro, e os valores morriam no `state` porque a fileira que os pinta não
/// os lê: o `labeled_number_field` tira o valor do **WidgetStore**.
///
/// ⚠️ **`Chain = 0` não é um `0` inócuo:** ele significa *«até à raiz»*, e é a queixa nº 1
/// documentada do default do Blender. O painel dizia ao artista exactamente a coisa que o modelo
/// existe para não fazer — e um `Mix = 0` lido como verdade é a restrição DESLIGADA.
///
/// ⛔ O CONTROLO é um valor por campo, todos diferentes: um painel que semeasse só um, ou que
/// semeasse todos com o mesmo, passaria num gate escrito com um número só.
#[test]
fn the_number_fields_show_the_document_not_the_value_they_were_born_with() {
    let mut host = MockPanelHost::with_panel::<SkeletonPanel>();
    let mut st = SkeletonPanelState;
    publica_tudo();
    state::set_current_bone(Some(ph2d_skeleton::bend::BoneSpec::straight(20.0, 1.5)));
    state::set_current_bone_ik(Some((0.5, 0.25, 4.0, ph2d_skeleton::BendSide::Keep)));
    // Pintar UMA vez: o `painted_rect` corre o painel inteiro e devolve o rect de um id.
    host.painted_rect::<SkeletonPanel>(&mut st, VIEWPORT, ids::VECTOR_BONE_IK_CHAIN)
        .expect("o campo Chain tem de ser pintado com um osso com ancora em foco");
    for (id, esperado, nome) in [
        (ids::VECTOR_BONE_LENGTH, 20.0, "Length"),
        (ids::VECTOR_BONE_STRENGTH, 1.5, "Strength"),
        (ids::VECTOR_BONE_IK_MIX, 0.5, "Mix"),
        (ids::VECTOR_BONE_IK_SOFTNESS, 0.25, "Softness"),
        (ids::VECTOR_BONE_IK_CHAIN, 4.0, "Chain"),
    ] {
        let (_, v, ..) = host
            .store()
            .number_input(id)
            .unwrap_or_else(|| panic!("{nome} nao esta' registado no store"));
        assert!(
            (v - esperado).abs() < 1e-9,
            "{nome} mostra {v} e o documento diz {esperado} — o painel mente sobre o que o artista \
             tem, e um `Chain = 0` significa «ate' a` raiz»"
        );
    }
    limpa();
}

/// ⭐⭐⭐ **O PAINEL NÃO MOSTRA UM CONTROLO QUE ESTE OSSO NÃO SABE LER** (F8) — as quatro alças da
/// curvatura só existem num osso com mais de um segmento.
///
/// ⚠️ **A régua é a LEI e não uma varredura:** num osso de um segmento a curvatura é *provadamente*
/// inerte — o `one_segment_never_bends_whatever_the_handles_say` (em `ph2d-skeleton`) mostra que a
/// fábrica devolve o osso rígido **ao bit**, seja qual for a alça. ⛔ Pintá-las ali seria o painel a
/// prometer quatro números que aceitam teclas, gravam no documento e não mudam um pixel — a espécie
/// de controlo morto que o `CLAUDE.md` §5.0 nomeia, e a lei que o L-System já pagou.
///
/// ⚠️ **As DUAS metades são obrigatórias:** sem o «aparece» o gate ficaria verde sobre um painel que
/// nunca as mostra, e sem o «não aparece» sobre um que as mostra sempre.
#[test]
fn the_curvature_is_only_offered_on_a_bone_that_can_read_it() {
    let alcas = [
        ids::VECTOR_BONE_CURVE_IN_X,
        ids::VECTOR_BONE_CURVE_IN_Y,
        ids::VECTOR_BONE_CURVE_OUT_X,
        ids::VECTOR_BONE_CURVE_OUT_Y,
    ];
    for (segments, esperado) in [(1u8, false), (2, true), (8, true)] {
        state::set_current_bone(Some(ph2d_skeleton::bend::BoneSpec {
            segments,
            ..ph2d_skeleton::bend::BoneSpec::straight(20.0, 1.0)
        }));
        for id in alcas {
            let mut host = MockPanelHost::with_panel::<SkeletonPanel>();
            let mut st = SkeletonPanelState;
            let pintado = host
                .painted_rect::<SkeletonPanel>(&mut st, VIEWPORT, id)
                .is_some();
            assert_eq!(
                pintado, esperado,
                "com segments = {segments}, {id:?} pintado = {pintado}"
            );
        }
        // E o campo dos SEGMENTOS existe sempre — é ele a porta de entrada da feature.
        let mut host = MockPanelHost::with_panel::<SkeletonPanel>();
        let mut st = SkeletonPanelState;
        assert!(
            host.painted_rect::<SkeletonPanel>(&mut st, VIEWPORT, ids::VECTOR_BONE_SEGMENTS)
                .is_some(),
            "sem o campo Segments a curvatura e' inalcancavel para sempre"
        );
    }
    limpa();
}

/// ⭐⭐⭐⭐ **A FILEIRA DE ONDE VÊM AS ALÇAS SÓ É PINTADA ONDE ELA TEM SUJEITO — e em `From Chain`
/// os quatro números da curvatura SOMEM.**
///
/// ⚠️ **São duas metades e nenhuma basta sozinha.** A primeira é a mesma lei dos quatro números:
/// num osso rígido a curvatura é **provadamente inerte**, logo um segmentado que grava no documento
/// sem mudar um pixel é o painel a mentir. A segunda é nova e é a que este modo obriga: em
/// `From Chain` as alças são **derivadas**, e quatro caixas que aceitam teclas cujo valor o quadro
/// seguinte recalcula são a mesma mentira com outra cara.
///
/// (Mutação: tirar o `state::current_bone_handles() != Some(1)` do `campos_do_osso` ⇒ RED.)
#[test]
fn a_fileira_das_alcas_so_existe_onde_ela_manda_e_em_auto_os_numeros_somem() {
    let pintado = |id| {
        let mut host = MockPanelHost::with_panel::<SkeletonPanel>();
        let mut st = SkeletonPanelState;
        host.painted_rect::<SkeletonPanel>(&mut st, VIEWPORT, id)
            .is_some()
    };
    let curvo = ph2d_skeleton::bend::BoneSpec {
        segments: 4,
        ..ph2d_skeleton::bend::BoneSpec::straight(20.0, 1.0)
    };

    // (a) Num osso RÍGIDO nem a fileira nem os números existem.
    state::set_current_bone(Some(ph2d_skeleton::bend::BoneSpec::straight(20.0, 1.0)));
    state::set_current_bone_handles(Some(0));
    assert!(
        !pintado(ids::VECTOR_BONE_HANDLES_AUTHORED) && !pintado(ids::VECTOR_BONE_CURVE_IN_Y),
        "um osso rigido nao devia oferecer curvatura nenhuma"
    );

    // (b) Num osso com segmentos e em `Manual`, as duas coisas existem.
    state::set_current_bone(Some(curvo));
    state::set_current_bone_handles(Some(0));
    assert!(
        pintado(ids::VECTOR_BONE_HANDLES_AUTHORED) && pintado(ids::VECTOR_BONE_HANDLES_AUTO),
        "a fileira sumiu num osso que a sabe ler"
    );
    assert!(
        pintado(ids::VECTOR_BONE_CURVE_IN_Y),
        "os numeros da curvatura sumiram em Manual"
    );

    // (c) E em `From Chain` a fileira FICA e os números SOMEM.
    state::set_current_bone_handles(Some(1));
    assert!(
        pintado(ids::VECTOR_BONE_HANDLES_AUTO),
        "a fileira tem de ficar — e' por ela que se volta ao Manual"
    );
    assert!(
        !pintado(ids::VECTOR_BONE_CURVE_IN_Y) && !pintado(ids::VECTOR_BONE_CURVE_OUT_X),
        "os numeros da curvatura ficaram pintados em From Chain — eles nao sao autorados la'"
    );
    limpa();
}

/// ⭐⭐⭐ **O SELECTOR DE «QUEM MANDA NA PONTA» existe, lista os filhos, e a escolha CHEGA AO
/// BARRAMENTO** — o irmão do de cima, para a ordem do dono de 2026-09-16 (*«escolher qual dos vários
/// filhos manda na curva»*).
///
/// ⚠️ **As DUAS metades, e o gesto é REAL**: o chip tem de ABRIR (ele é `Dropdown` no store e botão
/// na tela) e a linha de dentro tem de virar `Click` que ATRAVESSA. *Um `Click` sintético passa com
/// o chip morto sob o ponteiro*, que é a cicatriz dos quatro chips da booleana.
#[test]
fn the_tip_picker_lists_the_children_and_the_choice_reaches_the_bus() {
    estado_de(ph2d_panel_skeleton::ids::VECTOR_BONE_TIP);
    let mut host = MockPanelHost::with_panel::<SkeletonPanel>();
    let mut st = SkeletonPanelState;
    let chip = host
        .painted_rect::<SkeletonPanel>(&mut st, VIEWPORT, ph2d_panel_skeleton::ids::VECTOR_BONE_TIP)
        .expect("o chip da ponta não foi PINTADO — a escolha do filho não teria onde acontecer");
    host.dispatch_pointer_event(pointer(PointerKind::Down, chip.x + 2.0, chip.y + 2.0, SEC));
    let evs = host.dispatch_pointer_event(pointer(
        PointerKind::Up,
        chip.x + 2.0,
        chip.y + 2.0,
        SEC + SEC / 100,
    ));
    for ev in evs {
        host.apply_panel_event::<SkeletonPanel>(&mut st, ev);
    }
    // A 3.ª linha é o primeiro FILHO — o gesto que a ordem do dono pede.
    let opt = ids::VECTOR_BONE_TIP_IDS[2];
    let r = host
        .painted_rect::<SkeletonPanel>(&mut st, VIEWPORT, opt)
        .expect("com a lista ABERTA o filho tem de ser pintado — senão não é escolhível");
    let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    host.dispatch_pointer_event(pointer(PointerKind::Down, cx, cy, 2 * SEC));
    let evs = host.dispatch_pointer_event(pointer(PointerKind::Up, cx, cy, 2 * SEC + SEC / 100));
    assert!(
        evs.iter()
            .any(|e| matches!(e, WidgetEvent::Click(c) if *c == opt)),
        "a linha da lista está desenhada e MORTA sob o ponteiro"
    );
    for ev in evs {
        host.apply_panel_event::<SkeletonPanel>(&mut st, ev);
    }
    assert!(
        host.drained_actions().into_iter().any(|a| matches!(
            a,
            EditorAction::ToolPanelEvent(PanelEvent::Click(c)) if c == opt
        )),
        "o Click da linha não chegou ao barramento — escolher o filho não escreveria no mundo \
         (falta `VECTOR_BONE_TIP_IDS` na allowlist do `event_clicks`)"
    );
    limpa();
}

/// ⭐⭐⭐ **O ENVELOPE SÓ É PINTADO ONDE AINDA MANDA** (ordem do dono, 2026-09-18, a seguir à pergunta
/// dele: *«Por que o envelope já não influencia na deformação?»*).
///
/// ⛔⛔ Com os pesos do **padrão-ouro** uma imagem deforma **igual** a `1` e a `2` — medido, coluna a
/// coluna. Num rig só de imagens este campo aceitava teclas, gravava no documento e **não mudava um
/// pixel**: a mesma mentira que as quatro alças de curvatura já tinham pago neste painel.
///
/// ⚠️ **As DUAS metades**, porque as curas são opostas: um gate que só pedisse a ausência ficaria
/// verde sobre um painel que **nunca** o pinta — e aí seria o envelope das formas vectoriais a
/// desaparecer, que é um controlo VIVO escondido.
#[test]
fn the_envelope_is_painted_only_where_it_still_rules() {
    let pintado = || {
        let mut host = MockPanelHost::with_panel::<SkeletonPanel>();
        let mut st = SkeletonPanelState;
        host.painted_rect::<SkeletonPanel>(&mut st, VIEWPORT, ids::VECTOR_BONE_STRENGTH)
            .is_some()
    };
    publica_tudo();
    state::set_current_envelope_manda(true);
    assert!(
        pintado(),
        "o envelope sumiu com uma forma VECTORIAL presa — ali ele manda como sempre (a lei \
         euclidiana), e escondê-lo e' esconder um controlo vivo"
    );
    state::set_current_envelope_manda(false);
    assert!(
        !pintado(),
        "o envelope foi pintado num rig so' de IMAGENS, onde ele e' provadamente inerte: o painel \
         promete um numero que nao muda um pixel"
    );
    limpa();
    state::set_current_envelope_manda(true);
}

/// ⭐⭐⭐ **A FILEIRA `Deform By` EXISTE, SEGUE A SELECÇÃO E O CLIQUE CHEGA AO BARRAMENTO** — a
/// escolha que o dono mandou construir (2026-09-19: *«construa. por desenho»*).
///
/// ⛔⛔ **O clique é REAL (down+up sobre o rectângulo pintado) e não um `WidgetEvent::Click`
/// sintético**, pela razão que esta família pagou SETE vezes: um chip pintado, hit-indexado e
/// **morto sob o dedo** dá exactamente o mesmo report que um chip nunca pintado, e só o gesto real
/// os separa — o sintético prova a allowlist e **pula a focabilidade no store**.
///
/// ⚠️ **As QUATRO metades são quatro defeitos:** sem a (a) a fileira aparece num painel sem sujeito
/// (um selector sem pele, que é a classe de controlo morto do `CLAUDE.md` §5.0); sem a (b) ela não
/// alcança uma IMAGEM (o report de 2026-09-18, que já custou os dois botões de saída); sem a (c) o
/// chip não diz em que lei o desenho está; sem a (d) ele acende e **nada muda**.
#[test]
fn a_fileira_da_lei_de_pele_existe_segue_a_seleccao_e_o_clique_chega() {
    limpa();
    let pintado = |id| {
        let mut host = MockPanelHost::with_panel::<SkeletonPanel>();
        let mut st = SkeletonPanelState;
        host.painted_rect::<SkeletonPanel>(&mut st, VIEWPORT, id)
            .is_some()
    };

    // (a) SEM pele escolhida não há sujeito, e a fileira não existe.
    state::set_current_skinned(state::Skinned::default());
    assert!(
        !pintado(ids::VECTOR_BONE_SKIN_LAW_AUTO),
        "a fileira `Deform By` apareceu sem nada preso escolhido: sem pele nao ha' lei de pele, e \
         um selector sem sujeito e' a classe de controlo morto que o CLAUDE.md §5.0 nomeia"
    );

    // (b) Com uma IMAGEM presa ela existe — a pergunta é a mesma para as duas mídias.
    state::set_current_skinned(state::Skinned {
        vector: false,
        imagem: true,
    });
    assert!(
        pintado(ids::VECTOR_BONE_SKIN_LAW_AUTO) && pintado(ids::VECTOR_BONE_SKIN_LAW_ENVELOPE),
        "a fileira `Deform By` nao alcanca uma IMAGEM presa — e' o report de 2026-09-18 outra vez, \
         que ja' custou os dois botoes de saida"
    );

    // (c) E o clique REAL chega ao barramento, nos DOIS sentidos.
    for id in ids::VECTOR_BONE_SKIN_LAW_IDS {
        let acoes = clica(id, "chip da lei de pele");
        assert!(
            !acoes.is_empty(),
            "o chip da lei de pele esta' MORTO sob o dedo: ele acende e o desenho nao muda, que e' \
             o defeito que esta familia ja' pagou sete vezes"
        );
    }
    limpa();
}

/// ⭐⭐⭐ **A ÂNCORA QUE APONTA MOSTRA OUTRA COISA — e a lista é MEDIDA, não escolhida.**
///
/// A `sonda_do_apontar_tests` da `ph2d-app-skeleton` mediu que, com a corrente resolvida em UM, a
/// *Softness* e o *Bend* movem o osso **zero** e a *Mix* continua a mandar. ⛔ Pintá-los ali seria
/// a classe de controlo morto que o `CLAUDE.md` §5.0 nomeia — o artista mexe e nada acontece.
///
/// ⚠️ **As DUAS metades são dois defeitos diferentes:** esconder o que é vivo apaga um controlo, e
/// pintar o que é morto promete um efeito que não existe. Um gate com uma metade só fica verde
/// sobre a outra.
#[test]
fn a_ancora_que_aponta_esconde_o_que_a_medicao_diz_ser_inerte() {
    let pintado = |id: ph2d_a11y::NodeId| {
        let mut host = MockPanelHost::with_panel::<SkeletonPanel>();
        let mut st = SkeletonPanelState;
        host.painted_rect::<SkeletonPanel>(&mut st, VIEWPORT, id)
            .is_some()
    };
    publica_tudo();
    state::set_current_bone_ik(Some((1.0, 0.0, 2.0, ph2d_skeleton::BendSide::Keep)));

    // (a) Ela ALCANÇA: a suavidade e o lado existem, o desvio não.
    state::set_current_bone_aim(None);
    assert!(
        pintado(ids::VECTOR_BONE_IK_SOFTNESS) && pintado(ids::VECTOR_BONE_IK_BEND_AUTO),
        "a suavidade ou o lado sumiram de uma ancora que ALCANCA: ali os dois mandam, e escondê-los \
         apaga dois controlos vivos"
    );
    assert!(
        !pintado(ids::VECTOR_BONE_IK_OFFSET),
        "o desvio apareceu numa ancora que ALCANCA: somá-lo ali quebraria o alcance que ela acabou \
         de resolver, e o painel estaria a prometê-lo"
    );

    // (b) Ela APONTA: o desvio existe, os dois inertes somem — e a mistura FICA.
    state::set_current_bone_aim(Some(0.0));
    assert!(
        pintado(ids::VECTOR_BONE_IK_OFFSET),
        "o desvio nao e' pintado numa ancora que APONTA: e' o unico numero novo que ela le'"
    );
    assert!(
        !pintado(ids::VECTOR_BONE_IK_SOFTNESS) && !pintado(ids::VECTOR_BONE_IK_BEND_AUTO),
        "a suavidade ou o lado ficaram numa ancora que APONTA: medido, os dois movem o osso ZERO ali"
    );
    assert!(
        pintado(ids::VECTOR_BONE_IK_MIX),
        "a MISTURA sumiu: ela e' o unico dos tres que continua a mandar, e sem ela o apontar nao se \
         desliga"
    );
    limpa();
}

/// ⭐⭐ **E as DUAS portas de entrada existem sem âncora** — *Add IK* (alcançar) e *Look At*
/// (apontar).
///
/// ⚠️ Sem a segunda, o apontar continua a ser alcançável **só** por quem souber pôr o `IK Chain` em
/// `1` à mão — que é a definição de um motor que o artista não tem.
#[test]
fn as_duas_portas_da_ancora_sao_oferecidas() {
    publica_tudo();
    state::set_current_bone_ik(None);
    state::set_current_bone_aim(None);
    for id in [ids::VECTOR_BONE_IK_ADD, ids::VECTOR_BONE_LOOK_AT] {
        let mut host = MockPanelHost::with_panel::<SkeletonPanel>();
        let mut st = SkeletonPanelState;
        assert!(
            host.painted_rect::<SkeletonPanel>(&mut st, VIEWPORT, id)
                .is_some(),
            "{id:?} nao e' pintado sem ancora: uma das duas portas de entrada nao existe"
        );
    }
    limpa();
}
