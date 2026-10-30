//! ⭐⭐ **O osso que cresceu ESTICA a arte ao longo do eixo** (dono 30/10) — o [`SkinBone::bent_esticado`].

use super::*;

/// Repouso rodado 90° em `(2, 1)`; agora rodado 30° e movido — uma BASE, não os eixos (uma fixtura
/// alinhada aos eixos não mede a composição).
fn poses() -> (Xform, Xform) {
    let rot = |a: f64, t: [f64; 2]| {
        let (s, c) = a.sin_cos();
        Xform([c, s, -s, c, t[0], t[1]])
    };
    (
        rot(std::f64::consts::FRAC_PI_2, [2.0, 1.0]),
        rot(0.5, [7.0, -3.0]),
    )
}

fn perto(a: [f64; 2], b: [f64; 2]) -> bool {
    (a[0] - b[0]).abs() < 1e-9 && (a[1] - b[1]).abs() < 1e-9
}

fn osso(rest: Xform, mundo: Xform, spec: bend::BoneSpec, bind: f64) -> Vec<SkinBone> {
    let mut out = Vec::new();
    SkinBone::bent_esticado(rest, spec, bind, mundo, Xform::IDENTITY, 0, &mut out);
    out
}

/// ⭐ A ponta do BIND vai à ponta de AGORA; a raiz fica; a espessura (o `y` local) não engorda; o
/// alcance continua o do bind (os pesos não mudam com o crescimento).
#[test]
fn o_ponto_da_ponta_do_bind_segue_a_ponta_que_cresceu() {
    let (rest, mundo) = poses();
    let o = osso(rest, mundo, bend::BoneSpec::straight(6.0, 1.0), 4.0);
    assert_eq!(o.len(), 1);
    let p = |l: [f64; 2]| o[0].pose.apply(rest.apply(l));
    assert!(
        perto(p([4.0, 0.0]), mundo.apply([6.0, 0.0])),
        "{:?}",
        p([4.0, 0.0])
    );
    assert!(perto(p([0.0, 0.0]), mundo.apply([0.0, 0.0])));
    assert!(perto(p([2.0, 1.0]), mundo.apply([3.0, 1.0])));
    assert!((o[0].radius - 4.0).abs() < 1e-9, "{}", o[0].radius);
    assert!(perto(o[0].rest_b, rest.apply([4.0, 0.0])));
}

/// ⛔ CONTROLO: bind sem comprimento (ficheiro `<= 210`) ou igual ao de agora ⇒ o `bent` de sempre,
/// AO BIT (a pose e o eixo).
#[test]
fn sem_comprimento_do_bind_a_pele_e_a_de_sempre() {
    let (rest, mundo) = poses();
    let spec = bend::BoneSpec::straight(6.0, 1.0);
    let mut sempre = Vec::new();
    SkinBone::bent(rest, spec, mundo, Xform::IDENTITY, 0, &mut sempre);
    for bind in [0.0, 6.0] {
        let o = osso(rest, mundo, spec, bind);
        assert_eq!(o[0].pose, sempre[0].pose);
        assert_eq!(o[0].rest_b, sempre[0].rest_b);
    }
    // E a mesma conta COM esticão desloca a ponta (a lei não é um no-op).
    let o = osso(rest, mundo, spec, 4.0);
    assert!(!perto(
        o[0].pose.apply(rest.apply([4.0, 0.0])),
        sempre[0].pose.apply(rest.apply([4.0, 0.0]))
    ));
}

/// ⭐ O *bendy bone* estica igual: a raiz do 1.º sub-osso fica, a ponta do bind vai à ponta de agora
/// pelo ÚLTIMO (a curvatura não mexe a ponta).
#[test]
fn o_osso_curvo_estica_pelos_sub_ossos() {
    let (rest, mundo) = poses();
    let spec = bend::BoneSpec {
        segments: 4,
        curve: bend::Bend {
            inn: [0.0, 1.5],
            out: [0.0, -1.0],
        },
        ..bend::BoneSpec::straight(6.0, 1.0)
    };
    let o = osso(rest, mundo, spec, 4.0);
    assert_eq!(o.len(), 4);
    let ponta = o[3].pose.apply(rest.apply([4.0, 0.0]));
    assert!(perto(ponta, mundo.apply([6.0, 0.0])), "{ponta:?}");
    assert!(perto(
        o[0].pose.apply(rest.apply([0.0, 0.0])),
        mundo.apply([0.0, 0.0])
    ));
    // As juntas continuam fechadas: o fim do sub-osso k é o começo do k+1.
    for k in 0..3 {
        let j = rest.apply([4.0 * f64::from(k + 1) / 4.0, 0.0]);
        let (a, b) = (o[k as usize].pose.apply(j), o[k as usize + 1].pose.apply(j));
        assert!(perto(a, b), "junta {k}: {a:?} × {b:?}");
    }
}
