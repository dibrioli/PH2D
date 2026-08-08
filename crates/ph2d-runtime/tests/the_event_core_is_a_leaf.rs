//! **A contenção é ESTRUTURAL, não disciplinar.**
//!
//! Esta crate é o tipo que a timeline, a física, o áudio, o Luau e a UI compartilham para falar
//! de um sinal. Se ela ganhar uma dependência, TODO consumidor a paga — e o candidato natural
//! para hospedar isto, o `ph2d-script`, arrasta uma VM Luau (`mlua`) mais o `bevy_ecs`. O mixer
//! de áudio linkando um interpretador para saber que uma porta abriu é a mesma fronteira que o
//! `no_codec_reaches_the_mixer` (ADR-0118) e o `no_ml_runtime_reaches_the_mixer` (ADR-0123)
//! defendem do outro lado da casa; este é o gate irmão, escrito ANTES de o primeiro consumidor
//! existir.
//!
//! ⚠️ **A propriedade é "zero", não uma lista de proibidos.** Uma lista enumera o que alguém
//! lembrou de proibir, e apodrece na primeira crate pesada que ninguém previu; "nenhuma" não
//! tem como envelhecer. O preço é que uma dep genuinamente leve, um dia, terá de EDITAR este
//! gate — que é exatamente a fricção desejada, porque a decisão passa a ser deliberada.
//!
//! `[dev-dependencies]` fica de fora de propósito: ela não alcança consumidor nenhum (o
//! precedente são as crates-nó em `[dev-dependencies]` da `ph2d-gpu-cook`, machete-safe).

use std::path::Path;

/// As seções cujo conteúdo VIAJA para quem depende desta crate.
const REACHES_CONSUMERS: &[&str] = &["[dependencies]", "[build-dependencies]"];

#[test]
fn the_event_core_has_no_dependencies_at_all() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    let text = std::fs::read_to_string(&manifest).expect("read ph2d-runtime/Cargo.toml");

    // O controle POSITIVO, e ele vem primeiro: um gate que procura uma seção ausente encontra
    // zero deps e passa provando nada — o modo de falha de todo gate baseado em parser.
    assert!(
        text.contains("[dependencies]"),
        "a seção `[dependencies]` sumiu do manifesto. Ou o arquivo foi renomeado, ou este gate \
         passou a medir a coisa errada — em nenhum dos dois casos ele está provando que a crate \
         é uma folha."
    );

    for section in REACHES_CONSUMERS {
        let found = deps_in(&text, section);
        assert!(
            found.is_empty(),
            "`ph2d-runtime` ganhou dependências em `{section}`: {found:?}.\n\n\
             Esta crate é o tipo que o ÁUDIO, o Luau, a UI e o shell compartilham para ler um \
             NOME. Toda dep aqui é paga por cada um deles — e é por isso que o núcleo de eventos \
             NÃO mora no `ph2d-script`, que arrastaria uma VM Luau para dentro do mixer.\n\
             Se um sinal precisa de algo pesado, quem precisa é o CONSUMIDOR, do lado dele da \
             fronteira."
        );
    }
}

/// Os nomes de dependência declarados sob `header`, até a próxima seção.
fn deps_in(manifest: &str, header: &str) -> Vec<String> {
    manifest
        .lines()
        .skip_while(|l| l.trim() != header)
        .skip(1)
        .take_while(|l| !l.trim_start().starts_with('['))
        .filter(|l| {
            let t = l.trim();
            !t.is_empty() && !t.starts_with('#')
        })
        .map(|l| {
            l.trim()
                .split(['=', ' '])
                .next()
                .unwrap_or("")
                .trim()
                .to_owned()
        })
        .collect()
}
