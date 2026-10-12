//! ⭐⭐ **Os cartões do CORPO na luz, lado do mundo** — «Light Occluder» (`BloqueiaLuz`) e «Unlit»
//! (`NaoRecebeLuz`). O instantâneo reflete os componentes e cada edição do cartão age sobre o SEU
//! componente; as da lâmpada continuam a ir à `Luz`. A costura do painel está no `seam_luz_corpo`
//! do `ph2d-panel-inspector`.

use ph2d_app_light2d::inspector::{apply_all, build_corpo_info};
use ph2d_ecs::{Entity, SimWorld, Transform};
use ph2d_editor_core::luz_edits::{InspectorCorpoDaLuzInfo, LuzFieldEdit as E};
use ph2d_light2d_ecs::{BloqueiaLuz, Luz, NaoRecebeLuz};

fn corpo(sim: &mut SimWorld, oclusor: Option<f32>, sem_luz: bool) -> u64 {
    let w = sim.world_mut();
    let mut e = w.spawn(Transform::IDENTITY);
    if let Some(o) = oclusor {
        e.insert(BloqueiaLuz { opacidade: o });
    }
    if sem_luz {
        e.insert(NaoRecebeLuz);
    }
    e.id().to_bits()
}

fn opacidade(sim: &SimWorld, bits: u64) -> Option<f32> {
    sim.world()
        .get::<BloqueiaLuz>(Entity::from_bits(bits))
        .map(|b| b.opacidade)
}

/// ⭐ O instantâneo existe só onde há algum dos dois, e diz quais.
///
/// **Mutações que devem sangrar:** `Some` sem componente · trocar `oclusor`/`sem_luz`.
#[test]
fn o_instantaneo_reflete_os_componentes() {
    let mut sim = SimWorld::new();
    let nada = corpo(&mut sim, None, false);
    let os_dois = corpo(&mut sim, Some(0.5), true);
    let so_bloqueia = corpo(&mut sim, Some(0.25), false);
    let so_sem_luz = corpo(&mut sim, None, true);
    let w = sim.world();
    assert_eq!(build_corpo_info(w, Some(nada)), None);
    assert_eq!(build_corpo_info(w, None), None);
    assert_eq!(
        build_corpo_info(w, Some(os_dois)),
        Some(InspectorCorpoDaLuzInfo {
            entity_bits: os_dois,
            oclusor: Some(0.5),
            sem_luz: true,
        })
    );
    assert_eq!(
        build_corpo_info(w, Some(so_bloqueia)).map(|i| (i.oclusor, i.sem_luz)),
        Some((Some(0.25), false))
    );
    assert_eq!(
        build_corpo_info(w, Some(so_sem_luz)).map(|i| (i.oclusor, i.sem_luz)),
        Some((None, true))
    );
}

/// ⭐⭐ A OPACIDADE escreve-se, fora da faixa prende-se a `0..=1`, `NaN` é recusado, e sem
/// `BloqueiaLuz` nada se escreve (nem se cria o componente).
#[test]
fn a_opacidade_escreve_e_prende_a_faixa() {
    let mut sim = SimWorld::new();
    let b = corpo(&mut sim, Some(1.0), false);
    assert!(apply_all(&mut sim, &[(b, E::Opacidade(0.3))]));
    assert_eq!(opacidade(&sim, b), Some(0.3));
    assert!(
        !apply_all(&mut sim, &[(b, E::Opacidade(0.3))]),
        "o mesmo valor não é mudança"
    );
    assert!(apply_all(&mut sim, &[(b, E::Opacidade(7.0))]));
    assert_eq!(opacidade(&sim, b), Some(1.0));
    assert!(apply_all(&mut sim, &[(b, E::Opacidade(-2.0))]));
    assert_eq!(opacidade(&sim, b), Some(0.0));
    assert!(!apply_all(&mut sim, &[(b, E::Opacidade(f32::NAN))]));
    assert_eq!(opacidade(&sim, b), Some(0.0));
    let sem = corpo(&mut sim, None, true);
    assert!(!apply_all(&mut sim, &[(sem, E::Opacidade(0.5))]));
    assert_eq!(opacidade(&sim, sem), None, "a edição não cria o componente");
}

/// ⭐⭐⭐ Cada botão de RETIRAR tira o SEU componente e deixa o outro.
#[test]
fn retirar_tira_so_o_seu_componente() {
    let mut sim = SimWorld::new();
    let a = corpo(&mut sim, Some(0.5), true);
    assert!(apply_all(&mut sim, &[(a, E::RetirarSemLuz)]));
    let e = Entity::from_bits(a);
    assert!(sim.world().get::<NaoRecebeLuz>(e).is_none());
    assert_eq!(opacidade(&sim, a), Some(0.5), "o oclusor ficou");
    assert!(!apply_all(&mut sim, &[(a, E::RetirarSemLuz)]), "já não há");
    assert!(apply_all(&mut sim, &[(a, E::RetirarOclusor)]));
    assert_eq!(opacidade(&sim, a), None);
    assert!(!apply_all(&mut sim, &[(a, E::RetirarOclusor)]), "já não há");
    assert_eq!(
        build_corpo_info(sim.world(), Some(a)),
        None,
        "o cartão some"
    );
}

/// ⭐ As edições da LÂMPADA continuam a ir à `Luz`, e as do corpo não a tocam.
#[test]
fn as_edicoes_da_lampada_continuam_na_luz() {
    let mut sim = SimWorld::new();
    let bits = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Luz::default(), BloqueiaLuz::default()))
        .id()
        .to_bits();
    assert!(apply_all(
        &mut sim,
        &[(bits, E::Intensidade(3.0)), (bits, E::Opacidade(0.4))]
    ));
    let e = Entity::from_bits(bits);
    assert_eq!(sim.world().get::<Luz>(e).map(|l| l.intensidade), Some(3.0));
    assert_eq!(opacidade(&sim, bits), Some(0.4));
    assert!(apply_all(&mut sim, &[(bits, E::RetirarOclusor)]));
    assert!(sim.world().get::<Luz>(e).is_some(), "a lâmpada ficou");
}
