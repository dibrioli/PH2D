//! **O WIDGET DIZ O QUE O NÚMERO É** — a convenção do catálogo, agora executável.
//!
//! Todo param é um `f32` no `NodeManifest` (contrato CONGELADO, §6), então o TIPO não distingue
//! *uma magnitude* de *uma escolha* nem de *uma semente*. Quem distingue é o `ParamWidget`, que é
//! side-metadata — e side-metadata sem gate **apodrece por straggler**: a varredura de 2026-08-08
//! achou 11 nós pintando `seed` como dado e **4 pintando como slider**, 17 pintando `mode` como
//! palavras e **1 como o número cru**.
//!
//! ⚠️ **Isto é a doença de "mesmo fato, duas respostas", não gosto de UI.** Arrastar uma semente
//! não quer dizer nada — toda vizinha é tão boa quanto ela, e o que o artista quer é *outra*, que
//! é exatamente o botão de re-rolar. Ler `2` onde o nó tem três modos nomeados é o artista
//! decorando um índice que a fonte já nomeia.
//!
//! ⚠️ E a 3ª regra é a única enunciada como **PROPRIEDADE** em vez de nome, o que a torna a mais
//! forte das três: *um slider cujo passo mede o curso inteiro tem duas posições*. Nenhum nome
//! precisa ser conhecido para ela morder.

use ph2d_node_registry::{NodeRegistry, ParamWidget};

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("every node registers");
    reg
}

/// Toda linha `(nó, param, widget)` do catálogo, para as três leis abaixo compartilharem UMA
/// varredura — três varreduras seriam três oportunidades de uma delas ficar para trás.
fn hints() -> Vec<(&'static str, &'static str, ParamWidget, f32, f32, f32)> {
    let reg = registry();
    let mut out = Vec::new();
    for m in reg.manifests() {
        for h in reg.param_ui(m.id).unwrap_or(&[]) {
            out.push((m.name, h.param, h.widget, h.min, h.max, h.step));
        }
    }
    out
}

/// **Uma SEMENTE é re-rolada, não arrastada.**
#[test]
fn a_seed_wears_the_seed_widget() {
    let rows = hints();
    let seeds: Vec<_> = rows
        .iter()
        .filter(|(_, p, ..)| *p == "seed" || p.ends_with("_seed"))
        .collect();

    // Controle positivo: uma varredura vazia passaria em silêncio por vácuo.
    assert!(
        seeds.len() >= 10,
        "a varredura achou {} sementes -- o scanner quebrou, nao o catalogo",
        seeds.len()
    );

    let wrong: Vec<String> = seeds
        .iter()
        .filter(|(_, _, w, ..)| *w != ParamWidget::Seed)
        .map(|(n, p, w, ..)| format!("{n}.{p} = {w:?}"))
        .collect();
    assert!(
        wrong.is_empty(),
        "semente pintada como magnitude (arrastar uma semente nao quer dizer nada): {wrong:?}"
    );
}

/// **Um MODO mostra PALAVRAS.** `Channels` conta: o picker de canais É a lista de escolhas do
/// `value.attribute`, e um `Enum` ao lado dele seria a segunda porta para a mesma pergunta.
#[test]
fn a_mode_wears_words_not_an_index() {
    let rows = hints();
    let modes: Vec<_> = rows
        .iter()
        .filter(|(_, p, ..)| *p == "mode" || p.ends_with("_mode"))
        .collect();

    assert!(
        modes.len() >= 15,
        "a varredura achou {} modos -- o scanner quebrou, nao o catalogo",
        modes.len()
    );

    let wrong: Vec<String> = modes
        .iter()
        .filter(|(_, _, w, ..)| {
            !matches!(w, ParamWidget::Enum { .. } | ParamWidget::Channels { .. })
        })
        .map(|(n, p, w, ..)| format!("{n}.{p} = {w:?}"))
        .collect();
    assert!(
        wrong.is_empty(),
        "modo pintado como indice cru (o artista decora um numero que a fonte ja nomeia): {wrong:?}"
    );
}

/// **Um slider de duas posições é um TOGGLE** — e esta é a única das três leis que não conhece
/// nome nenhum, então ela morde um param que ninguem previu.
#[test]
fn a_two_position_slider_is_a_toggle() {
    let rows = hints();

    let mut scanned = 0usize;
    let wrong: Vec<String> = rows
        .iter()
        .filter(|(_, _, w, ..)| matches!(w, ParamWidget::Slider | ParamWidget::IntSlider))
        .inspect(|_| scanned += 1)
        .filter(|(_, _, _, min, max, step)| *step > 0.0 && *step >= (*max - *min))
        .map(|(n, p, _, min, max, step)| format!("{n}.{p} (range {min}..{max}, step {step})"))
        .collect();

    assert!(
        scanned >= 200,
        "a varredura olhou {scanned} sliders -- o scanner quebrou, nao o catalogo"
    );
    assert!(
        wrong.is_empty(),
        "slider cujo passo mede o curso inteiro: tem duas posicoes, entao e um Toggle -- \
         pinta-lo como arrasto continuo promete um meio-termo que o eval nao le: {wrong:?}"
    );
}
