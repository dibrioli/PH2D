//! ⭐⭐⭐ **A DENSIDADE do bake não muda o que a silhueta faz** (A13). A silhueta do contacto (união,
//! ganchos, bola) tem de dar a MESMA resposta a metade, ao dobro e ao produto (`AMOSTRAS_POR_FORMA`)
//! — o óptimo da amostragem mudou com a lei do ângulo, e a união não pode depender dele (a identidade
//! fora do contacto, até ao produto: ver o gate dela).

use super::super::maior_zona_apertada;
use super::dentes::{assa_na, entrada};
use crate::skin_desenho::AMOSTRAS_POR_FORMA;
use ph2d_vec_boolean::overlap::{PAREDE_MINIMA, RAIO_DO_VINCO};

/// As densidades da varredura (amostras por forma): metade, o produto e o dobro.
const DENSIDADES: [usize; 3] = [
    AMOSTRAS_POR_FORMA / 2,
    AMOSTRAS_POR_FORMA,
    2 * AMOSTRAS_POR_FORMA,
];

fn viragem_maxima(v: &[ph2d_vec_scene::VecVertex]) -> f64 {
    (0..v.len())
        .filter_map(|i| ph2d_vec_boolean::overlap::viragem_do_vertice(v, i))
        .fold(0.0, f64::max)
}

/// ⭐⭐⭐ **GATE — o braço dobrado de volta não tem dentes a densidade nenhuma**: a grelha de `387`
/// poses (`primeira ∈ {160, 165, 170, 172, 174, 176, 178}`, `segunda` de `−178` a `178` de `6` em
/// `6`) a cada densidade, e nenhum nó da silhueta passa da `PAREDE_MINIMA`.
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
    for por_forma in DENSIDADES {
        let mut pior = (0.0_f64, (0.0, 0.0));
        let mut n = 0;
        for &(p1, p2) in &poses {
            let (d, quinas) = entrada(p1, p2, por_forma);
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
            "  {por_forma} amostras: pior {:.1}° em {:?} · {n} poses com dente",
            pior.0, pior.1
        );
        if n > 0 {
            maus.push((por_forma, n, pior));
        }
    }
    assert!(maus.is_empty(), "dentes na silhueta: {maus:?}");
}

/// ⭐⭐⭐ **GATE — o braço ABERTO com a 2.ª junta a `±110°…134°` não tem dentes a densidade nenhuma**
/// (A13, a varredura de `2°` em `2°` do produto: a `(22°…28°, 130°)` a união deixava um LAÇO de `0,01`
/// e a ponta dele ficava a `75°`–`145°` — [`ph2d_vec_boolean::laco`]; e a `(20…48, −112…−128)` os
/// esporões de `180°` da medição de 06/10). A metade e no produto. ⚠️ Ao dobro há UM nó a `17,9°` a
/// `(36°, 118°)`, longe do vinco (`355` soldas): nasce no fecho, que ali arredonda um bico da malha
/// do campo que só a `2048` amostras se resolve — o lado de lá da amostragem óptima, a família do
/// [`fora_do_contacto_a_silhueta_nao_mexe_a_densidade_nenhuma`].
#[test]
fn o_braco_aberto_nao_tem_dentes_a_densidade_nenhuma() {
    let mut poses: Vec<(f32, f32)> = Vec::new();
    for a in (20..=48_u16).step_by(2) {
        for b in (110..=134_u16).step_by(4) {
            poses.extend([(f32::from(a), f32::from(b)), (f32::from(a), -f32::from(b))]);
        }
    }
    let mut maus = Vec::new();
    for por_forma in &DENSIDADES[..2] {
        let por_forma = *por_forma;
        let mut pior = (0.0_f64, (0.0, 0.0));
        let mut n = 0;
        for &(p1, p2) in &poses {
            let (d, quinas) = entrada(p1, p2, por_forma);
            let s = ph2d_vec_boolean::silhueta_da_pele(&d, &quinas).unwrap_or(d);
            let v = viragem_maxima(&s.verts);
            n += usize::from(v >= PAREDE_MINIMA);
            if v > pior.0 {
                pior = (v, (p1, p2));
            }
        }
        println!(
            "  {por_forma} amostras: pior {:.1}° em {:?} · {n} poses com dente",
            pior.0, pior.1
        );
        if n > 0 {
            maus.push((por_forma, n, pior));
        }
    }
    assert!(maus.is_empty(), "dentes na silhueta: {maus:?}");
}

