//! A cena 27 (`PH2D_GAME_SMOKE=27`, ficha GEF 08): dois alvos com vida e Motion Input, a «Lamp» sem vida (o aviso) e
//! oito balas com Damage — o Play tira vida sozinho e o grafo `=134` lê-a pelos nomes «vida» e «boom».

use ph2d_app_game::smoke::motion_input as cena;
use ph2d_ecs::{Entity, MotionInput, MotionSource, Name, SimWorld};
use ph2d_physics_ecs::{
    BodyKind, Damage, Health, HealthNow, OnHit, PhysicsBridge, ProjectileMotion, RigidBody,
};

fn por_nome(sim: &mut SimWorld, nome: &str) -> Vec<Entity> {
    let w = sim.world_mut();
    let mut q = w.query::<(Entity, &Name)>();
    q.iter(w)
        .filter(|(_, n)| n.0 == nome)
        .map(|(e, _)| e)
        .collect()
}

fn um(sim: &mut SimWorld, nome: &str) -> Entity {
    let v = por_nome(sim, nome);
    assert_eq!(v.len(), 1, "{nome}");
    v[0]
}

/// O «Health Dummy» abre escolhido; cada objecto tem a fonte e o nome da tabela; só a «Lamp» não tem vida; cada
/// bala tem corpo Kinematic com forma, o Projectile Motion e o Damage que some ao acertar.
#[test]
fn a_cena_27_monta_os_alvos_a_lampada_e_as_balas() {
    let mut sim = SimWorld::new();
    let aberto = crate::comp::montar(&mut sim, cena::CENA);
    assert_eq!(um(&mut sim, cena::ALVO_VIDA), aberto);
    for (nome, _, fonte, entrada) in cena::OBJETOS {
        let e = um(&mut sim, nome);
        let w = sim.world();
        let mi = w.get::<MotionInput>(e).expect(nome);
        assert_eq!((mi.name.as_str(), mi.source), (entrada, fonte), "{nome}");
        let lampada = nome == cena::LAMPADA;
        assert_eq!(w.get::<Health>(e).is_none(), lampada, "{nome}: vida");
        assert_eq!(ph2d_physics_ecs::tem_forma(w, e), !lampada, "{nome}: forma");
    }
    let fontes: Vec<_> = cena::OBJETOS.iter().map(|o| (o.0, o.2)).collect();
    assert!(fontes.contains(&(cena::ALVO_GOLPE, MotionSource::Damage)));
    let balas = por_nome(&mut sim, "Bullet");
    assert_eq!(balas.len(), 8);
    let w = sim.world();
    for b in balas {
        assert_eq!(
            w.get::<ProjectileMotion>(b).map(|p| p.initial_speed),
            Some(cena::RAPIDEZ)
        );
        let kind = w.get::<RigidBody>(b).map(|r| r.kind);
        assert_eq!(kind, Some(BodyKind::Kinematic));
        assert!(ph2d_physics_ecs::tem_forma(w, b));
        let d = w.get::<Damage>(b).expect("Damage");
        assert_eq!((d.amount, d.on_hit), (cena::GOLPE, OnHit::Vanish));
    }
}

/// ⭐ **O Play tira vida sozinho**: aos 5 s as quatro balas de cada lado acertaram — os dois alvos ficam em 20 %,
/// vivos. (Mutação: a bala sem Damage, ou a rapidez a zero ⇒ a vida fica cheia e RED.)
#[test]
fn no_play_as_balas_tiram_a_vida_dos_dois_alvos() {
    let mut sim = SimWorld::new();
    crate::comp::montar(&mut sim, cena::CENA);
    let alvos = [
        um(&mut sim, cena::ALVO_VIDA),
        um(&mut sim, cena::ALVO_GOLPE),
    ];
    let mut bridge = PhysicsBridge::new();
    for tick in 1..=300_u64 {
        bridge.dispatch(&mut sim, true, tick);
        // O dreno do Play (`mortes_anunciadas` sobre a cópia): a bala que acertou sai e abre caminho à seguinte.
        let gastas: Vec<Entity> = bridge.damage_spent().to_vec();
        for b in gastas {
            sim.world_mut().despawn(b);
        }
    }
    for a in alvos {
        let w = sim.world();
        let max = w.get::<Health>(a).expect("vida").max;
        let agora = w.get::<HealthNow>(a).expect("vida viva");
        let r = agora.pontos / f64::from(max);
        println!("[motion-input-27] {:?}: {r:.2}", w.get::<Name>(a));
        assert!(!agora.morta && (r - 0.2).abs() < 1e-3, "acaba em 20 %: {r}");
    }
}

