//! **O TEMPO DE CARIMBO DO LOG TEM DE VIR COM O NÚMERO DE ENTREGAS.**
//!
//! ⚠️ **Este gate nasceu de um log de smoke em que a linha não decidia nada**
//! (Enio, 2026-07-31): `stamps: media 105,82ms pico 531,42ms em 26/120 frames`
//! — e as duas leituras possíveis pedem curas **OPOSTAS**:
//!
//! - **UM** re-stamp de forma inteira a 105 ms ⇒ o alvo é o re-stamp;
//! - **CINQUENTA** entregas incrementais a 2 ms ⇒ o alvo é a TAXA de entrega, e
//!   o comentário do `painter_canvas_move` já descreve a espiral que a produz.
//!
//! O custo por-evento foi medido em **2,03 ms** (censo dos quatro meios) e o
//! custo do mesmo CAMINHO é constante em 16 ou em 640 entregas (doc 28 §5.47) —
//! então a diferença entre as duas leituras é de UMA ordem de grandeza no
//! número de eventos, e o log a escondia atrás de uma média.
//!
//! É a mesma doença que o balde da poça acabara de curar no outro lado da mesma
//! linha (`wet_diag::note_cells`): *um custo sem divisor não é atribuível*.
//!
//! Arch-gate sobre o fonte porque o `[frame]` só existe com janela — nenhum
//! teste de unidade alcança aquele `eprintln`.

const SRC: &str = include_str!("../src/render_loop/mod.rs");

/// O divisor é IMPRESSO ao lado do tempo que ele divide.
///
/// Mutação que sangra: tirar `{stamp_ev}` do formato — a linha volta a dizer só
/// a média, e um smoke não distingue as duas curas.
#[test]
fn the_stamp_line_prints_how_many_deliveries_it_averages() {
    // ⚠️ **A âncora é o PLACEHOLDER, nunca a frase.** A 1ª versão procurava
    // `"stamps: media"` e casou com o doc-comment que — nesta mesma wave —
    // passou a CITAR a linha do log para explicar por que o divisor existe: o
    // scanner leu a prosa e reprovou o código correto. *Um oráculo que casa com
    // a documentação de si mesmo não está olhando para o produto.*
    let line = SRC
        .split("{stamp_avg:")
        .nth(1)
        .expect("o `[frame]` imprime a linha de stamps");
    // A janela é o resto do literal de formato até a quebra seguinte.
    let head: String = line.chars().take(220).collect();
    assert!(
        head.contains("{stamp_ev}"),
        "a linha de stamps nao imprime o NUMERO de entregas: `media 105ms` nao \
         distingue um re-stamp de forma inteira de cinquenta eventos incrementais, \
         e as curas sao opostas.\n  achei: {head}"
    );
    assert!(
        head.contains("{stamp_per:"),
        "a linha de stamps nao imprime o custo POR ENTREGA — que e o numero com \
         que se compara o censo dos quatro meios (2,03 ms/move a 4096).\n  achei: {head}"
    );
}

/// O divisor é acumulado no MESMO sítio que a soma.
///
/// ⚠️ Contado noutro lugar, os dois baldes discordariam sobre QUAIS frames
/// entraram na janela, e a média por entrega seria de uma amostra que ninguém
/// escolheu — verde, e mentindo. Mutação que sangra: mover o `STAMP_EV` para
/// fora do bloco `if st > 0`.
#[test]
fn the_divisor_is_counted_where_the_sum_is() {
    let block = SRC
        .split("FRAME_PROF_STAMP_SUM_US.with")
        .nth(1)
        .expect("a soma da janela de stamps existe");
    // Do acumulador da soma ate o fecho do bloco `if st > 0`.
    let end = block
        .find("\n                }")
        .expect("o bloco `if st > 0` fecha");
    let inside = &block[..end];
    assert!(
        inside.contains("FRAME_PROF_STAMP_EV"),
        "o divisor nao e contado dentro do MESMO `if st > 0` da soma: os dois baldes \
         passam a descrever janelas diferentes, e a media por entrega deixa de ser \
         a media daquela soma.\n  bloco: {inside}"
    );
}
