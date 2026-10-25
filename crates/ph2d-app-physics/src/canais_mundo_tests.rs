//! Os gates da [`super`] — a família MUNDO: a curva chega ao mundo vivo sem limpar o scrub e sem
//! tocar no autorado, e a mão ganha-lhe no quadro em que muda.

use std::collections::BTreeSet;

use super::*;
use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Transform};
use ph2d_physics_ecs::{BodyKind, Collider, RigidBody};

/// Um corpo a cair livre, a Cena, e a ponte.
fn queda() -> (SimWorld, Entity, PhysicsBridge) {
    let mut sim = SimWorld::new();
    let e = sim
        .world_mut()
        .spawn((
            Transform::from_translation(Vec2::new(0.0, 50.0)),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider::default(),
        ))
        .id();
    ph2d_ecs::garante_cena(sim.world_mut());
    (sim, e, PhysicsBridge::default())
}

fn curva(sim: &mut SimWorld, campo: u16, v: f32) {
    let c = cena(sim).expect("cena");
    sim.world_mut()
        .get_resource_or_insert_with(CanaisEscritos::default)
        .escrever(c, k(campo), v);
}

fn lido(sim: &mut SimWorld, ponte: &PhysicsBridge, campo: u16) -> f32 {
    publicar(sim, ponte);
    let c = cena(sim).expect("cena");
    sim.world()
        .resource::<CanaisLidos>()
        .valor(c, k(campo))
        .expect("publicado")
}

fn y(sim: &SimWorld, e: Entity) -> f32 {
    sim.world()
        .get::<Transform>(e)
        .expect("corpo")
        .translation
        .y
}

/// ⭐⭐⭐ A curva inverte a gravidade a MEIO da queda: o corpo trava (controlo: sem ela continua a
/// cair), o anel do scrub fica (o `set_settings` limpá-lo-ia — controlo da régua), e o autorado não
/// muda.
#[test]
fn a_curva_muda_a_gravidade_do_mundo_sem_limpar_o_scrub() {
    let correr = |com_curva: bool| {
        let (mut sim, e, mut ponte) = queda();
        let mut cond = ConducaoMundo::default();
        for t in 1..=60 {
            aplicar(&mut sim, &mut ponte, &mut cond);
            ponte.dispatch(&mut sim, true, t);
        }
        let anel = ponte.ring_stats().0;
        assert!(anel > 0, "o scrub guardou pontos");
        for t in 61..=150 {
            if com_curva {
                curva(&mut sim, m::GRAVIDADE_Y, 9.81);
            }
            aplicar(&mut sim, &mut ponte, &mut cond);
            ponte.dispatch(&mut sim, true, t);
        }
        assert!(ponte.ring_stats().0 >= anel, "o anel não foi limpo");
        assert_eq!(ponte.settings(), PhysicsSettings::default(), "o autorado");
        (y(&sim, e), ponte)
    };
    let (sem, _) = correr(false);
    let (com, ponte) = correr(true);
    assert!(com > sem + 5.0, "a curva travou a queda: {com} × {sem}");
    assert!((ponte.settings_vivos().gravity_y - 9.81).abs() < 1e-6);
    // Controlo da régua: a porta do painel limpa o anel.
    let mut p = ponte;
    p.set_settings(PhysicsSettings {
        gravity_y: 9.81,
        ..PhysicsSettings::default()
    });
    assert_eq!(p.ring_stats().0, 0, "o set_settings limpa — a régua mede");
}

/// ⭐ O que o save grava (`project_save.rs` lê `PhysicsBridge::settings`) é o AUTORADO, com uma
/// curva activa a mudar o mundo.
#[test]
fn o_save_grava_o_autorado_com_uma_curva_activa() {
    let (mut sim, _, mut ponte) = queda();
    let mut cond = ConducaoMundo::default();
    curva(&mut sim, m::GRAVIDADE_Y, 4.0);
    curva(&mut sim, m::SUBPASSOS, 8.0);
    assert!(aplicar(&mut sim, &mut ponte, &mut cond), "o mundo mudou");
    assert_eq!(ponte.settings_vivos().substeps, 8);
    assert_eq!(ponte.settings(), PhysicsSettings::default());
    // O eco: o Auto Key lê a curva, não uma diferença.
    assert!((lido(&mut sim, &ponte, m::GRAVIDADE_Y) - 4.0).abs() < 1e-6);
}

/// ⭐ A MÃO ganha à curva no quadro em que muda o autorado — no mundo e no que o Auto Key lê —, e
/// sem curva o mundo corre o autorado.
#[test]
fn a_mao_ganha_a_curva_no_quadro_em_que_muda() {
    let (mut sim, _, mut ponte) = queda();
    let mut cond = ConducaoMundo::default();
    curva(&mut sim, m::GRAVIDADE_Y, 5.0);
    aplicar(&mut sim, &mut ponte, &mut cond);
    assert!((lido(&mut sim, &ponte, m::GRAVIDADE_Y) - 5.0).abs() < 1e-6);
    // O painel muda o autorado (fim do quadro); a curva ainda diz 5 no seguinte.
    ponte.set_settings(PhysicsSettings {
        gravity_y: 2.0,
        ..ponte.settings()
    });
    curva(&mut sim, m::GRAVIDADE_Y, 5.0);
    aplicar(&mut sim, &mut ponte, &mut cond);
    assert!(
        (ponte.settings_vivos().gravity_y - 2.0).abs() < 1e-6,
        "o mundo é da mão"
    );
    assert!(
        (lido(&mut sim, &ponte, m::GRAVIDADE_Y) - 2.0).abs() < 1e-6,
        "o Auto Key vê a mão"
    );
    // A chave nasceu: a curva diz 2 — e volta a conduzir.
    curva(&mut sim, m::GRAVIDADE_Y, 3.0);
    aplicar(&mut sim, &mut ponte, &mut cond);
    assert!((ponte.settings_vivos().gravity_y - 3.0).abs() < 1e-6);
    // Sem curva: o autorado.
    aplicar(&mut sim, &mut ponte, &mut cond);
    assert!((ponte.settings_vivos().gravity_y - 2.0).abs() < 1e-6);
    assert!((lido(&mut sim, &ponte, m::GRAVIDADE_Y) - 2.0).abs() < 1e-6);
}

/// Todo campo lê o que se escreveu; cada grupo tem nome traduzido e único.
#[test]
fn cada_campo_do_mundo_le_o_que_escreveu() {
    let mut nomes = BTreeSet::new();
    for campo in m::CAMPOS {
        let mut s = PhysicsSettings::default();
        let antes = ler(&s, campo).expect("leitura");
        let alvo = if m::ADORMECE == campo {
            1.0 - antes
        } else {
            antes + 2.0
        };
        escrever(&mut s, campo, alvo);
        assert_eq!(ler(&s.clamped(), campo), Some(alvo), "campo {campo}");
        let rotulo = k(campo).info().expect("info").rotulo;
        let nome = ph2d_i18n::tr(rotulo);
        assert_ne!(nome, rotulo, "{campo} sem tradução");
        if campo != m::GRAVIDADE_Y {
            assert!(nomes.insert(nome), "dois grupos com o nome {nome}");
        }
    }
}
