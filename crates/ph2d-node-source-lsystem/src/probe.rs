//! **AS PORTAS DE SONDA — como um gate ou uma bancada alcança o produto**
//!
//! Separado do [`super`] pelo teto de LOC (HR-18, 700 na workspace), no corte que a
//! pergunta desenha: lá fica *o produto*, aqui *as portas por onde ele se mede*.
//!
//! ⚠️ Elas são `pub` sem `#[cfg(test)]` de propósito: as bancadas (`examples/`, `tests/`)
//! são alvos de integração e não veem itens de teste. Uma porta que só o teste unitário
//! alcança obrigaria a bancada a reimplementar o caminho — e aí ela mediria outro código.

use super::*;

/// O default que o MANIFESTO declara para um param — a única fonte.
fn manifest_default(name: &str) -> f32 {
    MANIFEST
        .params
        .iter()
        .find(|p| p.name == name)
        .map_or(0.0, |p| p.default)
}

/// **A porta de SONDA da ÂNCORA** — o factor de comprimento em `frac → 0`, que é onde a
/// interpolação começa.
///
/// ⚠️ A âncora deixou de ser uma constante em 2026-08-29: ela é o valor, no início da
/// travessia, da normalização que o [`build`] faz a cada instante (*«que comprimento põe a
/// figura na rampa recta entre a geração anterior e a nova inteira?»*). Aqui mede-se esse
/// valor sozinho, para um gate poder afirmar o NÚMERO e não só o efeito.
#[must_use]
pub fn probe_anchor(axiom: &str, rules: &str, generations: f32, overrides: &[(&str, f32)]) -> f32 {
    // ⚠️ **Ela RECEBE os overrides desde 2026-08-30** — media sempre a `25°`, e a Koch e o
    // Dragon são `90°` **por definição**. É o mesmo defeito que a irmã
    // [`probe_growth_ratio`] já tinha corrigido, e que ficou nesta porta: *uma sonda que não
    // recebe o estado mede outro produto.* Escapava por acidente (a âncora de uma gramática
    // auto-semelhante é uma razão de ESCALA, e escala não vê ângulo) e deixaria de escapar na
    // primeira que não o fosse.
    let mut ov: Vec<(&str, f32)> = vec![(param::CONTINUOUS_ANGLE, 1.0)];
    ov.extend_from_slice(overrides);
    let p = probe_params(generations, &ov);
    let params = |n: &str| p.by_name(n);
    let (gens, _) = generation_plan(p.generations);
    let d = derive::derive(
        &derive::axiom_modules(axiom, &params),
        &grammar::parse_rules(rules),
        gens,
        1,
        MAX_MODULES,
        &params,
    );
    if d.previous.is_empty() {
        return 1.0;
    }
    let setup = |ang: f32| turtle::Setup {
        angle: p.angle,
        step: p.step,
        width: p.width,
        width_scale: p.width_scale,
        length_scale: p.length_scale,
        root_angle: p.root_angle,
        tropism: p.tropism,
        tropism_angle: p.tropism_angle,
        angle_frac: ang,
        youngest: (d.generations, 1.0),
        orient_world: true,
        leaf_first_level: p.leaf_first_level,
        leaf_angle: p.leaf_angle,
        leaf_spread: p.leaf_spread,
        leaf_effects: p.leaf_effects.round() as i32 != 0,
        seed: p.seed,
    };
    let antes = turtle::mean_width(&d.previous, &setup(1.0));
    // ⚠️ **Com as dobras FECHADAS** — é a pose de onde a interpolação parte.
    //
    // ⛔⛔ **A NOTA QUE AQUI ESTAVA FOI REFUTADA em 2026-08-30** (auditoria adversarial). Ela
    // dizia *«medi-la aberta dá `1/3` onde a resposta é `1/5`»* — com a régua invariante à
    // rotação as duas dão `1/3` (`0,333333` fechada contra `0,333289` aberta, **`0,013 %`**),
    // e a mutação que troca `setup(0.0)` por `setup(1.0)` **volta a sobreviver**. *A largura
    // média é quase cega ao dobrar; era a caixa de eixo que via aquela diferença.*
    //
    // ⇒ quem guarda esta linha é a fixtura do Weed no gate da âncora (não auto-semelhante), e
    // a pose tem gate próprio em `turtle_tests::the_newest_generation_opens_its_folds…`.
    let achatada = turtle::mean_width(&d.chain, &setup(0.0));
    if antes > 1e-6 && achatada > 1e-6 {
        (antes / achatada).clamp(0.02, 1.0)
    } else {
        1.0
    }
}

