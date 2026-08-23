//! **SONDA — o morph aguenta trocar o VERBO de uma booleana?**
//!
//! A pergunta que decide o desenho da compatibilidade booleana×States (Enio, 2026-08-23:
//! *"inclusive com a possibilidade de mudar o tipo do boolean no meio da animação"*).
//!
//! Uma operação booleana é um valor **DISCRETO**: não há meio-caminho entre `Union` e `Subtract`.
//! O Blender e o After Effects resolvem isto com interpolação **constante** — o valor salta. Mas o
//! *resultado* de uma booleana é uma FORMA, e este app já sabe interpolar uma forma noutra
//! (`Plan`). A sonda mede se ele aguenta os pares que a troca de verbo produz — sobretudo a
//! **mudança de topologia** (a união tem 1 contorno; a subtração tem 2, com o buraco).
//!
//! Rodar: `cargo test -p ph2d-vec-blend --test probe_boolean_morph -- --ignored --nocapture`

use ph2d_vec_blend::Plan;
use ph2d_vec_boolean::{PathfinderOp, area, pathfinder};
use ph2d_vec_scene::{VecPath, rectangle};

/// Quantos pontos de arco a régua de CONTINUIDADE amostra em cada contorno.
const SAMPLES: usize = 64;
/// Quantos quadros a passagem de um estado a outro tem, a 60 fps numa transição de ~1 s.
const FRAMES: usize = 60;

/// Duas fixtures, e a segunda existe por disciplina: a primeira **não contém o fenômeno**.
///
/// - `TRIO` — a cena de smoke `=48`: dois que se cruzam e uma barra que morde os dois. Os quatro
///   verbos dão **1 contorno**, então ela nunca exercita a mudança de topologia.
/// - `DONUT` — um retângulo grande com um pequeno **inteiramente dentro**. Aqui `Subtract` e
///   `Exclude` produzem **2 contornos** (o de fora e o buraco) e `Union` produz 1: é exactamente o
///   par que faz um buraco NASCER no meio da animação, e o único que decide se o morph serve.
fn rig(donut: bool) -> Vec<VecPath> {
    if donut {
        vec![
            rectangle([0.0, 0.0], [20.0, 20.0]),
            rectangle([6.0, 6.0], [14.0, 14.0]),
        ]
    } else {
        vec![
            rectangle([0.0, 0.0], [20.0, 20.0]),
            rectangle([10.0, 0.0], [30.0, 20.0]),
            rectangle([15.0, 5.0], [25.0, 15.0]),
        ]
    }
}

fn cook_in(donut: bool, op: PathfinderOp) -> Option<VecPath> {
    let t = rig(donut);
    let refs: Vec<&VecPath> = t.iter().collect();
    pathfinder(&refs, op).ok()?.into_iter().next()
}

#[test]
#[ignore = "sonda de desenho — roda sob demanda"]
fn probe_boolean_verb_morph() {
    for donut in [false, true] {
        println!(
            "\n===== RIG: {} =====",
            if donut {
                "DONUT (o buraco NASCE)"
            } else {
                "TRIO (1 contorno em todos)"
            }
        );
        probe_rig(donut);
    }
}

