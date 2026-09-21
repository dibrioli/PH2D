//! Os gates da aritmética do upload INCREMENTAL da tinta fina — irmão
//! (`#[path]`) do [`super`], e **sem device de propósito**: o que aqui se mede é
//! onde cada byte é escrito, e isso não precisa de placa nenhuma.

use super::corridas_das_sujas;

/// ⭐⭐⭐ **GATE — as corridas cobrem EXACTAMENTE as amostras sujas, e os bytes
/// são os delas.**
///
/// ⛔⛔ **É o único sítio onde um erro desta wave escreve bytes VÁLIDOS no
/// endereço ERRADO**, que é o modo de falha que não estoura, não avisa e pinta
/// a cor de uma amostra noutra. As três metades são independentes:
///
/// 1. **cobertura** — o conjunto de amostras que as corridas alcançam é o
///    conjunto das sujas, nem mais nem menos (*«nem mais»* é o que impede uma
///    corrida esticada de reescrever um vizinho com bytes que por acaso estão
///    certos hoje e errados no dia seguinte);
/// 2. **a unidade é `12` bytes** — três `f32` por amostra, e é ela que faz o
///    offset do device bater com o índice da lei;
/// 3. **agrupar de facto** — sem esta metade uma implementação que devolvesse
///    uma corrida por amostra passaria as duas de cima e mataria o quadro com
///    chamadas de driver, que é o defeito que a função existe para não ter.
#[test]
fn as_corridas_cobrem_exactamente_as_amostras_sujas() {
    // Por ordem de TOQUE e com repetidos — é assim que a janela do traço chega.
    let mut sujas = vec![7, 3, 4, 40, 5, 3, 41, 100, 6];
    let mut out = Vec::new();
    corridas_das_sujas(&mut sujas, &mut out);

    let esperado: Vec<u32> = vec![3, 4, 5, 6, 7, 40, 41, 100];
    let alcancadas: Vec<u32> = out
        .iter()
        .flat_map(|(de, ate)| {
            assert_eq!(de % 12, 0, "a corrida {de}..{ate} não começa numa amostra");
            assert_eq!(ate % 12, 0, "a corrida {de}..{ate} não acaba numa amostra");
            (*de as u32 / 12)..(*ate as u32 / 12)
        })
        .collect();
    assert_eq!(
        alcancadas, esperado,
        "as corridas {out:?} não descrevem as amostras sujas"
    );
    assert_eq!(
        out.len(),
        3,
        "as amostras contíguas não foram agrupadas: {out:?} -- uma corrida por \
         amostra é uma chamada de driver por 12 bytes"
    );
}

/// ⚠️ **Nada sujo, nada escrito** — e o CONTROLO é a metade que a torna uma
/// medição: a mesma função com UMA amostra devolve UMA corrida.
#[test]
fn sem_amostras_sujas_nao_ha_corrida_nenhuma() {
    let mut vazia = Vec::new();
    let mut out = vec![(9usize, 9usize)];
    corridas_das_sujas(&mut vazia, &mut out);
    assert!(out.is_empty(), "uma janela vazia devolveu {out:?}");

    let mut uma = vec![5];
    corridas_das_sujas(&mut uma, &mut out);
    assert_eq!(
        out,
        vec![(60, 72)],
        "CONTROLO: uma amostra só devia dar a corrida dos 12 bytes dela"
    );
}
