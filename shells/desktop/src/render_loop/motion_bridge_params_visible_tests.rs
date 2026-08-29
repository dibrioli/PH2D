//! **A OUTRA METADE DA CURA: o painel não PINTA o knob inerte.**
//!
//! O gate irmão (`param_gates_are_exact`, em `ph2d-node-registry-init`) prova que cada
//! `ParamGate` cobre exactamente os modos em que o knob **age** — ele mede o KERNEL. Este prova
//! que o construtor de rows **honra** essa tabela — ele mede o PAINEL.
//!
//! ⚠️ **As duas são necessárias e nenhuma implica a outra.** Uma tabela correcta que o
//! construtor não lê continua a pintar o controle morto (o defeito original, exactamente); e um
//! construtor que lê uma tabela errada esconde um controle que funciona (o defeito oposto, que
//! é pior — o artista fica sem gesto nenhum para o alcançar).
//!
//! ⚠️ Esta metade lê a tabela como VERDADE. Ela não re-mede quais modos agem — isso é o
//! trabalho do gate do kernel, e repeti-lo aqui seria medir duas vezes a mesma coisa e não
//! medir uma vez a outra.

use super::params_visible::Visibility;
use ph2d_node_registry::{NodeRegistry, ParamGate, ParamGateAbove};
use ph2d_nodegraph::node::NodeManifest;

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("os nos registram");
    reg
}

fn manifest_of(reg: &NodeRegistry, name: &str) -> &'static NodeManifest {
    reg.manifests()
        .find(|m| m.name == name)
        .unwrap_or_else(|| panic!("o no' `{name}` tem de existir"))
}

/// As onze crates curadas em 2026-08-22 (doc 90 §2). ⚠️ **A lista é de NÓS, e os params saem
/// das tabelas do próprio nó** — escrevê-los aqui seria uma terceira cópia da mesma verdade.
const CURED_NODES: &[&str] = &[
    "motion.stagger",
    "motion.tint",
    "motion.wiggle",
    "motion.emitter",
    "motion.boids",
    "force.wind",
    "fx.rgb_split",
    "value.instance_field",
    "value.map_range",
    "value.noise",
    "value.step",
];

/// **TODO GATE DE ENUM DAS CRATES CURADAS ESCONDE E MOSTRA A ROW.**
///
/// Para cada `ParamGate` registado: com o seletor num índice de FORA da lista, a row não é
/// mostrada; com ele num índice DE DENTRO, é. É a leitura do construtor, não da tabela.
#[test]
fn a_gated_row_is_hidden_outside_its_modes_and_shown_inside() {
    let reg = registry();
    let mut checked = 0usize;
    for node in CURED_NODES {
        let m = manifest_of(&reg, node);
        let vis = Visibility::of(&reg, m.id);
        let Some(gates) = reg.param_gates(m.id) else {
            continue;
        };
        for g in gates {
            let inside = *g.values.first().unwrap_or_else(|| {
                panic!("`{node}::{}`: um gate vazio esconde para SEMPRE", g.param)
            });
            // Um índice de fora: o primeiro `0..=64` que a lista não contém. A faixa é
            // folgada de propósito — nenhum seletor deste catálogo tem 65 opções.
            let outside = (0..=64)
                .find(|k| !g.values.contains(k))
                .expect("algum indice tem de estar fora da lista");
            let at = |k: i32| {
                let value_of = |p: &str| {
                    if p == g.when {
                        k as f32
                    } else {
                        m.param_default(p).unwrap_or(0.0)
                    }
                };
                vis.shows(g.param, &value_of, &|_| false)
            };
            assert!(
                !at(outside),
                "`{node}::{}` aparece com `{} = {outside}`, fora do gate — o knob morto ficou",
                g.param,
                g.when
            );
            assert!(
                at(inside),
                "`{node}::{}` NAO aparece com `{} = {inside}`, dentro do gate — \
                 o controle existe e nenhum gesto o alcanca",
                g.param,
                g.when
            );
            checked += 1;
        }
    }
    // ⚠️ Um laço sobre uma lista precisa de alguém a contar a lista: sem isto, apagar uma
    // tabela de gates faria este teste passar varrendo menos.
    assert!(
        checked >= 13,
        "esperava pelo menos 13 gates de enum nas crates curadas, vi {checked}"
    );
}

