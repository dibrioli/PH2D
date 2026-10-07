use super::*;
use ph2d_vec_scene::VecPath;

#[path = "laco_fixtura_tests.rs"]
mod fixtura;

#[path = "laco_silhueta_fixtura_tests.rs"]
mod silhueta;

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

/// ⭐⭐ **GATE — a bola do laço tem o raio INTEIRO.** Um rabo-de-porco no topo de um rectângulo:
/// o segmento que desce de `(2,2; 3,25)` corta o topo em `(2,2; 3)`, e o laço vai até `0,472` do
/// cruzamento — entre MEIO raio (`0,3`) e o raio (`0,6`). Ele sai; com meio raio ficava.
#[test]
fn um_laco_entre_meio_raio_e_o_raio_sai() {
    let v: Vec<VecVertex> = [
        [0.0, 0.0],
        [4.0, 0.0],
        [4.0, 3.0],
        [2.0, 3.0],
        [1.8, 3.0],
        [1.8, 3.25],
        [2.2, 3.25],
        [2.2, 2.8],
        [0.0, 2.8],
    ]
    .iter()
    .map(|p| VecVertex::corner(*p))
    .collect();
    let raio = 0.6;
    let x = [2.2, 3.0];
    let alcance = [[2.0, 3.0], [1.8, 3.0], [1.8, 3.25], [2.2, 3.25]]
        .iter()
        .map(|p: &[f64; 2]| (p[0] - x[0]).hypot(p[1] - x[1]))
        .fold(0.0, f64::max);
    assert!(
        alcance > 0.5 * raio && alcance < raio,
        "a fixtura saiu da faixa: {alcance}"
    );
    assert_eq!(cruzamentos(&v), vec![(2, 6)], "a fixtura perdeu o laço");
    let s = tira_os_lacos(v, raio);
    assert!(
        cruzamentos(&s).is_empty(),
        "o laço de {alcance} ficou com a bola de {raio}"
    );
}

/// ⭐⭐ **GATE — a SILHUETA corre a passagem** (o fio, A13). O gate de cima chama a função; este lê a
/// porta que o desenho chama: na entrada do braço a `(22°, 130°)` a união deixa um laço (o CONTROLO) e
/// a [`crate::silhueta_da_pele`] sai sem cruzamento nenhum.
#[test]
fn a_silhueta_tira_o_laco_que_a_uniao_deixa() {
    let p = VecPath {
        verts: verts(silhueta::VERTS),
        closed: true,
        ..VecPath::default()
    };
    let u = crate::resolve_overlap(&p).expect("os membros cruzam-se");
    assert!(
        !cruzamentos(&u.verts).is_empty(),
        "a união deixou de deixar o laço — a fixtura não mede o fio"
    );
    let s = crate::silhueta_da_pele(&p, silhueta::QUINAS).expect("silhueta");
    assert!(
        cruzamentos(&s.verts).is_empty(),
        "a silhueta saiu com o laço da união: {:?}",
        cruzamentos(&s.verts)
    );
}
