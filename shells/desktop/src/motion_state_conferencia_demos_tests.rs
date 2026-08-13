//! Os gates das quatro cenas da conferência (`PH2D_GPU_COOK_DEMO=32..35`).
//!
//! ⚠️ **Uma cena que não coze é pior que cena nenhuma:** ela abre a tela vazia e o
//! artista lê isso como *"a feature não funciona"*. Cada gate aqui cozinha o
//! documento pela porta real e afirma o que a cena existe para mostrar — e o que
//! ele afirma é a **DIFERENÇA** entre as duas metades, nunca só que a cena tem
//! nós dentro.

use super::*;
use ph2d_eval_motion::MotionCookPump;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::node::NodeTypeId;
use ph2d_nodegraph::value::CookValue;

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("registry builds");
    reg
}

/// Cozinha a cena no playhead pedido e devolve as posições.
fn cook(doc: &MotionDoc, reg: &NodeRegistry, sink: NodeId, t: f64) -> Vec<[f32; 2]> {
    let mut c = Cook::new();
    let out = c.cook(&doc.graph, reg, sink, t).expect("a cena coze");
    let CookValue::Instances(s) = &out[0] else {
        panic!("a saida e um stream")
    };
    match ph2d_nodegraph::attr::Stream::get(s, "P") {
        Some(Column::Vec2(v)) => v.clone(),
        _ => Vec::new(),
    }
}

fn rot(doc: &MotionDoc, reg: &NodeRegistry, sink: NodeId) -> Vec<f32> {
    let mut c = Cook::new();
    let out = c.cook(&doc.graph, reg, sink, 0.0).expect("a cena coze");
    let CookValue::Instances(s) = &out[0] else {
        panic!("stream")
    };
    match ph2d_nodegraph::attr::Stream::get(s, "rot") {
        Some(Column::Scalar(v)) => v.clone(),
        _ => Vec::new(),
    }
}

fn has(doc: &MotionDoc, ty: &str) -> bool {
    doc.graph
        .nodes()
        .iter()
        .any(|n| n.type_id() == NodeTypeId::of(ty))
}

/// **`=32` — metade gira, metade não.**
///
/// As duas filas percorrem a MESMA curva, então a diferença tem de estar só no
/// `rot`: a de cima varre um leque de ângulos, a de baixo fica em zero. Um gate
/// que só pedisse "a cena coze" ficaria verde com o toggle morto nas duas.
#[test]
fn the_write_on_scene_turns_one_row_and_leaves_the_control_flat() {
    let reg = registry();
    let mut doc = MotionDoc::new();
    let sinks = build_write_on_demo_document(&mut doc, &reg).expect("cena bem-tipada");
    assert!(
        has(&doc, "motion.spline_wrap"),
        "a cena contem o no que smoka"
    );

    let r = rot(&doc, &reg, *sinks.first().expect("um sink"));
    assert!(!r.is_empty(), "a cena produz uma coluna de rotacao");
    let (turning, flat) = r.split_at(r.len() / 2);
    let spread = |v: &[f32]| {
        v.iter().fold(f32::MIN, |a, b| a.max(*b)) - v.iter().fold(f32::MAX, |a, b| a.min(*b))
    };
    assert!(
        spread(turning) > 40.0,
        "a fila de cima varre um leque de angulos, deu {}",
        spread(turning)
    );
    assert!(
        flat.iter().all(|a| a.abs() < 1e-4),
        "e a de baixo -- o CONTROLE -- fica em zero: {:?}",
        &flat[..flat.len().min(4)]
    );
}

