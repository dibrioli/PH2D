//! ⭐⭐ **UM FILETE ACIMA DE `90°` É UM CÍRCULO EM DUAS METADES — nos três escritores** (2026-09-16).
//!
//! A lei da casa ([`crate::shapes::arc`]): um arco escreve-se em cúbicas de até `90°`. Os
//! arredondadores de quina — o da polilinha ([`crate::corners`]), o vivo ([`crate::corner_live`]) e o
//! arco curto da suavização ([`crate::smooth`]) — eram os únicos escritores de arco que a violavam, e
//! uma quina aguda saía numa cúbica só, a errar até `1,29e-2·r` do raio pedido. Este ficheiro prova as
//! três metades (a suavização tem o gate dela no fim):
//!
//! 1. acima de `90°` a quina sai em **duas** cúbicas, e cada uma fica na precisão de um quarto de
//!    círculo (a mesma barra com que o modelador 3D reconhece um arco);
//! 2. até `90°` sai **uma**, como sempre;
//! 3. um blend **assimétrico** (recuos diferentes, porque um lado curva) não é um círculo, e fica
//!    numa cúbica só — partir o que não é um círculo não o tornaria um.

use crate::{VecVertex, VertexKind};

/// A barra: o erro radial do quarto de círculo canónico, `2,7253e-4·r` (medido; ver o
/// `ERRO_DO_QUARTO` do `ph2d-field-profile`, que esta crate não pode ler).
const QUARTO: f64 = 2.7254e-4;

/// Os pontos de um V cuja quina em `(0, 0)` vira `graus`, fechado longe dela.
///
/// ⚠️ Os lados medem `3`: o recuo é `r/tan(θ/2)` e SATURA em meia aresta — com lados de `1` a quina
/// de `170°` saturava (`r` efectivo `0,0437` em vez de `0,05`), e a régua, que usa o `r` pedido,
/// acusava `6,6e-2` sobre um arco correcto.
fn v_pts(graus: f64) -> [[f64; 2]; 4] {
    let half = (std::f64::consts::PI - graus.to_radians()) * 0.5;
    [
        [-3.0 * half.sin(), 3.0 * half.cos()],
        [0.0, 0.0],
        [3.0 * half.sin(), 3.0 * half.cos()],
        [0.0, 9.0],
    ]
}

/// O pior afastamento radial das cúbicas entre `de` e `ate` (índices de vértice, fecho incluído)
/// ao círculo verdadeiro do filete, que tem o centro na bissectriz, a `r/sin(θ/2)` da quina.
fn pior_radial(verts: &[VecVertex], de: usize, ate: usize, graus: f64, r: f64) -> f64 {
    let half = (std::f64::consts::PI - graus.to_radians()) * 0.5;
    let centro = [0.0, r / half.sin()];
    let mut pior = 0.0f64;
    for i in de..ate {
        let (a, b) = (&verts[i], &verts[i + 1]);
        let p = [a.anchor, a.out_handle, b.in_handle, b.anchor];
        for k in 0..=200 {
            let t = f64::from(k) / 200.0;
            let u = 1.0 - t;
            let (w0, w1, w2, w3) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
            let x = w0 * p[0][0] + w1 * p[1][0] + w2 * p[2][0] + w3 * p[3][0];
            let y = w0 * p[0][1] + w1 * p[1][1] + w2 * p[2][1] + w3 * p[3][1];
            pior = pior.max(((x - centro[0]).hypot(y - centro[1]) - r).abs());
        }
    }
    pior
}

#[test]
fn um_filete_acima_de_90_graus_e_um_circulo_em_duas_metades() {
    let r = 0.05;
    for graus in [30.0, 60.0, 90.0, 100.0, 127.0, 150.0, 170.0] {
        let pts = v_pts(graus);
        let polilinha = crate::corners::round_closed_corners(&pts, &[0.0, r, 0.0, 0.0]);
        let fonte: Vec<VecVertex> = pts
            .iter()
            .enumerate()
            .map(|(i, &p)| VecVertex {
                corner_radius: if i == 1 { r } else { 0.0 },
                ..VecVertex::corner(p)
            })
            .collect();
        let vivo =
            crate::corner_live::round_authored_corners(&fonte, true).expect("a quina arredonda");
        let partes = if graus > 90.0 { 2 } else { 1 };
        for (nome, verts) in [("polilinha", &polilinha.verts), ("vivo", &vivo)] {
            assert_eq!(
                verts.len(),
                4 + partes,
                "[{nome} · {graus}°] a quina tem de sair em {partes} cúbica(s)"
            );
            // A quina é o vértice 1: os que a substituem são `1..=1 + partes`.
            let erro = pior_radial(verts, 1, 1 + partes, graus, r);
            assert!(
                erro <= QUARTO * r,
                "[{nome} · {graus}°] o arco afasta-se {erro:.3e} do círculo (barra {:.3e})",
                QUARTO * r
            );
            if partes == 2 {
                let m = &verts[2];
                assert_eq!(
                    m.kind,
                    VertexKind::Smooth,
                    "[{nome} · {graus}°] o meio é liso"
                );
                let (d_in, d_out) = (
                    [m.anchor[0] - m.in_handle[0], m.anchor[1] - m.in_handle[1]],
                    [m.out_handle[0] - m.anchor[0], m.out_handle[1] - m.anchor[1]],
                );
                assert!(
                    (d_in[0] * d_out[1] - d_in[1] * d_out[0]).abs() < 1e-12
                        && d_in[0] * d_out[0] + d_in[1] * d_out[1] > 0.0,
                    "[{nome} · {graus}°] o meio tem de ser tangente (G1): {m:?}"
                );
            }
        }
    }
}

