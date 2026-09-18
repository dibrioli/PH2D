//! ⭐⭐⭐ **O CUSTO DO QUADRO DA CENA DO DONO** — Boids + Shape + colisão.
//!
//! Report de 2026-09-18: *«aqui queremos lidar com centenas a milhares de objetos em runtime.
//! Algumas poucas centenas já trava usando Boids+Shape com colisão»*.
//!
//! ⚠️ **Estas sondas existem porque eu tinha medido o PASSE e não o QUADRO** — e as duas primeiras
//! redacções delas mediram o vazio, cada uma por uma razão diferente:
//!
//! - chamar `cook` num tique parado bate no **MEMO** (lia `0,002 ms` para 500 boids);
//! - a `source.shape` lê um **EXTERNAL que a shell publica**, e sem ele emite zero linhas;
//! - e o controlo contava `instances` quando uma forma desce para **`vector_instances`** — a lei do
//!   `geometry_id`. *Contar a lista errada lê-se exactamente como uma cena vazia.*
//!
//! ⇒ toda sonda aqui imprime a POPULAÇÃO que produziu, ao lado do relógio.

use crate::motion_state::MotionState;

/// O índice do `Square` no enum `kind` da `source.shape`, LIDO do registo.
///
/// ⚠️ **Não é um literal:** um índice de enum é uma posição numa lista que outra pessoa pode
/// reordenar — a lei que a `sim_demo::indice_de` já escreve para as cenas. Aqui ela é repetida em
/// miniatura porque aquela é privada da família das cenas, e uma sonda não é uma cena.
fn indice_do_quadrado(reg: &ph2d_node_registry::NodeRegistry) -> Option<f32> {
    use ph2d_node_registry::ParamWidget;
    let tid = ph2d_nodegraph::node::NodeTypeId::of("source.shape");
    let hint = reg.param_ui(tid)?.iter().find(|h| h.param == "kind")?;
    let ParamWidget::Enum { labels } = hint.widget else {
        return None;
    };
    let i = labels.iter().position(|l| *l == "Square")?;
    #[expect(
        clippy::cast_precision_loss,
        reason = "um indice de enum, sempre pequeno"
    )]
    Some(i as f32)
}

/// ⭐⭐⭐ **A CENA DO DONO — Boids + Shape + colisão — corre na PLACA ou no caminho LENTO?**
///
/// Report de 2026-09-18: *«aqui queremos lidar com centenas a milhares de objetos em runtime.
/// Algumas poucas centenas já trava usando Boids+Shape com colisão»*.
///
/// ⚠️ A cena `=7` shipa `motion.boids` com `count = 1 048 576` a 60 fps **no dispositivo**, e a
/// mesma rota na CPU é `O(N²)`. ⇒ antes de optimizar seja o que for, a pergunta é qual das duas
/// rotas a cena dele toma, e o que a decide.
#[test]
#[ignore = "sonda de medição, não gate"]
fn sonda_a_cena_do_dono_corre_na_placa() {
    let m = MotionState::new();
    let reg = &m.registry;
    for (nome, com_colisao) in [("Boids + Shape", false), ("Boids + Shape + COLISAO", true)] {
        let mut doc = ph2d_motion_doc::MotionDoc::default();
        let g = &mut doc.graph;
        let boids = g.add_node("motion.boids");
        g.set_param(boids, "count", 500.0);
        let forma = g.add_node("source.shape");
        g.set_param(forma, ph2d_node_motion_shape::param::SIZE, 0.09);
        if com_colisao {
            g.set_param(forma, ph2d_node_motion_shape::param::COLLIDE, 1.0);
        }
        let dup = g.add_node("motion.duplicator");
        let saida = g.add_node("motion.output");
        if com_colisao {
            g.set_param(saida, ph2d_eval_motion::SINK_COLLIDE_PARAM, 1.0);
        }
        // O laço de estado do boids, que é como toda simulação desta casa fecha.
        for (a, ap, b, bp, atrasada) in [
            (boids, 2u16, boids, 2u16, true),
            (forma, 0, dup, 0, false),
            (boids, 0, dup, 1, false),
            (dup, 0, saida, 0, false),
        ] {
            let _ = g.connect(ph2d_nodegraph::graph::Edge {
                from: (a, ap),
                to: (b, bp),
                delayed: atrasada,
            });
        }
        let plan = ph2d_gpu_cook::plan(&doc.graph, reg, reg, saida);
        let fronteira: Vec<String> = plan
            .boundaries
            .iter()
            .map(|&(n, _)| {
                doc.graph
                    .node(n)
                    .map_or("?".into(), |i| i.type_name.clone())
            })
            .collect();
        eprintln!(
            "  {nome:<26} -> {} · {} etapa(s) de GPU · fronteira: {}",
            if plan.is_fully_gpu() {
                "PLACA"
            } else {
                "CAMINHO LENTO"
            },
            plan.dispatching_stages(reg),
            if fronteira.is_empty() {
                "—".into()
            } else {
                fronteira.join(", ")
            }
        );
    }
}

