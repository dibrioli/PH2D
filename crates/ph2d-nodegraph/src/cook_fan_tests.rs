//! Gates dos **LEQUES DE TEMPO** (`TimeFans`) — a porta 0 de um nó cozida em N
//! instantes em vez de uma vez.
//!
//! Irmão do `cook_scope_tests`: declarado no `cook_tests.rs` como `#[path]`, de
//! forma que `super` é o arnês (os nós de teste + `ops()`).
//!
//! ⚠️ **O oráculo é `test.clock`**, que emite o playhead em que foi puxado. É a
//! única sonda que responde *"em que instante esta sub-árvore foi lida?"* — uma
//! régua de contagem de linhas diria «três fatias» sobre três fatias todas do
//! mesmo instante, que é exactamente o defeito que a máquina pode ter.

use super::*;
use crate::cook::TimeFans;
use crate::time::TimeMap;

pub(super) static FAN_MAN: NodeManifest = NodeManifest {
    id: NodeTypeId::of("test.fan"),
    name: "test.fan",
    inputs: &[port("in")],
    outputs: &[port("out")],
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[],
    lowerings: &[LoweringKind::Cpu],
};

/// **A sonda do LEQUE** (`TimeFans`): emite uma linha por fatia, carregando o
/// valor que aquela fatia trouxe — ou seja, o instante em que a sub-árvore de
/// cima foi cozida.
///
/// ⚠️ Com leque VAZIO ele emite a porta 0 tal e qual, que é o controle: um nó
/// sem leque tem de sair byte-idêntico ao que sairia sem a máquina toda.
pub(super) struct FanNode;
impl NodeOp for FanNode {
    fn manifest(&self) -> &'static NodeManifest {
        &FAN_MAN
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        if ctx.fan_len() == 0 {
            let passthrough = ctx.input(0).clone();
            ctx.emit(passthrough);
            return;
        }
        let vals: Vec<f32> = (0..ctx.fan_len())
            .map(|k| match ctx.fan(k).get("v") {
                Some(Column::Scalar(v)) => v.first().copied().unwrap_or(f32::NAN),
                _ => f32::NAN,
            })
            .collect();
        ctx.emit(Stream::new(vals.len()).with("v", Column::Scalar(vals)));
    }
}

/// Um mapa que só desloca: `t' = t + offset`.
fn shift(offset: f64) -> TimeMap {
    TimeMap {
        offset,
        ..TimeMap::default()
    }
}

/// `clock -> fan`. O leque tem de trazer a MESMA sub-árvore lida em instantes
/// diferentes.
#[test]
fn a_fan_cooks_the_same_input_at_every_instant_it_names() {
    let mut g = Graph::new();
    let clock = g.add_node("test.clock");
    let fan = g.add_node("test.fan");
    g.connect(Edge {
        from: (clock, 0),
        to: (fan, 0),
        delayed: false,
    })
    .unwrap();
    let o = ops();
    let mut cook = Cook::new();

    let fans: TimeFans = [(fan, vec![shift(-2.0), shift(-1.0), TimeMap::default()])].into();
    let out = cook
        .cook_scoped_fanned(&g, &o, fan, 10.0, &TimeScopes::new(), &fans)
        .unwrap();
    assert_eq!(
        out_scalars(&out[0]),
        vec![8.0, 9.0, 10.0],
        "o leque tem de ler o passado, o passado recente e o agora"
    );

    // ⭐ O FUTURO — a coisa que um ring de estado não pode conter por construção.
    let ahead: TimeFans = [(fan, vec![shift(1.0), shift(3.0)])].into();
    let out = cook
        .cook_scoped_fanned(&g, &o, fan, 10.0, &TimeScopes::new(), &ahead)
        .unwrap();
    assert_eq!(out_scalars(&out[0]), vec![11.0, 13.0], "eco para a FRENTE");
}

/// **Um leque VAZIO é exactamente a cozedura de sempre** — a redução que faz
/// desta máquina um ponto de extensão em vez de um segundo motor.
#[test]
fn an_empty_fan_is_the_cook_that_shipped() {
    let mut g = Graph::new();
    let clock = g.add_node("test.clock");
    let fan = g.add_node("test.fan");
    g.connect(Edge {
        from: (clock, 0),
        to: (fan, 0),
        delayed: false,
    })
    .unwrap();
    let o = ops();

    let mut a = Cook::new();
    let plain = out_scalars(&a.cook(&g, &o, fan, 4.0).unwrap()[0]);
    let mut b = Cook::new();
    let empty = out_scalars(
        &b.cook_scoped_fanned(&g, &o, fan, 4.0, &TimeScopes::new(), &TimeFans::new())
            .unwrap()[0],
    );
    assert_eq!(plain, empty, "o leque vazio mudou a cozedura");
    assert_eq!(plain, vec![4.0], "e ela continua a ser o passe da porta 0");
}

