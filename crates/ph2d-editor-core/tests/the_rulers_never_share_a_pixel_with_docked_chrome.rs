//! **As RÉGUAS deixam de partilhar coordenada com o chrome** — a primeira lei do modelo de
//! áreas (D5, `docs/UI_New_and_Simple/spec/01_modelo_de_areas.md` §4).
//!
//! # O defeito que este portão fixa
//!
//! Até 2026-08-30 a régua era ancorada em `HeroLayout::canvas`, que **é a viewport inteira**
//! (`layout.rs`, `let canvas = Rect::new(viewport.x, viewport.y, viewport.w, viewport.h)`).
//! O trilho de ferramentas nasce em `x = viewport.x` e a barra de topo em `y = viewport.y + 14`,
//! e as duas são pintadas **depois** da régua. ⇒ no viewport de referência (1366 × 1024, que é
//! o iPad Pro 12,9" — o alvo declarado dos tokens) a régua da esquerda ficava **87,8 % tapada** e
//! a de cima **29,4 %** (medição em `docs/UI_New_and_Simple/medicoes/02_a_area_tapada.md`).
//!
//! ⭐ **A cura não é uma verificação — é a ausência de coordenada partilhada.** As réguas passam
//! a ser regiões da [`HeroLayout::draw_area`], que começa depois da coluna da esquerda e acaba
//! antes da da direita. Duas regiões irmãs não se tapam porque não ocupam o mesmo espaço.
//!
//! # ⛔ E há uma segunda metade, que é de INPUT e não se vê
//!
//! A régua **não está no `HitIndex`**: o gesto de guia é geométrico
//! (`ruler::hit(host, p)`) e corre em `input_dispatch.rs` **antes** do hit-test de chrome, com
//! um `return` quando acerta. Enquanto o hospedeiro foi a janela inteira, um press nos
//! **6 px de cima de qualquer botão da barra** (a banda de cima é `y ∈ [0, 20]`, a barra começa
//! em `y = 14`) ou nos **3 px da esquerda de qualquer chip do trilho** (a banda esquerda é
//! `x ∈ [0, 20]`, o chip começa em `x = 17`) **nascia uma guia em vez de carregar no botão** —
//! e nenhum gate do repo media isto, porque todos perguntam pelo `HitIndex`, onde a régua não
//! está.
//!
//! Os dois testes de CONTROLO abaixo reproduzem os dois defeitos com a âncora antiga. Sem eles
//! a lei podia passar por a função de medida devolver zero por engano — *um zero de «não medido»
//! e um de «perfeito» são o mesmo byte*.

use ph2d_editor_core::ruler;
use ph2d_editor_core::screens::layout::{
    DockSides, HERO_VIEWPORT_H, HERO_VIEWPORT_W, HeroLayout, RIGHT_DOCK_PANELS, rail_w,
};
use ph2d_editor_core::zones::Rect;

const LAYOUT_SRC: &str = include_str!("../src/screens/layout.rs");

fn reference_viewport() -> Rect {
    Rect::new(0.0, 0.0, HERO_VIEWPORT_W, HERO_VIEWPORT_H)
}

/// Área da intersecção de dois rects (0 quando não se tocam).
fn overlap_area(a: Rect, b: Rect) -> f32 {
    let w = (a.x + a.w).min(b.x + b.w) - a.x.max(b.x);
    let h = (a.y + a.h).min(b.y + b.h) - a.y.max(b.y);
    if w <= 0.0 || h <= 0.0 { 0.0 } else { w * h }
}

/// O chrome **docado** deste layout — o que ocupa faixa fixa, e por isso o que uma região
/// irmã nunca pode tocar. ⚠️ Os painéis só entram quando a coluna deles está aberta: um
/// painel fechado não é pintado, e reservar-lhe espaço seria o defeito simétrico.
fn docked_chrome(l: &HeroLayout, docks: DockSides, mirrored: bool) -> Vec<(&'static str, Rect)> {
    let mut v = vec![
        ("top_bar", l.top_bar),
        ("left_rail", l.left_rail),
        ("bottom_hud", l.bottom_hud),
    ];
    let (left_panel, right_panel) = if mirrored {
        (("inspector", l.inspector), ("hierarchy", l.hierarchy))
    } else {
        (("hierarchy", l.hierarchy), ("inspector", l.inspector))
    };
    if docks.left {
        v.push(left_panel);
    }
    if docks.right {
        v.push(right_panel);
    }
    v
}

