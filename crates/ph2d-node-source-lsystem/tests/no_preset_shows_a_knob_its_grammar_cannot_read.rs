//! ⭐⭐⭐ **NENHUM MOLDE MOSTRA UM KNOB QUE A GRAMÁTICA DELE NÃO SABE LER** — e nenhum esconde
//! um que ela leia.
//!
//! # Por que este ficheiro existe
//!
//! Report do Enio, 2026-08-31: *"descubra para cada Preset quais os parâmetros que não são
//! usados e esconda do painel. Não quero parâmetros mortos"*.
//!
//! ⚠️⚠️ **A régua é o PRODUTO, e não uma leitura do TEXTO da gramática.** O único gate
//! por-molde deste género que existia (`the_read_gates_agree_with_what_each_grammar_contains`)
//! comparava a lista de visibilidade com [`ls::Reads::of`], que procura `!` e `"` no texto — e
//! isso responde por **dois** knobs de vinte e nove. Os três que esta jornada achou não têm
//! símbolo nenhum a denunciá-los: o `Step Scale` morre porque numa gramática **paramétrica**
//! (`A(s)`) o comprimento viaja no módulo e o `Setup::step` nunca é lido; os dois interruptores
//! do crescimento suave morrem em metades **complementares** do corpus porque o `build`
//! pergunta `grows_by_refining()` e só lê um deles em cada braço. *Um scanner de símbolos nunca
//! os veria, e o produto responde às três de uma vez.*
//!
//! # A régua, e as DUAS leituras
//!
//! Varre-se cada param pela faixa que o `ParamUiHint` dele declara e conta-se quantas saídas
//! **distintas ao bit** o nó emite ([`ls::probe_param_prints`] — a mesma porta que a bancada
//! `examples/dead_params_report.rs` usa, de propósito).
//!
//! ⚠️⚠️ **UMA leitura só fabrica uma lista de dívida.** Um param cujo sujeito é criado por
//! OUTRO param mede-se morto com os defaults: o `Tropism Direction` não tem o que virar com
//! `Tropism = 0`, o `Seed` não tem o que semear com `Leaf Spread = 0`, e os dois interruptores
//! do crescimento suave só agem em geração **fraccionária**. Medido: com uma leitura só, esta
//! bancada acusava **12** params; com as duas, **9** — e dos 3 que caíram, esconder qualquer um
//! teria apagado um controlo vivo. ⇒ cada célula é medida no enquadramento do molde **e** com o
//! contexto aceso, e só `inerte nas duas` conta.
//!
//! # As duas metades, e por que a segunda é obrigatória
//!
//! 1. **Toda lista de visibilidade bate com a medição.** Um molde que passe a ler um knob
//!    escondido, ou a deixar de ler um mostrado, fica vermelho aqui.
//! 2. **Todo par (param, molde) medido inerte tem de estar EXPLICADO.** Sem esta metade a
//!    primeira é uma catraca que só desce: acrescentar um molde cego a um knob não acusaria
//!    ninguém. E a explicação **não é um nome numa lista de isenções** — é uma afirmação que
//!    se volta a medir:
//!    - gateado por molde ⇒ a metade 1 já responde;
//!    - gateado pelo `Mode` ⇒ o gate existe e é lido do registo;
//!    - **dependente** ⇒ ele tem de ACORDAR na 2.ª leitura. No dia em que alguém o matar a
//!      sério, a isenção deixa de se verificar e este ficheiro acusa-o.
//!
//! ⛔ *Uma catraca sem censo de obsolescência não desce: ela vira licença* (CLAUDE.md §5.0).

use ph2d_node_source_lsystem as ls;

/// Os valores com que se varre um param — a faixa que o painel pinta, mais o default.
///
/// ⚠️ **Nove amostras e não duas.** Uma lei com ponto de simetria (um ângulo a `0`, um espelho
/// a meio) daria o mesmo carimbo nas duas pontas e leria-se morta.
fn samples(hint: &ph2d_node_registry::ParamUiHint, def: f32) -> Vec<f32> {
    const N: usize = 9;
    let mut v: Vec<f32> = (0..N)
        .map(|i| hint.min + (hint.max - hint.min) * (i as f32) / ((N - 1) as f32))
        .collect();
    v.push(def);
    v
}

