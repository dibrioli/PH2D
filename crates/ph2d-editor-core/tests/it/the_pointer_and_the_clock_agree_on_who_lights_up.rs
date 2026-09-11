//! **Se o PONTEIRO o acende, o RELÓGIO tem de o conduzir.**
//!
//! # A classe de defeito que este gate torna impossível
//!
//! O eixo do hover tem três elos, e cada um pode estar certo sozinho enquanto a cadeia está morta:
//!
//! 1. o **ponteiro** promove o estado (`dispatch::hover`);
//! 2. o **relógio** publica um alvo para esse id (`WidgetStore::hover_targets`) e integra;
//! 3. o **pintor** mistura a tinta por `t`, e o painel entrega-lhe o par.
//!
//! Medido em 2026-08-15, **quatro famílias de widget tinham 1 e 3 e não tinham 2** — campo de
//! texto, área de texto, chip numérico e dropdown. Os pintores declaravam a lei (`Border →
//! BorderEmph`), os painéis passavam `(estado, hover_live(id))`, o `dropdown_visual` devolvia o
//! par — e como ninguém publicava o alvo, o `hover_live` devolvia `SETTLED` **para sempre** e cada
//! uma dessas chamadas entregava o NEUTRO. Eles reagiam e SALTAVAM, com a suíte inteira verde.
//!
//! ⚠️ **Um gate por-widget não fecha isto**, e é por isso que este existe: os gates de eixo passam
//! o par À MÃO (`Some((live, t))`), então provam que o *pintor* interpola — nunca que alguém lhe dá
//! um `t`. O elo que falha é o do meio, e ele não aparece em nenhuma das duas pontas.
//!
//! # Por que a PROMOÇÃO é o oráculo
//!
//! O `dispatch::hover` promover um tipo é a declaração executável de que *este widget tem um
//! hover*. Não há segunda lista a mantar: quem acrescentar um braço lá e esquecer o do relógio
//! falha aqui, com o nome do tipo na mensagem.
//!
//! ⚠️ **A direcção é UMA só, de propósito.** O inverso (*tudo o que o relógio conduz o ponteiro
//! promove*) seria FALSO por desenho: o `Slider` conta `Dragging` e o `Button` conta `Focused`,
//! estados que nascem do arrasto e do teclado, não do `hover.rs`.

use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

/// Os `InteractiveState::X` que um ficheiro menciona.
fn variants(src: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut rest = src;
    while let Some(i) = rest.find("InteractiveState::") {
        rest = &rest[i + "InteractiveState::".len()..];
        let name: String = rest
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
            .collect();
        if !name.is_empty() && name.chars().next().is_some_and(char::is_uppercase) {
            out.push(name);
        }
    }
    out.sort_unstable();
    out.dedup();
    out
}

/// O corpo do `hover_targets`, e só ele.
///
/// ⚠️ **Ler o ficheiro inteiro daria um falso VERDE:** o `store_census.rs` também hospeda o
/// `number_fields`, que menciona `InteractiveState::NumberInput` — o tipo passaria por publicado
/// sem estar. *Um oráculo que casa com o vizinho da função não está a olhar para a função.*
fn hover_targets_body(src: &str) -> &str {
    let start = src
        .find("pub fn hover_targets")
        .expect("o `hover_targets` mudou de nome ou de ficheiro");
    let tail = &src[start..];
    let end = tail
        .find("\n    }\n")
        .expect("o corpo do `hover_targets` nao fecha como esperado");
    &tail[..end]
}

#[test]
fn every_type_the_pointer_promotes_is_driven_by_the_clock() {
    let hover = std::fs::read_to_string(root().join("src/interaction/dispatch/hover.rs")).expect(
        "o `dispatch::hover` mudou de sitio — este gate perdeu o alvo, nao ficou satisfeito",
    );
    let census = std::fs::read_to_string(root().join("src/interaction/state/store_census.rs"))
        .expect("o `store_census` mudou de sitio");

    let promoted = variants(&hover);
    let driven = variants(hover_targets_body(&census));

    // CONTROLO POSITIVO nas duas pontas: uma varredura vazia é um gate satisfeito com nada.
    assert!(
        promoted.len() >= 6,
        "so {} tipos promovidos pelo ponteiro — o scanner partiu-se: {promoted:?}",
        promoted.len()
    );
    assert!(
        driven.len() >= 6,
        "so {} tipos conduzidos pelo relogio — o recorte do corpo partiu-se: {driven:?}",
        driven.len()
    );

    let orphans: Vec<&String> = promoted.iter().filter(|p| !driven.contains(p)).collect();
    assert!(
        orphans.is_empty(),
        "o ponteiro ACENDE estes tipos e o relogio nao os CONDUZ: {orphans:?}\n\
         o `hover_live` deles devolve SETTLED para sempre, entao o pintor recebe o NEUTRO e o\n\
         widget REAGE mas SALTA — com os gates de eixo verdes, porque eles passam o par a mao.\n\
         fix: acrescentar o braco em `WidgetStore::hover_targets`, com a lei DERIVADA do `soft`\n\
         que o pintor daquele widget ja declara."
    );
}
