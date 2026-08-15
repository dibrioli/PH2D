//! Gates da LEI da tabela — a gramática, o ciclo e o layout que o device lê.
//!
//! O ORÁCULO do caminho legado é o `pattern_value` do `lib.rs`, que **não é
//! tocado por esta wave** — então a metade *"sem tabela nada muda"* mora no
//! irmão `lib_tests.rs`, onde o cook a exercita de ponta a ponta.

use super::*;

/// **A gramática:** espaço e vírgula separam, e os dois lêem igual.
#[test]
fn the_grammar_takes_spaces_and_commas() {
    assert_eq!(parse("0.1 0.5 0.9"), vec![0.1, 0.5, 0.9]);
    assert_eq!(parse("0.1,0.5,0.9"), vec![0.1, 0.5, 0.9]);
    assert_eq!(parse("0.1, 0.5,  0.9"), vec![0.1, 0.5, 0.9]);
    // Separador sobrando não é token: uma vírgula final é digitação normal.
    assert_eq!(parse("0.1, 0.5,"), vec![0.1, 0.5]);
    assert_eq!(parse("  1  \n 2 \t 3 "), vec![1.0, 2.0, 3.0]);
}

/// **Nada autorado é a LISTA VAZIA**, e é ela que faz o nó cair nos oito slots.
#[test]
fn nothing_authored_is_the_empty_list() {
    assert!(parse("").is_empty());
    assert!(parse("   ").is_empty());
    assert!(parse(",, ;").is_empty());
}

/// **Um token ruim invalida a tabela INTEIRA** — nunca é pulado.
///
/// ⚠️ Pular o token daria um padrão com um passo A MENOS, em silêncio; recusar
/// devolve o nó ao caminho legado, que o painel mostra (os nove controles
/// legados voltam a ser pintados).
#[test]
fn one_bad_token_rejects_the_whole_table() {
    assert!(parse("0.1 oops 0.9").is_empty());
    assert!(parse("0.1 0.5 0.9x").is_empty());
    // E o CONTROLE: sem o token ruim a mesma string é aceita.
    assert_eq!(parse("0.1 0.9"), vec![0.1, 0.9]);
}

/// **Não-finito é malformado**, embora o `parse` do Rust o aceite.
///
/// Um NaN aqui atravessaria toda a cadeia a jusante; o gate existe porque
/// `"nan".parse::<f32>()` é `Ok`, e uma leitura ingênua o deixaria entrar.
#[test]
fn non_finite_is_malformed_even_though_rust_parses_it() {
    assert!("nan".parse::<f32>().is_ok(), "a premissa deste gate");
    assert!("inf".parse::<f32>().is_ok(), "a premissa deste gate");
    assert!(parse("0.1 nan 0.9").is_empty());
    assert!(parse("inf").is_empty());
    assert!(parse("-inf 2").is_empty());
}

/// **Acima do teto a lista é TRUNCADA, não recusada** — os primeiros
/// `MAX_ENTRIES` continuam a funcionar.
#[test]
fn past_the_cap_the_list_is_truncated_not_refused() {
    let long: String = (0..MAX_ENTRIES + 50)
        .map(|k| format!("{k} "))
        .collect::<String>();
    let got = parse(&long);
    assert_eq!(got.len(), MAX_ENTRIES, "o teto é o comprimento");
    assert_eq!(got[0], 0.0, "e o começo é o que o artista escreveu");
    #[expect(clippy::cast_precision_loss, reason = "MAX_ENTRIES cabe num f32")]
    let last = (MAX_ENTRIES - 1) as f32;
    assert_eq!(got[MAX_ENTRIES - 1], last);
}

/// **O ciclo é por índice**, o mesmo que os oito slots sempre fizeram.
#[test]
fn the_table_cycles_by_index() {
    let t = vec![10.0, 20.0, 30.0];
    let got: Vec<f32> = (0..7).map(|i| value_at(i, &t).unwrap()).collect();
    assert_eq!(got, vec![10.0, 20.0, 30.0, 10.0, 20.0, 30.0, 10.0]);
}

/// **Lista vazia não responde** — é assim que o chamador sabe cair no legado.
#[test]
fn an_empty_table_answers_nothing() {
    assert_eq!(value_at(0, &[]), None);
    assert_eq!(value_at(99, &[]), None);
}

