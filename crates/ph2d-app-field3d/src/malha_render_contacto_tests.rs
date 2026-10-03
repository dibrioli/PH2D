//! A sonda da oclusão própria contra o Cycles numa malha que o Render extraiu.

/// ⭐ A sonda contra o Cycles numa malha NOSSA: `PH2D_SONDA_MALHA=<cena>:<objeto>` escreve o `.obj`
/// em `PH2D_SONDA_DIR`, e, se já houver `oraculo_<cena>_<objeto>.csv` lá
/// (`docs/3DModeling/ferramentas/oraculo_oclusao_malha_blender.py`), compara as duas contas nos pontos
/// do Cycles: a lei no VOLUME (o que o app assa) e a de Quilez de antes.
#[test]
#[ignore = "sonda"]
fn sonda_oclusao_da_malha() {
    let dir = std::env::var("PH2D_SONDA_DIR").expect("PH2D_SONDA_DIR");
    let alvo = std::env::var("PH2D_SONDA_MALHA").expect("PH2D_SONDA_MALHA=<cena>:<objeto>");
    let (c, k) = alvo.split_once(':').expect("<cena>:<objeto>");
    let (n_cena, k): (u32, usize) = (c.parse().expect("cena"), k.parse().expect("objeto"));
    let reg = crate::smoke::sampled_registry();
    let doc = crate::smoke::scenes::scene(n_cena);
    let mut sim = ph2d_ecs::SimWorld::new();
    crate::scene::sync_scene(&mut sim, Some(&doc), 0.0);
    let root = {
        let world = sim.world_mut();
        let mut q = world.query::<(bevy_ecs::entity::Entity, &ph2d_field_ecs::FieldObject)>();
        q.iter(world).next().map(|(e, _)| e).expect("a peça")
    };
    let objs = crate::malha_render::extrair(sim.world(), root, &reg);
    let entrada = crate::malha_render::colhe(sim.world(), root);
    let o = &objs[k];
    let docs: Vec<ph2d_field::FieldDoc> = o
        .unidades
        .iter()
        .filter_map(|u| {
            entrada
                .postos
                .iter()
                .find(|(e, _)| e == u)
                .map(|(_, d)| d.clone())
        })
        .collect();
    let proprio = crate::malha_render::uniao(&docs).expect("o campo do objeto");
    let base = std::path::Path::new(&dir);
    let m = &o.malha;
    let mut s = String::new();
    for p in &m.posicoes {
        s += &format!("v {} {} {}\n", p[0], p[1], p[2]);
    }
    for n in &m.normais {
        s += &format!("vn {} {} {}\n", n[0], n[1], n[2]);
    }
    for t in m.indices.chunks(3) {
        s += &format!(
            "f {0}//{0} {1}//{1} {2}//{2}\n",
            t[0] + 1,
            t[1] + 1,
            t[2] + 1
        );
    }
    std::fs::write(base.join(format!("malha_{n_cena}_{k}.obj")), s).expect("obj");
    let csv = base.join(format!("oraculo_{n_cena}_{k}.csv"));
    let Ok(texto) = std::fs::read_to_string(&csv) else {
        println!(
            "SONDA: escrito o .obj; corra o oráculo para {}",
            csv.display()
        );
        return;
    };
    let pts: Vec<[f32; 7]> = texto
        .lines()
        .filter(|l| !l.starts_with('#'))
        .skip(1)
        .map(|l| {
            let v: Vec<f32> = l
                .split(',')
                .skip(2)
                .map(|x| x.parse().expect("n"))
                .collect();
            [v[0], v[1], v[2], v[3], v[4], v[5], v[6]]
        })
        .collect();
    let pos: Vec<[f32; 3]> = pts.iter().map(|p| [p[0], p[1], p[2]]).collect();
    let nrm: Vec<[f32; 3]> = pts
        .iter()
        .map(|p| {
            let l = (p[3] * p[3] + p[4] * p[4] + p[5] * p[5]).sqrt();
            [p[3] / l, p[4] / l, p[5] / l]
        })
        .collect();
    let (mut lo, mut hi) = ([f32::INFINITY; 3], [f32::NEG_INFINITY; 3]);
    for q in &m.posicoes {
        for e in 0..3 {
            lo[e] = lo[e].min(q[e]);
            hi[e] = hi[e].max(q[e]);
        }
    }
    let maior = (0..3).map(|e| hi[e] - lo[e]).fold(0.0, f32::max);
    let diag = (0..3).map(|e| (hi[e] - lo[e]).powi(2)).sum::<f32>().sqrt();
    let mede = |nome: &str, est: &[f32]| {
        let (mut sm, mut sp, mut np, mut vies) = (0.0f32, 0.0f32, 0usize, 0.0f32);
        for (e, p) in est.iter().zip(&pts) {
            sm += (e - p[6]).abs();
            vies += e - p[6];
            if p[6] < 0.9 {
                sp += (e - p[6]).abs();
                np += 1;
            }
        }
        println!(
            "SONDA cena {n_cena} objeto {k} · {nome}: {} pontos · |Δ| médio {:.4} · perto {:.4} ({np}) · viés {:+.4}",
            pts.len(),
            sm / pts.len() as f32,
            sp / np.max(1) as f32,
            vies / pts.len() as f32
        );
    };
    for lado in [64usize, 128] {
        let m2 = maior * (2.0 / (lado - 1) as f32);
        let vol = ph2d_contacto::Volume::de(lo.map(|c| c - m2), hi.map(|c| c + m2), lado, |q| {
            ph2d_field_eval::par::valores(&proprio, &reg, q).expect("campo")
        })
        .como_distancia();
        mede(
            &format!("volume {lado}³"),
            &ph2d_contacto::visibilidade_propria(&vol, &pos, &nrm, diag),
        );
    }
    // A oclusão de Quilez que a casa assava (no campo do objeto, `d = f/|∇f|`).
    let quilez: Vec<f32> = pos
        .iter()
        .zip(&nrm)
        .map(|(p, n)| {
            let (mut occ, mut w) = (0.0f32, 0.0f32);
            for (i, hq) in [0.01f32, 0.02, 0.04, 0.08, 0.16].iter().enumerate() {
                let q = [0, 1, 2].map(|e| p[e] + n[e] * hq);
                let (f, g) =
                    ph2d_field_eval::par::valores_e_gradientes(&proprio, &reg, &[q], 1.0e-4)
                        .expect("campo")[0];
                let m = (g[0] * g[0] + g[1] * g[1] + g[2] * g[2]).sqrt().max(0.05);
                let pe = 0.5f32.powi(i as i32);
                occ += pe * ((hq - f / m) / hq).clamp(0.0, 1.0);
                w += pe;
            }
            1.0 - occ / w
        })
        .collect();
    mede("Quilez (o de antes)", &quilez);
}

