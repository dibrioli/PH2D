//! **A BANCADA** — como se monta um nó sozinho e se mede se um param o move.
//!
//! ⚠️ **Um módulo partilhado, e a razão é uma só:** os dois consumidores têm de fazer a MESMA
//! pergunta. A varredura ([`super`]`::dead_knob_sweep`) procura knobs mortos; o gate
//! `param_gates_are_exact` prova que cada `ParamGate` cobre exactamente os modos em que o knob
//! age. Se cada um montasse a sua bancada, eles podiam **discordar** — e um gate que discorda
//! da sonda que o motivou não prova nada.
//!
//! ⚠️ As oito curas de ponto cego que a verificação de 2026-08-22 pagou vivem TODAS aqui
//! (doc 90 §4). Ler os doc-comments abaixo antes de mexer: cada `⚠️` é um lote de falsos
//! positivos já pago.

//! ⚠️ **O `allow(dead_code)` não é preguiça:** este módulo compila dentro de CADA
//! binário de teste que o declara, e nenhum deles usa a bancada inteira — sem ele, cada
//! binário acusaria de morto exactamente o que o outro consome.
#![allow(dead_code)]

use ph2d_node_registry::{NodeRegistry, ParamUiHint, ParamWidget};
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};
use ph2d_nodegraph::node::NodeManifest;
use ph2d_nodegraph::port::PortType;
use ph2d_nodegraph::value::CookValue;

/// Quantos quadros correr antes de ler a saída.
///
/// ⚠️ **Não é 1 de propósito.** Um param de simulação (`damping`, `stiffness`) não muda nada no
/// primeiro tique — o estado ainda é a pose inicial. Uma sonda de um quadro acusaria de morta
/// metade da pilha de simulação.
/// ⚠️ **Era 12, e 12 é curto demais para um ENVELOPE.** Com os defaults do `pulse.adsr` o
/// segmento de release só começa aos `0,35 s` — 21 quadros — então `release` e `release_shape`
/// liam mortos por a sonda ter acabado antes de eles existirem. 48 quadros são `0,8 s`, que
/// cobre o envelope inteiro e a segunda batida de um metrónomo a 120 BPM.
pub const TICKS: usize = 48;
pub const DT: f64 = 1.0 / 60.0;

/// Os alimentadores preferidos, por ordem.
///
/// ⚠️ **A primeira versão desta sonda alimentava toda porta VALUE com o `debug.const` — o único
/// nó-fonte registado cujo tipo bate — e ele emite `Stream::new(1).with("v", [1.0])`.** Isso
/// acusou de MORTO o `strength` do `value.gain`, que é vivíssimo: **1.0 é ponto fixo de toda
/// curva de ganho**, e uma fixture parada no ponto fixo da transformação não distingue knob
/// nenhum. *Uma fixture que não contém o fenômeno não prova a ausência dele* — ela só prova
/// que a fixture é fraca. É por isso que o alimentador passou a ser uma CADEIA (abaixo).
pub const PREFERRED: &[&str] = &[
    "motion.grid",
    "motion.emitter",
    "value.instance_field",
    "source.shape",
    "debug.wave",
    "debug.const",
];

/// Até que profundidade encadear nós para produzir o tipo de uma porta.
///
/// Nenhuma fonte registada emite um VALOR que VARIE — o `debug.const` é constante e o
/// `value.instance_field` precisa de um stream para saber quantos elementos emitir. A cadeia
/// `motion.grid → value.instance_field` produz `n` valores distintos, que é a fixture que a
/// varredura precisa. Dois saltos chegam para todo tipo do catálogo; mais do que isso começaria
/// a montar cenas em vez de bancadas.
pub const MAX_CHAIN: usize = 2;

pub fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("todo nó registra");
    reg
}

