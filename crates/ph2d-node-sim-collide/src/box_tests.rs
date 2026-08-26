//! Gates da **CAIXA SÓLIDA** (doc 89, folha 13 — *mais formas de colisor*).
//!
//! A lei tem quatro metades: fora da caixa o contacto é o do rectângulo ARREDONDADO (a
//! distância ao ponto mais próximo, com a normal na direcção dela), dentro dela a saída é
//! pelo eixo de MENOR penetração, o `angle` roda-a de verdade, e uma peça longe não toca.

use super::*;

/// Metades de uma caixa `4 × 2`.
const HALF: [f32; 2] = [2.0, 1.0];
/// A normal do plano sem inclinação — a mesma que a porta do `angle` produz em `0°`.
fn flat_n() -> [f32; 2] {
    plane_normal(0.0)
}

fn hit(p: [f32; 2], r: f32, n: [f32; 2]) -> Option<([f32; 2], f32)> {
    box_contact(p, [0.0, 0.0], HALF, r, n)
}

/// Uma peça bem longe não toca em nada — o gate mais barato de que a lei não é «sempre».
#[test]
fn a_piece_far_away_never_touches_the_box() {
    for p in [[9.0, 0.0], [0.0, -7.5], [-6.0, 6.0]] {
        assert!(hit(p, 0.3, flat_n()).is_none(), "ponto {p:?}");
    }
}

/// **FORA, de frente para uma face:** a normal é o eixo, e a profundidade é o que falta para
/// a peça caber. É o caso que um chão também responderia — e tem de coincidir com ele.
#[test]
fn outside_a_face_the_normal_is_the_axis() {
    // A peça está a `2,2` do centro em x; a face está em `2,0`; a folga é `0,2`.
    let (n, d) = hit([2.2, 0.0], 0.5, flat_n()).expect("toca");
    assert!(
        (n[0] - 1.0).abs() < 1e-5 && n[1].abs() < 1e-5,
        "normal {n:?}"
    );
    assert!((d - 0.3).abs() < 1e-5, "profundidade {d}");
    // E do outro lado a normal aponta ao contrário.
    let (n2, _) = hit([-2.2, 0.0], 0.5, flat_n()).expect("toca");
    assert!((n2[0] + 1.0).abs() < 1e-5, "normal {n2:?}");
}

/// ⭐ **FORA, na DIAGONAL de uma quina:** a normal é a diagonal, não uma das faces. É isto
/// que arredonda os cantos — sem este ramo uma peça encostada a uma quina sairia pela face
/// mais próxima e daria um salto lateral que o olho lê como um empurrão fantasma.
#[test]
fn outside_a_corner_the_normal_is_the_diagonal() {
    // A quina está em `(2, 1)`; a peça está na diagonal, a `0,3` dela.
    let k = 0.3 / core::f32::consts::SQRT_2;
    let (n, d) = hit([2.0 + k, 1.0 + k], 0.5, flat_n()).expect("toca");
    assert!(
        (n[0] - n[1]).abs() < 1e-4 && n[0] > 0.0,
        "a normal tem de ser a diagonal: {n:?}"
    );
    assert!((d - 0.2).abs() < 1e-4, "profundidade {d}");
    // E ela é UNITÁRIA — uma normal que não o fosse escalaria a resposta em silêncio.
    let len = (n[0] * n[0] + n[1] * n[1]).sqrt();
    assert!((len - 1.0).abs() < 1e-5, "|n| = {len}");
}

/// ⭐⭐ **DENTRO: sai pelo eixo de MENOR penetração.** Sem este ramo uma peça que nasceu
/// dentro da caixa ficaria presa lá para sempre — o mesmo problema que o centro exacto de um
/// disco tem, e que aquele resolve com a direcção arbitrária.
#[test]
fn inside_the_box_the_shallowest_axis_wins() {
    // Perto da face de cima (`y = 1`): faltam `0,1` em y e `1,5` em x ⇒ sai em +y.
    let (n, d) = hit([0.5, 0.9], 0.0, flat_n()).expect("toca");
    assert!(
        n[0].abs() < 1e-5 && (n[1] - 1.0).abs() < 1e-5,
        "normal {n:?}"
    );
    assert!((d - 0.1).abs() < 1e-5, "profundidade {d}");
    // Perto da face esquerda: faltam `0,05` em x ⇒ sai em −x.
    let (n2, _) = hit([-1.95, 0.0], 0.0, flat_n()).expect("toca");
    assert!(
        (n2[0] + 1.0).abs() < 1e-5 && n2[1].abs() < 1e-5,
        "normal {n2:?}"
    );
    // E o centro exacto sai por ALGUM lado, com profundidade finita — nunca `NaN`.
    let (n3, d3) = hit([0.0, 0.0], 0.0, flat_n()).expect("toca");
    assert!(
        n3.iter().all(|x| x.is_finite()) && d3.is_finite() && d3 > 0.0,
        "centro: {n3:?} {d3}"
    );
}

