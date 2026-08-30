//! Gates do leitor. A fixtura de cada um **contém o fenómeno** que ele afirma.

use super::*;

fn skip_name(t: &Table, i: usize) -> &str {
    match &t.notes[i] {
        Note::Skipped { name, .. } => name,
        other => panic!("esperava Skipped, veio {other:?}"),
    }
}
fn skip_reason(t: &Table, i: usize) -> &SkipReason {
    match &t.notes[i] {
        Note::Skipped { reason, .. } => reason,
        other => panic!("esperava Skipped, veio {other:?}"),
    }
}

#[test]
fn a_header_row_names_the_columns_and_the_body_is_the_data() {
    let t = parse("nome,altura,peso\na,1.5,70\nb,1.8,82\n");
    assert_eq!(t.rows, 2);
    assert!(t.had_header);
    // `nome` é texto ⇒ saltada e NOMEADA.
    assert_eq!(
        t.columns
            .iter()
            .map(|c| c.name.as_str())
            .collect::<Vec<_>>(),
        ["altura", "peso"]
    );
    assert_eq!(t.column("altura").unwrap().values, [1.5, 1.8]);
    assert_eq!(t.notes.len(), 1);
    assert_eq!(skip_name(&t, 0), "nome");
}

#[test]
fn a_table_with_no_header_gets_counted_names_from_one() {
    let t = parse("1,2,3\n4,5,6\n");
    assert!(!t.had_header, "tudo numérico ⇒ não há cabeçalho");
    assert_eq!(t.rows, 2);
    assert_eq!(
        t.columns
            .iter()
            .map(|c| c.name.as_str())
            .collect::<Vec<_>>(),
        ["col1", "col2", "col3"]
    );
}

/// ⛔ **A divergência DELIBERADA do Blender**, e a fixtura contém exactamente o caso que a
/// regra dele estraga: a 1.ª célula é um número e o resto é texto.
#[test]
fn a_column_whose_first_value_is_a_number_but_the_rest_is_not_is_skipped_not_zeroed() {
    let t = parse("ano_nota\n1990\nsem registo\n1994\n");
    assert!(
        t.columns.is_empty(),
        "com a regra do Blender (tipo do 1.º valor) isto entrava como zeros: {:?}",
        t.columns
    );
    assert_eq!(
        t.notes,
        [Note::Skipped {
            name: "ano_nota".into(),
            reason: SkipReason::NotNumeric {
                first_bad: "sem registo".into()
            }
        }]
    );
    // ⭐ E a recusa CHEGA a uma frase — sem isto a divergência paga o preço e não entrega nada.
    assert!(t.report().unwrap().contains("ano_nota"), "{:?}", t.report());
}

#[test]
fn an_empty_cell_is_zero_the_way_houdini_reads_an_out_of_bounds_column() {
    let t = parse("a,b\n1,\n,2\n");
    assert_eq!(t.column("a").unwrap().values, [1.0, 0.0]);
    assert_eq!(t.column("b").unwrap().values, [0.0, 2.0]);
}

#[test]
fn a_repeated_header_keeps_the_first_and_names_the_second() {
    let t = parse("x,x\n1,9\n");
    assert_eq!(t.columns.len(), 1);
    assert_eq!(t.column("x").unwrap().values, [1.0]);
    assert_eq!(skip_reason(&t, 0), &SkipReason::DuplicateName);
}

/// ⭐ O separador é DETETADO — sem isto, metade da Europa vê uma coluna só.
#[test]
fn the_delimiter_is_detected_and_a_decimal_comma_survives_a_semicolon_file() {
    let t = parse("a;b\n1,5;2\n");
    assert_eq!(t.delimiter, ';');
    assert_eq!(t.column("a").unwrap().values, [1.5], "vírgula decimal");
    // ⚠️ **O CONTROLE**: num ficheiro separado por VÍRGULA, `1,5` são duas células — e tem de
    // continuar a ser, senão a deteção estaria a adivinhar.
    let c = parse("a,b\n1,5\n");
    assert_eq!(c.delimiter, ',');
    assert_eq!(c.column("a").unwrap().values, [1.0]);
    assert_eq!(c.column("b").unwrap().values, [5.0]);
}

#[test]
fn quotes_protect_the_delimiter_and_a_doubled_quote_is_one() {
    let t = parse("nome,v\n\"a,b\",1\n\"diz \"\"oi\"\"\",2\n");
    assert_eq!(t.rows, 2);
    assert_eq!(t.column("v").unwrap().values, [1.0, 2.0]);
    assert_eq!(skip_name(&t, 0), "nome");
}

#[test]
fn a_percent_sign_is_eaten_and_the_scaling_is_left_to_whoever_draws() {
    let t = parse("uso\n12%\n50%\n");
    assert_eq!(t.column("uso").unwrap().values, [12.0, 50.0]);
}

/// ⚠️ Nunca entra em pânico e nunca falha: o que não é uma tabela devolve zero colunas.
#[test]
fn nothing_that_is_not_a_table_makes_it_panic() {
    for junk in [
        "",
        "\n\n\n",
        "\u{0}\u{1}",
        "\"",
        "\"\"\"\"\"",
        ",,,,,",
        "a\n\n\nb",
        &"x,".repeat(5000),
    ] {
        let t = parse(junk);
        assert!(t.rows < 10_000);
    }
    // NaN e infinito não entram como números: viram «não numérico», nunca um valor venenoso.
    let t = parse("v\nNaN\n");
    assert!(t.columns.is_empty(), "{:?}", t.columns);
    let t = parse("v\ninf\n");
    assert!(t.columns.is_empty(), "{:?}", t.columns);
}

