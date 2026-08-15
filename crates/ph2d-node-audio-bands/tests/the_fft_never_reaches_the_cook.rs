//! **A cerca do doc 63 §6, tornada ESTRUTURAL.**
//!
//! O plano escreveu, antes de existir código: *"FFT NUNCA entra no cook. A shell
//! bridge computa bandas por frame via `ph2d-audio-spectral` e publica como INPUT
//! do grafo — determinismo: bandas são função do ARQUIVO + playhead ⇒ scrub-exato."*
//!
//! Uma regra escrita num doc é disciplinar: ela vale enquanto alguém a lembra. Sem
//! uma dependência de áudio, este nó **não CONSEGUE** decodificar nem transformar
//! — a regra deixa de precisar de memória. É o mesmo movimento do
//! `no_codec_reaches_the_mixer` (ADR-0118), do `no_ml_runtime_reaches_the_mixer`
//! (ADR-0123) e do `the_event_core_is_a_leaf`, e do arch-gate que mantém a
//! `ph2d-paint-gpu` sem alcance à `ph2d-painter-brush`.
//!
//! ⚠️ **A propriedade é uma LISTA-BRANCA, não uma lista de proibidos.** Enumerar
//! *"nem `ph2d-audio-spectral`, nem `symphonia`, nem `realfft`…"* apodrece na
//! primeira crate pesada que ninguém previu; dizer **quem PODE entrar** não tem
//! como envelhecer, e o preço — uma dep genuinamente necessária um dia editar este
//! arquivo — é exatamente a fricção desejada.
//!
//! ⚠️ **`[dev-dependencies]` fica de fora de propósito:** ela não viaja para
//! consumidor nenhum (o precedente são as crates-nó em `[dev-dependencies]` da
//! `ph2d-gpu-cook`, machete-safe).

use std::path::Path;

/// As seções cujo conteúdo VIAJA para quem depende desta crate.
const REACHES_CONSUMERS: &[&str] = &["[dependencies]", "[build-dependencies]"];

/// O que um nó precisa para SER um nó, e nada mais.
const ALLOWED: &[&str] = &["ph2d-nodegraph", "ph2d-node-registry"];

#[test]
fn the_node_cannot_reach_a_decoder_or_a_transform() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    let text = std::fs::read_to_string(&manifest).expect("read ph2d-node-audio-bands/Cargo.toml");

    // O controle POSITIVO, e ele vem primeiro: um gate que procura uma seção
    // ausente encontra zero deps e passa provando NADA — o modo de falha de todo
    // gate baseado em parser.
    assert!(
        text.contains("[dependencies]"),
        "a seção `[dependencies]` sumiu do manifesto. Ou o arquivo foi renomeado, ou este gate \
         passou a medir outra coisa — e em nenhum dos dois casos ele prova que a FFT está fora."
    );

    for section in REACHES_CONSUMERS {
        for dep in deps_in(&text, section) {
            assert!(
                ALLOWED.contains(&dep.as_str()),
                "`ph2d-node-audio-bands` ganhou a dependência `{dep}` em `{section}`.\n\n\
                 Este nó recebe params, entradas e o playhead — nada mais. Quem decodifica um \
                 arquivo e corre a transformada é o SHELL (`motion_audio_gen`), que publica os \
                 níveis no canal externo; o nó lê a chave e mais nada.\n\
                 Doc 63 §6: *FFT NUNCA entra no cook.* Se o nó precisar de um número novo, quem \
                 o computa é a membrana, do lado dela da fronteira."
            );
        }
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
