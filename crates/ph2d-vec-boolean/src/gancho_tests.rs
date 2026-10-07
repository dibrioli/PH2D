use super::*;

fn v(a: P, i: P, o: P) -> VecVertex {
    let mut x = VecVertex::corner(a);
    x.in_handle = i;
    x.out_handle = o;
    x
}

/// Um quadrado cujo lado direito tem, a meio, o nó do gancho: a cúbica seguinte nasce com a alça
/// em cima do nó e a outra `atras` ATRÁS dele — a forma medida na cena `=4` a `(125°, 34°)`.
fn com_gancho(atras: f64) -> Vec<VecVertex> {
    vec![
        v([0.0, 0.0], [0.0, 0.0], [0.0, 0.0]),
        v([1.0, 0.0], [1.0, 0.0], [1.0, 0.0]),
        v([1.0, 0.5], [1.0, 0.4], [1.0, 0.5]),
        v([1.0, 1.0], [1.0, 0.5 - atras], [1.0, 1.0]),
        v([0.0, 1.0], [0.0, 1.0], [0.0, 1.0]),
    ]
}

fn viragens(vs: &[VecVertex]) -> Vec<f64> {
    (0..vs.len())
        .map(|i| crate::overlap::viragem_do_vertice(vs, i).unwrap_or(0.0))
        .collect()
}

/// ⭐ **GATE — a inversão que só existe na TANGENTE sai; um recuo que se VÊ fica; uma quina do
/// artista fica.** O CONTROLO é a fixtura: antes do passe o nó vira `180°`.
#[test]
fn o_gancho_microscopico_sai_e_o_que_se_ve_fica() {
    let fino = com_gancho(0.006);
    assert!(
        viragens(&fino)[2] > 170.0,
        "a fixtura deixou de conter o gancho"
    );
    let curado = desfaz_os_ganchos(fino.clone(), &[], 0.0035);
    assert!(
        viragens(&curado)[2] < 1.0,
        "o gancho de 0,006 continua: {:?}",
        viragens(&curado)
    );
    // Um recuo de MEIO lado é forma — a troca mudá-la-ia mais que a tolerância.
    let grande = com_gancho(0.5);
    assert_eq!(desfaz_os_ganchos(grande.clone(), &[], 0.0035), grande);
    // A mesma inversão, declarada quina do artista com a viragem dela, fica.
    let quinas = [([1.0, 0.5], 180.0)];
    assert_eq!(desfaz_os_ganchos(fino.clone(), &quinas, 0.0035), fino);
    // Sem gancho nenhum, o contorno sai ao bit.
    let liso = com_gancho(-0.1);
    assert_eq!(desfaz_os_ganchos(liso.clone(), &[], 0.0035), liso);
}

/// ⭐ **GATE — o LAÇO por dentro de uma cúbica sai** (medido a `(110°, 17,5°)`): as duas alças
/// cruzam-se — a 1.ª passa à frente do fim, a 2.ª fica atrás do início —, e a curva vai, volta e vai
/// sem que nenhum NÓ vire. O CONTROLO: nenhum nó vira mais de `1°` e a cúbica dobra por dentro.
#[test]
fn o_laco_dentro_da_cubica_sai() {
    let laco = vec![
        v([0.0, 0.0], [0.0, 0.0], [0.0, 0.0]),
        v([1.0, 0.0], [1.0, 0.0], [1.0, 0.0]),
        v([1.0, 0.5], [1.0, 0.45], [1.0, 0.503]),
        v([1.0, 0.5015], [1.0, 0.4985], [1.0, 0.6]),
        v([1.0, 1.0], [1.0, 0.9], [1.0, 1.0]),
        v([0.0, 1.0], [0.0, 1.0], [0.0, 1.0]),
    ];
    assert!(viragens(&laco)[2] < 1.0 && viragens(&laco)[3] < 1.0);
    let c = [
        laco[2].anchor,
        laco[2].out_handle,
        laco[3].in_handle,
        laco[3].anchor,
    ];
    assert!(dobra(None, &c, None), "a fixtura deixou de ter o laço");
    let curado = desfaz_os_ganchos(laco.clone(), &[], 0.0035);
    let c2 = [
        curado[2].anchor,
        curado[2].out_handle,
        curado[3].in_handle,
        curado[3].anchor,
    ];
    assert!(!dobra(None, &c2, None), "o laço continua");
}

