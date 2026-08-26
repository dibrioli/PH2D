//! Os gates deste nó — cortados do `lib.rs` no teto de LOC (HR-18) pela costura que o
//! irmão `motion.drive` já tinha (`src/tests.rs`).
//!
//! ⚠️ O corte é por RESPONSABILIDADE e não por tamanho: o `lib.rs` responde *o que um ruído
//! É* e este arquivo *o que se afirma sobre ele*. E o nome não é decoração — o gate de LOC
//! exclui `**/tests.rs` de propósito, porque um arquivo de testes cresce com a cobertura e
//! cortá-lo por tamanho seria cortar prova.

use super::*;
use crate::noise::Basis;
use ph2d_nodegraph::cook::{Cook, OpResolver};
use ph2d_nodegraph::graph::{Graph, NodeId as GNodeId};

struct Reg;
impl OpResolver for Reg {
    fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
        // A tiny grid source + this node.
        static SRC: NodeManifest = NodeManifest {
            id: NodeTypeId::of("test.grid"),
            name: "test.grid",
            inputs: &[],
            outputs: &[PortSpec {
                name: "out",
                ty: INST_VEC2,
            }],
            effect: Effect::Pure,
            clock: Clock::Frame,
            params: &[],
            lowerings: &[LoweringKind::Cpu],
        };
        struct Src;
        impl NodeOp for Src {
            fn manifest(&self) -> &'static NodeManifest {
                &SRC
            }
            fn eval(&self, ctx: &mut EvalCtx<'_>) {
                // A 4-across row so neighbours are spatially close.
                let p: Vec<[f32; 2]> = (0..8).map(|i| [i as f32, 0.0]).collect();
                ctx.emit(Stream::new(8).with("P", Column::Vec2(p)));
            }
        }
        // ⚠️ **Um BLOCO, e ele existe por uma medição.** A fileira acima tem
        // `y = 0` em toda peça, e um `scale_y` multiplica exactamente esse zero:
        // o primeiro gate do eixo Y reprovou com `move 0` sobre código CORRECTO.
        // *Uma fixture só prova o que ela contém* — e a rotação, essa, mostra-se
        // numa fileira (rodar `(x, 0)` por 90° dá `(0, x)`), que é o que esconde
        // o buraco de quem só olha para um dos dois.
        static BLOCK: NodeManifest = NodeManifest {
            id: NodeTypeId::of("test.block"),
            name: "test.block",
            inputs: &[],
            outputs: &[PortSpec {
                name: "out",
                ty: INST_VEC2,
            }],
            effect: Effect::Pure,
            clock: Clock::Frame,
            params: &[],
            lowerings: &[LoweringKind::Cpu],
        };
        struct Block;
        impl NodeOp for Block {
            fn manifest(&self) -> &'static NodeManifest {
                &BLOCK
            }
            fn eval(&self, ctx: &mut EvalCtx<'_>) {
                let p: Vec<[f32; 2]> = (0..3)
                    .flat_map(|r| (0..3).map(move |c| [c as f32, r as f32]))
                    .collect();
                ctx.emit(Stream::new(9).with("P", Column::Vec2(p)));
            }
        }
        match ty {
            t if t == MANIFEST.id => {
                static N: MotionNoise = MotionNoise;
                Some(&N)
            }
            t if t == SRC.id => {
                static S: Src = Src;
                Some(&S)
            }
            t if t == BLOCK.id => {
                static B: Block = Block;
                Some(&B)
            }
            _ => None,
        }
    }
}

fn cook_y(graph: &Graph, node: GNodeId, t: f64) -> Vec<f32> {
    let mut cook = Cook::new();
    let out = cook.cook(graph, &Reg, node, t).unwrap();
    match out[0].as_stream().get("P").unwrap() {
        Column::Vec2(v) => v.iter().map(|p| p[1]).collect(),
        _ => panic!("P"),
    }
}