/// ⭐⭐⭐ **O QUADRO DA CENA DO DONO, PEDAÇO A PEDAÇO** — *«algumas poucas centenas já trava»*.
#[test]
#[ignore = "sonda de medição, não gate"]
fn sonda_o_quadro_da_cena_do_dono() {
    use std::time::Instant;
    eprintln!("\n  ═══ O QUADRO, PEDAÇO A PEDAÇO (Boids + Shape + colisão, CPU) ═══\n");
    eprintln!(
        "  {:<8} │ {:>11} │ {:>13} │ {:>11} │ {:>9} │ {:>7}",
        "boids", "so' boids", "+ forma/dup", "+ colisao", "% quadro", "FPS"
    );
    eprintln!("  ---------|-------------|---------------|-------------|-----------|--------");
    for n in [100usize, 250, 500, 1000, 2000] {
        let mut col = [0.0f64; 3];
        for (i, (com_forma, com_colisao)) in [(false, false), (true, false), (true, true)]
            .into_iter()
            .enumerate()
        {
            let mut m = MotionState::new();
            let quadrado = indice_do_quadrado(&m.registry);
            let g = &mut m.doc.graph;
            let boids = g.add_node("motion.boids");
            #[expect(clippy::cast_precision_loss, reason = "uma contagem de cena")]
            g.set_param(boids, "count", n as f32);
            let saida = g.add_node("motion.output");
            let mut arestas = vec![(boids, 2u16, boids, 2u16, true)];
            if com_forma {
                let forma = g.add_node("source.shape");
                // ⚠️ O `kind` vem do REGISTO, como na cena `=123` — sem ele a forma não emite nada.
                if let Some(q) = quadrado {
                    g.set_param(forma, ph2d_node_motion_shape::param::KIND, q);
                }
                g.set_param(forma, ph2d_node_motion_shape::param::SIZE, 0.09);
                if com_colisao {
                    g.set_param(forma, ph2d_node_motion_shape::param::COLLIDE, 1.0);
                    g.set_param(saida, ph2d_eval_motion::SINK_COLLIDE_PARAM, 1.0);
                }
                let dup = g.add_node("motion.duplicator");
                arestas.push((forma, 0, dup, 0, false));
                arestas.push((boids, 0, dup, 1, false));
                arestas.push((dup, 0, saida, 0, false));
            } else {
                arestas.push((boids, 0, saida, 0, false));
            }
            for (a, ap, b, bp, atrasada) in arestas {
                // ⚠️ Falha ALTO: a 1.ª redacção ignorava o `Result` e a sonda media um grafo
                // desligado (o controlo leu `0 instancias`).
                g.connect(ph2d_nodegraph::graph::Edge {
                    from: (a, ap),
                    to: (b, bp),
                    delayed: atrasada,
                })
                .unwrap_or_else(|e| panic!("aresta {ap}->{bp} recusada: {e:?}"));
            }
            // ⚠️⚠️ **Pela porta do QUADRO, e não por `cook`**: o memo do cozedor é chaveado
            // pelo TIQUE, e o tique só anda dentro da marcha — medir com `cook` num tique parado
            // lia `0,002 ms` para 500 boids, que é o memo a devolver o quadro anterior.
            let uv = [0.0, 0.0, 1.0, 1.0];
            let tam = [1.0, 1.0];
            let mut tique = 0u64;
            let marcha = |m: &mut MotionState, t: &mut u64| {
                // ⚠️ **A `source.shape` lê um EXTERNAL que a SHELL publica** — sem isto ela emite
                // zero linhas e a tabela mede o vazio (foi o controlo que o apanhou).
                let ph = f64::from(u32::try_from(*t).unwrap_or(0)) / 60.0;
                crate::motion_shape_gen::publish(m, ph);
                m.pump.mark_dirty();
                let ok = m
                    .pump
                    .pump(&m.doc.graph, &m.registry, &[saida], *t, ph, uv, tam);
                *t += 1;
                ok
            };
            for _ in 0..8 {
                marcha(&mut m, &mut tique);
            }
            let mut melhor = f64::INFINITY;
            for _ in 0..12 {
                let t = Instant::now();
                assert!(marcha(&mut m, &mut tique), "o quadro tem de cozinhar");
                melhor = melhor.min(t.elapsed().as_secs_f64() * 1e3);
            }
            // ⭐ **O CONTROLO, que faltava:** um quadro que não desenha nada é rápido por não
            // fazer nada. Sem este piso a tabela media o vazio — 0,003 ms para 500 boids.
            eprintln!(
                // ⚠️ **As DUAS listas**: uma linha com geometria desce para `vector_instances` e
                // uma de ladrilho para `instances` (a lei do `geometry_id`). Contar só a primeira
                // lia ZERO sobre uma cena de formas — e foi isso que a 1.ª redacção fez.
                "      [controlo] n={n} forma={com_forma} colisao={com_colisao} -> {} quad + {} vector",
                m.pump.instances.len(),
                m.pump.vector_instances.len()
            );
            col[i] = melhor;
        }
        eprintln!(
            "  {n:<8} │ {:>8.3} ms │ {:>10.3} ms │ {:>8.3} ms │ {:>8.0}% │ {:>7.1}",
            col[0],
            col[1],
            col[2],
            col[2] / 16.67 * 100.0,
            1000.0 / col[2].max(1e-9)
        );
    }
    eprintln!(
        "\n  load: {}\n",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .split_whitespace()
            .next()
            .unwrap_or("?")
    );
}

