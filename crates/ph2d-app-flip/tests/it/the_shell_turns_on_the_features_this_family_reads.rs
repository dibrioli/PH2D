//! ⛔⛔ **ARCH-GATE — a shell ACENDE as features que esta família lê** (HOWTO §2.4).
//!
//! # O defeito que ele existe para impedir
//!
//! **As features NÃO viajam com o código.** Quando um ficheiro sai da `shells/desktop` para
//! uma crate que não declara a feature que o governa, o `#[cfg(feature = "x")]` dele passa a
//! ser falso **por construção** — e a crate compila **VERDE** com aquele braço desligado.
//! O piloto da W2 (`field3d`, 11/09) perdeu assim o carregador do matcap: a cena sairia
//! cinzenta, e o único aviso foi um `unexpected cfg condition value` no meio de 93 erros.
//!
//! Esta família tem **dois** `cfg` de feature no código que se mudou
//! ([`ph2d_app_flip::strip_drag`] e [`ph2d_app_flip::bridge`]), e os dois governam a fiação
//! com os painéis do Flip. Um deles decide se o arrasto da tira chega ao painel de quadros.
//!
//! # Por que o gate é TEXTUAL, e por que isso aqui basta
//!
//! A pergunta é sobre a RESOLUÇÃO DE FEATURES do cargo, que acontece antes de existir código
//! para observar: dentro de um teste só se vê o mundo já resolvido, e um `#[cfg]` aqui diria
//! apenas o que esta crate recebeu **nesta** corrida — nunca o que a shell pede. O que
//! decide é o `Cargo.toml` da shell, e é ele que se lê.
//!
//! ⚠️ **A metade que falta a um gate destes é a de OBSOLESCÊNCIA**, e ela está aqui: se um
//! `cfg` desaparecer do código da família, o gate reprova a dizer que a linha do
//! `Cargo.toml` já não descreve nada — senão ele vira licença (CLAUDE.md §5.0).

use std::fs;
use std::path::Path;

/// As features que governam código DESTA crate, e que a shell tem de repassar.
const GOVERNAM: &[&str] = &["panel-flip", "panel-flip-frames"];

fn raiz() -> &'static Path {
    // `CARGO_MANIFEST_DIR` = .../crates/ph2d-app-flip
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("a raiz do workspace fica dois níveis acima da crate")
}

/// Os `cfg(feature = "…")` que o código desta crate de facto escreve.
fn cfgs_no_codigo() -> Vec<String> {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut achados = Vec::new();
    let mut vistos = 0usize;
    for e in fs::read_dir(&src).expect("src/ legível") {
        let p = e.expect("entrada").path();
        if p.extension().is_none_or(|x| x != "rs") {
            continue;
        }
        vistos += 1;
        let txt = fs::read_to_string(&p).expect("ficheiro legível");
        for m in txt.match_indices("cfg(feature = \"") {
            let resto = &txt[m.0 + "cfg(feature = \"".len()..];
            if let Some(fim) = resto.find('"') {
                achados.push(resto[..fim].to_string());
            }
        }
    }
    // ⚠️ PISO DE POPULAÇÃO (HOWTO §2.7): uma varredura que perdeu o sujeito devolve uma
    // lista vazia, e `vazio ⊆ qualquer coisa` passa TRIVIALMENTE.
    assert!(
        vistos >= 40,
        "esta varredura viu {vistos} ficheiros e esperava >= 40 — perdeu o sujeito"
    );
    achados.sort();
    achados.dedup();
    achados
}

#[test]
fn the_shell_turns_on_the_features_this_family_reads() {
    let manifesto = fs::read_to_string(raiz().join("shells/desktop/Cargo.toml"))
        .expect("o Cargo.toml da shell");
    for f in GOVERNAM {
        let repasse = format!("ph2d-app-flip/{f}");
        assert!(
            manifesto.contains(&repasse),
            "a feature `{f}` governa código de `ph2d-app-flip`, e a shell NÃO a repassa \
             (esperava `{repasse}` no Cargo.toml dela).\n\
             Sem o repasse o `#[cfg]` é falso POR CONSTRUÇÃO e a crate compila verde com \
             aquele braço desligado — HOWTO §2.4, a armadilha MUDA."
        );
    }
}

#[test]
fn no_feature_gate_of_this_family_is_left_unforwarded() {
    let no_codigo = cfgs_no_codigo();
    for f in &no_codigo {
        assert!(
            GOVERNAM.contains(&f.as_str()),
            "o código desta crate lê `cfg(feature = \"{f}\")`, que NÃO está na lista \
             `GOVERNAM` — logo ninguém verifica se a shell o repassa.\n\
             Acrescente-o à lista (e o repasse ao Cargo.toml da shell)."
        );
    }
    // A metade de OBSOLESCÊNCIA: uma entrada que já não descreve nada é licença.
    for f in GOVERNAM {
        assert!(
            no_codigo.iter().any(|c| c == f),
            "`{f}` está em `GOVERNAM` e NENHUM ficheiro desta crate a lê — a entrada \
             envelheceu. Retire-a daqui e do repasse da shell, ou o gate vira licença."
        );
    }
}