/// ⭐ **Onda 2**: os quatro leitores com a fonte, o nome e o desenho da tabela; o «Health Bar Reader» lê do «Health
/// Dummy» pelo From (sem Health próprio) e o «Max Health Reader» do «Hit Dummy»; a bola tem corpo Dynamic que quica e é a única no `estado`.
#[test]
fn a_cena_27_monta_os_leitores_da_onda_2() {
    let mut sim = SimWorld::new();
    crate::comp::montar(&mut sim, cena::CENA);
    let sid = |sim: &mut SimWorld, n: &str| {
        let e = um(sim, n);
        sim.world().get::<ph2d_ecs::StableId>(e).expect(n).0
    };
    let (dummy, golpe) = (
        sid(&mut sim, cena::ALVO_VIDA),
        sid(&mut sim, cena::ALVO_GOLPE),
    );
    let casos = [
        (
            cena::LEITOR_VIDA,
            MotionSource::Health,
            cena::ENTRADA_VIDA,
            dummy,
        ),
        (
            cena::LEITOR_CAMPO,
            MotionSource::Property,
            cena::ENTRADA_VIDA,
            golpe,
        ),
        (
            cena::LEITOR_ACCAO,
            MotionSource::Action,
            cena::ENTRADA_VIDA,
            0,
        ),
        (
            cena::LEITOR_RAPIDEZ,
            MotionSource::Speed,
            cena::ENTRADA_ESTADO,
            0,
        ),
    ];
    for (nome, fonte, entrada, from) in casos {
        let e = um(&mut sim, nome);
        let mi = sim.world().get::<MotionInput>(e).expect(nome).clone();
        assert_eq!(
            (mi.source, mi.name.as_str(), mi.from),
            (fonte, entrada, from),
            "{nome}"
        );
        assert!(
            sim.world().get::<Health>(e).is_none(),
            "{nome}: lê de fora, sem vida própria"
        );
    }
    let (ec, ea) = (
        um(&mut sim, cena::LEITOR_CAMPO),
        um(&mut sim, cena::LEITOR_ACCAO),
    );
    let campo = sim.world().get::<MotionInput>(ec).cloned().expect("campo");
    assert_eq!(
        (campo.property.as_str(), campo.remap, campo.remap_to),
        (cena::CAMPO, true, 200.0)
    );
    assert_eq!(
        sim.world()
            .get::<MotionInput>(ea)
            .map(|m| m.action.clone())
            .as_deref(),
        Some(cena::ACCAO)
    );
    let bola = um(&mut sim, cena::LEITOR_RAPIDEZ);
    let w = sim.world();
    let mi = w.get::<MotionInput>(bola).expect("bola");
    assert_eq!(
        (mi.remap, mi.remap_to, mi.smooth),
        (true, cena::RAPIDEZ_TIPICA, cena::SUAVE_S)
    );
    assert_eq!(
        w.get::<RigidBody>(bola).map(|r| r.kind),
        Some(BodyKind::Dynamic)
    );
    assert!(ph2d_physics_ecs::tem_forma(w, bola));
    // A ordem que o cartão mostra: o leitor da barra passa à frente do Health Dummy, a bola à frente do Tap Reader.
    assert!(cena::LEITOR_VIDA < cena::ALVO_VIDA && cena::ALVO_VIDA < cena::LEITOR_CAMPO);
    assert!(cena::LEITOR_CAMPO < cena::LEITOR_ACCAO);
    // O `estado` tem UM só escritor: o Toggle lê o campo inteiro, e dois desarrumavam a grelha (foto de 16/11).
    let estado = casos.iter().filter(|c| c.2 == cena::ENTRADA_ESTADO).count();
    assert_eq!(estado, 1);
}

/// ⭐ **Em Play a bola cruza a grelha e volta**: a ponte publica no `estado` (1.ª linha = o «Speed Reader») um valor
/// que passa de 0,5 (o anel) a cair, e volta abaixo de 0,5 quando a bola assenta no «Ledge»; a `vida` tem como 1.ª
/// linha o «Health Bar Reader» com a vida do «Health Dummy». (Mutação: sem Dynamic ⇒ nunca passa de 0,5 e RED.)
#[test]
fn no_play_a_bola_cruza_o_estado_e_o_leitor_da_barra_vem_primeiro() {
    use ph2d_app_components::motion_input_fonte::Fontes;
    use ph2d_ecs::motion_inputs::{Escritor, Valor};
    let mut sim = SimWorld::new();
    crate::comp::montar(&mut sim, cena::CENA);
    let mut bridge = PhysicsBridge::new();
    let out = ph2d_runtime::SignalOutbox::new();
    let accoes = std::collections::BTreeMap::new();
    let (mut anel, mut volta) = (false, false);
    for tick in 1..=600_u64 {
        bridge.dispatch(&mut sim, true, tick);
        let gastas: Vec<Entity> = bridge.damage_spent().to_vec();
        for b in gastas {
            sim.world_mut().despawn(b);
        }
        let f = Fontes {
            dt: 1.0 / 60.0,
            physics: Some(&bridge),
            accoes: &accoes,
            registry: None,
        };
        ph2d_app_components::motion_input_bridge::frame(&mut sim, &out, &f);
        let m = sim.world().resource::<ph2d_ecs::MotionInputs>();
        let (quem, v) = m.primeira_linha(cena::ENTRADA_ESTADO).expect("estado");
        assert!(matches!(quem, Escritor::Objeto { nome, .. } if nome == cena::LEITOR_RAPIDEZ));
        let Valor::Numero(v) = v else {
            panic!("número")
        };
        anel |= v >= 0.5;
        volta |= anel && v < 0.5;
        if tick == 2 {
            let (quem, _) = m.primeira_linha(cena::ENTRADA_VIDA).expect("vida");
            assert!(matches!(quem, Escritor::Objeto { nome, .. } if nome == cena::LEITOR_VIDA));
        }
    }
    let bola = um(&mut sim, cena::LEITOR_RAPIDEZ);
    let y = sim
        .world()
        .get::<ph2d_ecs::Transform>(bola)
        .map(|t| t.translation.y);
    println!("[motion-input-27] anel {anel} volta {volta} y {y:?}");
    assert!(anel && volta, "a grelha cruza para o anel e volta");
    assert!(
        y.is_some_and(|y| y > cena::PARAPEITO_AT.0[1]),
        "assenta no Ledge: {y:?}"
    );
}