pub fn catalogue(reg: &NodeRegistry) -> Vec<&'static NodeManifest> {
    let mut v: Vec<&'static NodeManifest> = reg.manifests().collect();
    // Os preferidos primeiro, o resto por nome — a ordem É a política de alimentação.
    v.sort_by_key(|m| {
        (
            PREFERRED
                .iter()
                .position(|p| *p == m.name)
                .unwrap_or(usize::MAX),
            m.name,
        )
    });
    v
}

/// Monta, dentro de `g`, uma sub-árvore que PRODUZ o tipo `want`, e devolve a porta de saída.
///
/// ⚠️ **É recursiva de propósito.** Nenhuma fonte registada emite um VALOR que varie, então
/// alimentar `value.*` com uma fonte direta é alimentar com uma constante — o erro que a
/// primeira versão cometeu. A cadeia `motion.grid → value.instance_field` produz `n` valores
/// distintos porque o segundo nó lê o comprimento do primeiro.
///
/// ⚠️ Uma tentativa falhada deixa nós órfãos no grafo. É inofensivo: o cook é *pull*, e um nó
/// sem consumidor nunca é cozido — mas é a razão de esta função não poder ser usada para medir
/// contagem de nós.
/// `rotate` gira a ordem dos candidatos — é como duas portas do mesmo tipo recebem fontes
/// DIFERENTES.
///
/// ⚠️ **Sem isto, todo nó binário recebe `a ≡ b` ao bit**, e um param que só se vê na
/// diferença lê morto: o `epsilon` do `value.math` (que compara `|a−b|` contra ele), o `clamp`
/// do `field.combine` (só o `Add` sai da faixa quando os dois lados são iguais), e o
/// `field.shape` inteiro — a nuvem e o polígono eram o MESMO conjunto de pontos, logo cada
/// elemento caía exactamente sobre um vértice e a distância era zero para toda a gente.
/// *Duas entradas iguais não são uma fixture de um nó de duas entradas.*
pub fn feed(
    g: &mut Graph,
    all: &[&'static NodeManifest],
    want: PortType,
    depth: usize,
    used: &mut Vec<&'static str>,
    rotate: usize,
) -> Option<(NodeId, u16)> {
    // ⚠️ A rotação corre sobre os CANDIDATOS que servem, nunca sobre o catálogo inteiro —
    // girar a lista toda destruiria a ordem de preferência e daria alimentadores piores à
    // primeira porta, que é a que mais importa.
    let mut fits: Vec<(&'static NodeManifest, usize)> = all
        .iter()
        .filter_map(|m| {
            m.outputs
                .iter()
                .position(|o| o.ty.connects_directly(want))
                .map(|op| (*m, op))
        })
        .collect();
    let n_fits = fits.len();
    if n_fits > 0 {
        fits.rotate_left(rotate % n_fits);
    }
    for (m, op) in fits {
        if m.inputs.is_empty() {
            used.push(m.name);
            return Some((g.add_node(m.name), op as u16));
        }
        if depth == 0 {
            continue;
        }
        let node = g.add_node(m.name);
        let mut ok = true;
        for (i, p) in m.inputs.iter().enumerate() {
            match feed(g, all, p.ty, depth - 1, used, rotate) {
                Some((s, sp)) => {
                    if g.connect(Edge {
                        from: (s, sp),
                        to: (node, i as u16),
                        delayed: false,
                    })
                    .is_err()
                    {
                        ok = false;
                        break;
                    }
                }
                None => {
                    ok = false;
                    break;
                }
            }
        }
        if ok {
            used.push(m.name);
            return Some((node, op as u16));
        }
    }
    None
}

/// Uma bancada para o nó.
///
/// - `loop_port`: qual porta (se alguma) recebe a saída do próprio nó, com atraso — o `pre` que
///   um nó de ESTADO precisa (`motion.trail`, `motion.strobe`, `motion.wave`, a pilha `sim.*`).
/// - `const_first`: inverte a preferência de alimentador, pondo as fontes CONSTANTES à frente
///   das cadeias que variam.
///
/// ⚠️ **As duas opções nasceram da verificação das acusações, e cada uma explicava DEZENAS de
/// falsos positivos.**
///
/// 1. *Laçar TODAS as portas do tipo da saída de uma vez esvazia o nó.* O `motion.trail` tem
///    `in` **e** `state`, ambos do tipo da saída: laçar os dois deixa o nó sem fonte, o stream
///    sai vazio, e `fade`/`shrink`/`spacing`/`hue_shift`/`saturation` leem todos mortos. A
///    variante certa é laçar **uma** porta e alimentar as outras — por isso `loop_port` é um
///    índice, e não um booleano.
/// 2. *Uma fonte que VARIA pode ser pior que uma constante.* Uma porta de campo lida por
///    `.first()` (o `driven_value` do doc 58) recebe do `value.instance_field(Ramp)` o elemento
///    zero, que é exatamente `0.0` — e um `amount = 0` desliga o nó inteiro em silêncio. Foi o
///    que matou os treze knobs de geometria do `motion.spline_wrap` e as iterações do
///    `motion.voronoi`. A cadeia continua a ser a preferência **por defeito**, porque é ela que
///    cura o ponto fixo do `value.gain`; a constante entra como bancada IRMÃ.
///
/// *As duas leis têm a mesma forma: uma bancada não prova a ausência de um efeito, só a ausência
/// dele NAQUELA bancada — e a cura é ter mais de uma, nunca uma melhor.*
pub fn bench(
    all: &[&'static NodeManifest],
    m: &'static NodeManifest,
    loop_port: Option<usize>,
    const_first: bool,
    required: Option<&'static [&'static str]>,
) -> Option<(Graph, NodeId, String)> {
    let mut g = Graph::new();
    let n = g.add_node(m.name);
    let mut pool: Vec<&'static NodeManifest> = all.to_vec();
    if const_first {
        // As fontes sem entrada primeiro — elas emitem um valor constante e não-nulo.
        pool.sort_by_key(|x| (!x.inputs.is_empty(), x.name));
    }
    let mut used: Vec<&'static str> = Vec::new();
    for (i, p) in m.inputs.iter().enumerate() {
        // ⚠️ **Uma porta OPCIONAL ligada é uma porta que muda a lei do nó.** Meia dúzia de nós
        // deste catálogo dizem, no cabeçalho, que um param só vale quando a porta homónima
        // está DESLIGADA (`amount_of` = `porta.first().unwrap_or(param)`), e outros tantos têm
        // um `reset` cujo estado alto congela o contador. Ligar tudo — o que a 1ª sonda fazia —
        // é escolher a metade do espaço em que esses params são inertes **por contrato**.
        // *O registry já sabe quais portas são obrigatórias; a sonda é que não estava a
        // perguntar.*
        if required.is_some_and(|req| !req.contains(&p.name)) {
            used.push("<solta>");
            continue;
        }
        if loop_port == Some(i) {
            g.connect(Edge {
                from: (n, 0),
                to: (n, i as u16),
                delayed: true,
            })
            .ok()?;
            used.push("<self>");
            continue;
        }
        // `i` como rotação: a porta 0 recebe o alimentador preferido, a porta 1 o seguinte.
        let (s, sp) = feed(&mut g, &pool, p.ty, MAX_CHAIN, &mut used, i)?;
        g.connect(Edge {
            from: (s, sp),
            to: (n, i as u16),
            delayed: false,
        })
        .ok()?;
    }
    used.dedup();
    Some((g, n, used.join("+")))
}

/// Todas as bancadas a tentar para um nó: sem laço e com o laço em cada porta que o aceita,
/// vezes as duas políticas de alimentador, vezes ligar-tudo / deixar-as-opcionais-soltas.
///
/// ⚠️ **Um knob só é MORTO se NENHUMA delas o faz mexer** — é a direcção conservadora, e é o
/// que separa uma lista de acusações de uma lista de ruído.
pub fn all_benches(
    reg: &NodeRegistry,
    all: &[&'static NodeManifest],
    m: &'static NodeManifest,
) -> Vec<(Graph, NodeId, String)> {
    let out_ty = m.outputs.first().map(|o| o.ty);
    let mut loops: Vec<Option<usize>> = vec![None];
    for (i, p) in m.inputs.iter().enumerate() {
        if out_ty.is_some_and(|t| t.connects_directly(p.ty)) {
            loops.push(Some(i));
        }
    }
    // `None` = liga toda porta. `Some(req)` = liga só as obrigatórias, deixando as opcionais
    // soltas (o estado em que o artista larga o nó).
    let mut wirings: Vec<Option<&'static [&'static str]>> = vec![None];
    if let Some(req) = reg.required_inputs(m.id)
        && req.len() < m.inputs.len()
    {
        wirings.push(Some(req));
    }
    let mut out = Vec::new();
    for lp in loops {
        for const_first in [false, true] {
            for required in &wirings {
                if let Some(b) = bench(all, m, lp, const_first, *required) {
                    out.push(b);
                }
            }
        }
    }
    out
}

/// Os bits de uma coluna — comparação ao BIT, não por `==`.
///
/// ⚠️ **`f32::NaN != f32::NaN`**, então um `PartialEq` cru diria *"mudou"* de duas saídas
/// idênticas que contivessem um NaN, e a sonda acusaria de VIVO um knob morto num nó que
/// produz NaN. Os bits não têm essa opinião.
pub fn bits(c: &Column) -> Vec<u32> {
    match c {
        Column::Scalar(v) => v.iter().map(|x| x.to_bits()).collect(),
        Column::Vec2(v) => v.iter().flatten().map(|x| x.to_bits()).collect(),
        Column::Vec3(v) => v.iter().flatten().map(|x| x.to_bits()).collect(),
        Column::Vec4(v) => v.iter().flatten().map(|x| x.to_bits()).collect(),
    }
}

/// O TRAÇO do nó ao longo de `TICKS` quadros — todos os quadros, não só o último.
///
/// ⚠️ **Comparar só o último quadro é cegueira temporal, e ela custou caro.** Um envelope
/// (`pulse.adsr`) tem `attack_shape` a governar os primeiros três quadros e `release_shape` os
/// do fim: no instante em que a sonda lia, os dois já tinham entregado o mesmo patamar de
/// sustain, e os DOIS liam mortos. O mesmo para o `sim.spawn::scatter`, que só escolhe a linha
/// de um nascimento — e no quadro comparado não nascia ninguém.
///
/// *Um param temporal age numa JANELA; ler um instante é escolher não ver a janela dele.*
/// ⚠️ **Um nó que ASSERTA sobre as suas entradas está a dizer que a bancada não se aplica —
/// não que o produto está partido.** O `motion.wiggle` recusa uma porta `time` cujo comprimento
/// não bate com o stream (*"a porta `time` tem 2 valores para 91 instâncias"*), e a variante da
/// bancada que liga tudo produz exactamente isso. Sem este `catch_unwind`, uma bancada
/// inaplicável derruba a varredura inteira em vez de ser saltada — e o sintoma seria um teste
/// vermelho a apontar para uma crate que não tem defeito nenhum.
///
/// *Um explorador que morre no primeiro sítio onde não pode entrar não é um explorador.*
pub fn snapshot(g: &Graph, reg: &NodeRegistry, n: NodeId) -> Option<Vec<(String, Vec<u32>)>> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| snapshot_inner(g, reg, n))).ok()?
}

