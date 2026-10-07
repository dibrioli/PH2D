use super::*;

#[path = "laco_fixtura_tests.rs"]
mod fixtura;

fn verts(vs: &[[[f64; 2]; 3]]) -> Vec<VecVertex> {
    vs.iter()
        .map(|[a, i, o]| {
            let mut x = VecVertex::corner(*a);
            x.in_handle = *i;
            x.out_handle = *o;
            x
        })
        .collect()
}

/// Os cruzamentos próprios do contorno (pares de segmentos).
fn cruzamentos(v: &[VecVertex]) -> Vec<(usize, usize)> {
    let n = v.len();
    (0..n)
        .flat_map(|i| (i + 1..n).map(move |k| (i, k)))
        .filter(|&(i, k)| cruzamento(v, i, k).is_some())
        .collect()
}

/// ⭐⭐ **GATE — o laço que a união deixa sai** (A13): na saída da união do braço a `(24°, 130°)`
/// o segmento `26` corta o `28` (um laço de `0,01`, menor que a bola); depois da passagem o
/// contorno não se cruza e perde só os nós do laço.
#[test]
fn o_laco_que_a_uniao_deixa_sai() {
    let v = verts(fixtura::VERTS);
    assert_eq!(cruzamentos(&v), vec![(26, 28)], "a fixtura perdeu o laço");
    let s = tira_os_lacos(v.clone(), fixtura::RAIO);
    assert!(
        cruzamentos(&s).is_empty(),
        "o laço ficou: {:?}",
        cruzamentos(&s)
    );
    assert_eq!(
        s.len(),
        v.len() - 1,
        "saem os dois nós do laço e entra o cruzamento"
    );
}

/// GATE — sem cruzamento nada muda, AO BIT; e um laço maior que a bola é forma e fica.
#[test]
fn sem_laco_nada_muda_e_o_laco_grande_fica() {
    let v = verts(fixtura::VERTS);
    let limpo = tira_os_lacos(v.clone(), fixtura::RAIO);
    assert_eq!(tira_os_lacos(limpo.clone(), fixtura::RAIO), limpo);
    let pequena = tira_os_lacos(v.clone(), 0.2 * fixtura::RAIO);
    assert_eq!(pequena, v, "com a bola menor que o laço ele fica");
}