/// O contexto ACESO: o que dá sujeito aos params que dependem de um vizinho.
///
/// ⚠️ **Não é «valores maiores», é «o vizinho ligado»** — cada entrada aqui existe porque a
/// bancada nomeou o param que acorda um dependente (`tropism` acorda o `tropism_angle`,
/// `leaf_spread` acorda o `seed`, `root_angle` acorda o `tropism` no Sprig, cuja cadeia
/// persistente é exactamente vertical e portanto anti-paralela ao puxão).
fn awake() -> Vec<(&'static str, f32)> {
    vec![
        (ls::param::TROPISM, 0.5),
        (ls::param::LEAF_SPREAD, 45.0),
        (ls::param::LEAF_ANGLE, 30.0),
        (ls::param::ROOT_ANGLE, 75.0),
        (ls::param::ORIENT, 1.0),
    ]
}

/// O enquadramento que o molde carrega — os quatro números que trocar de molde escreve.
fn frame(p: &ls::Preset) -> Vec<(&'static str, f32)> {
    vec![
        (ls::param::ANGLE, p.angle),
        (ls::param::STEP, p.step),
        (ls::param::WIDTH, p.width),
        (ls::param::LEAF_FIRST_LEVEL, p.leaf_first_level),
        // ⚠️ **`Branches` é o modo do PRODUTO** (o default do manifesto), e a porta de sonda
        // abre em `Segments`. Medir no modo velho responderia por uma planta que já ninguém vê.
        (ls::param::GEOMETRY, ls::GEOMETRY_BRANCHES as f32),
    ]
}

/// `(inerte no enquadramento, inerte também com tudo aceso)`.
fn readings(
    p: &ls::Preset,
    hint: &ph2d_node_registry::ParamUiHint,
    def: f32,
    param: &str,
) -> (bool, bool) {
    let vals = samples(hint, def);
    let plain = ls::probe_param_prints(p.axiom, p.rules, p.generations, &frame(p), param, &vals);
    let mut lit = frame(p);
    for (n, v) in awake() {
        if n == param {
            continue;
        }
        lit.retain(|(k, _)| *k != n);
        lit.push((n, v));
    }
    // ⚠️ A meia geração não é um param: é o estado em que o `Generations` está a ser ANIMADO,
    // e é o único sítio onde os dois interruptores do crescimento suave têm o que interpolar.
    let wake = ls::probe_param_prints(p.axiom, p.rules, p.generations + 0.5, &lit, param, &vals);
    (plain == 1, plain == 1 && wake == 1)
}

/// Os params que esta régua NÃO alcança: a shell é que os lê, e o `build` não os vê.
///
/// ⚠️ **A cerca deles é o `geometry`** — em `Segments` a membrana sai por `continue` e nenhum
/// deles tem consumidor, então os cinco são gateados por `GEOMETRY = Branches`.
///
/// ⛔⛔ **O `LEAF_EFFECTS` SAIU desta lista em 2026-08-31** (doc 96 §4.3): ele dá **2** saídas
/// distintas — a tartaruga escreve o `TINT_MASK_COLUMN` no próprio esqueleto —, logo o nó
/// **lê-o** e ele nunca foi «shell-side». Estar aqui tirava-o da população do instrumento mais
/// forte da crate, que é exactamente onde um knob morto se esconde.
///
/// ⚠️ **E a justificação que aqui estava — *«a sonda estouraria neles»* — era falsa em três dos
/// seis.** Ela agora é RE-MEDIDA por
/// [`every_shell_side_exemption_still_describes_a_param_the_build_cannot_read`]:
/// *uma catraca sem censo de obsolescência não desce, vira licença*.
const SHELL_SIDE: &[&str] = &[
    ls::param::TIP_TAPER,
    ls::param::LEAF_FRONT,
    ls::param::LEAF_SIZE,
    ls::param::LEAF_SIZE_JITTER,
    ls::param::LEAF_POS_JITTER,
];

/// Os que escolhem O QUE se mede — varrê-los mediria outra planta, não este param.
const STRUCTURAL: &[&str] = &[ls::param::PRESET, ls::param::MODE, ls::param::GEOMETRY];

