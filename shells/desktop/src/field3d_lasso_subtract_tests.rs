//! ⭐⭐⭐ **O LAÇO QUE SUBTRAI** (W112) — os gates do modo, do pedido e da fileira que o mostra.
//!
//! # Por que um arquivo irmão
//!
//! O [`super::lasso_tests`] passou dos `600` do teto de LOC do shell (HR-18) com esta wave.
//! ⛔ *Split, nunca allowlist* — e o corte é por assunto: aqui está tudo o que responde *«o que o
//! laço FAZ ao que apanhou»*, e nada do que responde *«o que ele apanha»*, que fica no irmão.
//!
//! ⚠️ **O arnês é o do irmão** (`armed`, `begin`, `win`), e de propósito: um segundo `armed_with`
//! seria uma segunda definição de *«o módulo está a correr»*.

use super::lasso_tests::{armed, begin, win};
use crate::field3d_scene::SelectRequest;
use crate::field3d_smoke::Drag;
use ph2d_ecs::SimWorld;

/// ⭐⭐⭐ **O MESMO LAÇO, COM O MODO EM «SUBTRACT», PEDE PARA TIRAR** (W112) — a metade que faltava
/// desde a W58.
///
/// ⚠️ **A colheita é a MESMA e só o pedido muda**, e é isso que o gate afirma: os dois modos apanham
/// os mesmos dois objetos. *Duas colheitas seriam duas respostas a «o que está dentro do
/// rectângulo».*
#[test]
fn the_same_lasso_asks_to_remove_when_the_mode_says_subtract() {
    let (a, b) = ([20.0f32, 20.0], [380.0f32, 280.0]);
    let arrasta = |sim: &mut SimWorld| {
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
        crate::field3d_scene::ecs_bridge(sim, None, &[], &crate::field3d_scene::no_drawing())
    };
    let mut somados = Vec::new();
    armed(|sim| {
        crate::field3d_smoke::with_smoke(|s| s.lasso_subtracts = false);
        let Some(SelectRequest::AddMany(bits)) = arrasta(sim) else {
            panic!("no modo «Add» o laço tem de pedir para SOMAR");
        };
        somados = bits;
    });
    armed(|sim| {
        crate::field3d_smoke::with_smoke(|s| s.lasso_subtracts = true);
        let req = arrasta(sim);
        let Some(SelectRequest::RemoveMany(bits)) = req else {
            panic!("no modo «Subtract» o laço tinha de pedir para TIRAR, e pediu {req:?}");
        };
        assert_eq!(
            bits, somados,
            "os dois modos têm de apanhar exactamente os mesmos objetos — o modo escolhe o VERBO, \
             não a colheita"
        );
        assert!(bits.len() >= 2, "o laço cobre as duas bolas");
    });
}

/// ⭐⭐ **E o pedido de TIRAR tira mesmo** — a lei aplicada, na porta única
/// ([`crate::field3d_scene::apply`]).
///
/// ⚠️ **As três metades importam:** o primário sai e um extra é promovido · um extra sai e o
/// primário fica · quem não estava seleccionado é ignorado em silêncio. ⛔ Sem a terceira, um laço
/// que apanhasse um vizinho não seleccionado podia limpar a selecção inteira.
#[test]
fn removing_from_the_selection_promotes_an_extra_and_ignores_a_stranger() {
    let mut g = ph2d_editor::screens::hero::GizmoStateGroup::default();
    g.replace_selection(Some(1));
    g.add_to_selection(2);
    g.add_to_selection(3);
    crate::field3d_scene::apply(&mut g, SelectRequest::RemoveMany(vec![1, 9]));
    assert_eq!(
        g.selection,
        Some(2),
        "o primário saiu e o extra mais velho subiu"
    );
    assert_eq!(
        g.extra_selection,
        vec![3],
        "o outro extra ficou onde estava"
    );
    crate::field3d_scene::apply(&mut g, SelectRequest::RemoveMany(vec![3]));
    assert_eq!(g.selection, Some(2), "tirar um extra não mexe no primário");
    assert!(g.extra_selection.is_empty());
    crate::field3d_scene::apply(&mut g, SelectRequest::RemoveMany(vec![7]));
    assert_eq!(
        g.selection,
        Some(2),
        "tirar quem não estava seleccionado tem de ser um no-op — senão um laço que apanha um \
         vizinho apaga a selecção"
    );
}

