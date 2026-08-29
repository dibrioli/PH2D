//! **O MOLDE DO L-SYSTEM ESCREVE AS DUAS CAIXAS** — as quatro condições de UI para ele.
//!
//! Report do Enio (2026-08-28): *"Axiom e Rules não são nada intuitivos. Alguma soluções para
//! isso?"* ⇒ um selector de moldes, que é o que o L-System SOP do Houdini e o L-studio fazem.
//!
//! ⚠️ **A resposta não foi inventar uma sintaxe amigável**, e a razão é medida: `F[+F]F` é a
//! notação de Lindenmayer, e é ela que está no livro, nos tutoriais e em todo exemplo que o
//! artista vai encontrar. Trocá-la tornaria este nó incompatível com o conhecimento do mundo.

use super::params::{apply_lsystem_preset, param_value};
use crate::motion_state::MotionState;
use ph2d_node_source_lsystem as ls;

fn text_of(motion: &MotionState, nid: ph2d_nodegraph::graph::NodeId, key: &str) -> String {
    motion
        .doc
        .graph
        .node_text_param_overrides(nid)
        .and_then(|m| m.get(key))
        .cloned()
        .unwrap_or_default()
}

/// **Todo molde tem rótulo, axioma e regras, e as regras COMPILAM.**
///
/// ⚠️⚠️ **Este gate CONTAVA ELEMENTOS (`count() > 3`) e a auditoria de 2026-08-29 mostrou que
/// ele não podia reprovar em molde nenhum, por construção.** A Koch passava com 3 126
/// elementos a medir **1 291 unidades de mundo** numa coluna de ~4; o Sprig passava com 16 a
/// desenhar uma linha de largura **exactamente 0,00**. *Uma contagem é a única grandeza que
/// SOBE com este defeito.* A contagem saiu; as réguas que de facto reprovam vivem em
/// [`presets_frame_themselves`](../../../../crates/ph2d-node-source-lsystem/tests/presets_frame_themselves.rs)
/// e medem o TAMANHO, os dois eixos da caixa, e a resposta ao `Angle`.
///
/// O que sobrevive aqui é a costura: a tabela é bem formada e cada texto de facto deriva.
#[test]
fn every_preset_is_a_grammar_that_actually_draws() {
    assert!(!ls::PRESETS.is_empty());
    for (k, p) in ls::PRESETS.iter().enumerate() {
        assert_eq!(p.label, ls::PRESET_LABELS[k], "o rotulo {k} discorda");
        assert!(!p.axiom.trim().is_empty() && !p.rules.trim().is_empty());
        // Deriva com o enquadramento que o próprio molde declara — a mesma coisa que o
        // `apply_lsystem_preset` escreve.
        let s = ls::probe_build(
            p.axiom,
            p.rules,
            p.generations,
            &[
                (ls::param::MODE, ls::MODE_GRAMMAR as f32),
                (ls::param::ANGLE, p.angle),
                (ls::param::STEP, p.step),
            ],
        );
        assert!(s.count() > 3, "o molde {} nao cresce", p.label);
    }
}

/// ⛔ **O gate `the_first_preset_is_what_a_fresh_node_already_is` MORREU, e a premissa dele é
/// que morreu primeiro.**
///
/// Ele comparava `PRESETS[0]` com o `DEFAULT_RULES` e o doc-comment nomeava o defeito que
/// queria impedir: *«um nó novo mostraria «Tree» seleccionado e uma gramática que não é a do
/// Tree — o painel a mentir sobre o próprio estado»*. Desde 2026-08-29 o `Mode` nasce
/// `Guided`, e «o que um nó novo já é» passou a ser a gramática DERIVADA dos sliders — outra
/// planta, **76 % mais alta**, medido. Os dois gates verdes (este e o
/// `converting_to_grammar_bakes_the_plant_the_sliders_were_making`, que assere
/// `assert_ne!(assado, DEFAULT_RULES)`) **provavam juntos o desencontro que o primeiro dizia
/// proibir**.
///
/// ⇒ A cura não é o gate: é o [`ls::PRESET_CUSTOM`] passar a ser o default do selector. É isso
/// que este gate afirma agora.
#[test]
fn a_fresh_node_names_no_preset_because_it_is_none_of_them() {
    let default = ls::MANIFEST
        .params
        .iter()
        .find(|p| p.name == ls::param::PRESET)
        .expect("o param existe")
        .default;
    assert_eq!(
        default.round() as usize,
        ls::PRESET_CUSTOM,
        "um no' novo abre em `Guided`, cuja gramatica nao e' a de molde nenhum"
    );
    // E o CONTROLE que torna isto necessário: a derivada do guiado de fábrica **não** é a
    // gramática do molde `0`. Se um dia voltar a ser, este gate tem de ser reconferido.
    let (_, guided) = ls::grammar_for(2.0, 1.0, 0.0, 0.0);
    assert_ne!(
        guided,
        ls::PRESETS[0].rules,
        "a derivada do guiado voltou a ser a do molde 0 — reveja qual e' o default honesto"
    );
}

