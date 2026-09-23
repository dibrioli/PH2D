//! Os gates do SNAPSHOT da secção CAMERA — as quatro colunas derivadas.
//!
//! ⚠️ **O que se prova aqui é o que o painel NÃO poderia calcular**: quem manda, se o alvo existe,
//! e se a cerca cabe na janela. Os campos que são cópia direta do componente não têm gate, e é de
//! propósito — *um gate que copia a atribuição que testa mede o teste*.

use ph2d_core::Vec2;
use ph2d_ecs::{
    CameraFollow, CameraLimits, GameCamera, Name, SimWorld, Transform, assign_missing_stable_ids,
};

use super::build_camera_info;

const ASPECT: f32 = 16.0 / 9.0;

fn cena() -> SimWorld {
    SimWorld::default()
}

fn camera(sim: &mut SimWorld, nome: &str, cam: GameCamera) -> ph2d_ecs::Entity {
    let e = sim
        .world_mut()
        .spawn((
            Transform::from_translation(Vec2::ZERO),
            Name::new(nome),
            cam,
        ))
        .id();
    assign_missing_stable_ids(sim.world_mut());
    e
}

/// ⚠️ **Sem `GameCamera` não há secção** — é o ADR-0166. Um objecto com `CameraFollow` sozinho não
/// pinta nada, e isso está certo: seguir sem enquadrar não é uma pergunta.
#[test]
fn an_object_without_a_camera_has_no_section() {
    let mut sim = cena();
    let e = sim
        .world_mut()
        .spawn((
            Transform::from_translation(Vec2::ZERO),
            Name::new("Nada"),
            CameraFollow::default(),
        ))
        .id();
    assign_missing_stable_ids(sim.world_mut());
    assert!(
        build_camera_info(sim.world_mut(), e.to_bits(), 1, ASPECT, false).is_none(),
        "o Follow sozinho nao abre a seccao"
    );
}

/// ⭐⭐ **A coluna «esta é a que manda»** — sem ela o artista afina uma câmera que ninguém usa.
///
/// **Mutação que deve sangrar:** pôr `is_active_camera = true` incondicionalmente.
#[test]
fn the_snapshot_says_which_camera_commands() {
    let mut sim = cena();
    let baixa = camera(
        &mut sim,
        "Baixa",
        GameCamera {
            priority: 1,
            ..GameCamera::default()
        },
    );
    let alta = camera(
        &mut sim,
        "Alta",
        GameCamera {
            priority: 5,
            ..GameCamera::default()
        },
    );
    let i = build_camera_info(sim.world_mut(), baixa.to_bits(), 1, ASPECT, false).unwrap();
    assert_eq!(i.camera_count, 2);
    assert!(!i.is_active_camera, "a de prioridade 1 NAO manda");
    let i = build_camera_info(sim.world_mut(), alta.to_bits(), 1, ASPECT, false).unwrap();
    assert!(i.is_active_camera, "a de prioridade 5 manda");
}

