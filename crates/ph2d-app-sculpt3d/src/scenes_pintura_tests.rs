//! **OS GATES DA CENA DA PINTURA** (`=51`) — irmão (`#[path]`) da [`super`].

use super::*;

/// ⭐⭐ **ELA ABRE COM O PINCEL DE PINTURA NA MÃO — as DUAS metades.**
///
/// O roteiro diz *«o pincel de pintura já está na sua mão»*, e essa frase é uma
/// AFIRMAÇÃO sobre o arranque. Ela parte-se em duas porque cada metade falha por
/// um motivo diferente:
///
/// 1. **a LEI** — com a cena armada, o verbo escolhido é o da pintura;
/// 2. **o FIO** — o prólogo da crate chama esta cena.
///
/// ⛔ **Sem a segunda, um gate que chama a função em vez de percorrer a rota
/// afirma que a lei existe e nunca que a cena a usa** (§24). E ela é lida por
/// [`include_str!`], que **deixa de compilar** se o irmão mudar de ficheiro —
/// em vez de ficar verde a medir menos.
#[test]
fn a_cena_da_pintura_abre_com_o_pincel_de_pintura() {
    // A LEI, pura: a cena escolhe a pintura, e só ela.
    assert_eq!(
        verbo_da_cena(true),
        Some(Verb::Paint),
        "a =51 não arma o pincel de pintura, e o roteiro promete-o na 1.ª linha"
    );
    assert_eq!(
        verbo_da_cena(false),
        None,
        "o prólogo desta cena mexe no pincel de OUTRA cena — um verbo trocado \
         debaixo de um roteiro que não o menciona"
    );
    // O FIO, em duas pontas: o prólogo chama esta cena, e ela pergunta pela env.
    const ROTEADOR: &str = include_str!("scenes.rs");
    const ESTA: &str = include_str!("scenes_pintura.rs");
    assert!(
        ROTEADOR.contains("pintura::arma(cena);"),
        "o prólogo da crate não chama a =51: a lei existe e ninguém a corre"
    );
    assert!(
        ESTA.contains("verbo_da_cena(pintura_scene())"),
        "o `arma` deixou de perguntar pela cena: ele passaria a armar a pintura \
         em TODA cena do módulo"
    );
}

/// ⭐⭐⭐ **O ROTEIRO NOMEIA CONTROLOS QUE EXISTEM, E NO NÍVEL QUE ELE PROMETE.**
///
/// ⛔ *Um passo que manda clicar numa linha AFIRMA que ela está na tela*, e o
/// dono aprova o smoke com o passo impossível dentro — foi o que aconteceu com o
/// `Auto-Smooth` da `=41`, que é `Pro` num painel que nasce `Basic`.
///
/// ⚠️ **As três pistas de cor só existem com o pincel de pintura em mãos**, e é
/// por isso que a régua as pergunta com ele armado: perguntá-las com o pincel de
/// fábrica leria *«não existem»* sobre um painel correcto.
#[test]
fn o_roteiro_nomeia_as_pistas_de_cor_que_o_painel_pinta() {
    use ph2d_panel_sculpt3d::rows::rows;
    use ph2d_panel_sculpt3d::state::{Sculpt3dUi, UiLevel};

    let mut ui = Sculpt3dUi {
        ui_level: UiLevel::Basic,
        ..Sculpt3dUi::default()
    };
    ui.brush.verb = Verb::Paint;
    for label in [
        "panel.sculpt3d.color_r",
        "panel.sculpt3d.color_g",
        "panel.sculpt3d.color_b",
    ] {
        let row = rows()
            .find(|r| r.label == label)
            .unwrap_or_else(|| panic!("o roteiro nomeia `{label}` e a tabela não o tem"));
        assert!(
            row.visible(&ui),
            "o roteiro manda escolher a cor no nível de fábrica e `{label}` não é \
             pintada ali — o passo (1) é impossível"
        );
    }
    // ⛔ **O CONTROLO, e sem ele o gate passa com três pistas pintadas SEMPRE:**
    // com um pincel que não deposita a cor do pincel, elas têm de sumir.
    ui.brush.verb = Verb::Blur;
    assert!(
        !rows()
            .filter(|r| r.label.starts_with("panel.sculpt3d.color_"))
            .any(|r| r.visible(&ui)),
        "as pistas de cor são pintadas com o `Blur` em mãos, que NÃO as lê — \
         três knobs mortos, a espécie que o dono reporta como «não vejo efeito»"
    );
}

/// ⚠️ **A peça desta cena é a `96×144`, e o número é o que a torna legível** —
/// ver o cabeçalho do módulo. ⛔ E ela é MUITO mais leve que o default do módulo
/// de propósito: a §24 desta linha registou uma cena que fabricava a peça pesada
/// e fazia o dono reportar o pincel como lento.
#[test]
fn a_peca_da_cena_e_densa_e_muito_mais_leve_que_o_default() {
    let m = peca();
    // ⚠️ **A contagem NÃO é `LATITUDES × LONGITUDES`**, e a diferença é a
    // construção da esfera: os dois pólos são um vértice cada e a costura do
    // meridiano não se repete. *Escrever o produto aqui seria escrever de
    // memória um número que a porta calcula* — o gate lê-o da peça e afirma só
    // a ORDEM DE GRANDEZA, que é o que a cena precisa.
    assert!(
        m.vert_count() > LATITUDES * LONGITUDES * 9 / 10,
        "a peça tem {} vértices contra ~{} da grelha pedida: a porta mudou de \
         construção e a densidade desta cena deixou de ser a medida",
        m.vert_count(),
        LATITUDES * LONGITUDES
    );
    assert!(
        m.vert_count() > 10_000,
        "a peça tem {} vértices: abaixo disto uma marca de tinta lê-se como uma \
         mancha de vértices soltos e o passo (3) do roteiro não tem o que mostrar",
        m.vert_count()
    );
    assert!(
        m.vert_count() < 98_306 / 2,
        "a peça tem {} vértices, metade ou mais do default do módulo — esta cena \
         voltaria a medir o tamanho da peça em vez da ferramenta",
        m.vert_count()
    );
}
