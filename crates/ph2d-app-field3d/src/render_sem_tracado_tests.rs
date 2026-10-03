//! ⭐⭐⭐ **O modo Render não chega ao traçado** (ordem do dono, 03/10: *«pode apagar o render
//! antigo»*). O Render desenha por malha; sem aparelho ele mostra o Matcap (o único traçado que
//! fica), e nenhuma variável de ambiente o devolve ao traçado antigo.

use crate::scene::lasso_tests::{armed_with, two_balls};
use crate::shading::Shading;
use std::sync::atomic::Ordering;

/// Desenha até o quadro do viewport activo assentar (nada em voo, o pedido cheio respondido).
fn quadro_assente() -> Vec<u8> {
    let mut text = ph2d_text::TextSystem::without_system_fonts();
    let t = std::time::Instant::now();
    loop {
        let mut scene = ph2d_vector::VectorScene::new();
        crate::smoke::draw(
            crate::scene::lasso_tests::AREA,
            ph2d_tokens::Theme::default(),
            &mut text,
            &mut scene,
        );
        let pronto = crate::smoke::with_smoke(|s| {
            let v = s.vp();
            let assente = v.requested.as_ref().is_some_and(|r| !r.4);
            (v.inflight.is_none() && assente)
                .then(|| v.frame.as_ref().map(|f| f.rgba().to_vec()))
                .flatten()
        })
        .flatten();
        if let Some(rgba) = pronto {
            return rgba;
        }
        assert!(t.elapsed().as_secs() < 120, "o quadro não assentou em 120 s");
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}

fn escolhe(sim: &mut ph2d_ecs::SimWorld, modo: Shading) {
    let slot = Shading::ALL
        .iter()
        .position(|s| *s == modo)
        .expect("o modo está na fileira");
    ph2d_panel_model3d::state::push_intent_for_test(ph2d_panel_model3d::ModelIntent::SetShading {
        slot,
    });
    crate::scene::apply_intents_for_test(sim.world_mut(), &[]);
    let t = std::time::Instant::now();
    loop {
        crate::scene::ecs_bridge(sim, None, &[], &crate::scene::no_drawing());
        let pronto = modo == Shading::Matcap
            || crate::malha_render_estado::com(|e| !e.objetos.is_empty() && !e.esperando())
                .unwrap_or(false);
        if pronto {
            break;
        }
        assert!(t.elapsed().as_secs() < 60, "a malha do Render não ficou pronta em 60 s");
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    crate::smoke::with_smoke(|s| {
        let v = s.vp_mut();
        v.frame = None;
        v.requested = None;
    });
}

/// ⭐ Sem aparelho, o Render mostra EXACTAMENTE o quadro do Matcap — nunca o sombreado do traçado
/// antigo (luz, sombra, ricochete, chão), que era o recuo até 03/10.
#[test]
fn o_render_sem_aparelho_mostra_o_matcap_e_nao_o_tracado() {
    armed_with(&two_balls(), |sim| {
        let matcap = quadro_assente();
        crate::malha_render_quadro::SEM_APARELHO.store(true, Ordering::Relaxed);
        escolhe(sim, Shading::Render);
        let render = quadro_assente();
        crate::malha_render_quadro::SEM_APARELHO.store(false, Ordering::Relaxed);
        escolhe(sim, Shading::Matcap);
        assert_eq!(matcap.len(), render.len(), "o mesmo tamanho de quadro");
        let diferentes = matcap
            .chunks_exact(4)
            .zip(render.chunks_exact(4))
            .filter(|(a, b)| a != b)
            .count();
        assert_eq!(
            diferentes,
            0,
            "o Render sem aparelho difere do Matcap em {diferentes} de {} px — chegou ao traçado",
            matcap.len() / 4
        );
    });
}

/// ⭐ Nenhum código do repositório lê a porta de bissecção do traçado antigo.
#[test]
fn ninguem_le_a_porta_do_render_tracado() {
    let agulha = concat!("PH2D_FIELD_", "RENDER_TRACADO");
    let raiz = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut leitores = Vec::new();
    let mut pilha = vec![raiz.join("crates"), raiz.join("shells")];
    let mut vistos = 0usize;
    while let Some(d) = pilha.pop() {
        let Ok(it) = std::fs::read_dir(&d) else {
            continue;
        };
        for e in it.flatten() {
            let p = e.path();
            if p.is_dir() {
                if p.file_name().is_some_and(|n| n != "target") {
                    pilha.push(p);
                }
            } else if p.extension().is_some_and(|x| x == "rs") {
                vistos += 1;
                if std::fs::read_to_string(&p).is_ok_and(|s| s.contains(agulha)) {
                    leitores.push(p);
                }
            }
        }
    }
    assert!(vistos > 1000, "a varredura não viu o repositório ({vistos} ficheiros)");
    assert!(leitores.is_empty(), "ainda lêem {agulha}: {leitores:#?}");
}
