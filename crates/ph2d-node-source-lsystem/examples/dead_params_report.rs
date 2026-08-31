//! **A BANCADA DOS KNOBS MORTOS** — que param, em que molde, não muda um bit.
//!
//! Report do Enio (2026-08-31): *"descubra para cada Preset quais os parâmetros que não são
//! usados e esconda do painel. Não quero parâmetros mortos"*.
//!
//! ⚠️ **A régua é o PRODUTO, e a pergunta é binária:** varre-se o param pela faixa que o
//! `ParamUiHint` dele declara e compara-se a IMPRESSÃO DIGITAL do stream que o nó emite. Se as
//! amostras todas derem o mesmo carimbo, aquele knob não tem sujeito naquele molde.
//!
//! ⚠️⚠️ **A faixa sai do HINT, nunca de números escritos aqui** — uma segunda tabela de faixas
//! ao lado da que o painel pinta daria duas respostas à mesma pergunta, e a que o artista vê
//! seria a que envelhecia. O `min`/`max`/`step` que este ficheiro varre são os mesmos que o
//! slider arrasta.
//!
//! ⚠️⚠️ **E ela mede METADE do produto, de propósito — a metade do NÓ.** O que o `build` emite
//! é o esqueleto; a fita, a ponta afinada e a aparência da folha nascem na shell, a partir dele.
//! Um param que esta bancada diz inerte é inerte **no esqueleto**, e a coluna `via` diz por onde
//! ele ainda pode agir. *Uma régua que mede um lado e fala pelos dois é a que inventa a lista de
//! dívida* — por isso a saída separa as duas populações em vez de as somar.
//!
//!   cargo run -p ph2d-node-source-lsystem --example dead_params_report --release

use ph2d_node_source_lsystem as ls;
use ph2d_nodegraph::attr::Column;

/// O default que o MANIFESTO declara — a única fonte.
fn default_of(name: &str) -> f32 {
    ls::MANIFEST
        .params
        .iter()
        .find(|p| p.name == name)
        .map_or(0.0, |p| p.default)
}

/// **A IMPRESSÃO DIGITAL de um stream** — todas as colunas, pelos BITS.
///
/// ⚠️ Os bits e não o decimal: duas larguras que se imprimem iguais podem diferir, e um param
/// que só mexesse na 8.ª casa seria declarado morto por um `to_string`.
///
/// ⚠️ **Ordenada pelo NOME da coluna** — a ordem de iteração de um mapa não é a pergunta, e
/// deixá-la entrar faria a impressão mudar sem o produto mudar.
fn fingerprint(s: &ph2d_nodegraph::attr::Stream) -> u64 {
    let mut cols: Vec<(&String, &Column)> = s.columns().collect();
    cols.sort_by(|a, b| a.0.cmp(b.0));
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    let mut eat = |x: u32| {
        h ^= u64::from(x);
        h = h.wrapping_mul(0x0100_0000_01b3);
    };
    eat(s.count() as u32);
    for (name, col) in cols {
        for b in name.as_bytes() {
            eat(u32::from(*b));
        }
        match col {
            Column::Scalar(v) => v.iter().for_each(|x| eat(x.to_bits())),
            Column::Vec2(v) => v.iter().flatten().for_each(|x| eat(x.to_bits())),
            Column::Vec3(v) => v.iter().flatten().for_each(|x| eat(x.to_bits())),
            Column::Vec4(v) => v.iter().flatten().for_each(|x| eat(x.to_bits())),
        }
    }
    h
}

