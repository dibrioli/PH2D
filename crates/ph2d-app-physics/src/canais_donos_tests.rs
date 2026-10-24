//! ⭐⭐ **Uma propriedade, um canal** (plano F27 F (d)) — cada faixa do catálogo que a FISICA ou a
//! JUNTA já é: escrever pelo funil da família muda o que o catálogo leria do MESMO componente (é a
//! mesma propriedade). Que o catálogo não a publica é gate da `ph2d-ecs`
//! (`uma_faixa_com_dono_nao_e_canal_do_catalogo`).

use super::*;
use crate::canais_junta as cj;
use crate::physics_tests::{apply, registry, sprite_scene};
use ph2d_ecs::canal::Especie;
use ph2d_ecs::canal::familia::JUNTA;
use ph2d_ecs::canal_componente::{self as cc, DONOS, Dono};
use ph2d_ecs::canal_junta as j;
use ph2d_physics_ecs::{JointKind, PhysicsJoint, PulleyWheel};
use ph2d_timeline::PropKind;

fn perto(a: f32, b: f32) -> bool {
    (a - b).abs() <= 1e-4 * a.abs().max(1.0)
}

fn da(familia: u16) -> Vec<(&'static str, u16, u16, FieldKey)> {
    (DONOS.iter())
        .filter_map(|&(c, f, x, d)| match d {
            Dono::Canal(k) if k.familia == familia => Some((c, f, x, k)),
            _ => None,
        })
        .collect()
}

/// Um valor diferente de `antes`, no domínio do campo.
fn outro(k: FieldKey, antes: f32) -> f32 {
    match k.info().expect("info").especie {
        Especie::Opcoes(o) => {
            #[expect(clippy::cast_precision_loss, reason = "uma lista curta")]
            let n = o.len() as f32;
            (antes + 1.0) % n
        }
        Especie::Ligado => 1.0 - antes,
        Especie::Numero => antes + 1.0,
        _ => [0.375, 0.625, 1.5]
            .into_iter()
            .find(|&v| !perto(v, antes))
            .expect("um valor"),
    }
}

#[test]
fn escrever_pela_fisica_muda_o_que_o_catalogo_le() {
    let reg = registry();
    let pares = da(FISICA);
    assert_eq!(pares.len(), 28, "o censo da física");
    for (c, fid, x, k) in pares {
        let (mut sim, e) = sprite_scene();
        apply(&mut sim, e, PhysicsFieldEdit::Add);
        let b = e.to_bits();
        // O que a §11 exige antes: a zona é de um sensor; a massa, da fonte manual.
        if (f::FORCA_X..=f::ZONA_QUEDA).contains(&k.campo) {
            escrever(&mut sim, &reg, b, f::SENSOR, 1.0);
        }
        if k.campo == f::MASSA {
            escrever(&mut sim, &reg, b, f::MASSA_MANUAL, 1.0);
        }
        let antes = ler(&retrato(&sim, b).expect("retrato"), k.campo).expect("campo");
        let (_, ficou) = escrever(&mut sim, &reg, b, k.campo, outro(k, antes)).expect("escreve");
        assert!(!perto(ficou, antes), "{k:?}: a escrita não pegou ({antes})");
        let lido = cc::ler_pelo_catalogo(&reg, sim.world(), e, c, fid, x)
            .unwrap_or_else(|| panic!("{c}.{fid}.{x}: o catálogo não lê"));
        assert!(
            perto(lido, ficou),
            "{c}.{fid}.{x} = {lido} ↔ {k:?} = {ficou}"
        );
    }
}

#[test]
fn escrever_pela_junta_muda_o_que_o_catalogo_le() {
    let reg = registry();
    let pares = da(JUNTA);
    assert_eq!(pares.len(), 22, "o censo da junta (sem os quatro PropKind)");
    let base = PhysicsJoint {
        kind: JointKind::Custom,
        ..PhysicsJoint::default()
    };
    for (c, fid, x, k) in pares {
        let mut sim = SimWorld::new();
        let (e, ficou, antes) = if (j::RODA_RAIO..=j::RODA_QUEBRA_FORCA).contains(&k.campo) {
            let w = PulleyWheel::default();
            let antes = cj::ler_roda(&w, false, k).expect("campo");
            let (w, _) = cj::escrever_roda(w, false, k, outro(k, antes));
            let ficou = cj::ler_roda(&w, false, k).expect("campo");
            (sim.world_mut().spawn(w).id(), ficou, antes)
        } else {
            let antes = cj::ler(&base, k).expect("campo");
            let pj = cj::escrever(base, k, outro(k, antes));
            let ficou = cj::ler(&pj, k).expect("campo");
            (sim.world_mut().spawn(pj).id(), ficou, antes)
        };
        assert!(!perto(ficou, antes), "{k:?}: a escrita não pegou ({antes})");
        let lido = cc::ler_pelo_catalogo(&reg, sim.world(), e, c, fid, x)
            .unwrap_or_else(|| panic!("{c}.{fid}.{x}: o catálogo não lê"));
        assert!(
            perto(lido, ficou),
            "{c}.{fid}.{x} = {lido} ↔ {k:?} = {ficou}"
        );
    }
}

/// Os quatro `PropKind` da junta leem `PhysicsJoint::{motor_target, motor_speed, rest_length,
/// max_length}` (`ph2d_timeline::apply_prop::JointField::of`, `pub(crate)` lá): o catálogo lê os
/// mesmos números.
#[test]
fn os_quatro_numeros_da_junta_sao_os_prop_kind() {
    let reg = registry();
    let pj = PhysicsJoint {
        motor_target: 0.25,
        motor_speed: 1.25,
        rest_length: 2.25,
        max_length: 3.25,
        ..PhysicsJoint::default()
    };
    let mut sim = SimWorld::new();
    let e = sim.world_mut().spawn(pj).id();
    for (p, quer) in [
        (PropKind::JointMotorTarget, pj.motor_target),
        (PropKind::JointMotorSpeed, pj.motor_speed),
        (PropKind::JointRestLength, pj.rest_length),
        (PropKind::JointMaxLength, pj.max_length),
    ] {
        let &(c, fid, x, _) = (DONOS.iter())
            .find(|e| e.3 == Dono::Prop(p.target().get()))
            .unwrap_or_else(|| panic!("{p:?} sem faixa"));
        let lido = cc::ler_pelo_catalogo(&reg, sim.world(), e, c, fid, x);
        assert_eq!(lido, Some(quer), "{c}.{fid}.{x} ↔ {p:?}");
    }
}
