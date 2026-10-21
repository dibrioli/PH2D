//! Os gates da [`super`] — a família FISICA: os dois funis do painel, a peça, e a curva que chega
//! ao solver sem chegar à fotografia.

use super::*;
use crate::physics_tests::{apply, registry, sprite_scene};
use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, GravityScale, InitialVelocity, LockRotation, PhysicsBridge,
    RigidBody,
};

/// Um corpo dinâmico com a caixa da sprite (a face inteira da §11).
fn corpo() -> (SimWorld, Entity) {
    let (mut sim, e) = sprite_scene();
    apply(&mut sim, e, PhysicsFieldEdit::Add);
    (sim, e)
}

fn valor(sim: &SimWorld, e: Entity, campo: u16) -> f32 {
    ler(&retrato(sim, e.to_bits()).expect("retrato"), campo).expect("campo")
}

/// Todo campo do vocabulário tem um controlo da §11 e uma leitura; e cada grupo tem nome no
/// dope-sheet, traduzido e único.
#[test]
fn todo_campo_tem_controlo_leitura_e_nome() {
    let (sim, e) = corpo();
    let info = retrato(&sim, e.to_bits()).expect("retrato");
    let mut nomes = BTreeSet::new();
    let mut grupos = BTreeSet::new();
    for campo in f::CAMPOS {
        assert!(porta(campo).is_some(), "campo {campo} sem controlo");
        assert!(ler(&info, campo).is_some(), "campo {campo} sem leitura");
        let kk = k(campo);
        if grupos.insert(kk.grupo()) {
            let rotulo = kk.info().expect("info").rotulo;
            let nome = ph2d_i18n::tr(rotulo);
            assert_ne!(nome, rotulo, "{campo} sem tradução");
            assert!(nomes.insert(nome), "dois grupos com o nome {nome}");
        }
    }
}

/// ⭐ Todo campo que a §11 mostra lê de volta o que se escreveu, pelos dois funis do painel (cada
/// um numa cena nova: um campo muda o que os outros significam).
#[test]
fn cada_campo_mostrado_le_o_que_escreveu() {
    let reg = registry();
    let info = {
        let (sim, e) = corpo();
        retrato(&sim, e.to_bits()).expect("retrato")
    };
    let mostrados: Vec<u16> = f::CAMPOS
        .iter()
        .copied()
        .filter(|&c| mostrado(&info, c))
        .collect();
    assert!(
        mostrados.len() >= 25,
        "a face do corpo: {}",
        mostrados.len()
    );
    assert!(!mostrados.contains(&f::RAIO), "uma caixa não mostra o raio");
    assert!(
        !mostrados.contains(&f::MASSA),
        "a massa auto não mostra a massa"
    );
    for campo in mostrados {
        let (mut sim, e) = corpo();
        let antes = valor(&sim, e, campo);
        let discreto = k(campo).info().is_some_and(|i| i.discreto);
        let alvo = match (discreto, antes == 1.0) {
            (true, true) => 0.0,
            (true, false) => 1.0,
            (false, _) => 0.75,
        };
        let (a, ficou) = escrever(&mut sim, &reg, e.to_bits(), campo, alvo).expect("escreve");
        assert_eq!(a, antes);
        assert!(
            (ficou - alvo).abs() < 1e-5,
            "campo {campo}: escreveu {alvo}, leu {ficou}"
        );
    }
}

/// A forma decide as medidas: o raio escrito numa CAIXA não a troca por uma bola (o funil
/// trocaria); depois da forma, o raio pega. As escolhas prendem-se à tabela.
#[test]
fn a_forma_decide_as_medidas() {
    let reg = registry();
    let (mut sim, e) = corpo();
    let b = e.to_bits();
    let _ = escrever(&mut sim, &reg, b, f::RAIO, 0.3);
    assert!(matches!(
        sim.world().get::<Collider>(e).unwrap().shape,
        ColliderShape::Cuboid { .. }
    ));
    let _ = escrever(&mut sim, &reg, b, f::FORMA, 0.2);
    let _ = escrever(&mut sim, &reg, b, f::RAIO, 0.3);
    assert_eq!(
        sim.world().get::<Collider>(e).unwrap().shape,
        ColliderShape::Ball { radius: 0.3 }
    );
    let _ = escrever(&mut sim, &reg, b, f::CAMADA, 99.0);
    assert_eq!(
        usize::from(sim.world().get::<Collider>(e).unwrap().layer),
        insp::INSP_PHYS_LAYER.len() - 1
    );
}

