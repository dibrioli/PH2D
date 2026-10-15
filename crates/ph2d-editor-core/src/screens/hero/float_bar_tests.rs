use super::*;

fn area() -> Rect {
    Rect::new(100.0, 50.0, 600.0, 400.0)
}

#[test]
fn a_coluna_separa_os_grupos_e_nao_sobrepoe() {
    let r = column_rects(area(), &[2, 3]);
    assert_eq!(r.len(), 5);
    let step = button_px() + gap();
    assert!(
        (r[1].y - r[0].y - step).abs() < 1e-3,
        "dentro do grupo: um passo"
    );
    assert!(
        (r[2].y - r[1].y - (step + group_sep() - gap())).abs() < 1e-3,
        "entre grupos: o passo mais o separador"
    );
    for w in r.windows(2) {
        assert!(w[1].y >= w[0].y + w[0].h, "botões sobrepostos: {w:?}");
    }
    assert!(
        r.iter().all(|b| b.x >= area().x),
        "a coluna sai da área pela esquerda"
    );
}

#[test]
fn a_grelha_do_flyout_nunca_passa_do_fundo_da_area() {
    let a = area();
    let anchor = Rect::new(a.x + 8.0, a.y + a.h - button_px(), button_px(), button_px());
    let g = grid_rects(a, anchor, 12, 3);
    assert_eq!(g.len(), 12);
    let bottom = g.iter().map(|r| r.y + r.h).fold(0.0_f32, f32::max);
    assert!(
        bottom <= a.y + a.h,
        "a grelha passa do fundo ({bottom} > {})",
        a.y + a.h
    );
    assert!(
        g.iter().all(|r| r.x > anchor.x + anchor.w),
        "a grelha tapa o botão que a abre"
    );
}

#[test]
fn a_barra_da_selecao_sobe_e_desce_conforme_cabe() {
    let a = area();
    let alta = Rect::new(300.0, 300.0, 80.0, 60.0);
    let r = strip_rects(a, alta, &[4, 2], strip_button_px());
    assert!(
        r.iter().all(|b| b.y + b.h <= alta.y),
        "com lugar por cima, fica por cima"
    );
    let no_topo = Rect::new(300.0, a.y + 2.0, 80.0, 60.0);
    let r = strip_rects(a, no_topo, &[4, 2], strip_button_px());
    assert!(
        r.iter().all(|b| b.y >= no_topo.y + no_topo.h),
        "sem lugar por cima, desce para baixo da seleção"
    );
    assert!(
        r.iter().all(|b| b.x >= a.x && b.x + b.w <= a.x + a.w),
        "presa à área"
    );
}

#[test]
fn a_barra_fixada_ignora_a_selecao_e_fica_no_topo() {
    let a = area();
    let r = pinned_strip_rects(a, &[4, 2], button_px());
    assert_eq!(r.len(), 6);
    let top = r.iter().map(|b| b.y).fold(f32::MAX, f32::min);
    assert!(top < a.y + button_px(), "fixada mora no topo da área");
}

#[test]
fn um_grupo_vazio_nao_abre_um_buraco() {
    let a = area();
    let sel = Rect::new(300.0, 300.0, 80.0, 60.0);
    let com = strip_rects(a, sel, &[3, 0, 2], strip_button_px());
    let sem = strip_rects(a, sel, &[3, 2], strip_button_px());
    assert_eq!(com, sem);
}

/// ⭐ Os ícones do módulo desenham 50 % maiores (dono, 08/10) — e o botão e a barra não mexem.
#[test]
fn a_module_icon_draws_half_again_bigger_inside_the_same_button() {
    use crate::paint::icon_glyph_side;
    let r = Rect::new(10.0, 20.0, button_px(), button_px());
    let antes = icon_glyph_side(inner(r).w);
    let depois = icon_glyph_side(module_face_rect(r).w);
    // ⚠️ O número do DONO escrito aqui, e não a constante: comparar com a própria constante
    //    aprovava qualquer valor dela (a mutação `1.0` sobrevivia).
    assert!(
        (depois / antes - 1.5 * 1.15).abs() < 1e-3,
        "o desenho cresceu {:.3}× (o dono pediu +50 % e depois mais 10–20 %: 1,5 × 1,15)",
        depois / antes
    );
    // O DESENHO com o traço cabe no botão (o quadrado pode passar: é o respiro invisível).
    let traco = ph2d_tokens::StrokeToken::Default.px();
    assert!(
        depois + traco <= r.w.min(r.h) + 1e-3,
        "o ícone ({depois:.1} px + traço {traco}) passa do botão de {} px",
        r.w
    );
    assert_eq!(
        column_rects(area(), &[3])[0].w,
        button_px(),
        "o botão não muda de tamanho"
    );
}

/// ⭐ **A margem do enquadramento (F8) livra as DUAS barras flutuantes** — a coluna de ferramentas
/// cabe na faixa da esquerda, e a ④ cabe por CIMA de uma seleção enquadrada encostada à margem de
/// cima (senão ela salta para baixo e tapa a arte). E cai dentro da faixa medida nos oráculos
/// (`oraculo_enquadrar/`: Excalidraw 24 px, tldraw 64 px).
#[test]
fn the_frame_margin_clears_both_floating_bars() {
    let m = frame_margin();
    let a = area();
    let col = panel_of(column_rects(a, &[3, 2])).unwrap();
    assert!(
        col.x + col.w - a.x <= m,
        "a coluna ({}) passa da margem {m}",
        col.x + col.w - a.x
    );
    let sel = Rect::new(a.x + m, a.y + m, 200.0, 120.0);
    let strip = panel_of(strip_rects(a, sel, &[4, 2], button_px())).unwrap();
    assert!(
        strip.y - a.y >= Spacing::Xs.px(),
        "a ④ cola na borda de cima (sem o ar `Xs`): {strip:?}"
    );
    assert!(
        strip.y + strip.h <= sel.y,
        "a ④ não cabe por cima da seleção: {strip:?}"
    );
    assert!((24.0..=64.0).contains(&m), "{m} fora da faixa dos oráculos");
}

