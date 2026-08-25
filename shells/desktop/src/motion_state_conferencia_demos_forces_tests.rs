//! Gates da cena `=95` — o que uma força não sabia dizer (folha 02).

use super::*;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;

const TICKS: usize = 150;
const DT: f64 = 1.0 / 60.0;

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("os nos registram");
    reg
}

fn scene() -> (MotionDoc, NodeRegistry, Vec<NodeId>) {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_forces_demo_document(&mut doc, &reg).expect("a cena monta");
    doc.graph.validate(&reg).expect("bem-tipada");
    (doc, reg, sinks)
}

/// Corre a cena e devolve a última pose de cada sink pedido.
fn settle(doc: &MotionDoc, reg: &NodeRegistry, sinks: &[NodeId]) -> Vec<Vec<[f32; 2]>> {
    let mut cook = Cook::new();
    let mut last = vec![Vec::new(); sinks.len()];
    for k in 0..TICKS {
        let t = k as f64 * DT;
        cook.advance_tick(&doc.graph, reg, t).expect("avanca");
        for (i, &s) in sinks.iter().enumerate() {
            let out = cook.cook(&doc.graph, reg, s, t).expect("coze");
            if let Some(Column::Vec2(p)) = out[0].as_stream().get("P") {
                last[i] = p.clone();
            }
        }
    }
    last
}

/// A distância do ponto mais afastado ao centroide.
fn spread(p: &[[f32; 2]]) -> f32 {
    if p.is_empty() {
        return 0.0;
    }
    let c = p
        .iter()
        .fold([0.0_f32; 2], |a, q| [a[0] + q[0], a[1] + q[1]])
        .map(|v| v / p.len() as f32);
    p.iter()
        .map(|q| (q[0] - c[0]).hypot(q[1] - c[1]))
        .fold(0.0_f32, f32::max)
}

/// **A CENA MONTA AS OITO BANDAS**, e as oito cospem sem explodir.
#[test]
fn the_forces_scene_builds_all_eight_bands() {
    let (doc, reg, sinks) = scene();
    assert_eq!(sinks.len(), 8, "quatro pares");
    assert_eq!(band_labels().count(), 8, "um rotulo por banda");
    let poses = settle(&doc, &reg, &sinks);
    for (k, p) in poses.iter().enumerate() {
        assert!(!p.is_empty(), "banda {k} vazia");
        for q in p {
            assert!(q[0].is_finite() && q[1].is_finite(), "banda {k} explodiu");
        }
    }
}

/// SONDA — imprime o que cada par de facto faz, para as barras saírem de medição.
#[test]
#[ignore = "sonda de medicao, nao gate"]
fn measure_the_force_pairs() {
    let (doc, reg, sinks) = scene();
    let poses = settle(&doc, &reg, &sinks);
    // ⚠️ **Ao CENTROIDE, não à origem do mundo** — o `finish` desloca cada banda para o
    // quadrante dela, e uma régua ancorada na origem media o deslocamento do quadrante.
    let closest = |p: &[[f32; 2]]| {
        let c = p
            .iter()
            .fold([0.0_f32; 2], |a, q| [a[0] + q[0], a[1] + q[1]])
            .map(|v| v / p.len() as f32);
        p.iter()
            .map(|q| (q[0] - c[0]).hypot(q[1] - c[1]))
            .fold(f32::MAX, f32::min)
    };
    let far = |p: &[[f32; 2]]| p.iter().map(|q| q[0].hypot(q[1])).fold(0.0_f32, f32::max);
    for (k, p) in poses.iter().enumerate() {
        println!(
            "banda {k}: dispersao {:.4} · mais perto do centro {:.4} · mais longe {:.4}",
            spread(p),
            closest(p),
            far(p)
        );
    }
    let mut cook = Cook::new();
    for k in 0..TICKS {
        let t = k as f64 * DT;
        cook.advance_tick(&doc.graph, &reg, t).expect("avanca");
        for &s in &sinks {
            let _ = cook.cook(&doc.graph, &reg, s, t);
        }
        if k == 60 || k == TICKS - 1 {
            for (j, &s) in sinks.iter().enumerate().skip(2).take(4) {
                let out = cook.cook(&doc.graph, &reg, s, t).expect("coze");
                if let Some(Column::Vec2(v)) = out[0].as_stream().get("vel") {
                    let hi = v.iter().map(|q| q[0].hypot(q[1])).fold(0.0_f32, f32::max);
                    println!("  tique {k} banda {j}: maior velocidade {hi:.4}");
                }
            }
        }
    }
}