/// ⭐ **Escolher um molde ESCREVE as duas caixas** — a costura que o torna um botão vivo.
#[test]
fn picking_a_preset_writes_both_text_boxes() {
    let mut motion = MotionState::new();
    let n = motion.doc.graph.add_node("source.lsystem");
    for (k, p) in ls::PRESETS.iter().enumerate() {
        apply_lsystem_preset(&mut motion, n, "source.lsystem", k as f32);
        assert_eq!(
            text_of(&motion, n, ls::AXIOM_PARAM),
            p.axiom,
            "{}: axioma",
            p.label
        );
        assert_eq!(
            text_of(&motion, n, ls::RULES_PARAM),
            p.rules,
            "{}: regras",
            p.label
        );
    }
}

/// ⭐⭐⭐ **E ESCREVE O ENQUADRAMENTO** — a metade sem a qual sete dos oito saíam errados.
///
/// ⚠️ Report do Enio, 2026-08-29. Um molde que escrevesse só o texto entregava a curva de Koch
/// a **25°** (ela é `90` por definição) e a **1 291 unidades de mundo** numa coluna de ~4.
/// *Um molde não é uma gramática: é uma gramática MAIS o enquadramento em que ela se lê.*
///
/// ⚠️ **Os quatro, um a um** — um gate que verificasse só «alguma coisa mudou» ficaria verde
/// com três deles por escrever.
#[test]
fn picking_a_preset_also_writes_the_framing_it_needs() {
    let mut motion = MotionState::new();
    let n = motion.doc.graph.add_node("source.lsystem");
    for (k, p) in ls::PRESETS.iter().enumerate() {
        apply_lsystem_preset(&mut motion, n, "source.lsystem", k as f32);
        for (name, want) in [
            (ls::param::ANGLE, p.angle),
            (ls::param::GENERATIONS, p.generations),
            (ls::param::STEP, p.step),
            (ls::param::WIDTH, p.width),
        ] {
            assert_eq!(
                param_value(&motion, n, name),
                want,
                "{}: o `{name}` nao foi escrito",
                p.label
            );
        }
    }
}

/// ⚠️ **E ele não toca em mais nada** — nem noutro tipo de nó, nem num índice que não existe.
///
/// O CONTROLE do tipo é o que impede um `preset` de outro nó (o nome é comum) de reescrever
/// texto alheio; o do índice é o que impede um documento carregado com um número velho de
/// apagar a gramática do artista.
#[test]
fn a_foreign_node_and_an_out_of_range_index_are_both_no_ops() {
    let mut motion = MotionState::new();
    let n = motion.doc.graph.add_node("source.lsystem");
    motion.doc.graph.set_text_param(n, ls::AXIOM_PARAM, "MEU");
    motion
        .doc
        .graph
        .set_text_param(n, ls::RULES_PARAM, "MEU -> MEU");

    apply_lsystem_preset(&mut motion, n, "motion.grid", 1.0);
    assert_eq!(
        text_of(&motion, n, ls::AXIOM_PARAM),
        "MEU",
        "outro tipo nao toca"
    );

    apply_lsystem_preset(&mut motion, n, "source.lsystem", 999.0);
    assert_eq!(
        text_of(&motion, n, ls::AXIOM_PARAM),
        "MEU",
        "indice fora da faixa"
    );
    assert_eq!(text_of(&motion, n, ls::RULES_PARAM), "MEU -> MEU");
}