fn all_dock_states() -> [DockSides; 4] {
    [
        DockSides::BOTH,
        DockSides::NONE,
        DockSides {
            left: true,
            right: false,
        },
        DockSides {
            left: false,
            right: true,
        },
    ]
}

/// **A LEI.** Nas duas orientações e nos quatro estados de coluna, nenhuma das duas faixas de
/// régua partilha um único pixel com chrome docado.
#[test]
fn the_rulers_never_share_a_pixel_with_docked_chrome() {
    for mirrored in [false, true] {
        for docks in all_dock_states() {
            let l = HeroLayout::for_viewport_docked(
                reference_viewport(),
                mirrored,
                rail_w(),
                ph2d_editor_core::screens::layout::CenterSplit::None,
                docks,
            );
            assert!(
                l.draw_area.w > 0.0 && l.draw_area.h > 0.0,
                "a area de desenho colapsou (mirrored={mirrored}, docks={docks:?}): {:?}",
                l.draw_area
            );
            for (band_name, band) in [
                ("top", ruler::top_band(l.draw_area)),
                ("left", ruler::left_band(l.draw_area)),
            ] {
                for (chrome_name, rect) in docked_chrome(&l, docks, mirrored) {
                    let a = overlap_area(band, rect);
                    assert_eq!(
                        a, 0.0,
                        "a regua '{band_name}' {band:?} partilha {a} px2 com '{chrome_name}' \
                         {rect:?} (mirrored={mirrored}, docks={docks:?})"
                    );
                }
            }
        }
    }
}

/// **CONTROLO nº 1 — a medida vê uma sobreposição quando ela existe.**
///
/// Com a âncora antiga (`layout.canvas`, a viewport inteira) a régua da esquerda é comida pelo
/// trilho e a de cima pela barra. Os números vêm da medição de 2026-08-30 e são reproduzidos
/// aqui a partir do próprio layout: se um dia a geometria do chrome mudar, este controlo
/// muda de número mas **não pode ir a zero** — se fosse, o teste da lei acima estaria a
/// afirmar o vazio.
#[test]
fn the_control_the_old_anchor_was_covered_and_the_measure_sees_it() {
    let l = HeroLayout::for_viewport_docked(
        reference_viewport(),
        false,
        rail_w(),
        ph2d_editor_core::screens::layout::CenterSplit::None,
        DockSides::BOTH,
    );
    let old_left = ruler::left_band(l.canvas);
    let old_top = ruler::top_band(l.canvas);

    let left_covered = overlap_area(old_left, l.left_rail)
        + overlap_area(old_left, l.top_bar)
        + overlap_area(old_left, l.bottom_hud);
    let top_covered = overlap_area(old_top, l.top_bar);

    let left_frac = left_covered / old_left.area();
    let top_frac = top_covered / old_top.area();

    assert!(
        left_frac > 0.8,
        "a ancora antiga da regua esquerda deixou de estar tapada ({left_frac:.3}) - \
         ou a geometria do chrome mudou, ou este controlo deixou de medir o que dizia"
    );
    assert!(
        top_frac > 0.2,
        "a ancora antiga da regua de cima deixou de estar tapada ({top_frac:.3})"
    );
    // E a mesma medida, sobre a area de desenho, da' zero — a cura, lado a lado com o defeito.
    assert_eq!(
        overlap_area(ruler::left_band(l.draw_area), l.left_rail),
        0.0,
        "a regua esquerda ancorada na area de desenho ainda toca o trilho"
    );
}

/// **CONTROLO nº 2 — o roubo de CLIQUE, que nenhuma sonda do repo via.**
///
/// O gesto da guia é geométrico e corre antes do hit-test de chrome. Com a âncora antiga, o
/// topo de um botão da barra e a esquerda de um chip do trilho respondiam «régua». Com a área
/// de desenho, respondem `None` — e o botão volta a ser um botão.
#[test]
fn the_ruler_no_longer_steals_the_click_from_the_top_bar_and_the_rail() {
    let l = HeroLayout::for_viewport_docked(
        reference_viewport(),
        false,
        rail_w(),
        ph2d_editor_core::screens::layout::CenterSplit::None,
        DockSides::BOTH,
    );
    // Um ponto na metade de CIMA da barra de topo, e um na coluna esquerda de um chip do
    // trilho — os dois dentro de chrome que o artista quer carregar.
    let in_top_bar = (l.top_bar.x + l.top_bar.w * 0.5, l.top_bar.y + 1.0);
    let in_rail = (l.left_rail.x + 18.0, l.left_rail.y + 10.0);

    assert!(
        ruler::hit(l.canvas, in_top_bar).is_some(),
        "o controlo falhou: com a ancora antiga o topo da barra TINHA de responder regua"
    );
    assert!(
        ruler::hit(l.canvas, in_rail).is_some(),
        "o controlo falhou: com a ancora antiga a esquerda do chip TINHA de responder regua"
    );

    assert!(
        ruler::hit(l.draw_area, in_top_bar).is_none(),
        "a regua continua a roubar o clique da barra de topo"
    );
    assert!(
        ruler::hit(l.draw_area, in_rail).is_none(),
        "a regua continua a roubar o clique do trilho"
    );
}