/// **QUÃO FUNDO cada boia está**, em decis `[p10, mediana, p90]`.
///
/// ⚠️ **A pose chega em coordenadas de MUNDO e a simulação correu em LOCAIS** — o `finish`
/// desloca a banda para o quadrante dela depois de tudo. Comparar a pose com a superfície
/// sem desfazer esse deslocamento mediria o quadrante, e não a água.
fn submersions(p: &[[f32; 2]], band: usize, t: f32) -> [f32; 3] {
    let at = band_at(band);
    let (amp, lambda, speed, _draft, _sub) = sea_authored();
    // A esquerda é a senoide única; a direita é o espectro.
    let waves = if band.is_multiple_of(2) {
        1.0
    } else {
        authored().2
    };
    let mut d: Vec<f32> = p
        .iter()
        .map(|q| {
            let (x, y) = (q[0] - at[0], q[1] - at[1]);
            ph2d_node_force_buoyancy::surface_at(x, t, 0.0, amp, lambda, speed, waves) - y
        })
        .collect();
    d.sort_by(f32::total_cmp);
    let pick = |f: f32| d[((d.len() - 1) as f32 * f) as usize];
    [pick(0.1), pick(0.5), pick(0.9)]
}

/// SONDA — o **mar**: a média de `y` de cada banda ao longo do tempo.
///
/// ⚠️ **A régua é a DERIVA da média, e não a dispersão.** Uma nuvem que assenta numa
/// superfície tem média estável; uma que foi LANÇADA tem média a subir para sempre, e as
/// duas podem ter a mesma dispersão.
#[test]
#[ignore = "sonda de medicao, nao gate"]
fn measure_the_sea() {
    let (doc, reg, sinks) = scene();
    let mut cook = Cook::new();
    let mut trace: Vec<Vec<f32>> = vec![Vec::new(); 2];
    // ⚠️ Mais longo que o gate de propósito: o que se procura aqui é ONDE assenta, e o
    // transiente do mergulho inicial dura mais que a janela que os gates medem.
    const LONG: usize = 900;
    for k in 0..LONG {
        let t = k as f64 * DT;
        cook.advance_tick(&doc.graph, &reg, t).expect("avanca");
        for &s in &sinks {
            let _ = cook.cook(&doc.graph, &reg, s, t);
        }
        if k % 75 == 0 || k == LONG - 1 {
            for (j, &s) in sinks.iter().enumerate().skip(6) {
                let out = cook.cook(&doc.graph, &reg, s, t).expect("coze");
                if let Some(Column::Vec2(p)) = out[0].as_stream().get("P") {
                    let mean = p.iter().map(|q| q[1]).sum::<f32>() / p.len() as f32;
                    let wx = p.iter().map(|q| q[0]).fold(f32::MIN, f32::max)
                        - p.iter().map(|q| q[0]).fold(f32::MAX, f32::min);
                    let d = submersions(p, j, t as f32);
                    println!(
                        "tique {k:3} banda {j}: media y {mean:8.4} · largura x {wx:6.3} · \
                         submersao mediana {:7.4} (p10 {:7.4} · p90 {:7.4})",
                        d[1], d[0], d[2]
                    );
                    trace[j - 6].push(mean);
                }
            }
        }
    }
    for (j, tr) in trace.iter().enumerate() {
        let drift = tr.windows(2).map(|w| w[1] - w[0]).fold(0.0_f32, f32::max);
        println!("banda {}: MAIOR subida entre amostras {drift:.4}", j + 6);
    }
    // ⭐ **O que cada boia FAZ**, no regime já assentado: quanto ela sobe e desce, e quanto
    // ela anda de lado. É isto que decide se o mar se VÊ a mexer.
    let mut track: Vec<Vec<Vec<[f32; 2]>>> = vec![Vec::new(); 2];
    for k in LONG..LONG + 300 {
        let t = k as f64 * DT;
        cook.advance_tick(&doc.graph, &reg, t).expect("avanca");
        for (j, &s) in sinks.iter().enumerate() {
            let o = cook.cook(&doc.graph, &reg, s, t).expect("coze");
            if j >= 6
                && k % 5 == 0
                && let Some(Column::Vec2(p)) = o[0].as_stream().get("P")
            {
                track[j - 6].push(p.clone());
            }
        }
    }
    for (j, frames) in track.iter().enumerate() {
        let n = frames[0].len();
        let span = |axis: usize| {
            let mut v: Vec<f32> = (0..n)
                .map(|i| {
                    let lo = frames.iter().map(|f| f[i][axis]).fold(f32::MAX, f32::min);
                    let hi = frames.iter().map(|f| f[i][axis]).fold(f32::MIN, f32::max);
                    hi - lo
                })
                .collect();
            v.sort_by(f32::total_cmp);
            (v[n / 2], v[n - 1])
        };
        let (my, xy) = span(1);
        let (mx, xx) = span(0);
        println!(
            "banda {}: excursao VERTICAL mediana {my:.4} (max {xy:.4}) · HORIZONTAL mediana {mx:.4} (max {xx:.4})",
            j + 6
        );
    }
}