/// Cooked through the substrate: the noise displaces Y, and the field is
/// COHERENT — neighbouring elements move by similar amounts (a smooth field),
/// unlike a per-element jitter which would be uncorrelated. This is the whole
/// point of a spatial noise field vs `motion.wiggle`.
#[test]
fn the_field_displaces_y_coherently_across_neighbours() {
    let mut g = Graph::new();
    let src = g.add_node("test.grid");
    let noise = g.add_node("motion.noise");
    g.connect(ph2d_nodegraph::graph::Edge {
        from: (src, 0),
        to: (noise, 0),
        delayed: false,
    })
    .unwrap();
    g.set_param(noise, "amplitude", 1.0);
    g.set_param(noise, "scale", 0.25); // large features → strong neighbour correlation
    g.set_param(noise, "octaves", 1.0);

    let ys = cook_y(&g, noise, 0.5);
    // Something moved (the field is not flat).
    assert!(
        ys.iter().any(|&y| y.abs() > 0.01),
        "the field displaced nothing"
    );
    // Coherence: at a large feature size, adjacent elements differ far less
    // than the amplitude — they belong to the same swell, not independent
    // random draws. Max neighbour step is a fraction of the peak.
    let max_step = ys
        .windows(2)
        .map(|w| (w[1] - w[0]).abs())
        .fold(0.0_f32, f32::max);
    let peak = ys.iter().map(|y| y.abs()).fold(0.0_f32, f32::max);
    assert!(
        max_step < peak * 0.9,
        "neighbours should move coherently: step {max_step} vs peak {peak}"
    );
}

/// SONDA: quanto o ciclo deriva ao longo de muitas voltas (precisao de f32).
#[test]
#[ignore = "sonda de medição"]
fn measure_the_loop_drift() {
    let l = 3.0f32;
    println!("\n=== tempo LIDO pelo campo, volta a volta (L = 3.0) ===");
    for volta in [0u32, 1, 2, 10, 100, 1000] {
        let t = 0.125 + f32::from(u16::try_from(volta).unwrap()) * l;
        let (a, _b, w) = ph2d_fbm::loop_times(t, l);
        let mut g = Graph::new();
        let src = g.add_node("test.grid");
        let noise = g.add_node("motion.noise");
        g.connect(ph2d_nodegraph::graph::Edge {
            from: (src, 0),
            to: (noise, 0),
            delayed: false,
        })
        .unwrap();
        g.set_param(noise, "speed", 1.0);
        g.set_param(noise, "loop_len", l);
        let base = cook_y(&g, noise, 0.125);
        let here = cook_y(&g, noise, f64::from(t));
        let dev = base
            .iter()
            .zip(&here)
            .map(|(a, b)| (a - b).abs())
            .fold(0.0f32, f32::max);
        println!(
            "  volta {volta:>5}: t={t:>10.4}  tau={a:>12.9}  w={w:>12.9}  desvio no VALOR={dev:.3e}"
        );
    }
    println!();
}

/// SONDA: a inclinação do campo ao longo do ciclo — a costura é quina ou é ruído?
#[test]
#[ignore = "sonda de medição"]
fn measure_the_seam_slope() {
    let mut g = Graph::new();
    let src = g.add_node("test.grid");
    let noise = g.add_node("motion.noise");
    g.connect(ph2d_nodegraph::graph::Edge {
        from: (src, 0),
        to: (noise, 0),
        delayed: false,
    })
    .unwrap();
    g.set_param(noise, "speed", 1.0);
    let l = 3.0f64;
    g.set_param(noise, "loop_len", l as f32);
    for d in [0.05f64, 0.02, 0.005, 0.001] {
        let slope = |t: f64| -> f32 {
            let lo = cook_y(&g, noise, t - d);
            let hi = cook_y(&g, noise, t + d);
            ((hi[0] - lo[0]) as f64 / (2.0 * d)) as f32
        };
        print!("d={d:>6} |");
        for frac in [0.02f64, 0.25, 0.5, 0.75, 0.98] {
            print!("  tau={:>5.2}: {:>7.3}", frac * l, slope(frac * l));
        }
        println!(
            "   salto na costura: {:.4}",
            (slope(l - 2.0 * d) - slope(2.0 * d)).abs()
        );
    }
}

