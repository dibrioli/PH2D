//! ⭐ **Cada corrida de um oráculo é um gate** (§0.9): as fixtures foram gravadas por scripts sobre
//! entradas NOSSAS, duas corridas iguais byte a byte (`docs/MiroClone/ferramentas/facilitacao_oracle/`).

use ph2d_board_words::{Lang, segment, stem};

/// ⭐ A raiz Snowball 3.1.1 vendorizada = a `libstemmer` 3.1.1 instalada, palavra a palavra: 7 515
/// palavras (cada 10.ª do vocabulário oficial do Snowball, inglês e português, mais as das nossas
/// entradas e as `inter*` onde o `waken_snowball` divergia). ⛔ O `rust-stemmers` dava 57 inglesas
/// diferentes e o `waken_snowball` 17 (o estudo §1.3).
#[test]
fn stems_match_libstemmer() {
    let tsv = include_str!("../fixtures/raizes_libstemmer_3_1_1.tsv");
    let (mut n, mut bad) = (0, Vec::new());
    for line in tsv.lines().filter(|l| !l.starts_with('#')) {
        let mut f = line.split('\t');
        let (lang, word, want) = (f.next().unwrap(), f.next().unwrap(), f.next().unwrap());
        let lang = match lang {
            "english" => Lang::English,
            "portuguese" => Lang::Portuguese,
            other => panic!("língua {other}"),
        };
        n += 1;
        let got = stem(lang, word);
        if got != want {
            bad.push(format!("{word}: {got} ≠ {want}"));
        }
    }
    assert!(n > 7_000, "li só {n} palavras — a fixture partiu-se");
    assert!(
        bad.is_empty(),
        "{} raízes diferentes da libstemmer:\n{}",
        bad.len(),
        bad.join("\n")
    );
}

/// ⭐ A segmentação = o `Intl.Segmenter` do Node 24.19 (ICU 78.3, Unicode 17) sobre as 20 notas das
/// entradas: o hífen parte (`retro-board`), o ponto entre algarismos não (`v2.0`), o apóstrofo (recto
/// ou curvo) fica na palavra (`team’s`), a ligadura também (`ﬁle`).
#[test]
fn segments_match_icu() {
    let json: serde_json::Value =
        serde_json::from_str(include_str!("../fixtures/segmentos_icu.json")).unwrap();
    let notes = json["notas"].as_array().unwrap();
    assert_eq!(notes.len(), 20, "as 20 notas das entradas");
    for n in notes {
        let text = n["text"].as_str().unwrap();
        let words: Vec<&str> = n["words"]
            .as_array()
            .unwrap()
            .iter()
            .map(|w| w.as_str().unwrap())
            .collect();
        assert_eq!(segment(text), words, "a nota {text:?}");
    }
}
