//! **Uma dica que quebra em duas linhas tem de OCUPAR duas linhas.**
//!
//! ⚠️ **O defeito que este gate existe para prender é o do smoke de 2026-08-22:** a §5 pintava as
//! suas três dicas com `paint_text` — que **quebra** o texto à largura do painel — e avançava o
//! cursor `label_font`, ou seja **uma** linha. Num painel estreito as dicas ocupavam duas ou três,
//! e o resultado foi o que o Enio fotografou: *«labels muito emboladas, layout ruim»* — a dica do
//! `Simple` escrita por cima de «Borders L / T», a legenda da grelha por cima de «Tile Mode».
//!
//! O `paint_text_block` existe exatamente para isto e o doc-comment dele **já descrevia este
//! acidente**, vindo do painel de física. Ter o remédio escrito não impediu a recaída; o que a
//! impede é este ficheiro.
//!
//! # Porque MEDIR e não olhar
//!
//! Não se testa isto contando linhas — isso seria reimplementar o parley e as duas contas
//! divergiriam. Testa-se pela **consequência geométrica**: estreitar o painel faz a dica quebrar
//! em mais linhas, portanto **tem de** empurrar para baixo tudo o que vem depois dela. Se a altura
//! for estimada em vez de medida, os dois layouts saem à MESMA altura — e é isso que o gate vê.
//!
//! O intervalo medido é o que fica **entre a grelha 3×3 e o segmented do Tile Mode**, que contém
//! a legenda da grelha e mais nada. ⚠️ Ele já ancorou no «Remove» — e esse botão foi retirado
//! (ele era a segunda porta para «desligado»). *Um gate ancorado num controlo morre com ele*, por
//! isso a âncora de baixo é agora o primeiro segmento do Tile Mode, que a seção não pode perder
//! sem perder a feature.

use ph2d_editor_core::ids;
use ph2d_editor_core::screens::hero::{InspectorSliceInfo, InspectorSliceMixed};
use ph2d_editor_core::zones::Rect;
use ph2d_panel_inspector::{InspectorPanel, InspectorState, set_current_inspector_slice};
use ph2d_ui_testkit::MockPanelHost;

const ENTITY: u64 = 0x5EED_1234;
/// Largura estreita — a do painel real do Enio na fotografia.
const NARROW_W: f32 = 300.0;
/// Largura larga o bastante para as dicas caberem numa linha só.
const WIDE_W: f32 = 1100.0;

/// A §5 LIGADA — é aí que vivem as dicas que quebram (a do Tile Mode e a legenda da grelha).
///
/// ⚠️ Um fixture desligado não serve: a seção desligada mostra a caixa e o «Remove», e nenhuma
/// dica. Foi o que ela passou a ser quando a caixa «Enable 9-slice» substituiu o segmentado.
fn simple_slice() -> InspectorSliceInfo {
    InspectorSliceInfo {
        entity_bits: ENTITY,
        present: true,
        draw_mode_tag: 1,
        borders: [8.0; 4],
        size: [0.0, 0.0],
        tile_modes: [0; 8],
        centre_tile_mode: 0,
        tile_mode_tag: 0,
        fill_center: true,
        selected_count: 1,
        mixed: InspectorSliceMixed::default(),
    }
}

/// O vão entre o fundo da grelha 3×3 e o topo do segmented do Tile Mode, a esta largura.
///
/// ⚠️ **A largura tem de entrar pelo `HeroLayout`, não pelo viewport.** O Inspector é uma doca de
/// largura FIXA: alargar o viewport só afasta o canvas, e a primeira versão deste gate media
/// 55,56 px nas duas pontas — não porque a altura estivesse certa, mas porque as duas corridas
/// eram o mesmo layout. *Um gate cuja variável independente não varia mede o silêncio.*
fn hint_gap(width: f32) -> f32 {
    set_current_inspector_slice(Some(simple_slice()));
    let mut host = MockPanelHost::with_panel_and_shared_chrome::<InspectorPanel>();
    let mut state = InspectorState::default();
    host.settle_section_folds();
    let viewport = Rect {
        x: 0.0,
        y: 0.0,
        w: 2400.0,
        h: 8000.0,
    };
    let mut layout = ph2d_editor_core::screens::layout::HeroLayout::for_viewport(viewport);
    layout.inspector = Rect {
        x: 0.0,
        y: 0.0,
        w: width,
        h: 8000.0,
    };
    let rects = host.paint_with_layout::<InspectorPanel>(&mut state, layout, viewport);
    let find = |id| {
        rects
            .iter()
            .find(|(n, _)| *n == id)
            .map(|(_, r)| *r)
            .unwrap_or_else(|| panic!("id nao foi pintado a largura {width}"))
    };
    // A célula mais baixa da grelha (o canto BR) fecha o bloco de cima; o primeiro segmento do
    // Tile Mode abre o de baixo. Entre os dois só existe a LEGENDA, que é o que este gate mede —
    // e o topo é o MÍNIMO dos segmentos, porque o segmented adapta e pode quebrar em duas filas.
    let grid_bottom = ids::INSP_SLICE_REGION
        .iter()
        .map(|&id| {
            let r = find(id);
            r.y + r.h
        })
        .fold(f32::MIN, f32::max);
    let seg_top = ids::INSP_SLICE_TILE_MODE
        .iter()
        .map(|&id| find(id).y)
        .fold(f32::MAX, f32::min);
    seg_top - grid_bottom
}

/// ⚠️ **A prova.** Estreitar o painel faz a dica quebrar em mais linhas; se a altura dela for
/// medida, o vão CRESCE. Se for estimada numa linha, os dois vãos são iguais — e é assim que o
/// texto acaba por cima do rótulo seguinte.
#[test]
fn a_hint_that_wraps_pushes_what_comes_after_it_down() {
    let narrow = hint_gap(NARROW_W);
    let wide = hint_gap(WIDE_W);
    assert!(
        narrow > wide,
        "o vao da dica nao mudou com a largura ({narrow} vs {wide}): a altura dela esta a ser \
         ESTIMADA, e a linha seguinte vai ser escrita por cima"
    );
    // E a diferença é de LINHAS, não de arredondamento: a legenda tem ~90 caracteres e a 300 px
    // ela ocupa pelo menos mais uma linha inteira do que a 1100 px.
    let one_line = ph2d_tokens::TypeToken::Sm.px();
    assert!(
        narrow - wide >= one_line,
        "cresceu {} px, menos de uma linha ({one_line} px) — isto e' ruido de layout, nao a \
         quebra a ser contada",
        narrow - wide
    );
}

/// A dica **existe**: um vão de zero significaria que ela deixou de ser pintada — e o teste
/// acima passaria na mesma se as duas larguras dessem zero.
#[test]
fn the_grid_legend_is_actually_painted() {
    assert!(
        hint_gap(WIDE_W) > 0.0,
        "nao ha' vao nenhum entre a grelha e o Tile Mode: a legenda desapareceu"
    );
}