/// **O ciclo FECHA: o campo em `t` e em `t + L` é o mesmo.**
///
/// Nasce vermelho sobre o cross-fade ingênuo (misturar `t` com `t − L` sem wrapar o tempo
/// primeiro), que é o erro natural aqui — ele produz um valor contínuo e um ciclo que NÃO
/// fecha, errando por O(1); nenhum outro gate desta crate o distingue.
///
/// Amostra o ciclo inteiro, não os endpoints: uma lei que só casasse em `t = 0` passaria
/// por um oráculo de dois pontos.
///
/// ⚠️ **A tolerância é MEDIDA, não escolhida, e o mecanismo dela é `f32`:** a igualdade é
/// exata em ℝ, mas `frac(t / L)` perde mantissa conforme `t` cresce, então o tempo que o
/// campo lê deriva. Medido (sonda `measure_the_loop_drift`, L = 3): **2,1e-7 na 1ª volta ·
/// 1,4e-5 na centésima · 1,1e-4 na milésima** — 50 minutos de relógio para um desvio de um
/// décimo de milésimo de unidade de mundo. Fazer o wrap em `f64` cortaria isso, e foi
/// RECUSADO: o WGSL só tem `f32`, então os dois lados divergiriam e a paridade CPU×GPU —
/// que é o que prova que o device concorda — passaria a ter um épsilon que ninguém mediu.
#[test]
fn the_loop_closes_the_field_repeats_exactly() {
    let mut g = Graph::new();
    let src = g.add_node("test.grid");
    let noise = g.add_node("motion.noise");
    g.connect(ph2d_nodegraph::graph::Edge {
        from: (src, 0),
        to: (noise, 0),
        delayed: false,
    })
    .unwrap();
    g.set_param(noise, "speed", 1.0);
    g.set_param(noise, "loop_len", 3.0);
    // 10× o desvio medido na 2ª volta — aperta o bastante para o cross-fade ingênuo, que
    // erra por O(1), morrer com folga de quatro ordens.
    const TOL: f32 = 5e-6;
    let dev = |a: &[f32], b: &[f32]| {
        a.iter()
            .zip(b)
            .map(|(x, y)| (x - y).abs())
            .fold(0.0f32, f32::max)
    };
    for k in 0..24 {
        let t = f64::from(k) * 0.125;
        let base = cook_y(&g, noise, t);
        let d1 = dev(&base, &cook_y(&g, noise, t + 3.0));
        let d2 = dev(&base, &cook_y(&g, noise, t + 6.0));
        assert!(
            d1 < TOL && d2 < TOL,
            "o campo em t={t} nao volta em t+L: desvio {d1} numa volta, {d2} em duas"
        );
    }
}

/// **Loop desligado é o mundo de sempre, AO BIT** — o default não move um número.
///
/// A metade oposta do gate acima: sem ela, "faça o ciclo fechar" tem a resposta trivial de
/// congelar o campo, que fecha o ciclo perfeitamente e destrói o nó.
#[test]
fn no_loop_is_the_old_world_and_the_field_still_evolves() {
    let mut g = Graph::new();
    let src = g.add_node("test.grid");
    let noise = g.add_node("motion.noise");
    g.connect(ph2d_nodegraph::graph::Edge {
        from: (src, 0),
        to: (noise, 0),
        delayed: false,
    })
    .unwrap();
    g.set_param(noise, "speed", 1.0);
    // Sem loop o campo NUNCA se repete no alcance medido.
    assert_ne!(cook_y(&g, noise, 0.0), cook_y(&g, noise, 3.0));
    assert_ne!(cook_y(&g, noise, 0.0), cook_y(&g, noise, 1.0));
    // E com o loop ARMADO ele continua evoluindo DENTRO do ciclo (não congela).
    g.set_param(noise, "loop_len", 3.0);
    assert_ne!(cook_y(&g, noise, 0.0), cook_y(&g, noise, 1.0));
    assert_ne!(cook_y(&g, noise, 1.0), cook_y(&g, noise, 2.0));
}

