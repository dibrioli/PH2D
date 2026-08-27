//! Gates do **CICLO DE VIDA** da zona (doc 89, folha 13, célula 60).
//!
//! ⚠️ Arquivo próprio por teto de LOC (HR-18): o `tests.rs` já mede 239 linhas.
//!
//! A lei é uma função pura do relógio, então quase tudo aqui é uma tabela de instantes — e é
//! isso que a torna testável sem cozinhar um grafo. O gate que carrega num cook de verdade é
//! o da **byte-identidade**, porque é o único que pode dizer *"nenhum documento de hoje muda"*.

use super::life::{Emit, Life, MIN_DURATION, Mode};

/// Um ciclo com números redondos: começa em `1`, corre `2`, descansa `1`.
fn cycled(mode: Mode) -> Life {
    Life {
        start: 1.0,
        duration: 2.0,
        rest: 1.0,
        mode,
    }
}

/// **O DEFAULT é a zona de sempre, e a pergunta é UMA.** É o `is_default` que corta a
/// maquinaria fora do caminho — sem ele, a byte-identidade seria uma promessa.
#[test]
fn the_default_is_the_zone_that_shipped() {
    let d = Life {
        start: 0.0,
        duration: 2.0,
        rest: 0.0,
        mode: Mode::Forever,
    };
    assert!(d.is_default(), "os defaults do manifesto");
    // ⚠️ E a duração NÃO participa da resposta em `Forever`: ela existe no manifesto com um
    // valor útil (2 s) para o artista que troca de modo não cair num slider a zero.
    assert!(
        Life {
            duration: 999.0,
            ..d
        }
        .is_default()
    );
    // Qualquer uma das duas metades tira a zona do caminho de sempre.
    assert!(!Life { start: 0.5, ..d }.is_default(), "um atraso e' ciclo");
    assert!(
        !Life {
            mode: Mode::Once,
            ..d
        }
        .is_default(),
        "um fim e' ciclo"
    );
}

/// ⭐ **ADIAR** — antes do `start` a zona não emite nada, e no instante dele SEMEIA.
#[test]
fn nothing_exists_before_the_start_and_the_first_running_tick_seeds() {
    let l = cycled(Mode::Once);
    let dt = 1.0 / 60.0;
    assert_eq!(
        l.emit(0.0, dt, true),
        Emit::Nothing,
        "t=0 e' antes de start=1"
    );
    assert_eq!(l.emit(0.99, dt, true), Emit::Nothing);
    // O primeiro tique dentro da janela relê o `init`…
    assert_eq!(l.emit(1.0, dt, true), Emit::Seed);
    // …e o seguinte continua do estado.
    assert_eq!(l.emit(1.0 + dt, dt, true), Emit::Carry);
}

/// ⭐⭐ **REINICIAR** — em `Loop`, cada ciclo volta a semear, e entre eles não há nada.
#[test]
fn every_cycle_seeds_again_and_the_rest_is_empty() {
    let l = cycled(Mode::Loop);
    let dt = 1.0 / 60.0;
    // Ciclo 1: `[1, 3)` a correr, `[3, 4)` a descansar.
    assert_eq!(l.emit(1.0, dt, true), Emit::Seed);
    assert_eq!(l.emit(2.5, dt, true), Emit::Carry);
    assert_eq!(l.emit(3.5, dt, true), Emit::Nothing, "o descanso");
    // Ciclo 2 começa em `4` — e SEMEIA outra vez, que é a coisa toda.
    assert_eq!(l.emit(4.0, dt, true), Emit::Seed);
    assert_eq!(l.emit(5.0, dt, true), Emit::Carry);
    assert_eq!(l.emit(6.5, dt, true), Emit::Nothing);
    assert_eq!(l.emit(7.0, dt, true), Emit::Seed, "o terceiro ciclo");
}

