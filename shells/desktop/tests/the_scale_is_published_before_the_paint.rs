//! **Arch-gate: a escala vai para a forma de runtime, e vai no sítio certo** (plano UI/UX W4c.2).
//!
//! # Por que isto é um arch-gate e não um teste de unidade
//!
//! O que a W4c.2 entrega é que `Spacing::Md.px()` devolva o número que o artista escolheu — e isso
//! depende de **alguém chamar `num_runtime::publish` uma vez por quadro**, na ordem certa. Nenhum
//! teste de unidade vê essa chamada: as suítes de `ph2d-tokens` publicam elas próprias, e as de
//! painel também. Apague a linha do produto e **toda a workspace fica verde** com a escala morta.
//!
//! ⚠️ E a ORDEM é metade da entrega. A publicação corre no **fim** do `tokens_bridge::dispatch`,
//! depois de a ponte ter escrito a camada com as edições deste quadro. Movê-la para o topo faria o
//! quadro em que o artista digita um número pintar com o valor **anterior** — o chip mostraria 20 e
//! a tela mediria 12, e o número "piscaria de volta". É o mesmo motivo que põe o read-back do
//! picker de cor antes dos intents, e a mesma classe de defeito.

use std::fs;

const BRIDGE: &str = "src/render_loop/tokens_bridge.rs";
const LOOP: &str = "src/render_loop/mod.rs";

fn read(path: &str) -> String {
    fs::read_to_string(path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

/// A publicação existe, e é a ÚLTIMA coisa que a ponte faz.
#[test]
fn the_bridge_publishes_the_scale_after_it_has_written_the_layer() {
    let src = read(BRIDGE);

    let publish = src.find("num_runtime::publish(").expect(
        "a ponte de tokens nao publica a escala — `Spacing::px()` devolve a fabrica para \
                 sempre, e nenhum teste de unidade da' por isso",
    );

    // ⚠️ O oráculo é POSICIONAL de propósito, e a âncora é o laço de intents (onde a camada é
    // escrita) — não uma distância em bytes, que expira quando alguém acrescenta um intent.
    let intents = src
        .find("for intent in ph2d_panel_tokens::drain_intents()")
        .expect(
            "o laco de intents mudou de forma — re-ancore este gate no sitio que ESCREVE a \
                 camada, nunca num numero de linhas",
        );
    assert!(
        publish > intents,
        "a publicacao corre ANTES do laco que escreve a camada: o quadro em que o artista digita \
         um numero pintaria com o valor anterior"
    );

    // E ela é o último efeito — nada escreve a camada depois dela.
    let tail = &src[publish..];
    for writer in [
        "set_num_override(",
        "set_num_overrides(",
        "set_color_override(",
        "set_color_overrides(",
    ] {
        assert!(
            !tail.contains(writer),
            "`{writer}` corre DEPOIS da publicacao — essa escrita nao chega a tabela deste quadro"
        );
    }
}

/// **O CONTROLE POSITIVO:** as âncoras existem mesmo. Sem ele, um arquivo renomeado tornaria os
/// dois `find` em `None`, o `expect` dispararia… e uma varredura *vazia* (o arquivo lido de outro
/// caminho) passaria calada.
#[test]
fn the_gate_is_reading_the_file_it_thinks_it_is() {
    let src = read(BRIDGE);
    assert!(
        src.contains("pub(crate) fn dispatch("),
        "o {BRIDGE} nao tem a fn `dispatch` — este gate esta' a ler outro arquivo"
    );
    assert!(
        src.len() > 2_000,
        "{BRIDGE} veio curto demais para ser o produto"
    );
}

/// A ponte é chamada pelo laço de quadro **sem condição** — publicar dentro dela só vale se ela
/// corre sempre.
///
/// ⚠️ Se um dia alguém a gatear na visibilidade do painel (uma optimização plausível — a
/// precificação do áudio é gateada assim), a escala congelaria em silêncio para quem fechasse o
/// painel. Este gate torna essa mudança ruidosa.
#[test]
fn the_frame_calls_the_bridge_without_gating_it_on_the_panel() {
    let src = read(LOOP);
    let call = src
        .find("tokens_bridge::dispatch(")
        .expect("o laco de quadro nao chama a ponte de tokens");

    // A linha da chamada, e a que vem antes dela.
    let line_start = src[..call].rfind('\n').map_or(0, |i| i + 1);
    let prev_start = src[..line_start.saturating_sub(1)]
        .rfind('\n')
        .map_or(0, |i| i + 1);
    let two_lines = &src[prev_start..src[call..].find('\n').map_or(src.len(), |i| call + i)];

    for gate in ["is_panel_visible", "panel_visible", "is_collapsed"] {
        assert!(
            !two_lines.contains(gate),
            "a ponte de tokens passou a ser chamada sob `{gate}` — a escala congela para quem \
             fecha o painel, e o app pinta com a escala do modo anterior"
        );
    }
}