/// **A costura é C¹: o salto de inclinação CONVERGE A ZERO quando a amostragem aperta.**
///
/// O peso smoothstep existe só para isto — com peso LINEAR o valor fecha e a derivada
/// salta, e um salto de derivada num campo de movimento lê como um tranco a cada volta.
///
/// ⚠️ **O oráculo é a CONVERGÊNCIA, não um número**, e as duas versões anteriores deste
/// gate erraram de maneiras opostas — vale mais que a lei:
///
/// 1. A primeira media a inclinação de `a + (b − a)·w`, uma mistura de TEMPOS, e ficava
///    **VERDE sobre o peso linear**: ali `w = u` colapsa a expressão em `τ − L·(τ/L) = 0`,
///    constante. Mas o campo faz `lerp(fbm(a), fbm(b), w)`, e `fbm` não é linear — misturar
///    tempos não é misturar campos. Era espelho da aritmética, não do fenômeno.
/// 2. A segunda amostrou o campo COZIDO (certo) com uma diferença central de `d = 0,02` e
///    **REPROVOU a lei correta**, acusando um salto de 0,60. Medido (sonda
///    `measure_the_seam_slope`), o salto é **0,9653 · 0,2609 · 0,0206 · 0,0009** para
///    `d = 0,05 · 0,02 · 0,005 · 0,001`: ele converge a zero, que é a assinatura de uma
///    derivada que EXISTE. Uma quina de verdade daria salto constante.
///
/// Por isso o gate compara o salto em duas resoluções: se a derivada existe ele encolhe com
/// `d`; se há quina, ele fica onde está.
#[test]
fn the_seam_of_the_loop_is_smooth_not_a_kink() {
    let mut g = Graph::new();
    let src = g.add_node("test.grid");
    let noise = g.add_node("motion.noise");
    g.connect(ph2d_nodegraph::graph::Edge {
        from: (src, 0),
        to: (noise, 0),
        delayed: false,
    })
    .unwrap();
    g.set_param(noise, "speed", 1.0);
    let l = 3.0f64;
    g.set_param(noise, "loop_len", l as f32);

    // O salto de inclinação através da costura, medido com passo `d`.
    let jump = |d: f64| -> f32 {
        let slope = |t: f64| -> Vec<f32> {
            let lo = cook_y(&g, noise, t - d);
            let hi = cook_y(&g, noise, t + d);
            hi.iter()
                .zip(&lo)
                .map(|(a, b)| ((a - b) as f64 / (2.0 * d)) as f32)
                .collect()
        };
        let before = slope(l - 2.0 * d);
        let after = slope(2.0 * d);
        before
            .iter()
            .zip(&after)
            .map(|(a, b)| (a - b).abs())
            .fold(0.0f32, f32::max)
    };

    let coarse = jump(0.02);
    let fine = jump(0.001);
    assert!(
        fine < coarse * 0.25,
        "o salto de inclinacao NAO converge ({coarse} em d=0.02 contra {fine} em d=0.001) \
         -- a costura tem QUINA, e nao erro de amostragem"
    );
}

/// Uma fileira sob o ruído, com os params que a chamada pedir.
fn field_with(setup: impl FnOnce(&mut Graph, GNodeId)) -> Vec<f32> {
    field_on("test.grid", setup)
}

/// O mesmo, sobre o BLOCO 3×3 — a fixture que **contém** o eixo Y (ver o `Reg`).
fn block_with(setup: impl FnOnce(&mut Graph, GNodeId)) -> Vec<f32> {
    field_on("test.block", setup)
}

fn field_on(source: &str, setup: impl FnOnce(&mut Graph, GNodeId)) -> Vec<f32> {
    let mut g = Graph::new();
    let src = g.add_node(source);
    let noise = g.add_node("motion.noise");
    g.connect(ph2d_nodegraph::graph::Edge {
        from: (src, 0),
        to: (noise, 0),
        delayed: false,
    })
    .unwrap();
    // Campo ESTÁTICO: sem o relógio a rolar, o que muda é só o espaço.
    g.set_param(noise, "speed", 0.0);
    setup(&mut g, noise);
    cook_y(&g, noise, 0.0)
}

