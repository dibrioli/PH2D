//! **O TEMPO DE DISPATCH DO LOG TEM DE VIR COM A ÁREA QUE ELE MOVE.**
//!
//! ⚠️ **Irmão do [`the_stamp_line_carries_its_divisor`], e a mesma doença um
//! sistema adiante** (Enio, 2026-08-01):
//!
//! ```text
//! [frame] total=16.21ms (~62 fps) | painter-dispatch(cpu)=11.80ms
//!   tool-tick: media 3.89ms em 115/120 | stamps: 0 entregas
//!   worker: busy 66% away 18% sleep 16% | TAXA DA AGUA 38.6 Hz
//! ```
//!
//! **11,80 ms de dispatch sem um único carimbo** — e a linha não decide nada,
//! porque o custo é dominado pelo *gather + premultiply + upload* da região que
//! o dreno publicou, e a região não estava no log. As duas leituras pedem curas
//! opostas: *um retângulo grande de vez em quando* (o alvo é a frequência) ×
//! *um retângulo grande sempre* (o alvo é o TAMANHO da região).
//!
//! Medido headless antes de gastar um smoke (doc 28 §5.53): com a água correndo
//! e a mão parada o dreno publica **8,26 M px por quadro drenado** numa tela de
//! 16,8 M — **metade dela** — para **2,07 M células** de água viva, ou seja o
//! retângulo pede **3,99×** o que a água tocou. A bbox de uma faixa diagonal é
//! um múltiplo da faixa, que é exatamente o que o censo do `live_span_cells` já
//! tinha nomeado para o outro lado.
//!
//! Arch-gate sobre o fonte porque o `[frame]` só existe com janela — nenhum
//! teste de unidade alcança aquele `eprintln`.

const SRC: &str = include_str!("../src/render_loop/mod.rs");
const BRIDGE: &str = include_str!("../src/render_loop/painter_bridge.rs");

/// O divisor é IMPRESSO ao lado do tempo que ele divide.
///
/// Mutação que sangra: tirar `{prev_mpx:` do formato — a linha volta a dizer só
/// o milissegundo, e um smoke não distingue as duas curas.
#[test]
fn the_dispatch_line_prints_the_area_it_moves() {
    // ⚠️ **A âncora é o PLACEHOLDER, nunca a frase** — a lição que o gate irmão
    // pagou quando casou com o próprio doc-comment.
    let line = SRC
        .split("painter-dispatch(cpu)={dispatch_ms:")
        .nth(1)
        .expect("o `[frame]` imprime o tempo de dispatch");
    let head: String = line.chars().take(200).collect();
    assert!(
        head.contains("{prev_mpx:"),
        "o `[frame]` imprime o dispatch sem a AREA publicada, e `11,80ms` sozinho \
         nao distingue um retangulo grande de um pequeno:\n{head}"
    );
    assert!(
        head.contains("{prev_n}"),
        "a area sai sem o numero de QUADROS que a produziram — uma soma sem o seu \
         proprio divisor e' o defeito que este gate existe para pegar:\n{head}"
    );
}

/// **A área é contada ONDE a bbox é resolvida**, e não num sítio que a adivinhe.
///
/// ⚠️ O numerador (`dispatch_ms`) e o divisor (`prev_mpx`) têm de descrever os
/// MESMOS quadros. A contagem mora dentro do braço que faz o dreno de CPU, logo
/// depois do `take_preview_upload_bbox` — noutro lugar ela contaria quadros que
/// o dispatch não pagou.
///
/// Mutação que sangra: mover o `note_preview_px` para fora do braço do dreno.
#[test]
fn the_area_is_counted_where_the_bbox_is_resolved() {
    // ⚠️ **A JANELA É ESTRUTURAL, NÃO UMA DISTÂNCIA EM BYTES.** A 1ª versão
    // varria 600 caracteres depois da bbox e **quebrou com o próprio comentário
    // que o fix da fórmula acrescentou** — o código estava certo e o gate
    // reprovou. É a classe que a `line/Vector` já pagou: *um arch-gate ancorado
    // numa distância é um proxy que expira*. A pergunta verdadeira é *a
    // contagem está DENTRO do braço que resolveu a bbox?*, e o fim desse braço
    // é a estrutura, não um número.
    let at_bbox = BRIDGE
        .find("painter_dirty_bbox = painter.take_preview_upload_bbox();")
        .expect("o dreno de CPU resolve a bbox de upload");
    let rest = &BRIDGE[at_bbox..];
    // O braço fecha na primeira linha que volta à indentação de 8 espaços com
    // uma chave — tudo antes disso ainda é o mesmo bloco.
    let arm_end = rest.find("\n        }").unwrap_or(rest.len());
    let arm = &rest[..arm_end];
    assert!(
        arm.contains("note_preview_px"),
        "a area publicada nao e' contada DENTRO do braco que resolve a bbox — o \
         divisor passaria a descrever outros quadros que o numerador"
    );
    // E uma vez só: dois sítios contariam o mesmo quadro duas vezes.
    assert_eq!(
        BRIDGE.matches("note_preview_px").count(),
        1,
        "ha mais de um sitio contando a area publicada — o divisor contaria o \
         mesmo quadro duas vezes"
    );
}