/// ⭐⭐⭐ **O QUADRO DA FOTO — 189 discos encostados, `Collide Sweeps = 1024`, gizmo LIGADO.**
///
/// ⛔ A sonda irmã mede o passe; esta mede o QUADRO pela porta do produto, que é onde uma repetição
/// escondida apareceria. ⚠️ A tomada é armada **como o gizmo do colisor a arma** (o próprio sink).
#[test]
#[ignore = "sonda de medição, não gate"]
fn sonda_o_quadro_da_foto() {
    use std::time::Instant;
    eprintln!("\n  ═══ O QUADRO DA FOTO (189 discos encostados) ═══\n");
    eprintln!(
        "  {:<9} │ {:<9} │ {:<8} │ {:<12} │ {:>11} │ {:>9} │ {:>7}",
        "objectos", "varreduras", "gizmo", "quadro", "relógio", "% quadro", "FPS"
    );
    eprintln!(
        "  ----------|-----------|----------|--------------|-------------|-----------|--------"
    );
    // ⭐ A última coluna é o que o report do dono de facto mede: um quadro ATRASADO recupera
    // vários tiques de simulação, e só o último é desenhado.
    for (n, varreduras, com_gizmo, tiques, so_o_ultimo) in [
        (189usize, 1024.0f32, true, 8usize, false),
        (189, 1024.0, true, 8, true),
        // ⭐ O ponto do dono (*«roda bem com Sweeps 64, 500 objetos = 100 FPS»*) e o que ele pede a
        // seguir: os MIL.
        (500, 64.0, true, 1, true),
        (1000, 64.0, true, 1, true),
        (2000, 64.0, true, 1, true),
        (1000, 1024.0, true, 1, true),
    ] {
        let mut m = MotionState::new();
        let quadrado = indice_do_quadrado(&m.registry);
        let g = &mut m.doc.graph;
        // Os PONTOS: uma grelha apertada, como a da foto (discos a tocar-se).
        let grelha = g.add_node("motion.grid");
        #[expect(clippy::cast_precision_loss, reason = "uma contagem de cena")]
        let cols = (n as f32).sqrt().ceil();
        g.set_param(grelha, "rows", cols);
        g.set_param(grelha, "cols", cols);
        g.set_param(grelha, "gap_x", 0.17);
        g.set_param(grelha, "gap_y", 0.17);
        let forma = g.add_node("source.shape");
        if let Some(q) = quadrado {
            g.set_param(forma, ph2d_node_motion_shape::param::KIND, q);
        }
        g.set_param(forma, ph2d_node_motion_shape::param::SIZE, 0.09);
        g.set_param(forma, ph2d_node_motion_shape::param::COLLIDE, 1.0);
        let dup = g.add_node("motion.duplicator");
        let saida = g.add_node("motion.output");
        g.set_param(saida, ph2d_eval_motion::SINK_COLLIDE_PARAM, 1.0);
        g.set_param(
            saida,
            ph2d_eval_motion::SINK_COLLIDE_ITERATIONS_PARAM,
            varreduras,
        );
        for (a, ap, b, bp) in [
            (forma, 0u16, dup, 0u16),
            (grelha, 0, dup, 1),
            (dup, 0, saida, 0),
        ] {
            g.connect(ph2d_nodegraph::graph::Edge {
                from: (a, ap),
                to: (b, bp),
                delayed: false,
            })
            .expect("aresta");
        }
        // ⚠️ **A tomada do gizmo do colisor é o PRÓPRIO sink** (`collider_gizmo::taps_for`).
        if com_gizmo {
            m.pump.set_taps(&[forma, saida]);
        }
        let (uv, tam) = ([0.0, 0.0, 1.0, 1.0], [1.0, 1.0]);
        // Um QUADRO = `tiques` cozimentos, e só o último é desenhado (a lei do `ticks_owed`).
        let quadro = |m: &mut MotionState, t: &mut u64| {
            for k in 0..tiques {
                let ph = f64::from(u32::try_from(*t).unwrap_or(0)) / 60.0;
                crate::motion_shape_gen::publish(m, ph);
                m.pump.mark_dirty();
                m.pump.set_separa_o_desenho(!so_o_ultimo || k + 1 == tiques);
                assert!(
                    m.pump
                        .pump(&m.doc.graph, &m.registry, &[saida], *t, ph, uv, tam),
                    "o quadro tem de cozinhar"
                );
                *t += 1;
            }
        };
        let mut tique = 0u64;
        for _ in 0..2 {
            quadro(&mut m, &mut tique);
        }
        let mut melhor = f64::INFINITY;
        for _ in 0..4 {
            let t = Instant::now();
            quadro(&mut m, &mut tique);
            melhor = melhor.min(t.elapsed().as_secs_f64() * 1e3);
        }
        eprintln!(
            "  {n:<9} │ {varreduras:<9.0} │ {:<8} │ {tiques:>2} tiq{} │ {melhor:>8.1} ms │ {:>8.0}% │ {:>7.1}   [{} vector]",
            if com_gizmo { "ligado" } else { "—" },
            if so_o_ultimo { ", 1 sep" } else { ", N sep" },
            melhor / 16.67 * 100.0,
            1000.0 / melhor.max(1e-9),
            m.pump.vector_instances.len()
        );
    }
    eprintln!(
        "\n  load: {}\n",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .split_whitespace()
            .next()
            .unwrap_or("?")
    );
}

