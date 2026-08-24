//! ⭐⭐ **A SELEÇÃO MÚLTIPLA NASCE NO CANVAS?** (W58) — os gates do clique aditivo e do laço.
//!
//! ⚠️ **A `DIRETIVA_IMPLEMENTACAO` §1 chama-lhe a causa nº 1 da semana perdida no Painter:** *"a
//! alça está pintada, o arrasto está correto, e ninguém liga os dois"* passa em todo teste de
//! unidade dos dois lados. Por isso estes gates atravessam a costura inteira — do botão do rato até
//! ao pedido de seleção que o shell consome.

use ph2d_ecs::SimWorld;
use ph2d_field::{Blend, FieldDoc, Node, NodeId, NodeKind, Op, Primitive, Xform};

use crate::field3d_input::begin;
use crate::field3d_scene::SelectRequest;
use crate::field3d_smoke::Drag;

const AREA: ph2d_editor::zones::Rect = ph2d_editor::zones::Rect {
    x: 100.0,
    y: 50.0,
    w: 400.0,
    h: 300.0,
};

/// Duas esferas afastadas em X, penduradas numa união — o caso em que um laço apanha **duas**.
fn two_balls() -> FieldDoc {
    let ball = |x: f32| {
        ph2d_field_eval::leaf(
            Primitive::Sphere { radius: 0.22 },
            Xform {
                translation: [x, 0.0, 0.0],
                ..Xform::IDENTITY
            },
        )
    };
    FieldDoc::new(
        vec![
            ball(-0.35),
            ball(0.35),
            Node::new(
                Xform::IDENTITY,
                NodeKind::Combine {
                    op: Op::Union(Blend::Sharp),
                    children: vec![NodeId(0), NodeId(1)],
                },
            ),
        ],
        NodeId(2),
    )
    .expect("a peça")
}

/// Arma o módulo com a área publicada e a peça cozida — o estado de quem está a olhar para ela.
fn armed<R>(f: impl FnOnce(&mut SimWorld) -> R) -> R {
    crate::field3d_smoke::set_armed_by_panel(true);
    let mut sim = SimWorld::new();
    crate::field3d_scene::sync_scene(&mut sim, Some(&two_balls()), 0.0);
    crate::field3d_smoke::with_smoke(|s| {
        s.area = Some(AREA);
        // Uma vista de frente enquadrando a peça — as duas bolas caem nos dois lados do centro.
        s.cam = ph2d_field_render::Orbit::from_yaw_pitch(0.0, 0.0);
        s.manual = true;
    });
    // Um quadro para a ponte cozinhar e publicar o documento que o picker consome.
    crate::field3d_scene::ecs_bridge(&mut sim, None, &[], &crate::field3d_scene::no_drawing());
    let out = f(&mut sim);
    crate::field3d_smoke::set_armed_by_panel(false);
    let _ = crate::field3d_smoke::with_smoke(|_| ());
    out
}

/// O centro da área, em coordenadas de JANELA.
fn win(local: [f32; 2]) -> (f32, f32) {
    (AREA.x + local[0], AREA.y + local[1])
}

/// ⭐⭐ **O MODIFICADOR ABRE O LAÇO, E SÓ ELE** — arrastar continua a orbitar.
///
/// ⛔ É a decisão de produto desta wave, e ela protege o gesto principal: a pesquisa do navball mede
/// os utilizadores *quase 2× mais rápidos* a arrastar do que a clicar. Um laço em espaço vazio faria
/// o mesmo botão fazer duas coisas conforme o que estivesse por baixo.
#[test]
fn the_modifier_opens_the_lasso_and_a_plain_drag_still_orbits() {
    armed(|_| {
        crate::field3d_smoke::with_smoke(|s| {
            let p = win([200.0, 150.0]);
            assert!(begin(
                s,
                winit::event::MouseButton::Left,
                Drag::Orbit,
                false,
                p
            ));
            assert_eq!(
                s.drag,
                Some(Drag::Orbit),
                "um arrasto SEM modificador deixou de orbitar — o gesto principal do módulo"
            );
            assert!(
                s.lasso.is_none(),
                "um arrasto sem modificador abriu um laço"
            );
            s.drag = None;
            assert!(begin(
                s,
                winit::event::MouseButton::Left,
                Drag::Orbit,
                true,
                p
            ));
            assert_eq!(
                s.drag,
                Some(Drag::Lasso),
                "o modificador não abriu o laço — a seleção múltipla continua inexprimível no canvas"
            );
            assert!(s.lasso.is_some(), "o laço abriu sem moldura para pintar");
        });
    });
}

/// ⭐⭐⭐ **UM LAÇO SOBRE AS DUAS BOLAS PEDE AS DUAS** — a costura inteira, do botão ao pedido.
#[test]
fn a_lasso_over_both_balls_asks_for_both() {
    armed(|sim| {
        let (a, b) = ([20.0f32, 20.0], [380.0f32, 280.0]);
        crate::field3d_smoke::with_smoke(|s| {
            assert!(begin(
                s,
                winit::event::MouseButton::Left,
                Drag::Orbit,
                true,
                win(a)
            ));
            s.last_pointer = win(b);
            s.lasso = s.lasso.map(|(from, _)| (from, b));
            crate::field3d_input::finish_for_test(s);
            assert!(
                s.pending_lasso.is_some(),
                "soltar o laço não deixou pedido nenhum — a moldura desenhava e não escolhia"
            );
        });
        let req =
            crate::field3d_scene::ecs_bridge(sim, None, &[], &crate::field3d_scene::no_drawing());
        let Some(SelectRequest::ToggleMany(bits)) = req else {
            panic!("o laço não pediu uma seleção múltipla: {req:?}");
        };
        assert_eq!(
            bits.len(),
            2,
            "o laço apanhou {} objetos das duas bolas que ele cobre",
            bits.len()
        );
    });
}