/// Um molde a medir: o nome, a gramática, e o enquadramento que ele carrega.
struct Case {
    label: &'static str,
    axiom: &'static str,
    rules: &'static str,
    frame: Vec<(&'static str, f32)>,
    generations: f32,
}

fn cases() -> Vec<Case> {
    let mut v: Vec<Case> = ls::PRESETS
        .iter()
        .map(|p| Case {
            label: p.label,
            axiom: p.axiom,
            rules: p.rules,
            // ⚠️ **O enquadramento do molde, e não os defaults do painel** — trocar de molde
            // escreve estes cinco (ver `Preset`), então medir com os defaults mediria uma
            // planta que o artista nunca vê.
            frame: vec![
                (ls::param::ANGLE, p.angle),
                (ls::param::STEP, p.step),
                (ls::param::WIDTH, p.width),
                (ls::param::LEAF_FIRST_LEVEL, p.leaf_first_level),
            ],
            generations: p.generations,
        })
        .collect();
    // ⚠️ O `Custom` **não é um molde**: é *"nenhum destes"*, e a gramática que lá vive é a de
    // fábrica. Ele entra porque é o único índice onde um param escondido em todos os moldes
    // ainda tem de aparecer — quem escreve `"` à mão precisa do `Length Scale`.
    v.push(Case {
        label: "Custom",
        axiom: ls::DEFAULT_AXIOM,
        rules: ls::DEFAULT_RULES,
        frame: vec![
            (ls::param::ANGLE, default_of(ls::param::ANGLE)),
            (ls::param::STEP, default_of(ls::param::STEP)),
            (ls::param::WIDTH, default_of(ls::param::WIDTH)),
            (
                ls::param::LEAF_FIRST_LEVEL,
                default_of(ls::param::LEAF_FIRST_LEVEL),
            ),
        ],
        generations: default_of(ls::param::GENERATIONS),
    });
    v
}

/// **As amostras de um param** — da faixa que o hint declara, mais o default.
///
/// ⚠️ **Nove amostras e não duas.** Uma lei com um ponto de simetria (um ângulo a `0`, um
/// espelho a `0,5`) daria o mesmo carimbo nas duas pontas e leria-se morta.
fn samples(hint: &ph2d_node_registry::ParamUiHint, def: f32) -> Vec<f32> {
    const N: usize = 9;
    let mut v: Vec<f32> = (0..N)
        .map(|i| hint.min + (hint.max - hint.min) * (i as f32) / ((N - 1) as f32))
        .collect();
    v.push(def);
    v
}

fn main() {
    let mut reg = ph2d_node_registry::NodeRegistry::default();
    ls::register(&mut reg).expect("o nó regista");
    let hints = reg
        .param_ui(ls::MANIFEST.id)
        .expect("o nó regista hints de param");
    let gates = reg.param_gates(ls::MANIFEST.id).unwrap_or(&[]);

    // ⚠️ **Os que a shell lê**, e que por isso NÃO se medem aqui. A lista sai da diferença
    // entre o manifesto e o que a porta de sonda do nó aceita — ver a nota do módulo.
    let shell_side = [
        ls::param::TIP_TAPER,
        ls::param::LEAF_FRONT,
        ls::param::LEAF_EFFECTS,
        ls::param::LEAF_SIZE,
        ls::param::LEAF_SIZE_JITTER,
        ls::param::LEAF_POS_JITTER,
    ];
    // Os estruturais: eles escolhem O QUE se mede, então varrê-los mediria outra planta.
    let structural = [ls::param::PRESET, ls::param::MODE, ls::param::GEOMETRY];

    // ⚠️⚠️ **O CONTEXTO ACORDADO — sem ele esta bancada INVENTA uma lista de dívida.**
    //
    // Um param cujo sujeito é criado por OUTRO param mede-se morto quando o outro está no
    // default: o `tropism_angle` não tem o que virar com `tropism = 0`, e os dois interruptores
    // do crescimento suave só agem em gerações FRACCIONÁRIAS. Medir só com os defaults acusaria
    // os três, e escondê-los seria apagar controlos vivos.
    //
    // ⇒ **Cada param é medido DUAS vezes**: no enquadramento do molde e com tudo aceso. Inerte
    // nos dois ⇒ a gramática não o lê (cura: gate por molde). Inerte só no primeiro ⇒ ele
    // DEPENDE de outro (cura: gate de dependência, e ⛔ nunca um gate por molde).
    let awake: Vec<(&'static str, f32)> = vec![
        (ls::param::TROPISM, 0.5),
        (ls::param::LEAF_SPREAD, 45.0),
        (ls::param::LEAF_ANGLE, 30.0),
        (ls::param::ROOT_ANGLE, 75.0),
        (ls::param::ORIENT, 1.0),
    ];

    let cs = cases();
    println!("# Knobs mortos por molde — o que o ESQUELETO não vê\n");
    println!(
        "> Duas leituras por célula: `enquadramento / acordado`. `✗` = inerte nas duas.\n\
         > `dep` = inerte só no enquadramento ⇒ **depende de outro param**, não do molde.\n\
         > ⚠️ As gerações FRACCIONÁRIAS entram na 2.ª leitura (`g + 0,5`) — é lá que os dois\n\
         > interruptores do crescimento suave têm sujeito.\n"
    );
    print!("| param |");
    for c in &cs {
        print!(" {} |", c.label);
    }
    println!("\n|---|{}", "---|".repeat(cs.len()));

    let mut dead: Vec<(&str, Vec<&str>)> = Vec::new();
    let mut depends: Vec<(&str, Vec<&str>)> = Vec::new();
    for spec in ls::MANIFEST.params {
        let name = spec.name;
        if structural.contains(&name) {
            continue;
        }
        if shell_side.contains(&name) {
            println!("| `{name}` |{}", " shell |".repeat(cs.len()));
            continue;
        }
        let Some(hint) = hints.iter().find(|h| h.param == name) else {
            println!("| `{name}` |{}", " SEM HINT |".repeat(cs.len()));
            continue;
        };
        print!("| `{name}` |");
        let mut mortos: Vec<&str> = Vec::new();
        let mut deps: Vec<&str> = Vec::new();
        for c in &cs {
            let count = |wake: bool| {
                let mut prints = std::collections::BTreeSet::new();
                for v in samples(hint, spec.default) {
                    let mut ov = c.frame.clone();
                    // ⚠️ A meia geração é o que dá SUJEITO aos dois interruptores do
                    // crescimento suave: em geração inteira eles não têm o que interpolar.
                    let mut gens = if wake {
                        c.generations + 0.5
                    } else {
                        c.generations
                    };
                    if wake {
                        for (n, w) in &awake {
                            ov.retain(|(k, _)| k != n);
                            ov.push((n, *w));
                        }
                    }
                    if name == ls::param::GENERATIONS {
                        gens = v;
                    } else {
                        ov.retain(|(n, _)| *n != name);
                        ov.push((name, v));
                    }
                    // ⚠️ **`Branches` é o modo do PRODUTO** (o default do manifesto), e a porta
                    // de sonda abre em `Segments`. Medir no modo velho responderia por uma
                    // planta que o artista já não vê.
                    ov.push((ls::param::GEOMETRY, ls::GEOMETRY_BRANCHES as f32));
                    prints.insert(fingerprint(&ls::probe_build(c.axiom, c.rules, gens, &ov)));
                }
                prints.len()
            };
            let (plain, wake) = (count(false), count(true));
            if plain == 1 && wake == 1 {
                print!(" ✗ |");
                mortos.push(c.label);
            } else if plain == 1 {
                print!(" dep |");
                deps.push(c.label);
            } else {
                print!(" {plain}/{wake} |");
            }
        }
        println!();
        if !mortos.is_empty() {
            dead.push((name, mortos));
        }
        if !deps.is_empty() {
            depends.push((name, deps));
        }
    }

    // ⭐ **O número que o artista vê** — quantos controlos o painel pinta em cada molde, contra
    // os que ele pintaria sem gate nenhum. É a única coluna desta bancada que não é um
    // diagnóstico: é o produto.
    println!("\n## Quantos controlos o painel pinta, por molde\n");
    println!("| molde | sem gates | hoje | escondidos |");
    println!("|---|---|---|---|");
    let total = ls::MANIFEST
        .params
        .iter()
        .filter(|p| !structural.contains(&p.name))
        .count();
    for (i, c) in cs.iter().enumerate() {
        let escondidos = ls::MANIFEST
            .params
            .iter()
            .filter(|p| !structural.contains(&p.name))
            .filter(|p| {
                gates
                    .iter()
                    .filter(|g| g.param == p.name && g.when == ls::param::PRESET)
                    .any(|g| !g.values.contains(&(i as i32)))
                    // ⚠️ Os do `Mode`: a bancada corre em `Grammar`, que é onde um molde vive.
                    || gates
                        .iter()
                        .filter(|g| g.param == p.name && g.when == ls::param::MODE)
                        .any(|g| !g.values.contains(&ls::MODE_GRAMMAR))
                    // ⚠️ E o limiar, lido com os defaults de fábrica — que é o estado em que o
                    // painel abre, e portanto o que o artista de facto encontra.
                    || reg
                        .param_gates_above(ls::MANIFEST.id)
                        .into_iter()
                        .flatten()
                        .any(|g| g.param == p.name && default_of(g.when) <= g.above)
            })
            .count();
        println!(
            "| {} | {total} | {} | {escondidos} |",
            c.label,
            total - escondidos
        );
    }

    println!("\n## O que já está gateado hoje\n");
    for g in gates {
        println!(
            "- `{}` aparece quando `{}` ∈ {:?}",
            g.param, g.when, g.values
        );
    }

    println!("\n## O veredito\n");
    if dead.is_empty() {
        println!("Nenhum param inerte no esqueleto.");
    }
    println!("### Mortos pela GRAMÁTICA — cura: esconder por molde\n");
    for (name, mortos) in &dead {
        println!(
            "- `{name}` — inerte em {} molde(s): {mortos:?}",
            mortos.len()
        );
    }
    println!("\n### DEPENDENTES — ⛔ cura NÃO é esconder por molde\n");
    // ⚠️ **QUAL param o acorda é o que ESCREVE o gate.** Sem esta coluna a secção acima diz
    // *"depende de alguma coisa"*, que não é accionável — e a tentação seguinte é escondê-lo
    // por molde, que é precisamente a cura errada.
    for (name, d) in &depends {
        let Some(hint) = hints.iter().find(|h| h.param == *name) else {
            continue;
        };
        // O molde onde ele dorme, para perguntar lá.
        let c = cs.iter().find(|c| c.label == d[0]).expect("molde medido");
        let mut culpados: Vec<&str> = Vec::new();
        for (waker, wv) in &awake {
            if waker == name {
                continue;
            }
            let mut prints = std::collections::BTreeSet::new();
            for v in samples(hint, default_of(name)) {
                let mut ov = c.frame.clone();
                ov.retain(|(k, _)| k != waker && k != name);
                ov.push((waker, *wv));
                ov.push((name, v));
                ov.push((ls::param::GEOMETRY, ls::GEOMETRY_BRANCHES as f32));
                prints.insert(fingerprint(&ls::probe_build(
                    c.axiom,
                    c.rules,
                    c.generations,
                    &ov,
                )));
            }
            if prints.len() > 1 {
                culpados.push(waker);
            }
        }
        // A meia geração não é um param: ela é o estado em que o `Generations` está a ser
        // animado, e é o que dá sujeito aos dois interruptores do crescimento suave.
        let mut meia = std::collections::BTreeSet::new();
        for v in samples(hint, default_of(name)) {
            let mut ov = c.frame.clone();
            ov.retain(|(k, _)| k != name);
            ov.push((name, v));
            ov.push((ls::param::GEOMETRY, ls::GEOMETRY_BRANCHES as f32));
            meia.insert(fingerprint(&ls::probe_build(
                c.axiom,
                c.rules,
                c.generations + 0.5,
                &ov,
            )));
        }
        if meia.len() > 1 {
            culpados.push("(geração fraccionária)");
        }
        println!(
            "- `{name}` — dorme em {} molde(s) {d:?}; ACORDA com {culpados:?}",
            d.len()
        );
    }
}
