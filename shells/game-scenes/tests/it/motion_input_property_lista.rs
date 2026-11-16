//! ⭐ A lista da fonte Property só oferece o que se LÊ (controlo morto, GEF 08 MI onda 3): cada campo
//! oferecido de cada componente registado lê um valor, e o Transform lê-se pelo endereço do Auto
//! Key/Timeline (`PropKind` + `read_prop_kind`), não pelo funil do catálogo (que o cede).

use ph2d_app_components::motion_input_bridge::frame;
use ph2d_app_components::motion_input_fonte::{Fontes, campos_numericos, propriedade};
use ph2d_ecs::motion_inputs::Valor;
use ph2d_ecs::{ChildOf, MotionInput, MotionInputs, MotionSource, SimWorld, Transform};
use ph2d_runtime::SignalOutbox;
use std::collections::{BTreeMap, BTreeSet};

/// ⭐⭐ Todo campo oferecido lê um valor, varrendo TODOS os componentes registados (o padrão de
/// cada um); endereços sem repetidos. Antes: o Transform oferecia cinco linhas na chave
/// `ph2d::ecs::Transform/` e nenhuma lia; `InitialVelocity/linvel`, `AreaEffector/force` e
/// `Collider/offset` (cedidas à física à mão) também não.
#[test]
fn todo_campo_oferecido_pela_property_le_um_valor() {
    let reg = crate::comp::registo();
    let (mut ofertas, mut mortos) = (BTreeSet::new(), Vec::new());
    for entry in reg.iter() {
        let Some(inserir) = entry.insert_default else {
            continue;
        };
        let mut sim = SimWorld::default();
        let e = sim.world_mut().spawn_empty().id();
        if inserir(sim.world_mut(), e).is_err() {
            continue;
        }
        let lista = campos_numericos(&reg, sim.world(), e);
        let unicos: BTreeSet<_> = lista.iter().map(|(_, k)| k).collect();
        assert_eq!(
            unicos.len(),
            lista.len(),
            "{}: {lista:?}",
            entry.canonical_name
        );
        for (rotulo, k) in lista {
            if propriedade(Some(&reg), sim.world(), e, &k).is_none() {
                mortos.push(format!("{rotulo} ({k})"));
            }
            ofertas.insert(k);
        }
    }
    assert!(mortos.is_empty(), "oferecidos que não leem: {mortos:#?}");
    for k in [
        "Transform/position",
        "Transform/rotation",
        "Transform/scale",
    ] {
        assert!(
            ofertas.contains(&format!("ph2d::ecs::{k}")),
            "controlo: {k}"
        );
    }
    assert!(
        ofertas.len() > 50,
        "controlo: a varredura achou {}",
        ofertas.len()
    );
    for k in [
        "ph2d::ecs::Transform/",
        "ph2d::physics::InitialVelocity/linvel",
        "ph2d::physics::AreaEffector/force",
        "ph2d::physics::Collider/offset",
    ] {
        assert!(
            !ofertas.contains(k),
            "{k} não se lê pelo funil e não se oferece"
        );
    }
}

/// O valor do Motion Input `nome` publicado neste quadro.
fn valor(sim: &SimWorld, nome: &str) -> Option<Vec<Valor>> {
    let m = sim.world().get_resource::<MotionInputs>()?;
    Some(m.valores().find(|(n, _)| *n == nome)?.1.clone())
}

/// ⭐⭐ Property Transform/Position de um objecto que se MOVE manda o par certo a cada quadro;
/// Rotation manda um número (radianos, o que o Auto Key grava); Scale o par.
#[test]
fn a_property_transform_position_segue_o_objeto_e_rotation_e_numero() {
    let reg = crate::comp::registo();
    let mut sim = SimWorld::default();
    let pai = sim.world_mut().spawn(Transform::default()).id();
    for (nome, campo) in [("pos", "position"), ("rot", "rotation"), ("esc", "scale")] {
        let mi = MotionInput {
            name: nome.into(),
            source: MotionSource::Property,
            property: format!("ph2d::ecs::Transform/{campo}"),
            ..MotionInput::default()
        };
        sim.world_mut().spawn((mi, ChildOf(pai)));
    }
    let accoes = BTreeMap::new();
    let f = Fontes {
        dt: 1.0 / 60.0,
        physics: None,
        accoes: &accoes,
        registry: Some(&reg),
    };
    for passo in 1..=3 {
        let x = passo as f32 * 10.0;
        {
            let mut t = sim.world_mut().get_mut::<Transform>(pai).unwrap();
            t.translation.x = x;
            t.translation.y = -x / 2.0;
            t.rotation = 0.25 * passo as f32;
            t.scale.x = 2.0;
        }
        frame(&mut sim, &SignalOutbox::new(), &f);
        assert_eq!(valor(&sim, "pos"), Some(vec![Valor::Vetor([x, -x / 2.0])]));
        assert_eq!(
            valor(&sim, "rot"),
            Some(vec![Valor::Numero(0.25 * passo as f32)])
        );
        assert_eq!(valor(&sim, "esc"), Some(vec![Valor::Vetor([2.0, 1.0])]));
    }
}
