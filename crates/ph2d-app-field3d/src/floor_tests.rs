//! Os gates da âncora do chão — ver [`super`].

use super::anchored;
use crate::shading::Shading;
use ph2d_field::{FieldDoc, NodeId, Primitive, Xform};
use ph2d_field_eval::hybrid::Registry;

fn esfera(y: f32) -> FieldDoc {
    FieldDoc::new(
        vec![ph2d_field_eval::leaf(
            Primitive::Sphere { radius: 0.25 },
            Xform::at(0.0, y, 0.0),
        )],
        NodeId(0),
    )
    .expect("a esfera")
}

/// ⭐⭐⭐ **Levantar a peça NÃO levanta o chão** — é o que deixa a sombra separar-se dela (a régua da
/// `W4`). E voltar a ligar o Render pousa-o outra vez.
#[test]
fn o_chao_fica_onde_foi_pousado_ate_o_render_voltar_a_ligar() {
    let reg = Registry::new();
    let mut ancora = None;
    let pousada = anchored(&mut ancora, Shading::Render, &esfera(0.25), &reg)
        .expect("o Render ligado tem chão");
    assert!(
        pousada.height.abs() < 1e-3,
        "a esfera pousa em 0: {pousada:?}"
    );
    let levantada =
        anchored(&mut ancora, Shading::Render, &esfera(0.35), &reg).expect("continua a haver chão");
    assert_eq!(
        levantada, pousada,
        "levantar a peça não pode levar o chão com ela — a sombra nunca se separaria"
    );
    // ⚠️ O Render voltou a ligar (quem o faz é o `Smoke::set_shading`, que esquece a âncora).
    ancora = None;
    let de_novo = anchored(&mut ancora, Shading::Render, &esfera(0.35), &reg).expect("chão");
    assert!(
        (de_novo.height - 0.10).abs() < 1e-3,
        "voltar a ligar pousa o chão sob a peça outra vez: {de_novo:?}"
    );
}

/// ⚠️ **Fora do Render não há chão**, e a âncora não é lida: o matcap é sombreamento de vista.
#[test]
fn fora_do_render_nao_ha_chao() {
    let mut ancora = None;
    assert_eq!(
        anchored(
            &mut ancora,
            Shading::Matcap,
            &esfera(0.25),
            &Registry::new()
        ),
        None
    );
    assert_eq!(
        ancora, None,
        "o matcap não pode ancorar um chão que não usa"
    );
}

/// ⭐ **Mudar para o Render esquece a âncora** — a outra metade da lei, que vive no estado do módulo.
#[test]
fn ligar_o_render_esquece_a_ancora() {
    crate::scene::lasso_tests::armed_with(&esfera(0.25), |_| {
        crate::smoke::with_smoke(|s| {
            s.set_shading(Shading::Matcap);
            s.floor = Some(0.7);
            s.set_shading(Shading::Render);
            assert_eq!(s.floor, None, "ligar o Render tem de pousar o chão de novo");
            s.floor = Some(0.7);
            s.set_shading(Shading::Render);
            assert_eq!(
                s.floor,
                Some(0.7),
                "pedir o modo que já está ligado não é ligá-lo: a âncora fica"
            );
        })
        .expect("o módulo está armado");
    });
}
