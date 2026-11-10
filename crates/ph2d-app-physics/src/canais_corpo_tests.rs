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
use std::collections::BTreeSet;

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
        // Os pontos do «Outline» editam-se no canvas: `ph2d_physics_outline::canais_pontos`.
        if !f::e_do_ponto(campo) {
            assert!(
                porta(campo).is_some() && ler(&info, campo).is_some(),
                "campo {campo}"
            );
        }
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
        let discreto = k(campo).info().is_some_and(|i| i.discreto());
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
    // Opção B: a forma mora na filha principal do corpo.
    let s = ph2d_physics_ecs::main_shape(sim.world(), e).expect("forma");
    let _ = escrever(&mut sim, &reg, b, f::RAIO, 0.3);
    assert!(matches!(
        sim.world().get::<Collider>(s).unwrap().shape,
        ColliderShape::Cuboid { .. }
    ));
    let _ = escrever(&mut sim, &reg, b, f::FORMA, 0.2);
    let _ = escrever(&mut sim, &reg, b, f::RAIO, 0.3);
    assert_eq!(
        sim.world().get::<Collider>(s).unwrap().shape,
        ColliderShape::Ball { radius: 0.3 }
    );
    let _ = escrever(&mut sim, &reg, b, f::CAMADA, 99.0);
    assert_eq!(
        usize::from(sim.world().get::<Collider>(s).unwrap().layer),
        insp::INSP_PHYS_LAYER.len() - 1
    );
}

/// ⭐ **O MESMO funil que o painel**: a rotação inicial escrita em graus pelo canal dá o corpo
/// que o Inspector dá ao emitir a edição dos graus; a trava, o marcador que o clique põe.
#[test]
fn o_canal_e_a_edicao_do_painel_dao_o_mesmo_corpo() {
    let reg = registry();
    let (mut pelo_canal, e) = corpo();
    let _ = escrever(
        &mut pelo_canal,
        &reg,
        e.to_bits(),
        f::VEL_ANGULAR,
        1_f32.to_degrees(),
    );
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
    // Opção B: o colisor nasce na filha principal; sem o corpo, é ela a peça.
    let peca = ph2d_physics_ecs::main_shape(sim.world(), e).expect("forma");
    sim.world_mut().entity_mut(e).remove::<RigidBody>();
    let solta = sim.world_mut().spawn(Transform::default()).id();
    publicar(&mut sim, &[peca.to_bits(), solta.to_bits()]);
    let l = sim.world().resource::<CanaisLidos>();
    let campos: Vec<u16> = l.campos(peca.to_bits()).map(|(kk, _)| kk.campo).collect();
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
    assert!(aplicar(&mut sim, &reg, &mut conducao, &Default::default()));
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
    // Parada a timeline, a curva já não muda nada: o valor dela passa a ser documento.
    conducao.a_tocar(false);
    conducao.settle();
    conducao.settle();
    assert!(conducao.is_empty());
}