/// **A RAZÃO DE EXPANSÃO QUE O NÓ MEDIU** — `1.0` quando a gramática converge (e portanto não
/// é remapeada pelo `Growth`).
///
/// ⚠️ Existe para um gate poder perguntar ao PRODUTO *«esta gramática é auto-semelhante?»* em
/// vez de responder sozinho. A 1.ª redacção do gate do arrasto tinha o seu próprio critério
/// (a razão entre duas gerações consecutivas) e **discordava do nó no Dragon**, cuja razão
/// oscila — ele ficava na família errada e o gate media a lei que o produto não aplica.
/// *Duas respostas à mesma pergunta, e a que o artista vê é a outra.*
///
/// ⚠️ **Ela recebe os OVERRIDES, e a 1.ª redacção não recebia** — media com o ângulo de
/// fábrica (`25°`) em vez do que o molde exige, e a Koch (que é `90°` por definição) saía a
/// `4,81` em vez de `3,00`. *Uma sonda que não recebe o estado mede outro produto.*
#[must_use]
pub fn probe_growth_ratio(axiom: &str, rules: &str, overrides: &[(&str, f32)]) -> f32 {
    measure_ratio(axiom, rules, &probe_params(5.0, overrides))
}

/// **A razão CRUA, antes do limiar** — e o limiar em si, para um gate poder medir a FOLGA com
/// que ele parte o corpus em vez de só confiar nela.
///
/// ⚠️ Uma régua nova reembaralha as duas famílias em silêncio: o limiar continua a devolver
/// `1,0` para um lado e o número para o outro, e nada acusa se a fronteira passar a cortar no
/// meio de um grupo. *Um limiar sem os dois lados medidos é um palpite com cara de medição.*
#[must_use]
pub fn probe_growth_ratio_raw(axiom: &str, rules: &str, overrides: &[(&str, f32)]) -> f32 {
    growth::raw_ratio(axiom, rules, &probe_params(5.0, overrides))
}

/// **A FAMÍLIA, pela porta do produto** — `true` = refina, `false` = cresce pela ponta.
///
/// ⚠️ Ela substituiu `probe_still_multiplies`: até 2026-08-30 a família saía de um limiar
/// sobre a razão medida, e ele **esgotou-se** (o modo guiado ficou a `0,017 %` dele). Ver
/// `derive::Derived::grows_by_refining` para os números que o mataram.
#[must_use]
pub fn probe_grows_by_refining(axiom: &str, rules: &str, overrides: &[(&str, f32)]) -> bool {
    growth::raw_ratio_and_family(axiom, rules, &probe_params(5.0, overrides)).1
}

/// **OS PESOS QUE O PARSER DE FACTO DEVOLVE** — a porta de sonda que impede um gate de
/// escrever o próprio oráculo.
///
/// ⚠️ O gate `variation_gives_three_weighted_rules_whose_weights_close_at_one` lia os pesos do
/// texto com um `str::parse::<f32>()` PRÓPRIO, e por isso ficava verde em `v = 1,0`: o texto
/// somava `1,0` (`0.000 + 0.500 + 0.500`) enquanto o motor somava `2,0` (o `(0.000)` virava o
/// neutro). **Dois leitores do mesmo texto, e o gate escolheu o que não está no produto.**
#[must_use]
pub fn probe_rule_weights(rules: &str) -> Vec<f32> {
    grammar::parse_rules(rules)
        .iter()
        .map(|r| r.weight)
        .collect()
}

