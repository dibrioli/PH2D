//! **O TETO DIGITÁVEL DE UM PARAM SEM LEI DE SATURAÇÃO É ONDE O PASSO DEIXA DE SIGNIFICAR ALGO.**
//!
//! Bloco Z, doc [91](../../../docs/Motion%20Nodes/91_os_tetos_que_ninguem_mediu.md) — a lei do
//! `CLAUDE.md` §0.0: *um limite legítimo diz de que RECURSO ele é, e traz a medição*.
//!
//! ## O recurso, nomeado: a precisão de representação
//!
//! Três params deste catálogo são **extensões de mundo puras** — `field.box::width`/`height` e
//! `field.radial_sweep::radius`/`inner_radius` — e um é uma **contagem pura**
//! (`field.remap::steps`). Nenhum deles satura: o nó honra qualquer valor finito, e uma caixa
//! maior que a cena é **exactamente o neutro que o doc-comment dele promete**.
//!
//! Então de que é o teto? Do `f32`. Acima de um certo módulo, somar o `step` do slider **não
//! move o número**: `v + step == v`, bit a bit. A partir dali dois valores autoráveis vizinhos
//! são o MESMO campo, e um teto digitável que os aceitasse *aceitaria e mentiria* — a mesma
//! lei que o `sim.spawn::burst` já aplica sobre `MAX_PER_TICK` (doc 88 §B2).
//!
//! ⚠️ **O que estes quatro tinham antes era pior que um teto errado: não tinham teto nenhum.**
//! Sem `ParamHardMax` o `ui.rs:206` é explícito — *"a param with no entry here types to its soft
//! `max`"* —, então o digitado parava no fim do ARRASTO (40 na caixa, 32 nos degraus). O neutro
//! que o doc-comment do `field.box` promete (*"a box larger than the scene with `soft = 0`"*) e
//! que o teste dele usa com `width = 100` era **inalcançável por gesto nenhum**.
//!
//! ## Por que isto é um GATE e não uma tabela escrita à mão
//!
//! O número depende do `step` do slider, que é do painel e pode mudar. Escrito à mão ele
//! envelhece em silêncio no dia em que alguém afinar o arrasto. Aqui ele é **derivado** do
//! `ParamUiHint` que o próprio nó registou, e o gate reprova quando os dois discordam.

use ph2d_node_registry::{NodeRegistry, ParamUiHint};
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, Graph};

/// **O maior `f32` em que somar `step` ainda MOVE o número** — a medição, não uma tabela.
///
/// Dentro de um binado `[2^e, 2^{e+1})` o `ulp` é constante (`2^{e-23}`), então `v + step != v`
/// vale para o binado inteiro ou para nenhum dele (a fronteira exacta é `step > ulp/2`, com o
/// desempate para par a decidir o caso de igualdade). A busca portanto é sobre o EXPOENTE, e o
/// teto é o maior representável do último binado que sobrevive.
fn step_ceiling(step: f32) -> f32 {
    assert!(
        step > 0.0 && step.is_finite(),
        "um step tem de ser um passo"
    );
    let mut last = step;
    for e in -20..=127 {
        let base = (2.0_f32).powi(e);
        if !base.is_finite() {
            break;
        }
        // O maior representável deste binado: o próximo binado menos um ulp.
        let top = f32::from_bits(((2.0_f32).powi(e + 1)).to_bits() - 1);
        if !top.is_finite() {
            break;
        }
        if base + step != base && top + step != top {
            last = top;
        }
    }
    last
}

/// De que lado o teto de precisão morde.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Bound {
    /// Uma magnitude: só o teto (o piso é o do desenho — `0`, `0.1`, `1`).
    Up,
    /// Um deslocamento COM SINAL: o piso é o simétrico do teto, e a razão é a mesma.
    Both,
}

