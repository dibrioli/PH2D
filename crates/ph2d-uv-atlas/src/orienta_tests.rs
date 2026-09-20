//! Gates da orientação.

use super::orienta::{eixo_da_caixa_minima, roda};

/// A caixa de uma nuvem no referencial de `eixo`, e a área dela.
fn caixa(p: &[[f32; 2]], eixo: [f32; 2]) -> f64 {
    let (mut lo, mut hi) = ([f64::MAX; 2], [f64::MIN; 2]);
    for &z in p {
        let q = roda(z, eixo);
        lo[0] = lo[0].min(f64::from(q[0]));
        hi[0] = hi[0].max(f64::from(q[0]));
        lo[1] = lo[1].min(f64::from(q[1]));
        hi[1] = hi[1].max(f64::from(q[1]));
    }
    (hi[0] - lo[0]) * (hi[1] - lo[1])
}

/// Um rectângulo `larg × alt` rodado de `ang`, com pontos ao longo dos lados.
fn rectangulo(larg: f32, alt: f32, ang: f32) -> Vec<[f32; 2]> {
    let (c, s) = (ang.cos(), ang.sin());
    let mut out = Vec::new();
    for i in 0..=10i16 {
        let t = larg * f32::from(i) / 10.0;
        for &v in &[0.0f32, alt] {
            out.push([t.mul_add(c, -(v * s)), t.mul_add(s, v * c)]);
        }
    }
    out
}

/// ⭐⭐⭐ **A caixa mínima de um rectângulo inclinado é o próprio rectângulo.**
///
/// ⚠️ E o controlo é a caixa ALINHADA AOS EIXOS da mesma nuvem: sem ela, uma função que
/// devolvesse sempre `[1, 0]` passaria em tudo o que mede só a caixa encontrada.
#[test]
fn um_rectangulo_inclinado_volta_a_ficar_direito() {
    let p = rectangulo(10.0, 1.0, 0.5);
    let e = eixo_da_caixa_minima(&p);
    let (min, eixos) = (caixa(&p, e), caixa(&p, [1.0, 0.0]));
    assert!(
        (min - 10.0).abs() < 1.0e-3,
        "a caixa minima e' a area do rectangulo: {min}"
    );
    assert!(
        eixos > 3.0 * min,
        "a fixtura tem de conter o fenomeno: alinhada {eixos}, minima {min}"
    );
}

/// ⭐⭐ **Nunca pior que a alinhada aos eixos**, e num rectângulo já direito ela **é** a
/// alinhada — o controlo que impede uma função que roda sempre.
#[test]
fn a_caixa_minima_nunca_e_pior_que_a_alinhada() {
    for ang in [0.0f32, 0.1, 0.4, std::f32::consts::FRAC_PI_4, 1.2, 1.5] {
        let p = rectangulo(7.0, 2.0, ang);
        let e = eixo_da_caixa_minima(&p);
        let (min, eixos) = (caixa(&p, e), caixa(&p, [1.0, 0.0]));
        assert!(min <= eixos + 1.0e-6, "ang {ang}: {min} contra {eixos}");
        assert!((min - 14.0).abs() < 1.0e-3, "ang {ang}: {min}");
    }
    let direito = rectangulo(7.0, 2.0, 0.0);
    let e = eixo_da_caixa_minima(&direito);
    assert!(
        e[1].abs() < 1.0e-6,
        "um rectangulo ja' direito nao se roda: {e:?}"
    );
}

/// ⛔⛔ **Rodar é um movimento RÍGIDO** — é isso, e só isso, que faz a injectividade que o
/// corte provou sobreviver à orientação. *Se rodar mudasse uma distância, o atlas podia
/// voltar a pintar duas vezes depois de o corte dizer que não.*
#[test]
fn rodar_nao_muda_uma_distancia_nem_uma_area() {
    let e = eixo_da_caixa_minima(&rectangulo(5.0, 3.0, 0.7));
    let p = [[1.0f32, 2.0], [4.0, -1.0], [-2.0, 0.5]];
    let q = p.map(|z| roda(z, e));
    for i in 0..3 {
        let j = (i + 1) % 3;
        let a = (p[i][0] - p[j][0]).hypot(p[i][1] - p[j][1]);
        let b = (q[i][0] - q[j][0]).hypot(q[i][1] - q[j][1]);
        assert!((a - b).abs() < 1.0e-5, "distancia {i}: {a} contra {b}");
    }
    let area = |t: [[f32; 2]; 3]| {
        f64::from(
            (t[1][0] - t[0][0]) * (t[2][1] - t[0][1]) - (t[1][1] - t[0][1]) * (t[2][0] - t[0][0]),
        )
    };
    assert!((area(p) - area(q)).abs() < 1.0e-4);
}

/// ⛔ **Uma nuvem degenerada devolve a IDENTIDADE** — um vector nulo normalizado é ruído
/// a fingir de direcção, e ele rodaria a peça para um sítio arbitrário.
#[test]
fn uma_nuvem_degenerada_devolve_a_identidade() {
    for p in [
        vec![],
        vec![[1.0f32, 1.0]],
        vec![[1.0f32, 1.0], [2.0, 2.0]],
        vec![[0.0f32, 0.0]; 12],
    ] {
        assert_eq!(eixo_da_caixa_minima(&p), [1.0, 0.0], "{p:?}");
    }
    // ⚠️ Colineares: o casco degenera e a caixa tem área zero em qualquer eixo. O que se
    // exige é que ela não estoure e devolva algo unitário.
    let linha: Vec<[f32; 2]> = (0..8i16)
        .map(|i| [f32::from(i), f32::from(i) * 2.0])
        .collect();
    let e = eixo_da_caixa_minima(&linha);
    assert!((e[0].hypot(e[1]) - 1.0).abs() < 1.0e-5, "{e:?}");
}

/// ⭐⭐⭐ **A orientação CHEGA ao atlas, e não desfaz o corte.**
///
/// ⛔ As de cima medem a PORTA; esta percorre o `build` inteiro — *um knob com a lei certa
/// e o atlas a não o chamar lê-se como um knob morto*, e a régua tem as duas metades: o
/// `(u, v)` muda, e a sobreposição continua a ler zero.
#[test]
fn a_orientacao_chega_ao_atlas_e_nao_devolve_a_sobreposicao() {
    let (mesh, cut, map, jumps) = super::lib_tests::fita_com(0, false, true);
    let com = super::build(&mesh, &cut, &map, &jumps);
    let sem = super::build_com(
        &mesh,
        &cut,
        &map,
        &jumps,
        super::Opcoes {
            orientar: false,
            ..super::Opcoes::default()
        },
    );
    assert_eq!(
        com.relatorio.ilhas, sem.relatorio.ilhas,
        "o corte e' o mesmo"
    );
    assert!(
        com.relatorio.aproveitamento >= sem.relatorio.aproveitamento - 1.0e-6,
        "orientar nunca piora a arrumacao: {} contra {}",
        com.relatorio.aproveitamento,
        sem.relatorio.aproveitamento
    );
    for c in super::sobreposicao::Classe::ALL {
        let s = super::sobreposicao::medir(&mesh, &com);
        assert!(
            s.area_por_classe[c.indice()] <= 0.0,
            "{} cruza depois de orientar: {:?}",
            c.nome(),
            s.pares
        );
    }
}