fn registry() -> ph2d_node_registry::NodeRegistry {
    let mut reg = ph2d_node_registry::NodeRegistry::default();
    ls::register(&mut reg).expect("o nó regista");
    reg
}

#[test]
fn every_preset_gate_lists_exactly_the_grammars_that_read_that_knob() {
    let reg = registry();
    let hints = reg.param_ui(ls::MANIFEST.id).expect("hints");
    let gates = reg.param_gates(ls::MANIFEST.id).expect("gates");

    let mut checked = 0usize;
    for g in gates.iter().filter(|g| g.when == ls::param::PRESET) {
        let hint = hints
            .iter()
            .find(|h| h.param == g.param)
            .unwrap_or_else(|| panic!("`{}` é gateado por molde e não tem hint", g.param));
        let def = ls::MANIFEST
            .params
            .iter()
            .find(|p| p.name == g.param)
            .map_or(0.0, |p| p.default);

        // ⚠️ O `Custom` **não é um molde**: é *"nenhum destes"*, e a gramática que lá vive é a
        // que o artista escreveu. O painel não pode saber o que ela lê ⇒ nunca se esconde nada
        // nele. É a mesma decisão que o `Width Scale` já tinha, e agora é uma LEI com gate.
        assert!(
            g.values.contains(&(ls::PRESET_CUSTOM as i32)),
            "`{}` esconde-se no Custom — e no Custom a gramática é a que o artista escreveu, \
             então esconder ali é adivinhar",
            g.param
        );

        for (i, p) in ls::PRESETS.iter().enumerate() {
            let (_, dead) = readings(p, hint, def, g.param);
            let shown = g.values.contains(&(i as i32));
            assert_eq!(
                shown,
                !dead,
                "`{}` no molde `{}`: o painel {} e a medição diz que ele {}",
                g.param,
                p.label,
                if shown { "MOSTRA" } else { "esconde" },
                if dead { "não muda um bit" } else { "AGE" },
            );
            checked += 1;
        }
    }
    // ⚠️ Um filtro que casasse ZERO gates deixaria este teste verde a não medir nada — o
    // controlo do próprio filtro (memória: *«um filtro que casa ZERO imprime SOBREVIVEU»*).
    assert!(
        checked >= ls::PRESETS.len() * 5,
        "só {checked} células medidas — o filtro de gates por molde apanhou pouco ou nada"
    );
}

#[test]
fn every_inert_knob_is_explained_and_no_explanation_has_gone_stale() {
    let reg = registry();
    let hints = reg.param_ui(ls::MANIFEST.id).expect("hints");
    let gates = reg.param_gates(ls::MANIFEST.id).expect("gates");

    // ⚠️ **«tem gate por molde» e «está escondido NESTE molde» são perguntas diferentes**, e a
    // 1.ª redacção deste ficheiro colapsou-as: o `Grow Length` tem gate por molde **e é
    // mostrado no Tree**, onde ele dorme só por a geração ser inteira. Perguntar a primeira
    // acusava-o de esconder um dependente que ele mostra. *Um predicado sobre o param não
    // responde por uma célula.*
    let hidden_here = |name: &str, i: usize| {
        gates
            .iter()
            .filter(|g| g.param == name && g.when == ls::param::PRESET)
            .any(|g| !g.values.contains(&(i as i32)))
    };
    let by_mode = |name: &str| {
        gates
            .iter()
            .any(|g| g.param == name && g.when == ls::param::MODE)
    };
    let above = reg.param_gates_above(ls::MANIFEST.id).unwrap_or(&[]);
    let by_threshold = |name: &str| above.iter().any(|g| g.param == name);

    let mut orfaos: Vec<String> = Vec::new();
    for spec in ls::MANIFEST.params {
        let name = spec.name;
        if SHELL_SIDE.contains(&name) || STRUCTURAL.contains(&name) {
            continue;
        }
        let Some(hint) = hints.iter().find(|h| h.param == name) else {
            continue;
        };
        for (i, p) in ls::PRESETS.iter().enumerate() {
            let (sleeps, dead) = readings(p, hint, spec.default, name);
            if !sleeps {
                continue;
            }
            if hidden_here(name, i) {
                // ⚠️ **A metade que protege o artista:** só se esconde o que está morto nas
                // DUAS leituras. Esconder um que acorda com o vizinho aceso seria apagar um
                // controlo vivo — e é a acusação que uma leitura só teria feito a três deles.
                assert!(
                    dead,
                    "`{name}` é escondido no molde `{}` e ACORDA com o contexto aceso — \
                     esconder um dependente apaga um controlo vivo",
                    p.label
                );
                continue;
            }
            // Um dependente ACORDA na 2.ª leitura. Enquanto acordar, a isenção descreve algo.
            if !dead || by_mode(name) || by_threshold(name) {
                continue;
            }
            orfaos.push(format!("{name} @ {}", p.label));
        }
    }
    assert!(
        orfaos.is_empty(),
        "estes knobs não mudam um bit no molde em que são pintados, e nada os esconde: {orfaos:#?}"
    );
}

