//! Gates do `pulse.signal`.
//!
//! O nó é deliberadamente magro — ele NÃO publica nada —, então o que há para afirmar é
//! exatamente isso: que ele não mexe no pulso, e que a pergunta *"disparou?"* mora aqui.

use super::*;

fn pulse_stream(rows: &[f32]) -> Stream {
    Stream::new(rows.len()).with(PULSE_COL, Column::Scalar(rows.to_vec()))
}

/// **O nó não altera o pulso que atravessa.** Ele é um medidor em série, e um medidor que
/// mudasse a corrente seria outra coisa.
#[test]
fn o_pulso_atravessa_intacto() {
    let input = pulse_stream(&[0.0, 1.0, 0.0, 1.0]);
    let out = passthrough(&input);
    let (Some(Column::Scalar(a)), Some(Column::Scalar(b))) =
        (input.get(PULSE_COL), out.get(PULSE_COL))
    else {
        panic!("os dois streams carregam a coluna de pulso");
    };
    assert_eq!(a, b, "byte a byte");
    assert_eq!(out.count(), input.count(), "e a contagem de linhas");
}

/// **A soleira é a da FAMÍLIA (`> 0.5`), e ela mora aqui** — não do lado de quem publica.
#[test]
fn a_contagem_de_disparos_usa_a_soleira_da_familia() {
    assert_eq!(fired_rows(&pulse_stream(&[0.0, 0.0, 0.0])), 0);
    assert_eq!(fired_rows(&pulse_stream(&[0.0, 1.0, 0.0])), 1);
    assert_eq!(fired_rows(&pulse_stream(&[1.0, 1.0, 1.0])), 3);
    // A soleira, dos dois lados: 0,5 não dispara, 0,51 dispara.
    assert_eq!(fired_rows(&pulse_stream(&[0.5])), 0, "0,5 nao e disparo");
    assert_eq!(fired_rows(&pulse_stream(&[0.51])), 1);
}

/// **Um stream SEM a coluna de pulso conta zero, nunca panica.** É o estado de um documento a
/// meio de uma edição, e a resposta honesta é *nada disparou* — a alternativa (unwrap) mataria
/// o quadro do artista por causa de um fio solto.
#[test]
fn um_stream_sem_coluna_de_pulso_conta_zero() {
    assert_eq!(fired_rows(&Stream::new(4)), 0);
    let wrong_kind = Stream::new(2).with(PULSE_COL, Column::Vec2(vec![[1.0, 1.0], [1.0, 1.0]]));
    assert_eq!(fired_rows(&wrong_kind), 0, "coluna do tipo errado tambem");
}

/// **O nome é TEXT PARAM, e o manifesto NÃO o declara** — é assim que ele existe sem tocar o
/// contrato congelado (`NodeManifest = 8`, params são `f32`).
#[test]
fn o_nome_nao_e_um_param_do_manifesto() {
    assert!(
        MANIFEST.params.is_empty(),
        "o nome viaja pelo canal de TEXTO do Graph, nao pelo manifesto"
    );
    assert!(
        PARAM_HINTS
            .iter()
            .any(|h| h.param == NAME_KEY && matches!(h.widget, ParamWidget::Text)),
        "e o painel o desenha como campo de texto"
    );
}

/// **Passthrough: a porta de saída tem o MESMO tipo da de entrada.** Sem isso o nó não caberia
/// no meio de um fio, e o artista teria de escolher entre nomear um pulso e usá-lo.
#[test]
fn a_saida_e_um_pulso_como_a_entrada() {
    assert_eq!(MANIFEST.inputs.len(), 1);
    assert_eq!(MANIFEST.outputs.len(), 1);
    assert_eq!(MANIFEST.inputs[0].ty, MANIFEST.outputs[0].ty);
    assert_eq!(MANIFEST.outputs[0].ty, PULSE);
}
