//! A12 — as passagens NOVAS que o traço enche cortam-se pela corda.
//!
//! ⚠️ A régua dos casos sintéticos é a GEOMETRIA conhecida da fixtura (o bico da fenda, a ponta do
//! dente, o vale da fonte): está ou não está sobre o contorno do desenho.

use super::{SEM_FECHO, fecha_as_fendas_que_o_traco_enche};
use ph2d_vec_scene::{Contour, VecPath, VecVertex};

/// A largura do traço das fixturas sintéticas.
const W: f64 = 2.0;

fn poligono(pts: &[[f64; 2]]) -> Contour {
    Contour {
        verts: pts.iter().map(|&p| VecVertex::corner(p)).collect(),
        closed: true,
    }
}

/// O caminho com estes contornos fechados e o traço `W`.
fn caminho(cs: Vec<Contour>) -> VecPath {
    let mut it = cs.into_iter();
    let c0 = it.next().expect("um contorno");
    VecPath {
        verts: c0.verts,
        closed: true,
        subpaths: it.collect(),
        stroke: Some(ph2d_vec_scene::StrokeSpec::new(
            ph2d_vec_scene::Rgba8::new(0, 0, 0, 255),
            W,
        )),
        ..VecPath::default()
    }
}

/// A união da fonte, com ou sem a lei.
fn uniao(fonte: &VecPath, com_a_lei: bool) -> VecPath {
    let mut u = ph2d_vec_boolean::resolve_overlap(fonte).expect("sobrepõe-se");
    if com_a_lei {
        fecha_as_fendas_que_o_traco_enche(&mut u, fonte);
    }
    u
}

/// A distância de `p` ao contorno desenhado de `u`.
fn ao_contorno(u: &VecPath, p: [f64; 2]) -> f64 {
    (0..u.contour_count())
        .filter_map(|c| u.contour(c))
        .flat_map(|(v, _)| {
            super::polilinha(v)
                .into_iter()
                .collect::<Vec<_>>()
                .windows(2)
                .map(|s| super::ao_troco(p, s[0], s[1]).0)
                .collect::<Vec<_>>()
        })
        .fold(f64::MAX, f64::min)
}

/// A soma das áreas com sinal dos contornos de `u`.
fn area(u: &VecPath) -> f64 {
    (0..u.contour_count())
        .filter_map(|c| u.contour(c))
        .map(|(v, _)| {
            let p = super::polilinha(v);
            0.5 * (0..p.len())
                .map(|i| {
                    let (a, b) = (p[i], p[(i + 1) % p.len()]);
                    a[0] * b[1] - b[0] * a[1]
                })
                .sum::<f64>()
        })
        .sum()
}

/// O rectângulo `[0, 10] × [0, 4]` com um VALE (da fonte) na base, bico em `(5, 3)`, e por cima um
/// quadrilátero cuja aresta de baixo (declive `0,3`) cruza o topo dele em `(9,33…, 4)`: entre os dois
/// fica uma FENDA nova, `0,2` de largura no canto `(10, 4)`.
fn fenda_e_vale() -> VecPath {
    caminho(vec![
        poligono(&[
            [0.0, 0.0],
            [4.4, 0.0],
            [5.0, 3.0],
            [5.6, 0.0],
            [10.0, 0.0],
            [10.0, 4.0],
            [0.0, 4.0],
        ]),
        poligono(&[[6.0, 3.0], [20.0, 7.2], [20.0, 20.0], [6.0, 20.0]]),
    ])
}

