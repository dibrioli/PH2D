//! Gates da **SILHUETA** — a caixa arredondada e a moldura da faixa.
//!
//! ⚠️ Aqui mora o que a FORMA é; que um verbo a CONSUMA é outra pergunta, e tem
//! gates próprios no `verb_strip_tests`. Um kernel correto não prova um produto
//! correto — a lição que esta linha pagou no `l-mode` do Smooth.

use super::*;

/// **`roundness = 1` É A DISTÂNCIA EUCLIDIANA, ao bit** — a âncora que torna a
/// forma nova uma generalização em vez de um segundo caminho.
#[test]
fn a_fully_round_tip_is_the_euclidean_distance_to_the_bit() {
    for i in 0..=20 {
        for j in 0..=20 {
            let (x, y) = (i as f32 / 20.0, j as f32 / 20.0);
            let euclid = (x * x + y * y).sqrt();
            let got = rounded_box(x, y, 1.0);
            // Fora da caixa a forma satura em 1 de propósito (ver o doc); a
            // identidade é afirmada onde as duas respondem.
            if x > 1.0 || y > 1.0 {
                continue;
            }
            assert_eq!(
                got,
                euclid.min(1.0),
                "({x}, {y}) devia ser euclidiano e deu {got}"
            );
        }
    }
}

/// **O MIOLO É CHATO, e é ele que separa uma faixa de um domo.**
#[test]
fn the_core_of_a_square_tip_is_flat_and_the_rim_is_a_step() {
    // Quina viva: tudo dentro vale 0, tudo fora vale 1.
    assert_eq!(rounded_box(0.0, 0.0, 0.0), 0.0);
    assert_eq!(rounded_box(0.99, 0.99, 0.0), 0.0);
    assert_eq!(rounded_box(1.01, 0.0, 0.0), 1.0);
    // Meio arredondada: o quadrado de meio-lado 0,5 é chato, e a partir dele
    // cresce.
    assert_eq!(rounded_box(0.4, 0.4, 0.5), 0.0);
    assert!(rounded_box(0.75, 0.0, 0.5) > 0.0);
    assert!((rounded_box(1.0, 0.0, 0.5) - 1.0).abs() < 1e-6);
}

/// **A QUINA É UM ARCO, não um bico** — a distância no canto tem de ser MAIOR
/// que a do lado à mesma coordenada, senão a "quina arredondada" é um quadrado
/// com outro nome.
#[test]
fn the_corner_is_rounded_not_mitred() {
    let side = rounded_box(0.8, 0.0, 0.5);
    let corner = rounded_box(0.8, 0.8, 0.5);
    assert!(
        corner > side * 1.3,
        "a quina ({corner:.4}) tinha de ficar mais longe que o lado ({side:.4})"
    );
}

/// **SEM DIREÇÃO A FAIXA NASCE REDONDA** — o caminho decide a ORIENTAÇÃO da
/// caixa e mais nada, e a profundidade é fato do PLANO.
///
/// ⚠️ **A 1ª versão deste gate afirmava o oposto** (*"sem direção ela recusa
/// existir"*), e o produto pagava por isso: o toque caía no
/// [`Footprint::Disc`], que não tem portão de profundidade, e depositava
/// `0,039998` de barro onde a lei manda **zero**. Uma ponta redonda não sabe
/// para onde os eixos apontam, então escolher um perpendicular qualquer não é
/// inventar orientação — é escolher entre respostas iguais.
#[test]
fn a_strip_without_a_path_is_born_round() {
    let n = [0.0, 0.0, 1.0];
    let round = |path: [f32; 3]| {
        let s = Strip::new([0.0; 3], n, path, 1.0, 3.0, 0.0).expect("há plano");
        // A meio raio abaixo do plano, onde o portão abre: numa ponta redonda o
        // `t` só depende da distância, então dois pontos à mesma distância em
        // eixos diferentes têm de dar o MESMO número.
        let a = s.at([0.7, 0.0, -0.5]).0;
        let b = s.at([0.0, 0.7, -0.5]).0;
        (a, b)
    };
    for path in [[0.0; 3], [0.0, 0.0, 2.0]] {
        let (a, b) = round(path);
        assert_eq!(a, b, "sem caminho ({path:?}) a pegada tem de ser redonda");
        assert!(a > 0.0 && a < 1.0, "e viva: {a}");
    }
    // Com caminho, a MESMA faixa (comprimento 3, quina viva) já distingue — e as
    // sondas têm de estar ALÉM da meia-largura, senão as duas caem no miolo
    // chato e o `assert_ne!` compara `0,0` com `0,0` (foi o que aconteceu na 1ª
    // corrida: uma fixture que não continha o fenômeno).
    let s = Strip::new([0.0; 3], n, [1.0, 0.0, 0.0], 1.0, 3.0, 0.0).expect("há plano");
    assert_eq!(
        s.at([1.5, 0.0, -0.5]).0,
        0.0,
        "um raio e meio AO LONGO ainda é miolo numa faixa de comprimento 3"
    );
    assert_eq!(
        s.at([0.0, 1.5, -0.5]).0,
        1.0,
        "e um raio e meio ATRAVESSADO está fora"
    );
    // Sem PLANO não há nem caixa nem profundidade — o `None` que sobra.
    assert!(Strip::new([0.0; 3], [0.0; 3], [1.0, 0.0, 0.0], 1.0, 1.0, 1.0).is_none());
}