/// ⭐ (F11) **A ④ flutua a uma alça inteira mais `Lg` da seleção** (o dono: *«abrem muito próximos
/// das formas»*; eram 12 px) — pela MESMA conta que o pintor ([`tool_bars::placed`]).
#[test]
fn the_action_bar_floats_one_handle_plus_lg_off_the_selection() {
    use crate::interaction::{BarButton, BarFace, BarFamily, ToolBars};
    let gap = crate::HANDLE_SIZE_PX + Spacing::Lg.px();
    assert!((selection_gap() - gap).abs() < 1e-3);
    let b = |id| BarButton {
        id: NodeId(id),
        face: BarFace::Icon(crate::icons::IconId::Select),
        label: String::new(),
        on: false,
        family: BarFamily::Neutral,
    };
    let a = Rect::new(300.0, 80.0, 1200.0, 800.0);
    let mid = Rect::new(a.x + 400.0, a.y + 300.0, 120.0, 90.0);
    let bars = ToolBars {
        column: vec![vec![b(1), b(2), b(3)], vec![b(4)]],
        actions: vec![(100..104).map(b).collect()],
        selection: Some(mid),
        ..ToolBars::default()
    };
    let all = tool_bars::placed(&bars, a, false, false);
    let bar_bottom = all
        .iter()
        .filter(|(btn, _)| (100..104).contains(&btn.id.0))
        .map(|(_, r)| r.y + r.h)
        .fold(f32::MIN, f32::max);
    assert!(
        (mid.y - bar_bottom - gap).abs() < 1e-3,
        "a ④ a {} px da seleção, não {gap}",
        mid.y - bar_bottom
    );
}

/// ⭐ (dono 15/10) **A ④ em duas linhas**: a 2.ª (o Alinhar) fica por BAIXO da 1.ª, centrada nela,
/// e o bloco inteiro pousa à mesma folga da seleção — é a base da 2.ª que fica a `selection_gap`.
/// Fixada, as duas linhas descem juntas do topo da área.
#[test]
fn the_second_row_of_the_action_bar_sits_under_the_first_centred() {
    use crate::interaction::{BarButton, BarFace, BarFamily, ToolBars};
    let b = |id| BarButton {
        id: NodeId(id),
        face: BarFace::Icon(crate::icons::IconId::Select),
        label: String::new(),
        on: false,
        family: BarFamily::Neutral,
    };
    let a = Rect::new(300.0, 80.0, 1200.0, 800.0);
    let mid = Rect::new(a.x + 400.0, a.y + 300.0, 120.0, 90.0);
    let bars = ToolBars {
        actions: vec![(100..105).map(b).collect(), (105..109).map(b).collect()],
        actions_below: vec![(200..203).map(b).collect(), (203..206).map(b).collect()],
        selection: Some(mid),
        ..ToolBars::default()
    };
    for pinned in [false, true] {
        let all = tool_bars::placed(&bars, a, pinned, false);
        let span = |lo: u64, hi: u64| {
            let rs: Vec<Rect> = all
                .iter()
                .filter(|(btn, _)| (lo..hi).contains(&btn.id.0))
                .map(|(_, r)| *r)
                .collect();
            assert_eq!(rs.len() as u64, hi - lo, "faltam botões em {lo}..{hi}");
            let x0 = rs.iter().map(|r| r.x).fold(f32::MAX, f32::min);
            let x1 = rs.iter().map(|r| r.x + r.w).fold(f32::MIN, f32::max);
            let y0 = rs.iter().map(|r| r.y).fold(f32::MAX, f32::min);
            let y1 = rs.iter().map(|r| r.y + r.h).fold(f32::MIN, f32::max);
            (x0, x1, y0, y1)
        };
        // A 1.ª linha leva o chrome (o fixar) no fim: o id dele não é destes.
        let top = span(100, 109);
        let low = span(200, 206);
        assert!(
            (low.2 - top.3 - gap()).abs() < 1e-3,
            "a 2.ª linha a um `gap` da 1.ª, não {}",
            low.2 - top.3
        );
        let chrome_x1 = all
            .iter()
            .filter(|(btn, _)| btn.id == crate::ids::SELECTION_ACTIONS_PIN)
            .map(|(_, r)| r.x + r.w)
            .next()
            .expect("o fixar fecha a 1.ª linha");
        let centre_top = (top.0 + chrome_x1) / 2.0;
        assert!(
            ((low.0 + low.1) / 2.0 - centre_top).abs() < 1e-3,
            "a 2.ª linha centrada na 1.ª"
        );
        if pinned {
            assert!((top.2 - (a.y + Spacing::Md.px())).abs() < 1e-3, "fixada: no topo");
        } else {
            assert!(
                (mid.y - low.3 - selection_gap()).abs() < 1e-3,
                "a base da 2.ª linha a `selection_gap` da seleção, não {}",
                mid.y - low.3
            );
        }
    }
}
