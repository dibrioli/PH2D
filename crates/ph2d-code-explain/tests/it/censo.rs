//! **O censo do I8**: o corpus do I0c inteiro (`docs/CodeEditor/ferramentas/corpus_erros/`) passa
//! por [`explain`]. Uma mensagem sem explicação, uma classe sem texto ou uma classe que o corpus já
//! não produz é vermelho. Correr o corpus de novo (`corre.sh`) com uma mensagem nova acorda-o.

use ph2d_code_explain::{Explained, Kind, explain};
use ph2d_i18n::tr;
use std::collections::BTreeMap;
use std::path::PathBuf;

fn corpus(file: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/CodeEditor/ferramentas/corpus_erros")
        .join(file)
}

/// Todas as mensagens gravadas (via estática e via do motor).
fn messages() -> Vec<String> {
    let mut out = Vec::new();
    for file in ["estatico.jsonl", "runtime.jsonl"] {
        let text = std::fs::read_to_string(corpus(file)).expect("o corpus do I0c existe");
        for line in text.lines() {
            let row: serde_json::Value = serde_json::from_str(line).expect("jsonl");
            for d in row["diags"].as_array().expect("diags") {
                out.push(d["message"].as_str().expect("message").to_string());
            }
        }
    }
    out
}

/// A classe de cada mensagem do corpus, e as que ficaram sem nenhuma.
fn census() -> (BTreeMap<Kind, Vec<Explained>>, Vec<String>) {
    let mut hit: BTreeMap<Kind, Vec<Explained>> = BTreeMap::new();
    let mut none = Vec::new();
    for m in messages() {
        match explain(&m) {
            Some(e) => hit.entry(e.kind).or_default().push(e),
            None => none.push(m),
        }
    }
    (hit, none)
}

#[test]
fn toda_mensagem_do_corpus_tem_explicacao() {
    let (hit, none) = census();
    let read: usize = hit.values().map(Vec::len).sum::<usize>() + none.len();
    // Piso: 1 661 + 669 sítios mais as linhas de base (17/10). Uma leitura que encolhesse
    // devolveria zero acusados.
    assert!(read >= 2_400, "o censo leu só {read} mensagens do corpus");
    assert!(
        none.is_empty(),
        "{} mensagem(ns) do corpus sem explicação (classe nova ⇒ uma Kind e o texto em \
         ph2d-i18n/src/code_explain.rs):\n  {}",
        none.len(),
        none.iter()
            .map(|m| m.chars().take(160).collect::<String>())
            .collect::<Vec<_>>()
            .join("\n  ")
    );
}

#[test]
fn cada_classe_das_tabelas_tem_explicacao() {
    for file in ["classes_estatico.tsv", "classes_runtime.tsv"] {
        let text = std::fs::read_to_string(corpus(file)).expect("a tabela do I0c existe");
        let mut lines = text.lines();
        let head: Vec<&str> = lines.next().expect("cabeçalho").split('\t').collect();
        let at = |c: &str| head.iter().position(|h| *h == c).expect(c);
        let (class, example) = (at("classe"), at("exemplo"));
        let rows: Vec<Vec<&str>> = lines.map(|l| l.split('\t').collect()).collect();
        assert!(rows.len() >= 40, "{file}: só {} classes", rows.len());
        let mute: Vec<&str> = rows
            .iter()
            .filter(|r| explain(r[example]).is_none())
            .map(|r| r[class])
            .collect();
        assert!(mute.is_empty(), "{file}: classes sem explicação: {mute:#?}");
    }
}

/// Um `{nome}` que ficou por encher (o `{ speed = 2 }` de uma tabela não é marcador).
fn unfilled(s: &str) -> bool {
    s.split('{').skip(1).any(|r| {
        let end = r.find('}').unwrap_or(0);
        end > 0
            && r[..end]
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b == b'_')
    })
}

#[test]
fn toda_classe_tem_titulo_e_porque_sem_marcador_por_encher() {
    let (hit, _) = census();
    let mut samples: Vec<Explained> = hit.values().filter_map(|v| v.first().cloned()).collect();
    samples.push(Explained::unknown_field("ampiltude", None));
    samples.push(Explained::unknown_field("ampiltude", Some("amplitude")));
    for k in Kind::ALL {
        assert_ne!(tr(k.title_key()), k.title_key(), "{k:?} sem título");
        assert_ne!(tr(k.why_key()), k.why_key(), "{k:?} sem porquê");
    }
    for key in ["code.explain.type_table", "code.explain.type_function"] {
        assert_ne!(tr(key), key, "a palavra de um tipo enorme");
    }
    for e in &samples {
        let (title, why) = (e.title(), e.why());
        assert!(
            !unfilled(&title) && !unfilled(&why),
            "{:?}: {title} / {why}",
            e.kind
        );
        assert!(
            title.chars().count() <= 90,
            "{:?}: o título não cabe numa linha ({}): {title}",
            e.kind,
            title.chars().count()
        );
    }
}

#[test]
fn nenhuma_classe_ficou_sem_mensagem_no_corpus() {
    let (hit, _) = census();
    let dead: Vec<&Kind> = Kind::ALL
        .iter()
        .filter(|k| !k.ours() && !hit.contains_key(k))
        .collect();
    assert!(
        dead.is_empty(),
        "classes que o corpus já não produz: {dead:?}"
    );
}

#[test]
fn a_mensagem_do_1002_vira_a_chave_errada_numa_linha() {
    let long = messages()
        .into_iter()
        .find(|m| m.contains("Key 'proeprty'") && m.contains("*TRUNCATED*"))
        .expect("o 1002 de 567 caracteres do I0b está no corpus");
    let e = explain(&long).expect("explicado");
    assert_eq!(e.kind, Kind::UnknownKeyHere);
    assert_eq!(e.arg("key"), Some("proeprty"));
    assert_eq!(e.title(), "There is no \u{201c}proeprty\u{201d} here");
}

#[test]
fn o_igual_do_smoke_2_explica_o_valor_que_falta() {
    let e = explain("SyntaxError: Expected identifier when parsing expression, got '='")
        .expect("explicado");
    assert_eq!(e.kind, Kind::ExpectedExpression);
    assert_eq!(e.title(), "A value is missing before \u{201c}=\u{201d}");
    // O motor dá a mesma mensagem sem o prefixo, e às vezes com a posição de outro chunk.
    let motor = explain("bob.luau:3: Expected identifier when parsing expression, got '='");
    assert_eq!(motor, Some(e));
}
