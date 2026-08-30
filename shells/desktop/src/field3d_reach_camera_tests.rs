//! ⭐⭐ **A COSTURA DOS CHIPS DE CÂMERA** — todo chip mexe a câmera, e o aceso apaga-se quando ela
//! sai da vista dele.
//!
//! # Por que um arquivo irmão
//!
//! O [`super::field3d_reach_tests`] responde *«o painel oferece exactamente o que o gesto faz»* — a
//! lei-mãe da W34, sobre **fileiras**. Estes dois respondem outra coisa: *«e o chip de CÂMERA, faz o
//! que promete?»*. O arquivo estava em `606` linhas contra o tecto de `600` do shell — ⚠️ e **já
//! estava, antes desta wave**: o gate vive em `shells/desktop/tests/` e o `cargo test --bins` não
//! lhe toca. ⛔ *Split, nunca allowlist.*

use super::*;

/// ⭐⭐ **TODO CHIP DE CÂMERA MEXE NA CÂMERA** — a lei da W34, com a régua que estas fileiras pedem.
///
/// # ⚠️ Por que a régua NÃO pode ser a mesma
///
/// A lei da W34 mede *"a intenção muda o DOCUMENTO"*, e as fileiras de câmera **não podem** mudar o
/// documento: olhar a peça de frente não é uma edição, não entra no undo e não viaja no arquivo.
/// Medi-las por aquela régua daria **todas** mudas — e a conclusão errada seria remover os botões.
///
/// ⇒ *Uma lei de alcançabilidade tem uma régua por espécie de gesto.* Aqui a régua é a **câmera**.
///
/// ⚠️ E as fileiras também não entram na tabela `ROWS` por uma segunda razão: elas **não dependem da
/// seleção** — olhar de frente não precisa de nada escolhido —, e o `offered == acts` daquela tabela
/// é uma afirmação sobre a seleção.
#[test]
fn every_camera_chip_moves_the_camera() {
    armed(|| {
        let (mut sim, _root) = scene(&flat());
        let _ = ph2d_panel_model3d::drain_intents();
        crate::field3d_scene::sync_scene_and_birth(
            &mut sim,
            None,
            &[],
            0.0,
            &crate::field3d_scene::no_drawing(),
        );

        let snap = ph2d_panel_model3d::state::current();
        assert_eq!(
            snap.views.len(),
            crate::field3d_views::Standard::ALL.len(),
            "a fileira das vistas é DERIVADA de `Standard::ALL` — se divergir, um botão fica sem lei \
             ou uma lei fica sem botão"
        );
        assert_eq!(snap.camera.len(), 3, "a lente, o enquadrar e a divisão");

        // ⭐ Cada VISTA põe a câmera exatamente na orientação que o nome dela promete — **e
        // enquadra**.
        //
        // ⚠️ **A segunda metade veio de uma mutação sobrevivente:** tirar o `frame_the_part` do
        // `SetView` passava, porque o gate só olhava a **orientação**. Uma vista de frente que
        // deixasse a peça fora do quadro é a mesma tela vazia que a W45 e a W46 fecharam — e a
        // fixtura, centrada na origem, nunca o denunciaria sozinha. *Por isso o alvo é levado para
        // longe antes de cada chip.*
        for (slot, v) in crate::field3d_views::Standard::ALL.into_iter().enumerate() {
            // Longe dela, de propósito: sem isto o gate passaria com um `SetView` que não faz nada.
            crate::field3d_smoke::with_smoke(|s| {
                s.vp_mut().cam.rotation = ph2d_field_render::Orbit::default().rotation;
                s.vp_mut().cam.target = [9.0, 9.0, 9.0];
            });
            ph2d_panel_model3d::state::push_intent_for_test(ModelIntent::SetView { slot });
            crate::field3d_scene::sync_scene_and_birth(
                &mut sim,
                None,
                &[],
                0.0,
                &crate::field3d_scene::no_drawing(),
            );
            // ⭐ A câmera **viaja** (W51) — o chip pede a viagem; quem a serve é a mola da casa.
            assert!(
                crate::field3d_smoke::with_smoke(|s| s.flight.is_some()).unwrap_or(false),
                "o chip {slot} ({v:?}) não pediu viagem nenhuma"
            );
            crate::field3d_smoke::note_flight_progress(1.0);
            assert_eq!(
                crate::field3d_smoke::with_smoke(|s| crate::field3d_views::named_view(&s.vp().cam))
                    .flatten(),
                Some(v),
                "o chip {slot} ({v:?}) não pôs a câmera na vista dele"
            );
            let t = crate::field3d_smoke::with_smoke(|s| s.vp().cam.target).expect("armado");
            assert!(
                t.iter().all(|c| c.abs() < 1.0),
                "o chip {slot} ({v:?}) virou a câmera e deixou o alvo em {t:?} — a vista está certa \
                 e a peça está fora do quadro"
            );
        }

        // ⭐ A LENTE alterna, e volta.
        let lens_of = || {
            crate::field3d_smoke::with_smoke(|s| {
                matches!(s.vp().cam.lens, ph2d_field_render::Lens::Ortho)
            })
            .unwrap_or(false)
        };
        let before = lens_of();
        ph2d_panel_model3d::state::push_intent_for_test(ModelIntent::Camera {
            slot: crate::field3d_scene::panel::ORTHO_SLOT,
        });
        crate::field3d_scene::sync_scene_and_birth(
            &mut sim,
            None,
            &[],
            0.0,
            &crate::field3d_scene::no_drawing(),
        );
        assert_ne!(lens_of(), before, "o chip da lente não trocou a lente");
        // …e o retrato DIZ o estado novo: um interruptor que não acende mente sobre o que fez.
        assert_eq!(
            ph2d_panel_model3d::state::current().camera[crate::field3d_scene::panel::ORTHO_SLOT]
                .active,
            lens_of(),
            "o chip da lente não reflete a lente que está posta"
        );

        // ⭐ O ENQUADRAR mexe a câmera para a peça.
        crate::field3d_smoke::with_smoke(|s| {
            s.vp_mut().cam.target = [9.0, 9.0, 9.0];
        });
        ph2d_panel_model3d::state::push_intent_for_test(ModelIntent::Camera {
            slot: crate::field3d_scene::panel::FRAME_SLOT,
        });
        crate::field3d_scene::sync_scene_and_birth(
            &mut sim,
            None,
            &[],
            0.0,
            &crate::field3d_scene::no_drawing(),
        );
        crate::field3d_smoke::note_flight_progress(1.0);
        let t = crate::field3d_smoke::with_smoke(|s| s.vp().cam.target).expect("armado");
        assert!(
            t.iter().all(|c| c.abs() < 1.0),
            "o chip de enquadrar deixou o alvo em {t:?} — ele não foi buscar a peça"
        );

        // ⭐⭐⭐ **A DIVISÃO** (W90), e a régua dela é OUTRA — de propósito.
        //
        // ⚠️ **Este chip não mexe na câmera: ele muda QUANTAS câmeras há.** A lei desta fileira é
        // *«o chip faz o que a fileira promete»*, e a fileira promete *«como estou a olhar?»* — a
        // régua tem de ser a espécie do gesto, não uma cópia da do vizinho. *Foi este gate que
        // recusou o chip novo enquanto ele não tinha lei própria, que é exactamente o trabalho
        // dele.*
        let quantos = || crate::field3d_smoke::with_smoke(|s| s.vps.len()).unwrap_or(0);
        let antes = quantos();
        ph2d_panel_model3d::state::push_intent_for_test(ModelIntent::Camera {
            slot: crate::field3d_scene::panel::QUAD_SLOT,
        });
        crate::field3d_scene::sync_scene_and_birth(
            &mut sim,
            None,
            &[],
            0.0,
            &crate::field3d_scene::no_drawing(),
        );
        assert_eq!(
            quantos(),
            crate::field3d_layout::Split::quad().count(),
            "o chip da divisão não abriu as quatro vistas (estava em {antes})"
        );
        // …e o retrato DIZ o estado novo, como o da lente.
        assert!(
            ph2d_panel_model3d::state::current().camera[crate::field3d_scene::panel::QUAD_SLOT]
                .active,
            "o chip da divisão não acende com a divisão aberta"
        );
        // ⭐ **E fecha**, deixando UMA vista — um interruptor que só liga é meio interruptor.
        ph2d_panel_model3d::state::push_intent_for_test(ModelIntent::Camera {
            slot: crate::field3d_scene::panel::QUAD_SLOT,
        });
        crate::field3d_scene::sync_scene_and_birth(
            &mut sim,
            None,
            &[],
            0.0,
            &crate::field3d_scene::no_drawing(),
        );
        assert_eq!(quantos(), 1, "o chip da divisão não voltou à vista única");
    });
}

