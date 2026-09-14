//! ⭐⭐⭐ **TODO CARTÃO QUE UM ANÚNCIO MANDA CLICAR EXISTE NA CENA QUE ELE ANUNCIA.**
//!
//! ⛔⛔ **Um passo que manda clicar num cartão AFIRMA que ele está no grafo** — e a casa já pagou
//! por escrever um passo impossível
//! ([memória](../../project-memory/feedback_a_smoke_step_that_names_a_panel_row_must_prove_the_row_is_in_the_list.md)).
//! O gate irmão (`every_row_the_sim_tutorial_names_is_on_the_card`) defende o **tutorial em PDF**,
//! com uma lista escrita à mão; este defende o **anúncio do terminal**, que é o que o dono lê
//! primeiro, e **deriva a lista do próprio texto**.
//!
//! ⚠️⚠️ **Ele nasceu de dois defeitos reais, os dois meus, os dois na mesma sessão:** as cenas
//! `=116` e `=117` mandavam clicar num cartão `Value LFO` e o cartão chama-se **`LFO`**. *O nome de
//! um nó no código (`value.lfo`) e o nome do cartão na tela não são a mesma palavra, e quem escreve
//! o passo tem o primeiro na cabeça.*

use ph2d_editor_core::ProjectSettings;

/// A FONTE dos anúncios, lida para que a lista do gate e o texto não possam divergir.
const ANUNCIOS: &str = include_str!("motion_state_demo_announce.rs");

/// Os cartões que o bloco de uma cena manda clicar — todo `cartao `X`` do texto dela.
///
/// ⚠️ **O bloco é delimitado pelo marcador `[cena N]`**, que é o que cada anúncio imprime como
/// primeira coisa. *Uma varredura sobre o ficheiro inteiro atribuiria o cartão de uma cena a todas.*
fn cartoes_que_o_anuncio_pede(cena: &str) -> Vec<String> {
    let marca = format!("[cena {cena}]");
    let Some(i) = ANUNCIOS.find(&marca) else {
        return Vec::new();
    };
    let resto = &ANUNCIOS[i + marca.len()..];
    // Até ao princípio do próximo anúncio (ou ao fim do ficheiro).
    let bloco = resto.find("[cena ").map_or(resto, |j| &resto[..j]);
    let mut fora = Vec::new();
    // ⚠️⚠️ **A extracção é pela FORMA IMPERATIVA, e não por «a palavra cartão aparece».** Um
    // anúncio também NEGA cartões — a `=114` diz *«não há cartão `Collide` nenhum na linha da
    // simulação»* e *«DEU ERRADO se … aparecer um cartão `Collide`»* —, e a 1.ª redacção deste
    // gate reprovou sobre essas duas frases, que estão CERTAS. *Uma régua que confunde «clique
    // aqui» com «isto não existe» acusa o texto correcto.*
    //
    // ⛔ **As formas saem de uma varredura do próprio ficheiro** (`grep -o "…cartao \`"`), não de
    // memória, e o piso de população no gate acusa se elas deixarem de casar.
    for pedir in [
        "Clique no cartao `",
        "clique no cartao `",
        "No cartao `",
        "Volte ao cartao `",
        "Com o cartao `",
    ] {
        for pedaco in bloco.split(pedir).skip(1) {
            if let Some(nome) = pedaco.split('`').next() {
                let nome = nome.trim();
                if !nome.is_empty() && !fora.iter().any(|x: &String| x == nome) {
                    fora.push(nome.to_string());
                }
            }
        }
    }
    fora
}

/// ⚠️ **A metade NEGATIVA: os cartões que um anúncio diz NÃO existirem.**
///
/// Ela é uma afirmação tão forte como a outra — a `=114` ensina que *o colisor é da FORMA* dizendo
/// que **não há** um cartão `Collide` na linha da simulação, e no dia em que houver, o passo passa
/// a ensinar o contrário do que se vê.
fn cartoes_que_o_anuncio_nega(cena: &str) -> Vec<String> {
    let marca = format!("[cena {cena}]");
    let Some(i) = ANUNCIOS.find(&marca) else {
        return Vec::new();
    };
    let resto = &ANUNCIOS[i + marca.len()..];
    let bloco = resto.find("[cena ").map_or(resto, |j| &resto[..j]);
    let mut fora = Vec::new();
    for nega in ["nao ha' cartao `", "aparecer um cartao `"] {
        for pedaco in bloco.split(nega).skip(1) {
            if let Some(nome) = pedaco.split('`').next() {
                let nome = nome.trim();
                if !nome.is_empty() && !fora.iter().any(|x: &String| x == nome) {
                    fora.push(nome.to_string());
                }
            }
        }
    }
    fora
}

/// Os títulos dos cartões que a cena `cena` de facto pinta.
fn cartoes_da_cena(cena: &str) -> Vec<String> {
    let mut m = crate::motion_state::MotionState::new();
    let _ = crate::motion_demo_legend::monta(cena, &mut m.doc, &m.registry);
    let mut snap = ph2d_panel_motion_graph::snapshot_from(&m.doc.graph, &m.registry);
    crate::motion_bridge::params::card::stamp_card_params(
        &m,
        ProjectSettings::default(),
        &mut snap,
    );
    snap.nodes.iter().map(|v| v.display_name.clone()).collect()
}

/// As cenas de ciclo — as que ANUNCIAM passos. ⚠️ **Derivada da mesma tabela que o roteador lê**,
/// nunca uma lista escrita aqui: uma cena nova entra neste gate por existir.
fn cenas_de_ciclo() -> Vec<String> {
    (0..1000)
        .map(|n| n.to_string())
        .filter(|n| crate::motion_state::demo_router::is_cycle_scene(n))
        .collect()
}

#[test]
fn every_card_an_announcement_tells_you_to_click_exists_in_that_scene() {
    let cenas = cenas_de_ciclo();
    assert!(
        cenas.len() >= 6,
        "só {} cena(s) de ciclo -- a tabela do roteador partiu-se e este gate varre o vazio",
        cenas.len()
    );
    let mut pedidos = 0usize;
    for cena in &cenas {
        let pede = cartoes_que_o_anuncio_pede(cena);
        if pede.is_empty() {
            continue;
        }
        let tem = cartoes_da_cena(cena);
        for nome in &pede {
            pedidos += 1;
            assert!(
                tem.iter().any(|t| t == nome),
                "o anuncio da cena `={cena}` manda clicar no cartao `{nome}` e a cena nao tem \
                 nenhum com esse titulo -- ela tem {tem:?}"
            );
        }
        // ⚠️ E a metade NEGATIVA, que é uma afirmação igualmente forte.
        for nome in cartoes_que_o_anuncio_nega(cena) {
            pedidos += 1;
            assert!(
                !tem.contains(&nome),
                "o anuncio da cena `={cena}` ensina que NAO ha' cartao `{nome}`, e a cena tem um \
                 -- o passo passou a ensinar o contrario do que se ve': {tem:?}"
            );
        }
    }
    // ⚠️ **O piso outra vez:** `0 pedidos` lê-se igual a `0 defeitos`, e é o que um marcador
    // `[cena N]` renomeado produziria.
    assert!(
        pedidos >= 8,
        "só {pedidos} cartão(ões) pedido(s) em {} cena(s) -- a extracção do texto partiu-se",
        cenas.len()
    );
}
