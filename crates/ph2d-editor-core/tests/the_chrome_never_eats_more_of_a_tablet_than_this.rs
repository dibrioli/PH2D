//! ⭐⭐⭐ **O ORÇAMENTO DE ECRÃ É UM NÚMERO QUE REPROVA** — não uma boa intenção.
//!
//! > Enio, 2026-08-31: *«Lembre-se que esse app tem tablets e iPad como alvo. Não podemos ir
//! > perdendo espaço.»*
//!
//! # ⛔⛔ Por que uma nota não chegava
//!
//! A `D2` tem escrito *«⚠️ o preço que o Enio aceitou: a barra global come uma faixa de altura
//! permanente»* — e uma segunda faixa (o cabeçalho da área) foi construída na entrega 30 **e
//! revertida no mesmo dia**, porque a nota não trava nada. *Uma restrição sem instrumento é uma
//! nota que envelhece* (`CLAUDE.md` §2).
//!
//! ⇒ este gate mede a **área de desenho** como percentagem da janela, nos três tablets reais, e
//! reprova quando ela desce. Uma faixa nova passa a ter de dizer o que devolve.
//!
//! # ⚠️ Ele mede o PRODUTO, e por isso faz as DUAS passagens
//!
//! A altura da fila de ferramentas depende da largura da área (ela quebra em linhas), e a largura
//! da área não depende da altura da fila. É a mesma aritmética do `hero::frame_layout`: uma
//! passagem com a faixa a zero para ler a largura, e a definitiva a seguir. ⛔ Medir com a faixa
//! fixa daria a resposta de uma janela que não existe.
//!
//! # ⚠️ E a catraca tem CENSO, senão vira licença
//!
//! Toda linha traz um piso **e** um tecto: o piso reprova quando se perde área, e o tecto reprova
//! quando se GANHA muito — porque nesse dia a barra ficou obsoleta e tem de descer. *Uma catraca
//! sem censo de obsolescência não desce: ela vira licença* (`CLAUDE.md` §5.0).

use ph2d_editor_core::screens::hero::{HeroScreen, menu_bar, tool_bar};
use ph2d_editor_core::screens::layout::{CenterSplit, ChromeBands, DockSides, HeroLayout};
use ph2d_editor_core::widget::RailButtonSize;
use ph2d_editor_core::zones::Rect;

/// Os três tablets, em pontos lógicos. ⚠️ **O menor manda** — é nele que a aritmética do chrome
/// morde primeiro, e é ele que o `spec/01 §6` nomeia como a restrição que não escala.
const TABLETS: [(&str, f32, f32); 3] = [
    ("iPad 12.9", 1366.0, 1024.0),
    ("iPad 11", 1194.0, 834.0),
    ("iPad mini", 1133.0, 744.0),
];

/// ⭐ **A catraca**, medida em 2026-08-31 (entrega 31), com as duas colunas ABERTAS — que é o
/// estado de trabalho, não o melhor caso.
///
/// | alvo | sem pincel | com pincel |
/// |---|---:|---:|
/// | iPad 12.9 | 50,8 % | 50,8 % |
/// | iPad 11 | 44,0 % | **40,8 %** |
/// | iPad mini | 40,9 % | **37,6 %** |
///
/// ⛔⛔ **A diferença entre as duas colunas é a fila de ferramentas a QUEBRAR em duas linhas** —
/// `54 → 108 px` — e ela só quebra nos dois tablets menores. É a maior perda que este gate
/// documenta e ela **não está curada**: a cura decidida é o transbordo ir para um controlo, nunca
/// a faixa crescer (ver o handoff §26).
const FLOOR: [(&str, bool, f32); 6] = [
    ("iPad 12.9", false, 50.8),
    ("iPad 12.9", true, 50.8),
    ("iPad 11", false, 44.0),
    ("iPad 11", true, 40.8),
    ("iPad mini", false, 40.9),
    ("iPad mini", true, 37.6),
];

/// Quanto acima do piso é «ganhou-se área e a barra ficou obsoleta».
const STALE_ABOVE: f32 = 2.0;

