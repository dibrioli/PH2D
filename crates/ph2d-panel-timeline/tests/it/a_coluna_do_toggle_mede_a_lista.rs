//! ⛔⛔⛔ **A BARRA DE TRANSPORTE PINTAVA `PingPon…` NUM TOGGLE, E TERIA CORTADO SEIS NA PRIMEIRA
//! TRADUÇÃO.**
//!
//! Achado pela varredura das elisões (2026-09-18) e pela decisão de grafia do dono (2026-09-19:
//! *«Ping-Pong»*, unificando as duas grafias que o app tinha para a mesma coisa).
//!
//! A coluna de rótulo dos dez toggles era o literal `52,0 px`, escolhido pela palavra `AutoKey`
//! (`48,44`). Medido:
//!
//! | rótulo | inglês | idioma de teste |
//! |---|---:|---:|
//! | `Ping-Pong` | **60,45** | **91,34** |
//! | `AutoKey` | 48,44 | 71,37 |
//! | `Physics` | 44,51 | 67,97 |
//! | `Record` | 40,66 | 63,70 |
//! | `Speed` | 36,69 | 56,09 |
//! | `Onion` | 33,91 | 53,67 |
//! | `Snap` | 29,20 | 45,33 |
//! | `Loop` | 28,70 | 44,82 |
//! | `Keys` | 27,96 | 44,09 |
//! | `Path` | 25,78 | 42,02 |
//!
//! ⇒ a coluna passa a ser **medida da LISTA** (`transport::toggle_label_w`), e o pintor recebe o
//! MESMO número pelo mesmo argumento.
//!
//! # ⚠️ Porque este gate pinta o painel de verdade
//!
//! A régua da porta vive na crate do pintor (`ph2d-editor-core`,
//! `uma_coluna_de_rotulo_cabe_a_familia_inteira`) e afirma que a lei está certa. **Ela não afirma
//! que este painel a usa** — um painel que voltasse ao literal deixaria aquela verde. Por isso o
//! sujeito aqui é o CENSO DE ELISÕES do `paint` real: *o orçamento com que cada rótulo foi de
//! facto pintado*.

use ph2d_editor_core::text_elide::elisao::Medido;
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::{Idioma, pseudo, tr_em};
use ph2d_panel_timeline::TimelinePanel;
use ph2d_panel_timeline::state::{TimelinePanelState, set_current_timeline};
use ph2d_text::TextSystem;
use ph2d_timeline::TimelineViewSnapshot;
use ph2d_tokens::TypeToken;
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect::new(0.0, 0.0, 1600.0, 900.0);

/// Os dez rótulos da família, **em inglês**, na ordem em que a barra os dispõe.
///
/// ⚠️ Escrita à mão de propósito: é o CONTROLO da tabela que o produto lê
/// (`transport::toggle_key`). Uma lista derivada da mesma tabela concordaria com ela por
/// construção e não afirmaria nada — *um censo que se deriva do sujeito mede-se a si próprio*.
const ROTULOS: &[&str] = &[
    "Loop",
    "Ping-Pong",
    "Physics",
    "AutoKey",
    "Record",
    "Path",
    "Snap",
    "Speed",
    "Onion",
    "Keys",
];

/// Pinta a barra com o censo armado e devolve **só** os registos dos rótulos de toggle.
///
/// ⚠️ O filtro é por TEXTO e não por posição: a palavra `Keys` também é uma ABA da barra, pintada
/// noutra fonte (`13 px`) e com outro orçamento. Filtrar por texto sozinho misturaria as duas, e
/// o gate leria a coluna das abas como sendo a dos toggles ⇒ o filtro casa texto **e** fonte.
fn toggles_pintados() -> Vec<Medido> {
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut state = TimelinePanelState::default();
    set_current_timeline(Some(TimelineViewSnapshot {
        fps: 60.0,
        ..TimelineViewSnapshot::default()
    }));
    let fonte = TypeToken::Sm.px();
    host.medindo_a_pintura::<TimelinePanel>(&mut state, VIEWPORT)
        .into_iter()
        .filter(|m| m.fonte == fonte && ROTULOS.contains(&m.texto.as_str()))
        .collect()
}

/// A coluna com que a barra de facto pintou — e a prova de que ela é **UMA**.
fn coluna_pintada(medidos: &[Medido]) -> f32 {
    let primeira = medidos[0].largura;
    for m in medidos {
        assert_eq!(
            m.largura, primeira,
            "⛔ {:?} foi pintado com um orçamento ({}) diferente do dos irmãos ({primeira}) — os \
             dez toggles partilham UMA coluna, e dois números aqui querem dizer que alguém voltou \
             a medir por célula",
            m.texto, m.largura
        );
    }
    primeira
}

/// ⭐⭐⭐ **Os dez rótulos chegam à tela INTEIROS, e a coluna é uma só.**
#[test]
fn nenhum_toggle_da_barra_pinta_um_rotulo_cortado() {
    let medidos = toggles_pintados();
    let vistos: Vec<&str> = medidos.iter().map(|m| m.texto.as_str()).collect();
    for r in ROTULOS {
        assert!(
            vistos.contains(r),
            "⛔ o rótulo {r:?} não foi pintado — ou a barra deixou de o mostrar, ou a tabela \
             `toggle_key` esqueceu-o e ele saiu SEM rótulo nenhum. Vistos: {vistos:?}"
        );
    }
    let col = coluna_pintada(&medidos);
    for m in &medidos {
        assert!(
            m.coube(),
            "⛔ {:?} saiu {:?} numa coluna de {col} px",
            m.texto,
            m.pintado
        );
    }
}