fn probe_rig(donut: bool) {
    let ops = [
        ("Union", PathfinderOp::Union),
        ("Subtract", PathfinderOp::Subtract),
        ("Intersect", PathfinderOp::Intersect),
        ("Exclude", PathfinderOp::Exclude),
    ];
    println!("\n  verbo      | contornos | area");
    println!("  -----------|-----------|--------");
    let mut cooked: Vec<(&str, VecPath)> = Vec::new();
    for (name, op) in ops {
        if let Some(p) = cook_in(donut, op) {
            println!(
                "  {name:<10} | {:>9} | {:>6.1}",
                p.contour_count(),
                area(&p).abs()
            );
            cooked.push((name, p));
        } else {
            println!("  {name:<10} |     RECUSOU pelo motor");
        }
    }

    println!("\n  PAR (from -> to)          | Plan | contornos em t=0,5 | area 0,0 / 0,5 / 1,0");
    println!("  --------------------------|------|--------------------|----------------------");
    for (na, a) in &cooked {
        for (nb, b) in &cooked {
            if na == nb {
                continue;
            }
            let label = format!("{na} -> {nb}");
            match Plan::new(a, b) {
                Some(plan) => {
                    let (m0, m5, m1) = (plan.at(0.0), plan.at(0.5), plan.at(1.0));
                    println!(
                        "  {label:<25} |  sim | {:>18} | {:>5.1} / {:>5.1} / {:>5.1}",
                        m5.contour_count(),
                        area(&m0).abs(),
                        area(&m5).abs(),
                        area(&m1).abs()
                    );
                }
                None => println!("  {label:<25} |  NAO | (o Plan recusou o par)"),
            }
        }
    }
    println!();
}

// ============================================================================
// SONDA 2 — **e se os operandos ESTIVEREM A MOVER-SE?** (Enio, 2026-08-23)
// ============================================================================
//
// A troca de verbo não acontece numa cena parada: as formas podem estar animadas em posição,
// escala e rotação ao mesmo tempo. Isso muda a pergunta de sítio.
//
// ⚠️ **O `Plan` casa contornos por FASE, e o casamento é refeito a cada quadro** quando a entrada
// muda. Se ele escolher um casamento diferente entre dois quadros consecutivos, a forma
// **RE-PARAMETRIZA** — e uma re-parametrização é um POP na tela, mesmo com a área a variar
// suavemente. É por isso que a régua aqui **não pode ser a área**: ela é um escalar global, e um
// escalar global é cego a um salto de correspondência (a lição das réguas do quad remesh).
//
// A régua é a **maior distância que um ponto de arco anda entre dois quadros consecutivos**, com
// dois controlos ao lado:
//   - CONTROLE  — a mesma peça a mover-se **sem trocar de verbo**: o quanto o movimento sozinho
//     já move um ponto por quadro. É o piso; abaixo dele não há nada a pedir.
//   - SALTO     — o que Blender/AE/Rive fazem (verbo constante, troca no meio). O tecto do
//     defeito que estamos a tentar não ter.

/// O que uma via mediu: *(rótulo, pontos de arco por quadro, área por quadro, contornos por
/// quadro, o desenho de cada quadro)*.
type Lane = (
    &'static str,
    Vec<Vec<[f64; 2]>>,
    Vec<f64>,
    Vec<usize>,
    Vec<VecPath>,
);

/// A peça no instante `t`: o quadrado de fora parado, o de dentro a ATRAVESSAR a parede.
///
/// ⚠️ Em `t = 0` o de dentro está inteiramente contido (⇒ `Subtract` faz uma ROSQUINHA, 2
/// contornos) e em `t = 1` ele já saiu meio corpo (⇒ `Subtract` faz uma DENTADA, 1 contorno). A
/// fixture contém, portanto, as duas mudanças de topologia ao mesmo tempo: a que a troca de verbo
/// provoca e a que o movimento provoca.
fn moving_rig(t: f64, rig: usize) -> Vec<VecPath> {
    // 0 = o de dentro anda e FICA dentro · 1 = ele ATRAVESSA a parede · 2 = a PEÇA INTEIRA viaja.
    let (outer, inner) = match rig {
        0 => (0.0, 4.0 * t),
        1 => (0.0, 14.0 * t),
        _ => (100.0 * t, 100.0 * t),
    };
    vec![
        rectangle([outer, 0.0], [20.0 + outer, 20.0]),
        rectangle([6.0 + inner, 6.0], [14.0 + inner, 14.0]),
    ]
}