/// ⭐⭐ **GATE — a fenda NOVA fecha-se e o vale da FONTE fica.** ⛔ **O CONTROLO:** sem a lei o bico
/// da fenda está no contorno, e a união é menor.
#[test]
fn a_fenda_nova_fecha_e_o_vale_da_fonte_fica() {
    let fonte = fenda_e_vale();
    let (sem, com) = (uniao(&fonte, false), uniao(&fonte, true));
    // O bico da fenda: onde a aresta do triângulo cruza o topo do rectângulo.
    let bico = [6.0 + 1.0 / 0.3, 4.0];
    assert!(
        ao_contorno(&sem, bico) < 1e-6,
        "controlo: o bico da fenda está no contorno"
    );
    assert!(ao_contorno(&com, bico) > 0.1, "a fenda nova ficou");
    assert!(
        area(&com).abs() > area(&sem).abs() + 1e-3,
        "a fenda não se encheu"
    );
    // O vale da fonte (bico em (5, 3), boca de 1,2 < W) fica.
    assert!(
        ao_contorno(&com, [5.0, 3.0]) < 1e-6,
        "o vale da fonte foi fechado"
    );
}

/// O anel `[0, 20]²` com o buraco `[5, 15]²` e um DENTE fino que sai da parede de baixo e entra no
/// buraco até `(10, 9)` — na boca do buraco (`y = 5`) ele tem `0,69 < W`.
fn dente_no_buraco() -> VecPath {
    caminho(vec![
        poligono(&[[0.0, 0.0], [20.0, 0.0], [20.0, 20.0], [0.0, 20.0]]),
        poligono(&[[5.0, 5.0], [5.0, 15.0], [15.0, 15.0], [15.0, 5.0]]),
        poligono(&[[9.4, 2.0], [10.6, 2.0], [10.0, 9.0]]),
    ])
}

/// ⭐⭐ **GATE — a PONTA nova que entra num buraco corta-se pela parede** (a outra marca da `=5` a
/// `100°`). ⛔ **O CONTROLO:** sem a lei a ponta do dente está no contorno.
#[test]
fn a_ponta_nova_que_entra_num_buraco_corta_se() {
    let fonte = dente_no_buraco();
    let (sem, com) = (uniao(&fonte, false), uniao(&fonte, true));
    assert!(
        ao_contorno(&sem, [10.0, 9.0]) < 1e-6,
        "controlo: a ponta está no contorno"
    );
    assert!(ao_contorno(&com, [10.0, 9.0]) > 1.0, "a ponta nova ficou");
    // A parede do buraco continua lá.
    assert!(
        ao_contorno(&com, [7.0, 5.0]) < 1e-6,
        "a parede do buraco mexeu"
    );
}

/// ⭐⭐ **GATE — um contorno que o traço engole INTEIRO fica** (o buraco pequeno: fechá-lo foi
/// recusado pelo dono, F59-b), mesmo novo. ⛔ **O CONTROLO:** o buraco é novo (a fonte não o tem) e
/// mais estreito que o traço.
#[test]
fn o_buraco_que_o_traco_engole_inteiro_fica() {
    let fora = poligono(&[[0.0, 0.0], [20.0, 0.0], [20.0, 20.0], [0.0, 20.0]]);
    let furo = poligono(&[[10.0, 10.0], [10.0, 10.6], [10.6, 10.6], [10.6, 10.0]]);
    let fonte = caminho(vec![fora.clone()]);
    let antes = caminho(vec![fora, furo]);
    assert!(super::raio_inscrito(&super::polilinha(&antes.subpaths[0].verts)) < 0.5 * W);
    let mut u = antes.clone();
    fecha_as_fendas_que_o_traco_enche(&mut u, &fonte);
    assert_eq!(u, antes, "o buraco pequeno mexeu");
}

/// A lei desligada não mexe (a porta do controlo dos gates da cena).
#[test]
fn sem_a_lei_nada_muda() {
    let fonte = fenda_e_vale();
    let mut u = ph2d_vec_boolean::resolve_overlap(&fonte).expect("sobrepõe-se");
    let antes = u.clone();
    SEM_FECHO.with(|c| c.set(true));
    fecha_as_fendas_que_o_traco_enche(&mut u, &fonte);
    SEM_FECHO.with(|c| c.set(false));
    assert_eq!(u, antes);
}
