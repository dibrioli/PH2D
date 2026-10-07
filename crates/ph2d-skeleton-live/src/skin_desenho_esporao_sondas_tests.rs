//! Sonda: o esporão de `180°` que a silhueta deixa no braço em C com a lei do meio-ângulo
//! (`(40°, −118°)` e vizinhos, `diag_a_varredura_da_dobra`).

use super::super::diagonal;
use super::braco_em;
use ph2d_vec_scene::VecVertex;

fn viragens(vs: &[VecVertex]) -> Vec<(usize, f64)> {
    (0..vs.len())
        .filter_map(|i| ph2d_vec_boolean::overlap::viragem_do_vertice(vs, i).map(|v| (i, v)))
        .filter(|(_, v)| *v > 15.0)
        .collect()
}

#[test]
#[ignore = "sonda: imprime"]
fn diag_o_esporao_do_braco_em_c() {
    let poses: Vec<(f32, f32)> = std::env::var("SONDA_POSE")
        .ok()
        .and_then(|s| {
            s.split_once(',')
                .map(|(a, b)| (a.trim().parse().ok(), b.trim().parse().ok()))
        })
        .and_then(|(a, b)| Some(vec![(a?, b?)]))
        .unwrap_or_else(|| vec![(40.0, -118.0), (34.0, -122.0), (36.0, -144.0)]);
    for (a, b) in poses {
        let (sem, com) = braco_em(a, b);
        let d = diagonal(&sem);
        let solda = ph2d_vec_boolean::overlap::SOLDA_DA_QUINA * d;
        let raio = ph2d_vec_boolean::overlap::RAIO_DO_VINCO * d;
        println!("({a}, {b}) diagonal {d:.3} solda {solda:.4} raio {raio:.4}");
        let g0 = ph2d_vec_boolean::gancho::desfaz_os_ganchos(sem.verts.clone(), &[], solda);
        let mut p0 = sem.clone();
        p0.verts = g0;
        let Some(u) = ph2d_vec_boolean::resolve_overlap(&p0) else {
            println!("  não cruza");
            continue;
        };
        let g1 = ph2d_vec_boolean::gancho::desfaz_os_ganchos(u.verts.clone(), &[], solda);
        let e1 = ph2d_vec_boolean::esporao::tira_os_esporoes(g1.clone(), &[], solda);
        let r1 = ph2d_vec_boolean::bola::rola_a_bola(e1.clone(), &[], raio, solda);
        for (nome, vs) in [
            ("união", &u.verts),
            ("ganchos", &g1),
            ("esporões", &e1),
            ("bola", &r1),
            ("PRODUTO", &com.verts),
        ] {
            let v = viragens(vs);
            let txt: Vec<String> = v
                .iter()
                .map(|(i, x)| format!("v{i}@{:.3?} {x:.1}°", vs[*i].anchor))
                .collect();
            println!("  {nome:>9}: {} nós · {}", vs.len(), txt.join(" · "));
        }
        for (nome, vs) in [("união", &e1)] {
            if let Some(&(i, _)) = viragens(vs).first() {
                let n = vs.len();
                for k in [
                    (i + n - 2) % n,
                    (i + n - 1) % n,
                    i,
                    (i + 1) % n,
                    (i + 2) % n,
                ] {
                    let v = &vs[k];
                    println!(
                        "    {nome} v{k}: in {:.4?} anchor {:.4?} out {:.4?}",
                        v.in_handle, v.anchor, v.out_handle
                    );
                }
            }
        }
        // O vizinho do pior nó do produto: as cúbicas de um lado e do outro.
        if let Some(&(i, _)) = viragens(&com.verts)
            .iter()
            .max_by(|x, y| x.1.total_cmp(&y.1))
        {
            let n = com.verts.len();
            for k in [
                (i + n - 2) % n,
                (i + n - 1) % n,
                i,
                (i + 1) % n,
                (i + 2) % n,
            ] {
                let v = &com.verts[k];
                println!(
                    "    v{k}: in {:.4?} anchor {:.4?} out {:.4?}",
                    v.in_handle, v.anchor, v.out_handle
                );
            }
        }
    }
}
