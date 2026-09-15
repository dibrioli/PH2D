//! ⭐⭐⭐ **UM RÓTULO QUE NÃO CABE NA COLUNA DELE É UM RÓTULO CORTADO.**
//!
//! ⛔⛔ **Report do dono, 2026-09-14:** *«Label acima do campo numérico! Muito ruim!»*, e a cura
//! (o rótulo à esquerda, numa coluna que é uma FRACÇÃO da linha) trouxe a pergunta seguinte: *cabe?*
//!
//! Medido nesse dia com o sistema de texto REAL, à largura de omissão do Inspector:
//!
//! | | rótulos cortados |
//! |---|---|
//! | com a unidade no rótulo (`"Float Height (m)"`) | **20 de 39** |
//! | com a unidade no CAMPO (`"Float Height"` + chip `m`) | **1** |
//!
//! O `"Float Height (m)"` mede `92,1 px` numa coluna de `91,2` — ele perdia o `(m)` **e** o `t` do
//! *Height*. ⇒ a unidade mudou-se para dentro do campo
//! ([`sections::rows::num_row_unit`](../../src/sections/rows.rs)), que é onde ela é lida: ao lado
//! do valor.
//!
//! # ⚠️ A coluna é DERIVADA, não escrita
//!
//! `inspector-w` − 2×`panel-head-pad` − 2×`Spacing::Sm` (o recuo do cartão) é a largura real de uma
//! linha, e a coluna sai da porta [`ph2d_editor_core::widget::property_label_col_w`]. ⛔ Um número
//! escrito aqui mediria uma coluna que o produto já não tem.
//!
//! # ⛔ Por que a barra é a coluna e não «a coluna mais folga»
//!
//! A elisão é **correcta** — a coluna docada é arrastável, e um rótulo tem sempre de poder cortar.
//! O que este gate afirma é outra coisa: *à largura de OMISSÃO, o artista não devia ver nenhum*.

use ph2d_text::TextSystem;
use ph2d_tokens::{Spacing, TypeToken};

/// ⏳ **QUANTOS RÓTULOS ELIDEM, POR LARGURA DE PAINEL — e só ENCOLHE.**
///
/// ⛔⛔⛔ **A 1.ª redacção deste gate media UMA largura: a de OMISSÃO — e o dono não a usa.**
/// Ele reportou *«3 pontos (…) sendo usados antes de ficar estreito»* com o gate **verde**. A causa
/// estava no `~/.ph2d/layout.txt`: `dock_w_right = 220,9`, contra os `304` do token. A `304` não
/// elide nenhum; a `220,9` elidiam **16 de 52**.
///
/// ⇒ ***um gate calibrado na largura de OMISSÃO é cego à largura que o artista de facto tem***, e
/// a cura não é medir melhor um ponto: é medir a ESCADA.
///
/// ⚠️ **A largura `220,9` não é inventada — é lida do ficheiro de arrumação do dono.** As outras
/// três cercam-na: uma abaixo (o pior caso plausível), uma acima, e a de omissão.
const ELIDEM_POR_LARGURA: &[(f32, usize)] = &[
    (200.0, 13),
    // ⭐ A largura REAL do dock do dono quando ele reportou (workspace `drawing_2d`).
    (220.9, 3),
    (245.0, 0),
    // A largura de OMISSÃO (`inspector-w`), onde a coluna é exactamente a METADE que ele pediu.
    (304.0, 0),
];

/// ⭐⭐⭐ **VAZIO — e foi a decisão de APARÊNCIA do dono que o esvaziou.**
///
/// Esta lista nasceu com **12** entradas, cada uma um nome comprido que era elidido numa coluna de
/// `84,2 px`, e eu devolvi-lhe a escolha: encurtar os nomes, ou dar ao rótulo uma fatia maior da
/// linha. Ele escolheu a segunda por outra razão — *«as caixas numéricas são muito grandes. Maiores
/// que as labels»* — e a coluna passou a `120 px`, onde o mais comprido do app
/// (*«Swim Line (weights)»*, `113,9`) **cabe**.
///
/// ⚠️ ***Uma decisão de aparência do dono resolveu, de graça, o item que eu lhe tinha devolvido como
/// escolha.*** ⛔ E a lista fica aqui, vazia: qualquer rótulo novo que não caiba reprova o gate.
const AINDA_CORTAM: &[&str] = &[];