/// **O LAYOUT que o device lê:** o slot 0 é a CONTAGEM, os valores seguem.
///
/// ⚠️ É este gate que casa com o WGSL do `lib.rs` (`lut_vp_table[1u + (i % n)]`);
/// mover o cabeçalho sem mover o kernel deixaria o device a ler o primeiro valor
/// como se fosse um comprimento.
#[test]
fn the_lut_carries_the_count_in_slot_zero() {
    let mut buf = vec![0.0f32; LUT_LEN as usize];
    fill_lut("7 8 9", &mut buf);
    assert_eq!(buf[0], 3.0, "a contagem");
    assert_eq!(&buf[1..4], &[7.0, 8.0, 9.0], "e os valores, em ordem");
    // O device indexa `1 + (i % n)`, então isto é o que ele lê:
    let read = |i: usize| buf[1 + (i % (buf[0] as usize))];
    assert_eq!(
        (0..5).map(read).collect::<Vec<_>>(),
        vec![7.0, 8.0, 9.0, 7.0, 8.0]
    );
}

/// **Nada autorado escreve CONTAGEM ZERO** — o sinal que manda o kernel tomar o
/// ramo legado, o mesmo que a lista vazia dá à CPU.
#[test]
fn an_unauthored_lut_says_zero_and_that_is_the_legacy_signal() {
    for text in ["", "   ", "0.1 oops"] {
        let mut buf = vec![0.0f32; LUT_LEN as usize];
        fill_lut(text, &mut buf);
        assert_eq!(buf[0], 0.0, "texto {text:?}");
    }
}

/// **As duas metades concordam sobre a MESMA string** — a CPU pela lista, o
/// device pelo buffer.
///
/// ⚠️ Este é o gate que impede a gramática de existir em duas versões: ele lê o
/// buffer com a aritmética EXATA do WGSL e a compara com o `value_at`.
#[test]
fn the_cpu_list_and_the_device_buffer_answer_the_same() {
    for text in [
        "0.25 0.5 0.75",
        "1",
        "0.1, 0.2, 0.3, 0.4, 0.5",
        "3 1 4 1 5 9 2 6 5 3 5",
    ] {
        let table = parse(text);
        let mut buf = vec![0.0f32; LUT_LEN as usize];
        fill_lut(text, &mut buf);
        let n = buf[0] as usize;
        assert_eq!(n, table.len(), "contagem, texto {text:?}");
        for i in 0..40usize {
            let cpu = value_at(i, &table).unwrap();
            let gpu = buf[1 + (i % n)];
            assert_eq!(cpu, gpu, "elemento {i}, texto {text:?}");
        }
    }
}

/// **O buffer cabe por construção** — `LUT_LEN` é o cabeçalho mais o teto.
#[test]
fn the_buffer_holds_the_header_plus_the_cap() {
    assert_eq!(LUT_LEN as usize, MAX_ENTRIES + 1);
    let long: String = (0..MAX_ENTRIES * 2).map(|_| "1 ").collect::<String>();
    let mut buf = vec![0.0f32; LUT_LEN as usize];
    fill_lut(&long, &mut buf);
    assert_eq!(buf[0] as usize, MAX_ENTRIES);
    assert_eq!(buf[MAX_ENTRIES], 1.0, "o último slot é escrito");
}

/// **O CUSTO da tabela** — o recurso de que o teto é (`-- --ignored`).
///
/// O buffer é CONSTANTE (`LUT_LEN` é uma const do `LutSpec`), então a linha que
/// decide é *quanto o `fill` cobra*, e ele roda uma vez por encode.
#[test]
#[ignore = "sonda: imprime o custo do fill; roda com -- --ignored"]
fn measure_table_cost() {
    use std::time::Instant;
    println!(
        "buffer: {} floats = {} bytes por no",
        LUT_LEN,
        LUT_LEN as usize * 4
    );
    for n in [8usize, 64, 256, 1024] {
        let text: String = (0..n).map(|k| format!("{}.5 ", k % 10)).collect();
        let mut buf = vec![0.0f32; LUT_LEN as usize];
        // Aquece (a primeira passagem paga o first-touch do buffer).
        fill_lut(&text, &mut buf);
        let reps = 200;
        let t0 = Instant::now();
        for _ in 0..reps {
            fill_lut(&text, &mut buf);
        }
        let per = t0.elapsed().as_secs_f64() / f64::from(reps) * 1e6;
        assert_eq!(buf[0] as usize, n, "a fixture contém o fenômeno");
        println!("  {n:>5} entradas: fill {per:.2} us");
    }
}