/// Uma banda de mar sozinha, com os números que se quiserem — o arnês da varredura.
///
/// Devolve `(excursão vertical mediana, excursão horizontal mediana, submersão mediana)` no
/// regime já assentado.
fn one_sea(density: f32, grav: f32, drag: f32, speed: f32, waves: f32) -> (f32, f32, f32) {
    let reg = registry();
    let mut g = Graph::new();
    let (_, lambda, _, draft, _) = sea_authored();
    let amp = lambda * 0.1;
    let src = g.add_node("motion.grid");
    g.set_param(src, "rows", 2.0);
    g.set_param(src, "cols", 128.0);
    g.set_param(src, "gap_x", 7.0 / 127.0);
    g.set_param(src, "gap_y", 0.3);
    let up = g.add_node("motion.move");
    g.set_param(up, "dy", 0.6);
    let integ = g.add_node("motion.integrate");
    let w = g.add_node("force.wind");
    g.set_param(w, "angle", 270.0);
    g.set_param(w, "strength", grav);
    g.set_param(w, "gust", 0.0);
    let b = g.add_node("force.buoyancy");
    g.set_param(b, "level", 0.0);
    g.set_param(b, "density", density);
    g.set_param(b, "depth", draft);
    g.set_param(b, "drag", drag);
    g.set_param(b, "wave_amplitude", amp);
    g.set_param(b, "wave_length", lambda);
    g.set_param(b, "wave_speed", speed);
    g.set_param(b, ph2d_node_force_buoyancy::WAVES, waves);
    for (from, to, port, delayed) in [
        (src, up, 0, false),
        (up, integ, 0, false),
        (integ, w, 0, true),
        (w, b, 0, false),
        (b, integ, 1, false),
    ] {
        g.connect(ph2d_nodegraph::graph::Edge {
            from: (from, 0),
            to: (to, port),
            delayed,
        })
        .expect("liga");
    }
    g.validate(&reg).expect("bem-tipada");

    let mut cook = Cook::new();
    let mut frames: Vec<Vec<[f32; 2]>> = Vec::new();
    let mut last = Vec::new();
    for k in 0..1200 {
        let t = f64::from(k) / 60.0;
        cook.advance_tick(&g, &reg, t).expect("avanca");
        let o = cook.cook(&g, &reg, integ, t).expect("coze");
        if let Some(Column::Vec2(p)) = o[0].as_stream().get("P") {
            if k >= 900 && k % 5 == 0 {
                frames.push(p.clone());
            }
            last = p.clone();
        }
    }
    let n = frames[0].len();
    let median_span = |axis: usize| {
        let mut v: Vec<f32> = (0..n)
            .map(|i| {
                let lo = frames.iter().map(|f| f[i][axis]).fold(f32::MAX, f32::min);
                let hi = frames.iter().map(|f| f[i][axis]).fold(f32::MIN, f32::max);
                hi - lo
            })
            .collect();
        v.sort_by(f32::total_cmp);
        v[n / 2]
    };
    let t = 1199.0_f32 / 60.0;
    let mut d: Vec<f32> = last
        .iter()
        .map(|q| {
            ph2d_node_force_buoyancy::surface_at(q[0], t, 0.0, amp, lambda, speed, waves) - q[1]
        })
        .collect();
    d.sort_by(f32::total_cmp);
    // ⚠️ **A excursão horizontal não distingue ORBITAR de PARTIR.** Uma boia que vai e vem
    // meia onda tem a mesma excursão de uma que anda meia onda e nunca volta; o que separa
    // as duas é a deriva LÍQUIDA da banda.
    let mean_x = |f: &Vec<[f32; 2]>| f.iter().map(|q| q[0]).sum::<f32>() / f.len() as f32;
    let net = mean_x(frames.last().expect("frames")) - mean_x(&frames[0]);
    (median_span(1), net, d[d.len() / 2])
}

