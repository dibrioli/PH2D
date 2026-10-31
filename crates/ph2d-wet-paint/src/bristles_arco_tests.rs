use super::*;

/// O centro de massa (eixo menor) da coluna `s` da pegada à pressão inteira, dentro da elipse.
fn centro_da_coluna(arco: &Arco, s: f64) -> f64 {
    let h = (1.0 - s * s).sqrt();
    let (mut m, mut mt) = (0.0, 0.0);
    for i in 0..=400 {
        let t = -h + 2.0 * h * f64::from(i) / 400.0;
        let w = arco.no_ponto([s, t], 1.0);
        m += w;
        mt += w * t;
    }
    mt / m
}

#[test]
fn os_centros_dos_tufos_ficam_numa_curva() {
    let arco = Arco { curvatura: 1.0 };
    // O meio sobe, as colunas a 0,6 descem — e simétricas.
    let c0 = centro_da_coluna(&arco, 0.0);
    let c6 = centro_da_coluna(&arco, 0.6);
    let c6n = centro_da_coluna(&arco, -0.6);
    assert!(c0 < -0.4, "o meio do leque à frente: {c0}");
    assert!(c6 > c0 + 0.35, "as laterais atrás do meio: {c6} × {c0}");
    assert!((c6 - c6n).abs() < 1e-9, "simétrico: {c6} × {c6n}");
    // Sem arco a coluna centra-se no eixo.
    assert!(centro_da_coluna(&Arco { curvatura: 1e-9 }, 0.6).abs() < 1e-6);
    // A curva é a parábola: a 2.ª diferença dos centros é constante em s².
    for s in [0.0, 0.2, 0.4] {
        assert!((arco.centro(s) - (s * s - 0.5)).abs() < 1e-12);
    }
}

#[test]
fn a_pressao_leve_so_pousa_os_tufos_do_meio() {
    let arco = Arco { curvatura: 1.0 };
    let tinta = |s: f64, p: f64| arco.no_ponto([s, arco.centro(s) * (1.0 - s * s).sqrt()], p);
    // Leve: o meio pousa inteiro, a borda nada.
    assert_eq!(tinta(0.0, 0.2), 1.0);
    assert_eq!(tinta(0.9, 0.2), 0.0);
    assert_eq!(tinta(-0.9, 0.2), 0.0);
    // Inteira: o leque todo.
    for s in [-0.95, -0.5, 0.0, 0.5, 0.95] {
        assert_eq!(tinta(s, 1.0), 1.0, "s = {s}");
    }
    // A largura do que pousa cresce com a pressão.
    let largura = |p: f64| {
        (0..=200)
            .filter(|&i| tinta(-1.0 + f64::from(i) / 100.0, p) > 0.5)
            .count()
    };
    assert!(largura(0.1) < largura(0.4) && largura(0.4) < largura(0.9));
}

#[test]
fn a_pressao_volta_do_contacto_pela_lei_do_tamanho() {
    assert_eq!(Arco::pressao_do_contacto(0.7, 0.3), 0.0);
    assert_eq!(Arco::pressao_do_contacto(1.0, 0.3), 1.0);
    assert!((Arco::pressao_do_contacto(0.85, 0.3) - 0.5).abs() < 1e-12);
    assert_eq!(Arco::pressao_do_contacto(0.5, 0.0), 1.0);
    assert!(Arco::NEUTRO.neutro());
}