/// **`=32` — e o `To` de facto ENCOLHE o que a fila cobre.**
///
/// A cena não pode animar um param, então este gate faz pela porta o gesto que o
/// smoke pede à mão: baixar o `to` tem de encurtar o trecho ocupado. Sem ele a
/// cena prometeria um write-on que ninguém verificou.
#[test]
fn dragging_the_to_slider_shortens_what_the_row_covers() {
    let reg = registry();
    let mut doc = MotionDoc::new();
    let sinks = build_write_on_demo_document(&mut doc, &reg).expect("cena");
    let sink = *sinks.first().expect("sink");
    // ⚠️ A cena tem DUAS filas e o `combine` as concatena, então medir a união
    // deixa a fila intocada mascarar a que foi editada — a primeira versão deste
    // gate reprovou por isso, com o produto CERTO. Mede-se só a primeira metade,
    // que é a do `spline_wrap` que o gate mexe.
    let span = |p: &[[f32; 2]]| {
        let xs: Vec<f32> = p[..p.len() / 2].iter().map(|q| q[0]).collect();
        xs.iter().fold(f32::MIN, |a, b| a.max(*b)) - xs.iter().fold(f32::MAX, |a, b| a.min(*b))
    };
    let full = span(&cook(&doc, &reg, sink, 0.0));

    let sw = doc
        .graph
        .nodes()
        .iter()
        .find(|n| n.type_id() == NodeTypeId::of("motion.spline_wrap"))
        .expect("o no esta la")
        .id;
    doc.graph.set_param(sw, "to", 0.4);
    let short = span(&cook(&doc, &reg, sink, 0.0));
    assert!(
        short < full * 0.85,
        "baixar o `to` encurta o trecho: {short} contra {full}"
    );
}

/// **`=33` — uma grade FOGE e a outra fica.**
///
/// O sintoma da célula, medido: com o pivô na origem o centro do layout é
/// multiplicado pela escala e anda; com o centroide ele não se move um décimo.
#[test]
fn the_pivot_scene_shows_one_grid_running_away_and_one_staying() {
    let reg = registry();
    let mut doc = MotionDoc::new();
    let sinks = build_pivot_demo_document(&mut doc, &reg).expect("cena bem-tipada");
    let p = cook(&doc, &reg, *sinks.first().expect("sink"), 0.0);
    assert!(!p.is_empty(), "a cena produz posicoes");

    // As duas metades do `motion.combine`: a de cima pivota na origem.
    let (origin, centroid) = p.split_at(p.len() / 2);
    let cx = |v: &[[f32; 2]]| v.iter().map(|q| q[0]).sum::<f32>() / v.len() as f32;
    assert!(
        cx(origin) > 4.0,
        "o pivo na origem DOBRA o deslocamento (2,2 -> ~4,4), deu {}",
        cx(origin)
    );
    assert!(
        (cx(centroid) - 2.2).abs() < 0.1,
        "e o centroide fica onde o layout foi posto, deu {}",
        cx(centroid)
    );
}

