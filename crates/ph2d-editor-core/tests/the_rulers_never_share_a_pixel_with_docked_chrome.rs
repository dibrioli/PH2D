//! **As RÉGUAS deixam de partilhar coordenada com o chrome** — a primeira lei do modelo de
//! áreas (D5, `docs/UI_New_and_Simple/spec/01_modelo_de_areas.md` §4).
//!
//! # O defeito que este portão fixa
//!
//! Até 2026-08-30 a régua era ancorada em `HeroLayout::canvas`, que **é a viewport inteira**
//! (`layout.rs`, `let canvas = Rect::new(viewport.x, viewport.y, viewport.w, viewport.h)`).
//! O trilho de ferramentas nasce em `x = viewport.x` e a barra de topo em `y = viewport.y + 14`,
//! e as duas são pintadas **depois** da régua. ⇒ no viewport de referência (1366 × 1024, que é
//! o iPad Pro 12,9" — o alvo declarado dos tokens) a régua da esquerda ficava **86,8 % tapada** e
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
    DockSides, HERO_VIEWPORT_H, HERO_VIEWPORT_W, HeroLayout, rail_w,
};
use ph2d_editor_core::zones::Rect;

fn reference_viewport() -> Rect {
    Rect::new(0.0, 0.0, HERO_VIEWPORT_W, HERO_VIEWPORT_H)
}

/// Área da intersecção de dois rects (0 quando não se tocam).
#[path = "common/hero_sources.rs"]
mod hero_sources;

fn overlap_area(a: Rect, b: Rect) -> f32 {
    let w = (a.x + a.w).min(b.x + b.w) - a.x.max(b.x);
    let h = (a.y + a.h).min(b.y + b.h) - a.y.max(b.y);
    if w <= 0.0 || h <= 0.0 { 0.0 } else { w * h }
}

