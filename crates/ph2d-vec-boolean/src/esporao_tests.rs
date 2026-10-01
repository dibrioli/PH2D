use super::*;

type P = [f64; 2];

fn v(a: P, i: P, o: P) -> VecVertex {
    let mut x = VecVertex::corner(a);
    x.in_handle = i;
    x.out_handle = o;
    x
}

/// Um quadrado cujo lado de baixo vai até `x = 1` e VOLTA a `x = 1 - recuo` antes de subir — a forma
/// medida a `(0°, 174°)`: um segmento recto que refaz o fim do anterior, com o nó da ponta a virar
/// `180°`.
fn com_esporao(recuo: f64) -> Vec<VecVertex> {
    vec![
        v([0.0, 0.0], [0.0, 0.0], [0.0, 0.0]),
        v([1.0, 0.0], [1.0, 0.0], [1.0, 0.0]),
        v([1.0 - recuo, 0.0], [1.0 - recuo, 0.0], [1.0 - recuo, 0.0]),
        v([1.0 - recuo, 1.0], [1.0 - recuo, 1.0], [1.0 - recuo, 1.0]),
        v([0.0, 1.0], [0.0, 1.0], [0.0, 1.0]),
    ]
}

fn viragem_maxima(vs: &[VecVertex]) -> f64 {
    (0..vs.len())
        .filter_map(|i| crate::overlap::viragem_do_vertice(vs, i))
        .fold(0.0, f64::max)
}

/// ⭐ **GATE — o esporão sai, seja de que tamanho for, e o que fica é o mesmo conjunto de pontos.**
/// O CONTROLO é a fixtura: antes do passe a ponta vira `180°`. ⚠️ O recuo de `0,3` é `86×` a `tol`
/// e `~30×` o tecto do desfazer dos ganchos — é a metade que o distingue dele.
#[test]
fn o_esporao_sai_de_qualquer_tamanho() {
    for recuo in [0.01, 0.06, 0.3] {
        let antes = com_esporao(recuo);
        assert!(
            viragem_maxima(&antes) > 170.0,
            "a fixtura deixou de conter o esporão a {recuo}"
        );
        let depois = tira_os_esporoes(antes, 0.0035);
        assert_eq!(depois.len(), 4, "o nó da ponta não saiu a {recuo}");
        assert!(
            viragem_maxima(&depois) < 91.0,
            "ficou uma meia-volta a {recuo}: {:.1}",
            viragem_maxima(&depois)
        );
        // O lado de baixo acaba onde o esporão acabava — a forma desenhada não se mexe.
        let fim = depois[1].anchor;
        assert!(
            (fim[0] - (1.0 - recuo)).abs() < 1e-12 && fim[1].abs() < 1e-12,
            "o lado de baixo foi parar a {fim:?}"
        );
    }
}

/// ⭐ **GATE — o que NÃO é esporão fica, ao bit.** Uma meia-volta cujo regresso se AFASTA do caminho
/// de ida (um dente estreito com largura, não uma linha refeita) é forma, e um quadrado não tem
/// nada a tirar.
#[test]
fn o_que_nao_e_esporao_fica_intacto() {
    let quadrado = vec![
        v([0.0, 0.0], [0.0, 0.0], [0.0, 0.0]),
        v([1.0, 0.0], [1.0, 0.0], [1.0, 0.0]),
        v([1.0, 1.0], [1.0, 1.0], [1.0, 1.0]),
        v([0.0, 1.0], [0.0, 1.0], [0.0, 1.0]),
    ];
    assert_eq!(tira_os_esporoes(quadrado.clone(), 0.0035), quadrado);
    // Um dente: vai a `x = 1,5` em `y = 0` e volta em `y = 0,02` — volta a `0,02` do caminho,
    // `~6×` a tol, logo é um dente fino de verdade.
    let dente = vec![
        v([0.0, 0.0], [0.0, 0.0], [0.0, 0.0]),
        v([1.5, 0.0], [1.5, 0.0], [1.5, 0.0]),
        v([1.0, 0.02], [1.0, 0.02], [1.0, 0.02]),
        v([1.0, 1.0], [1.0, 1.0], [1.0, 1.0]),
        v([0.0, 1.0], [0.0, 1.0], [0.0, 1.0]),
    ];
    assert!(viragem_maxima(&dente) > 170.0, "a fixtura deixou de virar");
    assert_eq!(tira_os_esporoes(dente.clone(), 0.0035), dente);
}

/// ⭐ **GATE — o esporão de ENTRADA também sai** (o curto é o segmento que CHEGA à ponta).
#[test]
fn o_esporao_de_entrada_sai() {
    let mut vs = com_esporao(0.06);
    vs.reverse();
    assert!(viragem_maxima(&vs) > 170.0);
    let depois = tira_os_esporoes(vs, 0.0035);
    assert_eq!(depois.len(), 4);
    assert!(viragem_maxima(&depois) < 91.0);
}