/// ⚠️ **O chip aceso diz a VERDADE depois de um arrasto** — o realce é derivado da orientação, e não
/// de um modo guardado. Sem isto, o painel afirmaria *"estás em Frente"* sobre uma vista que o
/// artista já torceu.
#[test]
fn the_lit_view_chip_goes_out_when_the_camera_leaves_it() {
    armed(|| {
        let (mut sim, _root) = scene(&flat());
        let _ = ph2d_panel_model3d::drain_intents();
        ph2d_panel_model3d::state::push_intent_for_test(ModelIntent::SetView { slot: 0 });
        crate::field3d_scene::sync_scene_and_birth(
            &mut sim,
            None,
            &[],
            0.0,
            &crate::field3d_scene::no_drawing(),
        );
        crate::field3d_smoke::note_flight_progress(1.0);
        crate::field3d_scene::sync_scene_and_birth(
            &mut sim,
            None,
            &[],
            0.0,
            &crate::field3d_scene::no_drawing(),
        );
        assert!(
            ph2d_panel_model3d::state::current().views[0].active,
            "o controle: a vista escolhida tem de acender"
        );

        crate::field3d_smoke::with_smoke(|s| {
            crate::field3d_input::law::orbit(&mut s.vp_mut().cam, 4.0, 0.0);
        });
        crate::field3d_scene::sync_scene_and_birth(
            &mut sim,
            None,
            &[],
            0.0,
            &crate::field3d_scene::no_drawing(),
        );
        assert!(
            ph2d_panel_model3d::state::current()
                .views
                .iter()
                .all(|c| !c.active),
            "depois de arrastar, nenhum chip de vista pode continuar aceso"
        );
    });
}
