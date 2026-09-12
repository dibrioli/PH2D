//! **O LAÇO CORRE HORAS — nada pode acumular** (Enio, 2026-08-21: *"estamos numa game engine
//! onde o loop temporal deve rodar por horas e horas. Logo não podemos ter nada que acumula
//! dados"*).
//!
//! ⚠️ **Este arquivo existe porque uma correção pontual não é uma resposta.** A wave da folha
//! 14 matou o app com um `wgpu OOM` no quadro 19706, eu curei os dois caches culpados — e a
//! pergunta certa não era *"curou aquele?"*, era *"o que mais cresce?"*. Um varrimento de
//! código responde por palpite; isto responde **correndo o quadro** e medindo.
//!
//! # O que ele cobre, e o que NÃO
//!
//! Cobre tudo o que o quadro faz **sem placa de vídeo**: as três membranas
//! ([`super::publish_all`]), a varredura do store e o cook dos sinks. Não
//! cobre os dois assadores de tile (precisam de um adapter) — a disciplina deles é
//! estrutural e gateada onde é pura (`ShapeBake::stale_for_test`), e o `acquire`/`release`
//! está emparelhado no mesmo sítio nos dois.
//!
//! ⚠️ **A cena é escolhida por MUDAR TODO QUADRO.** Uma cena parada não prova nada: os
//! caches são de conteúdo, e conteúdo parado não cresce por construção. A `=76` conduz o
//! `trim_offset` pelo relógio, ou seja cunha uma chave de conteúdo nova a cada quadro — é o
//! pior caso que o catálogo sabe encenar, e foi o que matou o app.

use crate::motion_state::MotionState;

/// Quantos quadros a soak corre. **240 = 4 segundos a 60 fps** — o suficiente para separar
/// *cresce* de *não cresce* (o defeito medido crescia 1 por quadro), e barato o bastante
/// para viver no gate batched em vez de numa sonda que ninguém roda.
const FRAMES: u32 = 240;

/// O que uma medição de quadro devolve — cada campo é um cache que o quadro toca.
#[derive(Debug, PartialEq, Eq)]
struct Census {
    /// Geometrias vivas no `VecPathStore` (o que o OOM encheu, pela via do assador).
    store: usize,
    /// Chaves publicadas no canal externo do cook.
    externals: usize,
    /// ⚠️ **Tabelas vivas no `TableCache`** — acrescentado em 2026-08-30 porque ele é um
    /// acumulador NOVO que este censo não via: o doc dele dizia *"um despejo entraria no dia em
    /// que isto medisse alguma coisa"*, e nada media. *Um cache que nenhum censo conta é um
    /// cache cujo crescimento ninguém pode ver.*
    tables: usize,
}

/// Corre UM quadro na ordem do produto e mede.
///
/// ⚠️ **A ordem é a do `motion_bridge`, e ela é load-bearing:** quem limpa o canal externo é
/// a publicação das formas DESENHADAS (`motion_bridge_shapes::publish`), que corre antes das
/// membranas — e ela corre *"whether or not the Motion tool is active"*. Uma soak que não a
/// replicasse acusaria uma fuga de externals que o produto não tem.
fn frame(state: &mut MotionState, sinks: &[ph2d_nodegraph::graph::NodeId], sec: f64) -> Census {
    state.pump.cook.clear_externals();
    super::publish_all(state, sec);
    for sink in sinks {
        let _ = state
            .pump
            .cook
            .cook(&state.doc.graph, &state.registry, *sink, sec);
    }
    Census {
        store: state.shape_store.len(),
        externals: state.pump.cook.externals().len(),
        tables: state.table_cache.len(),
    }
}

/// **QUATRO SEGUNDOS DE UMA CENA QUE MUDA TODO QUADRO NÃO ACUMULAM NADA.**
///
/// O oráculo é a IGUALDADE entre o censo do 2º quadro e o do último — não um teto. Um teto
/// seria um número escolhido; a igualdade é a lei. (O 1º quadro é a subida legítima: é onde
/// as geometrias da cena nascem.)
#[test]
fn a_scene_that_changes_every_frame_does_not_accumulate() {
    let mut state = MotionState::new();
    let sinks = crate::motion_state::conferencia_demos_style::build_style_demo_document(
        &mut state.doc,
        &state.registry,
    )
    .expect("a cena monta");

    let _warm = frame(&mut state, &sinks, 0.0);
    let second = frame(&mut state, &sinks, 1.0 / 60.0);
    let mut worst = second.store;
    for f in 2..FRAMES {
        let now = frame(&mut state, &sinks, f64::from(f) / 60.0);
        worst = worst.max(now.store);
        assert_eq!(
            now, second,
            "o quadro {f} nao mede o mesmo que o quadro 2 — alguma coisa acumulou"
        );
    }
    // E a metade que impede um verde por acidente: a cena TEM de guardar geometria, senão
    // «não cresceu» seria «não desenhou nada».
    assert!(
        worst >= 6,
        "seis bandas, seis geometrias no minimo — a soak mediu {worst}"
    );
}

/// **E UM CENÁRIO PARADO TAMBÉM NÃO** — o controle que separa *o cache é de conteúdo* de
/// *o cache foi esvaziado*.
///
/// ⚠️ Sem esta metade, um `sweep` que apagasse tudo todo quadro passaria no gate acima (o
/// censo seria constante em zero) e o produto reconstruiria cada forma 60 vezes por segundo.
#[test]
fn a_still_scene_keeps_its_geometry_across_frames() {
    let mut state = MotionState::new();
    let n = state.doc.graph.add_node("source.shape");
    let out = state.doc.graph.add_node("motion.output");
    let _ = state.doc.graph.connect(ph2d_nodegraph::graph::Edge {
        from: (n, 0),
        to: (out, 0),
        delayed: false,
    });
    let first = frame(&mut state, &[out], 0.0);
    assert_eq!(first.store, 1, "a forma parada mora no store");
    for f in 1..FRAMES {
        assert_eq!(
            frame(&mut state, &[out], f64::from(f) / 60.0),
            first,
            "a forma parada tem de SOBREVIVER ao quadro {f}, nao ser reconstruida"
        );
    }
}