/// **AS DEZ CURAS DE ENUM ESTÃO NOS NÓS QUE A AUDITORIA NOMEOU.**
///
/// ⚠️ **Este teste nasceu de uma prova de mutação que ele reprovou.** Desregistar o gate do
/// `value.step` deixava o teste acima **VERDE**: ele itera `reg.param_gates(...)` e um nó sem
/// tabela cai no `continue` — a varredura fica menor, e menor não é vermelho. O piso
/// `checked >= 13` também não o apanhava, porque os outros dez gates chegavam para o satisfazer.
///
/// *Um laço que salta o que não encontra não pode provar que alguma coisa existe.* A contagem é
/// um piso; **a lista nomeada é o gate**.
#[test]
fn the_enum_cures_live_on_the_nodes_the_audit_named() {
    let reg = registry();
    let expected: &[(&str, &[&str])] = &[
        ("motion.stagger", &["ease_dir"]),
        ("motion.tint", &["r2"]),
        ("fx.rgb_split", &["strength", "x", "y"]),
        ("value.instance_field", &["seed"]),
        ("value.map_range", &["clamp"]),
        ("value.step", &["width"]),
        ("motion.emitter", &["shape_w", "shape_h"]),
    ];
    for (node, params) in expected {
        let m = manifest_of(&reg, node);
        let gates: &[ParamGate] = reg
            .param_gates(m.id)
            .unwrap_or_else(|| panic!("`{node}` tem de registar `ParamGate`s — a cura sumiu"));
        for p in *params {
            assert!(
                gates.iter().any(|g| g.param == *p),
                "`{node}::{p}` perdeu o gate de enum"
            );
        }
    }
}

/// **TODO GATE DE LIMIAR ESCONDE NO NEUTRO E MOSTRA ACIMA DELE.**
#[test]
fn a_threshold_gated_row_is_hidden_at_the_neutral_and_shown_above_it() {
    let reg = registry();
    let mut checked = 0usize;
    for node in CURED_NODES {
        let m = manifest_of(&reg, node);
        let vis = Visibility::of(&reg, m.id);
        let Some(gates) = reg.param_gates_above(m.id) else {
            continue;
        };
        for g in gates {
            let at = |v: f32| {
                let value_of = |p: &str| {
                    if p == g.when {
                        v
                    } else {
                        m.param_default(p).unwrap_or(0.0)
                    }
                };
                vis.shows(g.param, &value_of, &|_| false)
            };
            assert!(
                !at(g.above),
                "`{node}::{}` aparece com `{} = {}` (o limiar) — e' onde ele e' inerte",
                g.param,
                g.when,
                g.above
            );
            assert!(
                at(g.above + 1.0),
                "`{node}::{}` NAO aparece acima do limiar — o gate esconde-o sempre",
                g.param
            );
            checked += 1;
        }
    }
    assert_eq!(
        checked, 7,
        "as sete curas de limiar (5 do fBm + 2 do bando) tem de estar todas registadas"
    );
}

/// **UM NÓ SEM TABELA NENHUMA MOSTRA TUDO** — o controle negativo.
///
/// ⚠️ Sem ele, um `Visibility::shows` que devolvesse `false` para tudo faria os dois testes
/// acima reprovarem na metade certa e passarem na outra por acidente; e um que devolvesse
/// `true` para tudo passaria as duas metades «mostra dentro» sem esconder nada.
#[test]
fn a_node_with_no_gate_table_shows_every_param() {
    let reg = registry();
    // ⚠️ **O nó é DERIVADO, não nomeado.** Ele era `motion.grid` escrito à mão, e o dia em
    // que aquele nó ganhou uma tabela (a forma do domínio, doc 89 folha 01) este controle
    // reprovou sobre produto correcto. *Uma lista escrita à mão ao lado de um predicado é a
    // segunda resposta à mesma pergunta, e a que envelhece.*
    let m = reg
        .manifests()
        .find(|m| {
            m.params.len() >= 2
                && reg.param_gates(m.id).is_none()
                && reg.param_gates_above(m.id).is_none()
        })
        .expect("ha' pelo menos um no' SEM tabela de gates -- senao este controle e' vazio");
    let vis = Visibility::of(&reg, m.id);
    for p in m.params {
        assert!(
            vis.shows(p.name, &|_| 0.0, &|_| false),
            "`motion.grid::{}` sumiu sem gate nenhum a esconde-lo",
            p.name
        );
    }
}

/// **A ÂNCORA DO SWATCH É O QUE DECIDE** — o caso do `motion.tint`, que não é como os outros.
///
/// ⚠️ Os quatro canais de uma cor não pintam quatro rows: o construtor dobra-os num swatch só
/// (`consumed`, derivado dos grupos de cor e **não** do filtro de visibilidade), e a row é
/// emitida pela âncora. Gatear a âncora tira o swatch; gatear os outros três não decidiria
/// nada. Este teste fixa essa assimetria, que de outro modo pareceria uma cura incompleta.
#[test]
fn gating_the_colour_anchor_is_what_hides_the_whole_swatch() {
    let reg = registry();
    let m = manifest_of(&reg, "motion.tint");
    let gates: &[ParamGate] = reg.param_gates(m.id).expect("o tint declara gates");
    assert!(
        gates.iter().any(|g| g.param == "r2"),
        "a ancora `r2` e' que tem de estar gateada"
    );
    for ch in ["g2", "b2", "a2"] {
        assert!(
            !gates.iter().any(|g| g.param == ch),
            "`{ch}` nao precisa de gate: ele nao pinta row propria"
        );
    }
    let vis = Visibility::of(&reg, m.id);
    let solid = |p: &str| {
        let value_of = |q: &str| {
            if q == "mode" {
                0.0
            } else {
                m.param_default(q).unwrap_or(0.0)
            }
        };
        vis.shows(p, &value_of, &|_| false)
    };
    assert!(!solid("r2"), "em Solid o swatch `End` nao aparece");
}