/// ⭐⭐ **Um alvo por achar é RELATADO — e um alvo VAZIO não é «por achar».**
///
/// ⚠️ A distinção é o que impede o painel de gritar sobre uma câmera fixa correcta.
///
/// **Mutação que deve sangrar:** tirar o `!f.target.trim().is_empty() &&`.
#[test]
fn an_empty_target_is_not_a_missing_one() {
    let mut sim = cena();
    let cam = camera(&mut sim, "Cam", GameCamera::default());

    // Vazio: não segue ninguém — e NÃO é um alvo perdido.
    sim.world_mut()
        .entity_mut(cam)
        .insert(CameraFollow::default());
    let i = build_camera_info(sim.world_mut(), cam.to_bits(), 1, ASPECT, false).unwrap();
    assert!(!i.follow.as_ref().unwrap().target_found);
    assert!(i.follow.as_ref().unwrap().target.is_empty());

    // Um nome que não é de ninguém: PERDIDO.
    sim.world_mut().entity_mut(cam).insert(CameraFollow {
        target: "NaoExiste".into(),
        ..CameraFollow::default()
    });
    let i = build_camera_info(sim.world_mut(), cam.to_bits(), 1, ASPECT, false).unwrap();
    assert!(!i.follow.as_ref().unwrap().target_found, "nao existe");

    // Um nome que existe: ACHADO.
    sim.world_mut()
        .spawn((Transform::from_translation(Vec2::ZERO), Name::new("Heroi")));
    assign_missing_stable_ids(sim.world_mut());
    sim.world_mut().entity_mut(cam).insert(CameraFollow {
        target: "Heroi".into(),
        ..CameraFollow::default()
    });
    let i = build_camera_info(sim.world_mut(), cam.to_bits(), 1, ASPECT, false).unwrap();
    assert!(i.follow.as_ref().unwrap().target_found, "o Heroi existe");
}

/// ⭐⭐⭐ **A cerca mais estreita que a janela é NOMEADA** — e a condição é a MESMA da lei.
///
/// ⚠️ É o aviso que só o snapshot pode dar: ele depende da proporção do ecrã e da altura da câmera,
/// e não dos quatro números que o painel mostra. Quando ele é verdadeiro a câmera **fixa-se no
/// centro da caixa** e deixa de seguir — comportamento medido no oráculo do Godot.
///
/// **Mutação que deve sangrar:** trocar o `>` por `>=`, ou medir só um eixo.
#[test]
fn a_fence_narrower_than_the_window_is_named() {
    let mut sim = cena();
    // altura 10 ⇒ meia-altura 5, meia-largura 5·(16/9) = 8,888…
    let cam = camera(
        &mut sim,
        "Cam",
        GameCamera {
            height_world: 10.0,
            ..GameCamera::default()
        },
    );

    // Larga nos dois eixos: cabe.
    sim.world_mut().entity_mut(cam).insert(CameraLimits {
        min: [-100.0, -100.0],
        max: [100.0, 100.0],
    });
    let i = build_camera_info(sim.world_mut(), cam.to_bits(), 1, ASPECT, false).unwrap();
    assert!(!i.limits.as_ref().unwrap().smaller_than_view);

    // Estreita SÓ em X (largura 10 < 17,78): tem de acusar.
    sim.world_mut().entity_mut(cam).insert(CameraLimits {
        min: [-5.0, -100.0],
        max: [5.0, 100.0],
    });
    let i = build_camera_info(sim.world_mut(), cam.to_bits(), 1, ASPECT, false).unwrap();
    assert!(
        i.limits.as_ref().unwrap().smaller_than_view,
        "estreita num eixo ja' e' estreita — a camera fixa-se no centro da caixa"
    );

    // Estreita SÓ em Y (altura 8 < 10).
    sim.world_mut().entity_mut(cam).insert(CameraLimits {
        min: [-100.0, -4.0],
        max: [100.0, 4.0],
    });
    let i = build_camera_info(sim.world_mut(), cam.to_bits(), 1, ASPECT, false).unwrap();
    assert!(
        i.limits.as_ref().unwrap().smaller_than_view,
        "o eixo Y conta"
    );
}