/// A área de desenho, em % da janela, com as duas colunas abertas.
fn drawing_area_pct(
    store: &ph2d_editor_core::interaction::WidgetStore,
    w: f32,
    h: f32,
    painter: bool,
) -> f32 {
    let vp = Rect::new(0.0, 0.0, w, h);
    let mut bands = ChromeBands {
        rail_w: 0.0,
        top_bar_h: menu_bar::MENU_BAR_H,
        tool_bar_h: 0.0,
        ..ChromeBands::DEFAULT
    };
    let flat = HeroLayout::for_viewport_bands(vp, false, bands, CenterSplit::None, DockSides::BOTH);
    let lines = tool_bar::tool_bar_lines(store, painter, false, flat.draw_area.w);
    bands.tool_bar_h = tool_bar::tool_bar_h(RailButtonSize::Small, lines);
    let l = HeroLayout::for_viewport_bands(vp, false, bands, CenterSplit::None, DockSides::BOTH);
    100.0 * l.draw_area.w * l.draw_area.h / (w * h)
}

/// ⭐⭐⭐ **O chrome não come mais do tablet do que já comia.**
#[test]
fn the_chrome_never_eats_more_of_a_tablet_than_this() {
    let h = HeroScreen::new(ph2d_editor_core::NodeId(1));
    let mut measured = 0usize;
    let mut stale = Vec::new();
    for (name, painter, floor) in FLOOR {
        let (_, w, ht) = TABLETS
            .iter()
            .find(|(n, _, _)| *n == name)
            .copied()
            .unwrap_or_else(|| panic!("controlo: `{name}` saiu da lista de tablets"));
        let pct = drawing_area_pct(&h.store, w, ht, painter);
        println!(
            "{name:11} pincel={painter:5}  area de desenho = {pct:5.1} %  (piso {floor:.1} %)"
        );
        assert!(
            pct >= floor - 0.05,
            "{name}{}: a area de desenho caiu para {pct:.1} % (piso {floor:.1} %) — uma faixa nova \
             comeu ecra de um tablet",
            if painter { " a pintar" } else { "" }
        );
        if pct > floor + STALE_ABOVE {
            stale.push(format!(
                "{name} pincel={painter}: {pct:.1} % contra piso {floor:.1} %"
            ));
        }
        measured += 1;
    }
    assert_eq!(measured, FLOOR.len());
    assert!(
        stale.is_empty(),
        "a catraca ficou OBSOLETA — ganhou-se area e o piso nao desceu, logo ele deixou de \
         defender o que mede:\n  {}",
        stale.join("\n  ")
    );
}

/// ⛔⛔ **E RECOLHER as colunas devolve o ecrã** — é a maior alavanca que existe, e ela é medida.
///
/// ⚠️ Ela está aqui para que a diferença entre os dois estados **não encolha em silêncio**: no dia
/// em que uma faixa permanente nascer, é este número que deixa de valer a pena.
#[test]
fn collapsing_both_columns_still_gives_the_tablet_back() {
    let h = HeroScreen::new(ph2d_editor_core::NodeId(1));
    for (name, w, ht) in TABLETS {
        let vp = Rect::new(0.0, 0.0, w, ht);
        let bands = ChromeBands {
            rail_w: 0.0,
            top_bar_h: menu_bar::MENU_BAR_H,
            tool_bar_h: tool_bar::tool_bar_h(RailButtonSize::Small, 1),
            ..ChromeBands::DEFAULT
        };
        let closed = HeroLayout::for_viewport_bands(
            vp,
            false,
            bands,
            CenterSplit::None,
            DockSides {
                left: false,
                right: false,
            },
        );
        let pct = 100.0 * closed.draw_area.w * closed.draw_area.h / (w * ht);
        println!("{name:11} colunas fechadas = {pct:5.1} %");
        // ⚠️ `88,9` e não `89,0`: o menor dos três mede `88,97 %`, e uma barra escrita a partir do
        // número IMPRESSO (arredondado) reprova sobre produto correcto — foi o que aconteceu na
        // 1.ª corrida deste gate.
        assert!(
            pct >= 88.9,
            "{name}: fechar as duas colunas devolve so' {pct:.1} % — o chrome permanente cresceu"
        );
    }
    let _ = h;
}