fn snapshot_inner(g: &Graph, reg: &NodeRegistry, n: NodeId) -> Option<Vec<(String, Vec<u32>)>> {
    let mut cook = Cook::new();
    let mut trace: Vec<(String, Vec<u32>)> = Vec::new();
    let mut any = false;
    for k in 0..TICKS {
        let t = k as f64 * DT;
        let out = cook.cook(g, reg, n, t).ok()?;
        if let Some(CookValue::Instances(s)) = out.first() {
            let mut cols: Vec<(String, Vec<u32>)> = s
                .columns()
                .map(|(c, v)| (format!("{k}/{c}"), bits(v)))
                .collect();
            cols.sort_by(|a, b| a.0.cmp(&b.0));
            any |= !cols.is_empty();
            trace.extend(cols);
        }
        cook.advance_tick(g, reg, t).ok()?;
    }
    any.then_some(trace)
}

/// Os valores com que sondar um param: os EXTREMOS que a UI permite, mais o meio.
pub fn probe_values(hint: Option<&ParamUiHint>, default: f32) -> Vec<f32> {
    match hint {
        Some(h) => match h.widget {
            // Um enum tem um espaço FINITO e exato — varre-o inteiro.
            ParamWidget::Enum { labels } => (0..labels.len()).map(|i| i as f32).collect(),
            ParamWidget::Toggle => vec![0.0, 1.0],
            // ⚠️ **Os TERÇOS, e não `min / meio / max`.** Um param ANGULAR com faixa
            // `-360..360` — e há vários — tem `min`, `meio` e `max` todos **congruentes**:
            // `-360°`, `0°` e `+360°` são o mesmo ângulo depois do `frac()`, então nenhuma
            // rotação deste catálogo podia ser provada viva. Os terços dão quatro valores em
            // que pelo menos dois são angularmente distintos em qualquer faixa.
            // ⚠️ O meio CONTINUA a fazer falta pelo motivo oposto (uma faixa simétrica num nó
            // que só lê o módulo), e é por isso que a lista tem quatro pontos e não dois.
            _ => {
                let span = h.max - h.min;
                vec![h.min, h.min + span / 3.0, h.min + span * 2.0 / 3.0, h.max]
            }
        },
        // Sem hint o painel pinta um slider genérico; sem faixa declarada, chuta em volta
        // do default — e a chutada tem de mudar de ORDEM DE GRANDEZA, senão um param cujo
        // efeito é sub-pixel passa por morto.
        None => vec![default, default + 1.0, default * 8.0 + 3.0],
    }
}

