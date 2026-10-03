//! Gates da pilha que anda com o plano (`docs/3D/30` §11, W2).

use super::*;
use ph2d_mesh::shapes;

/// ⭐⭐⭐ **GATE — Quando a pilha nasce, o plano É recomposto dela**: as cores
/// `f32` de fora da grelha de bytes passam à grelha (a composição da camada
/// RGBA8), e a 2.ª chamada não muda nada.
#[test]
fn quando_a_pilha_nasce_o_plano_e_a_composicao_dela() {
    let m = shapes::octahedron(1.0);
    let faces = || m.faces().iter().map(ph2d_mesh::Face::verts);
    let mut t = Some(Tinta::nova(m.vert_count(), faces(), 2));
    for c in t.as_mut().expect("plano").amostras_mut() {
        *c = [1.0 / 3.0, 0.123_456, 0.9];
    }
    let mut pilha = None;
    assert!(
        acompanha(t.as_mut(), &mut pilha),
        "a pilha nasceu e o plano mudou"
    );
    let grelha = |x: f32| (x * 255.0).round() / 255.0;
    for c in t.as_ref().expect("plano").amostras() {
        assert_eq!(*c, [grelha(1.0 / 3.0), grelha(0.123_456), grelha(0.9)]);
    }
    assert!(!acompanha(t.as_mut(), &mut pilha), "a 2.ª vez nada muda");
    assert!(
        !acompanha(None, &mut pilha) && pilha.is_none(),
        "sem plano não há pilha"
    );
}

/// ⭐⭐⭐ **GATE — A pilha ESTACIONA e VOLTA com o plano** (desarmar e rearmar
/// o detalhe fino não perde as camadas).
#[test]
fn a_pilha_estaciona_e_volta_com_o_plano() {
    let m = shapes::octahedron(1.0);
    let (mut t, mut parque, mut pilha, mut pilha_parque) = (None, None, None, None);
    let garante = |nivel, t: &mut _, parque: &mut _, pilha: &mut _, pp: &mut _| {
        garante_com_pilha(&m, t, parque, pilha, pp, nivel, u64::MAX)
    };
    assert!(garante(
        Some(2),
        &mut t,
        &mut parque,
        &mut pilha,
        &mut pilha_parque
    ));
    let cima = pilha
        .as_mut()
        .map(|p: &mut PilhaDaPeca| p.nova_camada("cima").expect("camada"));
    let com_duas = pilha.clone();
    assert!(
        garante(None, &mut t, &mut parque, &mut pilha, &mut pilha_parque),
        "desarmar"
    );
    assert!(t.is_none() && pilha.is_none() && parque.is_some());
    assert_eq!(pilha_parque, com_duas, "a pilha foi estacionar com o plano");
    assert!(
        garante(Some(2), &mut t, &mut parque, &mut pilha, &mut pilha_parque),
        "rearmar"
    );
    assert_eq!(
        pilha, com_duas,
        "e voltou com ele — a camada de cima incluída"
    );
    assert!(
        pilha
            .as_ref()
            .and_then(|p| cima.and_then(|c| p.plano(c)))
            .is_some()
    );
}

/// ⛔⛔ **GATE — Uma pilha TRANSLÚCIDA recomposta FICA** (`docs/3D/30` §13).
/// O fundo lia-se da cor por vértice VIVA, que a recomposição reescreve com o
/// composto: cada passo do arrasto, desfazer ou balde andava a cor (`50`, `37`,
/// `27` degraus de sRGB8). O fundo fixado quando a pilha nasce cura-o.
#[test]
fn uma_pilha_translucida_recomposta_fica() {
    use crate::objects::{ObjectId, SceneObject};
    let mesh = crate::scenes::tinta_fina::peca();
    let mut obj = SceneObject::new(ObjectId(1), mesh, ph2d_mesh::Pose::default());
    obj.tinta = Some(crate::tinta_da_peca::semente(obj.stack.mesh(), 3));
    acompanha(obj.tinta.as_mut(), &mut obj.pilha);
    let p = obj.pilha.as_mut().expect("pilha");
    let base = p.base().expect("base");
    p.define_opacidade(base, 0.5);
    let n = p.amostras();
    // A base PINTADA de vermelho (a que nasceu da semente compõe-se nela mesma).
    p.plano_mut(base)
        .expect("plano")
        .escreve(&vec![[230, 20, 20, 255]; n], None);
    let vermelho = [230.0 / 255.0, 20.0 / 255.0, 20.0 / 255.0];
    let mut primeira: Option<Vec<[f32; 3]>> = None;
    for vez in 0..4 {
        recompoe(&mut obj);
        let lida = para_ler(&obj, obj.tinta.as_ref().expect("plano"))
            .amostras()
            .to_vec();
        match &primeira {
            None => {
                // CONTROLO: o fundo VÊ-SE (a pilha é mesmo translúcida).
                assert!(
                    lida.iter().any(|c| *c != vermelho),
                    "a base a 50 % deixa ver o fundo"
                );
                primeira = Some(lida);
            }
            Some(p) => assert!(
                lida == *p,
                "a recomposição {vez} andou a cor da peça sem nada mudar na pilha"
            ),
        }
    }
}