/// A largura real de uma linha de card do Inspector, à largura de omissão do painel.
fn largura_de_uma_linha() -> f32 {
    let painel = ph2d_tokens::INSPECTOR_W_PX - 2.0 * ph2d_tokens::PANEL_HEAD_PAD_PX;
    // O `card_frame` recua `Spacing::Sm` de cada lado antes de pintar as rows.
    painel - 2.0 * Spacing::Sm.px()
}

/// A coluna que a §14 de facto usa a uma dada largura de painel — **a da SECÇÃO**, medida sobre o
/// rótulo mais largo dela, como o pintor faz.
fn coluna_da_seccao(ts: &mut TextSystem, painel: f32) -> f32 {
    let fonte = TypeToken::Sm.px();
    let mais_largo = ph2d_panel_inspector::player_row_labels()
        .iter()
        .map(|t| ts.prefix_width(t, fonte))
        .fold(0.0_f32, f32::max);
    let linha = painel - 2.0 * ph2d_tokens::PANEL_HEAD_PAD_PX - 2.0 * Spacing::Sm.px();
    ph2d_editor_core::widget::property_label_col_w_for(0.0, linha, Some(mais_largo))
}

/// ⭐⭐⭐ **Quantos rótulos elidem, em cada largura da escada.**
#[test]
fn the_elision_ladder_only_shrinks() {
    let rotulos = ph2d_panel_inspector::player_row_labels();
    assert!(rotulos.len() >= 40, "a tabela encolheu: {}", rotulos.len());
    let fonte = TypeToken::Sm.px();
    let mut ts = TextSystem::new();
    for (painel, tecto) in ELIDEM_POR_LARGURA {
        let col = coluna_da_seccao(&mut ts, *painel);
        let n = rotulos
            .iter()
            .filter(|t| ts.prefix_width(t, fonte) > col)
            .count();
        assert!(
            n <= *tecto,
            "painel {painel}: {n} rotulos elidem e o tecto e' {tecto} — subiu"
        );
        // ⚠️ **A metade de OBSOLESCÊNCIA** (`CLAUDE.md` §5.0): se melhorou, o número desce AQUI.
        assert!(
            n == *tecto,
            "painel {painel}: elidem {n} e a tabela ainda diz {tecto} — aperte o numero"
        );
    }
}

#[test]
fn every_label_this_panel_paints_fits_its_column() {
    let rotulos = ph2d_panel_inspector::player_row_labels();
    // ⚠️ Piso de população: uma lista vazia passa trivialmente.
    assert!(
        rotulos.len() >= 40,
        "a §14 tem {} rotulos — a tabela encolheu?",
        rotulos.len()
    );
    let coluna = ph2d_editor_core::widget::property_label_col_w(0.0, largura_de_uma_linha());
    let fonte = TypeToken::Sm.px();
    let mut ts = TextSystem::new();
    let mut cortados = Vec::new();
    for r in &rotulos {
        let largura = ts.prefix_width(r, fonte);
        if largura > coluna && !AINDA_CORTAM.contains(r) {
            cortados.push(format!(
                "{r:?} mede {largura:.1} px numa coluna de {coluna:.1}"
            ));
        }
    }
    assert!(
        cortados.is_empty(),
        "{} rotulo(s) da §14 nao cabem na coluna deles a' largura de omissao:\n  {}\n\n\
         A unidade fisica vive no CAMPO desde 2026-09-14 (`num_row_unit`) — um rotulo que a \
         carregue outra vez volta a ser cortado.",
        cortados.len(),
        cortados.join("\n  ")
    );
}