/// Os params cujo teto é a **precisão**, e não a lei: nada neles satura antes do `f32`.
///
/// ⚠️ **Uma entrada nesta lista é uma AFIRMAÇÃO sobre o nó** — *"este número não tem lei que o
/// limite"*. Dois nomes conhecidos ficam **fora de propósito**:
/// - `sim.spawn::rate` tem lei (`MAX_PER_TICK` por tique) e é ela quem manda;
/// - `force.*::strength` tem lei de **ESTABILIDADE**, medida no gate irmão `integrator_ceilings`
///   — uma força que faz o passo divergir não é honrada, é reposta pela guarda de finitude.
///
/// ⚠️ **A lista não foi escrita a olho: ela é o que a sonda
/// `what_the_corpus_authors_and_no_one_can_type` ACUSOU** (22 valores em 13 params, 2026-08-23),
/// mais o **irmão de nome** de cada acusado dentro da mesma família — `dx` sem `dy`,
/// `width` sem `height`, um canto de quatro sem os outros sete é precisamente a inconsistência
/// que fez este defeito ser invisível durante meses.
const PRECISION_BOUND: &[(&str, &str, Bound)] = &[
    // Os campos: o NEUTRO de cada um era inalcançável (o doc-comment promete-o, a UI recusa-o).
    ("field.box", "width", Bound::Up),
    ("field.box", "height", Bound::Up),
    ("field.radial_sweep", "radius", Bound::Up),
    ("field.radial_sweep", "inner_radius", Bound::Up),
    ("field.remap", "steps", Bound::Up),
    // Raios de mundo — a cena `=13` autora `320` num campo que digita até `20`.
    ("force.vortex", "radius", Bound::Up),
    ("force.attractor", "radius", Bound::Up),
    ("motion.spherize", "radius", Bound::Up),
    ("force.buoyancy", "depth", Bound::Up),
    ("motion.voronoi", "width", Bound::Up),
    ("motion.voronoi", "height", Bound::Up),
    // Tempo e magnitude de um oscilador — a cena `=12` autora um período de `14 s` sobre `8`.
    ("value.lfo", "period", Bound::Up),
    ("value.lfo", "amplitude", Bound::Up),
    ("value.lfo", "offset", Bound::Both),
    // Velocidade de nascimento — a fonte da `=5` sai a `22` num campo que para em `20`.
    ("motion.emitter", "speed", Bound::Up),
    // Deslocamentos com sinal.
    ("motion.move", "dx", Bound::Both),
    ("motion.move", "dy", Bound::Both),
    ("motion.four_point_warp", "tl_dx", Bound::Both),
    ("motion.four_point_warp", "tl_dy", Bound::Both),
    ("motion.four_point_warp", "tr_dx", Bound::Both),
    ("motion.four_point_warp", "tr_dy", Bound::Both),
    ("motion.four_point_warp", "br_dx", Bound::Both),
    ("motion.four_point_warp", "br_dy", Bound::Both),
    ("motion.four_point_warp", "bl_dx", Bound::Both),
    ("motion.four_point_warp", "bl_dy", Bound::Both),
];

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("os nos registram");
    reg
}

fn hint<'a>(reg: &'a NodeRegistry, node: &str, param: &str) -> &'a ParamUiHint {
    let id = reg
        .manifests()
        .find(|m| m.name == node)
        .unwrap_or_else(|| panic!("o no {node} existe"))
        .id;
    reg.param_ui(id)
        .unwrap_or(&[])
        .iter()
        .find(|h| h.param == param)
        .unwrap_or_else(|| panic!("{node}::{param} tem hint"))
}

/// **A MEDIÇÃO, impressa** — a tabela que o doc 91 cita, derivada aqui e em lado nenhum mais.
#[test]
fn measure_precision_ceilings() {
    let reg = registry();
    eprintln!(
        "{:<38} {:>6} {:>12} {:>14}  binado",
        "param", "step", "arrasto", "teto medido"
    );
    for (node, param, _) in PRECISION_BOUND {
        let h = hint(&reg, node, param);
        let c = step_ceiling(h.step);
        eprintln!(
            "{:<38} {:>6} {:>12} {c:>14.3}  2^{}",
            format!("{node}::{param}"),
            h.step,
            h.max,
            c.log2().ceil() as i32
        );
    }
    // O controle: um passo GRANDE tem teto MAIOR. Sem esta linha a medição acima podia estar a
    // devolver uma constante e a tabela leria igual.
    assert!(
        step_ceiling(1.0) > step_ceiling(0.1),
        "um passo maior sobrevive mais longe"
    );
    assert!(
        step_ceiling(0.1) > 1.0e6,
        "um passo de 0,1 vive muito para lá do arrasto"
    );
}