/// **ACABAR** — em `Once` a janela fecha e não reabre, por longe que se vá.
#[test]
fn once_ends_and_never_comes_back() {
    let l = cycled(Mode::Once);
    let dt = 1.0 / 60.0;
    assert_eq!(l.emit(2.9, dt, true), Emit::Carry);
    for t in [3.0, 5.0, 60.0, 3600.0] {
        assert_eq!(l.emit(t, dt, true), Emit::Nothing, "t = {t}");
    }
}

/// ⚠️⚠️ **O `dt = 0` é o caso que obriga o `started` a ficar.** No primeiro tique de um cook
/// não há tique anterior, então a aresta de fase compara o instante consigo próprio e **não
/// pode disparar** — sem o `started` a sim nunca semearia.
#[test]
fn the_first_tick_of_a_cook_seeds_even_with_a_zero_delta() {
    let l = cycled(Mode::Loop);
    // A aresta de fase é cega aqui: `phase(t) == phase(t - 0)`.
    assert_eq!(l.phase(2.0), l.phase(2.0 - 0.0));
    // E mesmo assim o meio de um ciclo semeia, porque a zona nunca emitiu.
    assert_eq!(l.emit(2.0, 0.0, false), Emit::Seed);
    // O controle: com `started`, o meio de um ciclo continua do estado.
    assert_eq!(l.emit(2.0, 0.0, true), Emit::Carry);
}

/// **Uma duração não-positiva é COAGIDA, não interpretada.** Sem o piso, `Once` com `0` daria
/// `Ended` no primeiro tique (a sim nunca visível) e `Loop` dividiria por um ciclo nulo.
#[test]
fn a_non_positive_duration_is_floored_at_the_door() {
    for mode in [Mode::Once, Mode::Loop] {
        let l = Life {
            start: 0.0,
            duration: 0.0,
            rest: 0.0,
            mode,
        };
        // Dentro do piso ainda corre…
        assert_eq!(l.emit(0.0, 0.0, false), Emit::Seed, "{mode:?}");
        assert_eq!(
            l.emit(MIN_DURATION * 0.5, 1e-4, true),
            Emit::Carry,
            "{mode:?}"
        );
        // …e nada explode no fim.
        assert!(matches!(
            l.emit(1.0, 1e-4, true),
            Emit::Nothing | Emit::Seed | Emit::Carry
        ));
    }
    // E uma duração NEGATIVA (um fio pode entregá-la) não parte nada.
    let l = Life {
        start: 0.0,
        duration: -5.0,
        rest: -5.0,
        mode: Mode::Loop,
    };
    for t in [0.0, 0.5, 10.0] {
        let _ = l.emit(t, 1.0 / 60.0, true);
    }
}

/// **`Forever` ignora a duração e o descanso** — é literalmente a zona de sempre, e é por isso
/// que os dois knobs ficam escondidos nesse modo.
///
/// ⚠️ A fixtura tem `start > 0`: com `start = 0` e `Forever` o `is_default` corta a maquinaria
/// fora do caminho, e o `emit` **nunca é chamado** — medir ali seria medir código inalcançável.
#[test]
fn forever_reads_neither_the_duration_nor_the_rest() {
    let a = Life {
        start: 1.0,
        duration: 2.0,
        rest: 1.0,
        mode: Mode::Forever,
    };
    let b = Life {
        duration: 900.0,
        rest: 900.0,
        ..a
    };
    let dt = 1.0 / 60.0;
    for t in [1.5, 5.0, 100.0] {
        assert_eq!(a.emit(t, dt, true), b.emit(t, dt, true), "t = {t}");
        assert_eq!(a.emit(t, dt, true), Emit::Carry, "t = {t}");
    }
    // E antes do `start` as duas continuam caladas, também de acordo.
    assert_eq!(a.emit(0.5, dt, true), Emit::Nothing);
    assert_eq!(b.emit(0.5, dt, true), Emit::Nothing);
}

