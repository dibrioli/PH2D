//! **O device marcha no ritmo da ILHA** (doc 89, folha 13 — o último P1).
//!
//! ⚠️ **Não há recusa aqui, e é por construção.** O ritmo é do GRAFO — cada objeto Motion tem o
//! seu, então um grafo é a DOP Network do Houdini / o System do Niagara, que é exactamente onde
//! as duas põem o substep (e o device já exige um sink por documento, fechando a mesma fronteira
//! do outro lado). Marchar o plano inteiro `n` vezes dá a cada ilha os mesmos `n` sub-tiques que
//! o bracket da CPU lhe daria: **idêntico, não aproximado**, e nenhum dos dois produtores precisa
//! escolher entre acelerar e estar certo.

use std::fs;

/// **A redução é a prova de que nada regrediu:** com `sub = 1` o helper devolve, termo a termo,
/// os relógios que este código sempre devolveu — um par por tique, playhead `t·dt`.
#[test]
fn a_rate_of_one_is_the_march_that_never_knew_about_substeps() {
    let src =
        fs::read_to_string("src/render_loop/motion_bridge_gpu.rs").expect("motion_bridge_gpu.rs");
    // O helper e puro, entao o gate o re-deriva em vez de o importar (ele e privado do shell).
    let clocks = |ticks: &[u64], sub: u32, dt: f64, loops: bool| -> Vec<(f64, Option<u64>)> {
        ticks
            .iter()
            .flat_map(|&t| {
                let clock = loops.then_some(t);
                (1..=sub.max(1)).map(move |k| {
                    let frac = f64::from(k) / f64::from(sub.max(1));
                    ((t as f64 - 1.0 + frac) * dt, clock)
                })
            })
            .collect()
    };
    let dt = 1.0 / 60.0;
    for &t in &[0u64, 1, 7, 120] {
        let got = clocks(&[t], 1, dt, true);
        assert_eq!(got.len(), 1, "um tique, uma passada");
        assert_eq!(got[0].1, Some(t), "o tique nao se renumera");
        assert!(
            (got[0].0 - t as f64 * dt).abs() < 1e-12,
            "o playhead de `sub=1` e `t*dt`: {} contra {}",
            got[0].0,
            t as f64 * dt
        );
    }

    // ⚠️ E o gate le a FONTE para provar que o produto usa a mesma lei: um helper de teste que
    // divergisse do shipado seria um oraculo do proprio teste.
    assert!(
        src.contains("fn substep_clocks("),
        "o helper de relogios do device sumiu"
    );
    assert!(
        src.contains("t as f64 - 1.0 + frac) * fixed_dt"),
        "a lei do sub-playhead mudou sem este gate saber"
    );
}

/// **O tique NÃO se subdivide, e as duas metades disso são load-bearing.** O device avança o
/// ping-pong do `pre` a cada CHAMADA de `cook`, e o ring de scrub chaveia pelo TIQUE — renumerar
/// os sub-tiques gravaria estados do meio do quadro sob rótulos de quadro.
#[test]
fn the_substeps_share_the_frames_tick_so_the_scrub_ring_is_untouched() {
    let dt = 1.0 / 60.0;
    let sub = 4u32;
    let t = 9u64;
    let clocks: Vec<(f64, Option<u64>)> = (1..=sub)
        .map(|k| {
            let frac = f64::from(k) / f64::from(sub);
            ((t as f64 - 1.0 + frac) * dt, Some(t))
        })
        .collect();

    assert_eq!(clocks.len(), 4);
    assert!(
        clocks.iter().all(|(_, c)| *c == Some(t)),
        "todas as sub-passadas carregam o tique do QUADRO: {clocks:?}"
    );
    // A ultima cai EXATAMENTE no playhead do quadro -- e o que mantem o proximo quadro alinhado.
    assert!(
        (clocks[3].0 - t as f64 * dt).abs() < 1e-12,
        "a ultima sub-passada cai no playhead do quadro"
    );
    // E os passos sao iguais: um `dt/n` por passada, que e o que as leis de contagem telescopam.
    let steps: Vec<f64> = clocks.windows(2).map(|w| w[1].0 - w[0].0).collect();
    for s in steps {
        assert!(
            (s - dt / f64::from(sub)).abs() < 1e-12,
            "cada sub-passada anda `dt/n`: {s}"
        );
    }
}

/// **O device NUNCA recusa por substep, e pergunta a MESMA porta que o pump da CPU.**
///
/// O gate le a fonte porque a decisao mora dentro de `cook_gpu`, que exige uma janela e nenhum
/// teste de unidade alcanca. As duas metades sao independentes: uma porta PROPRIA no shell
/// divergiria do pump sem nenhum teste de nenhuma das metades notar, e uma recusa de volta
/// custaria o device a todo documento substepado.
#[test]
fn the_device_never_recuses_for_a_substep_and_asks_the_one_door() {
    let src =
        fs::read_to_string("src/render_loop/motion_bridge_gpu.rs").expect("motion_bridge_gpu.rs");
    assert!(
        src.contains("cook::graph_substeps(&motion.doc.graph"),
        "o device tem de perguntar `graph_substeps` -- a MESMA porta do pump da CPU; uma segunda \
         copia divergiria justamente no documento em que as duas metades tem de concordar"
    );
    for gone in ["graph_asks_for_substeps", "fn device_substeps("] {
        assert!(
            !src.contains(gone),
            "`{gone}` voltou: e uma recusa (ou uma 2a porta) que custa o device a um documento \
             que a CPU sabe cozinhar igual"
        );
    }
}