/// SONDA — **a boia ENCAIXA na cava?** A varredura que escolhe o arrasto do mar.
///
/// ⚠️ **A lei da armadilha:** a boia escorrega para a cava até o empurrão em declive igualar
/// o arrasto. Ela ENCAIXA se existir um declive onde isso acontece à velocidade da onda, ou
/// seja se `densidade · declive_máximo ≥ arrasto · velocidade`. Encaixada, a excursão
/// vertical dela é ZERO e a horizontal é a onda inteira — que foi o que se mediu na 1.ª
/// versão (`0,0056` contra `4,92`).
///
/// ⚠️ **E o espectro multiplica o declive pelo número de camadas** (cada oitava tem metade da
/// amplitude e metade do comprimento ⇒ o MESMO declive), então a fileira de 4 ondas precisa
/// de ~4× o arrasto da de 1 — e é ela que manda.
#[test]
#[ignore = "sonda de medicao, nao gate"]
fn measure_the_trapping_sweep() {
    // A altura da vaga, que é o que a excursão vertical tem de reproduzir.
    let (_, lambda, ..) = sea_authored();
    let height = 2.0 * lambda * 0.1;
    println!("altura da vaga = {height:.4}");
    println!(
        "densidade grav arrasto ondas | limiar | vertical (x altura) deriva_liquida submersao"
    );
    for (dens, grav) in [(12.0_f32, 4.0_f32), (6.0, 2.0)] {
        for drag in [6.0_f32, 11.0, 12.8, 16.7, 20.0] {
            for waves in [1.0_f32, 4.0] {
                // O limiar da armadilha, pela lei: `densidade · declive_max · inv_len / vel`.
                let slope = waves * std::f32::consts::TAU * 0.1;
                let bar = dens * slope / (1.0 + slope * slope).sqrt();
                let (v, net, s) = one_sea(dens, grav, drag, 1.0, waves);
                println!(
                    "{dens:8.1} {grav:4.1} {drag:7.1} {waves:5.1} | {bar:6.2} | {v:8.4} ({:4.2}) {net:13.4} {s:10.4}",
                    v / height
                );
            }
        }
    }
}

/// A distância do ponto MAIS PERTO do centroide — o quão vazio está o miolo.
fn hollow(p: &[[f32; 2]]) -> f32 {
    if p.is_empty() {
        return 0.0;
    }
    let c = p
        .iter()
        .fold([0.0_f32; 2], |a, q| [a[0] + q[0], a[1] + q[1]])
        .map(|v| v / p.len() as f32);
    p.iter()
        .map(|q| (q[0] - c[0]).hypot(q[1] - c[1]))
        .fold(f32::MAX, f32::min)
}

/// A maior velocidade da banda, em dois instantes.
fn speeds(doc: &MotionDoc, reg: &NodeRegistry, sinks: &[NodeId], at: &[usize]) -> Vec<Vec<f32>> {
    let mut cook = Cook::new();
    let mut out = vec![Vec::new(); sinks.len()];
    for k in 0..TICKS {
        let t = k as f64 * DT;
        cook.advance_tick(&doc.graph, reg, t).expect("avanca");
        for (i, &s) in sinks.iter().enumerate() {
            let o = cook.cook(&doc.graph, reg, s, t).expect("coze");
            if at.contains(&k)
                && let Some(Column::Vec2(v)) = o[0].as_stream().get("vel")
            {
                out[i].push(v.iter().map(|q| q[0].hypot(q[1])).fold(0.0_f32, f32::max));
            }
        }
    }
    out
}

