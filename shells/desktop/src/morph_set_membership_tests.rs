//! ⭐⭐⭐ **A PERTENÇA A UM CONJUNTO, e a ocultação DERIVADA dela** (plano 32 W11f) — irmão de
//! [`super`] pelo teto de 600 LOC.
//!
//! Enio, 2026-08-26: *"sendo uma forma que previamente não participava do Morph states, se for
//! arrastada na hierarquia e se tornar filha de um objeto Morph State, automaticamente passa a
//! fazer parte do sistema."*
//!
//! ⚠️ **Estes gates medem a resposta que o CANVAS lê** (`view_state().hidden`), nunca o componente
//! guardado: desde a W11f a ocultação de um membro é **derivada** de ser filho de um conjunto
//! ([`crate::morph_set::is_set_member`]), e um gate que olhasse o `Visibility` leria `None` sobre
//! uma forma que o canvas não desenha.

use super::super::world;
use super::hidden_on_canvas;
use crate::morph_set::{create, disconnect, graph_of, upkeep};
use ph2d_ecs::{ChildOf, Entity, Visibility};
use ph2d_vec_scene::VecPathId;

use crate::vec_entities::sync;

/// ⭐⭐⭐ **ARRASTAR NA HIERARQUIA MOVE AS DUAS METADES — a lista E o canvas** (plano 32 W11f).
///
/// Enio, 2026-08-26: *"sendo uma forma que previamente não participava do Morph states, se for
/// arrastada na hierarquia e se tornar filha de um objeto Morph State, automaticamente passa a
/// fazer parte do sistema."*
///
/// ⛔⛔ **A W11 entregou METADE disto, e o gate não existia.** A lista passou a ser derivada dos
/// filhos, mas a ocultação continuou a ser uma escrita do `upkeep` — então, MEDIDO em 26/08:
///
/// | gesto | a lista | o canvas (antes) |
/// |---|---|---|
/// | para DENTRO | entrava (3 -> 4) | ⛔ ficava **visível**, desenhada por cima do conjunto |
/// | para FORA | saía (4 -> 3) | ⛔ ficava **escondida** — a forma **desaparecia** |
///
/// ⚠️ E o smoke que eu escrevi **afirmava** que a de dentro sumia do canvas. *Um passo de smoke que
/// descreve o que devia acontecer aprova o defeito.*
///
/// ⚠️ **O gesto é reproduzido como a Hierarquia o faz** — `ChildOf` e mais nada
/// (`hero_intents::drain_reparent`). Um harness que escrevesse o `Visibility` à mão estaria a
/// testar um mundo que o produto não sabe produzir.
///
/// **Mutação que deve sangrar:** o `is_set_member` devolver `false`, ou ignorar o `ChildOf`.
#[test]
fn dragging_into_the_set_hides_and_dragging_out_shows() {
    let (mut sim, mut scene, mut map, ids) = world(4);
    let three: Vec<VecPathId> = ids[..3].to_vec();
    let mut pending = create(&sim, &mut scene, &map, &three, 9);
    sync(&mut sim, &mut scene, &mut map);
    upkeep(&mut sim, &scene, &map, &mut pending);
    let host = Entity::from_bits(map[&scene.paths().last().unwrap().id]);

    // O CONTROLE: a quarta ficou de fora, e por isso ve^-se.
    assert_eq!(graph_of(&sim, &map, host).shapes().len(), 3);
    assert!(
        !hidden_on_canvas(&sim, &map, ids[3]),
        "a fixtura perdeu a premissa: a 4a forma tinha de estar solta e visivel"
    );

    // ⭐ PARA DENTRO — o gesto da Hierarquia e' `ChildOf`, e mais nada.
    let e = Entity::from_bits(map[&ids[3]]);
    sim.world_mut().entity_mut(e).insert(ChildOf(host));
    assert_eq!(
        graph_of(&sim, &map, host).shapes().len(),
        4,
        "arrastar para dentro tem de a por na lista"
    );
    assert!(
        hidden_on_canvas(&sim, &map, ids[3]),
        "⛔ ela entrou na lista e CONTINUA a desenhar-se por cima do conjunto"
    );

    // ⭐ PARA FORA — e a metade que o doc do `disconnect` chamava de «a pior saida possivel».
    let m = Entity::from_bits(map[&ids[1]]);
    sim.world_mut().entity_mut(m).remove::<ChildOf>();
    assert_eq!(graph_of(&sim, &map, host).shapes().len(), 3);
    assert!(
        !hidden_on_canvas(&sim, &map, ids[1]),
        "⛔ ela saiu da lista e CONTINUA invisivel -- a forma desapareceu"
    );
}

