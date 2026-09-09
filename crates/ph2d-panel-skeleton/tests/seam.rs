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
    state::set_current_skinned(true);
    state::set_current_bone(Some((20.0, 1.0)));
}

fn limpa() {
    state::set_current_skinned(false);
    state::set_current_bone(None);
    state::set_current_bone_ik(None);
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
    // ⚠️ Os três segmentos do LADO vivem com os números da âncora — eles só têm sujeito quando ela
    // existe. A lista é derivada da tabela dos segmentos, não escrita à mão: é a mesma lição que
    // pôs a população deste ficheiro em `VECTOR_BONE_VERBS`.
    let precisa_de_ancora = id == ids::VECTOR_BONE_IK_REMOVE
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
        || id == ids::VECTOR_BONE_SMART_CLIP
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
    estado_de(ids::VECTOR_BONE_SMART_CLIP);
    let mut host = MockPanelHost::with_panel::<SkeletonPanel>();
    let mut st = SkeletonPanelState;
    let chip = host
        .painted_rect::<SkeletonPanel>(&mut st, VIEWPORT, ids::VECTOR_BONE_SMART_CLIP)
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
    estado_de(ids::VECTOR_BONE_SMART_CLIP);
    let mut host = MockPanelHost::with_panel::<SkeletonPanel>();
    let mut st = SkeletonPanelState;
    let chip = host
        .painted_rect::<SkeletonPanel>(&mut st, VIEWPORT, ids::VECTOR_BONE_SMART_CLIP)
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
        host.painted_rect::<SkeletonPanel>(&mut st, VIEWPORT, ids::VECTOR_BONE_SMART_CLIP)
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
    state::set_current_skinned(false);
    state::set_current_bone(None);
    assert!(sem(ids::VECTOR_BONE_BIND), "o Bind tem de estar la' sempre");
    assert!(
        !sem(ids::VECTOR_BONE_EXPAND) && !sem(ids::VECTOR_BONE_RELEASE),
        "as saidas foram pintadas sem nada preso"
    );
    state::set_current_skinned(true);
    assert!(
        sem(ids::VECTOR_BONE_EXPAND) && sem(ids::VECTOR_BONE_RELEASE),
        "as saidas sumiram com uma forma presa"
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
    state::set_current_skinned(false);
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
    state::set_current_bone(Some((20.0, 1.0)));
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
        (ids::VECTOR_BONE_ACT_CREATE, "Create"),
        (ids::VECTOR_BONE_ACT_TRANSFORM, "Transform"),
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
        pintado(ids::VECTOR_BONE_ACT_CREATE) && pintado(ids::VECTOR_BONE_ACT_TRANSFORM),
        "a porta do modo Osso não é pintada numa cena sem ossos — o artista abre o painel pelo menu \
         e não tem gesto nenhum que crie o primeiro"
    );
    // ⚠️ **Qual deles ACENDE mede-se na porta** (`section::aceso`, com gate próprio): o estado
    // *aceso* de um segmento não vive no `WidgetStore` — ele é passado ao pintor a cada quadro, e
    // este arnês só vê o retângulo. *Um gate que afirmasse a cor aqui mediria o instrumento.*
    state::set_current_bone_tool(Some(1));
    assert!(
        pintado(ids::VECTOR_BONE_ACT_CREATE) && pintado(ids::VECTOR_BONE_ACT_TRANSFORM),
        "com um osso escolhido os dois segmentos continuam a ser oferecidos"
    );
    state::set_current_bone_tool(None);
}