#[must_use]
fn probe_params(generations: f32, overrides: &[(&str, f32)]) -> Params {
    let mut p = Params {
        generations,
        angle: 25.0,
        step: 0.5,
        width: 1.0,
        width_scale: 0.7,
        length_scale: 0.9,
        root_angle: 90.0,
        tropism: 0.0,
        tropism_angle: -90.0,
        seed: 1.0,
        orient: 0.0,
        // ⚠️⚠️ **`Grammar`, e NÃO o default do manifesto.** Esta porta recebe um axioma e
        // umas regras nos ARGUMENTOS; abri-la em `Guided` faria o nó ignorá-los, e as
        // dezenas de gates que a chamam passariam a medir a gramática derivada em vez da
        // que escreveram — todos verdes, todos sobre outra coisa. *Uma porta de sonda tem
        // de honrar o que lhe é passado, e o modo é o que decide se ela o honra.*
        mode: MODE_GRAMMAR as f32,
        branches: 2.0,
        segments: 1.0,
        variation: 0.0,
        bend: 0.0,
        // ⚠️⚠️ **LIDOS DO MANIFESTO, nunca cravados** — e a diferença mordeu no mesmo dia: com
        // o `continuous_angle` a `1.0` aqui, a bancada continuou a imprimir os números CURADOS
        // depois de o default do produto ter ido a `0.0`. *Uma sonda com o default cravado
        // mede o que ela acha que o produto é.*
        continuous_length: manifest_default(param::CONTINUOUS_LENGTH),
        continuous_angle: manifest_default(param::CONTINUOUS_ANGLE),
        step_scale: manifest_default(param::STEP_SCALE),
        growth: manifest_default(param::GROWTH),
        // ⚠️ **`Segments`, e NÃO o default do manifesto** — pela mesma razão que o `mode` acima:
        // as sondas medem a CADEIA (contagens, larguras, a razão de expansão), e em `Branches` o
        // nó devolve fitas. Uma porta de sonda que abrisse no modo novo passaria a medir outra
        // coisa, com todos os gates verdes.
        geometry: GEOMETRY_SEGMENTS as f32,
        tip_taper: 0.0,
        // ⚠️ **Os defaults do MANIFESTO, escritos por nome** — uma sonda que os inventasse
        // mediria outra planta que a do artista.
        leaf_first_level: 3.0,
        leaf_angle: 0.0,
        leaf_spread: 0.0,
        leaf_front: 0.0,
        leaf_effects: 0.0,
    };
    for (n, v) in overrides {
        match *n {
            param::ANGLE => p.angle = *v,
            param::MODE => p.mode = *v,
            param::BRANCHES => p.branches = *v,
            param::SEGMENTS => p.segments = *v,
            param::VARIATION => p.variation = *v,
            param::BEND => p.bend = *v,
            param::CONTINUOUS_LENGTH => p.continuous_length = *v,
            param::CONTINUOUS_ANGLE => p.continuous_angle = *v,
            param::STEP_SCALE => p.step_scale = *v,
            param::GROWTH => p.growth = *v,
            param::STEP => p.step = *v,
            param::WIDTH => p.width = *v,
            param::WIDTH_SCALE => p.width_scale = *v,
            param::LENGTH_SCALE => p.length_scale = *v,
            param::ROOT_ANGLE => p.root_angle = *v,
            param::TROPISM => p.tropism = *v,
            param::TROPISM_ANGLE => p.tropism_angle = *v,
            param::SEED => p.seed = *v,
            param::ORIENT => p.orient = *v,
            param::TIP_TAPER => p.tip_taper = *v,
            param::GEOMETRY => p.geometry = *v,
            param::LEAF_FIRST_LEVEL => p.leaf_first_level = *v,
            param::LEAF_ANGLE => p.leaf_angle = *v,
            param::LEAF_SPREAD => p.leaf_spread = *v,
            param::LEAF_FRONT => p.leaf_front = *v,
            param::LEAF_EFFECTS => p.leaf_effects = *v,
            other => panic!("probe_params: param desconhecido {other}"),
        }
    }
    p
}

/// **A porta de SONDA** — derivar + interpretar com os defaults do manifesto, mudando só o
/// que quem mede quer mudar.
///
/// ⚠️ `pub` sem `#[cfg(test)]` de propósito: a bancada que MEDE o tecto
/// (`tests/measure_lsystem_ceiling.rs`) é um alvo de integração e não vê itens de teste.
#[must_use]
pub fn probe_build(
    axiom: &str,
    rules: &str,
    generations: f32,
    overrides: &[(&str, f32)],
) -> ph2d_nodegraph::attr::Stream {
    build(axiom, rules, &probe_params(generations, overrides))
}