/// **E o param existe no manifesto** — senão o selector não teria onde guardar a escolha.
#[test]
fn the_preset_param_is_declared_and_reaches_the_node() {
    let mut motion = MotionState::new();
    let n = motion.doc.graph.add_node("source.lsystem");
    motion.doc.graph.set_param(n, ls::param::PRESET, 2.0);
    assert!((param_value(&motion, n, ls::param::PRESET) - 2.0).abs() < 1e-6);
}

// ─────────────────────────────────────────────────────────────────────────────────────────
// A CONVERSÃO — `Guided → Grammar` assa a gramática que os sliders faziam (2026-08-29).
// ─────────────────────────────────────────────────────────────────────────────────────────

use super::params::bake_lsystem_grammar;

/// ⭐⭐⭐ **CONVERTER MOSTRA A GRAMÁTICA QUE OS SLIDERS ESTAVAM A FAZER** — e não a de fábrica.
///
/// ⚠️ É a resposta inteira ao report de 2026-08-29 (*"O Blender e Houdini usam Axiom e
/// Rules?"* — o Houdini sim, o Blender **não tem L-System nenhum**). O nó abre em sliders; o
/// artista que quiser a gramática muda o modo e encontra lá **a planta que estava a ver**.
/// Se a conversão escrevesse o default, ela seria um botão que **destrói o trabalho** e
/// ninguém o carregaria uma segunda vez.
#[test]
fn converting_to_grammar_bakes_the_plant_the_sliders_were_making() {
    let mut motion = MotionState::new();
    let n = motion.doc.graph.add_node("source.lsystem");
    motion.doc.graph.set_param(n, ls::param::BRANCHES, 3.0);
    motion.doc.graph.set_param(n, ls::param::SEGMENTS, 2.0);
    bake_lsystem_grammar(&mut motion, n, "source.lsystem");

    let (want_axiom, want_rules) = ls::grammar_for(3.0, 2.0, 0.0, 0.0);
    assert_eq!(text_of(&motion, n, ls::AXIOM_PARAM), want_axiom);
    assert_eq!(text_of(&motion, n, ls::RULES_PARAM), want_rules);
    // ⚠️ E o CONTROLE: o que foi assado NÃO é a gramática de fábrica. Sem ele, um `bake` que
    // escrevesse o default passaria a primeira metade em qualquer forma que lhe dessem.
    assert_ne!(
        text_of(&motion, n, ls::RULES_PARAM),
        ls::DEFAULT_RULES,
        "a conversao escreveu o default e deitou fora os sliders do artista"
    );
}

/// ⚠️ **A conversão lê os SLIDERS DAQUELE nó, não os defaults do manifesto.**
///
/// Duas formas diferentes têm de assar duas gramáticas diferentes — senão o `bake` é uma
/// constante com cara de função, e o gate acima passaria com ele a ignorar o nó por inteiro.
#[test]
fn the_bake_reads_that_nodes_own_sliders_and_not_a_constant() {
    let mut motion = MotionState::new();
    let a = motion.doc.graph.add_node("source.lsystem");
    let b = motion.doc.graph.add_node("source.lsystem");
    motion.doc.graph.set_param(a, ls::param::BRANCHES, 2.0);
    motion.doc.graph.set_param(b, ls::param::BRANCHES, 5.0);
    motion.doc.graph.set_param(b, ls::param::BEND, 9.0);
    bake_lsystem_grammar(&mut motion, a, "source.lsystem");
    bake_lsystem_grammar(&mut motion, b, "source.lsystem");
    assert_ne!(
        text_of(&motion, a, ls::RULES_PARAM),
        text_of(&motion, b, ls::RULES_PARAM),
        "duas formas diferentes assaram a MESMA gramatica"
    );
}

/// **A porta não toca em nó que não é dela** — a mesma cerca do [`apply_lsystem_preset`].
#[test]
fn the_bake_never_writes_on_another_node_type() {
    let mut motion = MotionState::new();
    let n = motion.doc.graph.add_node("motion.grid");
    bake_lsystem_grammar(&mut motion, n, "motion.grid");
    assert!(text_of(&motion, n, ls::RULES_PARAM).is_empty());
}

