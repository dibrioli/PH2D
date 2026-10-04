//! A7 — a SAÍDA RÁPIDA do recorte: sem triângulo virado nem dois triângulos posados que não se
//! tocam a sobrepor-se, nada fica tapado. Irmão de [`super`] pelo tecto de LOC.

use super::super::Posada;
use crate::skin_live::tests::palco;
use ph2d_skeleton_ecs::SkinBind;
use ph2d_vec_scene::VecPath;
use ph2d_vec_scene::effect::{FxEntry, PathEffect};

pub(super) type Fixtura = (
    VecPath,
    Vec<f64>,
    ph2d_vec_skin::pesos::CampoDoDominio,
    ph2d_skeleton::Skin,
    Vec<f64>,
);

/// A barra `40 × 10` com *Hatch* (`8`), presa a dois ossos e a ponta a `graus`.
pub(super) fn riscas_dobradas(graus: f32) -> Fixtura {
    let (mut sim, mut scene, map, id, [_, ponta]) = palco();
    scene.path_mut(id).expect("path").effects = vec![FxEntry::new(PathEffect::Hatch(
        ph2d_vec_scene::fx_hatch::HatchSpec {
            angle: 45.0,
            spacing: 8.0,
            cross: false,
        },
    ))];
    assert_eq!(
        crate::skin_live::bind(&mut sim, &mut scene, &map, &[id], None),
        1
    );
    sim.world_mut()
        .get_mut::<ph2d_ecs::Transform>(ponta)
        .expect("Transform")
        .rotation = graus.to_radians();
    let e = ph2d_ecs::Entity::from_bits(map[&id]);
    let skin = sim.world().get::<SkinBind>(e).expect("pele").clone();
    let g = crate::skinned_mesh::le(&skin.source).expect("fonte");
    let index = crate::skin_live::bone_index(&sim);
    let pele = crate::skin_live::resolve(&sim, &skin, e, &index).expect("pele");
    let prof = crate::esqueletos::profundidades(&sim, &skin, &index);
    (g.path, g.pesos, g.campo.expect("campo"), pele, prof)
}