fn rig_label(rig: usize) -> &'static str {
    match rig {
        0 => "o de dentro anda mas fica DENTRO: a topologia das duas pontas nao muda",
        1 => {
            "o de dentro ATRAVESSA a parede: a topologia do Subtract muda a meio (2 -> 1 contorno)"
        }
        _ => "a PECA INTEIRA viaja 100 unidades (5x a propria largura) enquanto o verbo troca",
    }
}

fn cook(shapes: &[VecPath], op: PathfinderOp) -> Vec<VecPath> {
    let refs: Vec<&VecPath> = shapes.iter().collect();
    pathfinder(&refs, op).unwrap_or_default()
}

/// **Uma lista de resultados vira UMA forma composta** — a porta que o cozimento morfado precisa.
///
/// Uma booleana pode devolver vários grupos disjuntos (uma dentada que parte a peça em duas), e o
/// `Plan` casa DUAS formas. Juntar os contornos todos numa forma composta é o que deixa o
/// casamento decidir peça a peça, em vez de alguém escolher por índice qual grupo vira qual.
fn as_one(items: &[VecPath]) -> Option<VecPath> {
    let mut it = items.iter();
    let mut out = it.next()?.clone();
    for extra in it {
        for c in 0..extra.contour_count() {
            if let Some((verts, closed)) = extra.contour(c) {
                out.subpaths.push(ph2d_vec_scene::Contour {
                    verts: verts.to_vec(),
                    closed,
                });
            }
        }
    }
    Some(out)
}

/// O contorno primário reamostrado em [`SAMPLES`] pontos por COMPRIMENTO DE ARCO.
///
/// ⚠️ Reamostrar por arco (e não usar os vértices) é o que torna a régua comparável entre dois
/// quadros: a booleana devolve contagens de vértices diferentes a cada quadro, e comparar
/// vértice-a-vértice mediria a contagem em vez do desenho.
fn arc_samples(path: &VecPath) -> Vec<[f64; 2]> {
    let pts: Vec<[f64; 2]> = path.verts.iter().map(|v| v.anchor).collect();
    if pts.len() < 2 {
        return vec![[0.0, 0.0]; SAMPLES];
    }
    let n = pts.len();
    let mut cum = vec![0.0f64];
    for i in 0..n {
        let a = pts[i];
        let b = pts[(i + 1) % n];
        cum.push(cum[i] + (b[0] - a[0]).hypot(b[1] - a[1]));
    }
    let total = *cum.last().unwrap_or(&0.0);
    if total <= f64::EPSILON {
        return vec![pts[0]; SAMPLES];
    }
    (0..SAMPLES)
        .map(|k| {
            #[allow(clippy::cast_precision_loss)]
            let target = total * (k as f64) / (SAMPLES as f64);
            let mut i = 0;
            while i + 1 < cum.len() && cum[i + 1] < target {
                i += 1;
            }
            let seg = cum[i + 1] - cum[i];
            let f = if seg > f64::EPSILON {
                (target - cum[i]) / seg
            } else {
                0.0
            };
            let (a, b) = (pts[i % n], pts[(i + 1) % n]);
            [a[0] + (b[0] - a[0]) * f, a[1] + (b[1] - a[1]) * f]
        })
        .collect()
}

/// O maior deslocamento de um ponto de arco entre dois desenhos.
///
/// ⚠️ **Ela mede RE-PARAMETRIZAÇÃO, e essa só é visível no morph.** Num desenho que apenas se
/// cozinha e se preenche, trocar por que ponto o contorno começa não muda um pixel; dentro de um
/// `Plan`, muda a forma inteira do meio. É por isso que ela é a coluna SECUNDÁRIA aqui, e não a
/// régua.
fn max_step(a: &[[f64; 2]], b: &[[f64; 2]]) -> f64 {
    a.iter()
        .zip(b.iter())
        .map(|(p, q)| (p[0] - q[0]).hypot(p[1] - q[1]))
        .fold(0.0f64, f64::max)
}