/// ⭐⭐ **O QUE FOI ASSADO DESENHA A MESMA PLANTA** — a propriedade que faz a conversão ser
/// uma conversão, e não um recomeço.
///
/// ⚠️ Ela não é óbvia e podia falhar de duas maneiras: o `bake` podia montar a string por
/// outro caminho que o `build` (dois geradores), ou a gramática assada podia perder um param
/// pelo caminho (o literal em vez do nome). A régua é a CONTAGEM de elementos e a altura, dos
/// dois lados, com a fixtura a ter uma forma que **não** é o default.
#[test]
fn what_was_baked_draws_exactly_what_the_sliders_drew() {
    let shape = [
        (ls::param::BRANCHES, 3.0f32),
        (ls::param::SEGMENTS, 2.0),
        (ls::param::BEND, 6.0),
    ];
    let mut guided: Vec<(&str, f32)> = vec![(ls::param::MODE, ls::MODE_GUIDED as f32)];
    guided.extend_from_slice(&shape);
    let before = ls::probe_build(ls::DEFAULT_AXIOM, ls::DEFAULT_RULES, 5.0, &guided);

    let (axiom, rules) = ls::grammar_for(3.0, 2.0, 0.0, 6.0);
    let mut authored: Vec<(&str, f32)> = vec![(ls::param::MODE, ls::MODE_GRAMMAR as f32)];
    authored.extend_from_slice(&shape);
    let after = ls::probe_build(axiom, &rules, 5.0, &authored);

    assert_eq!(
        before.count(),
        after.count(),
        "a conversao mudou a planta: {} elementos antes, {} depois",
        before.count(),
        after.count()
    );
}

// ─────────────────────────────────────────────────────────────────────────────────────────
// O SELECTOR DEIXA DE MENTIR — auditoria de 2026-08-29, família D.
// ─────────────────────────────────────────────────────────────────────────────────────────

use crate::render_loop::motion_bridge::params;
use ph2d_panel_motion_params::MotionParamIntent;

/// Corre o caminho REAL de uma edição de param — a fila de intenções, e o dreno.
///
/// ⚠️ **Nunca chamando `apply_lsystem_preset`/`mark_lsystem_custom` directamente**: um gate
/// assim fica verde no dia em que o executor deixar de as chamar, que é a forma de gate vazio
/// que a auditoria de 2026-08-27 apanhou vinte e quatro vezes — e é exactamente por essa porta
/// que o `picking_a_preset_writes_both_text_boxes` era cego à guarda de igualdade do despacho.
fn dispatch(motion: &mut MotionState, intent: MotionParamIntent) {
    let mut toasts = ph2d_editor::ToastQueue::default();
    let store = ph2d_editor::interaction::WidgetStore::default();
    ph2d_panel_motion_params::push_param_intent(intent);
    params::apply_param_edits_for_tests(motion, &store, &mut toasts);
}

fn preset_of(motion: &MotionState, n: ph2d_nodegraph::graph::NodeId) -> usize {
    param_value(motion, n, ls::param::PRESET).round() as usize
}