/// ⭐⭐ **UM LAÇO SOBRE UMA SÓ apanha uma só** — o controle que separa «escolhe o que se vê» de
/// «escolhe tudo».
#[test]
fn a_lasso_over_one_ball_asks_for_one() {
    armed(|sim| {
        // A metade ESQUERDA da área: uma bola dentro, a outra fora.
        let (a, b) = ([4.0f32, 4.0], [190.0f32, 296.0]);
        crate::field3d_smoke::with_smoke(|s| {
            begin(
                s,
                winit::event::MouseButton::Left,
                Drag::Orbit,
                true,
                win(a),
            );
            s.last_pointer = win(b);
            s.lasso = s.lasso.map(|(from, _)| (from, b));
            crate::field3d_input::finish_for_test(s);
        });
        let req =
            crate::field3d_scene::ecs_bridge(sim, None, &[], &crate::field3d_scene::no_drawing());
        let Some(SelectRequest::ToggleMany(bits)) = req else {
            panic!("o laço não pediu uma seleção: {req:?}");
        };
        assert_eq!(
            bits.len(),
            1,
            "um laço sobre METADE da peça apanhou {} objetos — ele está a escolher o que não se vê \
             dentro dele",
            bits.len()
        );
    });
}

/// ⭐⭐ **UM LAÇO NO VAZIO NÃO MEXE NA SELEÇÃO** — a tecla disse *acrescenta*, e limpar seria o
/// contrário do que ela pediu.
#[test]
fn a_lasso_over_nothing_leaves_the_selection_alone() {
    armed(|sim| {
        // Uma faixa fina na quina de cima — longe da peça, que está no meio.
        let (a, b) = ([2.0f32, 2.0], [40.0f32, 20.0]);
        crate::field3d_smoke::with_smoke(|s| {
            begin(
                s,
                winit::event::MouseButton::Left,
                Drag::Orbit,
                true,
                win(a),
            );
            s.last_pointer = win(b);
            s.lasso = s.lasso.map(|(from, _)| (from, b));
            crate::field3d_input::finish_for_test(s);
        });
        assert!(
            crate::field3d_scene::ecs_bridge(sim, None, &[], &crate::field3d_scene::no_drawing())
                .is_none(),
            "um laço que não apanhou nada pediu para mexer na seleção"
        );
    });
}

/// ⭐⭐ **UM CLIQUE COM MODIFICADOR ALTERNA, SEM MODIFICADOR SUBSTITUI** — e o fundo com a tecla em
/// baixo **não limpa**.
#[test]
fn the_modifier_turns_a_click_into_a_toggle() {
    armed(|sim| {
        // Sobre a bola da esquerda.
        crate::field3d_smoke::with_smoke(|s| {
            s.pending_pick = Some(([120.0, 150.0], true));
        });
        let req =
            crate::field3d_scene::ecs_bridge(sim, None, &[], &crate::field3d_scene::no_drawing());
        assert!(
            matches!(req, Some(SelectRequest::Toggle(_))),
            "um clique com modificador não pediu para ALTERNAR: {req:?}"
        );
        crate::field3d_smoke::with_smoke(|s| {
            s.pending_pick = Some(([120.0, 150.0], false));
        });
        let req =
            crate::field3d_scene::ecs_bridge(sim, None, &[], &crate::field3d_scene::no_drawing());
        assert!(
            matches!(req, Some(SelectRequest::Entity(_))),
            "um clique SEM modificador deixou de substituir a seleção: {req:?}"
        );
        // …e o fundo: sem tecla limpa, com tecla não mexe.
        crate::field3d_smoke::with_smoke(|s| {
            s.pending_pick = Some(([4.0, 4.0], false));
        });
        assert!(
            matches!(
                crate::field3d_scene::ecs_bridge(
                    sim,
                    None,
                    &[],
                    &crate::field3d_scene::no_drawing()
                ),
                Some(SelectRequest::Clear)
            ),
            "um clique no fundo deixou de limpar"
        );
        crate::field3d_smoke::with_smoke(|s| {
            s.pending_pick = Some(([4.0, 4.0], true));
        });
        assert!(
            crate::field3d_scene::ecs_bridge(sim, None, &[], &crate::field3d_scene::no_drawing())
                .is_none(),
            "um clique ADITIVO no fundo limpou a seleção — a tecla pediu o contrário"
        );
    });
}

/// ⭐ **`Shift`+clique sem arrastar é um clique aditivo, não um rectângulo de área zero.**
#[test]
fn a_lasso_that_never_moved_is_an_additive_click() {
    armed(|_| {
        crate::field3d_smoke::with_smoke(|s| {
            let p = win([120.0, 150.0]);
            begin(s, winit::event::MouseButton::Left, Drag::Orbit, true, p);
            s.last_pointer = p;
            crate::field3d_input::finish_for_test(s);
            assert!(
                s.pending_lasso.is_none(),
                "um laço que não andou virou um rectângulo de área zero"
            );
            assert_eq!(
                s.pending_pick.map(|(_, add)| add),
                Some(true),
                "`Shift`+clique não pediu uma alternância — o gesto morreu entre os dois ramos"
            );
        });
    });
}