/// ⭐⭐⭐ **O KNOB QUE ESTÁ MORTO NO INSTANTE EM QUE O PAINEL ABRE** — a espécie mais cara.
///
/// A metade acima isenta o **dependente** (dorme com os defaults, acorda com o vizinho) porque
/// escondê-lo por molde apagaria um controlo vivo. ⚠️ Mas essa isenção, sozinha, é uma licença:
/// o `Tropism Direction` dormia nos **nove** moldes, o `Tropism` nasce em `0`, e o artista que
/// abre a secção *Lean & Look* mexe primeiro naquele — *o dependente que dorme em TODO o corpus
/// não é um caso de borda, é um knob morto de fábrica.*
///
/// # A régua, e por que ela separa exactamente os três
///
/// A pergunta não é *«ele depende de alguém?»* mas *«existe algum molde onde ele já age, sem o
/// artista ligar nada?»*. Medido:
///
/// | param | dorme em | veredito |
/// |---|---|---|
/// | `tropism_angle` | **9 de 9** | morto de fábrica ⇒ **tem de ser gateado** (hoje: limiar sobre o `Tropism`) |
/// | `seed` | 8 de 9 | vivo no `Wild`, cuja gramática é estocástica ⇒ isento |
/// | `tropism` | 1 de 9 | vivo em 8 ⇒ isento (no `Sprig` a cadeia persistente é exactamente vertical, logo anti-paralela ao puxão) |
///
/// ⛔ **E é por isso que a cura do `seed` NÃO é um limiar sobre o `Leaf Spread`**, que é o único
/// despertador que esta régua vê: ele é também semeado pelo `Leaf Size Jitter` e pelo
/// `Leaf Pos Jitter`, que a SHELL lê e o `build` não — um limiar sobre um dos três apagaria o
/// knob para quem usasse os outros dois. *Uma isenção medida é mais barata que um gate errado.*
#[test]
fn no_knob_is_dead_across_the_whole_corpus_unless_something_hides_it() {
    let reg = registry();
    let hints = reg.param_ui(ls::MANIFEST.id).expect("hints");
    let gates = reg.param_gates(ls::MANIFEST.id).expect("gates");
    let above = reg.param_gates_above(ls::MANIFEST.id).unwrap_or(&[]);

    let mut mortos_de_fabrica: Vec<String> = Vec::new();
    let mut medidos = 0usize;
    for spec in ls::MANIFEST.params {
        let name = spec.name;
        if SHELL_SIDE.contains(&name) || STRUCTURAL.contains(&name) {
            continue;
        }
        let Some(hint) = hints.iter().find(|h| h.param == name) else {
            continue;
        };
        medidos += 1;
        let dorme_em = ls::PRESETS
            .iter()
            .filter(|p| readings(p, hint, spec.default, name).0)
            .count();
        if dorme_em < ls::PRESETS.len() {
            continue; // Há um molde onde ele já age de fábrica.
        }
        let escondido = gates
            .iter()
            .any(|g| g.param == name && !g.values.is_empty())
            || above.iter().any(|g| g.param == name);
        if !escondido {
            mortos_de_fabrica.push(format!(
                "{name} (dorme em {dorme_em}/{})",
                ls::PRESETS.len()
            ));
        }
    }
    // ⚠️ O controlo do próprio filtro: se a lista de params encolhesse a zero, este teste ficava
    // verde a não medir nada.
    assert!(
        medidos >= 15,
        "só {medidos} params medidos — o filtro apanhou pouco ou nada"
    );
    assert!(
        mortos_de_fabrica.is_empty(),
        "estes knobs não mudam um bit em molde NENHUM com os defaults de fábrica, e o painel \
         pinta-os na mesma: {mortos_de_fabrica:#?}"
    );
}

