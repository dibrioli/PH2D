//! ⭐⭐ **A COSTURA DOS CHIPS DE SOMBREAMENTO** — todo chip do pulldown muda o que se vê, e o aceso
//! diz o que está em vigor (`docs/Render3d/05`).
//!
//! # ⚠️ Por que uma lei própria, e não uma linha na tabela da W34
//!
//! A lei-mãe ([`super`]) mede *«a intenção muda o DOCUMENTO»* e só varre fileiras que **dependem da
//! selecção**. Estas três não são nem uma coisa nem outra: o sombreamento, a vista e a exposição são
//! **estado de VISTA** (o doc do [`crate::shading`] escreve-o), não entram no undo e não viajam no
//! ficheiro — medi-las por *«o documento mudou»* daria as três **mudas**, e a conclusão errada seria
//! apagar os botões.
//!
//! ⇒ é a mesma partição que o irmão [`super::camera`] já pagou: *uma lei de alcançabilidade tem uma
//! régua por espécie de gesto*. A régua aqui é o **estado do viewport e da cena**.
//!
//! # ⛔⛔ E a metade que quase ninguém escreve: o PEDIDO GUARDADO
//!
//! Trocar o olhar sem largar o traçado guardado é o **congelador** — o quadro seguinte reaproveita o
//! que já estava desenhado e o artista clica num chip que não faz nada visível. Ele é a espécie de
//! morto do `CLAUDE.md` §5.0 (*«o consumidor que projecta o valor fora»*): o fio está inteiro, o
//! valor chega, e quem o recebe descarta-o. ⚠️ **Nenhuma sonda de registo o vê**, e um gate que só
//! afirmasse o estado passaria por cima dele.

use super::*;

/// O estado de vista que estas três fileiras governam, lido do módulo.
fn view_now() -> (crate::shading::Shading, ph2d_view_transform::Look) {
    crate::smoke::with_smoke(|s| (s.vp().shading, s.look)).expect("o módulo está armado")
}

/// ⭐⭐⭐ **TODO CHIP DE SOMBREAMENTO PÕE O VIEWPORT NO MODO QUE ELE NOMEIA** — e o aceso segue.
///
/// ⚠️ **Os dois modos são varridos, e o de omissão por último**: um despacho que ignorasse o slot
/// passaria se o gate só pedisse o modo que não é o de partida.
///
/// **Mutação que deve sangrar:** `Shading::ALL.get(0)` no braço do `SetShading`.
#[test]
fn every_shading_chip_paints_the_piece_the_way_it_says() {
    armed(|| {
        let (mut sim, _root) = scene(&flat());
        let _ = ph2d_panel_model3d::drain_intents();
        crate::scene::sync_scene_and_birth(&mut sim, None, &[], 0.0, &crate::scene::no_drawing());
        let snap = ph2d_panel_model3d::state::current();
        assert_eq!(
            snap.shadings.len(),
            crate::shading::Shading::ALL.len(),
            "a fileira é DERIVADA de `Shading::ALL` — se divergir, um botão fica sem lei ou uma lei \
             fica sem botão"
        );

        // ⚠️ **A ordem visita o `Render` primeiro e volta ao `Matcap`**: o de omissão no fim é o que
        // prova que o chip do estado em vigor também é honrado (um `if != actual { … }` a mais
        // deixaria o segundo mudo).
        for slot in [1usize, 0] {
            ph2d_panel_model3d::state::push_intent_for_test(
                ph2d_panel_model3d::ModelIntent::SetShading { slot },
            );
            crate::scene::sync_scene_and_birth(
                &mut sim,
                None,
                &[],
                0.0,
                &crate::scene::no_drawing(),
            );
            assert_eq!(
                view_now().0,
                crate::shading::Shading::ALL[slot],
                "o chip {slot} não pôs o viewport no modo que ele nomeia"
            );
            // ⭐ **E o chip ACESO é o que está em vigor** — derivado do estado, nunca de um botão
            // guardado. Sem esta metade o artista não consegue ler em que modo está.
            let chips = ph2d_panel_model3d::state::current().shadings;
            assert_eq!(
                chips.iter().position(|c| c.active),
                Some(slot),
                "o aceso não seguiu o modo: {chips:?}"
            );
        }
    });
}