/// ⭐⭐⭐ **O CHIP DO PAINEL CHEGA AO LAÇO — e a fileira só existe quando serve** (W112).
///
/// ⚠️ **Empurrar a intenção prova o TRATADOR, nunca a ALCANÇABILIDADE** — é a lei que o cabeçalho
/// do `field3d_reach_tests` escreve, e a outra metade (o clique de verdade a virar a intenção
/// certa) é varrida pelo `every_chip_family_dispatches_its_own_intent` do painel. *As duas metades
/// vivem onde cada uma pode ser medida.*
///
/// ⭐⭐ **E a terceira lei é a que o preço comprou:** a fileira custa `+66 px` (`+11,9 %` do painel
/// cheio), logo ela só é publicada com **duas ou mais** peças escolhidas — e quando desaparece o
/// modo **volta a somar**. ⛔ Sem essa reposição, o artista ficaria com um laço que subtrai e nada
/// na tela a dizê-lo.
#[test]
fn the_panel_chip_reaches_the_lasso_and_the_row_only_exists_when_it_serves() {
    armed(|sim| {
        let alvos: Vec<bevy_ecs::entity::Entity> = {
            let world = sim.world_mut();
            let mut q = world.query::<(bevy_ecs::entity::Entity, &ph2d_field_ecs::FieldObject)>();
            let raiz = q.iter(world).next().map(|(e, _)| e).expect("a peça");
            ph2d_field_ecs::walk(world, raiz)
                .into_iter()
                .map(|(e, _)| e)
                .filter(|e| {
                    matches!(
                        world.get::<ph2d_field_ecs::FieldNode>(*e).map(|n| &n.shape),
                        Some(ph2d_field::NodeShape::Leaf(_))
                    )
                })
                .collect()
        };
        assert!(alvos.len() >= 2, "a fixtura das duas bolas tem duas folhas");
        for (slot, esperado) in [(1_usize, true), (0, false)] {
            let _ = ph2d_panel_model3d::drain_intents();
            ph2d_panel_model3d::state::push_intent_for_test(
                ph2d_panel_model3d::ModelIntent::SetLassoMode { slot },
            );
            crate::field3d_scene::sync_scene_and_birth(
                sim,
                None,
                &alvos,
                0.0,
                &crate::field3d_scene::no_drawing(),
            );
            assert_eq!(
                crate::field3d_smoke::with_smoke(|s| s.lasso_subtracts),
                Some(esperado),
                "o chip {slot} tinha de pôr o modo do laço em {esperado}"
            );
            // ⭐ E o retrato publicado tem de dizer o mesmo: um chip que não acende deixa o artista
            // sem saber em que modo está — que é precisamente o que torna um modo aceitável.
            let snap = ph2d_panel_model3d::state::current();
            assert_eq!(
                snap.selects.len(),
                2,
                "com duas peças escolhidas a fileira do laço tem de ser publicada"
            );
            assert!(
                snap.selects[slot].active,
                "o chip escolhido tinha de ficar aceso"
            );
        }
        // ⭐⭐⭐ A metade do PREÇO: arma o modo com duas escolhidas, depois desce a UMA.
        let _ = ph2d_panel_model3d::drain_intents();
        ph2d_panel_model3d::state::push_intent_for_test(
            ph2d_panel_model3d::ModelIntent::SetLassoMode { slot: 1 },
        );
        crate::field3d_scene::sync_scene_and_birth(
            sim,
            None,
            &alvos,
            0.0,
            &crate::field3d_scene::no_drawing(),
        );
        assert_eq!(
            crate::field3d_smoke::with_smoke(|s| s.lasso_subtracts),
            Some(true),
            "o controle: com duas escolhidas o modo fica armado"
        );
        crate::field3d_scene::sync_scene_and_birth(
            sim,
            None,
            &alvos[..1],
            0.0,
            &crate::field3d_scene::no_drawing(),
        );
        assert!(
            ph2d_panel_model3d::state::current().selects.is_empty(),
            "com UMA peça escolhida a fileira não tem o que subtrair — não pode ser publicada"
        );
        assert_eq!(
            crate::field3d_smoke::with_smoke(|s| s.lasso_subtracts),
            Some(false),
            "a fileira desapareceu e o modo TEM de voltar a somar — senão fica armado e invisível"
        );
    });
}