/// ⛔ **E o olho do artista SOBREVIVE ao conjunto** — a ocultação derivada só ACRESCENTA uma razão.
///
/// ⚠️ Sem esta lei, `disconnect` teria de remover o `Visibility` (era o que ele fazia até à W11f) e
/// **destruiria** a escolha de quem tivesse escondido a forma pelo olho da Hierarquia **antes** de
/// ela entrar no conjunto. *Um gesto reversível não pode apagar autoria alheia.*
///
/// **Mutação que deve sangrar:** o `disconnect` voltar a fazer `remove::<Visibility>()`.
#[test]
fn the_artists_own_eye_survives_joining_and_leaving_the_set() {
    let (mut sim, mut scene, mut map, ids) = world(3);
    // O artista esconde a forma ANTES de tudo.
    let kid = Entity::from_bits(map[&ids[1]]);
    sim.world_mut().entity_mut(kid).insert(Visibility::hidden());

    let mut pending = create(&sim, &mut scene, &map, &ids, 9);
    sync(&mut sim, &mut scene, &mut map);
    upkeep(&mut sim, &scene, &map, &mut pending);
    assert!(disconnect(&mut sim, &map, ids[1]));

    assert!(
        hidden_on_canvas(&sim, &map, ids[1]),
        "⛔ desconectar RE-ACENDEU uma forma que o artista tinha escondido pelo olho"
    );
    // O CONTROLE: uma forma que ele NAO escondeu volta visivel.
    assert!(disconnect(&mut sim, &map, ids[2]));
    assert!(!hidden_on_canvas(&sim, &map, ids[2]));
}

/// ⛔⛔ **TIRAR A PENÚLTIMA FORMA DISSOLVE O CONJUNTO** — a pergunta do §8.3, respondida por medição.
///
/// **MEDIDO em 2026-08-26:** um conjunto esvaziado pelo ⊘ mantinha o `VecMorph` que o `upkeep` lhe
/// deu, e o `sources` continuava a nomear a **primeira forma** — que já tinha saído. ⇒ o artista
/// desconectava as três e ficava com um **fantasma** com o desenho da primeira, que ele não sabe o
/// que é nem como apagar.
///
/// ⇒ a fronteira é a do `create` (`MIN_STATES = 2`): sair dela **dissolve**, exactamente como o
/// `ungroup` faz com o último filho. *Um objecto deixa de ser uma relação quando fica com um lado
/// só.*
///
/// ⚠️ **A dissolução devolve TUDO** — a forma que se pediu para desconectar **e** a que sobrava,
/// as duas soltas e visíveis. Meia dissolução deixaria a última presa a um pai que já não existe.
///
/// **Mutação que deve sangrar:** o `disconnect_row` desconectar sem olhar para a contagem, ou ler a
/// contagem DEPOIS de desconectar (a fronteira lê-se ao contrário).
#[test]
fn disconnecting_the_last_but_one_dissolves_the_set() {
    let (mut sim, mut scene, mut map, ids) = world(3);
    let mut pending = create(&sim, &mut scene, &map, &ids, 9);
    sync(&mut sim, &mut scene, &mut map);
    upkeep(&mut sim, &scene, &map, &mut pending);
    let host_id = scene.paths().last().unwrap().id;
    let host = Entity::from_bits(map[&host_id]);
    let mut states = ph2d_ui_state::StateSets::default();

    // A primeira saida e' normal: tres formas, sobram duas.
    assert_eq!(
        crate::morph_set::disconnect_row(&mut sim, &map, &mut states, host, 0),
        None,
        "com tres formas o ⊘ nao pode dissolver nada"
    );
    assert_eq!(graph_of(&sim, &map, host).shapes().len(), 2);

    // ⭐ A segunda cruza a fronteira: o conjunto DISSOLVE-SE, e nomeia o path a remover.
    assert_eq!(
        crate::morph_set::disconnect_row(&mut sim, &map, &mut states, host, 0),
        Some(host_id),
        "⛔ ficou um conjunto com UMA forma -- ele desenha um fantasma da primeira"
    );
    for id in &ids {
        assert!(
            !hidden_on_canvas(&sim, &map, *id),
            "a forma {id} tem de voltar VISIVEL -- a dissolucao devolve TODAS"
        );
        assert!(
            sim.world()
                .get::<ChildOf>(Entity::from_bits(map[id]))
                .is_none(),
            "a forma {id} continua presa a um pai que vai deixar de existir"
        );
    }
}
