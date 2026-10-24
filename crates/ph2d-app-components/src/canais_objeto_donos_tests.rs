//! ⭐⭐ **Uma propriedade, um canal** (plano F27 F (d)) — cada faixa do catálogo que a família
//! OBJETO já é: escrever pelo canal do objecto muda o que o catálogo leria do MESMO componente (é a
//! mesma propriedade), e o funil do catálogo não a publica.

use super::*;
use crate::component_registry_for_tests::registo;
use ph2d_ecs::canal_componente::{self as cc, DONOS, Dono};
use ph2d_timeline::{AnimTarget, PropKind};

fn cena() -> (SimWorld, Entity) {
    let mut sim = SimWorld::new();
    let e = sim
        .world_mut()
        .spawn((
            Transform::default(),
            Sprite::atlas(0, [1.0, 1.0], [0.5, 0.5, 0.5, 0.5]),
            SpriteCornerTint::IDENTITY,
            SpriteEmissive(1.5),
        ))
        .id();
    (sim, e)
}

fn do_objeto() -> Vec<(&'static str, u16, u16, FieldKey)> {
    (DONOS.iter())
        .filter_map(|&(c, f, x, d)| match d {
            Dono::Canal(k) if k.familia == OBJETO => Some((c, f, x, k)),
            _ => None,
        })
        .collect()
}

#[test]
fn escrever_pelo_objeto_muda_o_que_o_catalogo_le() {
    let reg = registo();
    let pares = do_objeto();
    assert_eq!(pares.len(), 26, "o censo do objecto");
    for (c, fid, x, k) in pares {
        let (mut sim, e) = cena();
        let antes = ler(sim.world(), e, k).expect("o objecto tem o campo");
        let alvo = if k.info().is_some_and(|i| i.discreto()) {
            1.0 - antes
        } else if k.campo == o::EMISSAO {
            3.25
        } else {
            0.125
        };
        assert_ne!(antes, alvo);
        escrever(sim.world_mut(), e, k, alvo);
        let pelo_catalogo = cc::ler_pelo_catalogo(&reg, sim.world(), e, c, fid, x);
        assert_eq!(pelo_catalogo, Some(alvo), "{c}.{fid}.{x} ↔ {k:?}");
    }
}

#[test]
fn o_catalogo_nao_publica_o_que_o_objeto_ja_e() {
    let reg = registo();
    let (sim, e) = cena();
    let publicados = crate::canais_componente::campos(&reg, sim.world(), e);
    for &(c, fid, x, _) in DONOS {
        let k = cc::chave(c, fid, x).expect("cabe");
        assert!(
            !publicados.contains(&k),
            "{c}.{fid}.{x} publicado pelo catálogo"
        );
    }
    // O que sobra da sprite no catálogo: flip x/y, âncora, centrada, deslocamento.
    let da_sprite = (publicados.iter())
        .filter(|k| {
            cc::resolver(**k).is_some_and(|(d, ..)| d.canonical_name == "ph2d::render::Sprite")
        })
        .count();
    assert_eq!(da_sprite, 7, "{publicados:?}");
}

/// Os donos `PropKind` existem e o Auto Key varre-os; o alfa do tint é o `PropKind::Opacity`
/// (`Sprite::tint[3]`, o mesmo número que o catálogo lê). Declarativo para a junta: o leitor dos
/// quatro `PropKind` dela mora na `ph2d-app-physics`.
#[test]
fn os_donos_prop_kind_sao_canais_do_auto_key() {
    let reg = registo();
    for &(c, fid, x, d) in DONOS {
        let Dono::Prop(p) = d else { continue };
        let pk = PropKind::from_target(AnimTarget::new(p)).unwrap_or_else(|| panic!("{p}"));
        assert!(PropKind::AUTOKEYED.contains(&pk), "{c}.{fid}.{x} ↔ {pk:?}");
    }
    let (mut sim, e) = cena();
    sim.world_mut().get_mut::<Sprite>(e).unwrap().tint[3] = 0.375;
    let alfa = cc::ler_pelo_catalogo(&reg, sim.world(), e, "ph2d::render::Sprite", 1, 3);
    assert_eq!(alfa, Some(0.375));
    let dono = cc::dono("ph2d::render::Sprite", 1, 3);
    assert_eq!(dono, Some(Dono::Prop(PropKind::Opacity.target().get())));
}

/// ⭐ O losango pintado pelo rótulo do catálogo aponta o MESMO grupo que a tabela por id.
#[test]
fn o_rotulo_do_catalogo_aponta_o_canal_da_tabela() {
    for (rotulo, quer) in [
        (
            "component.field.sprite.8",
            vec![PropKind::Field(FieldKey::new(OBJETO, o::SILHUETA, 0, 0))],
        ),
        (
            "component.field.sprite.7",
            vec![PropKind::Field(FieldKey::new(OBJETO, o::OPACIDADE, 0, 0))],
        ),
    ] {
        let ks: Vec<PropKind> = (cc::chaves_do_rotulo(rotulo).iter())
            .map(|&k| PropKind::Field(k))
            .collect();
        assert_eq!(ks, quer, "{rotulo}");
    }
    let tinta: Vec<FieldKey> = cc::chaves_do_rotulo("component.field.sprite.1").to_vec();
    assert_eq!(
        tinta,
        (0..3)
            .map(|c| FieldKey::new(OBJETO, o::TINTA + c, 0, 0))
            .collect::<Vec<_>>()
    );
    assert_eq!(cc::props_do_rotulo("component.field.sprite.1"), &[5]);
}