/// ⭐⭐⭐ **O MODO GUIADO TAMBÉM É UMA GRAMÁTICA, E O PAINEL TEM DE A LER** — o buraco que a
/// auditoria de seis lentes achou nesta própria bancada (doc 96 §1.2).
///
/// # Por que os dois gates de cima não podiam ver isto
///
/// Eles percorrem `ls::PRESETS` (oito moldes) e isentam o `PRESET_CUSTOM` em bloco, com o
/// argumento *«no Custom a gramática é a que o artista escreveu, então esconder ali é
/// adivinhar»*.
///
/// ⚠️⚠️ **O argumento é verdade para uma gramática ESCRITA À MÃO e falso para a GUIADA.** No
/// modo guiado o app **deriva** a gramática de quatro sliders — ele sabe exactamente qual é. E
/// como `Custom` é o preset de fábrica e `Guided` o modo de fábrica, o que a isenção deixava
/// passar não era uma esquina: era **o primeiro ecrã de um nó recém-largado**.
///
/// ⇒ o `Custom` deixa de ser uma isenção em bloco. Ele é *«escrita à mão»* (isento, porque
/// ninguém a pode ler) **ou** *«derivada dos sliders»* (medível, e medida aqui).
///
/// # A régua: TODO o espaço que os sliders alcançam, não os defaults
///
/// Um knob escondido num modo tem de estar morto em **todo** ele — bastava uma combinação de
/// sliders que refinasse para o `Grow Angle` ter sujeito, e escondê-lo apagaria um controlo
/// vivo. Medido em `branches × segments × variation × bend`, geração inteira **e** fraccionária.
#[test]
fn the_guided_grammar_hides_exactly_the_knobs_it_cannot_read() {
    let reg = registry();
    let hints = reg.param_ui(ls::MANIFEST.id).expect("hints");
    let gates = reg.param_gates(ls::MANIFEST.id).expect("gates");
    let default_of = |n: &str| {
        ls::MANIFEST
            .params
            .iter()
            .find(|p| p.name == n)
            .map_or(0.0, |p| p.default)
    };

    // O que o painel MOSTRA no estado guiado de fábrica: `mode = Guided`, `preset = Custom`.
    let visivel = |name: &str| {
        gates.iter().all(|g| {
            g.param != name
                || if g.when == ls::param::MODE {
                    g.values.contains(&ls::MODE_GUIDED)
                } else if g.when == ls::param::PRESET {
                    g.values.contains(&(ls::PRESET_CUSTOM as i32))
                } else {
                    // Os outros sujeitos (geometry…) ficam no default do manifesto.
                    g.values.contains(&(default_of(g.when).round() as i32))
                }
        })
    };

    let mut acusados: Vec<String> = Vec::new();
    let mut celulas = 0usize;
    // ⚠️ **Os CANTOS e o meio, não a grelha cheia.** A grelha 3×3×3×3 mede o mesmo e custa
    // `34 s`; um knob que só age numa combinação interior teria de agir sem agir em nenhum
    // extremo, e a lei que o decide (paramétrica? refina?) é estrutural, não contínua.
    for b in [1.0f32, 3.0, 5.0] {
        for sg in [1.0f32, 6.0] {
            for var in [0.0f32, 1.0] {
                for bend in [-30.0f32, 30.0] {
                    let sh = ls::shape::Shape {
                        branches: b,
                        segments: sg,
                        variation: var,
                        bend,
                    };
                    let rules = ls::shape::rules(&sh);
                    celulas += 1;
                    for spec in ls::MANIFEST.params {
                        let name = spec.name;
                        if SHELL_SIDE.contains(&name) || STRUCTURAL.contains(&name) {
                            continue;
                        }
                        let Some(hint) = hints.iter().find(|h| h.param == name) else {
                            continue;
                        };
                        let vals = samples(hint, spec.default);
                        let base = [(ls::param::GEOMETRY, ls::GEOMETRY_BRANCHES as f32)];
                        let vivo = [5.0f32, 5.5].iter().any(|g| {
                            ls::probe_param_prints(
                                ls::DEFAULT_AXIOM,
                                &rules,
                                *g,
                                &base,
                                name,
                                &vals,
                            ) > 1
                        });
                        if vivo && !visivel(name) {
                            acusados.push(format!(
                                "`{name}` AGE e está escondido (b={b} s={sg} v={var} bend={bend})"
                            ));
                        }
                        if !vivo
                            && visivel(name)
                            && (name == ls::param::STEP_SCALE
                                || name == ls::param::CONTINUOUS_ANGLE)
                        {
                            acusados.push(format!("`{name}` é pintado e MORTO"));
                        }
                    }
                }
            }
        }
    }
    assert!(
        celulas >= 24,
        "só {celulas} células do espaço guiado — a varredura apanhou pouco"
    );
    acusados.sort();
    acusados.dedup();
    assert!(
        acusados.is_empty(),
        "no modo GUIADO o painel discorda da gramática que o próprio app derivou: {acusados:#?}"
    );
}

