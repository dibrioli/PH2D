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
        });
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

/// ⭐⭐ **A oclusão própria é do OBJETO, não do grupo** — duas caixas a `1 cm` uma da outra caem no
/// mesmo grupo de extração (as bolas de bordo sobrepõem-se) mas são dois objetos: cada uma, convexa,
/// vê o céu todo SOZINHA. A vizinha tapa-a pela grelha dela, com a pose de agora — se a bake lesse o
/// campo do grupo, a vizinha contava duas vezes e ficava presa à pose da extracção.
#[test]
fn a_oclusao_propria_nao_conta_a_vizinha() {
    let reg = crate::smoke::sampled_registry();
    // ⚠️ CAIXAS e não bolas: as bolas de bordo de duas esferas a `1 cm` não se tocam (grupos
    // diferentes, e o gate não distinguia o campo do grupo do do objeto — a mutação sobrevivia); as
    // de duas caixas (meia-diagonal `0,35`) sobrepõem-se.
    let bola = |x: f32| {
        ph2d_field_eval::leaf(
            ph2d_field::Primitive::Box {
                half: [0.2; 3],
                round: 0.0,
                chamfer: 0.0,
            },
            ph2d_field::Xform::at(x, 0.2, 0.0),
        )
    };
    let doc = ph2d_field::FieldDoc::new(
        vec![
            bola(-0.205),
            bola(0.205),
            ph2d_field::Node::new(
                ph2d_field::Xform::IDENTITY,
                ph2d_field::NodeKind::Combine {
                    op: ph2d_field::Op::Union(ph2d_field::Blend::Sharp),
                    children: vec![ph2d_field::NodeId(0), ph2d_field::NodeId(1)],
                },
            ),
        ],
        ph2d_field::NodeId(2),
    )
    .expect("doc");
    let mut sim = ph2d_ecs::SimWorld::new();
    crate::scene::sync_scene(&mut sim, Some(&doc), 0.0);
    let root = {
        let world = sim.world_mut();
        let mut q = world.query::<(bevy_ecs::entity::Entity, &ph2d_field_ecs::FieldObject)>();
        q.iter(world).next().map(|(e, _)| e).expect("a peça")
    };
    let objs = crate::malha_render::extrair(sim.world(), root, &reg);
    assert_eq!(objs.len(), 2, "duas caixas soltas são dois objetos");
    for (k, o) in objs.iter().enumerate() {
        let min = o.malha.ao.iter().copied().fold(1.0f32, f32::min);
        assert!(o.contacto.is_some(), "objeto {k}: sem grelha");
        assert!(
            min > 0.97,
            "objeto {k}: a oclusão própria contou a vizinha (mínimo {min})"
        );
    }
}

/// Um quadro do Render por malha da cena `n` do smoke (a câmara da sonda), ou `None` sem aparelho.
fn quadro_do_render(n: u32, tamanho: (u32, u32)) -> Option<Vec<u8>> {
    let doc = crate::smoke::scenes::scene(n);
    let mut out = None;
    crate::scene::lasso_tests::armed_with(&doc, |sim| {
        crate::malha_render_estado::sync(sim, false, false);
        let slot = crate::shading::Shading::ALL
            .iter()
            .position(|s| *s == crate::shading::Shading::Render)
            .expect("Render");
        ph2d_panel_model3d::state::push_intent_for_test(
            ph2d_panel_model3d::ModelIntent::SetShading { slot },
        );
        crate::scene::apply_intents_for_test(sim.world_mut(), &[]);
        if let Some(mats) = crate::smoke::scenes::materiais_da_cena(n) {
            let world = sim.world_mut();
            let mut q = world.query::<(bevy_ecs::entity::Entity, &ph2d_field_ecs::FieldObject)>();
            let root = q.iter(world).next().map(|(e, _)| e).expect("a peça");
            let folhas = crate::materials::folhas(world, root);
            for ((e, _, _), m) in folhas.iter().zip(mats) {
                world.entity_mut(*e).insert(m);
            }
        }
        crate::smoke::with_smoke(|s| {
            s.vp_mut().cam = ph2d_field_render::Orbit::from_yaw_pitch(0.72, 0.52);
            crate::input::frame_the_part(s);
        });
        let t0 = std::time::Instant::now();
        while !crate::malha_render_estado::com(|e| !e.esperando()).unwrap_or(false)
            && t0.elapsed().as_secs() < 30
        {
            crate::scene::ecs_bridge(sim, None, &[], &crate::scene::no_drawing());
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        let doc = crate::smoke::with_smoke(|s| s.doc.clone())
            .flatten()
            .expect("doc");
        let t1 = std::time::Instant::now();
        while t1.elapsed().as_secs() < 30 {
            match crate::smoke::with_smoke(|s| {
                crate::malha_render_quadro::desenha(s, s.active, tamanho, &doc, false)
            }) {
                Some(crate::malha_render_quadro::Feito::Novo(rgba)) => {
                    out = Some(rgba);
                    break;
                }
                Some(crate::malha_render_quadro::Feito::SemAparelho) | None => break,
                _ => std::thread::sleep(std::time::Duration::from_millis(2)),
            }
        }
    });
    out
}

/// ⭐⭐⭐ **O contacto CHEGA ao quadro do Render** — a costura app → desenhista: a cena 40 desenhada
/// com as grelhas que o app assa e sem elas (o mesmo mundo, a mesma câmara). Com elas, as peças
/// escurecem onde se olham (milhares de pixels, e bastante); nenhum pixel CLAREIA (o contacto só
/// tapa céu). Controlo: sem as grelhas, o quadro é outro.
#[test]
#[ignore = "precisa de aparelho"]
fn o_contacto_chega_ao_quadro_do_render() {
    let tamanho = (480u32, 270u32);
    // ⚠️ COM primeiro: o desenhista é global, e o quadro SEM tem de não herdar as grelhas do outro
    // (subir uma malha no mesmo id esquece a grelha velha).
    let com = quadro_do_render(40, tamanho);
    crate::malha_render_quadro::SEM_CONTACTO.store(true, std::sync::atomic::Ordering::Relaxed);
    let sem = quadro_do_render(40, tamanho);
    crate::malha_render_quadro::SEM_CONTACTO.store(false, std::sync::atomic::Ordering::Relaxed);
    let (Some(sem), Some(com)) = (sem, com) else {
        eprintln!("sem aparelho — o gate não corre aqui");
        return;
    };
    let lum =
        |p: &[u8]| 0.2126 * f32::from(p[0]) + 0.7152 * f32::from(p[1]) + 0.0722 * f32::from(p[2]);
    let (mut escuros, mut pior, mut clareou, mut pecas) = (0usize, 1.0f32, 0usize, 0usize);
    for (a, b) in com.as_chunks::<4>().0.iter().zip(sem.as_chunks::<4>().0) {
        if a[3] < 255 || b[3] < 255 {
            continue;
        }
        pecas += 1;
        let r = lum(a) / lum(b).max(1.0);
        pior = pior.min(r);
        if r < 0.9 {
            escuros += 1;
        }
        if lum(a) > lum(b) + 2.0 {
            clareou += 1;
        }
    }
    eprintln!(
        "{pecas} px de peça · {escuros} escurecem > 10 % · o mais escuro × {pior:.3} · {clareou} clareiam"
    );
    assert!(pecas > 10_000, "a cena encolheu: {pecas} px");
    assert!(
        escuros > 500 && pior < 0.7,
        "o contacto não chegou ao quadro: {escuros} px · × {pior}"
    );
    assert!(clareou < 20, "o contacto CLAREOU {clareou} px");
}
