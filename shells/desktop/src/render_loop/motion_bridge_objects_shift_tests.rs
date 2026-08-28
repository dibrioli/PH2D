//! Gates do **canal da aparência DESLOCADA** (doc 89, folha 14) — irmão de
//! `motion_bridge_objects_tests`, cortado pelo mesmo assunto que o módulo de produto:
//! o pai mede *o que a cena tem*, este *quando olhar para isso*.
//!
//! FILHO de `objects` via `#[path]`, então `use super::*` alcança o `appearance_tile`
//! privado e o `publish_shifted`/`wanted_shifts` do irmão de produto.

use super::*;

// ── o canal da aparência DESLOCADA (doc 89, folha 14) ────────────────────────

/// Um estado com UM `source.object` nomeando `named`, deslocado por `off`.
///
/// ⚠️ **É um `MotionState`, e não um `Graph` solto, desde 2026-08-28:** o `off` pode vir de um
/// FIO, e resolver um param conduzido é cozinhar o driver — o que precisa do registry e do
/// cook, não só do documento.
fn state_with_shifted_source(named: &str, off: f32) -> crate::motion_state::MotionState {
    let mut m = crate::motion_state::MotionState::new();
    let n = m.doc.graph.add_node("source.object");
    m.doc.graph.set_text_param(n, "object", named);
    m.doc
        .graph
        .set_param(n, ph2d_node_source_object::TIME_OFFSET_PARAM, off);
    m
}

/// O mesmo, com o `time_offset` **CONDUZIDO POR UM FIO** em vez de autorado.
fn state_with_driven_shift(named: &str, off: f32) -> crate::motion_state::MotionState {
    let mut m = crate::motion_state::MotionState::new();
    let n = m.doc.graph.add_node("source.object");
    m.doc.graph.set_text_param(n, "object", named);
    let num = m.doc.graph.add_node("value.number");
    m.doc.graph.set_param(num, "value", off);
    m.doc
        .graph
        .drive_param(n, ph2d_node_source_object::TIME_OFFSET_PARAM, (num, 0))
        .expect("o offset aceita fio");
    m
}

/// Semeia o canal cru de `named` no cook do estado e devolve-o pronto a publicar.
fn seed_raw(m: &mut crate::motion_state::MotionState, named: &str, texture_id: u32) {
    m.pump.cook.set_external(
        named.to_string(),
        appearance_tile(
            [2.0, 3.0],
            [1.0, 1.0, 1.0, 1.0],
            [0.0, 0.0, 1.0, 1.0],
            texture_id,
        ),
    );
}

/// **Um offset num meio SEM animação própria é transparente, não um sumiço.**
///
/// Este é o gate que decide se o param é seguro de shipar: um sprite não tem um
/// desenho por quadro, então "meio segundo à frente" tem a mesma resposta que
/// "agora". Sem a cópia, o nó leria um external que ninguém publicou — stream vazio,
/// e o objeto **desaparece da cena** com o param parecendo funcionar noutro objeto.
#[test]
fn a_shift_on_a_still_medium_publishes_the_same_appearance() {
    let mut m = state_with_shifted_source("Ball", 0.25);
    seed_raw(&mut m, "Ball", 7);
    publish_shifted(&mut m, 0.0);

    let key = ph2d_nodegraph::external::appearance_of("Ball", 0.25);
    let e = m
        .pump
        .cook
        .externals()
        .get(&key)
        .expect("o canal deslocado tem de existir, senao o objeto some");
    assert_eq!(
        e.value.get("texture_id"),
        Some(&Column::Scalar(vec![7.0])),
        "um meio sem animacao mostra a MESMA coisa deslocado"
    );
}

/// **Um documento que não desloca ninguém publica exatamente o que sempre publicou.**
/// A neutralidade da wave, vista da membrana: nenhuma chave nova, nenhuma tile a mais.
#[test]
fn an_unshifted_document_publishes_no_extra_channel() {
    // O param escrito EXPLICITAMENTE em zero — um teste que chega ao estado por
    // omissão inverte de sentido no dia em que o default se mover.
    let mut m = state_with_shifted_source("Ball", 0.0);
    seed_raw(&mut m, "Ball", 7);
    let before = m.pump.cook.externals().len();
    publish_shifted(&mut m, 0.0);
    assert_eq!(
        m.pump.cook.externals().len(),
        before,
        "offset zero nao pode cunhar canal nenhum"
    );
}