/// **`=34` — a nuvem ORBITA, e nenhuma `force.*` está na cena.**
///
/// ⚠️ **A versão anterior deste gate afirmava só que a nuvem SE MOVEU, e por isso
/// ficou verde sobre a cena que o smoke reprovou:** `a = k·perp(P)` é uma espiral
/// que cresce como `e^{√(k/2)·t}`, e *"a maioria se moveu"* é exactamente o que
/// uma explosão faz. A propriedade que separa uma órbita de uma espiral é o
/// **RAIO**, e ela mata as duas doenças de uma vez — a espiral (o raio cresce) e
/// uma discordância entre o `ω` da semente e o `ω²` da força (o raio pulsa).
#[test]
fn the_formula_force_scene_orbits_with_no_force_node_in_it() {
    let reg = registry();
    let mut doc = MotionDoc::new();
    let sinks = build_formula_force_demo_document(&mut doc, &reg).expect("cena bem-tipada");

    for banned in [
        "force.vortex",
        "force.attractor",
        "force.curl",
        "force.wind",
    ] {
        assert!(!has(&doc, banned), "a cena nao tem `{banned}` -- e a tese");
    }
    assert!(
        has(&doc, "motion.make_point"),
        "e tem o no que escreve accel"
    );

    // ⚠️ Uma sim avança pelo PUMP, não por um `Cook::cook` cru: o cook devolve o
    // estado do tique pedido e NÃO o faz correr. A primeira versão deste gate
    // media com o relógio parado e reprovou sobre um produto correto.
    let scopes = ph2d_node_motion_time_remap::time_scopes(&doc.graph, &reg);
    let mut pump = MotionCookPump::new();
    let mut at0 = Vec::new();
    for tick in 0..40u64 {
        pump.advance_or_scrub_scoped(
            &doc.graph,
            &reg,
            &sinks,
            tick,
            |k| k as f64 / 60.0,
            [0.0, 0.0, 1.0, 1.0],
            [1.0, 1.0],
            &scopes,
        );
        if tick == 0 {
            at0 = pump.instances.iter().map(|i| i.world_pos).collect();
        }
    }
    let last: Vec<[f32; 2]> = pump.instances.iter().map(|i| i.world_pos).collect();
    assert!(!at0.is_empty(), "a cena produz instancias");
    assert_eq!(last.len(), at0.len(), "a contagem nao muda");

    // **Ela GIRA** — 40 tiques a ω = 2,5 rad/s são 95,5° previstos; medido, 94,3
    // (o tique de semente não anda, e o Euler semi-implícito desloca a frequência
    // um nada). Um alvo morto no `make_point` escreveria `P` e isto seria zero.
    let swept = |a: &[f32; 2], b: &[f32; 2]| {
        let d = (b[1].atan2(b[0]) - a[1].atan2(a[0])).to_degrees();
        d - 360.0 * (d / 360.0).round()
    };
    let turn = at0
        .iter()
        .zip(&last)
        .map(|(a, b)| swept(a, b).abs())
        .fold(0.0f32, f32::max);
    assert!(
        turn > 60.0,
        "a nuvem varre um bom arco em 40 tiques, deu {turn:.1} graus"
    );

    // **E NÃO SE ABRE** — a propriedade que a espiral quebrava. Medido 0,16%; o
    // bar a 3% deixa passar o erro do integrador e recusa qualquer crescimento
    // (a espiral de antes dobrava o raio a cada 0,62 s ⇒ +62% nesta janela).
    let drift = at0
        .iter()
        .zip(&last)
        .filter_map(|(a, b)| {
            let r0 = a[0].hypot(a[1]);
            (r0 > 0.2).then(|| ((b[0].hypot(b[1]) - r0) / r0).abs())
        })
        .fold(0.0f32, f32::max);
    assert!(
        drift < 0.03,
        "todo ponto guarda o seu raio -- e uma ORBITA, nao uma espiral: {:.2}%",
        drift * 100.0
    );
}

/// **`=35` — só a faixa mira, e as bordas miram PELA METADE.**
///
/// A cena existe para o gradiente ser visto; este gate afirma que ele existe, que
/// é a metade que um olho não consegue conferir sozinho (um campo binário e um
/// campo macio desenham quase a mesma coisa em miniatura).
#[test]
fn the_partial_aim_scene_has_a_band_that_aims_and_a_soft_edge() {
    let reg = registry();
    let mut doc = MotionDoc::new();
    let sinks = build_partial_aim_demo_document(&mut doc, &reg).expect("cena bem-tipada");
    assert!(has(&doc, "motion.look_at") && has(&doc, "field.box"));

    let r = rot(&doc, &reg, *sinks.first().expect("sink"));
    assert!(!r.is_empty(), "ha rotacao a olhar");
    let untouched = r.iter().filter(|a| a.abs() < 1e-4).count();
    assert!(
        untouched > 0,
        "quem esta fora do campo fica EXACTAMENTE como estava"
    );
    // A borda macia: existe ângulo estritamente entre "nada" e o máximo.
    let peak = r.iter().fold(0.0f32, |a, b| a.max(b.abs()));
    assert!(peak > 1.0, "e a faixa do meio mira de verdade, pico {peak}");
    let partial = r
        .iter()
        .filter(|a| a.abs() > 1e-3 && a.abs() < peak * 0.9)
        .count();
    assert!(
        partial > 0,
        "e ha quem mire PELA METADE -- e o contrato de familia, nao um degrau"
    );
}

