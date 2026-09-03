//! ⭐⭐ **A SELEÇÃO MÚLTIPLA NASCE NO CANVAS?** (W58) — os gates do clique aditivo e do laço.
//!
//! ⚠️ **A `DIRETIVA_IMPLEMENTACAO` §1 chama-lhe a causa nº 1 da semana perdida no Painter:** *"a
//! alça está pintada, o arrasto está correto, e ninguém liga os dois"* passa em todo teste de
//! unidade dos dois lados. Por isso estes gates atravessam a costura inteira — do botão do rato até
//! ao pedido de seleção que o shell consome.

use ph2d_ecs::SimWorld;
use ph2d_field::{Blend, FieldDoc, Node, NodeId, NodeKind, Op, Primitive, Xform};

pub(super) use crate::field3d_input::begin;
use crate::field3d_scene::SelectRequest;
use crate::field3d_smoke::Drag;

pub(crate) const AREA: ph2d_editor::zones::Rect = ph2d_editor::zones::Rect {
    x: 100.0,
    y: 50.0,
    w: 400.0,
    h: 300.0,
};

/// `n` esferas em fila, penduradas numa união.
///
/// ⚠️ **Ela nasceu com `n = 2` fixo, e foi exactamente aí que um defeito se escondeu** (Enio,
/// 2026-08-24: *"o retângulo de seleção não seleciona mais de 2 objetos ao mesmo tempo"*). Um teto
/// em dois passa **verde** numa fixtura de dois. *Uma fixtura que não contém o fenómeno mede outra
/// coisa* — e esta wave já tinha pago essa lição quatro vezes.
fn balls(n: usize) -> FieldDoc {
    let mut nodes: Vec<Node> = (0..n)
        .map(|i| {
            let t = if n == 1 {
                0.0
            } else {
                -0.6 + 1.2 * (i as f32) / ((n - 1) as f32)
            };
            ph2d_field_eval::leaf(
                Primitive::Sphere { radius: 0.14 },
                Xform {
                    translation: [t, 0.0, 0.0],
                    ..Xform::IDENTITY
                },
            )
        })
        .collect();
    nodes.push(Node::new(
        Xform::IDENTITY,
        NodeKind::Combine {
            op: Op::Union(Blend::Sharp),
            children: (0..n as u32).map(NodeId).collect(),
        },
    ));
    FieldDoc::new(nodes, NodeId(n as u32)).expect("a peça")
}

pub(crate) fn two_balls() -> FieldDoc {
    balls(2)
}

/// Arma o módulo com a área publicada e a peça cozida — o estado de quem está a olhar para ela.
fn armed<R>(f: impl FnOnce(&mut SimWorld) -> R) -> R {
    armed_with(&two_balls(), f)
}

pub(crate) fn armed_with<R>(doc: &FieldDoc, f: impl FnOnce(&mut SimWorld) -> R) -> R {
    crate::field3d_smoke::set_armed_by_panel(true);
    let mut sim = SimWorld::new();
    crate::field3d_scene::sync_scene(&mut sim, Some(doc), 0.0);
    crate::field3d_smoke::with_smoke(|s| {
        s.vp_mut().area = Some(AREA);
        // Uma vista de frente enquadrando a peça — as duas bolas caem nos dois lados do centro.
        s.vp_mut().cam = ph2d_field_render::Orbit::from_yaw_pitch(0.0, 0.0);
        s.vp_mut().manual = true;
    });
    // Um quadro para a ponte cozinhar e publicar o documento que o picker consome.
    crate::field3d_scene::ecs_bridge(&mut sim, None, &[], &crate::field3d_scene::no_drawing());
    let out = f(&mut sim);
    crate::field3d_smoke::set_armed_by_panel(false);
    let _ = crate::field3d_smoke::with_smoke(|_| ());
    out
}

/// O centro da área, em coordenadas de JANELA.
pub(super) fn win(local: [f32; 2]) -> (f32, f32) {
    (AREA.x + local[0], AREA.y + local[1])
}

