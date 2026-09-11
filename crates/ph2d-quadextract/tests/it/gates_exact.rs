//! **GATE 5 — o predicado de orientação concorda com o exacto em casos
//! adversariais** — mais a álgebra das transições, de que tudo o resto depende.
//!
//! ⚠️ **Esta crate não tem filtro rápido para desistir**, e por isso o gate muda de
//! forma sem baixar de barra: o predicado **é** a conta exacta (um determinante
//! `i128` sobre um domínio quantizado, ver `ph2d_quadextract::exact`), e o que se
//! prova aqui é que ele **acerta onde uma avaliação em `f64` erra** — que é a
//! propriedade pela qual a lei «nada de epsilon» existe.

use ph2d_quadextract::exact::{CARDINALS, P, Xf, area2, orient, same_sense, side_of_ray};

/// A mesma conta, em `f64` — o controlo que mostra o que se perde sem o exacto.
fn orient_f64(a: P, b: P, c: P) -> i8 {
    #[allow(clippy::cast_precision_loss)]
    let f = |p: P| [p[0] as f64, p[1] as f64];
    let (a, b, c) = (f(a), f(b), f(c));
    let d = (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0]);
    if d > 0.0 {
        1
    } else if d < 0.0 {
        -1
    } else {
        0
    }
}

#[test]
fn o_predicado_e_exacto_onde_o_f64_ja_nao_e() {
    // ⭐⭐ **A construção é um CANCELAMENTO, não uma quase-colinearidade qualquer.**
    //
    // ⛔ A primeira redacção deste gate punha três pontos quase alinhados numa recta
    // diagonal, e o controlo em `f64` **acertou nos 36 casos** — a fixtura não
    // continha o fenómeno, e um gate assim ficaria verde para sempre sobre um
    // predicado errado. *Um controlo positivo que nunca falha não é um controlo.*
    //
    // O que de facto quebra o `f64` é dois produtos **enormes e quase iguais**:
    // `(2^k+1)(2^k−1) − 2^k·2^k = −1`. Cada produto passa de `2^53` e arredonda para
    // o mesmo valor; a diferença deles em `f64` é **zero**, e a resposta certa é
    // `−1`. É exactamente a forma que um ponto de grade sobre uma aresta tem.
    let mut disagreements = 0usize;
    let mut cases = 0usize;
    for k in 27..53u32 {
        let s = 1i64 << k;
        let a: P = [0, 0];
        for sign in [1i64, -1] {
            let b: P = [s + sign, s];
            let c: P = [s, s - sign];
            cases += 1;
            assert_eq!(
                area2(a, b, c),
                -i128::from(sign) * i128::from(sign),
                "a conta exacta e' `-1` por construcao (k={k}, sign={sign})"
            );
            assert_eq!(orient(a, b, c), -1, "k={k} sign={sign}");
            if orient_f64(a, b, c) != -1 {
                disagreements += 1;
            }
        }
    }
    assert!(
        disagreements >= cases / 2,
        "o controlo em f64 tinha de errar na maioria destes casos e errou {disagreements} de {cases} \
         — se ele acerta sempre, a fixtura nao contem o fenomeno que este gate existe para medir"
    );
}

#[test]
fn o_zero_e_o_zero() {
    // Colinearidade exacta, em qualquer escala — é o caso que um epsilon decide por
    // sorteio, e é onde a extracção decide se um ponto cai sobre uma aresta.
    for shift in 0..50u32 {
        let s = 1i64 << shift;
        let a: P = [0, 0];
        let b: P = [3 * s, 6 * s];
        let c: P = [s, 2 * s];
        assert_eq!(orient(a, b, c), 0, "shift {shift}");
        assert_eq!(orient(b, a, c), 0, "shift {shift}");
    }
}

#[test]
fn o_predicado_e_antissimetrico_e_ciclico() {
    let pts: [P; 4] = [[0, 0], [7, 3], [-5, 11], [1 << 40, -(1 << 39)]];
    for a in pts {
        for b in pts {
            for c in pts {
                assert_eq!(orient(a, b, c), -orient(b, a, c), "antissimetria");
                assert_eq!(orient(a, b, c), orient(b, c, a), "ciclo");
            }
        }
    }
}

#[test]
fn a_transicao_compoe_inverte_e_roda_como_a_lei_diz() {
    let pts: [P; 3] = [[0, 0], [5, -9], [1 << 30, 1 << 20]];
    for r in 0..4u8 {
        for t in [[0i64, 0], [3, -7], [1 << 20, -(1 << 21)]] {
            let g = Xf { r, t };
            for p in pts {
                assert_eq!(g.inverse().apply(g.apply(p)), p, "inversa (r={r})");
                assert_eq!(g.then(g.inverse()).apply(p), p, "composicao com a inversa");
            }
            for r2 in 0..4u8 {
                let h = Xf { r: r2, t: [1, 2] };
                for p in pts {
                    assert_eq!(
                        g.then(h).apply(p),
                        h.apply(g.apply(p)),
                        "`a.then(b)` tem de ser «primeiro a, depois b» (r={r}, r2={r2})"
                    );
                }
            }
        }
    }
}

#[test]
fn a_rotacao_de_um_quarto_e_exacta_e_de_ordem_quatro() {
    let p: P = [123_456_789, -987_654_321];
    let mut q = p;
    for _ in 0..4 {
        q = Xf::rot(1, q);
    }
    assert_eq!(q, p, "quatro quartos de volta sao a identidade, ao bit");
    for r in 0..4u8 {
        for d in 0..4u8 {
            // rodar a direcção e rodar o vector cardinal têm de dar o mesmo.
            let by_dir = CARDINALS[Xf { r, t: [0, 0] }.dir(d) as usize];
            let by_vec = Xf::rot(r, CARDINALS[d as usize]);
            assert_eq!(by_dir, by_vec, "r={r} d={d}");
        }
    }
}

#[test]
fn o_lado_de_um_raio_concorda_com_o_determinante_geral() {
    for ray in [[1i64, 0], [0, 1], [3, 5], [-7, 2], [1 << 40, -3]] {
        for d in 0..4u8 {
            let c = CARDINALS[d as usize];
            let want = orient([0, 0], ray, c);
            assert_eq!(side_of_ray(ray, d), want, "ray={ray:?} d={d}");
            let dot = ray[0] * c[0] + ray[1] * c[1];
            assert_eq!(same_sense(ray, d), dot > 0, "ray={ray:?} d={d}");
        }
    }
}