/// Os contextos de modo: o vazio (o nó recém-nascido), cada índice de cada enum/toggle, e um
/// contexto com TODA magnitude fora do neutro.
///
/// ⚠️ **O último é o que faz o `mode` do `value.gain` aparecer vivo, e ele custou a segunda
/// iteração desta sonda.** Um param de MODO só se distingue quando a grandeza que ele modula
/// está fora do neutro: com `strength = 0` (o default) toda curva de ganho é a identidade, e os
/// dois modos dão a MESMA saída. Varrer só os enums, com as magnitudes no default, acusa de
/// morto todo seletor de um nó cuja força nasce em zero — que é a convenção deste catálogo.
///
/// *A regra geral: um knob que só age MULTIPLICANDO outro nunca se vê com o outro no neutro.*
pub fn contexts(
    reg: &NodeRegistry,
    m: &'static NodeManifest,
) -> Vec<(String, Vec<(&'static str, f32)>)> {
    let mut out = vec![("default".to_string(), Vec::new())];
    let Some(hints) = reg.param_ui(m.id) else {
        return out;
    };
    let mut hot: Vec<(&'static str, f32)> = Vec::new();
    for h in hints {
        let values: Vec<f32> = match h.widget {
            ParamWidget::Enum { labels } => (0..labels.len()).map(|i| i as f32).collect(),
            ParamWidget::Toggle => vec![0.0, 1.0],
            _ => {
                // Uma magnitude fora do neutro: o ponto a 3/4 da faixa, e nunca o default.
                let v = h.min + (h.max - h.min) * 0.75;
                let d = m.param_default(h.param).unwrap_or(0.0);
                hot.push((h.param, if (v - d).abs() > 1e-6 { v } else { h.min }));
                continue;
            }
        };
        for v in values {
            out.push((format!("{}={v}", h.param), vec![(h.param, v)]));
        }
    }
    if !hot.is_empty() {
        out.push(("magnitudes-quentes".to_string(), hot));
    }
    out
}
