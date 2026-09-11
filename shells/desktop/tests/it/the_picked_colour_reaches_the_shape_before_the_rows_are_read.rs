//! **A cor escolhida PINTA a forma, e pinta ANTES de a row ser derivada** — arch-gate.
//!
//! ⚠️ **Nenhum teste de unidade alcança isto.** A viagem de volta do picker mora no
//! `render_loop`, que exige janela e GPU; o que se pode afirmar sem elas é a FORMA do fonte, e é
//! o que este gate faz — o molde do `the_edit_frame_only_prices_when_the_delivery_section_is_open`
//! do editor de áudio.
//!
//! Duas propriedades, e as duas falham de maneiras diferentes:
//!
//! 1. **A escrita existe.** Sem ela a swatch abre o picker, o artista escolhe uma cor, e a row é
//!    re-derivada do documento no quadro seguinte — que não mudou. A cor volta à antiga sozinha,
//!    e um picker que abre e não pinta é pior que uma swatch muda: ele *parece* funcionar.
//! 2. **A escrita vem ANTES da leitura das rows.** Depois, a swatch mostraria a cor ANTIGA por um
//!    quadro — o piscar que faz o artista clicar duas vezes e escolher outra coisa.
//!
//! ⚠️ E a pergunta *"o alvo é uma row desta moldura?"* é a terceira metade: o picker é PARTILHADO
//! (Painter, Vector e timeline usam o mesmo canal), então escrever sem perguntar pintaria a forma
//! errada sempre que outro painel o abrisse.

use std::fs;

const SRC: &str = "src/render_loop/mod.rs";

fn source() -> String {
    fs::read_to_string(SRC).unwrap_or_else(|e| panic!("nao consegui ler {SRC}: {e}"))
}

/// **O retorno existe, e passa pela porta que sabe de quem é o alvo.**
#[test]
fn the_shell_writes_the_picked_colour_through_the_shape_lookup() {
    let s = source();
    assert!(
        s.contains("ui_panel_spec::picker_shape"),
        "o `render_loop` nao consulta o `picker_shape` — ou a viagem de volta do picker sumiu, ou \
         alguem a reescreveu sem perguntar de quem e' o alvo (e ai' ela pinta a forma errada \
         quando o Painter abre o picker)"
    );
    assert!(
        s.contains("picker_target()"),
        "o `render_loop` deixou de ler o alvo do picker"
    );
}

/// **E ele corre ANTES de as rows serem derivadas.**
///
/// ⚠️ O oráculo é a ORDEM no fonte, e não a presença: as duas linhas podem existir e estar
/// trocadas, que é precisamente o defeito de um quadro de atraso. Um gate que só contasse
/// ocorrências ficaria verde sobre a swatch a piscar.
#[test]
fn the_colour_lands_before_the_rows_are_derived() {
    let s = source();
    let write = s
        .find("ui_panel_spec::picker_shape")
        .expect("o retorno do picker nao esta' no `render_loop`");
    let read = s
        .find("ui_panel_spec::live_rows")
        .expect("a derivacao das rows nao esta' no `render_loop`");
    assert!(
        write < read,
        "a cor e' escrita DEPOIS de as rows serem lidas ({write} > {read}) — a swatch mostraria a \
         cor antiga por um quadro"
    );
}