/// O pixel LOCAL onde um ponto de mundo aparece.
///
/// ⚠️ **Derivado, nunca escrito à mão.** A 1.ª versão destes gates tinha `[120.0, 150.0]` colado —
/// e ao mudar a fixtura de duas bolas fixas para `balls(n)` as posições mexeram-se, o pixel caiu no
/// fundo, e o gate reprovou por uma razão que não tinha nada a ver com o que ele afirma.
pub(super) fn pixel_of(p: [f32; 3]) -> [f32; 2] {
    crate::field3d_smoke::with_smoke(|s| {
        let screen = ph2d_field_render::Screen::new(
            AREA.w.round() as u32,
            AREA.h.round() as u32,
            s.vp().cam.half_extent,
        );
        s.vp()
            .cam
            .project(p, screen)
            .expect("o ponto está no quadro")
            .0
    })
    .expect("o módulo está armado")
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
        let Some(SelectRequest::AddMany(bits)) = req else {
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
        let Some(SelectRequest::AddMany(bits)) = req else {
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
        let on_ball = pixel_of([-0.6, 0.0, 0.0]);
        crate::field3d_smoke::with_smoke(|s| {
            s.pending_pick = Some((on_ball, true));
        });
        let req =
            crate::field3d_scene::ecs_bridge(sim, None, &[], &crate::field3d_scene::no_drawing());
        assert!(
            matches!(req, Some(SelectRequest::Toggle(_))),
            "um clique com modificador não pediu para ALTERNAR: {req:?}"
        );
        // ⭐⭐ **E aplicar DUAS vezes tira-a** — a assimetria com o laço, medida sobre estado
        // prévio. ⛔ Sem esta metade o gate lia só o *pedido*, e uma mutação que trocava o verbo do
        // consumidor (`toggle` → `add`) sobrevivia: o pedido continuava a dizer `Toggle`.
        let mut gizmo = ph2d_editor::screens::hero::GizmoStateGroup::default();
        crate::field3d_scene::apply(&mut gizmo, req.expect("o pedido"));
        assert_eq!(
            gizmo.selected_len(),
            1,
            "o 1.º clique aditivo não escolheu nada"
        );
        crate::field3d_smoke::with_smoke(|s| {
            s.pending_pick = Some((on_ball, true));
        });
        let again =
            crate::field3d_scene::ecs_bridge(sim, None, &[], &crate::field3d_scene::no_drawing())
                .expect("o 2.º pedido");
        crate::field3d_scene::apply(&mut gizmo, again);
        assert_eq!(
            gizmo.selected_len(),
            0,
            "clicar com modificador numa peça JÁ escolhida não a tirou — um clique alterna, e é o \
             que o separa do laço, que soma"
        );
        crate::field3d_smoke::with_smoke(|s| {
            s.pending_pick = Some((on_ball, false));
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
        let on_ball = pixel_of([-0.6, 0.0, 0.0]);
        crate::field3d_smoke::with_smoke(|s| {
            let p = win(on_ball);
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

/// ⛔⛔ **UM LAÇO SOBRE `n` OBJETOS PEDE OS `n`** — o gate que faltava, e o defeito que ele apanha.
///
/// Enio, no smoke da W58: *"o retângulo de seleção não seleciona mais de 2 objetos ao mesmo tempo"*.
///
/// ⚠️ **O gate irmão media exactamente DOIS**, e um teto em dois passa verde nele. *A fixtura de um
/// laço tem de conter mais objetos do que o teto que se suspeita* — e como o teto é desconhecido, a
/// forma certa é **varrer**: 2, 3, 4, 5.
#[test]
fn a_lasso_over_many_asks_for_all_of_them() {
    for n in [2usize, 3, 4, 5] {
        let got = armed_with(&balls(n), |sim| {
            let (a, b) = ([4.0f32, 4.0], [396.0f32, 296.0]);
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
            match crate::field3d_scene::ecs_bridge(
                sim,
                None,
                &[],
                &crate::field3d_scene::no_drawing(),
            ) {
                Some(SelectRequest::AddMany(bits)) => bits.len(),
                other => panic!("{n} bolas: o laço não pediu uma seleção múltipla: {other:?}"),
            }
        });
        assert_eq!(
            got, n,
            "um laço sobre {n} bolas pediu {got} — o laço tem um teto que ninguém escreveu"
        );
    }
}

/// ⛔ **E O CONSUMIDOR TEM DE APLICAR OS `n`** — a outra metade da costura.
///
/// ⚠️ O gate acima prova que o **pedido** traz os `n`; este prova que a **seleção** fica com os `n`.
/// *Um gate que lê o pedido não prova a seleção* — é a terceira vez nesta linha que a metade que
/// falta é a de quem executa.
#[test]
fn the_consumer_puts_all_of_them_in_the_selection() {
    for n in [2usize, 3, 4, 5, 6] {
        let mut gizmo = ph2d_editor::screens::hero::GizmoStateGroup::default();
        // A MESMA lei que o `render_loop` corre sobre `SelectRequest::AddMany`.
        for bits in 1..=n as u64 {
            gizmo.toggle_in_selection(bits);
        }
        assert_eq!(
            gizmo.selected_len(),
            n,
            "alternar {n} objetos deixou {} na seleção",
            gizmo.selected_len()
        );
    }
}

/// ⛔⛔ **A SELEÇÃO DO LAÇO TEM DE SOBREVIVER AOS QUADROS SEGUINTES** — a costura no TEMPO.
///
/// ⚠️ **É a pergunta que os dois gates acima não fazem.** Um mede o *pedido*, o outro o *consumidor*
/// — e os dois correm **uma vez**. O app corre a ponte **todo quadro**, e um defeito que só aparece
/// no quadro a seguir lê-se, do lado do artista, como *"não seleciona mais de dois"*.
#[test]
fn the_lasso_selection_survives_the_next_frames() {
    for n in [2usize, 3, 4, 5] {
        armed_with(&balls(n), |sim| {
            let mut gizmo = ph2d_editor::screens::hero::GizmoStateGroup::default();
            let (a, b) = ([4.0f32, 4.0], [396.0f32, 296.0]);
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
            // ⭐ Quatro quadros: o do laço, e três de repouso — a MESMA lei que o `render_loop`
            // corre, incluindo a aplicação do pedido à seleção.
            for frame in 0..4 {
                let req = crate::field3d_scene::ecs_bridge(
                    sim,
                    gizmo.selection,
                    &gizmo.extra_selection,
                    &crate::field3d_scene::no_drawing(),
                );
                // ⭐ A porta do PRODUTO — ver `field3d_scene::apply`.
                if let Some(req) = req {
                    crate::field3d_scene::apply(&mut gizmo, req);
                }
                assert_eq!(
                    gizmo.selected_len(),
                    n,
                    "{n} bolas, quadro {frame}: a seleção ficou com {} — algo a jusante do laço \
                     está a comê-la",
                    gizmo.selected_len()
                );
            }
        });
    }
}

/// ⛔⛔ **A MOLDURA DO LAÇO PINTA-SE SEM NADA SELECIONADO** (W58c) — o gate que faltava.
///
/// Enio, 2026-08-24: *"o desenho do retângulo de seleção deixou de aparecer"*.
///
/// ⚠️ **A W58 gateou o gesto e a captura, e não a PINTURA** — e foi exactamente ali que o defeito
/// se meteu: a moldura estava dentro da guarda `if let Some(anchor) = smoke.gizmo`, isto é, só era
/// desenhada **com algo já selecionado**. O laço mais comum de todos — o primeiro, com a peça
/// acabada de abrir — desenhava nada. *As três perguntas de costura desta casa são pintado /
/// populado / clicado, e esta wave só tinha respondido às duas últimas.*
///
/// # ⚠️ A régua teve de ser corrigida: ela media o RECORTE
///
/// A 1.ª versão comparava o **tamanho** do `path_data` com e sem laço, e uma mutação que apagava a
/// chamada a `paint_lasso` **SOBREVIVEU**: o `push_clip` que a envolve também escreve um caminho na
/// cena, então a cena crescia na mesma. ⇒ a régua passa a comparar **dois rectângulos DIFERENTES**:
/// o recorte é o mesmo nos dois (é a área), então qualquer diferença nos bytes vem da moldura — e
/// uma moldura não pintada dá dois resultados **idênticos**.
#[test]
fn the_lasso_band_paints_with_nothing_selected() {
    use ph2d_vector::VectorScene;
    let doc = FieldDoc::new(
        vec![ph2d_field_eval::leaf(
            Primitive::Sphere { radius: 0.3 },
            Xform::IDENTITY,
        )],
        NodeId(0),
    )
    .expect("a peça");
    armed_with(&doc, |_| {
        let mut text = ph2d_text::TextSystem::without_system_fonts();
        let mut paint = |lasso: Option<([f32; 2], [f32; 2])>| {
            crate::field3d_smoke::with_smoke(|s| {
                s.lasso = lasso;
                // ⚠️ **NADA selecionado** — é a condição exacta do defeito.
                s.gizmo = None;
            });
            let mut scene = VectorScene::new();
            crate::field3d_smoke::draw(AREA, ph2d_tokens::Theme::default(), &mut text, &mut scene);
            scene.inner().encoding().path_data.clone()
        };
        let quiet = paint(None);
        let small = paint(Some(([20.0, 20.0], [80.0, 60.0])));
        let large = paint(Some(([20.0, 20.0], [300.0, 200.0])));
        assert!(
            large.len() > quiet.len(),
            "com um laço em curso e nada selecionado a cena não cresceu ({} -> {}) — a moldura não \
             está a ser pintada, e o artista arrasta às cegas",
            quiet.len(),
            large.len()
        );
        assert_ne!(
            small, large,
            "dois rectângulos de tamanhos diferentes puseram os MESMOS bytes na cena — o que cresceu \
             foi o recorte, não a moldura"
        );
    });
}
