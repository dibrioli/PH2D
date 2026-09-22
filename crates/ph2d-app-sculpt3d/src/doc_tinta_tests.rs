//! Os gates das CORRIDAS: a ida e volta AO BIT, as duas armadilhas de `f32`, e
//! a cerca da contagem.

use super::{das_corridas, em_corridas};

fn volta(a: &[[f32; 3]]) -> Option<Vec<[f32; 3]>> {
    das_corridas(&em_corridas(a), a.len())
}

/// ⭐⭐⭐ **A ida e volta é AO BIT** — e a fixtura tem as três formas que um
/// plano tem: o chapado, o de um traço (uma mancha no meio) e o pintado todo.
#[test]
fn as_corridas_devolvem_as_amostras_ao_bit() {
    let chapado = vec![[0.25f32, 0.5, 0.75]; 50];
    let mut um_traco = chapado.clone();
    for (i, a) in um_traco.iter_mut().enumerate().take(30).skip(20) {
        *a = [i as f32 / 30.0, 0.1, 0.9];
    }
    let todo: Vec<[f32; 3]> = (0..50).map(|i| [i as f32, -(i as f32), 0.5]).collect();

    for (nome, v) in [("chapado", chapado), ("um traço", um_traco), ("todo", todo)] {
        assert_eq!(
            volta(&v).as_deref(),
            Some(&v[..]),
            "{nome}: a ida e volta perdeu bits"
        );
    }
    assert!(
        volta(&[]).as_deref() == Some(&[][..]),
        "um plano vazio tem de voltar vazio"
    );
}

/// ⭐⭐ **E as corridas são MESMO corridas** — sem esta metade, um encoder que
/// nunca juntasse nada passava a ida-e-volta inteira.
#[test]
fn um_plano_chapado_cabe_numa_corrida_so() {
    let chapado = vec![[0.25f32, 0.5, 0.75]; 6_291_458 / 1000];
    let c = em_corridas(&chapado);
    assert_eq!(
        c.len(),
        1,
        "um plano chapado tem de caber numa corrida — deu {}",
        c.len()
    );
    assert_eq!(c[0].0 as usize, chapado.len(), "a contagem da corrida");
}

/// ⛔⛔⛔ **`-0.0 == 0.0` É VERDADE, e os bits são DIFERENTES.**
///
/// Uma corrida fechada por `==` juntaria os dois e devolveria o sinal errado —
/// *uma compressão que se diz sem perda e que troca um sinal de zero é pior do
/// que uma que se diz com perda*, porque ninguém vai procurar ali.
///
/// ⚠️ A asserção é sobre os **BITS** e não sobre o valor: `assert_eq!` em
/// `f32` diria que os dois são iguais e o gate ficaria VÁCUO.
#[test]
fn o_zero_negativo_nao_se_junta_ao_positivo() {
    let v = vec![[0.0f32, 0.0, 0.0], [-0.0f32, 0.0, 0.0], [0.0f32, 0.0, 0.0]];
    assert_eq!(
        em_corridas(&v).len(),
        3,
        "o -0.0 tem de partir a corrida: ele NÃO é o mesmo valor guardado"
    );
    let volta = volta(&v).expect("a contagem bate");
    let bits: Vec<u32> = volta.iter().map(|c| c[0].to_bits()).collect();
    assert_eq!(
        bits,
        vec![0.0f32.to_bits(), (-0.0f32).to_bits(), 0.0f32.to_bits()],
        "os bits do sinal do zero não voltaram — o `==` juntou-os"
    );
}

/// ⭐⭐ **E o `NaN` COMPORTA-SE, que é o outro lado da mesma escolha.**
///
/// Por `==` ele nunca é igual a si próprio, logo uma corrida de mil `NaN`
/// viraria mil corridas; pelos bits ela é UMA, e volta com os mesmos bits.
#[test]
fn uma_corrida_de_nan_e_uma_corrida() {
    let v = vec![[f32::NAN, 1.0, 2.0]; 8];
    let c = em_corridas(&v);
    assert_eq!(
        c.len(),
        1,
        "os NaN têm os mesmos bits, logo são UMA corrida"
    );
    let volta = volta(&v).expect("a contagem bate");
    assert!(
        volta.iter().all(|a| a[0].is_nan() && a[1] == 1.0),
        "o NaN não voltou"
    );
}

/// ⛔⛔ **A CERCA DA CONTAGEM** — corridas que não somam o que a topologia pede
/// são recusadas, e a soma é feita ANTES de alocar.
///
/// ⚠️ **As duas direcções**, porque as curas de quem lê são diferentes: a mais
/// curta instalaria um plano pequeno numa malha grande (tinta no sítio errado),
/// e a mais comprida faria um `with_capacity` de biliões sobre um documento
/// forjado.
#[test]
fn corridas_que_nao_somam_o_que_a_malha_pede_sao_recusadas() {
    let c = vec![(4u32, [1.0f32, 0.0, 0.0]), (4, [0.0, 1.0, 0.0])];
    assert!(das_corridas(&c, 8).is_some(), "o CONTROLO: 4 + 4 = 8");
    assert!(
        das_corridas(&c, 9).is_none(),
        "curta de mais: tem de recusar"
    );
    assert!(
        das_corridas(&c, 7).is_none(),
        "comprida de mais: tem de recusar"
    );
    let enorme = vec![(u32::MAX, [0.0f32; 3]); 8];
    assert!(
        das_corridas(&enorme, 8).is_none(),
        "um documento forjado não pode chegar ao `with_capacity`"
    );
}
