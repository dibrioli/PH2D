//! **O `Treat as Wind` DIZ A MESMA COISA NOS DOIS NÓS** (doc 89, folha 02).
//!
//! A referência põe o par *Wind / Air Resistance* no **POP Wind** e o *Treat as Wind* no
//! **POP Axis Force**; aqui eles são o `force.wind` e o `force.vortex`. A lei — `a =
//! resistência · (alvo − v)` — é **uma linha de aritmética em cada crate**, e duplicá-la
//! foi decisão: uma porta partilhada para uma expressão de um termo seria uma crate a mais
//! para nada, e é o que as cópias de `hash.rs`/`trig.rs` desta casa já fazem.
//!
//! ⚠️ **O que NÃO se pode duplicar é o VOCABULÁRIO.** Se um dos dois chamasse o param
//! `wind_mode` e o outro `mode`, ou se um oferecesse `Force / Target Velocity` e o outro
//! `Acceleration / Velocity`, o artista teria de aprender duas vezes o mesmo conceito — e
//! um documento que copiasse um param de um para o outro cairia no default em silêncio.
//! *Uma lei de uma linha não precisa de porta; um nome precisa.*

use ph2d_node_registry::{NodeRegistry, ParamWidget};

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("os nos registram");
    reg
}

/// Os dois nós que oferecem o modo alvo-velocidade.
const PAIR: &[&str] = &["force.wind", "force.vortex"];

/// ⭐⭐ **AS DUAS CHAVES, OS DOIS DEFAULTS E OS DOIS RÓTULOS SÃO OS MESMOS.**
#[test]
fn both_forces_speak_the_same_target_velocity_vocabulary() {
    let reg = registry();
    assert_eq!(
        ph2d_node_force_wind::MODE,
        ph2d_node_force_vortex::MODE,
        "a chave do modo"
    );
    assert_eq!(
        ph2d_node_force_wind::AIR_RESIST,
        ph2d_node_force_vortex::AIR_RESIST,
        "a chave da resistencia"
    );
    assert_eq!(
        ph2d_node_force_wind::MODE_LABELS,
        ph2d_node_force_vortex::MODE_LABELS,
        "os rotulos do modo"
    );
    // ⚠️⚠️ **Compara-se o TEXTO RESOLVIDO, e nao as duas constantes.** Desde que o rotulo e' uma
    // chave derivada do TIPO, estas duas sao `node.force.wind.param.mode` e
    // `node.force.vortex.param.mode` — diferentes **por construcao**, e uma igualdade entre elas
    // seria impossivel de satisfazer sem quebrar a lei da chave. ⭐ E o gate fica MAIS FORTE: ele
    // deixou de afirmar que dois literais estao escritos igual e passou a afirmar que o artista
    // le' a mesma palavra nos dois cartoes, que e' a propriedade que ele sempre quis.
    assert_eq!(
        ph2d_i18n::tr(ph2d_node_force_wind::MODE_LABEL),
        ph2d_i18n::tr(ph2d_node_force_vortex::MODE_LABEL),
        "o rotulo da PERGUNTA -- os dois fazem a mesma, logo ela tem UM nome"
    );
    let mode = ph2d_node_force_wind::MODE;
    let air = ph2d_node_force_wind::AIR_RESIST;
    for ty in PAIR {
        let m = reg
            .manifests()
            .find(|m| m.name == *ty)
            .unwrap_or_else(|| panic!("`{ty}` registado"));
        assert_eq!(
            m.param_default(mode),
            Some(0.0),
            "`{ty}` nasce em Force -- o modo novo nunca muda um documento que ja' existe"
        );
        assert_eq!(
            m.param_default(air),
            Some(1.0),
            "`{ty}`: a resistencia nasce a 1"
        );
        let hints = reg
            .param_ui(m.id)
            .unwrap_or_else(|| panic!("`{ty}` tem hints"));
        let h = hints
            .iter()
            .find(|h| h.param == mode)
            .unwrap_or_else(|| panic!("`{ty}` tem hint do modo"));
        let ParamWidget::Enum { labels } = h.widget else {
            panic!("`{ty}`: o modo e' um enum")
        };
        assert_eq!(labels, ph2d_node_force_wind::MODE_LABELS, "`{ty}`");
        assert_eq!(
            ph2d_i18n::tr(h.label),
            ph2d_i18n::tr(ph2d_node_force_wind::MODE_LABEL),
            "`{ty}`: o rotulo do painel"
        );
        let a = hints
            .iter()
            .find(|h| h.param == air)
            .unwrap_or_else(|| panic!("`{ty}` tem hint da resistencia"));
        assert_eq!(
            ph2d_i18n::tr(a.label),
            "Air Resistance",
            "`{ty}`: o rotulo do painel"
        );
        // E a resistência só aparece no modo que a lê, nos dois.
        let gates = reg
            .param_gates(m.id)
            .unwrap_or_else(|| panic!("`{ty}` tem gates"));
        let g = gates
            .iter()
            .find(|g| g.param == air)
            .unwrap_or_else(|| panic!("`{ty}`: a resistencia e' gateada"));
        assert_eq!(g.when, mode, "`{ty}`");
        assert_eq!(g.values, &[1], "`{ty}`");
    }
}