/// ⭐⭐⭐ **TODO CHIP DE OLHAR MUDA O OLHAR DA CENA** — a vista e a exposição, as duas fileiras.
///
/// ⚠️ **A vista e a exposição são campos do MESMO [`Look`]**, e cada fileira escreve um só: um chip
/// de exposição que reescrevesse a vista (ou o contrário) apagaria a escolha do vizinho em silêncio.
/// É isso que o gate afirma ao **cruzar** as duas — escolhe uma vista fora da omissão, e só depois
/// varre as exposições.
///
/// **Mutação que deve sangrar:** trocar `with_exposure` por `with_view` no braço do `SetExposure`.
#[test]
fn every_look_chip_changes_the_scene_look_and_leaves_its_neighbour_alone() {
    armed(|| {
        let (mut sim, _root) = scene(&flat());
        let _ = ph2d_panel_model3d::drain_intents();
        let aplica = |sim: &mut SimWorld, intent| {
            ph2d_panel_model3d::state::push_intent_for_test(intent);
            crate::scene::sync_scene_and_birth(sim, None, &[], 0.0, &crate::scene::no_drawing());
        };

        // A VISTA, as duas — e a de omissão no fim, pela razão do gate irmão.
        for slot in [1usize, 0] {
            aplica(&mut sim, ph2d_panel_model3d::ModelIntent::SetLook { slot });
            assert_eq!(
                view_now().1.view,
                ph2d_view_transform::ViewTransform::ALL[slot],
                "o chip de vista {slot} não mudou o olhar da cena"
            );
            assert_eq!(
                ph2d_panel_model3d::state::current()
                    .looks
                    .iter()
                    .position(|c| c.active),
                Some(slot),
                "o aceso da vista não seguiu"
            );
        }
        // ⭐⭐ **A CRUZ**: com uma vista fora da omissão em vigor, varrer as exposições não lhe pode
        // tocar. Sem isto, os dois braços podiam escrever o mesmo campo e o gate não via.
        aplica(
            &mut sim,
            ph2d_panel_model3d::ModelIntent::SetLook { slot: 1 },
        );
        for (slot, (stops, _)) in crate::shading::EXPOSURES.iter().enumerate() {
            aplica(
                &mut sim,
                ph2d_panel_model3d::ModelIntent::SetExposure { slot },
            );
            let look = view_now().1;
            assert!(
                (look.exposure_stops - stops).abs() < 1.0e-6,
                "o chip de exposição {slot} prometia {stops} stops e a cena ficou em {}",
                look.exposure_stops
            );
            assert_eq!(
                look.view,
                ph2d_view_transform::ViewTransform::ALL[1],
                "mexer na exposição apagou a VISTA escolhida — os dois braços escrevem o mesmo campo"
            );
            assert_eq!(
                ph2d_panel_model3d::state::current()
                    .exposures
                    .iter()
                    .position(|c| c.active),
                Some(slot),
                "o aceso da exposição não seguiu"
            );
        }
    });
}

/// ⭐⭐⭐ **E OS TRÊS LARGAM O TRAÇADO GUARDADO** — a metade que o §5.0 chama de «o consumidor que
/// projecta o valor fora».
///
/// # ⛔ O congelador
///
/// O traçado é caro, então o módulo guarda o pedido servido e reaproveita-o enquanto nada mudar.
/// **Um olhar novo não muda a geometria**, logo sem um `forget_requests` explícito o quadro seguinte
/// devolve o desenho antigo: o chip acende, o estado muda, e a **peça não muda de aparência**. O
/// artista lê isso como *«o botão não faz nada»* — e as três leis acima ficam todas verdes por cima.
///
/// **Mutação que deve sangrar:** apagar o `forget_requests` do `Smoke::set_look` ou do `set_shading`.
#[test]
fn changing_how_it_is_painted_drops_the_frame_that_was_already_traced() {
    armed(|| {
        let (mut sim, _root) = scene(&flat());
        let _ = ph2d_panel_model3d::drain_intents();
        crate::scene::sync_scene_and_birth(&mut sim, None, &[], 0.0, &crate::scene::no_drawing());

        // ⛔⛔ **OS SLOTS SÃO DERIVADOS DO ESTADO, nunca literais** — e esta linha é uma cura, não
        // uma elegância. A 1.ª redacção escrevia `SetLook { slot: 1 }`, que era *«a vista que não é
        // a de omissão»* no dia em que foi escrita; quando o dono mandou o módulo abrir em
        // `Neutral` (14/09), o `slot 1` passou a ser **o estado em vigor**, o `set_look` devolveu
        // cedo (ele não faz nada quando o olhar não muda) e o gate reprovou sobre produto correcto.
        //
        // ⚠️ *Uma fixtura que escolhe um slot LITERAL está a afirmar que ele difere do estado — e
        // deixa de o afirmar no dia em que a omissão se mexe, sem uma linha do gate mudar.*
        let (shading_agora, look_agora) = view_now();
        let outro_modo = crate::shading::Shading::ALL
            .iter()
            .position(|s| *s != shading_agora)
            .expect("há mais de um modo");
        let outra_vista = ph2d_view_transform::ViewTransform::ALL
            .iter()
            .position(|v| *v != look_agora.view)
            .expect("há mais de uma vista");
        let outra_exposicao = crate::shading::EXPOSURES
            .iter()
            .position(|(s, _)| (*s - look_agora.exposure_stops).abs() > 1.0e-6)
            .expect("há mais de uma exposição");
        for intent in [
            ph2d_panel_model3d::ModelIntent::SetShading { slot: outro_modo },
            ph2d_panel_model3d::ModelIntent::SetLook { slot: outra_vista },
            ph2d_panel_model3d::ModelIntent::SetExposure {
                slot: outra_exposicao,
            },
        ] {
            // Encena um quadro JÁ SERVIDO em TODAS as vistas: o olhar é da cena, então o gate
            // tem de poder ver as outras a serem largadas também.
            crate::smoke::with_smoke(|s| {
                for vp in &mut s.vps {
                    vp.probe_remember_a_served_request(flat());
                }
            })
            .expect("armado");
            assert!(
                crate::smoke::with_smoke(|s| s.vps.iter().all(|vp| vp.probe_has_request()))
                    == Some(true),
                "o arnês não encenou o quadro servido — o gate mediria um campo que já era `None`"
            );
            ph2d_panel_model3d::state::push_intent_for_test(intent);
            crate::scene::sync_scene_and_birth(
                &mut sim,
                None,
                &[],
                0.0,
                &crate::scene::no_drawing(),
            );
            assert_eq!(
                crate::smoke::with_smoke(|s| s.vps.iter().any(|vp| vp.probe_has_request())),
                Some(false),
                "⛔ {intent:?} deixou um traçado guardado de pé — o chip acende e a peça não muda"
            );
        }
    });
}