/// ⭐⭐ **O PAR 1: a rampa COLAPSA num ponto e o perfil assenta num ANEL.**
///
/// ⚠️ **As duas réguas são ancoradas no CENTROIDE**, e não na origem do mundo: o `finish`
/// desloca cada banda para o quadrante dela, e uma régua da origem mediria o deslocamento.
///
/// ⚠️ **E a fixture teve de ser corrigida duas vezes antes de medir alguma coisa**, o que é
/// o registo mais útil aqui: (a) sem ARRASTO nada assenta — a nuvem atravessa o centro e
/// sai —, e (b) com a nuvem MAIOR que o raio de influência os cantos ficam fora da força e
/// o que se media era o que ela não alcança. *Um par ANTES/DEPOIS mede a diferença entre
/// dois estados de equilíbrio; sem equilíbrio ele mede o transiente.*
///
/// Medido: rampa `0,3315` de dispersão com o miolo a `0,0011`; perfil `1,3535` com o miolo
/// a `0,3266`.
#[test]
fn the_ramp_collapses_to_a_point_where_the_profile_settles_on_a_ring() {
    let (doc, reg, sinks) = scene();
    let poses = settle(&doc, &reg, &sinks[..2]);
    let (ramp, profiled) = (spread(&poses[0]), spread(&poses[1]));
    assert!(
        profiled > ramp * 3.0,
        "o perfil tinha de deixar um anel onde a rampa colapsa: {ramp:.4} contra {profiled:.4}"
    );
    // E o MIOLO do anel está vazio — é o que a inversão compra, e é o que uma dispersão
    // maior sozinha não distingue de uma nuvem que simplesmente não colapsou.
    let (core_r, core_p) = (hollow(&poses[0]), hollow(&poses[1]));
    assert!(
        core_r < 0.05,
        "CONTROLE: a rampa junta tudo, entao o miolo dela e' cheio ({core_r:.4})"
    );
    assert!(
        core_p > 0.15,
        "o miolo do anel tinha de ficar VAZIO: {core_p:.4}"
    );
}

/// ⭐⭐⭐ **OS PARES 2 e 3: o modo alvo SATURA onde a aceleração constante não para.**
///
/// ⚠️ **A régua é a VELOCIDADE, e não a distância percorrida** — que foi a primeira que
/// escrevi e estava errada: com uma resistência alta o modo alvo chega depressa e pode
/// **andar mais** nos primeiros segundos. A afirmação é sobre a derivada, e é ela que se
/// mede: entre o tique 60 e o 149, o modo `Force` sobe e o modo alvo fica onde está.
///
/// Medido no vento: `2,0000 → 4,9667` contra `1,9079 → 1,9990` — e o `1,999` **é** a
/// `strength` do vento, que é exactamente o que «saturar» quer dizer.
#[test]
fn the_target_velocity_saturates_where_the_constant_force_keeps_accelerating() {
    let (doc, reg, sinks) = scene();
    let v = speeds(&doc, &reg, &sinks[2..6], &[60, TICKS - 1]);
    // ⚠️ **A subida vale só para o VENTO, e a razão é GEOMÉTRICA.** Um vento não tem
    // extensão: uma aceleração constante ali acelera para sempre (`2,0000 → 4,9667`). Um
    // vórtice TEM raio, então quem sai dele deixa de receber força — a versão `Force` dele
    // também estabiliza (`4,6723 → 4,9472`, 6%), mas **por ter fugido do campo** e não por
    // uma lei. *Duas saturações que se parecem e têm causas diferentes: exigir a mesma
    // subida das duas seria medir a geometria da fixture, não o modo.* No vórtice a
    // afirmação que se pode fazer é a de baixo — o modo alvo fica bem ABAIXO do constante.
    let (wf0, wf1) = (v[0][0], v[0][1]);
    assert!(
        wf1 > wf0 * 1.2,
        "o vento constante tinha de CRESCER ({wf0:.4} -> {wf1:.4})"
    );
    assert!(
        v[3][1] < v[2][1] * 0.5,
        "vortice: o modo alvo tinha de ficar bem abaixo do constante ({:.4} contra {:.4})",
        v[2][1],
        v[3][1]
    );
    // E a saturação, que vale para os dois.
    for (k, name) in [(0_usize, "vento"), (2, "vortice")] {
        let (t0, t1) = (v[k + 1][0], v[k + 1][1]);
        assert!(
            t1 <= t0 * 1.05,
            "{name}: o modo alvo tinha de SATURAR ({t0:.4} -> {t1:.4})"
        );
        assert!(
            t1 > 0.1,
            "{name}: e ele tem de andar alguma coisa ({t1:.4})"
        );
    }
}

