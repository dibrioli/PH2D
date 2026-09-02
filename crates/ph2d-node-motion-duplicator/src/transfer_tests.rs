//! Gates da **TRANSFERÊNCIA DE ATRIBUTO** (doc 89 folha 08).
//!
//! A lei tem quatro metades e as quatro são testadas aqui: **uma coluna que só o ponto tem
//! chega em TODO modo** (desde o report do Enio de 2026-09-01 — o modo inerte deixou de a
//! deitar fora, porque um modo de CONFLITO não decide sobre o que ninguém disputa), o modo
//! inerte deixa a forma vencer a coluna disputada, os dois lados combinam-se segundo o modo,
//! e o `size` fica fora porque tem porta própria.

use super::Transfer;
use crate::{Pick, duplicate};
use ph2d_nodegraph::attr::{Column, Stream};

/// Uma forma com uma coluna `tint` e uma `w`.
fn shape_with(tint: &[f32], w: &[f32]) -> Stream {
    let mut s = Stream::new(tint.len());
    s.set("P", Column::Vec2(vec![[0.0, 0.0]; tint.len()]));
    s.set("tint", Column::Scalar(tint.to_vec()));
    s.set("w", Column::Scalar(w.to_vec()));
    s
}

/// Pontos com uma coluna `tint` (que a forma também tem) e uma `only` (que só eles têm).
fn points_with(tint: &[f32], only: &[f32]) -> Stream {
    let mut s = Stream::new(tint.len());
    s.set("P", Column::Vec2(vec![[0.0, 0.0]; tint.len()]));
    s.set("tint", Column::Scalar(tint.to_vec()));
    s.set("only", Column::Scalar(only.to_vec()));
    s
}

fn scalar(s: &Stream, name: &str) -> Option<Vec<f32>> {
    match s.get(name) {
        Some(Column::Scalar(v)) => Some(v.clone()),
        _ => None,
    }
}

fn run(mode: Transfer) -> Stream {
    let shape = shape_with(&[0.25], &[7.0]);
    let points = points_with(&[0.5, 0.75], &[10.0, 20.0]);
    duplicate(&shape, &points, 2, Pick::Off, 0, 0.0, mode)
}

/// ⭐ **O CONTROLE do modo de sempre: a forma vence a coluna DISPUTADA.** É esta a pergunta
/// que o [`Transfer`] responde, e ela não mudou.
///
/// ⛔⛔ **A 1.ª redacção deste teste afirmava também que *«a coluna só-do-ponto NÃO chega no
/// modo de sempre»*, e isso era o DEFEITO escrito como contrato** — report do Enio,
/// 2026-09-01: *«o nó entrou em points corretamente mas a simulação morreu»*. Um modo que
/// resolve CONFLITO não tem opinião sobre uma coluna que ninguém disputa, e deitá-la fora
/// custava as **sete** colunas que um emissor entrega (`id`, `vel`, `age`, `life`, …) —
/// sem `vel` não há o que integrar e sem `id` o integrador não reconhece a partícula do
/// tique anterior.
///
/// ⚠️ O doc antigo dizia que a perda *«mantém toda arte já autorada de pé»*. Mantinha — de
/// pé e **muda**: a arte que autorava uma coluna nos pontos nunca a via.
#[test]
fn the_default_mode_still_lets_the_shape_win_the_contested_column() {
    let out = run(Transfer::ShapeWins);
    assert_eq!(
        scalar(&out, "tint"),
        Some(vec![0.25, 0.25]),
        "disputada: a forma vence, e e' isso que o modo quer dizer"
    );
    assert_eq!(
        scalar(&out, "w"),
        Some(vec![7.0, 7.0]),
        "a da forma espalha"
    );
}

/// ⚠️ **UMA COLUNA QUE SÓ O PONTO TEM CHEGA EM **TODOS** OS MODOS** — inclusive no de
/// sempre, desde o report do Enio de 2026-09-01. Não há com que a combinar, logo não há
/// conflito para um modo de conflito resolver.
#[test]
fn a_column_only_the_point_has_reaches_the_output_in_every_mode() {
    for mode in [
        Transfer::ShapeWins,
        Transfer::PointWins,
        Transfer::Add,
        Transfer::Multiply,
    ] {
        let out = run(mode);
        assert_eq!(
            scalar(&out, "only"),
            Some(vec![10.0, 20.0]),
            "{mode:?}: a coluna so'-do-ponto chega inteira (nao ha' com que a combinar)"
        );
    }
}