/// ⭐⭐⭐ **GATE — fora do contacto a silhueta não mexe, a densidade nenhuma até ao produto** (a lei
/// do `numa_dobra_forte_o_desenho_nao_se_cruza`, a metade e no produto): a barra da cena em C a
/// `30°…150°`; com a silhueta nada se cruza, e onde o desenho não se cruza nem tem zona mais apertada
/// que a bola ele sai AO BIT.
///
/// ⛔ Ao DOBRO não vale, e está medido (sonda [`diag_quem_mexe_fora_do_contacto`]): a `2048` o bake
/// já resolve os bicos da malha do campo em viragens côncavas mais apertadas que a bola (`9°`–`16°`
/// de giro abaixo de `0,9 r` a `30°`–`75°`) e o fecho arredonda-as — é o lado de lá da amostragem
/// óptima ([`crate::skin_desenho::AMOSTRAS_POR_FORMA`]), não uma passagem a correr fora do contacto.
#[test]
fn fora_do_contacto_a_silhueta_nao_mexe_a_densidade_nenhuma() {
    let dobras = [30.0_f32, 45.0, 60.0, 75.0, 90.0, 110.0, 130.0, 150.0];
    let mut maus = Vec::new();
    for por_forma in &DENSIDADES[..2] {
        let por_forma = *por_forma;
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
            let (sem, quinas) = assa_na(&sim, ph2d_ecs::Entity::from_bits(map[&id]), por_forma);
            let com =
                ph2d_vec_boolean::silhueta_da_pele(&sem, &quinas).unwrap_or_else(|| sem.clone());
            if ph2d_vec_boolean::overlap::resolve_overlap(&com).is_some() {
                maus.push((por_forma, g, "com a silhueta ainda se cruza"));
            }
            let r = RAIO_DO_VINCO * super::super::super::diagonal(&sem);
            if ph2d_vec_boolean::overlap::resolve_overlap(&sem).is_some() {
                cruzou += 1;
            } else if maior_zona_apertada(&sem, 0.9 * r).0 < 12.0 && com != sem {
                maus.push((por_forma, g, "sem contacto e o desenho mudou"));
            }
        }
        println!("  {por_forma} amostras: {cruzou} dobras cruzam sem a silhueta");
        if cruzou == 0 {
            maus.push((por_forma, 0.0, "o CONTROLO: nenhuma dobra cruza"));
        }
    }
    assert!(maus.is_empty(), "{maus:?}");
}

/// SONDA — qual passagem da silhueta mexe no desenho SEM contacto (a barra em C, `30°`/`60°`) a cada
/// densidade, e onde.
#[test]
#[ignore = "sonda: imprime"]
fn diag_quem_mexe_fora_do_contacto() {
    use ph2d_vec_boolean::overlap::SOLDA_DA_QUINA;
    for por_forma in DENSIDADES {
        for g in [30.0_f32, 45.0, 60.0, 75.0, 90.0] {
            let (mut sim, _scene, map, id, ossos) =
                crate::barra_da_cena_tests_support::barra_da_cena_com(false);
            for o in ossos.iter().skip(1) {
                sim.world_mut()
                    .get_mut::<ph2d_ecs::Transform>(*o)
                    .expect("Transform")
                    .rotation = g.to_radians();
            }
            let (sem, quinas) = assa_na(&sim, ph2d_ecs::Entity::from_bits(map[&id]), por_forma);
            let d = super::super::super::super::diagonal(&sem);
            let (solda, raio) = (SOLDA_DA_QUINA * d, RAIO_DO_VINCO * d);
            let s0 = sem.verts.clone();
            let s1 = ph2d_vec_boolean::gancho::desfaz_os_ganchos(s0.clone(), &quinas, solda);
            let cruza = ph2d_vec_boolean::overlap::resolve_overlap(&sem).is_some();
            let s5 = ph2d_vec_boolean::bola::rola_a_bola(s1.clone(), &quinas, raio, solda);
            let paredes: Vec<([f64; 2], f64)> = quinas.iter().map(|&(p, _)| (p, 180.0)).collect();
            let s6 =
                ph2d_vec_boolean::bola::rola_a_bola_por_dentro(s5.clone(), &paredes, raio, solda);
            let mut s8 = s6.clone();
            ph2d_vec_boolean::overlap::limpa_as_alcas(&mut s8, 0.01 * solda);
            let dif = |a: &[ph2d_vec_scene::VecVertex],
                       b: &[ph2d_vec_scene::VecVertex]|
             -> String {
                if a == b {
                    return "=".into();
                }
                if a.len() != b.len() {
                    return format!("{}→{} nós", a.len(), b.len());
                }
                let k = (0..a.len()).find(|&k| a[k] != b[k]).unwrap_or(0);
                format!(
                    "nó {k} {:?} in {:?}/{:?} out {:?}/{:?}",
                    a[k].anchor, a[k].in_handle, b[k].in_handle, a[k].out_handle, b[k].out_handle
                )
            };
            println!(
                "  {por_forma} {g}° cruza {cruza} · zona {:.1} · gancho {} · fecho {} · abertura {} · alças {}",
                maior_zona_apertada(&sem, 0.9 * raio).0,
                dif(&s0, &s1),
                dif(&s1, &s5),
                dif(&s5, &s6),
                dif(&s6, &s8)
            );
        }
    }
}