/// Quantos tiques o mar precisa para ASSENTAR — medido, não escolhido: o mergulho inicial
/// ainda domina no tique 150 (submersão mediana `0,29`), e a partir de ~375 a mediana já
/// está a 5% do valor de equilíbrio. `600` dá margem sem esticar o relógio da suíte.
const SEA_TICKS: usize = 600;

/// Corre só as duas bandas do mar, e devolve a pose delas em dois instantes.
fn sea_poses(
    doc: &MotionDoc,
    reg: &NodeRegistry,
    sinks: &[NodeId],
    at: &[usize],
) -> Vec<Vec<Vec<[f32; 2]>>> {
    let mut cook = Cook::new();
    let mut out = vec![Vec::new(); sinks.len()];
    for k in 0..SEA_TICKS {
        let t = k as f64 * DT;
        cook.advance_tick(&doc.graph, reg, t).expect("avanca");
        for (i, &s) in sinks.iter().enumerate() {
            let o = cook.cook(&doc.graph, reg, s, t).expect("coze");
            if at.contains(&k)
                && let Some(Column::Vec2(p)) = o[0].as_stream().get("P")
            {
                out[i].push(p.clone());
            }
        }
    }
    out
}

/// O que cada boia FAZ no regime já assentado, por banda: `(excursão vertical mediana,
/// deriva LÍQUIDA da banda)`.
///
/// ⚠️ **As duas juntas, e não uma delas.** A excursão horizontal sozinha não distingue
/// ORBITAR de PARTIR — uma boia que vai e vem meia onda mede o mesmo que uma que anda meia
/// onda e nunca volta.
fn sea_motion(doc: &MotionDoc, reg: &NodeRegistry, sinks: &[NodeId]) -> Vec<(f32, f32)> {
    let mut cook = Cook::new();
    let mut frames: Vec<Vec<Vec<[f32; 2]>>> = vec![Vec::new(); sinks.len()];
    for k in 0..SEA_TICKS + 300 {
        let t = k as f64 * DT;
        cook.advance_tick(&doc.graph, reg, t).expect("avanca");
        for (i, &s) in sinks.iter().enumerate() {
            let o = cook.cook(&doc.graph, reg, s, t).expect("coze");
            if k >= SEA_TICKS
                && k % 5 == 0
                && let Some(Column::Vec2(p)) = o[0].as_stream().get("P")
            {
                frames[i].push(p.clone());
            }
        }
    }
    frames
        .iter()
        .map(|f| {
            let n = f[0].len();
            let mut v: Vec<f32> = (0..n)
                .map(|i| {
                    let lo = f.iter().map(|q| q[i][1]).fold(f32::MAX, f32::min);
                    let hi = f.iter().map(|q| q[i][1]).fold(f32::MIN, f32::max);
                    hi - lo
                })
                .collect();
            v.sort_by(f32::total_cmp);
            let mean_x = |q: &Vec<[f32; 2]>| q.iter().map(|p| p[0]).sum::<f32>() / q.len() as f32;
            let net = mean_x(f.last().expect("frames")) - mean_x(&f[0]);
            (v[n / 2], net)
        })
        .collect()
}