/// **AS SETE CURAS DE LIMIAR SÃO AS QUE O DOC 90 CONTA** — a lista, nomeada.
///
/// ⚠️ Existe porque `assert_eq!(checked, 7)` acima conta **quantas**, e não **quais**: mover um
/// gate de um nó para outro manteria a contagem e mudaria o produto.
#[test]
fn the_threshold_cures_live_on_the_nodes_the_audit_named() {
    let reg = registry();
    let expected: &[(&str, &[&str])] = &[
        ("motion.wiggle", &["amp_mult"]),
        ("force.wind", &["lacunarity", "roughness"]),
        ("value.noise", &["roughness", "lacunarity"]),
        ("motion.boids", &["avoid_radius", "lookahead"]),
    ];
    for (node, params) in expected {
        let m = manifest_of(&reg, node);
        let gates: &[ParamGateAbove] = reg
            .param_gates_above(m.id)
            .unwrap_or_else(|| panic!("`{node}` tem de registar gates de limiar"));
        for p in *params {
            assert!(
                gates.iter().any(|g| g.param == *p),
                "`{node}::{p}` perdeu o gate de limiar"
            );
        }
    }
}

/// ⭐⭐⭐ **AS DUAS METADES DO L-SYSTEM NUNCA APARECEM JUNTAS, E NENHUM MODO FICA SEM AS SUAS.**
///
/// ⚠️ Report do Enio, 2026-08-29: *"eu havia te pedido o L-System estado da arte (…) O Blender
/// e Houdini usam Axiom e Rules?"*. A medição: o **Houdini** sim (`Premise` + `Rule 1..N`); o
/// **Blender não tem L-System nenhum** — a árvore dele é o *Sapling Tree Gen* (Weber & Penn),
/// sliders puros —, e o padrão da indústria (SpeedTree) é uma hierarquia de geradores com
/// sliders. ⇒ a gramática é o estado da arte do MOTOR, não da INTERFACE.
///
/// O `Mode` é a cura, e ele só é um MODO se o painel de facto trocar de metade:
/// - em `Guided` a gramática é derivada ⇒ as caixas mostrariam texto que **o nó não lê**;
/// - em `Grammar` os quatro números de forma não alimentam nada ⇒ seriam knobs mortos.
///
/// ⚠️ **As DUAS direcções, e a segunda é a que a família dos knobs mortos esquece**: um gate
/// que só afirmasse *"não aparece o que não age"* passaria com o painel a esconder tudo.
#[test]
fn the_lsystem_shows_exactly_one_authoring_half_per_mode() {
    use ph2d_node_source_lsystem as ls;
    let reg = registry();
    let m = manifest_of(&reg, ls::MANIFEST.name);
    let vis = Visibility::of(&reg, m.id);

    const GRAMMAR_HALF: &[&str] = &[ls::AXIOM_PARAM, ls::RULES_PARAM, ls::param::PRESET];
    const SHAPE_HALF: &[&str] = &[
        ls::param::BRANCHES,
        ls::param::SEGMENTS,
        ls::param::VARIATION,
        ls::param::BEND,
    ];

    for (mode, shown, hidden) in [
        (ls::MODE_GUIDED, SHAPE_HALF, GRAMMAR_HALF),
        (ls::MODE_GRAMMAR, GRAMMAR_HALF, SHAPE_HALF),
    ] {
        let value_of = |p: &str| {
            if p == ls::param::MODE {
                mode as f32
            } else {
                0.0
            }
        };
        let has_text = |_: &str| true;
        for p in shown {
            assert!(
                vis.shows(p, &value_of, &has_text),
                "modo {mode}: `{p}` tem de aparecer — sem ele o modo nao tem autoria nenhuma"
            );
        }
        for p in hidden {
            assert!(
                !vis.shows(p, &value_of, &has_text),
                "modo {mode}: `{p}` esta' a ser pintado e nao alimenta nada"
            );
        }
    }

    // ⚠️ E o CONTROLE do próprio `Mode`: ele aparece nos DOIS, senão o artista entra num modo
    // e fica sem o controle que o tira de lá.
    for mode in [ls::MODE_GUIDED, ls::MODE_GRAMMAR] {
        let value_of = |p: &str| {
            if p == ls::param::MODE {
                mode as f32
            } else {
                0.0
            }
        };
        assert!(
            vis.shows(ls::param::MODE, &value_of, &|_: &str| true),
            "o proprio `Mode` sumiu no modo {mode}"
        );
    }
}
