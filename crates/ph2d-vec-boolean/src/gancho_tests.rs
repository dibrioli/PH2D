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
