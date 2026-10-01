use super::*;

/// Um triângulo equilátero de raio inscrito `rho`, centrado na origem.
fn triangulo(rho: f64) -> Vec<VecVertex> {
    let r_circ = 2.0 * rho;
    (0..3)
        .map(|k| {
            let a = std::f64::consts::FRAC_PI_2 + f64::from(k) * std::f64::consts::TAU / 3.0;
            VecVertex::corner([r_circ * a.cos(), r_circ * a.sin()])
        })
        .collect()
}

/// ⭐ **GATE — a bola cabe numa ilha de raio inscrito acima do dela, e não abaixo.** A fronteira é a
/// do fecho: um buraco onde nenhuma bola de raio `r` cabe é cheio inteiro. O triângulo equilátero tem
/// o raio inscrito em forma fechada, e a régua mede-o dos DOIS lados a `±5 %` (a grelha erra
/// `~3 %` do raio nesta escala).
#[test]
fn a_bola_cabe_so_onde_o_raio_inscrito_e_maior_que_o_dela() {
    let r = 0.04;
    assert!(a_bola_cabe_dentro(&triangulo(1.05 * r), r));
    assert!(!a_bola_cabe_dentro(&triangulo(0.95 * r), r));
    assert!(a_bola_cabe_dentro(&triangulo(3.0 * r), r));
    assert!(!a_bola_cabe_dentro(&triangulo(0.4 * r), r));
}

fn viragem_maxima(vs: &[VecVertex]) -> f64 {
    (0..vs.len())
        .filter_map(|i| crate::overlap::viragem_do_vertice(vs, i))
        .fold(0.0, f64::max)
}

fn area(vs: &[VecVertex]) -> f64 {
    crate::area(&ph2d_vec_scene::VecPath {
        verts: vs.to_vec(),
        closed: true,
        ..ph2d_vec_scene::VecPath::default()
    })
    .abs()
}

/// ⭐⭐ **GATE — a bola rola por DENTRO de uma ilha e arredonda os TRÊS cantos.** Duas ilhas, uma
/// para cada defeito que a F44 curou: a PEQUENA (raio inscrito `2 r`, perímetro `20,8 r` contra
/// as janelas de `32 r` de cada lado — a procura comia a volta inteira de um lado só) e a GRANDE
/// (`20 r`, onde a janela não corta nada mas todo nó é um canto — sem nó livre a bola desistia).
///
/// ⚠️ Os CONTROLOS: a ilha nasce com cantos de `120°`, e a bola por FORA não lhe toca (como forma
/// ela é convexa) — sem essa metade, uma porta que rolasse sempre por fora passaria aqui.
#[test]
fn a_bola_arredonda_os_cantos_de_uma_ilha() {
    let r = 0.04;
    for rho in [2.0 * r, 20.0 * r] {
        let t = triangulo(rho);
        assert!(viragem_maxima(&t) > 100.0, "a fixtura deixou de ter cantos");
        assert_eq!(
            crate::bola::rola_a_bola(t.clone(), &[], r, 0.1 * r),
            t,
            "por FORA a bola mexeu num contorno convexo"
        );
        let d = crate::bola::rola_a_bola_por_dentro(t.clone(), &[], r, 0.1 * r);
        assert!(
            viragem_maxima(&d) < 1.0,
            "raio inscrito {rho}: a ilha ficou com um canto de {:.1}°",
            viragem_maxima(&d)
        );
        // O fecho só ACRESCENTA preenchimento: a ilha encolhe e não desaparece.
        assert!(area(&d) < area(&t) && area(&d) > 0.5 * area(&t));
    }
}
