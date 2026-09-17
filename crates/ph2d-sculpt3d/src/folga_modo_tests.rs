//! Os gates da [`super::FolgaModo`] — a tabela do cabeçalho dela, cobrada
//! célula a célula, mais as duas propriedades que a tornam um MODO e não um
//! segundo produto.

use super::FolgaModo;

/// ⭐⭐⭐ **A TABELA DO CABEÇALHO, célula a célula** — as três situações em que as
/// duas leis se separam, com os números escritos à mão.
///
/// ⚠️ **A 3.ª célula é a lei inteira num número:** com o alvo ATRÁS (`d < 0`) a
/// lei do alvo devolve `−0,6` sobre um vão de `0,5` — o barro **ultrapassa** a
/// peça que devia tocar —, e a simétrica devolve `−0,4`, que é parar a `0,1`
/// dela. *É a célula que nenhum rótulo prevê e que só uma medição mostra.*
#[test]
fn a_tabela_das_duas_leis_bate_celula_a_celula() {
    let casos: [(f32, f32, f32, f32); 3] = [
        // (d, folga, do alvo, simétrica)
        (0.5, 0.1, 0.4, 0.4),
        (0.5, 0.6, -0.1, 0.0),
        (-0.5, 0.1, -0.6, -0.4),
    ];
    for (d, folga, alvo, sim) in casos {
        let a = FolgaModo::DoAlvo.aplica(d, folga);
        let s = FolgaModo::Simetrica.aplica(d, folga);
        assert!(
            (a - alvo).abs() < 1e-6,
            "d={d} folga={folga}: a lei do alvo deu {a}, a tabela diz {alvo}"
        );
        assert!(
            (s - sim).abs() < 1e-6,
            "d={d} folga={folga}: a simetrica deu {s}, a tabela diz {sim}"
        );
    }
}

/// ⭐⭐ **A SIMÉTRICA nunca inverte o sentido, e a do alvo inverte** — a
/// propriedade que dá nome às duas, varrida em vez de amostrada.
///
/// ⛔ **A varredura é o que torna isto uma LEI e não três exemplos:** ela cobre
/// a folga de `0` a `2` sobre vãos dos dois sinais, e a asserção é sobre o
/// SINAL do resultado, nunca sobre o valor.
#[test]
fn a_simetrica_nunca_inverte_o_sentido_e_a_do_alvo_inverte() {
    let mut inverteu_no_alvo = 0;
    for i in 0..=40u8 {
        let folga = f32::from(i) * 0.05;
        for d in [0.5f32, -0.5, 0.25, -1.25] {
            let s = FolgaModo::Simetrica.aplica(d, folga);
            assert!(
                s == 0.0 || s.signum() == d.signum(),
                "folga={folga} d={d}: a simetrica devolveu {s} e inverteu o sentido"
            );
            assert!(
                s.abs() <= d.abs() + 1e-6,
                "folga={folga} d={d}: a simetrica devolveu {s}, MAIOR que o vao"
            );
            let a = FolgaModo::DoAlvo.aplica(d, folga);
            if a != 0.0 && a.signum() != d.signum() {
                inverteu_no_alvo += 1;
            }
        }
    }
    assert!(
        inverteu_no_alvo > 0,
        "a lei do alvo deixou de inverter — a razao de o botao existir desapareceu, \
         e este gate tem de ser relido antes de ser apagado"
    );
}

/// ⭐ **No ponto NEUTRO as duas são a IDENTIDADE, ao bit** — é isso que faz o
/// modo ser invisível para quem não lhe toca, e o que mantém as `14` fixturas
/// do oráculo fora do alcance desta wave.
///
/// ⚠️ **O `−0,0` está na varredura de propósito:** `(-0.0f32).signum()` é `−1` e
/// `(-0.0f32).abs()` é `0,0`, logo a simétrica devolve `−0,0` — que **é** igual
/// a `0,0` em `==` e tem outros bits. A asserção é de VALOR, e a nota existe
/// para quem a apertar a bits não ler isto como defeito.
#[test]
fn no_ponto_neutro_as_duas_sao_a_identidade() {
    for d in [0.0f32, -0.0, 0.5, -0.5, 1.0, -3.25, 1e-7, -1e-7] {
        for modo in FolgaModo::ALL {
            assert!(
                modo.aplica(d, 0.0) == d,
                "{modo:?}: com folga zero, {d} virou {}",
                modo.aplica(d, 0.0)
            );
        }
    }
}

/// ⚠️ **O valor de FÁBRICA é o do alvo** — e isto é um gate porque é o que
/// mantém o corpus do oráculo a medir o pincel que ele gravou.
#[test]
fn o_valor_de_fabrica_e_a_lei_do_alvo() {
    assert_eq!(
        FolgaModo::default(),
        FolgaModo::DoAlvo,
        "o default mudou — as 14 fixturas do oraculo passam a medir outro pincel"
    );
    assert_eq!(
        FolgaModo::ALL[0],
        FolgaModo::DoAlvo,
        "a ordem dos chips mudou"
    );
}
