//! Gates do [`super`] — cortados para o irmão no teto de LOC (HR-18).

use super::*;

fn vp() -> Rect {
    Rect::new(0.0, 0.0, HERO_VIEWPORT_W, HERO_VIEWPORT_H)
}

fn approx(a: f32, b: f32) -> bool {
    (a - b).abs() < 0.5
}

/// **A região de popover é a banda de CHROME** — começa abaixo da barra de topo, acaba acima
/// do HUD, e é a MESMA em que os painéis docados vivem.
///
/// As três metades importam por motivos diferentes: *abaixo da barra* é o que tira a lista de
/// cima dos botões dela; *acima do HUD* é o que a mantém dentro da janela pela outra ponta; e
/// *a mesma dos painéis* é o que garante que um popover aterra onde o painel dele está, em vez
/// de num sítio que só a janela conhece. Devolver a viewport satisfaz zero das três.
#[test]
fn the_popover_region_is_the_chrome_band_not_the_window() {
    let l = HeroLayout::for_viewport(vp());
    let r = l.popover_region();
    assert!(
        r.y >= l.top_bar.y + l.top_bar.h,
        "a regiao invade a barra de topo (region.y={} vs fim da barra={})",
        r.y,
        l.top_bar.y + l.top_bar.h
    );
    assert!(
        r.y + r.h <= l.viewport.y + l.viewport.h,
        "a regiao passa da borda de baixo da janela"
    );
    // ⛔ Havia aqui um `<` ESTRITO — *«a regiao encosta na borda de baixo»* era o defeito. Ele
    // media a reserva do HUD, e essa reserva CAIU em 2026-08-30 (o HUD é centrado, as colunas
    // vivem nas pontas, e reservar-lhe uma faixa custava a altura das duas para nada). A banda
    // encosta no fundo de propósito agora; o que este gate ainda defende é o TOPO — um popover
    // não nasce por cima da barra —, que era o defeito medido que o criou.
    // É a banda dos painéis docados: um popover nasce onde o painel dele vive.
    assert!(approx(r.y, l.inspector.y), "topo != topo do inspector");
    assert!(
        approx(r.y + r.h, l.inspector.y + l.inspector.h),
        "base != base do inspector"
    );
    // E na horizontal ele PODE transbordar para o canvas — a largura é a da janela.
    assert!(approx(r.x, l.viewport.x) && approx(r.w, l.viewport.w));
}

#[test]
fn no_split_scene_is_full_canvas_and_graph_is_empty() {
    let l = HeroLayout::for_viewport(vp());
    assert_eq!(l.center_viewport, l.canvas);
    assert_eq!(l.motion_graph.w, 0.0);
    assert_eq!(l.motion_graph.h, 0.0);
}

#[test]
fn horizontal_split_scene_on_top_graph_below_partition_the_band() {
    let l =
        HeroLayout::for_viewport_split(vp(), false, rail_w(), CenterSplit::Horizontal { t: 0.55 });
    // Scene sits above the graph; both share the band's x/width.
    assert!(l.center_viewport.y < l.motion_graph.y);
    assert!(approx(l.center_viewport.x, l.motion_graph.x));
    assert!(approx(l.center_viewport.w, l.motion_graph.w));
    // They tile the band with no gap/overlap at the divider.
    assert!(approx(
        l.center_viewport.y + l.center_viewport.h,
        l.motion_graph.y
    ));
    // Scene gets ~55 % of the band height.
    let band = l.center_viewport.h + l.motion_graph.h;
    assert!(approx(l.center_viewport.h / band, 0.55));
}

#[test]
fn vertical_split_scene_on_left_graph_on_right() {
    let l = HeroLayout::for_viewport_split(vp(), false, rail_w(), CenterSplit::Vertical { t: 0.5 });
    assert!(l.center_viewport.x < l.motion_graph.x);
    assert!(approx(l.center_viewport.y, l.motion_graph.y));
    assert!(approx(l.center_viewport.h, l.motion_graph.h));
    assert!(approx(
        l.center_viewport.x + l.center_viewport.w,
        l.motion_graph.x
    ));
}