/// **A FAIXA DEITA NA DIREÇÃO DO TRAÇO**: um ponto ao LADO do caminho sai da
/// pegada antes de um ponto À FRENTE dele, quando ela é mais longa que larga.
#[test]
fn the_strip_lies_along_the_path() {
    let n = [0.0, 0.0, 1.0];
    let s = Strip::new([0.0; 3], n, [1.0, 0.0, 0.0], 1.0, 3.0, 0.0).expect("há direção");
    // Meio raio ABAIXO do plano, onde a parábola de profundidade abre.
    let z = -0.5;
    let ahead = s.at([2.0, 0.0, z]);
    let beside = s.at([0.0, 2.0, z]);
    assert_eq!(
        ahead.0, 0.0,
        "dois raios À FRENTE ainda é miolo numa faixa de comprimento 3"
    );
    assert_eq!(
        beside.0, 1.0,
        "dois raios AO LADO está fora de uma faixa de meia-largura 1"
    );
}

/// **A PROFUNDIDADE É UM PORTÃO PARABÓLICO**: zero na superfície do plano, pico
/// a meio raio abaixo, zero um raio abaixo, e **nada acima do plano**.
#[test]
fn the_depth_gate_is_a_parabola_that_ignores_what_is_above_the_plane() {
    let s =
        Strip::new([0.0; 3], [0.0, 0.0, 1.0], [1.0, 0.0, 0.0], 1.0, 1.0, 0.5).expect("há direção");
    let gate = |z: f32| s.at([0.0, 0.0, z]).1;
    assert_eq!(gate(0.5), 0.0, "acima do plano não é barro desta passada");
    assert_eq!(gate(0.0), 0.0, "na superfície o portão abre em zero");
    let peak = gate(-0.5);
    // ⚠️ **A expectativa é DERIVADA da calibração, e não a função sob teste:** o
    // pico da parábola crua vale `0,25`, e o [`crate::STRIP_DEPTH_GAIN`] o repõe
    // onde a superfície em repouso está. Um literal aqui apodreceria no dia em
    // que o [`crate::STRIP_PLANE_FRACTION`] se movesse.
    let want = crate::STRIP_DEPTH_GAIN * 0.25;
    assert!(
        (peak - want).abs() < 1e-6,
        "o pico da parábola vale {want} a meio raio: {peak}"
    );
    assert_eq!(gate(-1.0), 0.0, "um raio abaixo ele fecha");
    assert_eq!(gate(-2.0), 0.0, "e continua fechado");
}

/// **UMA CAIXA NÃO CABE NO CÍRCULO QUE A INSCREVE** — o fator de consulta é o
/// que impede as quinas de serem comidas pela esfera da pegada.
#[test]
fn the_query_grows_to_hold_the_corners() {
    assert_eq!(Footprint::Disc.query_factor(), 1.0);
    let s =
        Strip::new([0.0; 3], [0.0, 0.0, 1.0], [1.0, 0.0, 0.0], 1.0, 1.0, 0.0).expect("há direção");
    let f = Footprint::Strip(s).query_factor();
    assert!(
        (f - 2.0f32.sqrt()).abs() < 1e-5,
        "uma faixa quadrada alcança √2 raios na quina: {f}"
    );
    let long =
        Strip::new([0.0; 3], [0.0, 0.0, 1.0], [1.0, 0.0, 0.0], 1.0, 3.0, 0.0).expect("há direção");
    let fl = Footprint::Strip(long).query_factor();
    assert!(
        (fl - 10.0f32.sqrt()).abs() < 1e-4,
        "uma faixa de comprimento 3 alcança √10: {fl}"
    );
}

/// **O DISCO ATRAVESSA ESTA CAMADA BYTE A BYTE** — a âncora de identidade.
#[test]
fn the_disc_is_the_distance_it_was_given() {
    for i in 0..50 {
        let dist = i as f32 * 0.037;
        let inv_r = 1.0 / 0.4;
        assert_eq!(
            Footprint::Disc.at([9.0, -3.0, 1.0], dist, inv_r),
            (dist * inv_r, 1.0)
        );
    }
}
