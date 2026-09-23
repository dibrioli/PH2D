//! ⭐⭐⭐ **A LINHA DE TOKEN É UMA LINHA DE LISTA — a etiqueta de cor QUADRADA da altura de uma
//! linha, no MESMO `x` em toda linha (autorada, avisada ou não), e o *Reset* um ÍCONE.**
//!
//! ⛔⛔ **Report do dono, 2026-09-21:** *«os seletores de cor de todo o app precisam ser
//! padronizados»* e *«quanto ao alinhamento precisamos melhorar em todos os lugares»*.
//!
//! Até 2026-09-23 este painel era a maior população de selectores do app — `86` quadrados de
//! `32 px`, a aresta sugerida de uma PALETA (`SwatchSize::Md`) — e o *Reset* era um botão de TEXTO
//! de `48 px` a sair do nome de toda linha autorada. Hoje a amostra tem a forma das outras LISTAS com
//! cor do app (a pilha de aparência do vetor, as camadas de forma do Painter).
//!
//! ⛔ **A barra na coluna do valor e a etiqueta à DIREITA do nome foram construídas e RECUSADAS por
//! medição** (`108` e `29` nomes comidos no dock mínimo contra `5`) — o porquê está no doc do
//! `paint_token_row`.
//!
//! ⚠️ **A régua geométrica do app não vê este painel, e é por isso que esta existe:** o
//! `um_seletor_de_cor_sozinho_na_fileira_ocupa_uma_caixa_estrutural` mede só os selectores
//! SOZINHOS na fileira, e toda linha daqui tem o botão de elo ao lado.
//!
//! **Mutações que devem sangrar:** a amostra voltar ao `SwatchSize::Md` · o *Reset* voltar a ser um
//! botão de texto · a amostra de uma linha autorada sair do `x` das outras.

use ph2d_editor_core::panel::{Panel, PanelHostInternal};
use ph2d_editor_core::zones::Rect;
use ph2d_panel_tokens::state::TokensPanelState;
use ph2d_panel_tokens::{TokensPanel, ids};
use ph2d_tokens::color::Color;
use ph2d_tokens::overrides::{TokenValue, clear_color_overrides, set_color_override};
use ph2d_tokens::{ColorToken, NumToken, ROW_H_PX, Theme};
use ph2d_ui_testkit::MockPanelHost;

/// ⚠️ Estreito de propósito: é no dock estreito que uma cauda variável morde o nome e a etiqueta.
const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 300.0,
    h: 12_000.0,
};

/// Todos os rects de `ids` que o painel registou, pela ordem.
fn rects(
    regs: &[(ph2d_a11y::NodeId, Rect)],
    ids: impl Iterator<Item = ph2d_a11y::NodeId>,
) -> Vec<Rect> {
    ids.filter_map(|id| regs.iter().find(|(n, _)| *n == id).map(|(_, r)| *r))
        .collect()
}

/// Os `x`/`w` que não concordam com o primeiro.
fn desalinhados(rs: &[Rect]) -> Vec<String> {
    let a = rs[0];
    rs.iter()
        .enumerate()
        .filter(|(_, r)| (r.x - a.x).abs() > 0.5 || (r.w - a.w).abs() > 0.5)
        .map(|(i, r)| {
            format!(
                "#{i}: x={:.1} w={:.1} (o 1.º: x={:.1} w={:.1})",
                r.x, r.w, a.x, a.w
            )
        })
        .collect()
}

#[test]
fn a_etiqueta_de_cor_e_o_quadrado_da_linha_no_mesmo_x_e_o_reset_e_um_icone() {
    clear_color_overrides();
    // ⭐ O CONTROLO que torna o gate honesto: uma linha AUTORADA (com *Reset*) e uma que SEGUE outra
    //    (rótulo mais longo). Sem elas toda linha tem a mesma cauda por acaso.
    let theme = Theme::default();
    set_color_override(
        theme,
        ColorToken::ALL[3],
        Some(TokenValue::Literal(Color::from_hex(0x00FF00))),
    )
    .unwrap();
    set_color_override(
        theme,
        ColorToken::ALL[5],
        Some(TokenValue::Alias(ColorToken::ALL[0])),
    )
    .unwrap();

    let mut h = MockPanelHost::with_panel::<TokensPanel>();
    h.set_panel_visible(TokensPanel::ID, true);
    let mut st = TokensPanelState::default();
    let _ = h.painted_rect::<TokensPanel>(&mut st, VIEWPORT, ids::tokens_swatch_id(0));
    let regs = h.registos_da_ultima_pintura();
    clear_color_overrides();

    let reset = rects(&regs, std::iter::once(ids::tokens_reset_id(3)));
    assert!(
        !reset.is_empty(),
        "o CONTROLO falhou: a linha autorada não pintou o Reset — o gate mediria só linhas iguais"
    );
    let etiquetas = rects(&regs, (0..ColorToken::ALL.len()).map(ids::tokens_swatch_id));
    let chips = rects(&regs, (0..NumToken::ALL.len()).map(ids::tokens_num_chip_id));
    assert!(
        etiquetas.len() >= 60 && chips.len() >= 10,
        "a varredura leu {} etiquetas e {} chips — ela está a medir o sítio errado",
        etiquetas.len(),
        chips.len()
    );
    let mut maus = desalinhados(&etiquetas);
    maus.extend(desalinhados(&chips));
    assert!(
        maus.is_empty(),
        "linhas do painel Tokens com o valor noutro sítio:\n  {}",
        maus.join("\n  ")
    );
    // ⭐ A ETIQUETA de uma linha de lista é um quadrado da altura de uma linha — nunca a aresta
    //    sugerida de uma PALETA (`SwatchSize::Md`, os `32 px` de antes).
    let e = etiquetas[0];
    assert!(
        (e.w - ROW_H_PX).abs() < 0.5 && (e.h - ROW_H_PX).abs() < 0.5,
        "a etiqueta de cor mede {:.1}×{:.1} — a de uma linha de lista é o quadrado de `ROW_H_PX` \
         ({ROW_H_PX})",
        e.w,
        e.h
    );
    // ⭐ E o *Reset* é um ÍCONE: com texto ele volta a comer o nome de toda linha autorada.
    assert!(
        reset[0].w <= ROW_H_PX,
        "o Reset mede {:.1} px de largura — voltou a ser um botão de texto",
        reset[0].w
    );
}
