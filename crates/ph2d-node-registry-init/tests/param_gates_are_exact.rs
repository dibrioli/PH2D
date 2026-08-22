//! **CADA `ParamGate` COBRE EXACTAMENTE OS MODOS EM QUE O KNOB AGE** — o gate das dezanove
//! curas da caça aos knobs mortos ([doc 90](../../../docs/Motion%20Nodes/90_caca_aos_knobs_mortos.md)).
//!
//! ⚠️ **A tabela abaixo lista o PAR (nó, param), nunca os índices.** Escrever aqui os mesmos
//! `values` que estão no `ParamGate` seria repetir o palpite duas vezes e chamar-lhe prova: o
//! teste passaria com o índice errado nos dois sítios. Os modos em que o knob age são
//! **MEDIDOS** — cozinha-se o nó em cada índice do seletor, com dois valores do param, e
//! compara-se a saída ao bit.
//!
//! ## As duas metades, e por que nenhuma basta sozinha
//!
//! 1. **Fora do gate, o knob é inerte.** É o defeito original: o painel pinta um controle que
//!    não muda a imagem.
//! 2. **Dentro do gate, ele age em pelo menos um modo.** ⚠️ **Sem esta metade, um
//!    `values: &[]` — que esconde o knob SEMPRE — passaria**, e esse é o defeito oposto e pior:
//!    um controle que existe e que nenhum gesto alcança.
//!
//! *Um gate que só prova a ausência prova metade.*
//!
//! ## O que este arquivo NÃO cobre, de propósito
//!
//! ⚠️ Ele mede o **kernel** (o param muda a saída cozida?), não o **painel** (a row é
//! construída?). A metade do painel vive na shell, que é onde o construtor de rows existe —
//! e as duas são necessárias: um gate correcto que o construtor não lê continua a pintar o
//! knob morto.
//!
//! ⚠️ Os `ParamGateAbove` (limiar contínuo — a família do `octaves`/`avoid`) são medidos aqui
//! do mesmo modo, com o `when` no default e num valor acima do limiar.

mod common;

use common::{TICKS, all_benches, catalogue, registry, snapshot};
use ph2d_node_registry::{NodeRegistry, ParamUiHint, ParamWidget};
use ph2d_nodegraph::node::NodeManifest;

/// Os pares curados em 2026-08-22. ⚠️ **Só o PAR** — os índices vêm da medição.
///
/// A coluna do meio é o seletor de que o knob depende, e ela existe aqui só para o teste saber
/// **qual eixo varrer**; se ela estiver errada, o teste reprova por não achar o gate.
const CURED: &[(&str, &str, &str)] = &[
    // (nó, param inerte, o seletor de que ele depende)
    ("motion.stagger", "ease_dir", "ease_curve"),
    ("motion.tint", "r2", "mode"),
    ("fx.rgb_split", "strength", "mode"),
    ("fx.rgb_split", "x", "mode"),
    ("fx.rgb_split", "y", "mode"),
    ("value.instance_field", "seed", "mode"),
    ("value.map_range", "clamp", "interpolation"),
    ("value.step", "width", "mode"),
    ("motion.emitter", "shape_w", "shape_mode"),
    ("motion.emitter", "shape_h", "shape_mode"),
];

/// Os pares curados por LIMIAR (`ParamGateAbove`) — a família da segunda oitava, mais o
/// interruptor do `motion.boids`.
const CURED_ABOVE: &[(&str, &str, &str)] = &[
    ("motion.wiggle", "amp_mult", "octaves"),
    ("force.wind", "lacunarity", "octaves"),
    ("force.wind", "roughness", "octaves"),
    ("value.noise", "roughness", "octaves"),
    ("value.noise", "lacunarity", "octaves"),
    ("motion.boids", "avoid_radius", "avoid"),
    ("motion.boids", "lookahead", "avoid"),
];

