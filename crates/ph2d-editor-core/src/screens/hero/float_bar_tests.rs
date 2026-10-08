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