/// **A SONDA das quatro cenas** — os números que a mensagem de smoke cita.
///
/// `cargo test -p ph2d-host-desktop --bin ph2d-host-desktop probe_conferencia_scenes -- --ignored --nocapture`
#[test]
#[ignore = "sonda"]
fn probe_conferencia_scenes() {
    let reg = registry();
    for (n, build) in [
        (
            32,
            build_write_on_demo_document
                as fn(&mut MotionDoc, &NodeRegistry) -> Option<Vec<NodeId>>,
        ),
        (33, build_pivot_demo_document),
        (34, build_formula_force_demo_document),
        (35, build_partial_aim_demo_document),
    ] {
        let mut doc = MotionDoc::new();
        let sinks = build(&mut doc, &reg).expect("cena");
        let p = cook(&doc, &reg, sinks[0], 0.0);
        let r = rot(&doc, &reg, sinks[0]);
        let ang = r.iter().fold(0.0f32, |a, b| a.max(b.abs()));
        println!(
            "[={n}] {} nos | {} instancias | rot max {ang:.1} graus",
            doc.graph.nodes().len(),
            p.len()
        );
    }
    // ⚠️ `=32` mede-se pelo caminho do APP (o pump → `RenderInstance.basis`), não
    // pelo cook: a rotação já estava no `basis` quando o smoke a reprovou, e o que
    // faltava era a razão TAMANHO/PASSO — com 0,63 os vizinhos se tocam e a fila
    // lê como uma fita.
    let mut doc = MotionDoc::new();
    let sinks = build_write_on_demo_document(&mut doc, &reg).expect("cena");
    let scopes = ph2d_node_motion_time_remap::time_scopes(&doc.graph, &reg);
    let mut pump = MotionCookPump::new();
    pump.advance_or_scrub_scoped(
        &doc.graph,
        &reg,
        &sinks,
        0,
        |k| k as f64 / 60.0,
        [0.0, 0.0, 1.0, 1.0],
        [1.0, 1.0],
        &scopes,
    );
    let n = pump.instances.len();
    let deg: Vec<f32> = pump
        .instances
        .iter()
        .map(|i| i.basis[1].atan2(i.basis[0]).to_degrees())
        .collect();
    let top = &deg[..n / 2];
    let near45 = top
        .iter()
        .filter(|a| (a.rem_euclid(90.0) - 45.0).abs() < 12.0)
        .count();
    // ⚠️ O passo mede-se dentro de UMA pista. A grade é row-major, então os
    // dezasseis primeiros são a pista de cima; percorrer a metade inteira saltaria
    // entre pistas e reportaria o salto como se fosse o passo ao longo do arco.
    let pos: Vec<[f32; 2]> = pump.instances[..16].iter().map(|i| i.world_pos).collect();
    let gaps: Vec<f32> = pos
        .windows(2)
        .map(|w| (w[1][0] - w[0][0]).hypot(w[1][1] - w[0][1]))
        .collect();
    let step = gaps.iter().sum::<f32>() / gaps.len() as f32;
    println!(
        "[=32] basis {:.1} .. {:.1} graus | {near45} de {} a menos de 12 de um losango | tamanho {:.2} passo {step:.3} razao {:.2}",
        top.iter().fold(f32::MAX, |a, b| a.min(*b)),
        top.iter().fold(f32::MIN, |a, b| a.max(*b)),
        top.len(),
        pump.instances[0].size[0],
        pump.instances[0].size[0] / step,
    );

    // A ESPESSURA da fita, e o que o `Height` faz com ela.
    for h in [1.0f32, 0.5, 0.0] {
        let mut doc = MotionDoc::new();
        let sinks = build_write_on_demo_document(&mut doc, &reg).expect("cena");
        for n in doc
            .graph
            .nodes()
            .iter()
            .map(|n| (n.id, n.type_id()))
            .collect::<Vec<_>>()
        {
            if n.1 == NodeTypeId::of("motion.spline_wrap") {
                doc.graph.set_param(n.0, "height_scale", h);
            }
        }
        let p = cook(&doc, &reg, sinks[0], 0.0);
        // A espessura: a maior distância entre dois elementos da MESMA coluna.
        let n = p.len() / 2;
        let thick = (0..n / 3)
            .map(|c| {
                let (a, b) = (p[c], p[c + 2 * (n / 3)]);
                (a[0] - b[0]).hypot(a[1] - b[1])
            })
            .fold(0.0f32, f32::max);
        println!("[=32] height {h:.1} -> espessura da fita {thick:.3}");
    }

    // O pivô, o número da célula.
    let mut doc = MotionDoc::new();
    let sinks = build_pivot_demo_document(&mut doc, &reg).expect("cena");
    let p = cook(&doc, &reg, sinks[0], 0.0);
    let (o, c) = p.split_at(p.len() / 2);
    let cx = |v: &[[f32; 2]]| v.iter().map(|q| q[0]).sum::<f32>() / v.len() as f32;
    println!("[=33] centro: origem {:.2} | centroide {:.2}", cx(o), cx(c));
    // O redemoinho, quanto anda em 40 tiques.
    let mut doc = MotionDoc::new();
    let sinks = build_formula_force_demo_document(&mut doc, &reg).expect("cena");
    let scopes = ph2d_node_motion_time_remap::time_scopes(&doc.graph, &reg);
    let mut pump = MotionCookPump::new();
    let mut at0 = Vec::new();
    for tick in 0..40u64 {
        pump.advance_or_scrub_scoped(
            &doc.graph,
            &reg,
            &sinks,
            tick,
            |k| k as f64 / 60.0,
            [0.0, 0.0, 1.0, 1.0],
            [1.0, 1.0],
            &scopes,
        );
        if tick == 0 {
            at0 = pump.instances.iter().map(|i| i.world_pos).collect();
        }
    }
    let last: Vec<[f32; 2]> = pump.instances.iter().map(|i| i.world_pos).collect();
    let d: Vec<f32> = at0
        .iter()
        .zip(&last)
        .map(|(a, b)| (a[0] - b[0]).hypot(a[1] - b[1]))
        .collect();
    // A propriedade da órbita: o RAIO de cada ponto é constante, e o ÂNGULO anda.
    let drift = at0
        .iter()
        .zip(&last)
        .map(|(a, b)| {
            let (r0, r1) = (a[0].hypot(a[1]), b[0].hypot(b[1]));
            if r0 > 0.2 {
                ((r1 - r0) / r0).abs()
            } else {
                0.0
            }
        })
        .fold(0.0f32, f32::max);
    let swept = at0
        .iter()
        .zip(&last)
        // ⚠️ O `atan2` tem CORTE DE RAMO: a diferença crua reportava 265,7° para
        // um ponto que varreu 94,3°, e um bar escrito sobre esse número mediria o
        // corte, não a órbita. Enrola-se para (−180, 180].
        .map(|(a, b)| {
            let d = (b[1].atan2(b[0]) - a[1].atan2(a[0])).to_degrees();
            d - 360.0 * (d / 360.0).round()
        })
        .fold(0.0f32, |m, v| m.max(v.abs()));
    println!(
        "[=34] {} de {} se moveram | deslocamento max {:.3} | deriva de raio max {:.2}% | varreu {swept:.1} graus",
        d.iter().filter(|x| **x > 0.01).count(),
        d.len(),
        d.iter().fold(0.0f32, |a, b| a.max(*b)),
        drift * 100.0
    );
}

