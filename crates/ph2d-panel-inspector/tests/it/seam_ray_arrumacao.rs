//! **O cartão Ray Sensor arrumado por pergunta** (dono 15/11: *«Layers e Sees deveriam ficar numa seção própria
//! mais próximos»*). Auditoria: `docs/GameEngineFuncional/componentes/07_auditoria_arrumacao_2026-11-15.md`.

use crate::o_cartao_da_fisica::{achado, clica};
use crate::seam_ray::{assente_com, nos_dois_modos, pinta, raio};
use ph2d_editor_core::zones::Rect;
use ph2d_panel_inspector::ids as i;
use ph2d_panel_inspector::set_current_inspector_ray;

const FOLGA: f32 = 0.5; // LITERAL-PX-OK: arredondamento de rects

/// ⭐⭐ A ordem pintada, de cima para baixo, é a das perguntas: Enabled < Angle < Aim at < Reach ≈ Shape <
/// cabeçalho Layers < Layer < Sees < Rename layers (só cartões) < cabeçalho Filter < cabeçalho Signals; a Layer e o
/// primeiro quadro do Sees são vizinhos (nenhum controlo entre eles); o clique REAL no cabeçalho Layers dobra e
/// desdobra Layer + Sees + botões.
///
/// **Mutação que deve sangrar:** em `ray_camadas.rs::sees_e_botoes`, pintar os botões ANTES do Sees (a ordem
/// antiga) — os botões ficam entre a Layer e o Sees.
#[test]
fn o_cartao_ray_esta_arrumado_por_pergunta() {
    nos_dois_modos(|c| {
        let (mut h, mut st, r) = assente_com(&raio(), c, false);
        let q = |id, nome: &str| achado(&r, id).unwrap_or_else(|| panic!("{nome} (cartões {c})"));
        let enabled = q(i::INSP_RAY_ENABLED, "Enabled");
        let angle = q(i::INSP_RAY_ANGLE, "Angle");
        let aim = q(i::INSP_RAY_AIM, "Aim at");
        let reach = q(i::INSP_RAY_REACH, "Reach");
        let shape = q(i::INSP_RAY_SHAPE, "Shape");
        let layers = q(i::INSP_RAY_FOLD_LAYERS, "o cabeçalho Layers");
        let layer = q(i::INSP_RAY_LAYER, "Layer");
        let sees = q(i::INSP_RAY_SEES[0], "o 1.º quadro do Sees");
        let filtro = q(i::INSP_RAY_FOLD_FILTER, "o cabeçalho Filter");
        let sinais = q(i::INSP_RAY_FOLD_SIGNALS, "o cabeçalho Signals");
        let topo = |a: Rect| a.y;
        let mut ordem = vec![
            ("Enabled", (enabled.y, enabled.x)),
            ("Angle", (angle.y, angle.x)),
            ("Aim at", (aim.y, aim.x)),
            ("Reach", (reach.y, reach.x)),
            ("Shape", (shape.y, shape.x)),
            ("Layers", (layers.y, 0.0)),
            ("Layer", (layer.y, 0.0)),
            ("Sees", (sees.y, 0.0)),
        ];
        if c {
            // Reach | Shape lado a lado nos cartões (a mesma linha).
            assert!(
                (topo(reach) - topo(shape)).abs() <= FOLGA,
                "Reach e Shape na mesma linha: {reach:?} {shape:?}"
            );
            let ren = q(i::INSP_RAY_LAYER_RENAME, "Rename layers…");
            ordem.push(("Rename layers", (ren.y, 0.0)));
        }
        ordem.push(("Filter", (filtro.y, 0.0)));
        ordem.push(("Signals", (sinais.y, 0.0)));
        for par in ordem.windows(2) {
            assert!(
                par[0].1 < par[1].1,
                "{} vem antes de {} (cartões {c}): {:?}",
                par[0].0,
                par[1].0,
                ordem
            );
        }
        // Layer e Sees vizinhos: nenhum controlo pintado no vão entre eles.
        let vao = (layer.y + layer.h - FOLGA, sees.y + FOLGA);
        let do_sees = |id| i::INSP_RAY_SEES.contains(&id) || id == i::INSP_RAY_SEES_APPLIES;
        let intrusos: Vec<_> = r
            .iter()
            .filter(|(id, q)| {
                q.w > 0.0 && q.h > 0.0 && !do_sees(*id) && q.y >= vao.0 && q.y + q.h <= vao.1
            })
            .collect();
        assert!(
            intrusos.is_empty(),
            "entre a Layer e o Sees (cartões {c}): {intrusos:?}"
        );
        // O cabeçalho Layers dobra e desdobra o grupo, pelo clique real.
        for (aberto, n) in [(false, "fechado"), (true, "reaberto")] {
            let cab = achado(&pinta(&mut h, &mut st, c), i::INSP_RAY_FOLD_LAYERS)
                .expect("o cabeçalho Layers");
            let _ = clica(&mut h, &mut st, cab);
            let r = pinta(&mut h, &mut st, c);
            let mut dentro = vec![i::INSP_RAY_LAYER, i::INSP_RAY_SEES[0]];
            if c {
                dentro.push(i::INSP_RAY_LAYER_RENAME);
            }
            for id in dentro {
                assert_eq!(
                    achado(&r, id).is_some(),
                    aberto,
                    "{id:?} com o grupo {n} (cartões {c})"
                );
            }
            assert!(achado(&r, i::INSP_RAY_FOLD_FILTER).is_some(), "Filter fica");
        }
        set_current_inspector_ray(None);
    });
}