/// ⭐ **GATE — o recuo que só existe ABAIXO DA AMOSTRAGEM sai** (F43-bis, medido a `(84°, −10,5°)`, a
/// foto do dono): a 2.ª alça em cima do nó e a 1.ª `0,0009` além dele — a cúbica recua só no fim do
/// parâmetro (`s < 0,0054` aqui). O CONTROLO tem duas metades: o nó vira `180°`, e os passos das
/// amostras NÃO o veem (nenhum par seguido vira mais que o limiar) — sem as tangentes exactas das
/// pontas, o passe deixava-o ficar.
#[test]
fn o_recuo_mais_fino_que_a_amostragem_sai() {
    let recuo = vec![
        v([0.0, 0.0], [0.0, 0.0], [0.0, 0.0]),
        v([1.0, 0.0], [1.0, 0.0], [1.0, 0.5009]),
        v([1.0, 0.5], [1.0, 0.5], [1.0, 0.6]),
        v([1.0, 1.0], [1.0, 0.9], [1.0, 1.0]),
        v([0.0, 1.0], [0.0, 1.0], [0.0, 1.0]),
    ];
    assert!(
        viragens(&recuo)[2] > 170.0,
        "a fixtura deixou de ter o recuo"
    );
    let c = [
        recuo[1].anchor,
        recuo[1].out_handle,
        recuo[2].in_handle,
        recuo[2].anchor,
    ];
    let pts = amostras(&c);
    let passos: Vec<P> = pts.windows(2).filter_map(|w| dir(w[0], w[1])).collect();
    let limiar = VIRAGEM_DO_GANCHO.to_radians().cos();
    assert!(
        passos
            .windows(2)
            .all(|w| w[0][0] * w[1][0] + w[0][1] * w[1][1] >= limiar),
        "as amostras já veem o recuo — a fixtura deixou de medir o que é mais fino que elas"
    );
    let curado = desfaz_os_ganchos(recuo, &[], 0.0035);
    assert!(
        viragens(&curado)[2] < 1.0,
        "o recuo continua: {:?}",
        viragens(&curado)
    );
}

/// Um quadrado cujo lado direito chega ao nó do meio PASSANDO dele: a 2.ª alça fica `e` além do nó
/// e `sx·e` para o lado. Com `sx = 0` o pedaço a mais fica em cima do segmento seguinte.
fn com_recuo(sx: f64, e: f64) -> Vec<VecVertex> {
    vec![
        v([0.0, 0.0], [0.0, 0.0], [0.0, 0.0]),
        v([1.0, 0.0], [1.0, 0.0], [1.0, 0.1]),
        v([1.0, 0.5], [1.0 + sx * e, 0.5 + e], [1.0, 0.6]),
        v([1.0, 1.0], [1.0, 0.9], [1.0, 1.0]),
        v([0.0, 1.0], [0.0, 1.0], [0.0, 1.0]),
    ]
}

/// ⭐ **GATE — o recuo SOBRE O PRÓPRIO CAMINHO sai, até ao tamanho da bola** (F45, medido a
/// `(166°, 74°)`: a união deixa uma cúbica que passa do nó `0,0059` — `1,4×` a solda — e volta
/// pela mesma recta). Três recuos de régua conhecida (a cúbica sozinha muda `2,5×`, `16×` e `4,6×`
/// a tolerância): o que volta pela recta sai; o maior que a bola fica; e o que sai DA recta, com a
/// mesma ordem de grandeza, fica — ele muda o desenho, e é isso que separa os dois.
#[test]
fn o_recuo_sobre_o_proprio_caminho_sai_ate_ao_tamanho_da_bola() {
    let tol = 0.0035;
    let sobre_si = com_recuo(0.0, 0.08);
    assert!(
        viragens(&sobre_si)[2] > 170.0,
        "a fixtura deixou de ter o recuo"
    );
    let curado = desfaz_os_ganchos(sobre_si, &[], tol);
    assert!(
        viragens(&curado)[2] < 1.0,
        "o recuo sobre si ficou: {:?}",
        viragens(&curado)
    );
    let grande = com_recuo(0.0, 0.25);
    assert_eq!(desfaz_os_ganchos(grande.clone(), &[], tol), grande);
    let de_lado = com_recuo(0.45, 0.08);
    assert!(
        viragens(&de_lado)[2] > VIRAGEM_DO_GANCHO,
        "o controlo deixou de ser um gancho"
    );
    assert_eq!(desfaz_os_ganchos(de_lado.clone(), &[], tol), de_lado);
}

#[path = "gancho_esporao_fixtura_tests.rs"]
mod esporao;

/// ⭐⭐ **GATE — desfazer um gancho nunca acrescenta um cruzamento** (A13). Na saída da união do braço
/// a `(160°, 134°)` (fixtura despejada), a ponta de um esporão vira `171,6°`; a Hermite que parte
/// pela tangente de chegada dava a volta e cortava o flanco de ida (um laço que nenhuma passagem
/// seguinte tira — o dente de `178,4°` da silhueta). O contorno sai sem cruzar-se, como entrou.
#[test]
fn desfazer_um_gancho_nunca_acrescenta_um_cruzamento() {
    let vs: Vec<VecVertex> = esporao::VERTS
        .iter()
        .map(|[a, i, o]| v(*a, *i, *o))
        .collect();
    let cubicas = |x: &[VecVertex]| -> Vec<Vec<P>> {
        let n = x.len();
        (0..n)
            .map(|j| {
                let jb = (j + 1) % n;
                amostras(&[x[j].anchor, x[j].out_handle, x[jb].in_handle, x[jb].anchor])
            })
            .collect()
    };
    let auto = |x: &[VecVertex]| -> usize {
        let cs = cubicas(x);
        (0..cs.len())
            .map(|j| {
                let resto: Vec<Vec<P>> = (0..cs.len())
                    .filter(|&i| i != j)
                    .map(|i| cs[i].clone())
                    .collect();
                cruzamentos(&cs[j], &resto)
            })
            .sum()
    };
    assert_eq!(auto(&vs), 0, "a saída da união não se cruza");
    let depois = desfaz_os_ganchos(vs, esporao::QUINAS, esporao::SOLDA);
    assert_eq!(
        auto(&depois),
        0,
        "o desfazer dos ganchos cruzou o contorno — o laço na ponta do esporão"
    );
}