/// ⚠️ **A fase é uma função PURA do relógio** — o mesmo instante dá a mesma resposta, venha
/// ele de uma reprodução ou de um *scrub* para trás. É a mesma disciplina que faz o id do
/// `sim.spawn` ser `floor(rate·t)` e não um contador (cerca 6 da folha 13).
#[test]
fn the_phase_is_a_pure_function_of_the_clock() {
    let l = cycled(Mode::Loop);
    let forwards: Vec<_> = (0..400).map(|k| l.phase(f64::from(k) / 60.0)).collect();
    let backwards: Vec<_> = (0..400)
        .rev()
        .map(|k| l.phase(f64::from(k) / 60.0))
        .collect();
    assert_eq!(
        forwards,
        backwards.into_iter().rev().collect::<Vec<_>>(),
        "um scrub para tras tem de dar a mesma fase"
    );
}

// ---------------------------------------------------------------------------
// A COSTURA — o ciclo de vida através de um cook DE VERDADE
// ---------------------------------------------------------------------------

use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("registry builds");
    reg
}

fn wire(g: &mut Graph, from: NodeId, fp: u16, to: NodeId, tp: u16, delayed: bool) {
    g.connect(Edge {
        from: (from, fp),
        to: (to, tp),
        delayed,
    })
    .expect("wire");
}

/// Uma zona com 5 peças no `init` e gravidade dentro — a fixtura da sonda
/// `measure_zone_life_cycle`, que é onde o buraco foi medido.
fn falling(params: &[(&str, f32)]) -> (Graph, NodeId) {
    let mut g = Graph::new();
    let grid = g.add_node("motion.grid");
    g.set_param(grid, "rows", 1.0);
    g.set_param(grid, "cols", 5.0);
    g.set_param(grid, "gap_x", 0.4);
    let zone = g.add_node("sim.zone");
    for (k, v) in params {
        g.set_param(zone, *k, *v);
    }
    wire(&mut g, grid, 0, zone, 0, false);
    // ⚠️ Não há `force.gravity` neste catálogo — a gravidade é o `force.wind` para baixo.
    let grav = g.add_node("force.wind");
    g.set_param(grav, "angle", 270.0);
    g.set_param(grav, "strength", 4.0);
    g.set_param(grav, "gust", 0.0);
    wire(&mut g, zone, 0, grav, 0, true);
    let step = g.add_node("sim.step");
    g.set_param(step, "damping", 1.0);
    wire(&mut g, grav, 0, step, 0, false);
    wire(&mut g, step, 0, zone, 1, false);
    (g, zone)
}

/// `(contagem, y medio)` em cada tique de `0..ticks`.
fn run(g: &Graph, reg: &NodeRegistry, sink: NodeId, ticks: usize) -> Vec<(usize, f32)> {
    let mut cook = Cook::new();
    let mut out = Vec::with_capacity(ticks);
    for k in 0..ticks {
        let t = k as f64 / 60.0;
        let v = cook.cook(g, reg, sink, t).expect("coza");
        let s = v[0].as_stream();
        out.push(match s.get("P") {
            Some(Column::Vec2(p)) if !p.is_empty() => (
                p.len(),
                p.iter().map(|q| q[1]).sum::<f32>() / p.len() as f32,
            ),
            _ => (0, 0.0),
        });
        cook.advance_tick(g, reg, t).expect("avanca");
    }
    out
}

/// ⭐⭐⭐ **BYTE-IDENTIDADE: nenhum documento de hoje muda.** Escrever os defaults tem de dar
/// exactamente o mesmo que não os escrever — é a única afirmação que protege as cenas salvas.
#[test]
fn the_defaults_cook_byte_identically_to_the_zone_that_shipped() {
    let reg = registry();
    let (a, za) = falling(&[]);
    let (b, zb) = falling(&[("start", 0.0), ("mode", 0.0), ("duration", 2.0)]);
    assert_eq!(
        run(&a, &reg, za, 240),
        run(&b, &reg, zb, 240),
        "os defaults nao podem mover um bit"
    );
    // E a fixtura CONTÉM o fenómeno: ela de facto simula (senão a igualdade seria vácua).
    let end = *run(&a, &reg, za, 240).last().expect("tiques");
    assert!(end.0 == 5 && end.1 < -20.0, "a sim tem de correr: {end:?}");
}

