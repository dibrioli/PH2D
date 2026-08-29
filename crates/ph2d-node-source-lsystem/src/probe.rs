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
pub fn probe_anchor(axiom: &str, rules: &str, generations: f32) -> f32 {
    let p = probe_params(generations, &[(param::CONTINUOUS_ANGLE, 1.0)]);
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
    };
    let antes = turtle::span(&d.previous, &setup(1.0));
    // ⚠️ **Com as dobras FECHADAS** — é a pose de onde a interpolação parte. Medi-la aberta dá
    // `1/3` onde a resposta é `1/5`, e uma mutação que trocasse as duas já SOBREVIVEU uma vez.
    let achatada = turtle::span(&d.chain, &setup(0.0));
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