/// ⚠️ **A metade de OBSOLESCÊNCIA** (`CLAUDE.md` §5.0) — uma tolerância que já não descreve nada
/// sai da lista.
#[test]
fn the_elision_tolerance_still_describes_something() {
    let rotulos = ph2d_panel_inspector::player_row_labels();
    let coluna = ph2d_editor_core::widget::property_label_col_w(0.0, largura_de_uma_linha());
    let fonte = TypeToken::Sm.px();
    let mut ts = TextSystem::new();
    let mortas: Vec<&&str> = AINDA_CORTAM
        .iter()
        .filter(|d| {
            !rotulos
                .iter()
                .any(|r| r == *d && ts.prefix_width(r, fonte) > coluna)
        })
        .collect();
    assert!(
        mortas.is_empty(),
        "entrada(s) STALE na tolerancia — o rotulo sumiu ou ja' cabe:\n  {mortas:?}"
    );
}

/// ⭐⭐⭐ **O PINTOR pede emprestado — e isto mede-o no PRODUTO, não na porta.**
///
/// ⛔⛔ **A 1.ª redacção do gate da escada calculava a coluna ELA PRÓPRIA** (chamando a porta com o
/// rótulo mais largo) e por isso **sobreviveu** à mutação que apaga o pedido no pintor: ela provava
/// a porta, não a fiação. *Um gate que refaz a conta do produto mede a conta, não o produto.*
///
/// ⇒ aqui a régua é o **rect que o painel REGISTOU** para um campo da §14: a largura dele diz onde
/// a coluna do rótulo acabou.
#[test]
fn the_painter_borrows_the_slack_the_control_does_not_need() {
    use ph2d_editor_core::screens::layout::HeroLayout;
    use ph2d_editor_core::zones::Rect;
    use ph2d_panel_inspector::{InspectorPanel, InspectorState, set_current_inspector_player};
    use ph2d_ui_testkit::MockPanelHost;

    /// A largura do controlo se a coluna do rótulo fosse SÓ a metade da linha.
    fn controlo_a_meio(painel: f32) -> f32 {
        let linha = painel - 2.0 * ph2d_tokens::PANEL_HEAD_PAD_PX - 2.0 * Spacing::Sm.px();
        let gap = Spacing::Md.px();
        let util = linha - ph2d_editor_core::widget::DECORATOR_W;
        util - (linha * 0.5 - gap) - gap
    }

    let campo = |painel_w: f32| -> f32 {
        let viewport = Rect {
            x: 0.0,
            y: 0.0,
            w: 2400.0,
            h: 8000.0,
        };
        let mut layout = HeroLayout::for_viewport(viewport);
        layout.inspector = Rect {
            x: viewport.w - painel_w,
            y: 0.0,
            w: painel_w,
            h: 8000.0,
        };
        let mut host = MockPanelHost::with_panel::<InspectorPanel>();
        let mut state = InspectorState::default();
        set_current_inspector_player(Some(crate::seam_player::player()));
        let rects = host.paint_with_layout::<InspectorPanel>(&mut state, layout, viewport);
        set_current_inspector_player(None);
        rects
            .iter()
            .find(|(n, _)| *n == ph2d_panel_inspector::ids::INSP_PLAYER_FLOAT)
            .map(|(_, r)| r.w)
            .expect("o campo Float Height nao foi pintado")
    };

    // ⭐ NA LARGURA DO DONO (`dock_w_right = 220,9`): o controlo cede espaço ao rótulo.
    let estreito = campo(220.9);
    let meio_estreito = controlo_a_meio(220.9);
    assert!(
        estreito < meio_estreito - 1.0,
        "a 220,9 o campo mede {estreito:.1} e a metade daria {meio_estreito:.1} — \
         o rotulo NAO pediu emprestado"
    );
    // ⭐ NA LARGURA DE OMISSÃO: a metade já chega a todos, e o desenho do dono fica intacto.
    let largo = campo(304.0);
    let meio_largo = controlo_a_meio(304.0);
    assert!(
        (largo - meio_largo).abs() < 1.0,
        "a 304 o campo mede {largo:.1} e devia ser a metade ({meio_largo:.1}) — \
         o emprestimo mexeu onde nao devia"
    );
}