/// ⭐ **ADIAR, através do cook.** Antes do `start` a zona não desenha nada; depois, cai.
#[test]
fn a_started_zone_draws_nothing_until_its_clock_says_so() {
    let reg = registry();
    let (g, z) = falling(&[("start", 1.0), ("mode", 0.0)]);
    let rows = run(&g, &reg, z, 180);
    for (k, (n, _)) in rows.iter().enumerate().take(59) {
        assert_eq!(*n, 0, "tique {k} e' antes do start");
    }
    assert_eq!(rows[60].0, 5, "no start a fileira aparece inteira");
    // E a partir dali ela CAI — sem isto, «apareceu» podia ser um congelamento.
    assert!(rows[179].1 < -1.0, "y no fim = {}", rows[179].1);
    // ⚠️ O CONTROLE: a mesma cena sem `start` já está a cair no mesmo tique.
    let (g0, z0) = falling(&[]);
    let base = run(&g0, &reg, z0, 180);
    assert!(
        base[60].1 < rows[60].1,
        "o controle ja' caiu: {:?}",
        base[60]
    );
}

/// ⭐⭐ **REINICIAR, através do cook** — a coisa que a sonda mostrou ser inexprimível.
#[test]
fn a_looping_zone_returns_to_its_seed_every_cycle() {
    let reg = registry();
    // Corre 1 s, descansa 0,5 s, repete.
    let (g, z) = falling(&[("mode", 2.0), ("duration", 1.0), ("loop_delay", 0.5)]);
    let rows = run(&g, &reg, z, 300);
    // Fim do 1.º ciclo: caiu.
    assert!(rows[59].1 < -1.0, "1o ciclo caiu: {}", rows[59].1);
    // O descanso: nada.
    assert_eq!(rows[70].0, 0, "o descanso e' vazio");
    // O 2.º ciclo REcomeça do zero — é isto que nenhuma composição alcançava.
    assert_eq!(rows[90].0, 5, "o 2o ciclo repovoa");
    assert!(
        rows[90].1 > rows[59].1,
        "o 2o ciclo tem de VOLTAR para cima: {} contra {}",
        rows[90].1,
        rows[59].1
    );
    // E ele volta a cair — um ciclo que semeia e congela não é um ciclo.
    assert!(rows[145].1 < rows[90].1, "e volta a cair");
    // ⚠️ **A prova de que ele repete de facto:** o 3.º ciclo repete o 2.º, tique a tique.
    let cycle = 90usize; // 1,5 s a 60 fps
    for k in 0..50 {
        let (a, b) = (rows[90 + k], rows[90 + cycle + k]);
        assert_eq!(a.0, b.0, "contagem no offset {k}");
        assert!(
            (a.1 - b.1).abs() < 1e-4,
            "y no offset {k}: {} vs {}",
            a.1,
            b.1
        );
    }
}

/// **ACABAR, através do cook** — em `Once` a zona apaga-se e não volta.
#[test]
fn a_once_zone_goes_quiet_and_stays_quiet() {
    let reg = registry();
    let (g, z) = falling(&[("mode", 1.0), ("duration", 1.0)]);
    let rows = run(&g, &reg, z, 300);
    assert_eq!(rows[30].0, 5, "durante, ela existe");
    for (k, (n, _)) in rows.iter().enumerate().skip(61) {
        assert_eq!(*n, 0, "tique {k} e' depois do fim");
    }
}

