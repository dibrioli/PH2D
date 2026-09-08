//! ⭐⭐⭐ **«EM TORNO DE QUÊ» É UMA PERGUNTA SÓ, E ELA TEM UM VOCABULÁRIO SÓ** (ciclo 3, W1 —
//! [doc 106 §2.3](../../../docs/Motion%20Nodes/106_ciclo_3_transformes_e_deformadores.md)).
//!
//! A auditoria do grupo dos transformes e deformadores mediu **seis** respostas diferentes à
//! mesma pergunta, e o que as tornava seis não era desenho: era não haver quem contasse. Este
//! censo é esse alguém.
//!
//! ⚠️ **Ele varre o CATÁLOGO INTEIRO**, não uma lista escrita à mão: um nó que nasça amanhã com
//! um `pivot_mode` entra sozinho, que é precisamente o que uma enumeração não faz.

use ph2d_node_registry::{NodeRegistry, ParamWidget};
use ph2d_nodegraph::gpu::KernelResolver;

/// Todo nó que declara o param do pivô pinta-o como um `Enum` com **os rótulos da porta, na
/// ordem da porta**, e chama-lhe **a mesma palavra**.
///
/// ⚠️ **A ordem é contrato porque o VALOR vai no ficheiro:** trocar duas entradas muda o
/// significado de toda cena já gravada. E o rótulo é contrato porque um artista que aprendeu o
/// `Pivot` do `Twist` tem de reconhecer o do `Transform` — foi ele que esteve dois anos a
/// chamar-se *«Scale About»* enquanto os irmãos diziam *«Pivot X»*.
///
/// ⚠️ **E o par de coordenadas tem de estar GATEADO ao modo que o lê.** Sem isso o cartão pinta
/// dois números que não fazem nada, que é a espécie do controlo morto — e, pior, o valor deles
/// **sobrevive** à troca de modo, que foi como um `pivot_x` esquecido chegou a vazar para o
/// dispositivo e a desenhar `1,369` unidades de mundo ao lado da CPU.
#[test]
fn every_node_that_asks_where_the_centre_is_asks_it_with_the_same_words() {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("registry builds");

    let mut vistos: Vec<&str> = Vec::new();
    for manifest in reg.manifests() {
        if !manifest
            .params
            .iter()
            .any(|p| p.name == ph2d_nodegraph::pivot::PARAM)
        {
            continue;
        }
        vistos.push(manifest.name);
        let hints = reg.param_ui(manifest.id).unwrap_or(&[]);
        let hint = hints
            .iter()
            .find(|h| h.param == ph2d_nodegraph::pivot::PARAM)
            .unwrap_or_else(|| {
                panic!(
                    "{}: declara `{}` e nao o pinta — um modo inalcancavel",
                    manifest.name,
                    ph2d_nodegraph::pivot::PARAM
                )
            });
        assert_eq!(
            hint.label,
            "Pivot",
            "{}: a mesma pergunta tem de ter a mesma palavra",
            manifest.name
        );
        match hint.widget {
            ParamWidget::Enum { labels } => assert_eq!(
                labels,
                ph2d_nodegraph::pivot::LABELS,
                "{}: os rotulos (e a ORDEM deles) sao os da porta",
                manifest.name
            ),
            outro => panic!("{}: o pivot_mode e' um Enum, e' {outro:?}", manifest.name),
        }

        // O par de coordenadas so' aparece no modo `Point`.
        let gates = reg.param_gates(manifest.id).unwrap_or(&[]);
        for eixo in ["pivot_x", "pivot_y"] {
            assert!(
                manifest.params.iter().any(|p| p.name == eixo),
                "{}: tem modo e nao tem `{eixo}` — o modo `Point` nao teria o que ler",
                manifest.name
            );
            let g = gates
                .iter()
                .find(|g| g.param == eixo)
                .unwrap_or_else(|| panic!("{}: `{eixo}` sem gate de modo", manifest.name));
            assert_eq!(g.when, ph2d_nodegraph::pivot::PARAM, "{}", manifest.name);
            assert_eq!(g.values, &[1], "{}: so' o modo Point o le'", manifest.name);
        }

        // E o modo `Centroid` tem de ter as duas somas — senao ele existe no painel e o
        // dispositivo nao o sabe calcular.
        let reduces = reg.reduces(manifest.id);
        for nome in ["cx", "cy"] {
            assert!(
                reduces.iter().any(|r| r.name == nome),
                "{}: oferece `Centroid` e nao declara a reducao `{nome}` — o modo existiria no \
                 cartao e o kernel leria um simbolo que nao ha'",
                manifest.name
            );
        }
    }

    // Controlo positivo: a wave adoptou QUATRO nós. Um piso, não um pino — o quinto entra
    // sozinho, e é para isso que o censo existe.
    assert!(
        vistos.len() >= 4,
        "so' {} no(s) declaram o pivo: {vistos:?} — a varredura foi as cegas",
        vistos.len()
    );
}
