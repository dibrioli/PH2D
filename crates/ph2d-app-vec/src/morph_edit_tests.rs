//! Os gates da costura das setas — **a lei**, que não precisa de janela, e o **fio**, que só se lê.

use super::{MorphCmd, apply, morph_cmd_for_id, morph_of_selection, publish};
use ph2d_ecs::{Entity, Name, SimWorld, VecMorph, VecMorphMachine};
use ph2d_morph_machine::MorphKey;
use ph2d_vec_scene::{VecPathId, VecScene};

use ph2d_vec_entities::entities::{VecEntityMap, sync};

fn actions() -> Vec<String> {
    vec!["jump".to_string(), "dash".to_string()]
}

/// **Um conjunto de DUAS formas, montado pela porta real**, com a 2.ª a responder ao `jump`.
///
/// ⚠️ **Deixou de poder ser fabricado à mão na W11**: a lista são os FILHOS, e um harness que os
/// dispensasse testaria um mundo que o produto não sabe produzir.
fn world() -> (SimWorld, VecEntityMap, Entity, Vec<VecPathId>) {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let ids: Vec<VecPathId> = (0..2)
        .map(|i| {
            #[allow(clippy::cast_precision_loss)]
            let x = i as f64 * 5.0;
            scene.push_path(ph2d_vec_scene::rectangle([x, -1.0], [x + 2.0, 1.0]))
        })
        .collect();
    sync(&mut sim, &mut scene, &mut map);
    let mut pending = ph2d_vec_entities::morph_set::create(&sim, &mut scene, &map, &ids, 9);
    sync(&mut sim, &mut scene, &mut map);
    ph2d_vec_entities::morph_set::upkeep(&mut sim, &scene, &map, &mut pending);
    let host = Entity::from_bits(map[&scene.paths().last().unwrap().id]);
    sim.world_mut()
        .get_mut::<VecMorphMachine>(host)
        .unwrap()
        .keys
        .insert(
            ids[1],
            MorphKey {
                when: "jump".into(),
                ..Default::default()
            },
        );
    (sim, map, host, ids)
}

/// A tecla da forma `row` deste conjunto.
fn key_at(sim: &SimWorld, map: &VecEntityMap, host: Entity, row: usize) -> String {
    ph2d_vec_entities::morph_set::graph_of(sim, map, host).states[row]
        .when
        .clone()
}

/// **Cada id de linha resolve para o comando dela** — a tabela, e não uma cadeia de `if`.
///
/// **Mutação que deve sangrar:** o laço parar na primeira linha (`0..1`) — as transições a partir
/// da segunda ficariam **mortas sob o ponteiro**, e só a de cima funcionaria.
#[test]
fn every_arrow_row_resolves_to_its_own_command() {
    // ⭐ O botão que FAZ o conjunto (W8) — ele é o único controlo da seção sem máquina nenhuma.
    assert_eq!(
        morph_cmd_for_id(ph2d_panel_vector::ids::VECTOR_MORPH_STATES_MAKE),
        Some(MorphCmd::MakeSet)
    );
    for row in 0..ph2d_panel_vector::ids::MAX_MORPH_STATES {
        assert_eq!(
            morph_cmd_for_id(ph2d_panel_vector::ids::morph_shape_key_option_id(row, 3)),
            Some(MorphCmd::SetWhen { row, action: 3 }),
            "a opcao 3 da linha {row} nao resolve"
        );
    }
    // O CONTROLE: um id que não é da seção tem de devolver `None`, senão a tabela engoliria
    // cliques alheios.
    assert_eq!(
        morph_cmd_for_id(ph2d_panel_vector::ids::VECTOR_BOOL_UNION),
        None
    );
}

/// ⭐ **A CONDIÇÃO é escolhida pelo ÍNDICE do menu, e o `0` é o «—».**
///
/// **Mutação que deve sangrar:** tratar o `0` como a primeira acção — tirar a condição passaria a
/// pôr `jump`, e o artista não teria gesto nenhum para a limpar.
#[test]
fn the_first_option_clears_the_condition_and_the_rest_pick_an_action() {
    let (mut sim, map, e, ids) = world();
    let _ = &ids;
    assert!(apply(
        &mut sim,
        &map,
        e,
        MorphCmd::SetWhen { row: 1, action: 2 },
        &actions()
    ));
    assert_eq!(key_at(&sim, &map, e, 1), "dash");
    assert!(apply(
        &mut sim,
        &map,
        e,
        MorphCmd::SetWhen { row: 1, action: 0 },
        &actions()
    ));
    assert_eq!(
        key_at(&sim, &map, e, 1),
        "",
        "o «—» tem de LIMPAR a tecla da forma"
    );
}

/// ⛔ **Um índice fora da lista publicada RECUSA**, em vez de escrever um nome inventado.
///
/// ⚠️ Ele é alcançável: o mapa pode mudar entre o menu abrir e o clique chegar.
#[test]
fn an_index_beyond_the_published_list_refuses() {
    let (mut sim, map, e, ids) = world();
    let _ = &ids;
    assert!(!apply(
        &mut sim,
        &map,
        e,
        MorphCmd::SetWhen { row: 1, action: 99 },
        &actions()
    ));
    assert_eq!(
        key_at(&sim, &map, e, 1),
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
    let mut seen = 0usize;
    for row in 0..ph2d_panel_vector::ids::MAX_MORPH_STATES {
        for a in 0..ph2d_panel_vector::ids::MAX_MORPH_ACTIONS {
            let cmd = morph_cmd_for_id(ph2d_panel_vector::ids::morph_shape_key_option_id(row, a));
            assert!(matches!(cmd, Some(MorphCmd::SetWhen { .. })));
            seen += 1;
        }
    }
    // O CONTROLE POSITIVO: o laço de facto correu sobre o pool inteiro.
    assert_eq!(
        seen,
        ph2d_panel_vector::ids::MAX_MORPH_STATES * ph2d_panel_vector::ids::MAX_MORPH_ACTIONS
    );
    // E a lista continua intacta depois de o único verbo dela correr.
    let (mut sim, map, e, ids) = world();
    let _ = &ids;
    assert!(apply(
        &mut sim,
        &map,
        e,
        MorphCmd::SetWhen { row: 1, action: 0 },
        &actions()
    ));
    assert_eq!(
        ph2d_vec_entities::morph_set::graph_of(&sim, &map, e)
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
    let (mut sim, _map, e, _ids) = world();
    let other = sim.world_mut().spawn(Name("um grupo".to_string())).id();
    // O MAPA `forma -> entidade`, que é a porta pela qual a seleção do vetor se resolve.
    let mut map = ph2d_vec_entities::entities::VecEntityMap::default();
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
    let (sim, _map, _e, _ids) = world();
    let empty = ph2d_vec_entities::entities::VecEntityMap::default();
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
    let e = sim.world_mut().spawn(VecMorph::new(10, 20)).id();
    let scene = ph2d_vec_scene::VecScene::new();
    let mut map = ph2d_vec_entities::entities::VecEntityMap::default();
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
    let mut map = ph2d_vec_entities::entities::VecEntityMap::default();
    let ids: Vec<u64> = (0..3)
        .map(|_| scene.push_path(ph2d_vec_scene::VecPath::default()))
        .collect();
    ph2d_vec_entities::entities::sync(&mut sim, &mut scene, &mut map);

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