/// **OS DEFAULTS DO ESPAÇO SÃO A IDENTIDADE, BIT-A-BIT.**
///
/// ⚠️ A metade que decide se esta wave pode ser integrada: `rotation = 0` e
/// `uniform = 1` têm de devolver **o mesmo `f32`** que a expressão que estava
/// escrita aqui antes (`fbm(px·scale, py·scale + …)`). A barra é `==`, não um ε —
/// *byte-idêntico* é a promessa que a coluna de risco da folha escreveu, e um ε
/// aceitaria uma promessa mais fraca sem ninguém reparar.
#[test]
fn the_field_space_defaults_are_the_identity() {
    let plain = field_with(|_, _| {});
    // Escritos À MÃO, não omitidos: um default que o manifesto mudasse por baixo
    // passaria despercebido se o gate só omitisse os params.
    let explicit = field_with(|g, n| {
        g.set_param(n, "rotation", 0.0);
        g.set_param(n, "uniform", 1.0);
        g.set_param(n, "scale_y", 999.0); // ignorado sob `uniform`
    });
    assert_eq!(
        plain, explicit,
        "com os defaults do espaço o campo tem de ser o de sempre"
    );
}

/// **A ROTAÇÃO roda o ESPAÇO, e a volta completa devolve o campo.**
///
/// Duas metades, e a segunda é o controle: se só se pedisse *"90° muda alguma
/// coisa"*, um `rotation` que multiplicasse o campo por qualquer coisa passaria.
/// **360° tem de voltar** — é o que separa *rodar o espaço* de *estragar o campo*.
#[test]
fn a_rotation_turns_the_space_and_a_full_turn_returns_it() {
    let plain = field_with(|_, _| {});
    let turned = field_with(|g, n| g.set_param(n, "rotation", 90.0));
    let worst = plain
        .iter()
        .zip(&turned)
        .fold(0.0f32, |m, (a, b)| m.max((a - b).abs()));
    assert!(
        worst > 0.05,
        "90° tem de amostrar o campo noutro sítio, e move {worst}"
    );
    let full = field_with(|g, n| g.set_param(n, "rotation", 360.0));
    let back = plain
        .iter()
        .zip(&full)
        .fold(0.0f32, |m, (a, b)| m.max((a - b).abs()));
    assert!(
        back < 1e-5,
        "uma volta completa tem de devolver o campo, e desvia {back} \
         (contra {worst} de 90°)"
    );
}

/// **O `scale_y` estica UM eixo, e o `uniform` é quem o deixa falar.**
///
/// ⚠️ As duas metades outra vez: sob `uniform = 1` o `scale_y` tem de ser **inerte**
/// (senão o trio do `motion.scale` não foi respeitado e um documento antigo muda de
/// aparência), e sob `uniform = 0` ele tem de MUDAR o campo.
#[test]
fn scale_y_stretches_one_axis_only_when_uniform_is_off() {
    let plain = block_with(|_, _| {});
    let gated = block_with(|g, n| {
        g.set_param(n, "uniform", 1.0);
        g.set_param(n, "scale_y", 3.0);
    });
    assert_eq!(plain, gated, "sob `uniform` o `scale_y` é inerte");

    let stretched = block_with(|g, n| {
        g.set_param(n, "uniform", 0.0);
        g.set_param(n, "scale_y", 3.0);
    });
    let worst = plain
        .iter()
        .zip(&stretched)
        .fold(0.0f32, |m, (a, b)| m.max((a - b).abs()));
    assert!(
        worst > 0.05,
        "com `uniform = 0` o `scale_y` tem de mudar o campo, e move {worst}"
    );
    // E o CONTROLE do próprio `scale_y`: repô-lo no valor do `scale` volta ao campo
    // isotrópico — é isso que prova que ele é o eixo Y e não um multiplicador solto.
    let same = block_with(|g, n| {
        g.set_param(n, "uniform", 0.0);
        g.set_param(n, "scale_y", 0.4); // o default do `scale`
    });
    assert_eq!(
        plain, same,
        "`scale_y` igual ao `scale` com `uniform = 0` é o campo isotrópico"
    );
}