/// ⭐⭐⭐ **O DESENHO DE N FORMAS, do lado da CPU** — o report de 2026-09-18: *«1000 = 40 fps.
/// Retirar o contorno azul não melhorou em nada»*.
///
/// O quadro dele mede `25 ms` e o Motion é `6,8`. ⚠️ **Esta sonda mede o que sobra do lado da
/// CPU**: construir a cena do Vello com as `N` formas. O que ela NÃO mede é a rasterização — e é
/// exactamente por isso que ela vale: *se o encode for barato, o que sobra é a PLACA*, e aí o que
/// conta não é o número de formas, é quantos PIXEIS elas cobrem.
#[test]
#[ignore = "sonda de medição, não gate"]
fn quanto_custa_desenhar_as_formas() {
    use std::time::Instant;
    eprintln!("\n  ═══ O ENCODE DAS FORMAS (CPU, sem placa) ═══\n");
    eprintln!(
        "  {:<9} │ {:>12} │ {:>13} │ {:>16}",
        "formas", "encode", "por forma", "% de um quadro"
    );
    eprintln!("  ----------|--------------|---------------|------------------");
    for n in [500usize, 1000, 2000] {
        let mut m = MotionState::new();
        let quadrado = indice_do_quadrado(&m.registry);
        let g = &mut m.doc.graph;
        let grelha = g.add_node("motion.grid");
        #[expect(clippy::cast_precision_loss, reason = "uma contagem de cena")]
        let cols = (n as f32).sqrt().ceil();
        g.set_param(grelha, "rows", cols);
        g.set_param(grelha, "cols", cols);
        g.set_param(grelha, "gap_x", 0.17);
        g.set_param(grelha, "gap_y", 0.17);
        let forma = g.add_node("source.shape");
        if let Some(q) = quadrado {
            g.set_param(forma, ph2d_node_motion_shape::param::KIND, q);
        }
        g.set_param(forma, ph2d_node_motion_shape::param::SIZE, 0.09);
        let dup = g.add_node("motion.duplicator");
        let saida = g.add_node("motion.output");
        for (a, ap, b, bp) in [
            (forma, 0u16, dup, 0u16),
            (grelha, 0, dup, 1),
            (dup, 0, saida, 0),
        ] {
            g.connect(ph2d_nodegraph::graph::Edge {
                from: (a, ap),
                to: (b, bp),
                delayed: false,
            })
            .expect("aresta");
        }
        crate::motion_shape_gen::publish(&mut m, 0.0);
        m.pump.mark_dirty();
        assert!(
            m.pump.pump(
                &m.doc.graph,
                &m.registry,
                &[saida],
                0,
                0.0,
                [0.0, 0.0, 1.0, 1.0],
                [1.0, 1.0]
            ),
            "a cena tem de cozinhar"
        );
        let insts = &m.pump.vector_instances;
        assert!(insts.len() >= n, "controlo: {} formas de {n}", insts.len());
        let mut melhor = f64::INFINITY;
        for _ in 0..10 {
            let mut cena = ph2d_vector::VectorScene::new();
            let mut sem_arte = |_: u32, _: [f32; 4]| None;
            let t = Instant::now();
            crate::motion_shape_gen::encode(
                insts,
                &m.shape_store,
                &mut sem_arte,
                ph2d_vector::Affine::IDENTITY,
                &mut cena,
            );
            melhor = melhor.min(t.elapsed().as_secs_f64() * 1e3);
            std::hint::black_box(&cena);
        }
        #[expect(clippy::cast_precision_loss, reason = "uma contagem de formas")]
        let nf = insts.len() as f64;
        eprintln!(
            "  {:<9} │ {melhor:>9.3} ms │ {:>10.2} µs │ {:>15.1}%",
            insts.len(),
            melhor * 1e3 / nf,
            melhor / 16.67 * 100.0
        );
    }
    eprintln!(
        "\n  ⚠️ Isto é SO' o encode. A rasterizacao e' da placa, e o que a governa nao e' o numero\n  \
         de formas — e' quantos PIXEIS elas cobrem.\n"
    );
}