/// ⚠️ **E o par não pode crescer sem esta conversa.** Um terceiro nó que ofereça um param
/// chamado `mode` com estes rótulos e não esteja em [`PAIR`] passou por baixo do censo.
#[test]
fn no_third_node_offers_this_vocabulary_unnoticed() {
    let reg = registry();
    let labels = ph2d_node_force_wind::MODE_LABELS;
    let stray: Vec<&str> = reg
        .manifests()
        .filter(|m| !PAIR.contains(&m.name))
        .filter(|m| {
            reg.param_ui(m.id).is_some_and(|hs| {
                hs.iter().any(|h| match h.widget {
                    ParamWidget::Enum { labels: l } => l == labels,
                    _ => false,
                })
            })
        })
        .map(|m| m.name)
        .collect();
    assert!(
        stray.is_empty(),
        "estes nos oferecem os rotulos `{labels:?}` e nao estao no par que este censo \
         compara -- ou entram nele, ou os rotulos deles dizem outra coisa: {stray:?}"
    );
}

/// ⭐⭐⭐ **O RÓTULO DA PERGUNTA TEM DE NOMEAR A PERGUNTA — e a régua é DERIVADA.**
///
/// ⚠️ **A régua não é uma lista de palavras proibidas nem um limiar escolhido.** Um rótulo
/// ensina quando ele significa **a mesma coisa em todo lado**: se outro nó pinta a MESMA
/// palavra sobre um enum de **valores diferentes**, então a palavra não é o nome de uma
/// pergunta — é um espaço reservado, e o artista tem de a reaprender em cada cartão.
///
/// ⇒ a lei, sem constante nenhuma: **todo nó que pinta este rótulo oferece as mesmas
/// opções.**
///
/// ## ⛔ O censo do repo, medido em 2026-09-09
///
/// A casa cumpre esta lei quase toda: `Curve` são **7** nós e **1** pergunta ·
/// `Noise Type` 2/1 · `Time Mode` 3/1 · `Range` 3/1 · `Edge` 2/1 · `Distance` 2/1.
/// O `Mode` é o oposto: **26 nós, 24 perguntas distintas** — a palavra mais usada do
/// catálogo é a que menos diz. Aqui ela era pior ainda, porque **os dois forces fazem a
/// mesma pergunta** e a referência já lhe dá nome (o *Treat as Wind* do POP Axis Force).
///
/// ⚠️ **Só o par deste ciclo é curado — os outros 24 ficam NOMEADOS com a medição**, como
/// os rótulos de mistura do ciclo 4 (`Blend`/`Shadow Blend`/`Flash Operator`/…): alinhar
/// o resto é wave dos ciclos que possuem aqueles grupos, e contrabandeá-la aqui mexeria
/// em nós que este ciclo não auditou.
///
/// A sonda que produziu esses números:
/// ```text
/// cargo test -p ph2d-node-registry-init --test wind_vocabulary -- --ignored --nocapture the_house_census_of_enum_labels
/// ```
#[test]
fn the_mode_label_names_the_question_it_asks() {
    let reg = registry();
    let mode = ph2d_node_force_wind::MODE;
    let nosso = {
        let m = reg
            .manifests()
            .find(|m| m.name == PAIR[0])
            .expect("`force.wind` registado");
        reg.param_ui(m.id)
            .expect("hints")
            .iter()
            .find(|h| h.param == mode)
            .expect("hint do modo")
            .label
    };
    // ⚠️⚠️ **A comparação é do TEXTO, e a razão é muda se se esquecer.** Desde que o rótulo é
    // uma chave derivada do TIPO, dois nós que façam a mesma pergunta têm chaves DIFERENTES por
    // construção — uma busca por `h.label == nosso` deixaria de achar ninguém, a lista de
    // acusados ficaria vazia, e **uma lista vazia lê-se aqui como aprovado**.
    let nosso = ph2d_i18n::tr(nosso);
    let alheios: Vec<(&str, &[&str])> = reg
        .manifests()
        .filter(|m| !PAIR.contains(&m.name))
        .filter_map(|m| {
            let hs = reg.param_ui(m.id)?;
            let h = hs.iter().find(|h| ph2d_i18n::tr(h.label) == nosso)?;
            match h.widget {
                ParamWidget::Enum { labels } => Some((m.name, labels)),
                _ => None,
            }
        })
        .filter(|(_, labels)| *labels != ph2d_node_force_wind::MODE_LABELS)
        .collect();
    assert!(
        alheios.is_empty(),
        "o rotulo `{nosso}` e' pintado por {} outro(s) no com opcoes DIFERENTES, logo ele \
         nao nomeia pergunta nenhuma -- o artista tem de o reaprender em cada cartao: {alheios:?}",
        alheios.len()
    );
}

/// O CENSO da casa: que rótulo de enum é pintado por quantos nós, e sobre quantas
/// perguntas distintas. É de onde saem os números do doc-comment acima.
#[test]
#[ignore = "sonda de auditoria — corra à mão"]
fn the_house_census_of_enum_labels() {
    use std::collections::BTreeMap;
    let reg = registry();
    let mut tabela: BTreeMap<&str, BTreeMap<String, Vec<&str>>> = BTreeMap::new();
    for m in reg.manifests() {
        let Some(hs) = reg.param_ui(m.id) else {
            continue;
        };
        for h in hs {
            if let ParamWidget::Enum { labels } = h.widget {
                tabela
                    .entry(ph2d_i18n::tr(h.label))
                    .or_default()
                    .entry(labels.join(" | "))
                    .or_default()
                    .push(m.name);
            }
        }
    }
    eprintln!("\n  rotulo               | nos | perguntas distintas");
    eprintln!("  ---------------------|-----|--------------------");
    for (rotulo, perguntas) in &tabela {
        let nos: usize = perguntas.values().map(Vec::len).sum();
        if nos < 2 {
            continue;
        }
        let marca = if perguntas.len() > 1 { "  " } else { "OK" };
        eprintln!("{marca} {rotulo:<20} | {nos:>3} | {}", perguntas.len());
        for (opcoes, quem) in perguntas {
            eprintln!("       {opcoes:<46} {quem:?}");
        }
    }
    eprintln!();
}