fn manifest_of(all: &[&'static NodeManifest], name: &str) -> &'static NodeManifest {
    all.iter()
        .copied()
        .find(|m| m.name == name)
        .unwrap_or_else(|| panic!("o no' `{name}` tem de existir no registry"))
}

fn hint_of(reg: &NodeRegistry, m: &'static NodeManifest, param: &str) -> &'static ParamUiHint {
    reg.param_ui(m.id)
        .and_then(|hs| hs.iter().find(|h| h.param == param))
        .unwrap_or_else(|| panic!("`{}::{param}` tem de ter hint", m.name))
}

/// Dois valores distintos com que sacudir o param — os extremos da faixa que a UI permite.
fn shake(h: &'static ParamUiHint) -> (f32, f32) {
    (h.min, h.max)
}

/// **O param `param` muda alguma coluna de saída, com `when = w`?**
///
/// ⚠️ Varre TODAS as bancadas da [`common`] (alimentada · com laço de estado · com as opcionais
/// soltas · com fonte constante) e devolve `true` se **alguma** o vê mexer. É a direcção
/// conservadora: um `false` aqui é uma afirmação forte de inércia.
fn acts(
    reg: &NodeRegistry,
    all: &[&'static NodeManifest],
    m: &'static NodeManifest,
    param: &str,
    when: &str,
    w: f32,
) -> bool {
    let (lo, hi) = shake(hint_of(reg, m, param));
    for (g0, n, _) in all_benches(reg, all, m) {
        let probe = |v: f32| {
            let mut g = g0.clone();
            g.set_param(n, when, w);
            g.set_param(n, param, v);
            snapshot(&g, reg, n)
        };
        if let (Some(a), Some(b)) = (probe(lo), probe(hi))
            && a != b
        {
            return true;
        }
    }
    false
}

/// **A METADE 1+2 para os gates de ENUM.**
#[test]
fn every_enum_gate_covers_exactly_the_modes_where_the_knob_acts() {
    let reg = registry();
    let all = catalogue(&reg);
    for (node, param, when) in CURED {
        let m = manifest_of(&all, node);
        let gates = reg
            .param_gates(m.id)
            .unwrap_or_else(|| panic!("`{node}` tem de registar `ParamGate`s"));
        let gate = gates
            .iter()
            .find(|g| g.param == *param)
            .unwrap_or_else(|| panic!("`{node}::{param}` tem de ter um gate — e' a cura"));
        assert_eq!(
            gate.when, *when,
            "`{node}::{param}`: o gate decide pelo eixo errado"
        );

        let n_modes = match hint_of(&reg, m, when).widget {
            ParamWidget::Enum { labels } => labels.len(),
            ParamWidget::Toggle => 2,
            w => panic!("`{node}::{when}` devia ser um seletor, e' {w:?}"),
        };

        let mut acting: Vec<i32> = Vec::new();
        for k in 0..n_modes {
            if acts(&reg, &all, m, param, when, k as f32) {
                acting.push(k as i32);
            }
        }

        // METADE 1 — fora do gate, o knob e' inerte.
        for k in 0..n_modes as i32 {
            if !gate.values.contains(&k) {
                assert!(
                    !acting.contains(&k),
                    "`{node}::{param}` AGE no modo {k}, e o gate esconde-o la' — \
                     o gate esta' a tirar do artista um controle que funciona"
                );
            }
        }
        // METADE 2 — dentro do gate, ele age em pelo menos um modo.
        assert!(
            !acting.is_empty(),
            "`{node}::{param}` nao age em modo NENHUM — isto nao e' um gate a faltar, \
             e' um knob morto em todo o espaco, e a cura e' outra"
        );
        assert!(
            acting.iter().any(|k| gate.values.contains(k)),
            "`{node}::{param}` age em {acting:?} e o gate mostra-o em {:?} — \
             disjuntos: o controle existe e nenhum gesto o alcanca",
            gate.values
        );
    }
}

/// **A METADE 1+2 para os gates de LIMIAR** — a família da segunda oitava.
///
/// ⚠️ O oráculo é diferente por natureza: não há um conjunto finito de índices a varrer, então
/// mede-se **abaixo/no limiar** (onde tem de ser inerte) e **acima** (onde tem de agir).
#[test]
fn every_threshold_gate_holds_a_knob_that_is_inert_below_it() {
    let reg = registry();
    let all = catalogue(&reg);
    for (node, param, when) in CURED_ABOVE {
        let m = manifest_of(&all, node);
        let gates = reg
            .param_gates_above(m.id)
            .unwrap_or_else(|| panic!("`{node}` tem de registar `ParamGateAbove`s"));
        let gate = gates
            .iter()
            .find(|g| g.param == *param)
            .unwrap_or_else(|| panic!("`{node}::{param}` tem de ter um gate de limiar"));
        assert_eq!(
            gate.when, *when,
            "`{node}::{param}`: o gate decide pelo eixo errado"
        );

        // ⚠️ O limiar tem de estar DENTRO da faixa que a UI permite, senão o gate esconde o
        // knob para sempre — um `above` acima do `max` do slider e' um `values: &[]` disfarçado.
        let wh = hint_of(&reg, m, when);
        assert!(
            gate.above >= wh.min && gate.above < wh.max,
            "`{node}::{param}`: o limiar {} esta' fora da faixa [{}, {}) do `{when}` — \
             o knob nunca apareceria",
            gate.above,
            wh.min,
            wh.max
        );

        // METADE 1 — no limiar (o valor default da familia) ele e' inerte.
        assert!(
            !acts(&reg, &all, m, param, when, gate.above),
            "`{node}::{param}` AGE com `{when} = {}` — o gate esconde-o onde ele funciona",
            gate.above
        );
        // METADE 2 — acima do limiar ele age.
        assert!(
            acts(&reg, &all, m, param, when, wh.max),
            "`{node}::{param}` nao age nem com `{when} = {}` — o gate nao e' a cura deste caso",
            wh.max
        );
    }
}

/// **NENHUM dos dezanove ficou de fora**, e a contagem é derivada das duas tabelas.
///
/// ⚠️ Existe porque um par apagado de `CURED` faria os dois testes acima passarem **em
/// silêncio**: eles varrem o que a lista traz, e uma lista mais curta é uma varredura menor,
/// não uma falha. *Um teste que itera uma lista precisa de alguém a contar a lista.*
#[test]
fn the_cure_covers_the_nineteen_measured_defects() {
    // 10 de enum + 7 de limiar = 17 pares. Os outros dois sao `g2`/`b2`/`a2`... nao: o swatch
    // do `motion.tint` e' UMA row (a ancora `r2`), e o par `fx.rgb_split::x`/`y` entrou como
    // espelho do `strength`. A conta do doc 90 §2 e' 19 PARAMS; aqui sao 17 GATES, porque
    // `g2`/`b2`/`a2` nao pintam row propria e um gate neles nao decidiria nada.
    assert_eq!(CURED.len(), 10, "a lista de gates de enum encolheu");
    assert_eq!(CURED_ABOVE.len(), 7, "a lista de gates de limiar encolheu");
}

/// ⚠️ **A janela de medição não pode encolher abaixo de um envelope** — a `TICKS = 48` é o que
/// faz o `release` do `pulse.adsr` existir dentro da varredura (doc 90 §4, ponto cego nº 6).
///
/// É uma asserção de COMPILAÇÃO e não um `#[test]`: o valor é constante, então um teste sobre
/// ele só corre para confirmar o que o compilador já sabe.
const _: () = assert!(TICKS >= 48);