/// **TODO controle que a cena `=32` oferece TEM de fazer alguma coisa NELA.**
///
/// ⚠️ **A lei do botão morto, aplicada à cena em vez de ao painel.** Um param pode
/// estar vivo no nó e **inerte na cena**, porque a cena não lhe deu entrada: o
/// `Height` desloca `p.y` ao longo da normal da curva, e a primeira versão desta
/// cena era uma FILA (`rows = 1`) ⇒ `p.y` zero ⇒ zero vezes qualquer coisa. O nó
/// estava certo, o slider estava pintado, e o Enio perguntou para que ele servia.
///
/// Foi a **terceira** vez nesta cena que um controle não demonstrava nada (a
/// rotação ilegível, a espiral, e agora este), e as três eram a mesma coisa: a
/// cena não continha o fenômeno que prometia. Este gate faz a pergunta uma vez
/// por param, em vez de eu me lembrar dela uma vez por smoke.
///
/// O oráculo é `P` **ou** `rot`: o `Follow Curve` não move um elemento, ele o
/// vira — pedir só posição deixaria o toggle passar por morto.
#[test]
fn every_control_the_write_on_scene_offers_does_something_in_it() {
    let reg = registry();
    let sw_id = NodeTypeId::of("motion.spline_wrap");

    let cook_both = |doc: &MotionDoc, sink: NodeId| -> (Vec<[f32; 2]>, Vec<f32>) {
        (cook(doc, &reg, sink, 0.0), rot(doc, &reg, sink))
    };
    let base_doc = {
        let mut d = MotionDoc::new();
        build_write_on_demo_document(&mut d, &reg).expect("cena");
        d
    };
    let manifest = reg
        .manifests()
        .find(|m| m.id == sw_id)
        .expect("o no existe");
    let hints = reg.param_ui(sw_id);

    for spec in manifest.params {
        let mut doc = MotionDoc::new();
        let sinks = build_write_on_demo_document(&mut doc, &reg).expect("cena");
        let sink = *sinks.first().expect("sink");
        let before = cook_both(&base_doc, sink);

        // Um valor DENTRO da faixa que o painel oferece e diferente do de hoje —
        // ⚠️ empurrar sempre para cima faria o `to` (default 1, teto 1) saturar e
        // o gate reprovaria um param VIVO por ter sido cutucado onde não anda.
        let (lo, hi) = hints
            .into_iter()
            .flatten()
            .find(|h| h.param == spec.name)
            .filter(|h| h.max > h.min)
            .map_or((spec.default - 1.0, spec.default + 1.0), |h| (h.min, h.max));
        let a = lo + (hi - lo) * 0.37;
        let nudged = if (a - spec.default).abs() > 1e-4 {
            a
        } else {
            lo + (hi - lo) * 0.71
        };

        for n in doc
            .graph
            .nodes()
            .iter()
            .filter(|n| n.type_id() == sw_id)
            .map(|n| n.id)
            .collect::<Vec<_>>()
        {
            doc.graph.set_param(n, spec.name, nudged);
        }
        let after = cook_both(&doc, sink);
        let moved = before
            .0
            .iter()
            .zip(&after.0)
            .any(|(a, b)| (a[0] - b[0]).hypot(a[1] - b[1]) > 1e-4);
        let turned = before.1.len() != after.1.len()
            || before
                .1
                .iter()
                .zip(&after.1)
                .any(|(a, b)| (a - b).abs() > 1e-4);
        assert!(
            moved || turned,
            "`{}` ({} -> {nudged}) nao mudou NADA na cena -- ou o param esta morto, \
             ou a cena nao lhe da entrada (foi o caso do `height_scale` com uma fila)",
            spec.name,
            spec.default
        );
    }
}
