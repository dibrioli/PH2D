//! Os gates do [`super::origem`] — onde cada píxel da vista nova estava na vista de antes.
//!
//! ⚠️ Os dois primeiros correm em vistas ORTOGRÁFICAS (a resposta calcula-se à mão). O da OCLUSÃO
//! corre em PERSPECTIVA de propósito: numa ortográfica de matriz identidade o `1/w` é o mesmo para
//! toda a superfície, e o teste de profundidade não teria como separar a peça da frente da de trás
//! — a fixture não conteria o fenómeno.

use ph2d_mesh::{Face, Mesh};

use super::origem;
use crate::tela_na_malha::Vista;
use crate::tela_na_malha_tests::{LADO, malha, vista};

/// Ortográfica deslocada `dx` em NDC: um ponto cai `dx · LADO / 2` píxeis à direita.
fn deslocada(dx: f32) -> Vista {
    let m = [
        1.0, 0.0, 0.0, 0.0, //
        0.0, 1.0, 0.0, 0.0, //
        0.0, 0.0, 1.0, 0.0, //
        dx, 0.0, 0.0, 1.0,
    ];
    Vista::nova(m, (LADO, LADO), [0.0, 0.0, 10.0])
}

/// Perspectiva de 45° com o olho em `(ex, 0, 3)` a olhar para `−z`.
fn perspectiva(ex: f32, w: u32, h: u32) -> Vista {
    let f = 1.0 / (22.5f32.to_radians()).tan();
    let asp = w as f32 / h as f32;
    let (n, fa) = (0.1f32, 100.0f32);
    let proj = [
        f / asp,
        0.0,
        0.0,
        0.0, //
        0.0,
        f,
        0.0,
        0.0, //
        0.0,
        0.0,
        fa / (n - fa),
        -1.0, //
        -f / asp * ex,
        0.0,
        -3.0 * fa / (n - fa) + fa * n / (n - fa),
        3.0,
    ];
    Vista::nova(proj, (w, h), [ex, 0.0, 3.0])
}

/// ⚠️ Recusa um píxel FORA da tela: com `as usize` um negativo satura a `0` e o gate leria o
/// píxel da borda — a 1.ª redacção do gate da oclusão passou assim, com a placa fora do ecrã.
fn em(o: &[Option<[f32; 2]>], w: u32, p: [f32; 2]) -> Option<[f32; 2]> {
    let h = o.len() as u32 / w;
    assert!(
        p[0] >= 0.0 && p[1] >= 0.0 && p[0] < w as f32 && p[1] < h as f32,
        "o ponto {p:?} caiu fora da tela {w}×{h} — a fixture não o contém"
    );
    o[p[1] as usize * w as usize + p[0] as usize]
}

/// ⭐ **A mesma vista devolve o próprio píxel** — o CONTROLO de que o mapa é a identidade quando
/// nada mudou.
#[test]
fn a_mesma_vista_devolve_o_proprio_pixel() {
    let m = malha(10);
    let v = vista();
    let o = origem(&m, &v, &v);
    for &(x, y) in &[(0u32, 0u32), (37, 12), (50, 50), (99, 99)] {
        let q = em(&o, LADO, [x as f32, y as f32]).expect("a peça enche a vista");
        assert!(
            (q[0] - (x as f32 + 0.5)).abs() < 1e-3 && (q[1] - (y as f32 + 0.5)).abs() < 1e-3,
            "({x},{y}) devolveu {q:?}"
        );
    }
}

/// ⭐⭐ **Uma vista que andou leva cada píxel ao sítio de antes** — e onde o sítio de antes caía
/// fora da tela não há origem.
#[test]
fn uma_vista_que_andou_leva_cada_pixel_ao_sitio_de_antes() {
    let m = malha(10);
    // A vista de antes via tudo 10 píxeis mais à direita.
    let antes = deslocada(0.2);
    let o = origem(&m, &vista(), &antes);
    let q = em(&o, LADO, [40.0, 20.0]).expect("visto antes");
    assert!(
        (q[0] - 50.5).abs() < 1e-3 && (q[1] - 20.5).abs() < 1e-3,
        "{q:?}"
    );
    // O CONTROLO da borda: 95 + 10 cai fora da tela de antes.
    assert_eq!(em(&o, LADO, [95.0, 20.0]), None);
    assert!(em(&o, LADO, [89.0, 20.0]).is_some());
}

/// ⭐⭐ **O que a vista de antes não via fica sem origem** — um ponto do chão que estava TAPADO por
/// uma placa à frente dele, e que a vista nova já vê. O CONTROLO é um ponto do mesmo chão que as
/// duas vistas viam, e ele tem origem no sítio certo.
#[test]
fn o_que_a_vista_de_antes_nao_via_fica_sem_origem() {
    let (w, h) = (200u32, 200u32);
    let (mut pos, mut faces) = crate::tela_na_malha_tests::grelha(20, 0.0, false);
    let base = pos.len() as u32;
    pos.extend([
        [-0.1, -0.1, 1.0],
        [0.1, -0.1, 1.0],
        [0.1, 0.1, 1.0],
        [-0.1, 0.1, 1.0],
    ]);
    faces.push(Face::quad(base, base + 1, base + 2, base + 3));
    let m = Mesh::from_parts(pos, faces).expect("válida");
    let antes = perspectiva(0.0, w, h);
    let nova = perspectiva(0.6, w, h);
    let o = origem(&m, &nova, &antes);
    // A origem do chão: tapada pela placa na vista de antes, à vista na nova.
    let escondido = nova.ecra([0.0, 0.0, 0.0]).expect("à frente");
    assert_eq!(
        em(&o, w, escondido),
        None,
        "o chão debaixo da placa não era visto antes"
    );
    // O CONTROLO: um ponto do chão que as duas vistas viam.
    let visto = [0.8, 0.6, 0.0];
    let q = em(&o, w, nova.ecra(visto).expect("à frente")).expect("as duas vistas o viam");
    let esperado = antes.ecra(visto).expect("à frente");
    assert!(
        (q[0] - esperado[0]).abs() < 1.5 && (q[1] - esperado[1]).abs() < 1.5,
        "{q:?} contra {esperado:?}"
    );
    // E a própria placa, que as duas viam, tem origem.
    assert!(em(&o, w, nova.ecra([0.0, 0.0, 1.0]).expect("à frente")).is_some());
}
