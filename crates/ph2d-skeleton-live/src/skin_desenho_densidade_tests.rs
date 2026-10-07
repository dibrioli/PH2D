//! ⭐⭐⭐ **A DENSIDADE do bake não muda o que a silhueta faz** (A13). O refino pelos pesos acrescenta
//! amostras (`passo` em peso); a silhueta do contacto (união, ganchos, bola) tem de dar a MESMA
//! resposta a toda densidade — medido antes das curas: dentes de `160°`–`175°` a `0,025`, `0,06`,
//! `0,075`, `0,1` e nenhum a `0,04`/`0,045`; a identidade fora do contacto a falhar a `0,03`/`0,035`.

use super::super::maior_zona_apertada;
use super::dentes::{assa_na, entrada};
use ph2d_vec_boolean::overlap::{PAREDE_MINIMA, RAIO_DO_VINCO};
use ph2d_vec_skin::curva::Refino;

/// As densidades da varredura.
const PASSOS: [f64; 9] = [0.025, 0.03, 0.035, 0.04, 0.045, 0.05, 0.06, 0.075, 0.1];

fn viragem_maxima(v: &[ph2d_vec_scene::VecVertex]) -> f64 {
    (0..v.len())
        .filter_map(|i| ph2d_vec_boolean::overlap::viragem_do_vertice(v, i))
        .fold(0.0, f64::max)
}

/// ⭐⭐⭐ **GATE — o braço dobrado de volta não tem dentes a densidade nenhuma**: a grelha de `387`
/// poses (`primeira ∈ {160, 165, 170, 172, 174, 176, 178}`, `segunda` de `−178` a `178` de `6` em
/// `6`) a cada `passo`, e nenhum nó da silhueta passa da `PAREDE_MINIMA`.
#[test]
fn o_braco_nao_tem_dentes_a_densidade_nenhuma() {
    let mut poses: Vec<(f32, f32)> = Vec::new();
    for a in [160_u16, 165, 170, 172, 174, 176, 178] {
        poses.extend(
            (0..=356_u16)
                .step_by(6)
                .map(|b| (f32::from(a), f32::from(b) - 178.0)),
        );
    }
    let mut maus = Vec::new();
    for passo in PASSOS {
        let mut pior = (0.0_f64, (0.0, 0.0));
        let mut n = 0;
        for &(p1, p2) in &poses {
            let (d, quinas) = entrada(p1, p2, Some(Refino { passo }));
            let s = ph2d_vec_boolean::silhueta_da_pele(&d, &quinas).unwrap_or(d);
            let v = viragem_maxima(&s.verts);
            if v >= PAREDE_MINIMA {
                n += 1;
            }
            if v > pior.0 {
                pior = (v, (p1, p2));
            }
        }
        println!(
            "  passo {passo}: pior {:.1}° em {:?} · {n} poses com dente",
            pior.0, pior.1
        );
        if n > 0 {
            maus.push((passo, n, pior));
        }
    }
    assert!(maus.is_empty(), "dentes na silhueta: {maus:?}");
}

/// ⭐⭐⭐ **GATE — fora do contacto a silhueta não mexe, a densidade nenhuma** (a lei do
/// `numa_dobra_forte_o_desenho_nao_se_cruza`, a cada `passo`): a barra da cena em C a
/// `30°…150°`; com a silhueta nada se cruza, e onde o desenho não se cruza nem tem zona mais apertada
/// que a bola ele sai AO BIT.
#[test]
fn fora_do_contacto_a_silhueta_nao_mexe_a_densidade_nenhuma() {
    let dobras = [30.0_f32, 45.0, 60.0, 75.0, 90.0, 110.0, 130.0, 150.0];
    let mut maus = Vec::new();
    for passo in PASSOS {
        let mut cruzou = 0;
        for &g in &dobras {
            let (mut sim, _scene, map, id, ossos) =
                crate::barra_da_cena_tests_support::barra_da_cena_com(false);
            for o in ossos.iter().skip(1) {
                sim.world_mut()
                    .get_mut::<ph2d_ecs::Transform>(*o)
                    .expect("Transform")
                    .rotation = g.to_radians();
            }
            let (sem, quinas) = assa_na(
                &sim,
                ph2d_ecs::Entity::from_bits(map[&id]),
                Some(Refino { passo }),
            );
            let com =
                ph2d_vec_boolean::silhueta_da_pele(&sem, &quinas).unwrap_or_else(|| sem.clone());
            if ph2d_vec_boolean::overlap::resolve_overlap(&com).is_some() {
                maus.push((passo, g, "com a silhueta ainda se cruza"));
            }
            let r = RAIO_DO_VINCO * super::super::super::diagonal(&sem);
            if ph2d_vec_boolean::overlap::resolve_overlap(&sem).is_some() {
                cruzou += 1;
            } else if maior_zona_apertada(&sem, 0.9 * r).0 < 12.0 && com != sem {
                maus.push((passo, g, "sem contacto e o desenho mudou"));
            }
        }
        println!("  passo {passo}: {cruzou} dobras cruzam sem a silhueta");
        if cruzou == 0 {
            maus.push((passo, 0.0, "o CONTROLO: nenhuma dobra cruza"));
        }
    }
    assert!(maus.is_empty(), "{maus:?}");
}
