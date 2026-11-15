//! **O cartão Ray Sensor arrumado por pergunta** (dono 15/11: *«Layers e Sees deveriam ficar numa seção própria
//! mais próximos»*). Auditoria: `docs/GameEngineFuncional/componentes/07_auditoria_arrumacao_2026-11-15.md`.

use crate::o_cartao_da_fisica::{achado, clica};
use crate::seam_ray::{assente_com, nos_dois_modos, pinta, raio};
use ph2d_editor_core::zones::Rect;
use ph2d_panel_inspector::ids as i;
use ph2d_panel_inspector::set_current_inspector_ray;

const FOLGA: f32 = 0.5; // LITERAL-PX-OK: arredondamento de rects

/// ⭐⭐ A ordem pintada, de cima para baixo, é a das perguntas: Enabled < Angle < Reach ≈ Aim at < Shape ≈ número <
/// cabeçalho Layers < Layer < Sees < Rename layers (só cartões) < cabeçalho Filter < cabeçalho Signals. Em cartões
/// (dono 15/11) o Reach | Aim at e a Shape | Thickness ou Cone partilham a linha, o primeiro à esquerda; fora deles
/// empilham-se nessa ordem; com Line não há número. A Layer e o primeiro quadro do Sees são vizinhos (nenhum
/// controlo entre eles); o clique REAL no cabeçalho Layers dobra e desdobra Layer + Sees + botões.
///
/// **Mutações que devem sangrar:** em `ray_camadas.rs::sees_e_botoes`, pintar os botões ANTES do Sees (a ordem
/// antiga) — os botões ficam entre a Layer e o Sees; em `ray_rumo.rs::alcance_e_mira`, pintar o Aim at na largura
/// toda (`x, w`) — ele deixa de estar à direita do Reach.
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
        assert!(
            achado(&r, i::INSP_RAY_THICKNESS).is_none(),
            "Line ⇒ sem Thickness (cartões {c})"
        );
        assert!(
            achado(&r, i::INSP_RAY_CONE).is_none(),
            "Line ⇒ sem Cone (cartões {c})"
        );
        let layers = q(i::INSP_RAY_FOLD_LAYERS, "o cabeçalho Layers");
        let layer = q(i::INSP_RAY_LAYER, "Layer");
        let sees = q(i::INSP_RAY_SEES[0], "o 1.º quadro do Sees");
        let filtro = q(i::INSP_RAY_FOLD_FILTER, "o cabeçalho Filter");
        let sinais = q(i::INSP_RAY_FOLD_SIGNALS, "o cabeçalho Signals");
        let mut ordem = vec![
            ("Enabled", (enabled.y, enabled.x)),
            ("Angle", (angle.y, angle.x)),
            ("Reach", (reach.y, reach.x)),
            ("Aim at", (aim.y, aim.x)),
            ("Shape", (shape.y, shape.x)),
            ("Layers", (layers.y, 0.0)),
            ("Layer", (layer.y, 0.0)),
            ("Sees", (sees.y, 0.0)),
        ];
        lado_a_lado(c, ("Reach", reach), ("Aim at", aim));
        // O conta-gotas mora DENTRO da caixa do Aim at, também na meia coluna.
        let gotas = q(i::INSP_RAY_AIM_PICK, "o conta-gotas do Aim at");
        assert!(
            gotas.x >= aim.x - FOLGA
                && gotas.x + gotas.w <= aim.x + aim.w + FOLGA
                && gotas.y >= aim.y - FOLGA
                && gotas.y + gotas.h <= aim.y + aim.h + FOLGA,
            "o conta-gotas dentro do Aim at (cartões {c}): {gotas:?} {aim:?}"
        );
        if c {
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

/// Em cartões, `a` e `b` na mesma linha com `a` à esquerda e sem se sobrepor; fora deles, `a` por cima de `b` e
/// `b` não mais estreito que `a` (a largura toda).
fn lado_a_lado(c: bool, (na, a): (&str, Rect), (nb, b): (&str, Rect)) {
    if c {
        assert!(
            (a.y - b.y).abs() <= FOLGA && a.x + a.w <= b.x + FOLGA,
            "{na} | {nb} na mesma linha, {na} à esquerda: {a:?} {b:?}"
        );
    } else {
        assert!(
            a.y + a.h <= b.y + FOLGA,
            "{na} por cima de {nb} (clássico): {a:?} {b:?}"
        );
    }
}

/// ⭐ Shape | Thickness e Shape | Cone na mesma linha em cartões (a Shape à esquerda), empilhados no clássico; o
/// número da forma que não está activa não se pinta.
#[test]
fn a_shape_e_o_seu_numero_partilham_a_linha() {
    use ph2d_editor_core::ray_edits::RayShapeKind;
    nos_dois_modos(|c| {
        for (forma, id, outro, nome) in [
            (
                RayShapeKind::Thick,
                i::INSP_RAY_THICKNESS,
                i::INSP_RAY_CONE,
                "Thickness",
            ),
            (
                RayShapeKind::Cone,
                i::INSP_RAY_CONE,
                i::INSP_RAY_THICKNESS,
                "Cone",
            ),
        ] {
            let mut info = raio();
            info.shape = forma;
            let (_h, _st, r) = assente_com(&info, c, false);
            let shape = achado(&r, i::INSP_RAY_SHAPE).expect("Shape");
            let numero = achado(&r, id).unwrap_or_else(|| panic!("{nome} (cartões {c})"));
            assert!(
                achado(&r, outro).is_none(),
                "só o número activo ({nome}, cartões {c})"
            );
            lado_a_lado(c, ("Shape", shape), (nome, numero));
            let layers = achado(&r, i::INSP_RAY_FOLD_LAYERS).expect("Layers");
            assert!(numero.y < layers.y, "{nome} antes de Layers (cartões {c})");
            set_current_inspector_ray(None);
        }
    });
}
