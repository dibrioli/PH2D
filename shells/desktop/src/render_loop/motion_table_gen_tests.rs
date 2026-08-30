//! Gates do lado da SHELL — a codificação, o renomear, e a chave que os dois nós têm de
//! escrever da mesma maneira.

use super::*;

/// ⭐⭐⭐ **OS DOIS NÓS NOMEIAM O FICHEIRO DA MESMA MANEIRA.**
///
/// ⛔⛔ Uma auditoria adversarial (2026-08-30) renomeou o `FILE_KEY` do `value.table` para
/// `"path"`, acompanhou a string no censo de alcance — o que um rename a sério faz — e **a
/// suíte inteira do shell ficou verde**, com o nó a não receber dado nenhum. As duas
/// constantes valiam `"file"` **por coincidência**, e o mascarador estava medido: a cena `=109`
/// tem um `source.table` a apontar o mesmo ficheiro, então o canal externo existia na mesma.
/// Um `value.table` SOZINHO ficaria vazio em silêncio — que é, palavra por palavra, *"o modo de
/// falha mais caro que este canal tem"*.
#[test]
fn the_two_table_nodes_name_the_file_the_same_way() {
    assert_eq!(
        ph2d_node_source_table::FILE_KEY,
        ph2d_node_value_table::FILE_KEY,
        "se as duas divergirem, o `publish` tem de aprender a ler cada uma — e hoje ele ja' \
         le' a de CADA no', entao isto e' a rede, nao a lei"
    );
}

/// ⭐⭐ **Um nome do MOTOR é RENOMEADO, nunca deixado passar nem deitado fora.**
///
/// ⛔ Deixar passar SEQUESTRA: medido, um CSV com uma coluna `falloff` passaria a multiplicar a
/// escala de tudo a jusante (o `motion.scale` lê-a, ausente ⇒ `1.0`), e um com `id` entraria no
/// motor **como identidade**. Deitar fora PERDE: `size` é o nome mais plausível para a coluna de
/// um gráfico de barras.
#[test]
fn a_column_named_like_the_engines_is_renamed_and_said() {
    let dir = std::env::temp_dir().join("ph2d_table_gate_reserved.csv");
    std::fs::write(&dir, "id,falloff,size,vendas\n1,0.5,3,10\n2,0.6,4,20\n").expect("escreve");
    let loaded = read_and_shape(&dir.to_string_lossy());
    let mut names: Vec<&String> = loaded.stream.columns().map(|(n, _)| n).collect();
    names.sort();
    assert_eq!(
        names,
        ["falloff_csv", "id_csv", "size_csv", "vendas"],
        "os tres nomes do motor sao RENOMEADOS e o inocente passa intacto"
    );
    let r = loaded.report.expect("renomear tem de ser DITO");
    for n in ["id", "falloff", "size"] {
        assert!(r.contains(n), "o relatorio tem de nomear `{n}`: {r}");
    }
    // ⚠️ **O CONTROLE**: o dado NÃO se perde — é o que separa renomear de saltar.
    match loaded.stream.get("size_csv") {
        Some(Column::Scalar(v)) => assert_eq!(v, &[3.0, 4.0]),
        other => panic!("o dado tem de sobreviver ao rename: {other:?}"),
    }
    let _ = std::fs::remove_file(&dir);
}

/// ⭐⭐ **Um ficheiro que não é UTF-8 é LIDO, e não devolvido como vazio.**
///
/// ⛔ Medido antes da cura: um CSV cp1252 do Excel (o default do *«CSV (separado por
/// vírgulas)»* no Windows) devolvia uma tabela vazia **e memoizada**, indistinguível de *"não há
/// ficheiro"*. A referência que este módulo cita — o *Table DAT* — expõe `Default Read
/// Encoding` **com o CP1252 na lista**.
#[test]
fn a_cp1252_file_is_read_instead_of_coming_back_empty() {
    let p = std::env::temp_dir().join("ph2d_table_gate_cp1252.csv");
    // `produto,preço\nAção,10\nMaçã,20\n` em cp1252.
    let mut bytes = b"produto,pre".to_vec();
    bytes.push(0xE7); // ç
    bytes.extend_from_slice(b"o\nA");
    bytes.push(0xE7);
    bytes.extend_from_slice(b"ao,10\nMa");
    bytes.push(0xE7);
    bytes.extend_from_slice(b"a,20\n");
    std::fs::write(&p, &bytes).expect("escreve");
    let loaded = read_and_shape(&p.to_string_lossy());
    match loaded.stream.get("preço") {
        Some(Column::Scalar(v)) => assert_eq!(v, &[10.0, 20.0]),
        other => panic!("a coluna acentuada tem de chegar: {other:?}"),
    }
    assert!(
        loaded.report.as_deref().unwrap_or("").contains("UTF-8"),
        "a codificacao tem de ser DITA: {:?}",
        loaded.report
    );
    let _ = std::fs::remove_file(&p);
}
