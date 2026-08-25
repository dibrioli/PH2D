//! **O MAR da cena `=95`** — os gates da fileira de baixo (doc 89, folha 02).
//!
//! ⚠️ **Ficheiro irmão de propósito.** As leis do mar são metade dos gates desta cena — os
//! outros três pares cabem em `..._tests.rs` —, e mantê-las juntas levava aquele ficheiro a
//! **684 linhas** contra o alvo de `600` do repo (o portão executável está em `700`, e um
//! integrador que apendesse ali rebentava-o).
//!
//! O que se afirma aqui, e porquê, está no [`Bug #6`](../../../docs/Motion%20Nodes/BUGS_motion_nodes.md):
//! duas causas independentes fazem a MESMA imagem no ecrã — peças a atravessar a banda sem
//! nunca subirem nem descerem — e as réguas que a cena tinha partilhavam essa assinatura com
//! elas.

use super::tests::{DT, registry, scene};
use super::*;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::Graph;

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