/// ⛔⛔⛔ **A ISENÇÃO `SHELL_SIDE` PRESTA CONTAS** — as três afirmações dela, re-medidas.
///
/// # Por que este gate existe
///
/// Auditoria de seis lentes, 2026-08-31 (doc 96 §4.3). A lista de saltos deste ficheiro
/// justifica-se com três frases, e **duas estavam erradas para parte da população que isentam**:
///
/// 1. *«o `build` não os vê»* — o `leaf_effects` dá **2** saídas distintas: a tartaruga escreve
///    o `TINT_MASK_COLUMN`, logo ele **é** lido pelo nó e não é «shell-side» de todo;
/// 2. *«[`ls::probe_param_prints`] estouraria neles»* — **três não estouram**;
/// 3. *«a cerca deles é o `geometry`, que já está gateado»* — verdade só para o `tip_taper`.
///
/// ⚠️⚠️ *Uma catraca sem censo de obsolescência não desce: ela vira licença* (`CLAUDE.md` §5.0).
/// Uma lista de isenções cuja justificação nunca se volta a medir é o sítio onde um knob morto
/// se esconde do instrumento mais forte da crate — e foi exactamente o que aconteceu.
///
/// ⇒ cada nome aqui tem de continuar a merecer o salto, e a prova é executada.
#[test]
fn every_shell_side_exemption_still_describes_a_param_the_build_cannot_read() {
    let reg = registry();
    let hints = reg.param_ui(ls::MANIFEST.id).expect("hints");
    let gates = reg.param_gates(ls::MANIFEST.id).expect("gates");
    let p = &ls::PRESETS[0];

    let mut queixas: Vec<String> = Vec::new();
    for name in SHELL_SIDE {
        // (a) O `build` de facto não o lê? — `probe_param_prints` estoura nos que a porta de
        // sonda não conhece, e isso conta como «não alcançável», que é a mesma resposta.
        let hint = hints
            .iter()
            .find(|h| h.param == *name)
            .unwrap_or_else(|| panic!("`{name}` está isento e nem sequer é pintado"));
        let def = ls::MANIFEST
            .params
            .iter()
            .find(|s| s.name == *name)
            .map_or(0.0, |s| s.default);
        let vals = samples(hint, def);
        let lido = std::panic::catch_unwind(|| {
            ls::probe_param_prints(p.axiom, p.rules, p.generations, &frame(p), name, &vals)
        });
        if let Ok(n) = lido
            && n > 1
        {
            queixas.push(format!(
                "`{name}` está em SHELL_SIDE e o `build` LÊ-O ({n} saídas distintas) — ele \
                 pertence à população medida, não à lista de saltos"
            ));
        }
        // (b) A cerca declarada existe? Um param que a régua não alcança tem de ser fechado por
        // OUTRA coisa, senão ele é simplesmente um knob sem nenhum instrumento em cima.
        let cercado = gates
            .iter()
            .any(|g| g.param == *name && g.when == ls::param::GEOMETRY);
        if !cercado {
            queixas.push(format!(
                "`{name}` está isento com a nota «a cerca deles é o `geometry`» e NÃO tem gate \
                 de `geometry` — em `Segments` a membrana sai por `continue` e ele é pintado \
                 sobre nada"
            ));
        }
    }
    // ⚠️ O controlo do próprio censo: uma lista vazia passaria calada.
    assert!(
        SHELL_SIDE.len() >= 4,
        "a lista de saltos encolheu para {} — re-confira se ela ainda descreve alguma coisa",
        SHELL_SIDE.len()
    );
    queixas.sort();
    assert!(
        queixas.is_empty(),
        "a lista `SHELL_SIDE` deixou de descrever o produto: {queixas:#?}"
    );
}

