//! Gates da cena `=38` — **a DIREÇÃO** (doc 89 §10.0, a linha que cinco famílias citaram).
//!
//! ⚠️ A cena é um **A/B**, e um A/B tem dois modos de falhar em silêncio: os dois lados
//! iguais (o canal não chegou) **e** os dois lados girando (o "controle" não é controle). Os
//! gates abaixo afirmam os dois, e o segundo é o que impede um gate de passar sobre uma cena
//! que perdeu o seu próprio contraste.

use super::*;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::value::CookValue;

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("os nos registram");
    reg
}

/// A coluna `rot` que um sink deixa, alguns tiques depois do repouso — a sim tem de ter
/// ANDADO, senão toda velocidade é zero e as duas metades concordam por vácuo.
fn rotations(doc: &MotionDoc, reg: &NodeRegistry, sink: NodeId) -> Vec<f32> {
    let mut c = Cook::new();
    // ⚠️ O laço `pre` é o que faz a velocidade existir: no tique 0 nada se move ainda. Cozer
    // uma vez e ler `rot` mediria um redemoinho parado.
    let mut last = Vec::new();
    for step in 0..24 {
        let t = f64::from(step) / 60.0;
        let out = c.cook(&doc.graph, reg, sink, t).expect("a cena coze");
        let CookValue::Instances(s) = &out[0] else {
            panic!("a saida e um stream")
        };
        last = match Stream::get(s, "rot") {
            Some(Column::Scalar(v)) => v.clone(),
            _ => Vec::new(),
        };
        // ⚠️ **Sem isto a velocidade nunca existe.** O `advance_tick` e' quem tira o snapshot
        // que a aresta `pre` le no tique seguinte, e o doc dele diz *"uma vez por quadro,
        // DEPOIS do cook"*. A 1ª versao deste harness so cozia: as duas nuvens ficavam
        // paradas, `rot` saia toda zero, e o gate acusava o CANAL de um defeito da fixture.
        c.advance_tick(&doc.graph, reg, t).expect("o tique avanca");
    }
    last
}

/// **A cena constrói os DOIS lados.** Se um `.ok()?` engolisse uma aresta, o roteador
/// devolveria `unwrap_or_default()` — uma tela VAZIA, que num smoke lê como *"a feature não
/// foi construída"* em vez de *"a cena está partida"*.
#[test]
fn the_direction_scene_builds_both_clouds() {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_direction_demo_document(&mut doc, &reg).expect("a cena constroi");
    assert_eq!(sinks.len(), 2, "duas nuvens: sem o canal e com ele");
}

/// **O lado alinhado GIRA e o controle NÃO** — a 4ª condição de UI (*a sequência leva a algum
/// lugar*), medida no stream que o render de facto consome.
///
/// ⚠️ As duas metades são independentes e ambas são precisas: sem a primeira, um canal que
/// nunca chegasse passaria (as duas nuvens paradas); sem a SEGUNDA, uma cena cujo controle
/// também girasse passaria, e o A/B que o olho julga teria deixado de existir sem nada dizer.
#[test]
fn only_the_aligned_cloud_turns_and_it_turns_by_a_lot() {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_direction_demo_document(&mut doc, &reg).expect("a cena constroi");

    let plain = rotations(&doc, &reg, sinks[0]);
    let aligned = rotations(&doc, &reg, sinks[1]);
    assert!(
        !aligned.is_empty(),
        "o lado alinhado escreve a coluna `rot`"
    );

    // O CONTROLE. ⚠️ **Escrito como AUSENCIA e nao como `all(|r| r < eps)`**: medido, sem o
    // canal a coluna `rot` nao existe, e `all` sobre um vetor VAZIO e verdade por VACUO — o
    // gate passaria tambem no dia em que o lado do controle deixasse de produzir alguma coisa.
    // A afirmacao honesta e' a que a medicao devolveu: *ninguem escreve `rot` ali*.
    assert!(
        plain.is_empty(),
        "o controle NAO pode girar: sem o canal nada escreve `rot`, e achei {} valores",
        plain.len()
    );
    // ...e o lado alinhado escreve UM angulo por peca, nao um punhado.
    assert_eq!(
        aligned.len(),
        (SIDE * SIDE) as usize,
        "o canal escreve por ELEMENTO"
    );

    // E o lado alinhado tem de cobrir o CIRCULO, nao um angulo qualquer: um redemoinho poe
    // pecas em todas as direcoes, e um canal que devolvesse uma constante (ou os zeros do
    // miss ordinario) daria uma faixa estreita.
    let lo = aligned.iter().cloned().fold(f32::INFINITY, f32::min);
    let hi = aligned.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    eprintln!(
        "[direction] controle: {} pecas, rot maxima {:.4} | alinhado: faixa {lo:.1}..{hi:.1} graus",
        plain.len(),
        plain.iter().fold(0.0_f32, |a, r| a.max(r.abs())),
    );
    assert!(
        hi - lo > 180.0,
        "as pecas de um redemoinho apontam para todo lado: a faixa medida e' {lo:.1}..{hi:.1} \
         graus, e uma faixa estreita seria um canal a devolver uma constante"
    );
    // E em GRAUS: `atan2` cobre −180..180, entao a faixa de um redemoinho nao cabe em
    // radianos (−pi..pi) por um fator de ~57.
    assert!(
        hi > 90.0 && lo < -90.0,
        "os extremos ({lo:.1}, {hi:.1}) tem de ser GRAUS — em radianos caberiam em +-3.15"
    );
}
