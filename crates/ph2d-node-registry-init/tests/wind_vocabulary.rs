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
        assert_eq!(h.label, "Mode", "`{ty}`: o rotulo do painel");
        let a = hints
            .iter()
            .find(|h| h.param == air)
            .unwrap_or_else(|| panic!("`{ty}` tem hint da resistencia"));
        assert_eq!(a.label, "Air Resistance", "`{ty}`: o rotulo do painel");
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