/// ⭐ **A condição do aviso é a MESMA que a lei usa para fixar** — se elas divergirem, o painel
/// mente sobre o que a câmera faz.
///
/// ⚠️ Este gate não copia a fórmula: ele pergunta ao PRODUTO dos dois lados — o aviso, e o sítio
/// onde a lei de facto põe a câmera.
#[test]
fn the_warning_agrees_with_where_the_law_actually_pins() {
    let mut sim = cena();
    let cam = camera(
        &mut sim,
        "Cam",
        GameCamera {
            height_world: 10.0,
            ..GameCamera::default()
        },
    );
    let lim = CameraLimits {
        min: [2.0, -100.0],
        max: [5.0, 100.0],
    };
    sim.world_mut().entity_mut(cam).insert(lim.clone());
    let i = build_camera_info(sim.world_mut(), cam.to_bits(), 1, ASPECT, false).unwrap();
    assert!(
        i.limits.as_ref().unwrap().smaller_than_view,
        "o aviso acende"
    );

    // E a lei de facto FIXA no centro da caixa: (2 + 5)/2 = 3,5, qualquer que seja o alvo.
    let half = ph2d_ecs::half_extent(10.0, ASPECT);
    for alvo in [-1000.0_f32, 0.0, 3.5, 1000.0] {
        let c = ph2d_ecs::clamp_axis_to_limits(alvo, half[0], lim.min[0], lim.max[0]);
        assert!(
            (c - 3.5).abs() < 1e-4,
            "o aviso diz «fixa no centro» e a lei poe em {c} com alvo {alvo}"
        );
    }
}

/// A pré-visualização é ESTADO DA SHELL e viaja pelo snapshot — o painel não tem como o saber.
#[test]
fn the_preview_flag_travels_in_the_snapshot() {
    let mut sim = cena();
    let cam = camera(&mut sim, "Cam", GameCamera::default());
    let off = build_camera_info(sim.world_mut(), cam.to_bits(), 1, ASPECT, false).unwrap();
    let on = build_camera_info(sim.world_mut(), cam.to_bits(), 1, ASPECT, true).unwrap();
    assert!(!off.preview_on);
    assert!(on.preview_on);
}

/// ⭐ **A faixa do `Dolly` é UMA porta** (auditoria 26, §2.6).
///
/// ⛔ A mutação que a auditoria achou SOBREVIVENTE era a ponta de cima do applier (`0,9 → 0,99`):
/// o painel e o applier escreviam a faixa cada um com o seu literal, e divergirem não partia teste
/// nenhum. Hoje as duas pontas são constantes do `ph2d-editor-core` e este gate afirma as duas
/// metades: **quem as lê** (o applier e a pista do painel, sem literal nenhum) e **que a de cima
/// fica DENTRO do domínio da lei** (`δ < 1`, e a escala existe para toda camada da cena de smoke).
#[test]
fn a_faixa_do_dolly_e_uma_porta_dentro_do_dominio_da_lei() {
    use ph2d_editor_core::screens::hero::{DOLLY_MAX, DOLLY_MIN};
    let applier = include_str!("camera_inspector.rs");
    let pista = include_str!("../../ph2d-panel-inspector/src/populate_camera.rs");
    for (nome, fonte) in [("o applier", applier), ("a pista", pista)] {
        assert!(
            fonte.contains("DOLLY_MIN") && fonte.contains("DOLLY_MAX"),
            "{nome} deixou de ler a faixa partilhada do dolly"
        );
    }
    // ⚠️ Duas constantes: o compilador dobra a asserção, logo ela vive num bloco `const` (e uma
    // faixa que exclua a identidade, ou que chegue a `δ = 1`, passa a ser erro de COMPILAÇÃO).
    const {
        assert!(
            DOLLY_MIN < 0.0 && DOLLY_MAX > 0.0,
            "a faixa tem de conter a identidade"
        )
    };
    const {
        assert!(
            DOLLY_MAX < 1.0,
            "em δ = 1 o plano do mundo tem tamanho zero"
        )
    };
    for k in [
        crate::parallax_smoke::K_ARVORES,
        crate::parallax_smoke::K_COLINAS,
        crate::parallax_smoke::K_CEU,
    ] {
        for d in [DOLLY_MIN, DOLLY_MAX] {
            assert!(
                ph2d_ecs::ScrollFactor::escala_do_dolly(k, d).is_some(),
                "k {k} · δ {d}: a ponta da faixa sai do dominio da lei"
            );
        }
    }
}