/// **TODO PARAM DESTA LISTA TEM O TETO QUE A MEDIÇÃO DEU.**
///
/// ⚠️ **As duas metades são obrigatórias.** Só *"existe um `ParamHardMax`"* passaria com o teto
/// escrito à mão a 40; só *"o número é o medido"* passaria com a entrada ausente, porque não há
/// entrada com que discordar.
#[test]
fn every_precision_bound_param_types_to_the_measured_ceiling() {
    let reg = registry();
    for (node, param, bound) in PRECISION_BOUND {
        let id = reg
            .manifests()
            .find(|m| m.name == *node)
            .unwrap_or_else(|| panic!("o no {node} existe"))
            .id;
        let h = hint(&reg, node, param);
        let hard = reg.param_hard_max(id, param).unwrap_or_else(|| {
            panic!("{node}::{param} nao tem ParamHardMax -- ele digita ate' o fim do ARRASTO")
        });
        let want = step_ceiling(h.step);
        assert_eq!(
            hard, want,
            "{node}::{param}: o teto digitavel tem de ser o MEDIDO ({want}), nao {hard}"
        );
        // E o arrasto continua onde a mão trabalha — subir o teto digitável não mexe no curso.
        assert!(
            h.max < hard,
            "{node}::{param}: o arrasto ({}) vive DENTRO do teto digitavel ({hard})",
            h.max
        );
        // ⚠️ **Um deslocamento com sinal precisa das DUAS pontas.** A sonda acusou
        // `motion.four_point_warp::br_dx = -40` num campo que digita `[-10, 10]`: um teto
        // generoso com o piso de ontem deixa metade do gesto inalcançável, e um gesto que só
        // funciona para um lado lê-se como bug do nó.
        if *bound == Bound::Both {
            let floor = reg.param_hard_min(id, param).unwrap_or_else(|| {
                panic!("{node}::{param} e' um deslocamento COM SINAL e nao tem ParamHardMin")
            });
            assert_eq!(
                floor, -want,
                "{node}::{param}: o piso digitavel e' o simetrico do teto ({}), nao {floor}",
                -want
            );
        }
    }
}

/// Quantos elementos o `sim.spawn` **de facto** põe no mundo em `secs` segundos a `rate`,
/// cozido ao relógio da casa.
///
/// ⚠️ **A cadência é `ph2d_core::time::DEFAULT_HZ` e não um `60.0` escrito aqui.** O teto do `rate` é
/// `MAX_PER_TICK × cadência`; medi-lo num relógio que o app não tem é exactamente o erro que
/// esta medição existe para apanhar.
fn spawned(reg: &NodeRegistry, rate: f32, secs: f64) -> usize {
    let dt = 1.0 / ph2d_core::time::DEFAULT_HZ;
    let mut g = Graph::new();
    let tpl = g.add_node("motion.grid");
    g.set_param(tpl, "rows", 1.0);
    g.set_param(tpl, "cols", 1.0);
    let sp = g.add_node("sim.spawn");
    g.set_param(sp, "rate", rate);
    g.connect(Edge {
        from: (tpl, 0),
        to: (sp, 0),
        delayed: false,
    })
    .expect("template alimenta o spawn");
    g.validate(reg).expect("o grafo e' valido");
    let mut cook = Cook::new();
    let mut total = 0usize;
    let ticks = (secs / dt).round() as u64;
    for k in 0..=ticks {
        let t = k as f64 * dt;
        total += cook.cook(&g, reg, sp, t).expect("coze")[0]
            .as_stream()
            .count();
        cook.advance_tick(&g, reg, t).expect("o quadro fecha");
    }
    total
}

/// **O TETO DO `rate` É O QUE A LEI HONRA — e a prova é uma MEDIÇÃO, não a constante.**
///
/// ⚠️ **Este param fica fora da lista [`PRECISION_BOUND`] de propósito**: o `f32` só desistiria
/// lá em cima, mas [`sim.spawn`] desiste muito antes — `born_in` grampeia a janela em
/// `first + MAX_PER_TICK`, e acima disso os nascimentos devidos são **saltados**, não adiados.
/// *Quando existe lei, é ela quem manda no teto, e não a representação.*
///
/// O gate lê o teto que o nó SHIPA e mede os dois lados dele: no teto todo nascimento devido
/// acontece; ao dobro, o nó entrega o mesmo — porque o excedente foi perdido.
#[test]
fn the_spawn_rate_ceiling_is_the_one_the_law_honours() {
    let reg = registry();
    let id = reg
        .manifests()
        .find(|m| m.name == "sim.spawn")
        .expect("o no existe")
        .id;
    let ceiling = reg
        .param_hard_max(id, "rate")
        .expect("o `rate` tem teto digitavel");
    let secs = 0.25;
    let at = spawned(&reg, ceiling, secs);
    let over = spawned(&reg, ceiling * 2.0, secs);
    let due = (f64::from(ceiling) * secs).round() as usize;
    eprintln!("rate {ceiling}/s por {secs}s: devidos {due} · entregues {at} · ao dobro {over}");
    // NO teto: o nó entrega o que era devido (a folga de um tique é o primeiro, que tem dt = 0).
    let slack = (f64::from(ceiling) / ph2d_core::time::DEFAULT_HZ).ceil() as usize;
    assert!(
        at + slack >= due && at <= due,
        "no teto ({ceiling}/s) todo nascimento devido acontece: {at} de {due}"
    );
    // ACIMA dele a lei satura — e é isso que torna o teto o número certo, e não um palpite.
    assert!(
        over < due * 2 - slack,
        "ao dobro do teto os nascimentos sao SALTADOS: {over} contra {} devidos",
        due * 2
    );
    assert_eq!(
        at, over,
        "e o que satura satura no MESMO numero: o excedente nao e' adiado, e' perdido"
    );
}
