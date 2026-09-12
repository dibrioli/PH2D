//! **A costura da TINTA: pintar uma cópia LIGADA muda as irmãs.**
//!
//! ⛔⛔ **Este gate não pôde ir com a família, e a razão é estrutural.** O actor dele é o
//! `hero_intents::texture_rebind::rebind_to_individual` — o re-alojamento dos pixels de uma sprite,
//! que vive na shell — e **uma crate nunca pode chamar o `bin`** (HOWTO §4). O resto do ficheiro de
//! onde ele saiu (`instance_report_tests`) mudou-se para a [`ph2d_app_components`] em 2026-09-12.
//!
//! ⚠️ **Usar o rebind de VERDADE é o ponto, não um detalhe:** escrever o `Sprite` à mão mediria
//! outro programa — o que este gate afirma é que a edição de pixels **sobe até à receita** pela
//! porta que o artista usa.
//!
//! ⏳ **Quando ele volta para lá:** quando o `texture_rebind` sair da shell (família da Sprite /
//! ferramentas de imagem, 5.ª rodada).
//!
//! ⚠️ O arnês vem do `ph2d_app_components::test_support`, atrás da feature `test-support`
//! (HOWTO §2.5).

use ph2d_app_components::instance_sync::MasterEcho;
use ph2d_app_components::test_support::{instantiate, pass, piece, plain_master, registo as reg};
use ph2d_ecs::SimWorld;
use ph2d_physics_ecs::PhysicsBridge;

/// ⭐⭐⭐ **O OUTRO REPORT: pintar uma cópia LIGADA muda as irmãs** (Enio, 2026-08-26 → 27).
///
/// > *«Pintei uma sprite de uma instância e as outras não mudaram.»*
///
/// A edição de pixels sobe até à receita ([`crate::hero_intents::texture_rebind`]) e o passe
/// leva-a a toda a gente.
///
/// ⚠️ **Em 2026-08-27 isto passou a ser o modo LIGADO** (`Instantiate Linked`, o `Alt+D`) e deixou
/// de valer para toda cópia — porque valer para todas era metade de uma incoerência: a tinta subia
/// e a geometria vetorial da mesma cópia virava excepção. O irmão
/// [`painting_an_unlinked_copy_keeps_it_to_itself`] guarda o outro lado.
///
/// ⚠️ **A metade que mantém o ponto fixo:** ela **não** pode virar excepção. Se virasse, a cópia
/// pintada ficava surda à receita para sempre — e o gate mede isso ao lado do resultado visível.
///
/// (Mutação: fazer `write_through_targets` devolver só a entidade ⇒ RED na irmã.)
#[test]
fn painting_one_copy_reaches_the_others() {
    use crate::hero_intents::texture_rebind::{SamplingWindow, rebind_to_individual};
    let mut sim = SimWorld::new();
    let r = reg();
    let bridge = PhysicsBridge::new();
    let mut echo = MasterEcho::default();
    let master = plain_master(&mut sim);
    let a = instantiate(&mut sim, &r, master, None).expect("instanciou A");
    let b = instantiate(&mut sim, &r, master, None).expect("instanciou B");
    // ⭐ As duas são LIGADAS — é este o modo cuja promessa o gate mede. (O `instantiate` local dá
    // `ArtLink::Own`, que é o outro lado e tem gate próprio no `texture_rebind`.)
    for root in [a, b] {
        for e in [root, piece(&sim, root, "Arm")] {
            sim.world_mut().entity_mut(e).insert(ph2d_ecs::LinkedArt);
        }
    }
    pass(&mut sim, &r, &bridge, &mut echo);

    let pixels = ph2d_asset::AssetId::from_bytes(b"os pixels pintados");
    rebind_to_individual(
        piece(&sim, a, "Arm"),
        &mut sim,
        7,
        pixels,
        [1.0, 1.0],
        false,
        SamplingWindow::Dies,
    );
    pass(&mut sim, &r, &bridge, &mut echo);

    for (name, root) in [("a receita", master), ("a irma", b)] {
        assert_eq!(
            sim.world()
                .get::<ph2d_ecs::SpritePixels>(piece(&sim, root, "Arm"))
                .map(|p| p.0),
            Some(pixels),
            "{name} nao recebeu os pixels pintados"
        );
    }
    assert_eq!(
        sim.world()
            .get::<ph2d_ecs::ObjectInstance>(a)
            .map_or(0, |o| o.overrides.len()),
        0,
        "pintar criou uma EXCEPCAO — a copia pintada fica surda a' receita para sempre"
    );
    assert_eq!(
        pass(&mut sim, &r, &bridge, &mut echo),
        0,
        "o passe deixou de ser ponto fixo depois de uma pintura"
    );
}
