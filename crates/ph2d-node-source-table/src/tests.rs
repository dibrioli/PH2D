//! Gates do nó. ⚠️ Eles cozem por um `Cook` com o canal externo POSTO à mão — é a única
//! maneira de este crate provar alguma coisa sem depender do leitor (que ele não pode ter).

use super::*;
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::Graph;
use ph2d_nodegraph::value::CookValue;

const PATH: &str = "/tmp/x.csv";

fn cooked(external: Option<Stream>, spacing: f32) -> Stream {
    let mut reg = ph2d_node_registry::NodeRegistry::new();
    register(&mut reg).expect("regista");
    let mut g = Graph::new();
    let n = g.add_node("source.table");
    g.set_text_param(n, FILE_KEY, PATH);
    g.set_param(n, param::SPACING, spacing);
    let mut cook = Cook::new();
    if let Some(s) = external {
        cook.set_external(ph2d_node_registry::table_external_key(PATH), s);
    }
    match &cook.cook(&g, &reg, n, 0.0).expect("coze")[0] {
        CookValue::Instances(s) => s.clone(),
        other => panic!("esperava instancias, veio {other:?}"),
    }
}

fn table(rows: usize) -> Stream {
    Stream::new(rows).with(
        "vendas",
        Column::Scalar((0..rows).map(|i| i as f32 * 2.0).collect::<Vec<_>>()),
    )
}

/// ⭐⭐⭐ **Sem ficheiro, o nó desenha NADA** — e não um ponto na origem.
///
/// ⚠️ Um elemento solitário lê-se como *"a tabela carregou e tem uma linha"*, que é a mentira
/// mais cara que este nó pode contar: o artista procuraria o defeito no ficheiro dele.
#[test]
fn with_no_file_the_node_draws_nothing_rather_than_one_lonely_dot() {
    assert_eq!(cooked(None, 0.25).count(), 0);
    // ⚠️ **O CONTROLE**: com tabela, ele desenha. Sem isto o gate passaria com um nó morto.
    assert_eq!(cooked(Some(table(5)), 0.25).count(), 5);
}

#[test]
fn every_column_of_the_file_arrives_by_its_own_name() {
    let s = cooked(Some(table(4)), 0.25);
    match s.get("vendas") {
        Some(Column::Scalar(v)) => assert_eq!(v, &[0.0, 2.0, 4.0, 6.0]),
        other => panic!("a coluna do ficheiro tem de chegar pelo NOME: {other:?}"),
    }
    // As colunas de escrituração que todo source emite.
    assert!(s.get("P").is_some() && s.get("Index").is_some() && s.get("Count").is_some());
}

/// A fileira é CENTRADA e o `Spacing` manda — é o que torna a tabela visível ao carregar.
#[test]
fn the_rows_land_in_a_centred_row_and_the_spacing_drives_it() {
    let xs = |sp: f32| match cooked(Some(table(4)), sp).get("P") {
        Some(Column::Vec2(v)) => v.iter().map(|p| p[0]).collect::<Vec<_>>(),
        _ => panic!("P"),
    };
    let a = xs(0.5);
    // Centrada: a soma das posições é zero.
    assert!(a.iter().sum::<f32>().abs() < 1e-5, "{a:?}");
    // Espaçamento honrado.
    assert!((a[1] - a[0] - 0.5).abs() < 1e-5, "{a:?}");
    // ⚠️ **O CONTROLE**: o slider tem de MANDAR — sem ele, um `Spacing` ignorado passaria.
    let b = xs(1.0);
    assert!((b[1] - b[0] - 1.0).abs() < 1e-5, "{b:?}");
}

/// ⭐ **As colunas de escrituração trazem os VALORES certos, e não só existem.**
///
/// ⛔ A 1.ª redacção do gate vizinho afirmava só `.is_some()` sobre `P`/`Index`/`Count`, e uma
/// auditoria mediu o buraco: emitir `Count` a zeros **e** `Index` a zeros sobrevivia à suíte
/// inteira (crates + shell, 4 082 testes). Um `Count` a zeros parte todo `value.attribute` e
/// toda normalização a jusante, mudo.
#[test]
fn the_bookkeeping_columns_carry_their_real_values() {
    let s = cooked(Some(table(4)), 0.25);
    match s.get("Index") {
        Some(Column::Scalar(v)) => assert_eq!(v, &[0.0, 1.0, 2.0, 3.0]),
        other => panic!("Index: {other:?}"),
    }
    match s.get("Count") {
        Some(Column::Scalar(v)) => assert_eq!(v, &[4.0; 4]),
        other => panic!("Count: {other:?}"),
    }
}