/// **A IMPRESSÃO DIGITAL de um stream** — todas as colunas, pelos BITS.
///
/// ⚠️ Os bits e não o decimal: duas larguras que se imprimem iguais podem diferir, e um param
/// que só mexesse na 8.ª casa seria declarado morto por um `to_string`.
///
/// ⚠️ **Ordenada pelo NOME da coluna** — a ordem de iteração de um mapa não é a pergunta, e
/// deixá-la entrar faria a impressão mudar sem o produto mudar.
fn fingerprint(s: &ph2d_nodegraph::attr::Stream) -> u64 {
    use ph2d_nodegraph::attr::Column;
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

/// ⭐⭐⭐ **QUANTAS SAÍDAS DISTINTAS este param produz** — a régua do knob morto, numa porta só.
///
/// `1` ⇒ varrer aquele param pela faixa toda não mexeu um bit: ele **não tem sujeito** naquela
/// gramática, com aqueles números. Qualquer valor acima de `1` ⇒ ele age.
///
/// ⚠️⚠️ **Porta ÚNICA de propósito.** A bancada (`examples/dead_params_report.rs`) e o portão
/// (`tests/no_preset_shows_a_knob_its_grammar_cannot_read.rs`) fazem a MESMA pergunta, e a lei
/// que a responde — *o que conta como «mudou»* — não pode viver escrita duas vezes: no dia em
/// que uma delas passasse a ignorar uma coluna, a bancada e o portão discordariam **em silêncio**
/// e o portão continuaria verde. *Uma lei escrita em dois sítios ainda não é uma lei.*
///
/// ⚠️ **O `generations` entra pelo ARGUMENTO, não pelos overrides** — [`probe_params`] recusa-o
/// (ele escolhe QUANTO se deriva, não como se interpreta), e sem este braço varrê-lo estouraria.
///
/// ⚠️ **Quem chama escolhe os VALORES**, e é isso que separa uma leitura honesta de uma acusação
/// fabricada: um param cujo sujeito outro param cria (o `tropism_angle` sem `tropism`) mede-se
/// morto com os defaults e vivo com o vizinho aceso. Ver o veredito da bancada, que faz as duas
/// leituras e **nunca** manda esconder pela primeira sozinha.
#[must_use]
pub fn probe_param_prints(
    axiom: &str,
    rules: &str,
    generations: f32,
    base: &[(&str, f32)],
    param: &str,
    values: &[f32],
) -> usize {
    let mut prints = std::collections::BTreeSet::new();
    for v in values {
        let mut ov: Vec<(&str, f32)> = base.to_vec();
        let mut gens = generations;
        if param == param::GENERATIONS {
            gens = *v;
        } else {
            ov.retain(|(n, _)| *n != param);
            ov.push((param, *v));
        }
        prints.insert(fingerprint(&probe_build(axiom, rules, gens, &ov)));
    }
    prints.len()
}

/// ⭐⭐⭐ **O QUE ESTÁ MAL NA GRAMÁTICA** — a porta pela qual o painel deixa de mentir.
///
/// # Por que ela existe
///
/// Report registado desde 2026-08-29 e aberto até aqui: *o feedback ao vivo de uma regra
/// malformada — hoje ela cai em silêncio*. A política de erro está certa e é deliberada (o que
/// não se entende **descarta a regra**, senão um erro de digitação apagava a planta enquanto se
/// escreve a segunda regra), mas descartar em silêncio faz o artista ler o resultado como *a
/// gramática que ele escreveu*. ⚠️ **A mais cara é a condição:** ela é o travão da recursão, e
/// `n <= 6` que não compila dá `16 384` módulos onde `n < 6` dá `32` — a planta muda de forma e
/// nada diz porquê.
///
/// ⚠️ **A lista sai do MESMO percurso que descarta as regras** ([`grammar::parse_rules_reporting`]),
/// nunca de um segundo leitor do texto: um contador escrito ao lado do parser é exactamente o
/// defeito que o gate dos pesos deste nó já pagou — *dois leitores do mesmo texto, e quem decide
/// é o que o artista não vê*.
///
/// ⚠️ **Vazio quer dizer «não há queixa»**, e não «não sei»: um axioma sem regras nenhumas é
/// legítimo (desenha o axioma), então a ausência de regras não é um erro.
#[must_use]
pub fn grammar_complaints(rules: &str) -> Vec<grammar::Complaint> {
    grammar::parse_rules_reporting(rules).1
}