/// The playhead scrolls the field (Temporal): the same elements read a
/// different slice of the field at a later time.
#[test]
fn the_field_evolves_with_the_playhead() {
    let mut g = Graph::new();
    let src = g.add_node("test.grid");
    let noise = g.add_node("motion.noise");
    g.connect(ph2d_nodegraph::graph::Edge {
        from: (src, 0),
        to: (noise, 0),
        delayed: false,
    })
    .unwrap();
    g.set_param(noise, "speed", 1.0);
    assert_ne!(cook_y(&g, noise, 0.0), cook_y(&g, noise, 1.0));
}

/// **`range_mode = 0` É O NÓ QUE SEMPRE SHIPOU — BIT-A-BIT, NOS TRÊS TIPOS.**
///
/// O gate de neutralidade de toda régua apendada: um documento já autorado não
/// pode mudar de imagem por um param que ele não conhece.
#[test]
fn the_amplitude_ruler_is_the_node_that_shipped_in_every_type() {
    for ty in 0..=2 {
        let before = field_with(|g, n| {
            g.set_param(n, "type", ty as f32);
            g.set_param(n, "amplitude", 0.7);
        });
        let after = field_with(|g, n| {
            g.set_param(n, "type", ty as f32);
            g.set_param(n, "amplitude", 0.7);
            g.set_param(n, "range_mode", 0.0);
            // ⚠️ Números NÃO-neutros nos dois: se o `range_mode` vazasse, isto
            // apanhava-o. Com `min = -1, max = 1` (os defaults) o vazamento seria
            // invisível para o `Fbm`.
            g.set_param(n, "min", 3.0);
            g.set_param(n, "max", 9.0);
        });
        assert_eq!(before, after, "tipo {ty}: a regua desligada mudou o campo");
    }
}

/// O DESLOCAMENTO que o nó aplicou, isolado da posição de cada peça no bloco.
///
/// ⚠️ **A primeira versão dos dois gates abaixo comparava a posição ABSOLUTA, e
/// reprovou sobre produto correto** (`Ridged` deu `[5,4 .. 8]` para uma faixa
/// pedida de `[2 .. 6]`): o `cook_y` devolve onde a peça FICOU, e o bloco 3×3 já
/// tem Y próprio. O campo estava certo; o instrumento é que somava o berço.
fn delta_on_block(setup: impl Fn(&mut Graph, GNodeId)) -> Vec<f32> {
    let moved = block_with(|g, n| setup(g, n));
    // `amplitude = 0` com a régua de sempre ⇒ deslocamento zero ⇒ o berço.
    let base = block_with(|g, n| g.set_param(n, "amplitude", 0.0));
    moved.iter().zip(&base).map(|(m, b)| m - b).collect()
}

/// **NADA SAI DA FAIXA PEDIDA — NOS TRÊS TIPOS.** O continente, que é a promessa
/// literal do nome do knob.
#[test]
fn nothing_leaves_the_range_you_asked_for() {
    let (min, max) = (2.0f32, 6.0f32);
    for ty in 0..=2 {
        let ys = delta_on_block(|g, n| {
            g.set_param(n, "type", ty as f32);
            g.set_param(n, "octaves", 4.0);
            g.set_param(n, "scale", 1.5);
            g.set_param(n, "range_mode", 1.0);
            g.set_param(n, "min", min);
            g.set_param(n, "max", max);
        });
        let lo = ys.iter().copied().fold(f32::INFINITY, f32::min);
        let hi = ys.iter().copied().fold(f32::NEG_INFINITY, f32::max);
        assert!(
            lo >= min - 1e-3 && hi <= max + 1e-3,
            "tipo {ty}: o deslocamento saiu da faixa [{min}, {max}]: [{lo}, {hi}]"
        );
        assert!(hi > lo, "tipo {ty}: o campo nao se moveu");
    }
}