/// ⭐ **O MESMO funil que o painel**: a rotação inicial escrita em radianos pelo canal dá o corpo
/// que o Inspector dá ao emitir a edição dos graus; a trava, o marcador que o clique põe.
#[test]
fn o_canal_e_a_edicao_do_painel_dao_o_mesmo_corpo() {
    let reg = registry();
    let (mut pelo_canal, e) = corpo();
    let _ = escrever(&mut pelo_canal, &reg, e.to_bits(), f::VEL_ANGULAR, 1.0);
    let _ = escrever(&mut pelo_canal, &reg, e.to_bits(), f::TRAVA_ROTACAO, 1.0);
    let (mut pelo_painel, e2) = corpo();
    apply(&mut pelo_painel, e2, PhysicsFieldEdit::Angvel(1.0));
    apply(&mut pelo_painel, e2, PhysicsFieldEdit::LockRotation(true));
    let iv = |s: &SimWorld, e| s.world().get::<InitialVelocity>(e).copied();
    let a = iv(&pelo_canal, e).expect("canal");
    let b = iv(&pelo_painel, e2).expect("painel");
    assert!((a.angvel - b.angvel).abs() < 1e-6);
    assert!(pelo_canal.world().get::<LockRotation>(e).is_some());
}

/// Uma PEÇA (colisor sem corpo) publica só a face dela; uma entidade sem física, nada.
#[test]
fn a_peca_publica_so_a_face_dela() {
    let (mut sim, e) = sprite_scene();
    apply(&mut sim, e, PhysicsFieldEdit::Add);
    sim.world_mut().entity_mut(e).remove::<RigidBody>();
    let solta = sim.world_mut().spawn(Transform::default()).id();
    publicar(&mut sim, &[e.to_bits(), solta.to_bits()]);
    let l = sim.world().resource::<CanaisLidos>();
    let campos: Vec<u16> = l.campos(e.to_bits()).map(|(kk, _)| kk.campo).collect();
    assert_eq!(
        campos,
        [
            f::FORMA,
            f::MEIO_X,
            f::MEIO_Y,
            f::DESLOC_X,
            f::DESLOC_Y,
            f::DENSIDADE
        ]
    );
    assert_eq!(l.campos(solta.to_bits()).count(), 0);
}

/// ⭐⭐ **Ponta-a-ponta**: a curva escreve em `CanaisEscritos` → `aplicar` muda o componente pelo
/// funil, o eco fica, `publicar` lê o novo valor — e a fotografia (`substituir`) vê o AUTORADO,
/// devolvendo o vivo depois (`repor`).
#[test]
fn a_curva_chega_ao_corpo_e_nao_a_fotografia() {
    let reg = registry();
    let (mut sim, e) = corpo();
    let b = e.to_bits();
    {
        let mut w = sim
            .world_mut()
            .get_resource_or_insert_with(CanaisEscritos::default);
        w.escrever(b, k(f::ESCALA_GRAVIDADE), 0.5);
        w.escrever(b, k(f::TIPO), 2.0);
    }
    let mut conducao = ConducaoFisica::default();
    assert!(aplicar(&mut sim, &reg, &mut conducao));
    assert_eq!(
        sim.world().get::<RigidBody>(e).unwrap().kind,
        BodyKind::Kinematic
    );
    publicar(&mut sim, &[b]);
    assert_eq!(
        sim.world().resource::<CanaisLidos>().valor(b, k(f::TIPO)),
        Some(2.0)
    );
    // Kinematic não aceita a gravidade (o painel só a mostra num Dynamic): ela fica.
    assert!(sim.world().get::<GravityScale>(e).is_none());

    let vivo = conducao.substituir(&mut sim, &reg);
    assert_eq!(
        sim.world().get::<RigidBody>(e).unwrap().kind,
        BodyKind::Dynamic,
        "a fotografia vê o autorado"
    );
    conducao.repor(&mut sim, &reg, vivo);
    assert_eq!(
        sim.world().get::<RigidBody>(e).unwrap().kind,
        BodyKind::Kinematic,
        "e o vivo volta"
    );
    conducao.settle();
    conducao.settle();
    assert!(conducao.is_empty());
}

/// ⭐⭐⭐ **E chega ao SOLVER** pelo caminho da edição: em repouso, a curva põe a escala de
/// gravidade a zero e o corpo não cai (controlo: sem a curva ele cai).
#[test]
fn a_curva_da_gravidade_chega_ao_solver_em_repouso() {
    let altura = |com_curva: bool| {
        let reg = registry();
        let mut sim = SimWorld::new();
        let e = sim
            .world_mut()
            .spawn((
                Transform::from_translation(Vec2::new(0.0, 3.0)),
                RigidBody {
                    kind: BodyKind::Dynamic,
                },
                Collider::default(),
            ))
            .id();
        let mut conducao = ConducaoFisica::default();
        if com_curva {
            sim.world_mut()
                .get_resource_or_insert_with(CanaisEscritos::default)
                .escrever(e.to_bits(), k(f::ESCALA_GRAVIDADE), 0.0);
        }
        aplicar(&mut sim, &reg, &mut conducao);
        let mut bridge = PhysicsBridge::default();
        for t in 1..=30 {
            bridge.dispatch(&mut sim, true, t);
        }
        sim.world().get::<Transform>(e).unwrap().translation.y
    };
    assert!(altura(false) < 2.9, "controlo: sem a curva o corpo cai");
    assert!(
        (altura(true) - 3.0).abs() < 1e-3,
        "a curva tirou-lhe a gravidade"
    );
}
