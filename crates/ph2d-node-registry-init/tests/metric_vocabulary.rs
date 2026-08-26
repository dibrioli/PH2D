//! **A MÉTRICA DIZ A MESMA COISA NOS DOIS NÓS** (doc 89, folha 06 linha 21).
//!
//! O `motion.voronoi` mede distância para decidir de que célula um ponto é; o `motion.noise`
//! mede distância ao ponto-feição na base **Cellular**. É a mesma pergunta, e um artista que
//! aprendeu «Chebyshev» num não pode encontrar «Máximo» no outro.
//!
//! ⚠️ **A lei de uma linha não precisa de porta; um NOME precisa.** A conta da distância é
//! três braços de `match` em cada crate e duplicá-la foi decisão — o que não se pode
//! duplicar é o vocabulário. É o irmão exacto do censo `wind_vocabulary`, e nasceu do mesmo
//! achado.

use ph2d_node_registry::{NodeRegistry, ParamWidget};

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("os nos registram");
    reg
}

/// Os rótulos do param `metric` de um nó, se ele o oferecer.
fn metric_labels(reg: &NodeRegistry, ty: &str) -> Option<&'static [&'static str]> {
    let m = reg.manifests().find(|m| m.name == ty)?;
    reg.param_ui(m.id)?.iter().find_map(|h| match h.widget {
        ParamWidget::Enum { labels } if h.param == "metric" => Some(labels),
        _ => None,
    })
}

/// Os dois nós que hoje medem uma distância escolhida pelo artista.
const PAIR: &[&str] = &["motion.voronoi", "motion.noise"];

/// ⭐⭐ **AS DUAS LISTAS SÃO A MESMA LISTA**, na mesma ordem.
///
/// ⚠️ **A ORDEM importa tanto quanto as palavras**: o valor guardado num documento é o
/// ÍNDICE, não o nome. Duas listas com as mesmas três palavras trocadas fariam um documento
/// copiado de um nó para o outro mudar de métrica em silêncio.
#[test]
fn both_nodes_speak_the_same_distance_vocabulary() {
    let reg = registry();
    let mut seen: Vec<(&str, &[&str])> = Vec::new();
    for ty in PAIR {
        let l = metric_labels(&reg, ty)
            .unwrap_or_else(|| panic!("`{ty}` oferece um param `metric` com rotulos"));
        seen.push((ty, l));
    }
    let (first_ty, first) = seen[0];
    for (ty, l) in &seen[1..] {
        assert_eq!(
            l, &first,
            "`{ty}` e `{first_ty}` medem a mesma coisa e chamam-lhe nomes diferentes"
        );
    }
    // E o rótulo do painel é o mesmo nos dois.
    for ty in PAIR {
        let m = reg.manifests().find(|m| m.name == *ty).expect("registado");
        let h = reg
            .param_ui(m.id)
            .expect("hints")
            .iter()
            .find(|h| h.param == "metric")
            .expect("hint da metrica");
        assert_eq!(
            h.label, "Distance",
            "`{ty}`: o rotulo do painel -- o `motion.voronoi` ja shipava `Distance`, e este \
             censo apanhou a divergencia no dia em que o segundo no' a escreveu"
        );
    }
}

/// ⚠️ **E o par não pode crescer sem esta conversa.** Um terceiro nó com um param `metric`
/// que não esteja em [`PAIR`] passou por baixo do censo.
#[test]
fn no_third_node_measures_distance_unnoticed() {
    let reg = registry();
    let stray: Vec<&str> = reg
        .manifests()
        .filter(|m| !PAIR.contains(&m.name))
        .filter(|m| {
            reg.param_ui(m.id)
                .is_some_and(|hs| hs.iter().any(|h| h.param == "metric"))
        })
        .map(|m| m.name)
        .collect();
    assert!(
        stray.is_empty(),
        "estes nos oferecem um param `metric` e nao estao no par que este censo compara -- \
         ou entram nele, ou o param deles chama-se outra coisa: {stray:?}"
    );
}