/// **A tile do FLIP no quadro deslocado VENCE a cópia transparente** — o P0 desta
/// wave. O par com o gate acima: um deles sozinho seria satisfeito por uma membrana
/// que copia sempre (e o offset seria um controle morto no único meio que o tem).
#[test]
fn a_flips_shifted_tile_beats_the_transparent_copy() {
    let mut m = state_with_shifted_source("Walk", 0.25);
    seed_raw(&mut m, "Walk", 7);
    // O bake respondeu o pedido deslocado com OUTRA tile (texture_id 90) — que é o
    // que um desenho diferente é, do lado do render.
    m.flip_object_bake
        .seed_named_shift_for_test(1, "Walk", 0.25, 90, [3.0, 4.0]);
    publish_shifted(&mut m, 0.0);

    let key = ph2d_nodegraph::external::appearance_of("Walk", 0.25);
    let e = m
        .pump
        .cook
        .externals()
        .get(&key)
        .expect("o canal deslocado existe");
    assert_eq!(
        e.value.get("texture_id"),
        Some(&Column::Scalar(vec![90.0])),
        "a tile do quadro deslocado tem de vencer a copia"
    );
}

/// **Os offsets que o bake assa são os que o DOCUMENTO pede, com o zero sempre
/// dentro** — e distintos, para que dois nós pedindo o mesmo deslocamento não
/// custem duas tiles.
#[test]
fn the_wanted_shifts_are_the_documents_own_plus_zero() {
    let mut m = crate::motion_state::MotionState::new();
    for off in [0.25_f32, 0.25, -0.5, 0.0] {
        let n = m.doc.graph.add_node("source.object");
        m.doc.graph.set_text_param(n, "object", "Walk");
        m.doc
            .graph
            .set_param(n, ph2d_node_source_object::TIME_OFFSET_PARAM, off);
    }
    let mut got = wanted_shifts(&mut m, 0.0);
    got.sort_by(|a, b| a.partial_cmp(b).unwrap());
    assert_eq!(
        got,
        vec![-0.5, 0.0, 0.25],
        "zero + os distintos, sem repetir"
    );
}

/// **UM `time_offset` CONDUZIDO POR FIO CUNHA A MESMA CHAVE QUE O NÓ VAI LER.**
///
/// ⚠️ **O defeito que este gate fecha era invisível a todos os outros deste arquivo.** O
/// `eval` do `source.object` monta a chave com `ctx.param(TIME_OFFSET_PARAM)`, que resolve
/// `conduzido → override → default`; esta membrana lia só `override → default`. Ligue um fio
/// ao offset e as duas chaves DIVERGEM: o nó pede uma aparência que ninguém publicou, o stream
/// vem vazio, e **o objeto desaparece da cena** — sem erro, sem aviso, com o mesmo param a
/// funcionar noutro objeto que não tenha fio.
///
/// ⚠️ **O CONTROLO é o `override` a discordar do fio**: o param autorado fica em `0.0` (que
/// não cunharia canal nenhum) e só o fio diz `0,25`. Um gate cujo override já valesse `0,25`
/// passaria com a membrana antiga, porque a escada errada acertaria por acidente.
#[test]
fn a_driven_time_offset_mints_the_key_the_node_will_ask_for() {
    let mut m = state_with_driven_shift("Ball", 0.25);
    seed_raw(&mut m, "Ball", 7);
    let before: Vec<String> = m.pump.cook.externals().keys().cloned().collect();
    // O autorado fica no default (`0.0`) — só o FIO diz `0,25`.
    publish_shifted(&mut m, 0.0);

    let key = ph2d_nodegraph::external::appearance_of("Ball", 0.25);
    let e = m
        .pump
        .cook
        .externals()
        .get(&key)
        .expect("a chave do valor CONDUZIDO tem de existir, senao o objeto some");
    assert_eq!(
        e.value.get("texture_id"),
        Some(&Column::Scalar(vec![7.0])),
        "e' a aparencia crua, copiada para o canal deslocado"
    );
    // ⚠️ **E cunhou EXATAMENTE UMA chave nova, que é a do fio.** A metade errada não seria
    // uma chave errada — seria chave NENHUMA (com o autorado em `0.0` a membrana antiga não
    // pedia nada), e uma contagem apanha isso onde um `contains_key` do valor autorado não
    // apanharia: `appearance_of(name, 0.0)` **é o nome cru**, que já estava lá.
    let novas: Vec<&String> = m
        .pump
        .cook
        .externals()
        .keys()
        .filter(|k| !before.contains(k))
        .collect();
    assert_eq!(
        novas,
        vec![&key],
        "uma chave nova, e e' a do valor conduzido"
    );
}