/// ⚠️ **O DEVICE recua, e o gate afirma AS DUAS metades.** Com os defaults a zona continua
/// reivindicável (residente na GPU); com o ciclo ligado ela deixa de o ser, e o preço está
/// escrito no [`super::ZONE_KERNEL`].
#[test]
fn the_device_claims_the_default_zone_and_retreats_from_a_cycled_one() {
    let applicable = super::ZONE_KERNEL.applicable.expect("o kernel declara-o");
    let with = |mode: f32, start: f32| {
        applicable(&|name: &str| match name {
            "mode" => mode,
            "start" => start,
            _ => 0.0,
        })
    };
    assert!(with(0.0, 0.0), "a zona de sempre continua no device");
    assert!(!with(1.0, 0.0), "`Once` recua");
    assert!(!with(2.0, 0.0), "`Loop` recua");
    assert!(!with(0.0, 1.0), "um atraso recua");
}

/// **O painel só oferece o que o modo LÊ** — e o gate mede as duas metades, senão uma lista
/// vazia passaria por vácuo.
#[test]
fn the_panel_hides_the_knobs_the_chosen_mode_never_reads() {
    let by = |p: &str| {
        super::PARAM_GATES
            .iter()
            .find(|g| g.param == p)
            .unwrap_or_else(|| panic!("o gate de `{p}`"))
    };
    assert_eq!(by("duration").when, "mode");
    assert_eq!(by("duration").values, &[1, 2], "duracao em Once e Loop");
    assert_eq!(by("loop_delay").values, &[2], "descanso so' em Loop");
    // O CONTROLE: `start` NÃO é gateado — ele vale nos três modos.
    assert!(
        !super::PARAM_GATES.iter().any(|g| g.param == "start"),
        "o `start` vale sempre"
    );
    // ⚠️ O piso do slider é DERIVADO do piso da lei, então este gate mede a derivação e não
    // dois literais: a folga que sobra é a precisão do `f32` (~2e-10), não uma discordância.
    let d = super::PARAM_HINTS
        .iter()
        .find(|h| h.param == "duration")
        .expect("a linha da duracao");
    assert!(
        (f64::from(d.min) - MIN_DURATION).abs() < 1e-8,
        "o piso do slider ({}) e o da lei ({MIN_DURATION}) tem de ser o mesmo",
        d.min
    );
    assert!(
        d.min > 0.0,
        "e ele e' positivo -- um slider a zero reabre o caso coagido"
    );
}

/// ⚠️⚠️ **O QUE O CURTO-CIRCUITO DO DEFAULT DE FACTO COMPRA — e a mutação teve de mo dizer.**
///
/// A 1.ª redacção do `eval` afirmava, em prosa, que com os defaults *"a maquinaria do ciclo não
/// corre"*, e **nenhuma mutação conseguiu matar essa afirmação**: desligar o curto-circuito
/// deixava a suíte inteira verde, porque para `t >= 0` a lei geral concorda com o ramo de
/// sempre, tique a tique. *Uma afirmação que mutação nenhuma mata é uma afirmação sobre nada.*
///
/// O que ele compra de facto é isto: **a zona de sempre não depende do relógio.** Num instante
/// NEGATIVO a lei do ciclo diria `Dormant` (nada existe antes do `start = 0`), e a zona que
/// shipava semeia — ela nunca perguntou as horas. É a diferença inteira, e agora é falsificável.
#[test]
fn the_default_zone_does_not_ask_the_clock_what_time_it_is() {
    let reg = registry();
    let (g, z) = falling(&[]);
    let mut cook = Cook::new();
    // Um relógio ANTES do zero — o único sítio onde os dois ramos discordam.
    let out = cook.cook(&g, &reg, z, -1.0).expect("coza");
    assert_eq!(
        out[0].as_stream().count(),
        5,
        "a zona de sempre semeia seja qual for o relogio"
    );
    // O CONTROLE: com o ciclo LIGADO, o mesmo instante está de facto antes do começo.
    let (gc, zc) = falling(&[("start", 1.0)]);
    let mut cook = Cook::new();
    let out = cook.cook(&gc, &reg, zc, -1.0).expect("coza");
    assert_eq!(out[0].as_stream().count(), 0, "com ciclo, -1 s e' Dormant");
}