fn fixturas(graus: f32) -> [(&'static str, Fixtura); 3] {
    [
        ("riscas", riscas_dobradas(graus)),
        ("barra em S", super::barra_em_s(graus)),
        ("cópias", super::fechados::copias_dobradas(graus)),
    ]
}

/// A maior folga entre os triângulos `a` e `b` ao longo das normais das seis arestas: positiva =
/// separados (é um MÍNIMO da distância); `≤ 0` = sobrepõem-se ou tocam-se.
fn folga(a: [[f64; 2]; 3], b: [[f64; 2]; 3]) -> f64 {
    let mut melhor = f64::MIN;
    for (p, q) in [(a, b), (b, a)] {
        for e in 0..3 {
            let (u, v, w) = (p[e], p[(e + 1) % 3], p[(e + 2) % 3]);
            let d = [v[0] - u[0], v[1] - u[1]];
            let l = d[0].hypot(d[1]);
            if l < 1e-18 {
                continue;
            }
            let mut n = [d[1] / l, -d[0] / l];
            if n[0] * (w[0] - u[0]) + n[1] * (w[1] - u[1]) > 0.0 {
                n = [-n[0], -n[1]];
            }
            let g = q
                .iter()
                .map(|x| n[0] * (x[0] - u[0]) + n[1] * (x[1] - u[1]))
                .fold(f64::MAX, f64::min);
            melhor = melhor.max(g);
        }
    }
    melhor
}

fn aresta_min(t: [[f64; 2]; 3]) -> f64 {
    (0..3)
        .map(|e| (t[(e + 1) % 3][0] - t[e][0]).hypot(t[(e + 1) % 3][1] - t[e][1]))
        .fold(f64::MAX, f64::min)
}

/// ⭐ **SONDA — o que a saída rápida precisa de saber:** (1) quanto o ponto POSADO (`onde`, a pele
/// não-linear) se afasta do triângulo posado LINEAR do dono, em arestas; (2) a menor folga entre
/// triângulos que não se tocam, em arestas; (3) virados e pares sobrepostos de chave diferente.
#[test]
#[ignore = "sonda: imprime"]
fn diag_o_desvio_e_a_folga_da_malha_posada() {
    for graus in [
        0f32, 15.0, 30.0, 45.0, 60.0, 75.0, 90.0, 100.0, 110.0, 130.0, 150.0,
    ] {
        for (nome, (fonte, _, campo, pele, prof)) in fixturas(graus) {
            let f = Posada::nova(&campo, None, &pele, &[], true, &prof)
                .expect("posada")
                .com_a_arte(&fonte);
            let tris = &campo.malha.tris;
            let tri = |i: usize, de: &[[f64; 2]]| tris[i].map(|v| de[v as usize]);
            let mut desvio = 0.0_f64;
            for i in 0..tris.len() {
                let r = tri(i, &f.repouso);
                for (a, b) in [(0.2, 0.2), (0.6, 0.2), (0.2, 0.6), (1.0 / 3.0, 1.0 / 3.0)] {
                    let p = [0, 1].map(|j| (1.0 - a - b) * r[0][j] + a * r[1][j] + b * r[2][j]);
                    let Some((d, q)) = f.onde(p) else { continue };
                    let (rd, pd) = (tri(d, &f.repouso), tri(d, &f.pos));
                    let Some((u, v)) = super::super::bari(p, rd[0], rd[1], rd[2]) else {
                        continue;
                    };
                    let lin =
                        [0, 1].map(|j| (1.0 - u - v) * pd[0][j] + u * pd[1][j] + v * pd[2][j]);
                    desvio = desvio.max((q[0] - lin[0]).hypot(q[1] - lin[1]) / aresta_min(pd));
                }
            }
            let (mut folga_min, mut sobrepostos) = (f64::MAX, 0);
            for i in 0..tris.len() {
                let ti = tri(i, &f.pos);
                let lo = [0, 1].map(|k| ti.iter().map(|p| p[k]).fold(f64::MAX, f64::min));
                let hi = [0, 1].map(|k| ti.iter().map(|p| p[k]).fold(f64::MIN, f64::max));
                let (c0, c1) = (f.grelha.celula(lo), f.grelha.celula(hi));
                for y in c0[1]..=c1[1] {
                    for x in c0[0]..=c1[0] {
                        for &k in &f.grelha.baldes[y * f.grelha.dim[0] + x] {
                            let k = k as usize;
                            if k <= i || tris[k].iter().any(|v| tris[i].contains(v)) {
                                continue;
                            }
                            let tk = tri(k, &f.pos);
                            let g = folga(ti, tk) / aresta_min(ti).min(aresta_min(tk));
                            if g <= 0.0 {
                                sobrepostos += usize::from(f.chave_tri[k] != f.chave_tri[i]);
                            } else {
                                folga_min = folga_min.min(g);
                            }
                        }
                    }
                }
            }
            let virados = f.virado.iter().filter(|v| **v).count();
            println!(
                "  {graus:>5}° {nome:<10} tris {:>4} · desvio máx {desvio:.4} arestas · folga mín \
                 {folga_min:.4} arestas · sobrepostos {sobrepostos} · virados {virados}",
                tris.len()
            );
        }
    }
}

/// `(pares de chave diferente que se sobrepõem, virados)` da malha posada de `f`.
fn sobreposicoes(f: &Posada<'_>) -> (usize, usize) {
    let tris = &f.campo.malha.tris;
    let tri = |i: usize| tris[i].map(|v| f.pos[v as usize]);
    let mut n = 0;
    for i in 0..tris.len() {
        let ti = tri(i);
        let lo = [0, 1].map(|k| ti.iter().map(|p| p[k]).fold(f64::MAX, f64::min));
        let hi = [0, 1].map(|k| ti.iter().map(|p| p[k]).fold(f64::MIN, f64::max));
        let (c0, c1) = (f.grelha.celula(lo), f.grelha.celula(hi));
        for y in c0[1]..=c1[1] {
            for x in c0[0]..=c1[0] {
                for &k in &f.grelha.baldes[y * f.grelha.dim[0] + x] {
                    let k = k as usize;
                    if f.chave_tri[k] > f.chave_tri[i]
                        && !tris[k].iter().any(|v| tris[i].contains(v))
                        && folga(ti, tri(k)) <= 0.0
                    {
                        n += 1;
                    }
                }
            }
        }
    }
    (n, f.virado.iter().filter(|v| **v).count())
}

/// ⭐ **SONDA — a lei corta alguma coisa numa pose SEM sobreposição nem virados?** Varre `0°…90°`.
#[test]
#[ignore = "sonda: imprime"]
fn diag_a_lei_corta_sem_sobreposicao() {
    let mut graus = 0f32;
    while graus <= 90.0 {
        for (nome, (fonte, pesos, campo, pele, prof)) in fixturas(graus) {
            let f = Posada::nova(&campo, None, &pele, &[], true, &prof)
                .expect("posada")
                .com_a_arte(&fonte);
            let (sob, vir) = sobreposicoes(&f);
            let riscas = super::super::so_o_que_se_ve(
                &fonte,
                &pesos,
                (&campo, None),
                (&pele, &[], true),
                &prof,
            )
            .is_some();
            let fechados = super::super::cortes_dos_fechados(
                &fonte,
                (&campo, None),
                (&pele, &[], true),
                &prof,
            )
            .is_some();
            if sob == 0 && (riscas || fechados) || graus % 15.0 == 0.0 {
                println!(
                    "  {graus:>5}° {nome:<10} sobrepostos {sob:>4} virados {vir:>3} · corta riscas {riscas} fechados {fechados}"
                );
            }
        }
        graus += 2.5;
    }
}

/// O que as duas portas do recorte devolvem, escrito AO BIT (o `Debug` de um `f64` é o menor texto
/// que o relê igual).
fn recorte(fx: &Fixtura, sem_saida: bool) -> String {
    let (fonte, pesos, campo, pele, prof) = fx;
    super::super::SEM_SAIDA_RAPIDA.with(|c| c.set(sem_saida));
    let riscas = super::super::so_o_que_se_ve(fonte, pesos, (campo, None), (pele, &[], true), prof);
    let fechados = super::super::cortes_dos_fechados(fonte, (campo, None), (pele, &[], true), prof);
    super::super::SEM_SAIDA_RAPIDA.with(|c| c.set(false));
    format!("{riscas:?} {fechados:?}")
}

/// ⭐⭐⭐ **GATE — a SAÍDA RÁPIDA não muda o recorte AO BIT** (A7), de `0°` a `150°` nas três fixturas
/// (riscas, a barra em S da `=5`, as cópias da `=6`).
///
/// ⛔ **O CONTROLO:** a saída dispara (sem dobra) e não dispara (na dobra) dentro da varredura — e
/// onde não dispara há mesmo um corte.
#[test]
fn a_saida_rapida_nao_muda_o_recorte_ao_bit() {
    let (mut saiu, mut cortou) = (0, 0);
    for graus in [
        0f32, 15.0, 30.0, 45.0, 60.0, 75.0, 90.0, 110.0, 130.0, 150.0,
    ] {
        for (nome, fx) in fixturas(graus) {
            let antes = super::super::AMOSTRAGENS.with(std::cell::Cell::get);
            let com = recorte(&fx, false);
            let amostrou = super::super::AMOSTRAGENS.with(std::cell::Cell::get) > antes;
            let sem = recorte(&fx, true);
            assert_eq!(
                com, sem,
                "{nome} a {graus}°: a saída rápida mudou o recorte"
            );
            let (_, _, campo, pele, prof) = &fx;
            let f = Posada::nova(campo, None, pele, &[], true, prof).expect("posada");
            // ⭐ E a saída POUPA: quando dispara nenhum contorno é amostrado (M8 sobrevivia).
            assert!(
                !(f.nada_tapa() && amostrou),
                "{nome} a {graus}°: a saída disparou e o contorno foi amostrado"
            );
            saiu += usize::from(f.nada_tapa());
            cortou += usize::from(!f.nada_tapa() && com != "None None");
        }
    }
    println!("  a saída rápida disparou em {saiu} de 30; cortou em {cortou}");
    assert!(
        saiu >= 10,
        "o CONTROLO: a saída rápida quase não disparou ({saiu})"
    );
    assert!(
        cortou >= 8,
        "o CONTROLO: a varredura não corta na dobra ({cortou})"
    );
}

/// ⭐⭐ **GATE — a saída rápida é a lei: sem um par sobreposto nada fica tapado**, e um par ou (com
/// o avesso a tapar) um triângulo virado chega para não sair. Sobre a função pura.
#[test]
fn a_saida_rapida_so_sai_sem_par_nem_virado() {
    use super::super::malha::nada_tapa;
    assert!(nada_tapa(false, true, &[false, false]));
    assert!(!nada_tapa(true, true, &[false, false]), "um par sobreposto");
    assert!(
        !nada_tapa(false, true, &[false, true]),
        "um virado com o avesso a tapar"
    );
    assert!(
        nada_tapa(false, false, &[false, true]),
        "o virado sem o avesso a tapar (o traço dos fechados)"
    );
}