/// The seam is still a seam: **nothing is carved until somebody asks** (the shell asks only
/// when the Motion tool AND the timeline are both on screen).
#[test]
fn timeline_slot_is_zero_height_until_it_is_docked_into() {
    let l =
        HeroLayout::for_viewport_split(vp(), false, rail_w(), CenterSplit::Horizontal { t: 0.6 });
    assert_eq!(l.motion_timeline_slot.h, 0.0);
    assert!(approx(
        l.motion_timeline_slot.y,
        l.motion_graph.y + l.motion_graph.h
    ));
}

/// **The dock takes the band from the GRAPH, and the timeline lands in it** (W4.T4).
///
/// The bug it fixes is that the two used to occupy the SAME pixels — `motion_graph` ran to the
/// chrome and `timeline` was the bottom strip — so the timeline painted over the graph. After
/// docking they must not overlap, and the graph must be exactly the shorter one.
#[test]
fn docking_the_timeline_takes_the_band_from_the_graph() {
    let split = CenterSplit::Horizontal { t: 0.6 };
    let before = HeroLayout::for_viewport_split(vp(), false, rail_w(), split);
    let mut l = HeroLayout::for_viewport_split(vp(), false, rail_w(), split);
    assert!(
        before.timeline.y < before.motion_graph.y + before.motion_graph.h,
        "the whole point: before the dock, the timeline lay ON the graph"
    );

    l.dock_timeline_into_motion();
    let band = l.motion_timeline_slot;
    assert!(band.h > 0.0, "the band exists");
    assert!(
        approx(l.motion_graph.h + band.h, before.motion_graph.h),
        "the band came OUT of the graph: {} + {} != {}",
        l.motion_graph.h,
        band.h,
        before.motion_graph.h
    );
    assert!(
        approx(band.y, l.motion_graph.y + l.motion_graph.h),
        "and it sits directly under it"
    );
    assert_eq!(l.timeline, band, "the timeline IS the band now");
    assert!(
        l.timeline.y >= l.motion_graph.y + l.motion_graph.h - 0.01,
        "they no longer overlap - that was the bug"
    );
}

/// Without a split there is no Motion workspace to dock into, and the call is inert: the
/// timeline keeps the bottom dock it has everywhere else.
#[test]
fn docking_without_a_motion_split_changes_nothing() {
    let mut l = HeroLayout::for_viewport_split(vp(), false, rail_w(), CenterSplit::None);
    let before = l.timeline;
    l.dock_timeline_into_motion();
    assert_eq!(l.timeline, before);
    assert_eq!(l.motion_timeline_slot.h, 0.0);
}

/// A short window must not let the dock eat its host: the band is capped at a fraction of the
/// graph, so the node editor never becomes a sliver.
#[test]
fn the_dock_never_eats_the_graph() {
    let short = Rect::new(0.0, 0.0, 1200.0, 420.0);
    let mut l =
        HeroLayout::for_viewport_split(short, false, rail_w(), CenterSplit::Horizontal { t: 0.6 });
    l.dock_timeline_into_motion();
    assert!(
        l.motion_graph.h > 0.0,
        "the graph survived: {}",
        l.motion_graph.h
    );
    assert!(
        l.motion_timeline_slot.h <= l.motion_graph.h,
        "the band ({}) must not be bigger than what is left of the graph ({})",
        l.motion_timeline_slot.h,
        l.motion_graph.h
    );
}

#[test]
fn divider_fraction_clamps() {
    assert_eq!(CenterSplit::clamp_t(0.9), CenterSplit::T_MAX);
    assert_eq!(CenterSplit::clamp_t(0.1), CenterSplit::T_MIN);
    // Orientation switch preserves the fraction.
    let h = CenterSplit::Horizontal { t: 0.4 };
    assert_eq!(h.to_vertical(), CenterSplit::Vertical { t: 0.4 });
    assert!((h.to_vertical().t() - 0.4).abs() < 1e-6);
}

