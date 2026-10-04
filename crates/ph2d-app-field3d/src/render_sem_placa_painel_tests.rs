//! ⭐⭐⭐ **SEM PLACA, AS FILEIRAS DO RENDER FICAM À VISTA E INERTES** — decisão do dono, 03/10:
//! *«se sem placa de vídeo, controles ficam inativos mas não somem»*. Sem aparelho o Render mostra o
//! Matcap, e o estilo, o brilho, o céu e a textura não têm leitor: ficam apagados, com a razão.

use ph2d_field::Param;
use std::sync::atomic::Ordering;

#[test]
fn sem_placa_as_fileiras_do_render_ficam_a_vista_e_inertes() {
    let _trava = crate::malha_render_quadro::TRAVA_SEM_APARELHO
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let doc = crate::scene::lasso_tests::two_balls();
    crate::scene::lasso_tests::armed_with(&doc, |sim| {
        crate::malha_render_quadro::SEM_APARELHO.store(true, Ordering::Relaxed);
        crate::smoke::with_smoke(|s| s.set_shading(crate::shading::Shading::Render));
        let world = sim.world_mut();
        let mut q = world.query::<(bevy_ecs::entity::Entity, &ph2d_field_ecs::FieldObject)>();
        let root = q.iter(world).next().map(|(e, _)| e).expect("a peça");
        crate::scene::publish_snapshot(world, root, &[], 2.4, 0.0);
        let rows = ph2d_panel_model3d::state::current().rows;
        crate::malha_render_quadro::SEM_APARELHO.store(false, Ordering::Relaxed);
        // ⚠️ O CONTROLO: com placa (quando a máquina a tem) nenhuma fileira leva esta razão.
        if crate::malha_render_quadro::tem_aparelho() {
            crate::scene::publish_snapshot(world, root, &[], 2.4, 0.0);
            let com_placa = ph2d_panel_model3d::state::current().rows;
            assert!(
                com_placa
                    .iter()
                    .all(|r| r.inert != Some(super::RAZAO_SEM_PLACA)),
                "com placa uma fileira do Render ficou apagada por falta de placa"
            );
        }
        crate::smoke::with_smoke(|s| s.set_shading(crate::shading::Shading::Matcap));

        let do_render: Vec<_> = rows
            .iter()
            .filter(|r| {
                matches!(
                    r.param,
                    Param::Style(_) | Param::Bloom(_) | Param::Sky(_) | Param::Texture(_)
                )
            })
            .collect();
        // ⭐ A população primeiro: as três secções da cena TÊM de estar à vista.
        for (nome, achou) in [
            (
                "estilo",
                do_render.iter().any(|r| matches!(r.param, Param::Style(_))),
            ),
            (
                "brilho",
                do_render.iter().any(|r| matches!(r.param, Param::Bloom(_))),
            ),
            (
                "céu",
                do_render.iter().any(|r| matches!(r.param, Param::Sky(_))),
            ),
        ] {
            assert!(
                achou,
                "sem placa as fileiras do {nome} sumiram — o dono quer-nas à vista"
            );
        }
        let vivas: Vec<_> = do_render
            .iter()
            .filter(|r| r.inert != Some(super::RAZAO_SEM_PLACA))
            .map(|r| r.key)
            .collect();
        assert!(
            vivas.is_empty(),
            "sem placa estas fileiras do Render aceitam o gesto sem leitor nenhum: {vivas:?}"
        );
    });
}