/// ⚠️ Um blend com um lado CURVO tem recuos diferentes até à interseção das tangentes — não existe
/// círculo tangente às duas nos dois pontos — e fica numa cúbica só, mesmo a virar mais de `90°`.
#[test]
fn um_blend_assimetrico_nao_e_partido() {
    let fonte = vec![
        VecVertex {
            anchor: [-10.0, 0.0],
            in_handle: [-10.0, 0.0],
            out_handle: [-6.0, 6.0],
            kind: VertexKind::Corner,
            corner_radius: 0.0,
        },
        VecVertex {
            anchor: [0.0, 0.0],
            in_handle: [-2.0, 2.0],
            out_handle: [0.0, 0.0],
            kind: VertexKind::Corner,
            corner_radius: 1.5,
        },
        VecVertex::corner([-10.0, -1.0]),
    ];
    // A marcha vira ~`129°` na quina (chega a descer a `−45°`, sai para `−174°`).
    let vivo = crate::corner_live::round_authored_corners(&fonte, true).expect("arredonda");
    assert_eq!(
        vivo.len(),
        4,
        "um blend assimétrico não é um círculo: a quina sai em DOIS vértices, sem meio"
    );
    assert!(
        vivo.iter().all(|v| v
            .anchor
            .iter()
            .chain(&v.in_handle)
            .chain(&v.out_handle)
            .all(|c| c.is_finite())),
        "o blend tem de ser finito: {vivo:?}"
    );
}

/// ⭐⭐ **A QUINA SUAVIZADA TAMBÉM PARTE O ARCO DO MEIO** (2026-09-16) — o terceiro escritor de arco.
///
/// A suavização ([`crate::smooth`]) troca o arco de uma quina por `asa + arco curto + asa`, e o arco
/// curto varre `α·(1 − s)`: numa quina que vira `120°` com `s = 0,05` são `114°` numa cúbica só, a
/// errar `~3×` o quarto de círculo. Hoje só o `RoundRect` a alcança no produto (quinas de `90°`, logo
/// um arco de até `90°`), mas a porta é pública e declara-se *«para QUALQUER ângulo»* — e o polígono e
/// a estrela são a próxima população que a pede.
///
/// ⚠️ O centro do arco curto é o do filete puro (a suavização não o move), e as asas não são arcos:
/// a régua mede só os vértices entre as duas pontas do arco.
#[test]
fn a_quina_suavizada_parte_o_arco_do_meio_acima_de_90_graus() {
    let r = 0.05;
    for graus in [60.0, 90.0, 120.0, 150.0, 170.0] {
        for s in [0.05, 0.2, 0.5] {
            let pts = v_pts(graus);
            let p = crate::corners::round_closed_corners_smooth(&pts, &[0.0, r, 0.0, 0.0], s);
            let arco = graus * (1.0 - s);
            let partes = if arco > 90.0 { 2 } else { 1 };
            // `[quina 0, P0, A, (meio), A', P0', quina 2, quina 3]`: o arco vai do vértice 2 ao `2 + partes`.
            assert_eq!(
                p.verts.len(),
                7 + partes - 1,
                "[{graus}° · s {s}] o arco do meio ({arco:.1}°) tem de sair em {partes} cúbica(s)"
            );
            let erro = pior_radial(&p.verts, 2, 2 + partes, graus, r);
            assert!(
                erro <= QUARTO * r,
                "[{graus}° · s {s}] o arco do meio ({arco:.1}°) afasta-se {erro:.3e} do círculo \
                 (barra {:.3e})",
                QUARTO * r
            );
            if partes == 2 {
                let m = &p.verts[3];
                let (d_in, d_out) = (
                    [m.anchor[0] - m.in_handle[0], m.anchor[1] - m.in_handle[1]],
                    [m.out_handle[0] - m.anchor[0], m.out_handle[1] - m.anchor[1]],
                );
                assert!(
                    m.kind == VertexKind::Smooth
                        && (d_in[0] * d_out[1] - d_in[1] * d_out[0]).abs() < 1e-12
                        && d_in[0] * d_out[0] + d_in[1] * d_out[1] > 0.0,
                    "[{graus}° · s {s}] o meio do arco tem de ser liso e tangente: {m:?}"
                );
            }
        }
    }
}