/// **A ARMADILHA, MEDIDA EXACTAMENTE: o knob DOBRA a excursão que a conta de
/// cabeça perde — e só nos tipos onde ela está errada.**
///
/// ⚠️ **É o oráculo certo, e a primeira versão deste gate usou o errado.** Eu pedi
/// que o campo *encostasse* nas duas pontas da faixa, e nove amostras de um fBm
/// não visitam os extremos da forma (medido em [`measure_natural_range`]: o
/// `Ridged` a 4 oitavas tem piso empírico `0,098`). Essa barra é sobre a FIXTURE,
/// não sobre o produto.
///
/// O que é exacto e não depende de fixture nenhuma: para as MESMAS amostras, a
/// razão entre a excursão com o knob e a excursão com a conta do artista
/// (`amplitude = (max−min)/2`) é `(hi_nat − lo_nat)/2` — **1× num campo bipolar**
/// (a conta está certa) e **2× num retificado** (a conta perde metade). É a
/// armadilha inteira, num número.
#[test]
fn the_knob_doubles_exactly_the_excursion_the_head_arithmetic_loses() {
    let (min, max) = (2.0f32, 6.0f32);
    let span_of = |ty: i32, ranged: bool| {
        let ys = delta_on_block(move |g, n| {
            g.set_param(n, "type", ty as f32);
            g.set_param(n, "octaves", 4.0);
            g.set_param(n, "scale", 1.5);
            if ranged {
                g.set_param(n, "range_mode", 1.0);
                g.set_param(n, "min", min);
                g.set_param(n, "max", max);
            } else {
                g.set_param(n, "amplitude", (max - min) * 0.5);
            }
        });
        let lo = ys.iter().copied().fold(f32::INFINITY, f32::min);
        let hi = ys.iter().copied().fold(f32::NEG_INFINITY, f32::max);
        hi - lo
    };
    for (ty, want) in [(0, 1.0f32), (1, 2.0), (2, 2.0)] {
        let ratio = span_of(ty, true) / span_of(ty, false);
        assert!(
            (ratio - want).abs() < 1e-3,
            "tipo {ty}: a razao tem de ser {want}, e' {ratio}"
        );
    }
}

/// **SONDA: a faixa que cada tipo de facto ocupa** — não a que a fórmula promete.
///
/// ⚠️ **Nasceu de um gate vermelho (2026-08-22).** Declarei `Ridged` como `[0,1]`
/// porque `(1−|n|)²` está em `[0,1]` *quando `|n| ≤ 1`* — e o ruído de gradiente
/// do Perlin 2002 **passa de 1**. A teoria estava certa sobre a fórmula e errada
/// sobre a ENTRADA dela.
///
/// `cargo test -p ph2d-node-motion-noise measure_natural_range -- --ignored --nocapture`
#[test]
#[ignore = "sonda, não um gate — `-- --ignored --nocapture`"]
fn measure_natural_range() {
    for ty in [
        ph2d_fbm::NoiseType::Fbm,
        ph2d_fbm::NoiseType::Turbulence,
        ph2d_fbm::NoiseType::Ridged,
    ] {
        for octaves in [1u32, 2, 4, 8] {
            let spec = ph2d_fbm::Spec {
                octaves,
                lacunarity: 2.0,
                roughness: 0.5,
                ty,
            };
            let (mut lo, mut hi) = (f32::INFINITY, f32::NEG_INFINITY);
            for a in -300..300 {
                for b in -300..300 {
                    let v = fbm(a as f32 * 0.031, b as f32 * 0.029, 7, spec, Basis::GRADIENT);
                    lo = lo.min(v);
                    hi = hi.max(v);
                }
            }
            let declared = ty.natural_range();
            println!("{ty:?} oct={octaves}: medido [{lo:.4} .. {hi:.4}]  declarado {declared:?}");
        }
    }
}
