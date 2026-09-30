//! Os gates da [`super::resolve_overlap`].

use super::*;
use ph2d_vec_scene::{Paint, Rgba8, StrokeSpec, VecVertex};

fn poligono(pts: &[[f64; 2]]) -> VecPath {
    VecPath {
        verts: pts.iter().map(|&p| VecVertex::corner(p)).collect(),
        closed: true,
        fill: Some(Paint::Solid(Rgba8::new(200, 120, 40, 255))),
        stroke: Some(StrokeSpec::new(Rgba8::new(20, 20, 20, 255), 0.1)),
        ..VecPath::default()
    }
}

/// Um «L» cujo braço vertical DESCE ALÉM da banda de baixo e volta para dentro dela — o
/// cotovelo de uma dobra forte, reduzido ao essencial: a face de dentro do braço atravessa a
/// aresta de baixo da banda DUAS vezes (em `x = 3` e em `x ≈ 2,67`).
fn cotovelo_cruzado() -> VecPath {
    poligono(&[
        [0.0, 0.0],
        [4.0, 0.0],
        [4.0, 4.0],
        [3.0, 4.0],
        [3.0, -0.5],
        [2.0, 1.0],
        [0.0, 1.0],
    ])
}

fn cruzes(p: &VecPath) -> bool {
    crosses_itself(&crate::to_bez(p))
}

/// ⭐⭐⭐ **GATE — a silhueta de um contorno que se cruza NÃO se cruza, e a área é a do
/// preenchimento.**
///
/// ⚠️ **As duas metades são obrigatórias:** sem a primeira (o CONTROLO: a fixtura cruza-se) o
/// gate passaria sobre uma forma que nunca teve contacto; sem a da área, uma porta que devolvesse
/// o CASCO convexo também não se cruzaria.
#[test]
fn a_silhueta_de_um_contorno_cruzado_nao_se_cruza() {
    let p = cotovelo_cruzado();
    assert!(
        cruzes(&p),
        "a fixtura deixou de se cruzar — não contém o fenómeno"
    );
    let s = resolve_overlap(&p).expect("um contorno que se cruza tem silhueta");
    assert!(!cruzes(&s), "a silhueta ainda se cruza");
    // A área é a da união do caminho com o VAZIO pelo motor — a régua é que a porta não perca
    // nem invente região ao reconstruir os contornos.
    let area = crate::area(&s);
    // ⚠️ O motor devolve UMA peça por ilha (aqui: o corpo e o triângulo que o braço deixa abaixo
    // da banda), e a silhueta leva-as todas num caminho só — a régua é a SOMA.
    let pecas = crate::apply(&p, &VecPath::default(), crate::BoolOp::Union);
    assert!(
        pecas.len() >= 2,
        "a fixtura deixou de ter duas ilhas: {}",
        pecas.len()
    );
    let base: f64 = pecas.iter().map(crate::area).sum();
    assert!(
        (area - base).abs() < 1e-9,
        "a silhueta tem área {area} e a união do caminho com o vazio tem {base}"
    );
}

/// ⭐⭐ **GATE — sem contacto a porta NÃO corre, e a forma fica como era.**
///
/// ⚠️ É o que mantém toda a pele fora do cotovelo byte-idêntica: a porta só troca a geometria
/// quando há o que trocar.
#[test]
fn sem_contacto_a_porta_nao_corre() {
    let p = poligono(&[
        [0.0, 0.0],
        [4.0, 0.0],
        [4.0, 4.0],
        [3.0, 4.0],
        [3.0, 1.0],
        [0.0, 1.0],
    ]);
    assert!(!cruzes(&p), "um L limpo não se cruza");
    assert!(resolve_overlap(&p).is_none(), "a porta correu sem contacto");
}

/// ⭐⭐ **GATE — o ESTILO é o da forma.** A porta muda a geometria e mais nada: o traço, a
/// opacidade e a mistura que a forma tinha são os que ela desenha a seguir.
#[test]
fn a_silhueta_leva_o_estilo_da_forma() {
    let mut p = cotovelo_cruzado();
    p.opacity = ph2d_vec_scene::Opacity::new(0.5);
    let s = resolve_overlap(&p).expect("silhueta");
    assert_eq!(s.fill, p.fill);
    assert_eq!(s.stroke, p.stroke);
    assert_eq!(s.opacity, p.opacity);
    assert_eq!(s.id, p.id);
}

/// ⭐ **GATE — um caminho ABERTO não tem silhueta**, mesmo que se cruze: não há interior.
#[test]
fn um_caminho_aberto_nao_tem_silhueta() {
    let mut p = cotovelo_cruzado();
    p.closed = false;
    assert!(resolve_overlap(&p).is_none());
}

/// ⭐ **GATE — dois segmentos que só se TOCAM não contam** (um toque num vértice não é contacto
/// que peça silhueta), e os VIZINHOS do mesmo contorno nunca contam.
#[test]
fn um_toque_nao_e_um_cruzamento() {
    // Um «8» que se toca num ponto, sem se atravessar: dois triângulos com um vértice comum.
    let p = poligono(&[
        [0.0, 0.0],
        [1.0, 1.0],
        [2.0, 0.0],
        [2.0, 2.0],
        [1.0, 1.0],
        [0.0, 2.0],
    ]);
    assert!(!cruzes(&p), "tocar num ponto foi lido como cruzar");
}