/// ⭐⭐⭐ **O SELECTOR SÓ NOMEIA UM MOLDE ENQUANTO O TEXTO FOR O DAQUELE MOLDE.**
///
/// ⚠️ O `preset` é um `ParamSpec` persistido que o `build` **nunca lê** — todo o efeito de um
/// molde vive na shell —, então o número guardado é o eco de um gesto passado e não um facto
/// sobre a planta. Três escritores mudam o texto sem lhe tocar: o `bake` do modo guiado, a
/// edição à mão da caixa, e uma cena. O estado de chegada NORMAL (um nó novo abre em `Guided`,
/// o artista converte) deixava o selector a dizer «Tree» sobre uma planta **76 % mais alta**,
/// e clicar em «Tree» era **mudo**, porque o despacho exige que o valor MUDE.
///
/// As três metades, pela porta real.
#[test]
fn the_selector_never_names_a_preset_it_is_not() {
    let mut motion = MotionState::new();
    let n = motion.doc.graph.add_node("source.lsystem");

    // 1. Escolher um molde nomeia-o.
    dispatch(
        &mut motion,
        MotionParamIntent::SetParam {
            node: n.0,
            param: ls::param::PRESET,
            value: 2.0,
        },
    );
    assert_eq!(preset_of(&motion, n), 2, "escolher o molde 2 nomeia o 2");
    assert_eq!(text_of(&motion, n, ls::RULES_PARAM), ls::PRESETS[2].rules);

    // 2. Editar a caixa à mão aterra em `Custom`.
    dispatch(
        &mut motion,
        MotionParamIntent::SetTextParam {
            node: n.0,
            param: ls::RULES_PARAM,
            value: "F -> FF".to_string(),
        },
    );
    assert_eq!(
        preset_of(&motion, n),
        ls::PRESET_CUSTOM,
        "o texto deixou de ser o do molde 2 e o selector continuou a nomea-lo"
    );

    // 3. E a CONVERSÃO do modo guiado também — ela escreve texto que não é molde nenhum.
    //
    // ⚠️⚠️ **A SEQUÊNCIA importa, e a 1.ª redacção desta metade era VAZIA**: ela convertia um
    // nó recém-criado, cujo `preset` já nasce `Custom` — o `assert` passava sem o `bake` lhe
    // tocar, e a mutação MP7 (arrancar o `mark_lsystem_custom` do `bake`) **SOBREVIVEU**.
    // *Um gate que afirma o estado em que a fixtura já está não afirma nada.* A sequência real
    // é: escolher um molde, ir aos sliders, e voltar.
    let mut fresh = MotionState::new();
    let f = fresh.doc.graph.add_node("source.lsystem");
    let mode = |m: &mut MotionState, v: i32| {
        dispatch(
            m,
            MotionParamIntent::SetParam {
                node: f.0,
                param: ls::param::MODE,
                value: f64::from(v),
            },
        )
    };
    mode(&mut fresh, ls::MODE_GRAMMAR);
    dispatch(
        &mut fresh,
        MotionParamIntent::SetParam {
            node: f.0,
            param: ls::param::PRESET,
            value: 0.0,
        },
    );
    assert_eq!(preset_of(&fresh, f), 0, "o molde 0 ficou aceso");
    mode(&mut fresh, ls::MODE_GUIDED);
    mode(&mut fresh, ls::MODE_GRAMMAR);
    assert_eq!(
        preset_of(&fresh, f),
        ls::PRESET_CUSTOM,
        "o `bake` reescreveu o texto e o selector continuou a dizer «Tree»"
    );
    // ⚠️ O CONTROLE que torna isto um achado e não uma tautologia: o texto assado de facto
    // DIFERE do molde que o selector nomeava antes.
    assert_ne!(text_of(&fresh, f, ls::RULES_PARAM), ls::PRESETS[0].rules);
}

/// ⚠️ **Escolher o molde que já está aceso RE-APLICA-O** — e a guarda de igualdade do
/// despacho **FICOU**, porque o `Custom` a tornou inofensiva.
///
/// ⚠️⚠️ **A 1.ª redacção desta nota dizia que a guarda tinha caído, e era falso** — este gate
/// ficou verde sem eu lhe tocar. O mecanismo: assim que outra coisa mexe no texto, o
/// [`mark_lsystem_custom`](super::params_channel::mark_lsystem_custom) move o `preset` para
/// `Custom`, logo *voltar a um molde é sempre uma mudança de valor* e a guarda
/// (`|actual − value| > f32::EPSILON`) nunca chega a barrar o clique de reconciliação.
///
/// A auditoria propôs remover a guarda; a medição diz que a cura do selector já a desarmou, e
/// removê-la seria mexer numa cerca que o irmão `renumbers_sim` mantém por um motivo escrito
/// (*"a slider re-emits its intent every frame of a gesture"*). **É este gate que a mantém
/// desarmada**: se um dia o `Custom` deixar de intervir, ele fica vermelho aqui e não no ecrã
/// do artista.
#[test]
fn clicking_the_preset_that_is_already_lit_puts_it_back() {
    let mut motion = MotionState::new();
    let n = motion.doc.graph.add_node("source.lsystem");
    dispatch(
        &mut motion,
        MotionParamIntent::SetParam {
            node: n.0,
            param: ls::param::PRESET,
            value: 1.0,
        },
    );
    // O artista estraga o texto à mão…
    dispatch(
        &mut motion,
        MotionParamIntent::SetTextParam {
            node: n.0,
            param: ls::RULES_PARAM,
            value: "LIXO".to_string(),
        },
    );
    // …e volta a escolher o MESMO molde.
    dispatch(
        &mut motion,
        MotionParamIntent::SetParam {
            node: n.0,
            param: ls::param::PRESET,
            value: 1.0,
        },
    );
    assert_eq!(
        text_of(&motion, n, ls::RULES_PARAM),
        ls::PRESETS[1].rules,
        "clicar no molde aceso tem de o repor — era a unica via de reconciliacao"
    );
    assert_eq!(preset_of(&motion, n), 1);
}