/// ⭐ **A RÉGUA: quanta TINTA mudou de sítio entre dois quadros** — a área da diferença simétrica.
///
/// É a única pergunta que o olho faz (*"o desenho saltou?"*) e a única cega à parametrização: dois
/// desenhos da MESMA região dão zero, por mais que os vértices tenham mudado de lugar. A coluna de
/// arco ao lado não pode ser a régua justamente porque ela não distingue as duas coisas.
fn ink_moved(a: &VecPath, b: &VecPath) -> f64 {
    let refs: Vec<&VecPath> = vec![a, b];
    pathfinder(&refs, PathfinderOp::Exclude)
        .map(|out| out.iter().map(|p| area(p).abs()).sum())
        .unwrap_or(f64::NAN)
}

#[test]
#[ignore = "sonda de desenho — roda sob demanda"]
fn probe_verb_morph_while_the_operands_move() {
    // ⚠️ **Duas travessias, e a segunda existe pela mesma disciplina da fixture DONUT:** é preciso
    // separar *"o verbo troca enquanto a peça se move"* de *"o MOVIMENTO sozinho muda a topologia
    // de uma das pontas"*. Sem as duas, um salto medido não teria dono.
    for rig in 0..3 {
        probe_moving(rig);
    }
}

fn probe_moving(rig: usize) {
    println!("\n===== OS OPERANDOS MOVEM-SE E O VERBO TROCA =====");
    println!("  ({})\n", rig_label(rig));

    let mut rows: Vec<Lane> = Vec::new();
    let mut cost_ns = 0u128;

    for lane in [
        "CONTROLE (so' o movimento)",
        "SALTO (Blender/AE)",
        "MORPH (par fresco)",
        "PERSEGUICAO (parte do vivo)",
    ] {
        // ⭐ **A PERSEGUIÇÃO é a lei da casa aplicada ao desenho:** a `Machine` já parte da pose
        // VIVA e nunca da autorada, e é isso que faz uma transição interrompida continuar de onde
        // está em vez de saltar. Aqui o mesmo: cada quadro morfa **do que está na tela** para o
        // resultado do verbo de chegada, pela fração do caminho que ainda falta.
        let mut alive: Option<VecPath> = None;
        let mut samples = Vec::new();
        let mut areas = Vec::new();
        let mut contours = Vec::new();
        let mut drawings: Vec<VecPath> = Vec::new();
        for i in 0..=FRAMES {
            #[allow(clippy::cast_precision_loss)]
            let t = i as f64 / FRAMES as f64;
            let shapes = moving_rig(t, rig);
            let drawn = match lane {
                "CONTROLE (so' o movimento)" => as_one(&cook(&shapes, PathfinderOp::Subtract)),
                "SALTO (Blender/AE)" => as_one(&cook(
                    &shapes,
                    if t < 0.5 {
                        PathfinderOp::Union
                    } else {
                        PathfinderOp::Subtract
                    },
                )),
                "MORPH (par fresco)" => {
                    let clock = std::time::Instant::now();
                    let from = as_one(&cook(&shapes, PathfinderOp::Union));
                    let to = as_one(&cook(&shapes, PathfinderOp::Subtract));
                    let out = match (from.as_ref(), to.as_ref()) {
                        (Some(a), Some(b)) => {
                            Plan::new(a, b).map_or_else(|| a.clone(), |p| p.at(t))
                        }
                        _ => from.clone().unwrap_or_default(),
                    };
                    cost_ns += clock.elapsed().as_nanos();
                    Some(out)
                }
                _ => {
                    let from = as_one(&cook(&shapes, PathfinderOp::Union));
                    let to = as_one(&cook(&shapes, PathfinderOp::Subtract));
                    #[allow(clippy::cast_precision_loss)]
                    let tp = (i.saturating_sub(1)) as f64 / FRAMES as f64;
                    // A fração do que FALTA, e não `t`: no último quadro ela vale 1, então a
                    // chegada é EXATA — a mesma promessa que o `Machine::arrive` faz.
                    let step = if i == 0 { 0.0 } else { (t - tp) / (1.0 - tp) };
                    let out = match (alive.as_ref(), from.as_ref(), to.as_ref()) {
                        (None, Some(a), _) => a.clone(),
                        (Some(prev), _, Some(b)) => {
                            Plan::new(prev, b).map_or_else(|| prev.clone(), |p| p.at(step))
                        }
                        (Some(prev), _, None) => prev.clone(),
                        _ => VecPath::default(),
                    };
                    alive = Some(out.clone());
                    Some(out)
                }
            };
            let d = drawn.unwrap_or_default();
            samples.push(arc_samples(&d));
            areas.push(area(&d).abs());
            contours.push(d.contour_count());
            drawings.push(d);
        }
        rows.push((lane, samples, areas, contours, drawings));
    }

    println!(
        "  via                        | TINTA que salta num quadro | passo de arco | area 0,0 / 0,5 / 1,0 | contornos"
    );
    println!(
        "  ---------------------------|----------------------------|---------------|----------------------|----------"
    );
    for (name, samples, areas, contours, drawings) in &rows {
        let worst = samples
            .windows(2)
            .map(|w| max_step(&w[0], &w[1]))
            .fold(0.0f64, f64::max);
        let jump = drawings
            .windows(2)
            .map(|w| ink_moved(&w[0], &w[1]))
            .fold(0.0f64, f64::max);
        let uniq: Vec<String> = {
            let mut v: Vec<usize> = contours.clone();
            v.dedup();
            v.iter().map(ToString::to_string).collect()
        };
        println!(
            "  {name:<26} | {jump:>26.3} | {worst:>13.3} | {:>5.1} / {:>5.1} / {:>5.1} | {}",
            areas[0],
            areas[FRAMES / 2],
            areas[FRAMES],
            uniq.join("->")
        );
    }

    // ⚠️ **O que a PERSEGUIÇÃO custa:** ela parte do que está na tela, então o lado de PARTIDA
    // deixa de acompanhar o movimento dos operandos — o desenho pode ficar para trás. Sem esta
    // coluna a perseguição parece grátis, e o preço dela é justamente o que a peça viajante mede.
    let drift = rows[2]
        .4
        .iter()
        .zip(rows[3].4.iter())
        .map(|(fresh, chase)| ink_moved(fresh, chase))
        .fold(0.0f64, f64::max);
    println!("\n  A PERSEGUICAO afasta-se do par fresco em ate' {drift:.3} de tinta");

    // ONDE ele salta — sem isto a coluna diz que há defeito e não diz onde. Os TRÊS piores
    // quadros bastam: um salto estrutural é um pico isolado, e um arrasto é uma rampa inteira.
    let morph = &rows[2];
    let mut worst: Vec<(f64, f64, usize, usize)> = (1..=FRAMES)
        .map(|i| {
            #[allow(clippy::cast_precision_loss)]
            let t = i as f64 / FRAMES as f64;
            (
                t,
                ink_moved(&morph.4[i - 1], &morph.4[i]),
                morph.3[i - 1],
                morph.3[i],
            )
        })
        .collect();
    worst.sort_by(|a, b| b.1.total_cmp(&a.1));
    println!("\n  MORPH (par fresco), os tres quadros que mais saltam:");
    println!("  t      | tinta que saltou | contornos out");
    println!("  -------|------------------|---------------");
    for (t, jump, a, b) in worst.iter().take(3) {
        println!("  {t:>5.3} | {jump:>16.3} | {a} -> {b}");
    }

    #[allow(clippy::cast_precision_loss)]
    let per_frame_ms = cost_ns as f64 / 1e6 / (FRAMES as f64 + 1.0);
    println!(
        "\n  CUSTO do morph: {per_frame_ms:.3} ms por quadro (dois cozimentos + Plan::new + at)"
    );
    println!("  (um quadro de 60 fps mede 16,667 ms)\n");
}