/// ⭐⭐ **O `angle` roda a caixa DE VERDADE** — e a prova é a assimetria: um ponto que está
/// fora da caixa alinhada tem de estar DENTRO da mesma caixa rodada 90°, porque a `4 × 2`
/// virada é uma `2 × 4`.
#[test]
fn the_tilt_actually_turns_the_box() {
    let p = [0.0, 1.5]; // fora em y (a meia-altura é 1), dentro em x
    assert!(hit(p, 0.0, flat_n()).is_none(), "alinhada: esta' fora");
    let turned = hit(p, 0.0, plane_normal(90.0)).expect("rodada: esta' dentro");
    assert!(turned.1 > 0.0, "profundidade {}", turned.1);
    // ⚠️ E a 90° a normal de saída é um eixo do MUNDO, não do referencial da caixa: a lei
    // devolve-a já rodada, e é isso que o `respond` espera.
    assert!(
        turned.0.iter().all(|x| x.is_finite()),
        "normal {:?}",
        turned.0
    );
}

/// O raio da peça faz a caixa CRESCER, como faz ao disco: o centro de uma peça de raio `r`
/// nunca pode estar a menos de `r` da superfície.
#[test]
fn the_particle_radius_grows_the_box() {
    // A `2,4` do centro em x, a face está a `0,4`. Sem raio não toca; com `0,5` toca.
    assert!(hit([2.4, 0.0], 0.0, flat_n()).is_none());
    let (_, d) = hit([2.4, 0.0], 0.5, flat_n()).expect("toca");
    assert!((d - 0.1).abs() < 1e-5, "profundidade {d}");
}

/// Uma caixa de extensão ZERO degenera num ponto e não parte nada — a rede contra um
/// documento que peça `box_width = 0`.
#[test]
fn a_degenerate_box_does_not_produce_nonsense() {
    let flat = box_contact([0.1, 0.0], [0.0, 0.0], [0.0, 0.0], 0.5, flat_n());
    let (n, d) = flat.expect("um ponto ainda empurra dentro do raio");
    assert!(n.iter().all(|x| x.is_finite()) && d.is_finite() && d > 0.0);
}

/// **A CADEIA e a CAIXA respondem a perguntas diferentes**, e é isso que a sonda mediu antes
/// desta forma existir: quatro planos encadeados põem as peças DENTRO de um rectângulo (uma
/// conjunção — um contentor); a caixa põe-nas FORA (um obstáculo). Este gate prende a
/// diferença no produto, para a nota nunca mais dizer «união».
#[test]
fn the_box_pushes_out_where_a_chain_of_planes_pushes_in() {
    let mut s = Stream::new(1);
    s.set("P", Column::Vec2(vec![[0.5, 0.0]])); // dentro da caixa
    s.set("vel", Column::Vec2(vec![[0.0, 0.0]]));
    let out = collide(
        &s,
        SHAPE_BOX,
        0.0,
        [0.0, 0.0],
        0.0,
        0.0,
        0.0,
        (RADIUS_POINT, 0.0, 0.0),
        plane_normal(0.0),
        (0.0, 0),
        HALF,
    );
    let Some(Column::Vec2(p)) = out.get("P") else {
        panic!("sai `P`")
    };
    // Saiu pela face de cima ou de baixo (a menor penetração a partir de `(0,5, 0)` é em y).
    assert!(
        p[0][1].abs() >= HALF[1] - 1e-4,
        "a peca tem de ser empurrada para FORA: {:?}",
        p[0]
    );
}