/// O chrome **docado** deste layout — o que ocupa faixa fixa, e por isso o que uma região
/// irmã nunca pode tocar. ⚠️ Os painéis só entram quando a coluna deles está aberta: um
/// painel fechado não é pintado, e reservar-lhe espaço seria o defeito simétrico.
fn docked_chrome(l: &HeroLayout, docks: DockSides, _mirrored: bool) -> Vec<(&'static str, Rect)> {
    let mut v = vec![
        ("top_bar", l.top_bar),
        ("left_rail", l.left_rail),
        ("bottom_hud", l.bottom_hud),
    ];
    // ⚠️ Por POSICAO, nunca por nome: o `mirrored` troca a Hierarchy com o dock de takeover, e
    // pedir «o rect da Hierarchy» chamando-lhe «a coluna da esquerda» e' o erro que o
    // compilador nao ve'.
    let (left_col, right_col) = l.side_columns();
    if docks.left {
        v.push(("coluna_esquerda", left_col));
    }
    if docks.right {
        v.push(("coluna_direita", right_col));
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

/// **Uma área estreita demais não pinta régua NEM responde a uma** — a porta única
/// [`ruler::live_bands`].
///
/// ⛔ Achado da auditoria de 2026-08-30: o desenho tinha a guarda `<= RULER_PX` e o hit-test
/// não. A wave das áreas tornou a faixa alcançável (deixou de exigir uma janela de 20 px e
/// passou a exigir uma de ~735 px de largura). *Visível ⇔ vivo*, e o inverso — responder sem
/// aparecer — é a forma pior.
#[test]
fn a_band_too_narrow_to_paint_is_also_too_narrow_to_answer() {
    let px = ph2d_editor_core::ruler::RULER_PX;
    for (w, h) in [
        (0.0, 500.0),
        (px - 1.0, 500.0),
        (px, 500.0),
        (500.0, px),
        (500.0, px - 1.0),
    ] {
        let r = Rect::new(10.0, 20.0, w, h);
        assert!(
            ruler::live_bands(r).is_none(),
            "uma area {w}x{h} nao comporta regua (RULER_PX={px}) e mesmo assim ofereceu faixas"
        );
        // O ponto no canto superior esquerdo, que e' onde a faixa nasceria.
        assert!(
            ruler::hit(r, (r.x + 1.0, r.y + 1.0)).is_none(),
            "a area {w}x{h} nao pinta regua e mesmo assim RESPONDE a uma - chrome morto sob o \
             dedo, ao contrario"
        );
    }
    // Controlo: logo acima do limiar as duas metades acordam JUNTAS.
    let ok = Rect::new(10.0, 20.0, px + 1.0, px + 1.0);
    assert!(ruler::live_bands(ok).is_some());
    assert!(ruler::hit(ok, (ok.x + 1.0, ok.y + 1.0)).is_some());
}

/// **Uma faixa docada no fundo não corre por baixo da régua** — o `timeline` nasce exactamente
/// no `area_x0` e partilhava 20 × 240 px² com a régua da esquerda.
#[test]
fn a_bottom_dock_takes_the_height_it_occupies_from_the_drawing_area() {
    let mut l = HeroLayout::for_viewport_docked(
        reference_viewport(),
        false,
        rail_w(),
        ph2d_editor_core::screens::layout::CenterSplit::None,
        DockSides::BOTH,
    );
    let before = overlap_area(ruler::left_band(l.draw_area), l.timeline);
    assert!(
        before > 0.0,
        "controlo: sem reserva, a regua esquerda TINHA de partilhar pixels com o dock do \
         timeline (partilhou {before})"
    );
    l.reserve_bottom_strip(l.timeline);
    assert_eq!(
        overlap_area(ruler::left_band(l.draw_area), l.timeline),
        0.0,
        "a regua esquerda continua a correr por baixo do dock do timeline"
    );
    assert!(l.draw_area.h > 0.0, "a area colapsou ao reservar a faixa");
    // Idempotente, e inerte para uma faixa vazia.
    let h = l.draw_area.h;
    l.reserve_bottom_strip(l.timeline);
    l.reserve_bottom_strip(Rect::new(0.0, 0.0, 0.0, 0.0));
    assert_eq!(l.draw_area.h, h);
}

/// **A reserva da faixa de fundo está FIADA** — a lei geométrica acima chama
/// `reserve_bottom_strip` à mão, então sem este gate apagar a chamada do produto deixa a suite
/// inteira verde e a régua volta a correr por baixo do timeline.
///
/// ⚠️ E a ORDEM é load-bearing: tem de vir **depois** do `dock_timeline_into_motion`, que MOVE
/// o rect do timeline — reservar antes reservaria o sítio errado.
#[test]
fn the_bottom_strip_reservation_is_wired_and_runs_after_the_motion_dock() {
    // ⚠️ **O ficheiro é PROCURADO, não nomeado** — ver `common/hero_sources.rs`: este gate
    // nomeava o `paint.rs` e o corte de 2026-08-30 mudou as três linhas para o irmão
    // `frame_layout.rs`, com a acusação falsa *«ninguém reserva a faixa»*.
    let (name, hero_paint) =
        hero_sources::hero_file_containing("layout.dock_timeline_into_motion();")
            .expect("alguém em screens/hero doca o timeline");
    let hero_paint = hero_paint.as_str();
    let dock = hero_paint
        .find("layout.dock_timeline_into_motion();")
        .expect("o timeline docado no Motion");
    let timeline = hero_paint
        .find("layout.reserve_bottom_strip(layout.timeline);")
        .expect(
            "o dock do timeline nao e' reservado: a regua da esquerda volta a correr por baixo \
             dele (20 x 240 px2 no viewport de referencia)",
        );
    let flip = hero_paint
        .find("layout.reserve_bottom_strip(layout.timeline);")
        .expect("a tira do Flip nao e' reservada — o irmao do timeline");
    assert!(
        timeline > dock && flip > dock,
        "{name}: a reserva corre ANTES do `dock_timeline_into_motion`, que move o rect do \
         timeline — reservaria o sitio errado"
    );
}

/// ⭐⭐⭐ **A ocupação das colunas vem do que os painéis PUBLICARAM, não de uma lista.**
///
/// ⛔ Esta é a 3.ª tentativa. A 1.ª foi uma lista de cinco chaves que **esquecia o painel
/// Vector** — e por isso falhava exactamente no único modo em que a régua então existia. A 2.ª
/// foi um teorema (*régua viva ⇒ Vector visível ⇒ coluna ocupada*) que **durou um dia**: o Enio
/// pediu as réguas em todos os modos e a primeira implicação evaporou-se.
///
/// A 3.ª não tem lista nem dedução: cruza os rects publicados com os rects das colunas. Um
/// inquilino novo, um *bridge* novo ou um painel que ninguém previu respondem sozinhos.
#[test]
fn the_columns_are_occupied_by_what_was_published_not_by_a_list_of_names() {
    let l = HeroLayout::for_viewport_docked(
        reference_viewport(),
        false,
        rail_w(),
        ph2d_editor_core::screens::layout::CenterSplit::None,
        DockSides::BOTH,
    );
    let (left_col, right_col) = l.side_columns();

    assert_eq!(
        DockSides::from_published(left_col, right_col, []),
        DockSides::NONE,
        "sem nada publicado, nenhuma coluna esta' ocupada"
    );
    assert_eq!(
        DockSides::from_published(left_col, right_col, [right_col]),
        DockSides {
            left: false,
            right: true
        },
        "o rect da coluna da direita ocupa a coluna da direita — e SO' ela"
    );
    assert_eq!(
        DockSides::from_published(left_col, right_col, [left_col, right_col]),
        DockSides::BOTH
    );

    // ⭐ O caso que mata a lista: um painel que NENHUMA lista conhece — desde que publique o
    // rect da coluna, ele ocupa-a. E' o painel Vector, o Physics, o Sculpt3D, e o proximo.
    let inquilino_desconhecido = Rect::new(right_col.x, right_col.y, right_col.w, right_col.h);
    assert!(
        DockSides::from_published(left_col, right_col, [inquilino_desconhecido]).right,
        "um inquilino que nenhuma lista nomeia ocupa a coluna na mesma — e' o ponto todo"
    );

    // ⚠️ E um painel FLUTUANTE que so' rocA a coluna nao a toma: sem isto a regua saltaria
    // enquanto o artista arrasta um popover por cima.
    let rocar = Rect::new(
        right_col.x + right_col.w * 0.8,
        right_col.y,
        right_col.w,
        40.0,
    );
    assert!(
        !DockSides::from_published(left_col, right_col, [rocar]).right,
        "um rect que so' rocA a coluna nao a ocupa"
    );
}

/// **As RÉGUAS vivem em TODO modo** (Enio, 2026-08-30) — a porta pergunta uma coisa só.
///
/// ⛔ Havia uma segunda condição (`is_panel_visible("vector")`) e ela caiu. O gate fica para que
/// o regresso dela seja uma decisão e não um descuido: era uma cerca com motivo escrito, e o
/// motivo (a faixa nascer INVISÍVEL debaixo do chrome) foi curado pela área de desenho.
#[test]
fn the_rulers_are_not_scoped_to_one_tool() {
    const OFFERS: &str = include_str!("../src/screens/hero/offers.rs");
    let body = OFFERS
        .split("pub fn rulers_live(&self) -> bool {")
        .nth(1)
        .expect("a porta `rulers_live`");
    let body = &body[..body.find('}').expect("o corpo da porta")];
    assert!(
        body.contains("self.view.rulers_visible"),
        "a porta deixou de perguntar pelo interruptor do artista"
    );
    assert!(
        !body.contains("is_panel_visible"),
        "as reguas voltaram a ser escopadas a uma ferramenta. O Enio pediu-as em TODOS os modos \
         e layouts (2026-08-30); se ha' motivo novo para as escopar, ele nao pode ser o antigo \
         (a faixa invisivel debaixo do chrome), que a area de desenho curou"
    );
}
