//! Os gates da costura das setas — **a lei**, que não precisa de janela, e o **fio**, que só se lê.

use super::{ArrowCmd, apply, arrow_cmd_for_id, morph_of_selection, publish};
use ph2d_ecs::{Name, SimWorld, VecMorph, VecMorphMachine};
use ph2d_morph_machine::{MorphEdge, MorphGraph};

const A: u64 = 10;
const B: u64 = 20;

fn actions() -> Vec<String> {
    vec!["jump".to_string(), "dash".to_string()]
}

fn world() -> (SimWorld, ph2d_ecs::Entity) {
    let mut sim = SimWorld::new();
    let mut m = VecMorphMachine::new(A);
    let mut jump = MorphEdge::new(A, B);
    jump.when = "jump".to_string();
    m.graph = MorphGraph {
        start: A,
        edges: vec![jump, MorphEdge::new(B, A)],
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
        arrow_cmd_for_id(i::VECTOR_MORPH_STATES_MAKE),
        Some(ArrowCmd::MakeSet)
    );
    for row in 0..i::MAX_MORPH_ARROWS {
        assert_eq!(
            arrow_cmd_for_id(i::morph_arrow_when_option_id(row, 3)),
            Some(ArrowCmd::SetWhen { row, action: 3 }),
            "a opcao 3 da linha {row} nao resolve"
        );
    }
    // O CONTROLE: um id que não é da seção tem de devolver `None`, senão a tabela engoliria
    // cliques alheios.
    assert_eq!(arrow_cmd_for_id(ph2d_editor::ids::VECTOR_BOOL_UNION), None);
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
        ArrowCmd::SetWhen { row: 1, action: 2 },
        &actions()
    ));
    assert_eq!(
        sim.world().get::<VecMorphMachine>(e).unwrap().graph.edges[1].when,
        "dash"
    );
    assert!(apply(
        &mut sim,
        e,
        ArrowCmd::SetWhen { row: 1, action: 0 },
        &actions()
    ));
    assert_eq!(
        sim.world().get::<VecMorphMachine>(e).unwrap().graph.edges[1].when,
        "",
        "o «—» tem de LIMPAR a condicao"
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
        ArrowCmd::SetWhen { row: 0, action: 99 },
        &actions()
    ));
    assert_eq!(
        sim.world().get::<VecMorphMachine>(e).unwrap().graph.edges[0].when,
        "jump",
        "a condicao antiga tem de ficar intacta"
    );
}

/// ⛔ **NÃO HÁ como apagar uma transição, e a ausência é a lei da W8.**
///
/// O grafo é o completo dirigido sobre as formas do conjunto — **derivado**, não autorado. Apagar
/// uma aresta seria apagar uma passagem que a próxima derivação repõe; *desligar é tirar a
/// condição*, e uma seta sem condição existe e nunca acontece.
///
/// ⚠️ **Este gate mede a AUSÊNCIA pelo lado que o artista alcança:** nenhum id da seção resolve
/// para outra coisa que não `MakeSet` ou `SetWhen`. Um verbo destrutivo que voltasse a ser
/// alcançável sangraria aqui.
#[test]
fn no_id_in_the_section_asks_to_destroy_an_edge() {
    use ph2d_editor::ids as i;
    let mut seen = 0usize;
    for row in 0..i::MAX_MORPH_ARROWS {
        for a in 0..i::MAX_MORPH_ACTIONS {
            let cmd = arrow_cmd_for_id(i::morph_arrow_when_option_id(row, a));
            assert!(matches!(cmd, Some(ArrowCmd::SetWhen { .. })));
            seen += 1;
        }
    }
    // O CONTROLE POSITIVO: o laço de facto correu sobre o pool inteiro.
    assert_eq!(seen, i::MAX_MORPH_ARROWS * i::MAX_MORPH_ACTIONS);
    // E o grafo continua intacto depois de o único verbo de seta correr.
    let (mut sim, e) = world();
    assert!(apply(
        &mut sim,
        e,
        ArrowCmd::SetWhen { row: 0, action: 0 },
        &actions()
    ));
    assert_eq!(
        sim.world()
            .get::<VecMorphMachine>(e)
            .unwrap()
            .graph
            .edges
            .len(),
        2,
        "tirar a condicao NAO tira a seta"
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
    let s =
        publish(&sim, &scene, &map, &[7], actions()).expect("um Morph SEM maquina ainda publica");
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
            "crate::vec_morph_edit::arrow_cmd_for_id(*id)",
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
            "pending_morph_arrow == Some(crate::vec_morph_edit::ArrowCmd::MakeSet)",
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
    ] {
        assert!(
            shell.contains(needle),
            "a shell perdeu o `{needle}` -- {what}. O botao pinta e nao faz nada."
        );
    }
    // E o painel tem de FORWARDAR os dois cliques, senão eles morrem antes de chegar aqui.
    let panel = include_str!("../../../crates/ph2d-panel-vector/src/event_clicks.rs");
    for (needle, what) in [
        (
            "ids::VECTOR_MORPH_STATES_MAKE",
            "o botao que faz o conjunto",
        ),
        (
            "morph_arrow_when_option_id(r, a)",
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

    let s = publish(&sim, &scene, &map, &ids, actions())
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
        publish(&sim, &scene, &map, &ids[..1], actions()).is_none(),
        "com UMA forma a seccao tem de sumir inteira"
    );
    // O CONTROLE da seleção vazia, que é o estado normal do app.
    assert!(publish(&sim, &scene, &map, &[], actions()).is_none());
}
