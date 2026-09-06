//! Os gates do que **o VETOR** sabe. ⚠️ A lei (pesos, órfão, C¹, escala) é gateada na
//! [`ph2d_skeleton`], que é a dona dela — repeti-la aqui poria a mesma afirmação em dois sítios,
//! e a segunda envelhece.

use super::*;
use ph2d_skeleton::{SkinBone, Xform};
use ph2d_vec_scene::{VecPath, VecVertex};

/// Um osso deitado sobre o eixo X, de `(x0,0)` a `(x0+len, 0)`, em repouso e sem pose.
fn osso(x0: f64, len: f64, strength: f64) -> SkinBone {
    SkinBone::new(
        Xform([1.0, 0.0, 0.0, 1.0, x0, 0.0]),
        len,
        strength,
        Xform([1.0, 0.0, 0.0, 1.0, x0, 0.0]),
        Xform::IDENTITY,
    )
    .expect("repouso nao-singular")
}

/// O mesmo osso, mas POSADO: transladado de `(dx, dy)` em mundo.
fn osso_movido(x0: f64, len: f64, strength: f64, d: [f64; 2]) -> SkinBone {
    SkinBone::new(
        Xform([1.0, 0.0, 0.0, 1.0, x0, 0.0]),
        len,
        strength,
        Xform([1.0, 0.0, 0.0, 1.0, x0 + d[0], d[1]]),
        Xform::IDENTITY,
    )
    .expect("repouso nao-singular")
}

/// ⭐⭐ **AS TRÊS METADES DE UM VÉRTICE RESPONDEM À POSIÇÃO DELAS** — o `CubicWeight` do Rive.
///
/// A âncora fica dentro do alcance do osso da ESQUERDA e a alça de saída dentro do da DIREITA; ao
/// mexer só o da direita, a alça anda e a âncora **não**. Pesar o vértice inteiro pela âncora
/// deixaria a alça parada, e o sintoma é a curva a rasgar-se numa junta.
///
/// ⚠️ **É o gate que justifica esta crate existir.** Ele não é exprimível na `ph2d-skeleton`: lá
/// não há vértice nenhum, só pontos.
#[test]
fn the_three_halves_of_a_vertex_answer_to_their_own_position() {
    let esq = osso(0.0, 4.0, 1.0);
    let dir_parado = osso(20.0, 4.0, 1.0);
    let dir_movido = osso_movido(20.0, 4.0, 1.0, [0.0, 7.0]);
    let v = VecVertex::smooth([1.0, 0.0], [-2.0, 0.0], [21.0, 0.0]);
    let forma = VecPath {
        verts: vec![v],
        ..VecPath::default()
    };

    let mut parado = forma.clone();
    apply(&Skin::new(vec![esq, dir_parado]).unwrap(), &mut parado);
    let mut movido = forma.clone();
    apply(&Skin::new(vec![esq, dir_movido]).unwrap(), &mut movido);

    let danca = |a: [f64; 2], b: [f64; 2]| (a[1] - b[1]).abs();
    assert!(
        danca(parado.verts[0].out_handle, movido.verts[0].out_handle) > 5.0,
        "a alca de saida esta' dentro do osso que se mexeu e nao o seguiu"
    );
    assert!(
        danca(parado.verts[0].anchor, movido.verts[0].anchor) < 1e-9,
        "a ancora esta' FORA do osso que se mexeu e mexeu-se na mesma - os pesos estao a sair da \
         posicao errada"
    );
}

/// ⭐ **NO REPOUSO O DESENHO NÃO SE MEXE** — a lei da casa, medida **na mídia**.
///
/// ⚠️ A lei já é gateada no módulo sobre pontos crus; este gate mede a metade que é daqui: que a
/// travessia do caminho alcança as três metades de todo vértice e não estraga nenhuma pelo
/// caminho.
#[test]
fn at_rest_the_drawing_does_not_move_a_single_half() {
    let pele = Skin::new(vec![osso(0.0, 6.0, 1.5), osso(5.0, 6.0, 1.5)]).expect("2 ossos");
    let antes = VecPath {
        verts: vec![
            VecVertex::smooth([0.0, 0.0], [-1.0, 0.5], [1.0, -0.5]),
            VecVertex::smooth([10.0, 0.0], [9.0, 1.0], [11.0, -1.0]),
            VecVertex::corner([5.0, 8.0]),
        ],
        closed: true,
        ..VecPath::default()
    };
    let mut depois = antes.clone();
    apply(&pele, &mut depois);
    let mut pior = 0.0_f64;
    for (a, b) in antes.verts_all().zip(depois.verts_all()) {
        for (p, q) in [
            (a.anchor, b.anchor),
            (a.in_handle, b.in_handle),
            (a.out_handle, b.out_handle),
        ] {
            pior = pior.max((p[0] - q[0]).abs()).max((p[1] - q[1]).abs());
        }
    }
    assert!(
        pior < 1e-12,
        "o repouso moveu o desenho em {pior} - o `rest` nao esta' a ser invertido pela pose"
    );
}

/// ⛔ **O RAIO DE QUINA NÃO É DEFORMADO** — ele é FONTE, e escalá-lo pediria um factor por
/// vértice. A decisão está escrita no doc de [`apply`]; este gate impede que ela se perca numa
/// refactoração silenciosa.
#[test]
fn the_corner_radius_travels_untouched_through_the_deformation() {
    let pele = Skin::new(vec![osso_movido(0.0, 10.0, 2.0, [4.0, 4.0])]).unwrap();
    let mut v = VecVertex::corner([2.0, 1.0]);
    v.corner_radius = 3.5;
    let mut forma = VecPath {
        verts: vec![v],
        ..VecPath::default()
    };
    apply(&pele, &mut forma);
    assert!(
        (forma.verts[0].anchor[0] - 2.0).abs() > 1e-9,
        "a fixtura tem de MEXER o vertice, senao ela nao mede nada"
    );
    assert_eq!(
        forma.verts[0].corner_radius, 3.5,
        "o raio de quina foi deformado - ele e' FONTE e viaja intacto"
    );
}
