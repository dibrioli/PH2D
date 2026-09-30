//! ⭐⭐⭐ **Todo rótulo que o tutorial da VIDA cita existe no painel** (plano 28, W7).
//!
//! O irmão do gate do tutorial da máquina de estados, e pela mesma razão: um passo que diz
//! *«troque o `Rate` para 1»* é uma **afirmação de que esse texto está na tela**, escrita num
//! ficheiro e decidida noutro. ⚠️ A leitura das tabelas e da marca é a do irmão
//! (`chaves_e_textos_de` · `rotulos_citados_de` · `desescapa`) — uma porta, dois gates.
//!
//! ⚠️ **As populações são DESTE tutorial:** as quatro secções da família da vida (e os irmãos que
//! o tecto de LOC lhes partiu), o editor da tabela de acções (a secção 8 cita `From Myself`), o
//! cabeçalho (`Add Component`) — e os VERBOS, cujo rótulo é resolvido pela chave que o
//! `SignalVerb::label_key` do `ph2d-ecs` devolve, logo esse ficheiro é o «pintor» deles.
//!
//! ⛔ Como o irmão, ele **não** mede que o rótulo chega a pixel.

use super::o_tutorial_nomeia_rotulos_que_existem::{
    chaves_e_textos_de, desescapa, rotulos_citados_de, sem_prosa,
};

/// A fonte do tutorial, tal como o gerador de PDF a lê.
const TUTORIAL: &str =
    include_str!("../../../../docs/Components/tutoriais/src/02_vida_e_dano.html");

/// Os pintores que produzem os rótulos citados.
const PINTORES: [&str; 9] = [
    include_str!("../../src/sections/vida.rs"),
    include_str!("../../src/sections/vida_dano.rs"),
    include_str!("../../src/sections/vida_resist.rs"),
    include_str!("../../src/sections/vida_barra.rs"),
    include_str!("../../src/sections/vida_impacto.rs"),
    include_str!("../../src/sections/actions_editor.rs"),
    include_str!("../../src/paint_head.rs"),
    include_str!("../../src/paint_cards.rs"),
    // ⚠️ **O «pintor» dos VERBOS** — o painel recebe o rótulo já resolvido pela chave que esta
    //    porta devolve (`SignalVerb::label_key`); sem ela `Start Timer` e `Restart Run` liam-se
    //    como órfãos sobre um produto CERTO.
    include_str!("../../../ph2d-ecs/src/signal_actions.rs"),
];

/// As tabelas onde as frases vivem desde a migração do HR-15.
const TABELAS: [&str; 4] = [
    include_str!("../../../ph2d-i18n/src/inspector_vida.rs"),
    include_str!("../../../ph2d-i18n/src/inspector.rs"),
    include_str!("../../../ph2d-i18n/src/inspector_game.rs"),
    include_str!("../../../ph2d-i18n/src/ecs_scene.rs"),
];

/// ⭐⭐⭐ **Cada rótulo citado existe num pintor** — escrito lá, ou texto de uma chave que um pintor
/// usa.
///
/// **Mutação que deve sangrar:** mudar `"Lasts"` para `"Duration"` na tabela da vida — o tutorial
/// passa a ensinar um campo que não existe, e isso reprova.
#[test]
fn o_tutorial_da_vida_so_cita_rotulos_que_o_painel_pinta() {
    // ⚠️ SEM a prosa: um comentário que nomeia um rótulo não o pinta.
    let pintores: Vec<String> = PINTORES.iter().map(|s| desescapa(&sem_prosa(s))).collect();
    let citados = rotulos_citados_de(TUTORIAL);
    // ⚠️ PISO DE POPULAÇÃO — uma marca renomeada faria o gate varrer ZERO e ficar verde.
    assert!(
        citados.len() >= 30,
        "o tutorial da vida deveria citar pelo menos 30 rotulos de tela; achei {}",
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
        "o tutorial da vida cita rotulos que nenhum pintor produz: {orfaos:?}"
    );
}

/// ⚠️ **A metade JUSTA** — um `contains` sobre uma string vazia passaria sempre.
#[test]
fn nenhum_rotulo_citado_no_tutorial_da_vida_e_vazio() {
    for r in rotulos_citados_de(TUTORIAL) {
        assert!(
            !r.trim().is_empty(),
            "um <code class=\"ui\"> vazio no tutorial da vida: ele afirma que a tela mostra NADA"
        );
    }
}