/// ⚠️ **Duas fatias que pedem o MESMO instante partilham a faixa de memo** — é o
/// que impede um `length` grande de custar `length` cozeduras da sub-árvore
/// inteira quando os instantes coincidem. Contado, não presumido.
#[test]
fn two_slices_at_the_same_instant_cost_one_cook() {
    let mut g = Graph::new();
    let clock = g.add_node("test.clock");
    let fan = g.add_node("test.fan");
    g.connect(Edge {
        from: (clock, 0),
        to: (fan, 0),
        delayed: false,
    })
    .unwrap();
    let o = ops();
    let mut cook = Cook::new();

    let repeated: TimeFans = [(fan, vec![shift(-1.0), shift(-1.0), shift(-1.0)])].into();
    let out = cook
        .cook_scoped_fanned(&g, &o, fan, 5.0, &TimeScopes::new(), &repeated)
        .unwrap();
    assert_eq!(out_scalars(&out[0]), vec![4.0, 4.0, 4.0]);
    let same = o.clock.calls.load(std::sync::atomic::Ordering::Relaxed);

    let distinct: TimeFans = [(fan, vec![shift(-1.0), shift(-2.0), shift(-3.0)])].into();
    let mut cook2 = Cook::new();
    let o2 = ops();
    cook2
        .cook_scoped_fanned(&g, &o2, fan, 5.0, &TimeScopes::new(), &distinct)
        .unwrap();
    let three = o2.clock.calls.load(std::sync::atomic::Ordering::Relaxed);

    // ⚠️ **A conta inclui a porta 0 no AGORA**, que o nó recebe na mesma: o leque
    // ACRESCENTA fatias, não substitui a entrada. Num rastro re-cozido isso não
    // custa nada — a geração 0 é a identidade e partilha a faixa —, e num leque
    // que só olha para a frente é uma cozedura a mais, na faixa que o resto do
    // grafo já pediu de qualquer forma.
    assert_eq!(
        same, 2,
        "a porta 0 mais UMA cozedura partilhada pelas tres fatias"
    );
    assert_eq!(
        three, 4,
        "CONTROLE: a porta 0 mais tres instantes distintos"
    );
}

/// **O leque entra na impressão digital do nó.** Mudar os mapas tem de
/// recomputar — senão arrastar o `length` de um rastro re-cozido devolveria a
/// cauda antiga do memo.
#[test]
fn changing_the_fan_recomputes_the_node() {
    let mut g = Graph::new();
    let clock = g.add_node("test.clock");
    let fan = g.add_node("test.fan");
    g.connect(Edge {
        from: (clock, 0),
        to: (fan, 0),
        delayed: false,
    })
    .unwrap();
    let o = ops();
    let mut cook = Cook::new();

    let two: TimeFans = [(fan, vec![shift(-1.0), TimeMap::default()])].into();
    let a = out_scalars(
        &cook
            .cook_scoped_fanned(&g, &o, fan, 6.0, &TimeScopes::new(), &two)
            .unwrap()[0],
    );
    let three: TimeFans = [(fan, vec![shift(-2.0), shift(-1.0), TimeMap::default()])].into();
    let b = out_scalars(
        &cook
            .cook_scoped_fanned(&g, &o, fan, 6.0, &TimeScopes::new(), &three)
            .unwrap()[0],
    );
    assert_eq!(a, vec![5.0, 6.0]);
    assert_eq!(b, vec![4.0, 5.0, 6.0], "o memo devolveu a cauda antiga");
}

/// ⚠️ **Sem aresta na porta 0, as FATIAS continuam a existir e o stream de cada
/// uma é VAZIO** — e a distinção custou um defeito.
///
/// A primeira versão deste gate afirmava que *"um nó sem aresta na porta 0 não
/// ganha leque nenhum"*, e o código fazia isso: só empurrava uma entrada quando
/// havia aresta. O resultado é que `fan_len()` contava as fatias da **PORTA**, e
/// um nó SEM portas — que é toda FONTE — lia **zero** fatias com o leque montado
/// e cheio. Medido no produto: o `motion.emitter` ignorava **529 amostras** da
/// própria história, em silêncio.
///
/// A lei que fica: **uma entrada por fatia, sempre**; o que falta é o *conteúdo*
/// da porta, não a fatia.
#[test]
fn a_node_with_no_input_port_still_gets_its_slices() {
    let mut g = Graph::new();
    let fan = g.add_node("test.fan");
    let o = ops();
    let mut cook = Cook::new();
    let fans: TimeFans = [(fan, vec![shift(-1.0), shift(-2.0)])].into();
    let out = cook
        .cook_scoped_fanned(&g, &o, fan, 3.0, &TimeScopes::new(), &fans)
        .unwrap();
    // O `test.fan` emite uma linha por fatia, lendo `v` de cada uma — todas
    // vazias aqui, logo `NaN`. O que importa é a CONTAGEM: duas fatias.
    assert_eq!(
        out_scalars(&out[0]).len(),
        2,
        "as fatias existem mesmo sem porta 0 — o que falta e' o conteudo delas"
    );
}