/// ⭐⭐⭐ **O SUB-RETÂNGULO DA CENA É UM NÚMERO INTEIRO DE PIXELS** — a cura do drift que
/// o Enio reportou em 2026-08-25 (*«acontece para Object e Chip, não para Star»*).
///
/// ⚠️ **A régua é a IGUALDADE entre o `f32` que o `set_viewport` recebe e o `u32` que
/// todo o resto deriva dele** (`as u32` no `scene_window_wh`/`scene_camera_window`, e
/// o `set_scissor_rect` ao lado do próprio viewport). Enquanto a porta devolvia a
/// fracção, o conteúdo raster era desenhado com `h·t / height_world` pixels por unidade
/// e o vectorial com `⌊h·t⌋ / height_world` — `0,095 %` de diferença a `768 · 0,55`.
///
/// ⚠️ **Os tamanhos vêm dos LOGS do Enio**, não de uma tabela inventada: `1022` e `768`
/// de altura, `t = 0,55` (o default), que dão `562,1` e `422,4` — as duas fracções que
/// produziram o defeito.
#[test]
fn the_scene_subrect_is_a_whole_number_of_pixels() {
    for (w, h) in [(1920.0_f32, 1022.0_f32), (1024.0, 768.0), (800.0, 601.0)] {
        for t in [
            CenterSplit::T_DEFAULT,
            CenterSplit::T_MIN,
            CenterSplit::T_MAX,
            0.333,
        ] {
            for split in [CenterSplit::Horizontal { t }, CenterSplit::Vertical { t }] {
                let r = split
                    .scene_viewport(w, h)
                    .expect("um split tem sub-retangulo");
                // ⭐ A metade que importa: o `f32` e o `u32` que dele se deriva são o
                // MESMO número. Uma fracção aqui é uma escala diferente entre as rotas.
                assert!(
                    (r[2] - (r[2] as u32) as f32).abs() < f32::EPSILON,
                    "largura {} nao e' inteira ({w}x{h}, t={t})",
                    r[2]
                );
                assert!(
                    (r[3] - (r[3] as u32) as f32).abs() < f32::EPSILON,
                    "altura {} nao e' inteira ({w}x{h}, t={t})",
                    r[3]
                );
                // E ela nunca colapsa a zero — um viewport de altura 0 nao desenha nada.
                assert!(r[2] >= 1.0 && r[3] >= 1.0);
            }
        }
    }
    // CONTROLE: sem split não há sub-retângulo (a janela inteira), e o gate acima
    // passaria por vácuo se ele passasse a devolver `Some` sempre.
    assert_eq!(CenterSplit::None.scene_viewport(1024.0, 768.0), None);
}