/// Os dois lados têm a coluna: o modo decide, e cada modo dá um número diferente — que é o
/// que separa um controlo real de três rótulos com a mesma resposta.
#[test]
fn when_both_sides_author_the_column_the_mode_decides() {
    assert_eq!(
        scalar(&run(Transfer::PointWins), "tint"),
        Some(vec![0.5, 0.75])
    );
    assert_eq!(
        scalar(&run(Transfer::Add), "tint"),
        Some(vec![0.75, 1.0]),
        "0,25 + 0,5 e 0,25 + 0,75"
    );
    assert_eq!(
        scalar(&run(Transfer::Multiply), "tint"),
        Some(vec![0.125, 0.1875]),
        "0,25 x 0,5 e 0,25 x 0,75"
    );
}

/// ⚠️ **O `size` tem porta própria** (`point_scale`) e o `transfer` salta-o pelo nome. Sem
/// isto, ligar a transferência escreveria a escala por uma segunda lei — e as duas
/// divergiriam no dia em que uma delas mudasse.
#[test]
fn the_transfer_never_writes_the_size_that_the_point_scale_owns() {
    let mut shape = Stream::new(1);
    shape.set("P", Column::Vec2(vec![[0.0, 0.0]]));
    shape.set("size", Column::Vec2(vec![[2.0, 2.0]]));
    let mut points = Stream::new(2);
    points.set("P", Column::Vec2(vec![[0.0, 0.0]; 2]));
    points.set("size", Column::Vec2(vec![[3.0, 3.0], [4.0, 4.0]]));
    // `point_scale = 0` ⇒ a porta do `size` está fechada; nenhum modo de transferência a abre.
    for mode in [
        Transfer::ShapeWins,
        Transfer::PointWins,
        Transfer::Add,
        Transfer::Multiply,
    ] {
        let out = duplicate(&shape, &points, 2, Pick::Off, 0, 0.0, mode);
        let Some(Column::Vec2(v)) = out.get("size") else {
            panic!("a forma autorou `size`, entao ela existe na saida")
        };
        assert_eq!(
            v.as_slice(),
            [[2.0, 2.0], [2.0, 2.0]],
            "{mode:?}: com `point_scale = 0` a escala e' a da FORMA, sempre"
        );
    }
}

/// ⚠️ Variantes discordantes: somar um `Scalar` a um `Vec2` não tem resposta, e trocar o
/// tipo da coluna a jusante em silêncio é pior que a perda que este módulo curou. A forma
/// vence, e é uma decisão declarada — não um acidente do `match`.
#[test]
fn a_variant_mismatch_keeps_the_shapes_column() {
    let mut shape = Stream::new(1);
    shape.set("P", Column::Vec2(vec![[0.0, 0.0]]));
    shape.set("k", Column::Scalar(vec![5.0]));
    let mut points = Stream::new(2);
    points.set("P", Column::Vec2(vec![[0.0, 0.0]; 2]));
    points.set("k", Column::Vec2(vec![[1.0, 1.0], [2.0, 2.0]]));
    let out = duplicate(&shape, &points, 2, Pick::Off, 0, 0.0, Transfer::PointWins);
    assert_eq!(
        scalar(&out, "k"),
        Some(vec![5.0, 5.0]),
        "variantes discordantes ⇒ a forma fica, e o tipo da coluna nao muda"
    );
}

/// O param fora de alcance cai no modo de sempre — a mesma lei do `Pick::of`.
#[test]
fn an_out_of_range_param_falls_back_to_the_world_that_shipped() {
    for v in [-3.0f32, 4.0, 99.0] {
        assert_eq!(Transfer::of(v), Transfer::ShapeWins, "param {v}");
    }
    assert!(Transfer::ShapeWins.is_inert());
    for v in [1.0f32, 2.0, 3.0] {
        assert!(!Transfer::of(v).is_inert(), "param {v} e' um modo vivo");
    }
}
