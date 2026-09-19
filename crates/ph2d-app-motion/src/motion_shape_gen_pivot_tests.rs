//! **OS GATES DO PIVÔ** do `source.shape` — ordem do dono (2026-09-19): *«crie no nó Shape o
//! offset do Pivot»*.
//!
//! ⚠️ **Irmão do [`super::tests`] pelo tecto de LOC (HR-18) e por PERGUNTA:** ali mede-se a PORTA
//! ÚNICA (a chave que o shell publica e o nó lê) e o catálogo de formas; aqui mede-se **onde a
//! forma se pendura**, que é uma lei sobre a caixa de corte e não sobre a receita de cada espécie.
//!
//! ⛔⛔ **O que este ficheiro existe para impedir é o CONTROLO MORTO** (§5.0): um param que o
//! cartão pinta e que o barro ignora. A única régua que o apanha é medir o PRODUTO com duas
//! posições do knob — *nenhum censo de registo o vê, porque ele está registado.*

use super::{build_shape_path, manifest_default};
use ph2d_node_motion_shape::{ShapeKind, ShapeParams, shape_key};

/// ⭐⭐⭐ **O PIVÔ CHEGA AO BARRO — a pergunta que o §5.0 diz que nenhum instrumento faz.**
///
/// Ordem do dono (2026-09-19): *«crie no nó Shape o offset do Pivot»*. Um param que o cartão
/// pinta e que a forma ignora é o **controlo morto** canónico desta casa, e a única régua que o
/// apanha é medir o PRODUTO com duas posições do knob.
///
/// A lei tem **três** metades, e cada uma mata uma cura barata:
/// 1. `0` é **no-op byte-idêntico** — senão isto mudava toda forma que já shipou;
/// 2. a forma **DESLOCA-SE** pela fracção pedida da própria extensão;
/// 3. e ela **NÃO se deforma** — a extensão fica a mesma. *Sem a 3.ª, um pivô implementado a
///    esticar a caixa (em vez de a transladar) passaria nas duas primeiras.*
#[test]
fn o_pivot_desloca_a_forma_sem_a_deformar() {
    let faixa = |kind: ShapeKind, pivot: [f32; 2]| {
        let p = build_shape_path(&ShapeParams {
            kind,
            size: 1.0,
            pivot,
            ..ShapeParams::read(manifest_default)
        });
        p.verts.iter().fold(
            (f64::MAX, f64::MIN, f64::MAX, f64::MIN),
            |(xl, xh, yl, yh), v| {
                (
                    xl.min(v.anchor[0]),
                    xh.max(v.anchor[0]),
                    yl.min(v.anchor[1]),
                    yh.max(v.anchor[1]),
                )
            },
        )
    };

    // (1) O NEUTRO, e ele é medido contra a forma construída SEM tocar no param.
    let nu = build_shape_path(&ShapeParams {
        kind: ShapeKind::Star,
        ..ShapeParams::read(manifest_default)
    });
    let neutro = build_shape_path(&ShapeParams {
        kind: ShapeKind::Star,
        pivot: [0.0, 0.0],
        ..ShapeParams::read(manifest_default)
    });
    assert_eq!(
        nu.verts.len(),
        neutro.verts.len(),
        "o neutro tem de ser a MESMA forma"
    );
    for (a, b) in nu.verts.iter().zip(&neutro.verts) {
        assert_eq!(a.anchor, b.anchor, "o pivot `0` tem de ser no-op AO BIT");
    }

    // (2) e (3): sobre a `Cross`, que é simétrica nos dois eixos — assim um deslocamento não se
    // pode confundir com a assimetria da própria silhueta.
    let (x0l, x0h, y0l, y0h) = faixa(ShapeKind::Cross, [0.0, 0.0]);
    for (eixo, pivot, esperado) in [
        ("x", [0.5_f32, 0.0], (x0h - x0l) * 0.5),
        ("y", [0.0, -0.25], -(y0h - y0l) * 0.25),
    ] {
        let (xl, xh, yl, yh) = faixa(ShapeKind::Cross, pivot);
        let (andou, largura, alvo) = if eixo == "x" {
            (xl - x0l, xh - xl, x0h - x0l)
        } else {
            (yl - y0l, yh - yl, y0h - y0l)
        };
        assert!(
            (andou - esperado).abs() < 1e-9,
            "no eixo {eixo} a forma tinha de andar {esperado} e andou {andou}"
        );
        assert!(
            (largura - alvo).abs() < 1e-9,
            "no eixo {eixo} a forma DEFORMOU-SE: {largura} contra {alvo}"
        );
    }
}

/// ⭐⭐ **O PIVÔ É UM OFFSET, e o `0` dele não quer dizer «no centro».**
///
/// O osso é cortado de `[0, 2s]` — ele pendura-se na CABEÇA por natureza —, logo `pivot = 0`
/// deixa-o pendurado e **`−0,5` é que o centra**. *É essa a diferença entre um «Pivot» e um
/// «Pivot Offset», e ela é o que o dono pediu pelo nome.*
#[test]
fn o_zero_do_pivot_e_o_pivo_natural_da_especie() {
    let lo_de = |kind: ShapeKind, pivot: [f32; 2]| {
        build_shape_path(&ShapeParams {
            kind,
            size: 1.0,
            pivot,
            ..ShapeParams::read(manifest_default)
        })
        .verts
        .iter()
        .fold(f64::MAX, |m, v| m.min(v.anchor[0]))
    };
    assert!(
        lo_de(ShapeKind::Bone, [0.0, 0.0]).abs() < 1e-9,
        "com o offset a `0` o osso continua PENDURADO na cabeca"
    );
    assert!(
        (lo_de(ShapeKind::Bone, [-0.5, 0.0]) + 1.0).abs() < 1e-9,
        "com `-0,5` ele passa a estar CENTRADO, como qualquer carimbo"
    );
    // ⛔ E o CONTROLO, na direcção oposta: numa forma de pivô natural CENTRADO, é o `+0,5` que a
    // pendura. *Sem ele, um `pivot` que somasse sempre meio semi-eixo passaria a primeira metade.*
    assert!(
        (lo_de(ShapeKind::Circle, [0.0, 0.0]) + 1.0).abs() < 1e-9,
        "um circulo com o offset a `0` fica centrado"
    );
    assert!(
        lo_de(ShapeKind::Circle, [0.5, 0.0]).abs() < 1e-9,
        "e com `+0,5` ele pendura-se pela aresta esquerda"
    );
}

/// ⚠️ **O PIVÔ ENTRA NA CHAVE DA GEOMETRIA** — senão a 1.ª forma cozida volta do cache para todos
/// os outros valores e o controlo fica **inerte depois da primeira vez**.
///
/// É o defeito que o cabeçalho de [`ph2d_node_motion_shape::param::ALL`] narra (o *Pattern Offset*
/// do sculpt3d, 2026-08-09), e a régua é a chave que o shell e o nó calculam pela MESMA porta.
#[test]
fn o_pivot_entra_na_chave_da_geometria() {
    let chave = |pivot: f32| {
        shape_key(|n| {
            if n == ph2d_node_motion_shape::param::PIVOT_X {
                pivot
            } else {
                manifest_default(n)
            }
        })
    };
    assert_ne!(
        chave(0.0),
        chave(0.25),
        "duas posicoes do pivot tem de nomear geometrias DIFERENTES"
    );
    assert_eq!(chave(0.0), chave(0.0), "e a mesma posicao a mesma chave");
}