/// **CENSO — a lista das chaves da coluna da direita descreve o que o construtor de facto
/// põe lá.**
///
/// Os painéis que partilham o dock da direita são declarados no construtor como aliases
/// (`let bgremoval = inspector;`), e o nome do campo **é** a chave de visibilidade. Um alias
/// novo que não venha para [`RIGHT_DOCK_PANELS`] faria a área de desenho crescer para dentro
/// de um painel que está lá — e o defeito voltaria só naquela ferramenta, que é a forma de
/// regressão que ninguém encontra.
///
/// ⚠️ A varredura **descasca comentários**: documentar a lei escrevendo `= inspector;` numa
/// linha de doc não pode contar como um alias (a armadilha do gate textual que reprova quem
/// documenta a cura).
#[test]
fn the_dock_column_census_matches_the_layout_aliases() {
    let mut found: Vec<&str> = Vec::new();
    for raw in LAYOUT_SRC.lines() {
        let line = raw.trim_start();
        if line.starts_with("//") {
            continue;
        }
        let Some(rest) = line.strip_prefix("let ") else {
            continue;
        };
        let Some((name, tail)) = rest.split_once(" = ") else {
            continue;
        };
        if tail.trim_end() == "inspector;" {
            found.push(name.trim());
        }
    }
    assert!(
        !found.is_empty(),
        "a varredura nao achou UM alias do dock da direita — o construtor mudou de forma e \
         este censo deixou de medir o que diz (controlo positivo)"
    );
    let mut declared: Vec<&str> = RIGHT_DOCK_PANELS.to_vec();
    // "inspector" e o dono do rect, nao um alias dele.
    let mut expected: Vec<&str> = found.clone();
    expected.push("inspector");
    expected.sort_unstable();
    declared.sort_unstable();
    assert_eq!(
        declared, expected,
        "RIGHT_DOCK_PANELS nao descreve os aliases do dock da direita em layout.rs.\n\
         declarado: {declared:?}\nno construtor: {expected:?}"
    );
}

/// **A COLUNA, nunca o PAINEL** — o `mirrored` troca as duas, e é a metade que se escreve ao
/// contrário sem o compilador reclamar.
///
/// ⚠️ Sem este teste, inverter os dois ramos de [`DockSides::resolve`] deixa a suite inteira
/// verde: a lei geométrica acima constrói os `DockSides` à mão, então nunca passa por aqui.
#[test]
fn the_dock_sides_name_a_column_and_the_mirror_swaps_them() {
    // So' a Hierarchy aberta.
    let only_hierarchy = |k: &str| k == "hierarchy";
    assert_eq!(
        DockSides::resolve(false, only_hierarchy),
        DockSides {
            left: true,
            right: false
        },
        "sem espelho a Hierarchy ocupa a coluna da ESQUERDA"
    );
    assert_eq!(
        DockSides::resolve(true, only_hierarchy),
        DockSides {
            left: false,
            right: true
        },
        "com espelho ela muda-se para a DIREITA — como os `x` no construtor"
    );

    // Qualquer um dos cinco do dock da direita ocupa a coluna, nao so' o Inspector: e' esse
    // o ponto de a lista existir. O `painter_layers` e' o que uma lista escrita a mao esquece.
    for key in RIGHT_DOCK_PANELS {
        let only = |k: &str| k == key;
        assert_eq!(
            DockSides::resolve(false, only),
            DockSides {
                left: false,
                right: true
            },
            "'{key}' partilha o dock da direita e tem de ocupar a coluna"
        );
    }

    assert_eq!(
        DockSides::resolve(false, |_| false),
        DockSides::NONE,
        "com tudo fechado a area de desenho vai do trilho a' borda"
    );
    assert_eq!(DockSides::resolve(false, |_| true), DockSides::BOTH);
}
