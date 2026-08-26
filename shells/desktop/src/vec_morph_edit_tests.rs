//! Os gates da costura das setas — **a lei**, que não precisa de janela, e o **fio**, que só se lê.

use super::{MorphCmd, apply, morph_cmd_for_id, morph_of_selection, publish};
use ph2d_ecs::{Name, SimWorld, VecMorph, VecMorphMachine};
use ph2d_morph_machine::{MorphGraph, MorphState};

const A: u64 = 10;
const B: u64 = 20;

fn actions() -> Vec<String> {
    vec!["jump".to_string(), "dash".to_string()]
}

fn world() -> (SimWorld, ph2d_ecs::Entity) {
    let mut sim = SimWorld::new();
    let mut m = VecMorphMachine::new(&[A]);
    let mut b = MorphState::new(B);
    b.when = "jump".to_string();
    m.graph = MorphGraph {
        states: vec![MorphState::new(A), b],
    };
    let e = sim.world_mut().spawn((VecMorph::new(A, B), m)).id();
    (sim, e)
}

/// **Cada id de linha resolve para o comando dela** — a tabela, e não uma cadeia de `if`.
///
/// **Mutação que deve sangrar:** o laço parar na primeira linha (`0..1`) — as transições a partir
/// da segunda ficariam **mortas sob o ponteiro**, e só a de cima funcionaria.
#[test]
fn every_arrow_row_resolves_to_its_own_command() {
    use ph2d_editor::ids as i;
    // ⭐ O botão que FAZ o conjunto (W8) — ele é o único controlo da seção sem máquina nenhuma.
    assert_eq!(
        morph_cmd_for_id(i::VECTOR_MORPH_STATES_MAKE),
        Some(MorphCmd::MakeSet)
    );
    for row in 0..i::MAX_MORPH_STATES {
        assert_eq!(
            morph_cmd_for_id(i::morph_shape_key_option_id(row, 3)),
            Some(MorphCmd::SetWhen { row, action: 3 }),
            "a opcao 3 da linha {row} nao resolve"
        );
    }
    // O CONTROLE: um id que não é da seção tem de devolver `None`, senão a tabela engoliria
    // cliques alheios.
    assert_eq!(morph_cmd_for_id(ph2d_editor::ids::VECTOR_BOOL_UNION), None);
}

/// ⭐ **A CONDIÇÃO é escolhida pelo ÍNDICE do menu, e o `0` é o «—».**
///
/// **Mutação que deve sangrar:** tratar o `0` como a primeira acção — tirar a condição passaria a
/// pôr `jump`, e o artista não teria gesto nenhum para a limpar.
#[test]
fn the_first_option_clears_the_condition_and_the_rest_pick_an_action() {
    let (mut sim, e) = world();
    assert!(apply(
        &mut sim,
        e,
        MorphCmd::SetWhen { row: 1, action: 2 },
        &actions()
    ));
    assert_eq!(
        sim.world().get::<VecMorphMachine>(e).unwrap().graph.states[1].when,
        "dash"
    );
    assert!(apply(
        &mut sim,
        e,
        MorphCmd::SetWhen { row: 1, action: 0 },
        &actions()
    ));
    assert_eq!(
        sim.world().get::<VecMorphMachine>(e).unwrap().graph.states[1].when,
        "",
        "o «—» tem de LIMPAR a tecla da forma"
    );
}

/// ⛔ **Um índice fora da lista publicada RECUSA**, em vez de escrever um nome inventado.
///
/// ⚠️ Ele é alcançável: o mapa pode mudar entre o menu abrir e o clique chegar.
#[test]
fn an_index_beyond_the_published_list_refuses() {
    let (mut sim, e) = world();
    assert!(!apply(
        &mut sim,
        e,
        MorphCmd::SetWhen { row: 1, action: 99 },
        &actions()
    ));
    assert_eq!(
        sim.world().get::<VecMorphMachine>(e).unwrap().graph.states[1].when,
        "jump",
        "a tecla antiga tem de ficar intacta"
    );
}