/// ⭐ **Uma peça CONVEXA sozinha vê o céu todo** — a oclusão própria assada de uma esfera e de uma
/// caixa é `1` em todo o vértice (e a grelha existe).
#[test]
fn uma_peca_convexa_nao_se_tapa() {
    let reg = crate::smoke::sampled_registry();
    for (nome, folha) in [
        (
            "esfera",
            ph2d_field_eval::leaf(
                ph2d_field::Primitive::Sphere { radius: 0.3 },
                ph2d_field::Xform::at(0.1, 0.3, -0.2),
            ),
        ),
        (
            "caixa",
            ph2d_field_eval::leaf(
                ph2d_field::Primitive::Box {
                    half: [0.2, 0.15, 0.25],
                    round: 0.0,
                    chamfer: 0.0,
                },
                ph2d_field::Xform::at(0.0, 0.15, 0.0),
            ),
        ),
    ] {
        let doc = ph2d_field::FieldDoc::new(vec![folha], ph2d_field::NodeId(0)).expect("doc");
        let mut sim = ph2d_ecs::SimWorld::new();
        crate::scene::sync_scene(&mut sim, Some(&doc), 0.0);
        let root = {
            let world = sim.world_mut();
            let mut q = world.query::<(bevy_ecs::entity::Entity, &ph2d_field_ecs::FieldObject)>();
            q.iter(world).next().map(|(e, _)| e).expect("a peça")
        };
        let objs = crate::malha_render::extrair(sim.world(), root, &reg);
        assert_eq!(objs.len(), 1, "{nome}");
        let ao = &objs[0].malha.ao;
        let min = ao.iter().copied().fold(1.0f32, f32::min);
        let media = ao.iter().sum::<f32>() / ao.len() as f32;
        eprintln!(
            "{nome}: {} vértices · AO mínimo {min:.3} · médio {media:.4}",
            ao.len()
        );
        assert!(objs[0].contacto.is_some(), "{nome}: sem grelha");
        assert!(
            min > 0.97,
            "{nome}: uma peça convexa tapou-se a si: mínimo {min}"
        );
    }
}
