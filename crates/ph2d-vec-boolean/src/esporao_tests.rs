use super::*;

fn tira_os_esporoes_sem_quinas(vs: Vec<VecVertex>, tol: f64) -> Vec<VecVertex> {
    tira_os_esporoes(vs, &[], tol)
}

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
        let depois = tira_os_esporoes_sem_quinas(antes, 0.0035);
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
    assert_eq!(
        tira_os_esporoes_sem_quinas(quadrado.clone(), 0.0035),
        quadrado
    );
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
    assert_eq!(tira_os_esporoes_sem_quinas(dente.clone(), 0.0035), dente);
}

/// ⭐ **GATE — o esporão de ENTRADA também sai** (o curto é o segmento que CHEGA à ponta).
#[test]
fn o_esporao_de_entrada_sai() {
    let mut vs = com_esporao(0.06);
    vs.reverse();
    assert!(viragem_maxima(&vs) > 170.0);
    let depois = tira_os_esporoes_sem_quinas(vs, 0.0035);
    assert_eq!(depois.len(), 4);
    assert!(viragem_maxima(&depois) < 91.0);
}

/// Uma CUNHA cujo lado de baixo é UMA cúbica que passa pelo nó `(1, 0)`, segue `alem` para lá dele e
/// volta pela mesma recta, e cujo lado de cima volta para `(0, 0,4)` — a forma medida a
/// `(36°, −144°)`. O nó vira `21,8°` pelas tangentes que o traço lê; a quina escondida é de `158°`. ⚠️ Os pontos de controlo da cúbica
/// são `(0,0) · (0,5, 0) · (1 + alem, 0) · (1, 0)`: o recuo é `~0,1·alem`, não `alem`.
fn com_ponta_alem(alem: f64) -> Vec<VecVertex> {
    vec![
        v([0.0, 0.0], [0.0, 0.0], [0.5, 0.0]),
        v([1.0, 0.0], [1.0 + alem, 0.0], [1.0, 0.0]),
        v([0.0, 0.4], [0.0, 0.4], [0.0, 0.4]),
    ]
}

/// A cúbica `j` amostrada passa ALÉM do nó de chegada, na direcção `x`?
fn passa_alem(vs: &[VecVertex], j: usize) -> f64 {
    let c = cubica(vs, j);
    (0..=256)
        .map(|k| c.eval(f64::from(k) / 256.0).x)
        .fold(f64::MIN, f64::max)
        - c.p3.x
}

/// ⭐ **GATE — o esporão DENTRO da cúbica sai**: a cúbica deixa de passar além do nó, e o nó
/// continua onde estava. ⚠️ O CONTROLO: um recuo que SAI da recta (uma cúbica que vai além e volta
/// por OUTRO caminho, `0,02` acima) é forma e fica, ao bit.
#[test]
fn o_esporao_dentro_da_cubica_sai() {
    let antes = com_ponta_alem(0.1);
    assert!(
        passa_alem(&antes, 0) > 5e-3,
        "a fixtura deixou de passar além"
    );
    let depois = tira_os_esporoes_sem_quinas(antes.clone(), 0.0035);
    assert!(
        passa_alem(&depois, 0) < 1e-9,
        "a cúbica ainda passa {:.2e} além do nó",
        passa_alem(&depois, 0)
    );
    assert_eq!(depois[1].anchor, antes[1].anchor, "o nó mexeu-se");
    let mut fora = com_ponta_alem(0.1);
    fora[1].in_handle = [1.1, 0.2];
    assert!(
        passa_alem(&fora, 0) > 5e-3,
        "o controlo deixou de passar além"
    );
    assert_eq!(tira_os_esporoes_sem_quinas(fora.clone(), 0.0035), fora);
    // ⚠️ E um LAÇO que passa EXACTAMENTE pelo nó de chegada (em `t = 0,5 − √0,15`) e volta a ele por
    // outro caminho — a única forma de exercitar a dobra sem que a passagem pelo nó já recuse: a
    // cúbica é o pedaço `[0, 0,5 + √0,15]` de `(0,0)·(2,1)·(−1,1)·(1,0)`, cortado no próprio cruzamento.
    let laco = vec![
        v(
            [0.0, 0.0],
            [0.0, 0.0],
            [1.774_596_669_241_483_4, 0.887_298_334_620_741_7],
        ),
        v(
            [0.5, 0.3],
            [-0.387_298_334_620_741_8, 0.987_298_334_620_741_7],
            [0.5, 0.3],
        ),
        v([-0.35, -0.25], [-0.35, -0.25], [-0.35, -0.25]),
    ];
    assert_eq!(tira_os_esporoes_sem_quinas(laco.clone(), 0.0035), laco);
    // ⚠️ E a quina escondida MANSA fica (medido a `(166°, −40°)`, `18°`): abaixo do limiar do gancho a
    // bola já a arredonda, e aparar trocava esse arco por um bico.
    let mut mansa = com_ponta_alem(0.1);
    // A saída do nó continua quase em frente (`18°` acima da chegada).
    mansa[2].anchor = [2.0, 0.325];
    mansa[2].in_handle = [2.0, 0.325];
    mansa[2].out_handle = [2.0, 0.325];
    assert!(
        passa_alem(&mansa, 0) > 5e-3,
        "o controlo manso deixou de passar além"
    );
    assert_eq!(tira_os_esporoes_sem_quinas(mansa.clone(), 0.0035), mansa);
}

/// ⭐ **GATE — e no INÍCIO da cúbica também** (o mesmo contorno percorrido ao contrário: o pedaço a
/// mais fica a seguir ao nó de partida).
#[test]
fn o_esporao_no_inicio_da_cubica_sai() {
    let mut vs = com_ponta_alem(0.1);
    vs.reverse();
    for x in &mut vs {
        std::mem::swap(&mut x.in_handle, &mut x.out_handle);
    }
    // O lado de baixo é agora a cúbica `1 → 2`, de `(1, 0)` para `(0, 0)`.
    let alem = |vs: &[VecVertex]| {
        let c = cubica(vs, 1);
        (0..=256)
            .map(|k| c.eval(f64::from(k) / 256.0).x)
            .fold(f64::MIN, f64::max)
            - c.p0.x
    };
    assert!(alem(&vs) > 5e-3, "a fixtura deixou de passar além");
    let depois = tira_os_esporoes_sem_quinas(vs, 0.0035);
    assert!(
        alem(&depois) < 1e-9,
        "a cúbica ainda passa {:.2e} além",
        alem(&depois)
    );
}

/// ⭐ **GATE — a ponta que o artista DESENHOU fica** (um bigode traçado em ida e volta, ou uma cúbica
/// que ele fez passar além do nó): com o nó da ponta na lista de quinas — a viragem de repouso dele
/// é a de agora —, nem o esporão nem o aparar lhe tocam, ao bit. O CONTROLO são os gates de cima: sem
/// a lista, as mesmas fixturas saem.
#[test]
fn a_ponta_desenhada_fica() {
    let bigode = com_esporao(0.06);
    let quinas: Vec<([f64; 2], f64)> = (0..bigode.len())
        .map(|i| {
            (
                bigode[i].anchor,
                crate::overlap::viragem_do_vertice(&bigode, i).unwrap_or(0.0),
            )
        })
        .collect();
    assert_eq!(tira_os_esporoes(bigode.clone(), &quinas, 0.0035), bigode);
    let alem = com_ponta_alem(0.1);
    let quinas = [(alem[1].anchor, 21.8)];
    assert_eq!(tira_os_esporoes(alem.clone(), &quinas, 0.0035), alem);
}