/// **UM LEQUE AO LADO DE UM NÓ SEQUENCIAL não estraga a fotografia do `pre`.**
///
/// O `advance_tick` fotografa as saídas da faixa RAIZ para as arestas `pre` do
/// tique seguinte, e o leque coze a mesma sub-árvore no passado. Este gate diz
/// que um rastro LEMBRADO ao lado de um RE-COZIDO continua a lembrar do que viu,
/// e não de uma fatia.
///
/// ⚠️ **E ele NÃO é o gate do `push_scope`** — ver
/// [`repeating_an_instant_out_of_order_still_hits_the_memo`], que é. Foi escrito
/// a perseguir uma mutação sobrevivente e mediu outra coisa; fica porque o
/// cenário (sequencial + leque no mesmo grafo) é o que o produto de facto monta,
/// e ninguém mais o cobre.
#[test]
fn a_fan_slice_never_clobbers_the_root_lane_the_pre_snapshot_reads() {
    let mut g = Graph::new();
    let clock = g.add_node("test.clock");
    let fan = g.add_node("test.fan");
    let acc = g.add_node("test.acc");
    // O relógio alimenta os dois: o leque (que o lê no passado) e um acumulador
    // sequencial (que o lê AGORA e guarda o resultado para o tique seguinte).
    for (to, port) in [(fan, 0u16), (acc, 0)] {
        g.connect(Edge {
            from: (clock, 0),
            to: (to, port),
            delayed: false,
        })
        .unwrap();
    }
    g.connect(Edge {
        from: (acc, 0),
        to: (acc, 1),
        delayed: true,
    })
    .unwrap();

    let o = ops();
    let mut cook = Cook::new();
    let fans: TimeFans = [(fan, vec![shift(-10.0), shift(-20.0)])].into();

    // Um tique: coze o leque PRIMEIRO (ele empurra o relógio para o passado), o
    // acumulador depois, e fecha o quadro.
    cook.cook_scoped_fanned(&g, &o, fan, 100.0, &TimeScopes::new(), &fans)
        .unwrap();
    let now = out_scalars(
        &cook
            .cook_scoped_fanned(&g, &o, acc, 100.0, &TimeScopes::new(), &fans)
            .unwrap()[0],
    );
    assert_eq!(now, vec![100.0], "o acumulador le^ o AGORA, nao uma fatia");
    cook.advance_tick_fanned(&g, &o, 100.0, &TimeScopes::new(), &fans)
        .unwrap();

    // O tique seguinte soma a fotografia. `100 + 101` — e nunca `80 + 101`.
    let next = out_scalars(
        &cook
            .cook_scoped_fanned(&g, &o, acc, 101.0, &TimeScopes::new(), &fans)
            .unwrap()[0],
    );
    assert_eq!(
        next,
        vec![201.0],
        "a fotografia do `pre` levou o instante de uma FATIA para o tique seguinte"
    );
}

/// ⭐⭐ **O QUE O `push_scope` DE FACTO COMPRA** — e a medição corrigiu a afirmação
/// antes do código.
///
/// O doc-comment do [`crate::cook::TimeFans`] dizia que fatias no mesmo instante
/// «partilham a faixa e o custo». **Uma mutação provou que era grande demais:**
/// trocar `push_scope(in_key, node, map)` por `in_key` deixava SEIS gates verdes,
/// porque dentro do laço cada leitura segue a própria cozedura — os valores saem
/// certos de qualquer maneira, e duas fatias ADJACENTES no mesmo instante batem
/// no memo mesmo partilhando a faixa.
///
/// O que a faixa própria compra é o instante repetido **fora de ordem**: com
/// faixas distintas, pedir `t−1` depois de `t−2` responde do memo; com uma faixa
/// só, a segunda passagem por `t−2` já foi despejada e recomputa. É o caso de um
/// espaçamento NÃO-UNIFORME (o eco adensado perto da cabeça), que é justamente
/// para onde esta máquina foi construída.
///
/// ⚠️ *Uma afirmação que nenhuma mutação mata é uma afirmação sobre nada* — e a
/// cura foi **encolher a afirmação até ao que a máquina faz**, não inventar um
/// gate para a versão grande.
#[test]
fn repeating_an_instant_out_of_order_still_hits_the_memo() {
    let mut g = Graph::new();
    let clock = g.add_node("test.clock");
    let fan = g.add_node("test.fan");
    g.connect(Edge {
        from: (clock, 0),
        to: (fan, 0),
        delayed: false,
    })
    .unwrap();
    let o = ops();
    let mut cook = Cook::new();

    // `−1, −2, −1`: a terceira fatia repete a primeira, com outra pelo meio.
    let zigzag: TimeFans = [(fan, vec![shift(-1.0), shift(-2.0), shift(-1.0)])].into();
    let out = cook
        .cook_scoped_fanned(&g, &o, fan, 9.0, &TimeScopes::new(), &zigzag)
        .unwrap();
    assert_eq!(out_scalars(&out[0]), vec![8.0, 7.0, 8.0], "os valores");
    let calls = o.clock.calls.load(std::sync::atomic::Ordering::Relaxed);
    assert_eq!(
        calls, 3,
        "a porta 0 no agora, `t−1` e `t−2` — a REPETIÇÃO tem de sair do memo"
    );
}