/// ⛔ **A coluna é TIGHT — ela é a palavra mais larga, não um número com folga.**
///
/// Sem esta metade, repor um literal generoso (digamos `120 px`) passaria o gate acima: nada
/// seria cortado e a barra gastaria meia linha por célula. *Uma folga escondida é onde o próximo
/// rótulo cabe por sorte e o seguinte não* — e ninguém saberia qual dos dois casos tem em mãos.
#[test]
fn a_coluna_da_barra_e_exactamente_a_do_rotulo_mais_largo() {
    let col = coluna_pintada(&toggles_pintados());
    let mut ts = TextSystem::without_system_fonts();
    let mais_largo = ph2d_editor_core::paint::label_column_width(
        &mut ts,
        TypeToken::Sm.px(),
        ROTULOS.iter().copied(),
    );
    assert!(
        (col - mais_largo).abs() < 0.01,
        "a barra reservou {col} px e o rótulo mais largo da lista mede {mais_largo}"
    );
}

/// ⭐⭐⭐ **E NA PRIMEIRA TRADUÇÃO TAMBÉM — porque a coluna deixou de ser um número.**
///
/// ⚠️ **A pintura em INGLÊS responde pelas duas línguas:** o idioma de teste é uma função pura do
/// inglês (`ph2d_i18n::pseudo`), logo cada rótulo re-deforma-se e a coluna re-mede-se pela MESMA
/// porta, sem tocar no `PH2D_LANG` do processo (que é um `OnceLock` — mexer nele faria disto mais
/// um membro da família de flakes de fan-out).
///
/// ⛔ **E o CONTROLO é a metade que dá valor a isto:** a coluna INGLESA **não** chega para os
/// rótulos deformados. Sem ele, este teste passaria num mundo onde o idioma de teste não deforma
/// nada, e seria uma frase bonita sobre o vazio.
///
/// ⚠️ **Medido em 2026-09-19:** contra a coluna inglesa de hoje (`60,45`) estouram **4** dos dez
/// (`Ping-Pong 91,34` · `AutoKey 71,37` · `Physics 67,97` · `Record 63,70`); contra o literal de
/// ontem (`52,0`) estouravam **seis**. *A barra é o número medido, não um `>= 1` que passaria com
/// a deformação quase apagada.*
#[test]
fn nenhum_toggle_seria_cortado_na_proxima_lingua() {
    let col_ingles = coluna_pintada(&toggles_pintados());
    let fonte = TypeToken::Sm.px();
    let mut ts = TextSystem::without_system_fonts();

    let deformados: Vec<String> = ROTULOS.iter().map(|r| pseudo::deforma(r)).collect();
    let col_teste = ph2d_editor_core::paint::label_column_width(
        &mut ts,
        fonte,
        deformados.iter().map(String::as_str),
    );

    let mut estouram_o_ingles = 0;
    for (r, d) in ROTULOS.iter().zip(&deformados) {
        let w = ts.prefix_width_weighted(d, fonte, ph2d_text::FontWeight::MEDIUM);
        assert!(
            w <= col_teste,
            "⛔ {r:?} deformado ({d:?}, {w} px) não cabe na coluna que a mesma porta daria \
             naquela língua ({col_teste} px)"
        );
        if w > col_ingles {
            estouram_o_ingles += 1;
        }
    }
    assert!(
        col_teste > col_ingles,
        "⛔ CONTROLO: a coluna do idioma de teste ({col_teste}) tinha de ser MAIOR que a inglesa \
         ({col_ingles}) — se a deformação deixou de alargar as palavras, este gate deixou de \
         medir o que diz medir"
    );
    assert!(
        estouram_o_ingles >= 4,
        "⛔ CONTROLO: só {estouram_o_ingles} rótulos deformados estouram a coluna inglesa \
         ({col_ingles} px), e a medição de 19/09 diz QUATRO — se este número desceu, ou a \
         deformação encolheu ou alguém pôs folga na coluna, e as duas mudam o que este gate afirma"
    );
}

/// ⭐⭐ **A decisão do dono é um GATE: `Ping-Pong`, com hífen, nas DUAS superfícies da timeline.**
///
/// Ele perguntou porque o app tinha duas grafias para uma coisa só — `PingPong` na barra e
/// `Ping-Pong` no menu da mesma timeline (e na tira do Flip, e na direcção de animação do
/// Inspector). ⚠️ *Uma decisão de produto que fica só na mensagem de commit volta na primeira vez
/// que alguém escrever o rótulo de memória.*
#[test]
fn a_barra_e_o_menu_da_timeline_escrevem_a_mesma_palavra() {
    let barra = tr_em(Idioma::Ingles, "panel.timeline.ping_pong");
    let menu = tr_em(Idioma::Ingles, "chrome.timeline_menu.ping_pong");
    assert_eq!(
        barra, menu,
        "⛔ a barra diz {barra:?} e o menu da mesma timeline diz {menu:?}"
    );
    assert_eq!(barra, "Ping-Pong", "a grafia que o Enio escolheu em 19/09");
}