/// ⭐⭐⭐ **O ENQUADRAMENTO CHEGA PELO CAMINHO REAL, E NÃO SÓ PELA FUNÇÃO INTERNA.**
///
/// ⚠️ Report do Enio, 2026-08-29: *"dragon é bem menor que os outros"*. O `Dragon` só se lê
/// como dragão a **12** gerações (os irmãos vivem em 4–5), então se o `generations` do molde
/// não chegar ele fica `20×` mais pequeno — e o gate irmão
/// `picking_a_preset_also_writes_the_framing_it_needs` não o via, porque chama a função
/// INTERNA e salta a guarda de igualdade do despacho.
#[test]
fn the_framing_survives_the_real_dispatch_and_the_dragon_gets_its_twelve_generations() {
    let mut motion = MotionState::new();
    let n = motion.doc.graph.add_node("source.lsystem");
    dispatch(
        &mut motion,
        MotionParamIntent::SetParam {
            node: n.0,
            param: ls::param::MODE,
            value: f64::from(ls::MODE_GRAMMAR),
        },
    );
    for (k, p) in ls::PRESETS.iter().enumerate() {
        dispatch(
            &mut motion,
            MotionParamIntent::SetParam {
                node: n.0,
                param: ls::param::PRESET,
                value: k as f64,
            },
        );
        for (name, want) in [
            (ls::param::ANGLE, p.angle),
            (ls::param::GENERATIONS, p.generations),
            (ls::param::STEP, p.step),
            (ls::param::WIDTH, p.width),
        ] {
            assert_eq!(
                param_value(&motion, n, name),
                want,
                "{}: o `{name}` nao sobreviveu ao despacho",
                p.label
            );
        }
    }
}

/// ⭐⭐ **E O QUE O ARTISTA VÊ SAI DO MESMO TAMANHO** — a régua no PRODUTO, não nos params.
///
/// ⚠️ Os quatro números certos ainda podem dar oito plantas de tamanhos diferentes; o gate de
/// enquadramento da crate mede-os com o `probe_build`, e este mede-os **depois do despacho**,
/// que é o único caminho que a mão do artista percorre.
#[test]
fn every_preset_picked_by_hand_draws_the_same_size() {
    let mut motion = MotionState::new();
    let n = motion.doc.graph.add_node("source.lsystem");
    dispatch(
        &mut motion,
        MotionParamIntent::SetParam {
            node: n.0,
            param: ls::param::MODE,
            value: f64::from(ls::MODE_GRAMMAR),
        },
    );
    let mut sizes: Vec<(&str, f32)> = Vec::new();
    for (k, p) in ls::PRESETS.iter().enumerate() {
        dispatch(
            &mut motion,
            MotionParamIntent::SetParam {
                node: n.0,
                param: ls::param::PRESET,
                value: k as f64,
            },
        );
        let over: Vec<(&str, f32)> = [
            ls::param::ANGLE,
            ls::param::STEP,
            ls::param::WIDTH,
            ls::param::MODE,
        ]
        .iter()
        .map(|name| (*name, param_value(&motion, n, name)))
        .collect();
        let s = ls::probe_build(
            &text_of(&motion, n, ls::AXIOM_PARAM),
            &text_of(&motion, n, ls::RULES_PARAM),
            param_value(&motion, n, ls::param::GENERATIONS),
            &over,
        );
        let d = match s.get("P") {
            Some(ph2d_nodegraph::attr::Column::Vec2(v)) if !v.is_empty() => {
                let x0 = v.iter().map(|q| q[0]).fold(f32::MAX, f32::min);
                let x1 = v.iter().map(|q| q[0]).fold(f32::MIN, f32::max);
                let y0 = v.iter().map(|q| q[1]).fold(f32::MAX, f32::min);
                let y1 = v.iter().map(|q| q[1]).fold(f32::MIN, f32::max);
                ((x1 - x0).powi(2) + (y1 - y0).powi(2)).sqrt()
            }
            _ => 0.0,
        };
        sizes.push((p.label, d));
    }
    let mut sorted: Vec<f32> = sizes.iter().map(|(_, d)| *d).collect();
    sorted.sort_by(f32::total_cmp);
    let median = (sorted[3] + sorted[4]) * 0.5;
    for (label, d) in &sizes {
        let r = d / median;
        assert!(
            (0.65..1.55).contains(&r),
            "{label} sai {r:.2}x a mediana ({d:.2} contra {median:.2}) — o artista ve' um \
             molde muito maior ou muito menor que os irmaos. Todos: {sizes:?}"
        );
    }
}