/// ⛔ **NÃO HÁ como apagar uma linha, e a ausência é a lei.**
///
/// A lista **É** o conjunto de formas do objecto. Apagar uma linha seria tirar uma forma do
/// conjunto — outro gesto, que ainda não existe. *Desligar uma forma é tirar-lhe a tecla* (o «—»
/// do menu), e uma forma sem tecla existe e nunca é alcançada.
///
/// ⚠️ **Este gate mede a AUSÊNCIA pelo lado que o artista alcança:** nenhum id da seção resolve
/// para outra coisa que não `MakeSet` ou `SetWhen`. Um verbo destrutivo que voltasse a ser
/// alcançável sangraria aqui.
#[test]
fn no_id_in_the_section_asks_to_destroy_a_state() {
    use ph2d_editor::ids as i;
    let mut seen = 0usize;
    for row in 0..i::MAX_MORPH_STATES {
        for a in 0..i::MAX_MORPH_ACTIONS {
            let cmd = morph_cmd_for_id(i::morph_shape_key_option_id(row, a));
            assert!(matches!(cmd, Some(MorphCmd::SetWhen { .. })));
            seen += 1;
        }
    }
    // O CONTROLE POSITIVO: o laço de facto correu sobre o pool inteiro.
    assert_eq!(seen, i::MAX_MORPH_STATES * i::MAX_MORPH_ACTIONS);
    // E a lista continua intacta depois de o único verbo dela correr.
    let (mut sim, e) = world();
    assert!(apply(
        &mut sim,
        e,
        MorphCmd::SetWhen { row: 1, action: 0 },
        &actions()
    ));
    assert_eq!(
        sim.world()
            .get::<VecMorphMachine>(e)
            .unwrap()
            .graph
            .states
            .len(),
        2,
        "tirar a tecla NAO tira a forma da lista"
    );
}

/// ⭐ **O Morph é achado na seleção INTEIRA, nunca no primeiro operando.**
///
/// **Mutação que deve sangrar:** usar `sel.first()` — tocar num morph traz o grupo, e a seção
/// mostraria as setas de um objecto enquanto o clique escreveria noutro.
#[test]
fn the_morph_is_found_anywhere_in_the_selection() {
    let (mut sim, e) = world();
    let other = sim.world_mut().spawn(Name("um grupo".to_string())).id();
    // O MAPA `forma -> entidade`, que é a porta pela qual a seleção do vetor se resolve.
    let mut map = crate::vec_entities::VecEntityMap::default();
    map.insert(1, other.to_bits());
    map.insert(2, e.to_bits());
    assert_eq!(morph_of_selection(&sim, &map, &[1, 2]), Some(e));
    // O CONTROLE: sem morph nenhum, `None`.
    assert_eq!(morph_of_selection(&sim, &map, &[1]), None);
}

/// ⛔⛔⛔ **UM ID DE FORMA NUNCA É LIDO COMO BITS DE ENTIDADE — e o `0` é o que MATA o processo.**
///
/// ⚠️ **É a regressão do pânico do smoke de 2026-08-25** (`PH2D_BUILD_SMOKE=74`, quadro 1639,
/// *"Attempted to initialize invalid bits as an entity"*), e o mecanismo está **medido**:
///
/// | `Entity::from_bits(v)` | resultado |
/// |---|---|
/// | `0` | ⛔ **PÂNICO** (`bevy_ecs/entity/mod.rs:580`) |
/// | `1` | `PLACEHOLDER` |
/// | `2`, `3`, `4` | uma entidade de **lixo** (`4294967293v0`), que nunca tem componente nenhum |
///
/// ⇒ o defeito tinha **duas caras**: com ids pequenos a seção simplesmente **nunca achava o
/// morph** (silêncio), e com o id **`0`** o app **morria**. ⭐ E o `0` não é um caso de canto: o
/// `VecScene` deriva `Default`, então `next_id` nasce em `0` e a **primeira forma da cena** tem
/// id `0`. Clicar nela era o gesto que matava.
///
/// ⛔⛔ **E a primeira versão deste gate NÃO apanhava nada:** ela alimentava `[1, 2, 3]`, que
/// decodificam para lixo mas **não entram em pânico** — a mutação sobreviveu, e foi isso que me
/// obrigou a medir em vez de supor. *Uma fixtura que não contém o fenómeno aprova a cura errada.*
#[test]
fn a_shape_id_is_never_read_as_entity_bits() {
    let (sim, _e) = world();
    let empty = crate::vec_entities::VecEntityMap::default();
    // ⭐ O `0` PRIMEIRO: e' o id da primeira forma de toda cena, e e' o que mata o processo.
    assert_eq!(morph_of_selection(&sim, &empty, &[0]), None);
    // E os pequenos, que nao matam -- eles achavam a entidade ERRADA, em silencio.
    assert_eq!(morph_of_selection(&sim, &empty, &[0, 1, 2, 3]), None);
}