/// ⭐⭐⭐ **O PAR 4 — O GATE QUE FALTAVA: as boias CAVALGAM a vaga, não são LEVADAS por ela.**
///
/// ⛔ **É o gate que teria apanhado o defeito que o Enio viu — nas DUAS formas em que ele
/// apareceu.** As duas leem-se igual no ecrã (*«não parece mar, parece partículas ao
/// vento»*) porque têm a mesma assinatura: as peças atravessam a banda **sem nunca subirem
/// nem descerem**.
///
/// | | causa | excursão vertical | deriva em 5 s |
/// |---|---|---|---|
/// | 1.ª versão | **sem gravidade** no grafo: o empuxo lança tudo | — | a média de `y` subia `0,58` por 25 tiques, sem abrandar |
/// | 2.ª versão | **armadilha de cava**: a boia encaixa e viaja com a onda | `0,0056` | `4,92` — `0,98` da velocidade da onda |
/// | hoje | — | `0,38`, que é `0,82` da altura da vaga | `0,067` |
///
/// ⚠️ **Nenhuma das réguas anteriores as via.** Dispersão e distância entre as duas bandas
/// são grandezas que um mar e uma nuvem lançada PARTILHAM; e uma régua só de `y` não vê a
/// segunda, que é horizontal. *Uma cena que mostra uma superfície tem de afirmar as duas
/// coisas que fazem dela uma superfície: que ela SOBE E DESCE, e que ela FICA.*
#[test]
fn the_floats_ride_the_wave_instead_of_being_carried_by_it() {
    let (doc, reg, sinks) = scene();
    let (amp, ..) = sea_authored();
    let height = 2.0 * amp;
    for (i, (bob, net)) in sea_motion(&doc, &reg, &sinks[6..8]).into_iter().enumerate() {
        assert!(
            bob > height * 0.3,
            "banda {}: a boia mediana sobe e desce {bob:.4} de uma vaga de {height:.4} -- \
             este mar esta' PRESO, e preso desliza de lado como uma linha rigida",
            6 + i
        );
        assert!(
            net.abs() < 0.3,
            "banda {}: a banda andou {net:.4} de lado em 5 s -- ela esta' a ser LEVADA",
            6 + i
        );
    }
}

/// ⭐⭐ **O ARRASTO LIMPA O LIMIAR DA ARMADILHA** — aritmética pura sobre os autorados.
///
/// ⚠️ **É a lei, e não a medição.** O gate acima mede o SINTOMA (a banda ficou?); este afirma
/// o MECANISMO, e por isso dispara na cara de quem baixar o arrasto, subir a esbelteza,
/// subir a densidade ou acrescentar camadas — quatro maneiras de reabrir a armadilha, e três
/// delas não parecem ter nada a ver com ela.
#[test]
fn the_drag_clears_the_trapping_threshold() {
    let bar = sea_trap_threshold(authored().2);
    let drag = sea_drag();
    assert!(
        drag > bar,
        "arrasto {drag:.3} contra um limiar de armadilha de {bar:.3}"
    );
    // ⚠️ E o limiar CRESCE com as camadas: a fileira de 4 ondas é a exigente, e é ela que
    // tem de mandar no número. Uma margem medida contra a de 1 onda deixaria a outra presa.
    assert!(
        sea_trap_threshold(4.0) > sea_trap_threshold(1.0),
        "o espectro tinha de tornar a armadilha MAIS facil, nao menos"
    );
}