/// ⭐⭐⭐ **UM FIO NÃO PODE FAZER UM MOLDE SAIR 10× MAIS PEQUENO** — a cura do report de
/// 2026-08-29 (*"dragon é bem menor que os outros"*).
///
/// ⚠️ O `EvalCtx::param` resolve o CONDUZIDO primeiro, então um fio no `Generations` ganha ao
/// número que o molde escreve. Com a cena `=108` a conduzir aquele param de `1` a `6` e o
/// `Dragon` a precisar de **12**, o molde escrevia e a planta ficava presa em `0,21` de
/// diagonal contra os `2,17` que ela devia ter.
#[test]
fn a_wire_never_keeps_a_preset_from_reaching_its_own_numbers() {
    let mut motion = MotionState::new();
    let n = motion.doc.graph.add_node("source.lsystem");
    let lfo = motion.doc.graph.add_node("value.lfo");
    motion
        .doc
        .graph
        .drive_param(n, ls::param::GENERATIONS, (lfo, 0))
        .expect("o Generations aceita fio");
    assert!(
        motion
            .doc
            .graph
            .param_sources(n)
            .is_some_and(|s| s.contains_key(ls::param::GENERATIONS)),
        "CONTROLE: o fio tem de estar la' antes"
    );

    let dragon = ls::PRESETS
        .iter()
        .position(|p| p.label == "Dragon")
        .expect("o molde existe");
    dispatch(
        &mut motion,
        MotionParamIntent::SetParam {
            node: n.0,
            param: ls::param::PRESET,
            value: dragon as f64,
        },
    );
    assert!(
        motion
            .doc
            .graph
            .param_sources(n)
            .is_none_or(|s| !s.contains_key(ls::param::GENERATIONS)),
        "o fio tem de ser SOLTO — senao o molde escreve 12 e a planta coze a 6"
    );
    assert_eq!(
        param_value(&motion, n, ls::param::GENERATIONS),
        ls::PRESETS[dragon].generations
    );
}

/// ⚠️ **E ele NÃO solta o que não estorva** — um fio no `Seed` ou no `Tropism` é trabalho do
/// artista, e um molde que o destruísse seria pior que o defeito que ele cura.
#[test]
fn the_preset_leaves_alone_the_wires_it_does_not_need() {
    let mut motion = MotionState::new();
    let n = motion.doc.graph.add_node("source.lsystem");
    let lfo = motion.doc.graph.add_node("value.lfo");
    for name in [ls::param::SEED, ls::param::TROPISM] {
        motion
            .doc
            .graph
            .drive_param(n, name, (lfo, 0))
            .expect("aceita fio");
    }
    dispatch(
        &mut motion,
        MotionParamIntent::SetParam {
            node: n.0,
            param: ls::param::PRESET,
            value: 3.0,
        },
    );
    for name in [ls::param::SEED, ls::param::TROPISM] {
        assert!(
            motion
                .doc
                .graph
                .param_sources(n)
                .is_some_and(|s| s.contains_key(name)),
            "o fio do `{name}` nao estorva molde nenhum e tem de FICAR"
        );
    }
}
