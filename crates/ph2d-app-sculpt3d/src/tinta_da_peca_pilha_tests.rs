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
    // A REFERÊNCIA: a peça composta antes de qualquer recomposição — a cor por
    // vértice ainda é o fundo com que a pilha nasceu.
    let mut referencia = obj.tinta.clone().expect("plano");
    let mesh = obj.stack.mesh();
    p.pinta_tinta(&mut referencia, || p.fundo_semeado(mesh, 3));
    let referencia = referencia.amostras().to_vec();
    // CONTROLO: o fundo VÊ-SE (a pilha é mesmo translúcida).
    assert!(
        referencia.iter().any(|c| *c != vermelho),
        "a base a 50 % deixa ver o fundo"
    );
    for vez in 0..4 {
        recompoe(&mut obj);
        let lida = para_ler(&obj, obj.tinta.as_ref().expect("plano"))
            .amostras()
            .to_vec();
        assert!(
            lida == referencia,
            "a recomposição {vez} andou a cor da peça sem nada mudar na pilha"
        );
    }
}

/// ⭐⭐ **Com um Gaussiano por cima de camadas com relevo, a recomposição deixa o relevo POR DOBRAR**
/// (a CPU não paga o calor no gesto: a placa dobra-o no `sync_mesh`, `docs/3D/30` §20) — **e os
/// leitores da CPU dobram-no**: o `para_ler` lê a dobra, e o `em_dia` põe-na na peça. CONTROLO: a
/// dobra borrada não é a nítida.
#[test]
fn o_relevo_por_dobrar_chega_aos_leitores_da_cpu() {
    use crate::objects::{ObjectId, SceneObject};
    use ph2d_tool_painter::{AdjustmentParams, GaussianBlurParams};
    let mesh = crate::scenes::tinta_fina::peca();
    let mut obj = SceneObject::new(ObjectId(1), mesh, ph2d_mesh::Pose::default());
    obj.tinta = Some(crate::tinta_da_peca::semente(obj.stack.mesh(), 3));
    acompanha(obj.tinta.as_mut(), &mut obj.pilha);
    let xs = crate::vizinhanca_da_peca::posicoes(obj.tinta.as_ref().expect("t"), obj.stack.mesh());
    let riscas: Vec<[f32; 2]> = xs
        .iter()
        .map(|&x| crate::scenes::relevo_camadas::relevo_da_base(x))
        .collect();
    let p = obj.pilha.as_mut().expect("pilha");
    let base = p.base().expect("base");
    let n = p.amostras();
    assert!(p.pinta_camada(base, &vec![[200, 190, 180, 255]; n], Some(riscas)));
    crate::scenes::relevo_borrado::poe_o_desfoque(p, obj.stack.mesh());
    recompoe(&mut obj);
    let bits = |r: Option<&[[f32; 2]]>| {
        r.map(|r| r.iter().map(|x| x.map(f32::to_bits)).collect::<Vec<_>>())
    };
    let nitido = bits(obj.tinta.as_ref().and_then(|t| t.relevo()));
    assert!(nitido.is_some(), "a peça tem o relevo das riscas");
    let ph2d_tool_painter::SpatialUnits::Surface { size } =
        crate::vizinhanca_da_peca::unidades(obj.stack.mesh())
    else {
        panic!("unidades da peça")
    };
    let p = obj.pilha.as_mut().expect("pilha");
    let topo = p.pilha().root()[0];
    let mut m = p.pilha().clone();
    m.adjustment_mut(topo).expect("o Gaussiano").params =
        AdjustmentParams::GaussianBlur(GaussianBlurParams {
            radius: ph2d_tool_painter::SURFACE_RADIUS_MAX * size,
        });
    p.troca_metadado(m).expect("o raio");
    recompoe(&mut obj);
    let p = obj.pilha.as_ref().expect("pilha");
    assert!(p.relevo_por_dobrar(), "o raio deixou o relevo por dobrar");
    assert_eq!(
        bits(obj.tinta.as_ref().and_then(|t| t.relevo())),
        nitido,
        "a recomposição pagou o calor na CPU"
    );
    let referencia = bits(p.relevo_composto().as_deref());
    assert_ne!(
        referencia, nitido,
        "CONTROLO: a dobra borrada não é a nítida"
    );
    let lida = bits(para_ler(&obj, obj.tinta.as_ref().expect("plano")).relevo());
    assert_eq!(lida, referencia, "o para_ler não leu a dobra");
    em_dia(&mut obj);
    assert!(!obj.pilha.as_ref().expect("p").relevo_por_dobrar());
    assert_eq!(
        bits(obj.tinta.as_ref().and_then(|t| t.relevo())),
        referencia,
        "o em_dia não pôs a dobra na peça"
    );
}
