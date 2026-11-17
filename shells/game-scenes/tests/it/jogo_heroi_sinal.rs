//! A cena 28 (`PH2D_GAME_SMOKE=28`, GEF 08 MI onda 3): o «Hero» com os filhos «vida» e «rastro», as pedras que
//! cruzam 0.25, a regra da «Warning Lamp» e o script da «Script Meter» que ouvem `low_health`.

use ph2d_app_game::smoke::heroi as cena;
use ph2d_ecs::{Entity, MotionInput, Name, SignalActions, SignalVerb, SimWorld, Visibility};
use ph2d_physics_ecs::{Damage, Health};

fn um(sim: &mut SimWorld, nome: &str) -> Entity {
    let w = sim.world_mut();
    let mut q = w.query::<(Entity, &Name)>();
    let v: Vec<Entity> = q
        .iter(w)
        .filter(|(_, n)| n.0 == nome)
        .map(|(e, _)| e)
        .collect();
    assert_eq!(v.len(), 1, "{nome}");
    v[0]
}

/// ⭐ Abre no filho «rastro»; o «Hero» tem vida e os dois filhos com as fontes da tabela; quatro pedras de 20
/// levam a vida de 1 a 0.2 (cruza 0.25 uma vez, viva); a lâmpada nasce escondida com a regra; o medidor tem o script.
#[test]
fn a_cena_28_monta_o_heroi_os_filhos_as_pedras_a_regra_e_o_script() {
    let mut sim = SimWorld::new();
    let aberto = crate::comp::montar(&mut sim, cena::CENA);
    let heroi = um(&mut sim, cena::HEROI);
    let w = sim.world_mut();
    assert!(w.get::<Health>(heroi).is_some(), "o Hero tem vida");
    assert!(
        w.get::<MotionInput>(heroi).is_none(),
        "o dono não leva o Motion Input"
    );
    let mut q = w.query::<(Entity, &MotionInput, &ph2d_ecs::ChildOf)>();
    let filhos: Vec<(Entity, String, ph2d_ecs::MotionSource)> = q
        .iter(w)
        .filter(|(_, _, c)| c.parent() == heroi)
        .map(|(e, mi, _)| (e, mi.name.clone(), mi.source))
        .collect();
    let lidos: Vec<_> = filhos.iter().map(|f| (f.1.as_str(), f.2)).collect();
    assert_eq!(lidos, cena::FILHOS, "os filhos e as fontes");
    assert_eq!(aberto, filhos[1].0, "abre no «rastro»");
    assert_eq!(
        w.get::<MotionInput>(aberto).map(|m| m.from),
        Some(0),
        "From = o pai"
    );
    let mut q = w.query::<(&Name, &Damage)>();
    let golpes: Vec<f32> = q
        .iter(w)
        .filter(|(n, _)| n.0 == cena::PEDRA)
        .map(|(_, d)| d.amount)
        .collect();
    assert_eq!(golpes.len(), cena::PEDRAS.len());
    let mut vida = 1.0f32;
    let mut cruzou = 0;
    for g in golpes {
        let antes = vida;
        vida -= g / 100.0;
        cruzou += usize::from(antes >= 0.25 && vida < 0.25);
    }
    assert_eq!(cruzou, 1, "cruza 0.25 uma vez");
    assert!(vida > 0.0, "acaba viva");
    let lampada = um(&mut sim, cena::LAMPADA);
    let w = sim.world();
    assert_eq!(w.get::<Visibility>(lampada).map(|v| v.hidden), Some(true));
    let regra = &w.get::<SignalActions>(lampada).expect("a regra").0[0];
    assert_eq!(
        (regra.on.as_str(), regra.verb),
        (cena::SINAL, SignalVerb::Show)
    );
    let medidor = um(&mut sim, cena::MEDIDOR);
    assert!(
        sim.world()
            .get::<ph2d_script::LuauScript>(medidor)
            .is_some(),
        "o script"
    );
    assert!(
        cena::SCRIPT.contains("function on_signal")
            && cena::SCRIPT.contains("motion.get(\"vida\")")
    );
}

/// ⭐ A regra reage: um `low_health` (sem origem, como o Motion publica) mostra a «Warning Lamp»; outro nome não.
#[test]
fn a_regra_da_lampada_mostra_a_ao_ouvir_low_health() {
    use ph2d_app_components::signal_actions_bridge::apply;
    for (nome, mostra) in [(cena::SINAL, true), ("other", false)] {
        let mut sim = SimWorld::new();
        crate::comp::montar(&mut sim, cena::CENA);
        let lampada = um(&mut sim, cena::LAMPADA);
        let disparos = [ph2d_ecs::Disparo {
            nome,
            quem: None,
            outro: None,
        }];
        let tags = ph2d_tags::TagTree::new();
        let efeitos = ph2d_ecs::resolve_signal_actions(sim.world_mut(), &tags, &disparos);
        let mut drive = ph2d_preview_drive::PreviewDrive::default();
        apply(&mut sim, &efeitos, 1.0, &mut drive, &mut |_, _, _| false);
        assert_eq!(
            sim.world().get::<Visibility>(lampada).map(|v| v.hidden),
            Some(!mostra),
            "{nome}"
        );
    }
}
