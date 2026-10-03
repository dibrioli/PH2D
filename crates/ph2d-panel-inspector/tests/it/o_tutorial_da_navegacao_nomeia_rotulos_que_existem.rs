//! ⭐⭐⭐ **Todo rótulo que o tutorial da NAVEGAÇÃO cita existe no painel** (plano 30, W8).
//!
//! O terceiro irmão do gate do tutorial da máquina de estados, pela mesma razão: um passo que diz
//! *«tire o visto de `Avoid Harm`»* é uma **afirmação de que esse texto está na tela**, escrita num
//! ficheiro e decidida noutro. A leitura das tabelas e da marca é a do irmão — uma porta, três gates.
//!
//! ⚠️ **As populações são DESTE tutorial:** as secções da navegação (`Nav Region` · `Nav Agent` ·
//! `Nav Cost Area` · `Nav Link`), a do `Top-Down Player` (o tutorial cita `Default Controls`), a da
//! vida (`Health`), o cabeçalho (`Add Component`) — e os VERBOS (`Start Navigation`), cujo rótulo
//! resolve a chave que o `SignalVerb::label_key` do `ph2d-ecs` devolve.
//!
//! ⛔ Como os irmãos, ele **não** mede que o rótulo chega a pixel.

use super::o_tutorial_nomeia_rotulos_que_existem::{
    chaves_e_textos_de, pintores_de, rotulos_citados_de,
};

/// A fonte do tutorial, tal como o gerador de PDF a lê.
const TUTORIAL: &str =
    include_str!("../../../../docs/Components/tutoriais/src/03_navegacao.html");

/// Os pintores que produzem os rótulos citados.
const PINTORES: [&str; 8] = [
    include_str!("../../src/sections/nav.rs"),
    include_str!("../../src/sections/nav_custo.rs"),
    include_str!("../../src/sections/nav_tag_row.rs"),
    include_str!("../../src/sections/topdown.rs"),
    include_str!("../../src/sections/vida.rs"),
    include_str!("../../src/paint_head.rs"),
    include_str!("../../src/paint_cards.rs"),
    include_str!("../../../ph2d-ecs/src/signal_actions.rs"),
];

/// As tabelas onde as frases vivem.
const TABELAS: [&str; 5] = [
    include_str!("../../../ph2d-i18n/src/inspector_nav.rs"),
    include_str!("../../../ph2d-i18n/src/inspector.rs"),
    include_str!("../../../ph2d-i18n/src/inspector_game.rs"),
    include_str!("../../../ph2d-i18n/src/inspector_vida.rs"),
    include_str!("../../../ph2d-i18n/src/ecs_scene.rs"),
];

/// ⭐⭐⭐ **Cada rótulo citado existe num pintor** — escrito lá, ou texto de uma chave que um pintor
/// usa.
///
/// **Mutação que deve sangrar:** mudar `"Avoid Harm"` para `"Avoid Danger"` na tabela da
/// navegação — o tutorial passa a ensinar uma caixa que não existe, e isso reprova.
#[test]
fn o_tutorial_da_navegacao_so_cita_rotulos_que_o_painel_pinta() {
    // ⚠️ SEM a prosa: um comentário que nomeia um rótulo não o pinta.
    let pintores = pintores_de(&PINTORES);
    let citados = rotulos_citados_de(TUTORIAL);
    // ⚠️ PISO DE POPULAÇÃO — uma marca renomeada faria o gate varrer ZERO e ficar verde.
    assert!(
        citados.len() >= 30,
        "o tutorial da navegacao deveria citar pelo menos 30 rotulos de tela; achei {}",
        citados.len()
    );
    let tabela = chaves_e_textos_de(&TABELAS);
    // ⚠️ PISO DA TABELA — um parser que deixe de casar devolve VAZIO, em silêncio.
    assert!(
        tabela.len() >= 200,
        "li {} pares chave/texto nas tabelas — o formato do `match` mudou?",
        tabela.len()
    );
    let vivo = |r: &String| {
        pintores.iter().any(|p| p.contains(r.as_str()))
            || tabela.iter().any(|(k, t)| {
                t.contains(r.as_str()) && pintores.iter().any(|p| p.contains(k.as_str()))
            })
    };
    let orfaos: Vec<&String> = citados.iter().filter(|r| !vivo(r)).collect();
    assert!(
        orfaos.is_empty(),
        "o tutorial da navegacao cita rotulos que nenhum pintor produz: {orfaos:?}"
    );
}

/// ⚠️ **A metade JUSTA** — um `contains` sobre uma string vazia passaria sempre.
#[test]
fn nenhum_rotulo_citado_no_tutorial_da_navegacao_e_vazio() {
    for r in rotulos_citados_de(TUTORIAL) {
        assert!(
            !r.trim().is_empty(),
            "um <code class=\"ui\"> vazio no tutorial da navegacao: ele afirma que a tela mostra NADA"
        );
    }
}