/// **SONDA: quanto custa uma FATIA de leque?** — o número que decide quantas
/// amostras um nó pode pedir por quadro.
///
/// `cargo test -p ph2d-nodegraph -- --ignored --nocapture custo_de_uma_fatia`
#[test]
#[ignore = "sonda de medição, não gate"]
fn custo_de_uma_fatia() {
    let mut g = Graph::new();
    let clock = g.add_node("test.clock");
    let fan = g.add_node("test.fan");
    g.connect(Edge {
        from: (clock, 0),
        to: (fan, 0),
        delayed: false,
    })
    .unwrap();
    let o = ops();
    eprintln!("\n=== CUSTO DE UM LEQUE (uma sub-árvore escalar, uma fatia = uma cozedura) ===");
    for n in [64usize, 256, 512, 1024, 2048, 4096] {
        let maps: Vec<TimeMap> = (0..n).map(|k| shift(-(k as f64) * 0.001)).collect();
        let fans: TimeFans = [(fan, maps)].into();
        let mut cook = Cook::new();
        const REPS: u32 = 20;
        let t0 = std::time::Instant::now();
        for r in 0..REPS {
            cook.cook_scoped_fanned(&g, &o, fan, 100.0 + f64::from(r), &TimeScopes::new(), &fans)
                .unwrap();
        }
        let ms = t0.elapsed().as_secs_f64() * 1e3 / f64::from(REPS);
        eprintln!(
            "  {n:>5} fatias: {ms:>7.3} ms/quadro  ({:>5.1}% de um quadro de 16,7)  \
             {:.0} ns/fatia",
            ms / 16.7 * 100.0,
            ms * 1e6 / n as f64
        );
    }
    eprintln!();
}

/// ⭐ **O LEQUE ALCANÇA UM NÓ SEM PORTAS DE ENTRADA** — a metade que faltava, e sem
/// a qual ele não serve fonte nenhuma.
///
/// O `motion.emitter` tem `inputs: &[]` e a origem dele é um **param dirigido**.
/// Um leque que só soubesse ler portas nunca lhe daria a própria história.
#[test]
fn a_fan_carries_the_driven_params_of_a_node_with_no_input_ports() {
    let mut g = Graph::new();
    let clock = g.add_node("test.clock");
    // `test.gen` não tem portas de entrada; o `scale` dele é dirigido pelo relógio.
    let sink = g.add_node("test.gen");
    g.drive_param(sink, "scale", (clock, 0)).unwrap();
    let o = ops();
    let mut cook = Cook::new();

    let fans: TimeFans = [(sink, vec![shift(-2.0), shift(-1.0), TimeMap::default()])].into();
    cook.cook_scoped_fanned(&g, &o, sink, 20.0, &TimeScopes::new(), &fans)
        .unwrap();
    let calls = o.clock.calls.load(std::sync::atomic::Ordering::Relaxed);
    assert_eq!(
        calls, 3,
        "o driver tinha de cozer nos TRES instantes do leque (e nao ha' porta 0 a pagar um quarto)"
    );
    // ⚠️ **E o nó tem de VER as três**, o que é outra afirmação: a primeira versão
    // deste gate só contava cozeduras, e passava enquanto o `fan_len()` devolvia
    // ZERO — ele contava as fatias da PORTA, e este nó não tem porta nenhuma.
    // Medido: o `motion.emitter` ignorava 529 amostras da própria história em
    // silêncio, com este gate verde. *Contar o trabalho feito não é contar o
    // trabalho ENTREGUE.*
    let seen = o
        .generator
        .fan_seen
        .load(std::sync::atomic::Ordering::Relaxed);
    assert_eq!(seen, 3, "o no' viu {seen} fatias, e o leque tinha 3");
}