/// ⛔⛔ **O P0 de 2026-08-30: um MILHAR entre aspas dividia por mil, em silêncio.**
///
/// O `split_row` devolve a célula com a vírgula intacta quando ela veio entre ASPAS — que é
/// como uma folha de cálculo escreve `1 200`. A regra da vírgula decimal apanhava-a e o valor
/// saía `1,2`, sem uma linha no relatório. É a exportação de Excel mais comum que existe.
#[test]
fn a_quoted_thousands_separator_is_not_a_decimal_comma() {
    let t = parse("item,valor\na,\"1,200\"\nb,\"12,500\"\n");
    assert_eq!(
        t.column("valor").map(|c| c.values.clone()),
        None,
        "num ficheiro de VIRGULAS, `1,200` nao e' um numero — tem de ser recusado e DITO, \
         nunca lido como 1,2: {:?}",
        t.column("valor")
    );
    assert!(t.report().unwrap().contains("valor"));
    // ⚠️ **O CONTROLE**: num ficheiro de `;` a vírgula decimal CONTINUA a valer.
    let c = parse("item;valor\na;1,5\n");
    assert_eq!(c.column("valor").unwrap().values, [1.5]);
}

/// ⛔ O BOM invisível envenenava o nome da 1.ª coluna e, sem cabeçalho, COMIA a 1.ª linha.
#[test]
fn a_byte_order_mark_does_not_poison_the_first_column_nor_eat_a_row() {
    let t = parse("\u{feff}tempo,valor\n0,10\n1,20\n");
    assert_eq!(t.column("tempo").unwrap().values, [0.0, 1.0]);
    assert!(t.had_header && t.rows == 2);
    // Sem cabeçalho: a 1.ª linha é DADO e tem de sobreviver.
    let d = parse("\u{feff}1,2\n3,4\n");
    assert_eq!(d.rows, 2, "a linha `1,2` evaporava: {:?}", d.columns);
}

/// ⭐ `hh:mm:ss` vira segundos — sem isto nenhuma fonte real preenche a caixa `Time Column`.
#[test]
fn a_clock_column_becomes_seconds() {
    let t = parse("t,v\n00:00:30,1\n00:01:30,2\n01:00:00,3\n");
    assert_eq!(t.column("t").unwrap().values, [30.0, 90.0, 3600.0]);
    // `mm:ss` também, e ⚠️ o CONTROLE: uma data NÃO é um número (e é dita).
    assert_eq!(parse("t\n2:05\n").column("t").unwrap().values, [125.0]);
    let d = parse("data\n2026-08-30\n");
    assert!(d.columns.is_empty() && d.report().is_some());
}

/// ⛔ Uma aspa solta transformava um valor real num `0` sem nada o dizer.
#[test]
fn a_ragged_row_is_counted_and_said() {
    let t = parse("t,v\n0,10\n\"x,20\n2,30\n");
    let ragged = t.notes.iter().any(|n| matches!(n, Note::RaggedRows { .. }));
    assert!(ragged, "a linha partida tem de ser DITA: {:?}", t.notes);
    assert!(t.report().unwrap().contains("celulas do cabecalho"));
}

/// ⛔ Uma célula vazia é inofensiva numa coluna de dados e VENENO na do tempo — vai contada.
#[test]
fn empty_cells_are_counted_per_column() {
    let t = parse("a,b\n1,\n,2\n");
    let n: Vec<_> = t
        .notes
        .iter()
        .filter_map(|n| match n {
            Note::EmptyCells { name, count } => Some((name.as_str(), *count)),
            _ => None,
        })
        .collect();
    assert_eq!(n, [("a", 1), ("b", 1)], "{:?}", t.notes);
}

/// ⚠️ **A FIXTURA QUE FALTAVA**: sem aspas no CABEÇALHO, a metade «fora de aspas» da deteção
/// de separador nunca era medida — e apagá-la sobrevivia aos dez gates.
#[test]
fn a_quoted_header_cell_does_not_confuse_the_delimiter_detection() {
    let t = parse("\"a,b\";c\n1;2\n");
    assert_eq!(t.delimiter, ';', "as virgulas estao DENTRO de aspas");
    // ⚠️ **DUAS colunas, e a 1.ª chama-se literalmente `a,b`** — as aspas eram do NOME, não
    // uma separação. Com a deteção partida (a olhar vírgulas dentro de aspas) o ficheiro seria
    // lido como três colunas e o dado da 2.ª desapareceria.
    assert_eq!(
        t.columns
            .iter()
            .map(|c| c.name.as_str())
            .collect::<Vec<_>>(),
        ["a,b", "c"],
        "{:?}",
        t.columns
    );
    assert_eq!(t.column("c").unwrap().values, [2.0]);
}

/// ⚠️ **A OUTRA FIXTURA QUE FALTAVA**: todas as tabelas eram rectangulares, então a política
/// «uma linha mais larga que o cabeçalho ganha colunas» não era medida — e fazer a largura vir
/// só do cabeçalho sobrevivia.
#[test]
fn a_data_row_wider_than_the_header_still_yields_its_column() {
    let t = parse("a,b\n1,2\n3,4,999\n");
    assert_eq!(t.columns.len(), 3, "{:?}", t.columns);
    assert_eq!(t.column("col3").unwrap().values, [0.0, 999.0]);
    // ⚠️ E ela é DITA: uma coluna que aparece do nada não pode aparecer calada.
    assert!(t.report().unwrap().contains("celulas do cabecalho"));
}