/// ⭐ **E as boias estão NA ÁGUA** — nem a voar por cima, nem afundadas.
///
/// ⛔ **A afirmação que eu tinha aqui era mais forte e estava ERRADA.** Ela dizia que a
/// mediana da submersão bate o equilíbrio estático `(gravidade/densidade) · calado`, e batia
/// — a `0,8%`. Só que ela **só batia porque as boias estavam PRESAS**: um corpo encaixado na
/// cava não se mexe, logo assenta no equilíbrio estático. Assim que ele passa a cavalgar a
/// vaga ele é FORÇADO, e o ponto dele deixa de ser o estático (medido: `0,29` contra os
/// `0,167` da conta). *Um gate que só passa quando a cena está morta é um gate a favor da
/// cena morta* — e este passou por cima do defeito que o Enio viu.
#[test]
fn the_floats_are_in_the_water() {
    let (doc, reg, sinks) = scene();
    let (_, _, _, draft, _) = sea_authored();
    let poses = sea_poses(&doc, &reg, &sinks[6..8], &[SEA_TICKS - 1]);
    let t = (SEA_TICKS - 1) as f32 * DT as f32;
    for (i, p) in poses.iter().enumerate() {
        let d = submersions(&p[0], 6 + i, t);
        // ⚠️ **A barra NÃO é «dentro de água»**, e a primeira versão dela era: reprovou com
        // `p10 = −0,0032`. Uma boia viva SALTA um pouco fora na descida da crista e é
        // COBERTA na subida da seguinte — é isso que uma rolha faz, e uma barra que o
        // proibisse estaria a pedir de volta o mar preso. O que se afirma é que ela nunca se
        // afasta da superfície mais do que meio calado, para cada lado.
        // Medido: `[−0,013 · 0,325 · 0,479]` e `[0,070 · 0,208 · 0,602]`, calado `0,5`.
        assert!(
            d[0] > -0.5 * draft,
            "banda {}: o decil de cima esta' no AR ({:.4}, calado {draft:.4})",
            6 + i,
            d[0]
        );
        assert!(
            d[2] < 1.5 * draft,
            "banda {}: o decil de baixo AFUNDOU ({:.4}, calado {draft:.4})",
            6 + i,
            d[2]
        );
        assert!(
            d[1] > 0.0 && d[1] < draft,
            "banda {}: a boia MEDIANA tinha de estar dentro de agua ({:.4})",
            6 + i,
            d[1]
        );
    }
}

/// ⭐ **AS BOIAS RESOLVEM A ONDA MAIS FINA** — Nyquist, sobre os números autorados.
///
/// ⚠️ **A onda mais fina não é escolhida por esta cena**: ela cai da razão entre camadas e do
/// tecto de camadas, que são decisões do `force.buoyancy` — daí `finest_wavelength`. Com as
/// `48` colunas da primeira versão havia **1,96 boias por período**, abaixo dos dois que
/// separam «uma onda amostrada» de «ruído».
///
/// ⛔ Este gate é aritmética pura sobre constantes, e é de propósito: ele dispara quando
/// alguém encolher a contagem de boias, alargar a banda **ou** subir o tecto de camadas —
/// as três estragam a mesma coisa, e só uma delas mora neste arquivo.
#[test]
fn the_floats_resolve_the_finest_wave() {
    let (_, lambda, _, _, _) = sea_authored();
    let finest = ph2d_node_force_buoyancy::finest_wavelength(lambda, authored().2);
    let per = finest / float_spacing();
    assert!(
        per >= 4.0,
        "so' {per:.2} boias na onda mais fina ({finest:.4}) -- ela sai como ruido"
    );
}

/// ⭐ **As duas superfícies não são a mesma**, e a diferença é a que o olho tem de apanhar.
///
/// ⚠️ **A afirmação sobre a FORMA das cristas vive no gate do crate**
/// (`the_spectrum_breaks_the_single_wavelength`, que mede a distância entre cristas
/// vizinhas). Aqui o que se afirma é que a fileira da direita **desenha outra água**.
#[test]
fn the_two_seas_are_not_the_same_sea() {
    let (doc, reg, sinks) = scene();
    let poses = sea_poses(&doc, &reg, &sinks[6..8], &[SEA_TICKS - 1]);
    assert_eq!(poses[0][0].len(), poses[1][0].len(), "a mesma contagem");
    let apart = poses[0][0]
        .iter()
        .zip(&poses[1][0])
        .map(|(a, b)| (a[1] - b[1]).abs())
        .fold(0.0_f32, f32::max);
    let (amp, ..) = sea_authored();
    assert!(
        apart > amp * 0.5,
        "as duas superficies quase coincidiram ({apart:.4} contra uma amplitude de {amp:.4})"
    );
}

/// As fichas do canvas: uma por banda, curta.
#[test]
fn every_band_carries_its_caption() {
    let caps = captions();
    assert_eq!(caps.len(), 8, "uma ficha por banda");
    for c in &caps {
        assert!(!c.text.contains("--"), "a ficha e' curta: {:?}", c.text);
        assert!(!c.text.is_empty(), "ficha vazia");
    }
}
