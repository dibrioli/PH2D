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

/// ⭐⭐ **GATE — a corrida das ALTURAS é a MESMA faixa de amostras** — `4`
/// bytes por amostra em vez de `12`. Um erro aqui escreve a espessura de uma
/// amostra na vizinha, com bytes válidos (`docs/3D/29`).
#[test]
fn a_corrida_das_alturas_e_a_mesma_faixa_de_amostras() {
    let mut sujas = vec![3, 4, 5, 40];
    let mut out = Vec::new();
    corridas_das_sujas(&mut sujas, &mut out);
    let alturas: Vec<(usize, usize)> = out.iter().map(|&c| super::em_alturas(c)).collect();
    assert_eq!(alturas, vec![(12, 24), (160, 164)], "de {out:?}");
    for ((de, ate), (ad, aa)) in out.iter().zip(&alturas) {
        assert_eq!(de / 12, ad / 4, "o início não é a mesma amostra");
        assert_eq!(ate / 12, aa / 4, "o fim não é a mesma amostra");
    }
}

/// ⭐⭐ **GATE — o bit do relevo só arma com relevo, e é o MESMO nas duas
/// pontas.** O CONTROLO (sem relevo ⇒ `armado == 1`, byte a byte o de antes)
/// é a metade que prova que uma peça sem impasto desenha como sempre.
#[test]
fn o_bit_do_relevo_e_o_mesmo_nas_duas_pontas() {
    use ph2d_mesh_colors::Tinta;
    let faces: Vec<Vec<u32>> = vec![vec![0, 1, 2, 3]];
    let it = || faces.iter().map(|f| &f[..]);

    let mut t = Tinta::nova(4, it(), 2);
    assert_eq!(
        super::cfg_de(Some(&t))[3],
        1,
        "CONTROLO: sem relevo arma só a cor"
    );
    t.alturas_mut()[0] = 0.25;
    assert_eq!(
        super::cfg_de(Some(&t))[3],
        1 | super::RELEVO,
        "com relevo o bit tem de ARMAR"
    );

    let agulha = format!("const TINTA_RELEVO: u32 = {}u;", super::RELEVO);
    assert_eq!(
        crate::fonte::TINTA_WGSL.matches(&agulha).count(),
        1,
        "o shader não declara o mesmo bit que a CPU escreve ({agulha})"
    );
}
