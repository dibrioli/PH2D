//! Gates for `source.object` (doc 86 §2).

use super::*;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::{Cook, OpResolver};
use ph2d_nodegraph::graph::Graph;

struct Ops;
impl OpResolver for Ops {
    fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
        (ty == MANIFEST.id).then_some(&SourceObject as &dyn NodeOp)
    }
}

/// A sprite tile as the membrane would publish it: one instance at the origin
/// carrying the appearance (`texture_id`, `uv_rect`, `size`, `tint`).
fn tile(texture_id: f32) -> Stream {
    Stream::new(1)
        .with("P", Column::Vec2(vec![[0.0, 0.0]]))
        .with("size", Column::Vec2(vec![[2.0, 3.0]]))
        .with("tint", Column::Vec4(vec![[1.0, 0.0, 0.0, 1.0]]))
        .with("uv_rect", Column::Vec4(vec![[0.1, 0.2, 0.3, 0.4]]))
        .with("texture_id", Column::Scalar(vec![texture_id]))
}

/// Publish `stream` under `published`, set the node's `object` to `named`, cook,
/// and hand back the output stream.
fn source(published: &str, named: &str, stream: Stream) -> Stream {
    let mut g = Graph::new();
    let n = g.add_node("source.object");
    g.set_text_param(n, OBJECT_PARAM, named);
    let mut cook = Cook::new();
    cook.set_external(published, stream);
    let out = cook.cook(&g, &Ops, n, 0.0).unwrap();
    out[0].as_stream().clone()
}

#[test]
fn the_source_emits_the_published_object_stream() {
    // The membrane published a sprite tile under `Ball`; the node names `Ball`
    // and emits exactly that appearance — the door the graph gains to say WHAT.
    let out = source("Ball", "Ball", tile(5.0));
    assert_eq!(out.count(), 1);
    let Some(Column::Scalar(ids)) = out.get("texture_id") else {
        panic!("texture_id")
    };
    assert_eq!(ids, &vec![5.0]);
    let Some(Column::Vec2(size)) = out.get("size") else {
        panic!("size")
    };
    assert_eq!(size, &vec![[2.0, 3.0]]);
    let Some(Column::Vec4(uv)) = out.get("uv_rect") else {
        panic!("uv_rect")
    };
    assert_eq!(uv, &vec![[0.1, 0.2, 0.3, 0.4]]);
    let Some(Column::Vec4(tint)) = out.get("tint") else {
        panic!("tint")
    };
    assert_eq!(tint, &vec![[1.0, 0.0, 0.0, 1.0]]);
}

#[test]
fn an_unpicked_source_emits_nothing() {
    // No object named (empty text param) → the empty external → an empty stream.
    // The node emits nothing rather than guessing or failing.
    let out = source("Ball", "", tile(5.0));
    assert_eq!(out.count(), 0);
}

#[test]
fn the_name_is_the_reference_a_mismatch_decouples() {
    // The object is published under `Box`, but the node names `Ball`. Nothing
    // resolves — renaming an object you referred to by name IS decoupling it.
    let out = source("Box", "Ball", tile(5.0));
    assert_eq!(out.count(), 0);
}

/// Publish `stream` under `published`, name `named`, deslocar por `offset`, cozinhar.
fn source_shifted(published: &str, named: &str, offset: f32, stream: Stream) -> Stream {
    let mut g = Graph::new();
    let n = g.add_node("source.object");
    g.set_text_param(n, OBJECT_PARAM, named);
    g.set_param(n, TIME_OFFSET_PARAM, offset);
    let mut cook = Cook::new();
    cook.set_external(published, stream);
    let out = cook.cook(&g, &Ops, n, 0.0).unwrap();
    out[0].as_stream().clone()
}

/// **O offset zero lê o NOME CRU** — a neutralidade da wave inteira, vista de dentro
/// do nó. Todo grafo já salvo tem este param em `0.0` (o default), então esta é a
/// afirmação de que nenhum deles mudou de canal.
///
/// ⚠️ O gate escreve o param EXPLICITAMENTE em vez de o deixar no default: um teste
/// que chega ao estado por omissão inverte de sentido no dia em que o default se
/// move, e segue verde testando o oposto.
#[test]
fn an_unshifted_source_reads_the_raw_name() {
    let s = source_shifted("Ball", "Ball", 0.0, tile(7.0));
    assert_eq!(s.count(), 1, "o offset zero tem de achar o nome cru");
    assert_eq!(
        s.get("texture_id"),
        Some(&Column::Scalar(vec![7.0])),
        "e tem de ser a MESMA tile"
    );
}

/// **Um offset lê um canal DIFERENTE** — e a metade que importa é que ele não cai de
/// volta no nome cru: se caísse, o param seria um controle morto que parece funcionar
/// (a cena desenharia, com o desenho errado, e nada reclamaria).
#[test]
fn a_shifted_source_does_not_read_the_unshifted_channel() {
    let s = source_shifted("Ball", "Ball", 0.25, tile(7.0));
    assert_eq!(
        s.count(),
        0,
        "o nome cru nao pode responder por um pedido deslocado"
    );
}

/// **E ele lê a chave que a porta única minta.** O par com o gate acima: um deles
/// sozinho seria satisfeito por um nó que lê sempre vazio.
#[test]
fn a_shifted_source_reads_the_key_the_one_door_mints() {
    let key = ph2d_nodegraph::external::appearance_of("Ball", 0.25);
    let s = source_shifted(&key, "Ball", 0.25, tile(9.0));
    assert_eq!(
        s.get("texture_id"),
        Some(&Column::Scalar(vec![9.0])),
        "o no tem de ler a chave que o shell publica"
    );
}

/// **Dois offsets diferentes são dois canais diferentes** — a propriedade sem a qual
/// uma cascata de cópias mostraria o mesmo desenho, que é exatamente o defeito que
/// esta wave existe para curar.
#[test]
fn two_offsets_are_two_channels() {
    let a = ph2d_nodegraph::external::appearance_of("Ball", 0.25);
    let s = source_shifted(&a, "Ball", 0.5, tile(9.0));
    assert_eq!(
        s.count(),
        0,
        "o pedido de 0.5 nao pode ser servido pela tile de 0.25"
    );
}