/// ⭐⭐ **A tocar, um quadro em que o relógio repete o instante não vira documento** (a irmã do
/// gate de `ph2d_app_vec::canais`): a curva escreve o mesmo valor, continua a conduzir.
#[test]
fn a_tocar_um_quadro_sem_avanco_nao_vira_documento() {
    let reg = registry();
    let (mut sim, e) = corpo();
    let b = e.to_bits();
    let mut conducao = ConducaoFisica::default();
    conducao.a_tocar(true);
    for _ in 0..2 {
        sim.world_mut()
            .get_resource_or_insert_with(CanaisEscritos::default)
            .escrever(b, k(f::TIPO), 2.0);
        aplicar(&mut sim, &reg, &mut conducao, &Default::default());
        conducao.settle();
    }
    let vivo = conducao.substituir(&mut sim, &reg);
    assert_eq!(
        sim.world().get::<RigidBody>(e).unwrap().kind,
        BodyKind::Dynamic,
        "a tocar, a fotografia vê o autorado"
    );
    conducao.repor(&mut sim, &reg, vivo);
    assert_eq!(
        sim.world().get::<RigidBody>(e).unwrap().kind,
        BodyKind::Kinematic
    );
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
        aplicar(&mut sim, &reg, &mut conducao, &Default::default());
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

/// ⭐⭐ **Opção B: o campo de forma de um corpo mora na filha.** A chave de Width com o CORPO como
/// alvo vai para a trilha da filha (o `StableId` dela); a reprodução — a trilha da filha, uma antiga
/// do corpo, e o material pela filha — move o collider da filha, e o corpo continua sem `Collider`.
#[test]
fn a_chave_da_forma_no_corpo_vai_para_a_filha() {
    let reg = registry();
    let (mut sim, e) = sprite_scene();
    ph2d_physics_ecs::torna_corpo(sim.world_mut(), e, BodyKind::Dynamic, [0.5, 0.5], true);
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    let b = e.to_bits();
    let filha = ph2d_physics_ecs::main_shape(sim.world(), e).expect("a filha");
    assert_ne!(filha, e, "a forma é filha");
    publicar(&mut sim, &[b]);
    let lidos = sim.world().resource::<CanaisLidos>();
    assert_eq!(
        lidos.valor(filha.to_bits(), k(f::MEIO_X)),
        Some(1.0),
        "Width = 2 × 0,5"
    );
    assert_eq!(
        lidos.valor(b, k(f::MEIO_X)),
        None,
        "a forma não se publica no corpo"
    );
    assert!(lidos.valor(b, k(f::TIPO)).is_some(), "o tipo fica no corpo");
    let alvo = dono_da_chave(sim.world(), b, ph2d_timeline::PropKind::Field(k(f::MEIO_X)));
    let id = |x: u64| {
        sim.world()
            .get::<ph2d_ecs::StableId>(Entity::from_bits(x))
            .copied()
    };
    assert!(id(alvo).is_some());
    assert_eq!(id(alvo), id(filha.to_bits()), "a trilha é a da filha");
    let tipo = dono_da_chave(sim.world(), b, ph2d_timeline::PropKind::Field(k(f::TIPO)));
    assert_eq!(tipo, b, "a do tipo, a do corpo");
    for (alvo, campo, v) in [
        (filha.to_bits(), f::MEIO_X, 0.8),
        (b, f::MEIO_X, 0.9),
        (filha.to_bits(), f::RESTITUICAO, 0.3),
    ] {
        sim.world_mut()
            .get_resource_or_insert_with(CanaisEscritos::default)
            .escrever(alvo, k(campo), v);
        aplicar(&mut sim, &reg, &mut Default::default(), &Default::default());
        let c = *sim
            .world()
            .get::<Collider>(filha)
            .expect("a filha tem a forma");
        let vale = match (campo, c.shape) {
            (f::MEIO_X, ColliderShape::Cuboid { half_x, .. }) => half_x,
            _ => c.restitution,
        };
        // A curva fala Width; o `Collider`, metades.
        let v = if campo == f::MEIO_X { v * 0.5 } else { v };
        assert!((vale - v).abs() < 1e-5, "{campo}: {vale} ≠ {v}");
        assert!(sim.world().get::<Collider>(e).is_none(), "nada no corpo");
    }
}

/// ⭐ A timeline lê a camada de colisão pelo MESMO rótulo do campo Layer: o nome dado, «Default» na 0,
/// «Layer n» sem nome — publicado com os valores, por quadro.
///
/// Mutação que deve sangrar: tirar a publicação dos nomes do `publicar`.
#[test]
fn a_timeline_le_a_camada_de_colisao_pelo_rotulo_do_cartao() {
    let (mut sim, e) = corpo();
    ph2d_physics_ecs::LayerNames::rename(sim.world_mut(), 3, "Coins");
    publicar(&mut sim, &[e.to_bits()]);
    let nomes = sim
        .world()
        .get_resource::<ph2d_ecs::canal_palavra::NomesDasCamadasDeColisao>()
        .expect("publicada");
    assert_eq!(nomes.0.len(), ph2d_physics_ecs::MAX_LAYERS);
    assert_eq!(nomes.0[0], ph2d_i18n::tr("card.physics.layer_zero"));
    assert_eq!(nomes.0[3], "Coins");
    assert_eq!(
        nomes.0[5],
        ph2d_panel_physics::aba::nome_inteiro(&ph2d_physics_ecs::LayerNames::of(sim.world()), 5)
    );
    assert_ne!(
        nomes.0[5], "5",
        "sem nome, o rótulo do cartão, não o número"
    );
}

/// ⭐ A curva diz o número do CARTÃO (dono 09/11): Width e Height inteiros, a altura inteira da cápsula, o giro em
/// graus por segundo; e as medidas da forma levam a unidade de comprimento do projecto na timeline.
///
/// Mutação que deve sangrar: o `ler` de `MEIO_X` voltar a `info.half_x`.
#[test]
fn a_curva_diz_o_numero_do_cartao() {
    use ph2d_ecs::canal::{Unidade, unidade};
    let reg = registry();
    let (mut sim, e) = corpo();
    let b = e.to_bits();
    let forma = |sim: &SimWorld| {
        let f = ph2d_physics_ecs::forma_para_editar(sim.world(), e).unwrap_or(e);
        sim.world().get::<Collider>(f).expect("forma").shape
    };
    let _ = escrever(&mut sim, &reg, b, f::FORMA, 1.0);
    let _ = escrever(&mut sim, &reg, b, f::MEIO_X, 3.0);
    match forma(&sim) {
        ColliderShape::Cuboid { half_x, .. } => assert!((half_x - 1.5).abs() < 1e-5, "{half_x}"),
        s => panic!("{s:?}"),
    }
    assert!((valor(&sim, e, f::MEIO_X) - 3.0).abs() < 1e-5, "Width 3");
    let _ = escrever(&mut sim, &reg, b, f::FORMA, 2.0);
    let _ = escrever(&mut sim, &reg, b, f::RAIO, 0.25);
    let _ = escrever(&mut sim, &reg, b, f::CAPSULA, 2.0);
    match forma(&sim) {
        ColliderShape::Capsule {
            half_height,
            radius,
        } => assert!(((half_height + radius) * 2.0 - 2.0).abs() < 1e-5),
        s => panic!("{s:?}"),
    }
    assert!((valor(&sim, e, f::CAPSULA) - 2.0).abs() < 1e-5, "Height 2");
    let _ = escrever(&mut sim, &reg, b, f::VEL_ANGULAR, 90.0);
    let giro = sim.world().get::<InitialVelocity>(e).expect("giro").angvel;
    assert!((giro - 90_f32.to_radians()).abs() < 1e-5, "90 °/s");
    assert!((valor(&sim, e, f::VEL_ANGULAR) - 90.0).abs() < 1e-3);
    for c in [
        f::RAIO,
        f::MEIO_X,
        f::MEIO_Y,
        f::CAPSULA,
        f::DESLOC_X,
        f::DESLOC_Y,
    ] {
        assert_eq!(unidade(k(c)), Unidade::Comprimento, "campo {c}");
    }
    assert_eq!(
        unidade(k(f::VEL_ANGULAR)),
        Unidade::Nenhuma,
        "°/s não é um ângulo"
    );
}