/// ⛔⛔ **A CERCA DA GEOMETRIA NÃO ESCONDE NENHUM KNOB QUE O ESQUELETO LEIA** — a metade
/// simétrica, e sem ela a outra é meia lei.
///
/// # Por que ela é obrigatória
///
/// O censo irmão exige que todo param de [`SHELL_SIDE`] TENHA cerca de `geometry`. Sozinho, ele
/// aprova pôr a cerca em qualquer param — inclusive num que o nó lê. Medido: a mutação que muda
/// a cerca do `Leaf Pos Jitter` para o `Leaf Effects` mata o censo irmão, mas **pelo motivo
/// errado** (o `Pos Jitter` ficou sem cerca), e o `Leaf Effects` — que a tartaruga escreve no
/// esqueleto — passaria a estar escondido em `Segments` sem ninguém reparar.
///
/// ⚠️⚠️ **Quatro dos oito knobs da secção *Leaves* estão VIVOS em `Segments`** — o
/// `leaf_effects` (escreve o `TINT_MASK_COLUMN`), o `leaf_first_level`, o `leaf_angle` e o
/// `leaf_spread` (o `mark_grow` e o `rot`). *Esconder a secção inteira apagaria quatro controlos
/// vivos para curar quatro mortos*, e é essa a recusa que este gate torna executável.
#[test]
fn the_geometry_fence_never_hides_a_knob_the_skeleton_reads() {
    let reg = registry();
    let hints = reg.param_ui(ls::MANIFEST.id).expect("hints");
    let gates = reg.param_gates(ls::MANIFEST.id).expect("gates");
    let p = &ls::PRESETS[0];

    // O enquadramento do molde, mas em `Segments` — o modo em que a cerca esconde.
    let mut seg: Vec<(&'static str, f32)> = frame(p);
    seg.retain(|(k, _)| *k != ls::param::GEOMETRY);
    seg.push((ls::param::GEOMETRY, ls::GEOMETRY_SEGMENTS as f32));

    let mut queixas: Vec<String> = Vec::new();
    let mut medidos = 0usize;
    for spec in ls::MANIFEST.params {
        let name = spec.name;
        if STRUCTURAL.contains(&name) {
            continue;
        }
        let escondido = gates
            .iter()
            .any(|g| g.param == name && g.when == ls::param::GEOMETRY);
        if !escondido {
            continue;
        }
        let Some(hint) = hints.iter().find(|h| h.param == name) else {
            continue;
        };
        medidos += 1;
        let vals = samples(hint, spec.default);
        // ⚠️ `catch_unwind`: os que a porta de sonda não conhece não são alcançáveis pelo nó, o
        // que é precisamente a resposta que os justifica.
        let lido = std::panic::catch_unwind(|| {
            ls::probe_param_prints(p.axiom, p.rules, p.generations, &seg, name, &vals)
        });
        if let Ok(n) = lido
            && n > 1
        {
            queixas.push(format!(
                "`{name}` é escondido pela cerca da GEOMETRIA e o esqueleto LÊ-O em `Segments` \
                 ({n} saídas distintas) — a cerca está a apagar um controlo vivo"
            ));
        }
    }
    assert!(
        medidos >= 5,
        "só {medidos} params gateados por `geometry` — a cerca encolheu ou o filtro partiu"
    );
    queixas.sort();
    assert!(queixas.is_empty(), "{queixas:#?}");
}