/// ⭐⭐ **Um Morph SEM máquina publica a face VAZIA — e nunca `None`.**
///
/// ⚠️ As duas coisas pintam faces diferentes: `None` = *"a seleção não é um Morph"* (a seção nem
/// fala de setas); vazio = *"é um Morph e ainda não tem setas"*, e é essa face que diz **como**
/// desenhar a primeira. Sem ela o artista vê um cabeçalho e nada por baixo.
///
/// **Mutação que deve sangrar:** o `publish` devolver `None` quando não há máquina.
#[test]
fn a_morph_without_a_machine_publishes_the_empty_face() {
    let mut sim = SimWorld::new();
    let e = sim.world_mut().spawn(VecMorph::new(A, B)).id();
    let scene = ph2d_vec_scene::VecScene::new();
    let mut map = crate::vec_entities::VecEntityMap::default();
    map.insert(7, e.to_bits());
    let s = publish(&sim, &scene, &map, &[7], false, actions())
        .expect("um Morph SEM maquina ainda publica");
    assert!(s.rows.is_empty(), "e a lista de setas vem vazia");
    assert_eq!(
        s.can_make, 0,
        "⛔ um conjunto ja' feito nao oferece o botao de o refazer por cima de si proprio"
    );
    assert_eq!(
        s.actions,
        actions(),
        "as accoes vem sempre -- o menu precisa delas"
    );
}

/// **A COSTURA: o clique da seta atravessa o barramento e chega ao mundo.**
///
/// ⚠️ Um gate de TEXTO, e a razão é a mesma do gesto: o caminho real precisa de um `AppGfx`. A
/// **lei** está gateada acima; o que sobra é provar que alguém a chama — e isso só se pode ler.
///
/// ⛔ Sem esta metade, os gates acima ficariam verdes sobre controlos que **pintam, acendem sob o
/// rato e cujo clique morre no painel** — que é o defeito que a décima lista do modo custou uma
/// wave atrás.
#[test]
fn the_arrow_click_reaches_the_world() {
    let shell = include_str!("render_loop/mod.rs");
    for (needle, what) in [
        (
            "crate::vec_morph_edit::morph_cmd_for_id(*id)",
            "RESOLVER o id do clique",
        ),
        (
            "crate::vec_morph_edit::apply(sim, e, cmd, &actions)",
            "APLICAR ao mundo",
        ),
        (
            "morph_of_selection(sim, &self.vec_entities, &sel)",
            "RESOLVER a seleccao pelo MAPA, e nunca como bits de entidade",
        ),
        (
            "set_morph_states_state(crate::vec_morph_edit::publish(",
            "PUBLICAR a projeccao",
        ),
    ] {
        assert!(
            shell.contains(needle),
            "a shell perdeu o `{needle}` -- {what}. A lei continua gateada e INALCANCAVEL."
        );
    }
    // ⭐ E o CONJUNTO sai por outra porta, porque ele cria a entidade que o `apply` exigiria já
    // existente. Sem esta linha o botão pinta, acende e o clique morre no `else if` do irmão.
    for (needle, what) in [
        (
            "pending_morph_arrow == Some(crate::vec_morph_edit::MorphCmd::MakeSet)",
            "reconhecer o pedido de FAZER o conjunto",
        ),
        (
            "crate::morph_set::create(",
            "CRIAR o conjunto (o path novo + o pendente)",
        ),
        (
            "crate::morph_set::upkeep(",
            "DRENAR o pendente: pendurar a maquina, reparentar e esconder os membros",
        ),
        (
            "self.vec_pen.select_many(&[p.path]);",
            "SELECCIONAR o conjunto novo -- senao a seleccao fica nos MEMBROS, que acabaram de \
             ficar ocultos e com dono, e a seccao oferece um SEGUNDO conjunto sobre eles",
        ),
    ] {
        assert!(
            shell.contains(needle),
            "a shell perdeu o `{needle}` -- {what}. O botao pinta e nao faz nada."
        );
    }
    // ⭐⭐ **O MODO DE PRÉ-VISUALIZAÇÃO** (W9): o interruptor, o que ele dirige, e — a metade que
    // dá sentido ao modo — o teclado que ele TOMA. ⛔ Sem a guarda, a tecla morfa a forma **e** faz
    // o que ela faz no editor: é o report do Enio (*"as setas do teclado movendo as formas"*).
    for (needle, what) in [
        (
            "*id == ph2d_editor::ids::VECTOR_MORPH_PREVIEW",
            "RECONHECER o clique no interruptor",
        ),
        (
            "self.morph_preview = !self.morph_preview",
            "LIGAR e DESLIGAR o modo",
        ),
        (
            "self.morph_preview,\n                self.fixed_step.fixed_dt(),",
            "DIRIGIR a maquina pelo MODO, e nao pelo playhead",
        ),
    ] {
        assert!(
            shell.contains(needle),
            "a shell perdeu o `{needle}` -- {what}."
        );
    }
    // ⛔ **E o playhead NÃO pode voltar a ser a porta:** ele não tranca o teclado do editor, que é
    // exactamente o conflito que este modo existe para curar.
    assert!(
        !shell.contains("self.playhead.is_playing(),\n                self.fixed_step.fixed_dt(),"),
        "o playhead voltou a dirigir a maquina -- o conflito de atalhos volta com ele"
    );
    let modal = include_str!("input_dispatch/keyboard_modal.rs");
    for (needle, what) in [
        (
            "if !self.morph_preview || self.modifiers.control_key()",
            "TOMAR o teclado enquanto o modo corre (e deixar passar os acordes)",
        ),
        ("self.morph_preview_leave = true", "o Esc PEDIR a saida"),
    ] {
        assert!(
            modal.contains(needle),
            "a porta modal perdeu o `{needle}` -- {what}. A tecla faz DUAS coisas."
        );
    }
    // ⚠️⚠️ **A ORDEM na cadeia é metade do desenho, e este é o gate dela.**
    //
    // A porta tem de correr **DEPOIS** do retrato dos dispositivos (`input.apply_event`) — barrar
    // antes mataria a própria acção que a máquina lê, e o modo ficaria **inerte com o teclado
    // tomado**, que é o pior dos dois mundos — e **ANTES** do primeiro consumidor do editor.
    let kb = include_str!("input_dispatch/keyboard.rs");
    let feed = kb
        .find("self.input.apply_event")
        .expect("o retrato dos dispositivos sumiu");
    let gate = kb
        .find("self.modal_owns_the_keyboard(")
        .expect("a porta modal deixou de ser chamada -- a tecla volta a fazer duas coisas");
    let editor = kb
        .find("crate::flip_peek::key_transition")
        .expect("o primeiro consumidor do editor sumiu");
    assert!(
        feed < gate,
        "a porta modal corre ANTES do retrato dos dispositivos: a maquina fica MUDA e o teclado \
         fica tomado ao mesmo tempo"
    );
    assert!(
        gate < editor,
        "a porta modal corre DEPOIS de um consumidor do editor: a tecla morfa a forma E faz o que \
         ela faz no editor -- e' o report do Enio de volta"
    );

    // E o painel tem de FORWARDAR os cliques, senão eles morrem antes de chegar aqui.
    let panel = include_str!("../../../crates/ph2d-panel-vector/src/event_clicks.rs");
    for (needle, what) in [
        (
            "ids::VECTOR_MORPH_STATES_MAKE",
            "o botao que faz o conjunto",
        ),
        (
            "ids::VECTOR_MORPH_PREVIEW",
            "o interruptor da pre-visualizacao",
        ),
        (
            "morph_shape_key_option_id(r, a)",
            "a opcao do menu da condicao",
        ),
    ] {
        assert!(
            panel.contains(needle),
            "o painel deixou de encaminhar {what}: ele acende sob o rato e morre ali"
        );
    }
}

