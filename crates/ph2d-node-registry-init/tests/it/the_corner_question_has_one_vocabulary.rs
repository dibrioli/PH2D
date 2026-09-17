//! ⛔ **OS QUATRO CANTOS TÊM UM VOCABULÁRIO SÓ** (ciclo 3, W4b —
//! [doc 106](../../../docs/Motion%20Nodes/106_ciclo_3_transformes_e_deformadores.md)).
//!
//! O `motion.four_point_warp` e o `motion.bezier_warp` deformam **a mesma caixa envolvente**
//! com **os mesmos oito nomes de param** (`tl_dx` … `bl_dy`), e chamavam-lhes coisas
//! diferentes: `TL X` num, `Top-Left X` no outro. É o achado §2.3 daquele ciclo — *seis
//! vocabulários para «onde é o centro»* — repetido um nível abaixo, e com o painel lateral fora
//! o cartão é a única superfície onde o artista lê estes nomes.
//!
//! ⚠️ **A população é DERIVADA do manifesto**, nunca uma lista de dois nós: qualquer nó que
//! venha a declarar `tl_dx` entra sozinho neste censo, que é precisamente o que uma enumeração
//! não faz.
//!
//! ⚠️ **A forma longa foi escolhida por MEDIÇÃO** — o gate `no_warp_label_is_cut_on_the_card`
//! (na crate do painel) pinta a row e conta os glifos: `Bottom-Right X` cabe inteiro nos
//! `190 px` do cartão ao lado do valor mais largo da faixa. Se não coubesse, a unificação teria
//! ido para o lado curto.

use ph2d_node_registry::NodeRegistry;

/// Os oito params dos cantos, e o rótulo que cada um tem de ter.
const CANTOS: [(&str, &str); 8] = [
    ("tl_dx", "Top-Left X"),
    ("tl_dy", "Top-Left Y"),
    ("tr_dx", "Top-Right X"),
    ("tr_dy", "Top-Right Y"),
    ("br_dx", "Bottom-Right X"),
    ("br_dy", "Bottom-Right Y"),
    ("bl_dx", "Bottom-Left X"),
    ("bl_dy", "Bottom-Left Y"),
];

#[test]
fn every_node_that_offsets_a_corner_calls_it_the_same_thing() {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("registry builds");

    let mut vistos = 0usize;
    let mut queixas: Vec<String> = Vec::new();
    for manifest in reg.manifests() {
        // ⚠️ A pergunta é feita ao MANIFESTO (quem declara o param), não aos hints — um nó que
        // declarasse o param e esquecesse o hint tem de ser acusado, não saltado.
        if manifest.param_default("tl_dx").is_none() {
            continue;
        }
        vistos += 1;
        let hints = reg.param_ui(manifest.id).unwrap_or(&[]);
        for (param, esperado) in CANTOS {
            let Some(h) = hints.iter().find(|h| h.param == param) else {
                queixas.push(format!("{}: `{param}` não tem hint nenhum", manifest.name));
                continue;
            };
            if ph2d_i18n::tr(h.label) != esperado {
                queixas.push(format!(
                    "{}: `{param}` diz `{}` e o vocabulário do grupo é `{esperado}`",
                    manifest.name,
                    ph2d_i18n::tr(h.label)
                ));
            }
        }
    }

    // Controle: um censo que não achou nenhum nó passaria vazio para sempre.
    assert!(
        vistos >= 2,
        "só {vistos} nó(s) declaram `tl_dx` — a pergunta precisa de pelo menos dois para haver \
         vocabulário a divergir (medido em 2026-09-08: o `four_point_warp` e o `bezier_warp`)"
    );
    assert!(
        queixas.is_empty(),
        "os cantos são a MESMA pergunta e têm de ter as mesmas palavras:\n  {}",
        queixas.join("\n  ")
    );
}