/// **E o sub-retângulo continua a ser a FRACÇÃO que o artista arrastou** — a cura não
/// pode virar «a cena fica sempre com metade».
///
/// ⚠️ O arredondamento tira no máximo UM pixel, e o gate mede exactamente isso: um
/// `floor` que se enganasse por mais do que um pixel moveria a divisória sozinho.
#[test]
fn rounding_the_subrect_never_moves_the_divider_by_more_than_a_pixel() {
    let (w, h) = (1024.0_f32, 768.0_f32);
    for t in [0.25_f32, 0.33, 0.55, 0.75] {
        let r = CenterSplit::Horizontal { t }
            .scene_viewport(w, h)
            .expect("split");
        assert!(
            (r[3] - h * t).abs() < 1.0,
            "a altura {} afastou-se de {} por mais de um pixel",
            r[3],
            h * t
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// ⭐⭐ **AS FAIXAS ENCAIXAM ENTRE AS COLUNAS** (Enio, 2026-08-31, com duas fotos e quatro setas:
// *«a timeline e o canvas dos Nodes devem ser bem encaixados entre os painéis laterais como na
// Godot, sem espaços»*).
//
// São DUAS regiões e o defeito de cada uma era o oposto do da outra — e é por isso que ambas
// precisam de gate:
//
// | região | o defeito | o que se via |
// |---|---|---|
// | `timeline` / `flip_strip` | 20 px a MENOS de cada lado (o `EDGE_PAD` que o `area_x0` perdeu em 30/08 e ela não) | um buraco entre o painel e a faixa |
// | `motion_graph` / `center_viewport` | a janela INTEIRA em vez da área | a banda do grafo por cima do fundo das duas colunas |
// ─────────────────────────────────────────────────────────────────────────────

/// Onde a área de desenho começa e acaba, com as duas colunas abertas.
fn area_span(l: &HeroLayout) -> (f32, f32) {
    (l.draw_area.x, l.draw_area.x + l.draw_area.w)
}

/// ⭐ **A faixa do fundo toca as DUAS colunas** — sem buraco de nenhum lado.
#[test]
fn the_bottom_strip_touches_both_columns() {
    let l = HeroLayout::for_viewport(vp());
    let (x0, x1) = area_span(&l);
    // Controlo: sem colunas com largura o gate mediria a janela contra ela própria.
    assert!(
        x0 > l.viewport.x + 1.0 && x1 < l.viewport.x + l.viewport.w - 1.0,
        "controlo: as colunas não estão reservadas ({x0}..{x1} numa janela de {})",
        l.viewport.w
    );
    for (name, r) in [("timeline", l.timeline), ("flip_strip", l.flip_strip)] {
        assert!(
            approx(r.x, x0),
            "{name} começa em {} e a coluna acaba em {x0} — {} px de buraco",
            r.x,
            r.x - x0
        );
        assert!(
            approx(r.x + r.w, x1),
            "{name} acaba em {} e a outra coluna começa em {x1} — {} px de buraco",
            r.x + r.w,
            x1 - (r.x + r.w)
        );
    }
}

/// ⭐⭐ **A banda do grafo NÃO partilha um pixel com nenhuma coluna** — nas duas orientações.
///
/// ⚠️ A régua é a INTERSECÇÃO com os rects das colunas, e não `x >= area_x0`: um split vertical
/// tem a banda encostada à direita, e só a intersecção apanha os dois lados com uma pergunta só.
#[test]
fn the_node_graph_never_covers_a_side_column() {
    for split in [
        CenterSplit::Horizontal { t: 0.55 },
        CenterSplit::Vertical { t: 0.55 },
    ] {
        let l = HeroLayout::for_viewport_split(vp(), false, rail_w(), split);
        assert!(
            l.motion_graph.w > 1.0 && l.motion_graph.h > 1.0,
            "controlo: {split:?} devolveu uma banda vazia e o gate mediria o nada"
        );
        for (name, col) in [("hierarchy", l.hierarchy), ("inspector", l.inspector)] {
            let ox = (l.motion_graph.x + l.motion_graph.w).min(col.x + col.w)
                - l.motion_graph.x.max(col.x);
            let oy = (l.motion_graph.y + l.motion_graph.h).min(col.y + col.h)
                - l.motion_graph.y.max(col.y);
            assert!(
                ox <= 0.5 || oy <= 0.5,
                "{split:?}: a banda do grafo tapa {} px² de `{name}` ({ox} × {oy})",
                ox * oy
            );
        }
    }
}

/// ⚠️ **E o CENTRO encolhe com ela** — as duas metades do split são a MESMA coluna.
///
/// ⛔ Narrar só a banda do grafo faria o painel dele ler um split horizontal como **vertical**:
/// a orientação é detectada por `rect.x > center.x`, e o arrasto do divisor é medido contra
/// `center + rect`. *Duas metades de uma partição não podem sair de janelas diferentes.*
#[test]
fn both_halves_of_the_split_shrink_together() {
    let l =
        HeroLayout::for_viewport_split(vp(), false, rail_w(), CenterSplit::Horizontal { t: 0.55 });
    let (x0, x1) = area_span(&l);
    assert!(approx(l.center_viewport.x, x0) && approx(l.motion_graph.x, x0));
    assert!(
        approx(l.center_viewport.x + l.center_viewport.w, x1)
            && approx(l.motion_graph.x + l.motion_graph.w, x1)
    );
    // A orientação continua a ler-se como horizontal.
    assert!(
        l.motion_graph.x <= l.center_viewport.x + 0.5,
        "um split HORIZONTAL passou a ler-se como vertical (graph.x={} > center.x={})",
        l.motion_graph.x,
        l.center_viewport.x
    );
}

/// ⭐ **E o timeline docado dentro do Motion herda a mesma coluna** — ele É a base da banda.
#[test]
fn the_timeline_docked_into_motion_keeps_the_column() {
    let mut l =
        HeroLayout::for_viewport_split(vp(), false, rail_w(), CenterSplit::Horizontal { t: 0.55 });
    let (x0, x1) = area_span(&l);
    l.dock_timeline_into_motion();
    assert!(l.timeline.h > 1.0, "controlo: a docagem não carvou nada");
    assert!(approx(l.timeline.x, x0) && approx(l.timeline.x + l.timeline.w, x1));
}