/// ⭐⭐ **UMA SELEÇÃO DE FORMAS SOLTAS PUBLICA A FACE QUE TRAZ O BOTÃO** (plano 32 W8).
///
/// ⚠️ **É a costura que torna a feature alcançável de todo.** Os gates do painel provam que o botão
/// está vivo *quando a shell publica `can_make`*; este prova que ela publica. Sem ele os dois lados
/// ficariam verdes sobre uma seção que **nunca aparece** — o artista escolhe três formas e o painel
/// não menciona estados.
///
/// **Mutação que deve sangrar:** o `publish` voltar ao `?` (devolver `None` sem Morph na seleção) —
/// a única porta para a máquina de estados só se abriria depois de a máquina existir.
#[test]
fn a_plain_multi_selection_publishes_the_face_that_offers_the_button() {
    let mut sim = SimWorld::new();
    let mut scene = ph2d_vec_scene::VecScene::new();
    let mut map = crate::vec_entities::VecEntityMap::default();
    let ids: Vec<u64> = (0..3)
        .map(|_| scene.push_path(ph2d_vec_scene::VecPath::default()))
        .collect();
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);

    let s = publish(&sim, &scene, &map, &ids, false, actions())
        .expect("tres formas soltas TEM de publicar -- e' a unica porta para a feature");
    assert_eq!(
        s.can_make, 3,
        "a contagem e' o que a face usa para prometer 3x2"
    );
    assert!(
        s.rows.is_empty(),
        "ainda nao ha' maquina, entao nao ha' transicoes"
    );
    assert_eq!(
        s.actions,
        actions(),
        "as accoes vem sempre -- o menu precisa delas"
    );

    // ⛔ E UMA forma só **não** publica: a seção não pode aparecer onde não há nada a oferecer.
    assert!(
        publish(&sim, &scene, &map, &ids[..1], false, actions()).is_none(),
        "com UMA forma a seccao tem de sumir inteira"
    );
    // O CONTROLE da seleção vazia, que é o estado normal do app.
    assert!(publish(&sim, &scene, &map, &[], false, actions()).is_none());
}
